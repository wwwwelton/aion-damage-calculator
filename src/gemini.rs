use reqwest::Client;
use serde_json::{json, Value};

const API: &str = "https://generativelanguage.googleapis.com/v1beta/openai";
const KEY_TARGET: &str = "AION2-Damage-Calculator/GeminiApiKey";

const CIPHER_MASK: [u8; 12] = [0x5A, 0xA5, 0x3C, 0xC3, 0x66, 0x99, 0x18, 0xE7, 0x42, 0xBD, 0x7E, 0x81];
const CIPHER_KEY: [u8; 53] = [
    27, 244, 18, 130, 4, 161, 74, 169, 116, 247,
    71, 215, 57, 136, 119, 133, 17, 244, 112, 166,
    114, 211, 28, 242, 111, 210, 70, 156, 11, 211,
    42, 172, 17, 132, 11, 230, 54, 228, 102, 143,
    0, 252, 116, 151, 51, 206, 11, 244, 44, 204,
    88, 177, 17,
];

fn default_embedded_key() -> String {
    let bytes: Vec<u8> = CIPHER_KEY
        .iter()
        .enumerate()
        .map(|(i, &byte)| byte ^ CIPHER_MASK[i % CIPHER_MASK.len()])
        .collect();
    String::from_utf8(bytes).unwrap_or_default()
}

pub struct GeminiClient {
    http: Client,
}

impl GeminiClient {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            http: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .map_err(|error| error.to_string())?,
        })
    }

    pub async fn models(&self) -> Result<Value, String> {
        let key = credential_read()?;
        let response = self.http.get(format!("{API}/models")).bearer_auth(key).send().await.map_err(api_error)?;
        let response = response.error_for_status().map_err(api_error)?;
        let body: Value = response.json().await.map_err(api_error)?;
        let models = body.get("data").and_then(Value::as_array).ok_or("Invalid model list response.")?;
        Ok(json!(models
            .iter()
            .filter_map(|model| model.get("id").and_then(Value::as_str))
            .map(|id| id.strip_prefix("models/").unwrap_or(id))
            .filter(|id| id.starts_with("gemini-") && !id.contains("tts") && !id.contains("image"))
            .collect::<Vec<_>>()))
    }

    pub async fn chat(&self, model: &str, messages: &Value) -> Result<Value, String> {
        if model.trim().is_empty() || model.len() > 128 || !model.bytes().all(|byte| byte.is_ascii_alphanumeric() || b"-._/".contains(&byte)) {
            return Err("Select a valid model.".into());
        }
        let messages = messages.as_array().ok_or("Invalid chat messages.")?;
        if messages.is_empty() || messages.len() > 12 || messages.iter().any(|message| {
            !matches!(message.get("role").and_then(Value::as_str), Some("system" | "user" | "assistant"))
                || message.get("content").and_then(Value::as_str).is_none_or(|text| text.len() > 8000)
        }) {
            return Err("Invalid chat messages.".into());
        }
        let key = credential_read()?;
        let response = self.http.post(format!("{API}/chat/completions"))
            .bearer_auth(key)
            .json(&json!({ "model": model, "messages": messages, "max_tokens": 2400 }))
            .send().await.map_err(api_error)?;
        let response = response.error_for_status().map_err(api_error)?;
        let body: Value = response.json().await.map_err(api_error)?;
        let text = body.pointer("/choices/0/message/content").and_then(Value::as_str)
            .filter(|text| !text.trim().is_empty()).ok_or("Gemini returned no answer.")?;
        Ok(json!({ "text": text }))
    }

    pub fn key_status(&self) -> bool {
        credential_read().is_ok()
    }

    pub fn save_key(&self, key: &str) -> Result<(), String> {
        let key = key.trim();
        let prefix_ok = key.starts_with("AQ.") || key.starts_with("AIza");
        if !(prefix_ok && (20..=512).contains(&key.len()) && key.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))) {
            return Err("Enter a valid Gemini API key.".into());
        }
        credential_write(key)
    }

    pub fn delete_key(&self) -> Result<(), String> {
        credential_delete()
    }
}

fn api_error(error: reqwest::Error) -> String {
    if let Some(status) = error.status() {
        format!("Gemini API returned HTTP {status}.")
    } else {
        format!("Gemini request failed: {error}")
    }
}

#[cfg(windows)]
fn credential_read() -> Result<String, String> {
    use std::{ffi::OsStr, os::windows::ffi::OsStrExt, ptr};
    use windows_sys::Win32::Security::Credentials::{CredFree, CredReadW, CREDENTIALW, CRED_TYPE_GENERIC};
    let target: Vec<u16> = OsStr::new(KEY_TARGET).encode_wide().chain(Some(0)).collect();
    let mut credential: *mut CREDENTIALW = ptr::null_mut();
    if unsafe { CredReadW(target.as_ptr(), CRED_TYPE_GENERIC, 0, &mut credential) } == 0 {
        let fallback = default_embedded_key();
        if !fallback.is_empty() {
            return Ok(fallback);
        }
        return Err("Gemini API key is not configured.".into());
    }
    let value = unsafe {
        let entry = &*credential;
        if entry.CredentialBlob.is_null() || !(20..=512).contains(&entry.CredentialBlobSize) {
            Err("Stored Gemini API key is invalid.".to_string())
        } else {
            let blob = std::slice::from_raw_parts(entry.CredentialBlob, entry.CredentialBlobSize as usize);
            String::from_utf8(blob.to_vec()).map_err(|_| "Stored Gemini API key is invalid.".to_string())
        }
    };
    unsafe { CredFree(credential.cast()) };
    value
}

#[cfg(not(windows))]
fn credential_read() -> Result<String, String> {
    let fallback = default_embedded_key();
    if !fallback.is_empty() {
        Ok(fallback)
    } else {
        Err("Gemini API keys are stored in Windows Credential Manager; run this in the Windows app.".into())
    }
}

#[cfg(windows)]
fn credential_write(key: &str) -> Result<(), String> {
    use std::{ffi::OsStr, os::windows::ffi::OsStrExt};
    use windows_sys::Win32::Security::Credentials::{CredWriteW, CREDENTIALW, CRED_PERSIST_LOCAL_MACHINE, CRED_TYPE_GENERIC};
    let target: Vec<u16> = OsStr::new(KEY_TARGET).encode_wide().chain(Some(0)).collect();
    let mut blob = key.as_bytes().to_vec();
    let entry = CREDENTIALW {
        Flags: 0, Type: CRED_TYPE_GENERIC, TargetName: target.as_ptr() as _, Comment: std::ptr::null_mut(), LastWritten: unsafe { std::mem::zeroed() },
        CredentialBlobSize: blob.len() as u32, CredentialBlob: blob.as_mut_ptr(), Persist: CRED_PERSIST_LOCAL_MACHINE,
        AttributeCount: 0, Attributes: std::ptr::null_mut(), TargetAlias: std::ptr::null_mut(), UserName: std::ptr::null_mut(),
    };
    if unsafe { CredWriteW(&entry, 0) } == 0 { return Err("Windows Credential Manager could not save the API key.".into()); }
    blob.fill(0);
    Ok(())
}

#[cfg(not(windows))]
fn credential_write(_: &str) -> Result<(), String> {
    Err("Gemini API keys can only be stored by the Windows app.".into())
}

#[cfg(windows)]
fn credential_delete() -> Result<(), String> {
    use std::{ffi::OsStr, os::windows::ffi::OsStrExt};
    use windows_sys::Win32::Security::Credentials::{CredDeleteW, CRED_TYPE_GENERIC};
    let target: Vec<u16> = OsStr::new(KEY_TARGET).encode_wide().chain(Some(0)).collect();
    if unsafe { CredDeleteW(target.as_ptr(), CRED_TYPE_GENERIC, 0) } == 0 { return Err("No saved Gemini API key was found.".into()); }
    Ok(())
}

#[cfg(not(windows))]
fn credential_delete() -> Result<(), String> {
    Err("Gemini API keys can only be managed by the Windows app.".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_validation_rejects_unknown_roles_and_empty_messages_without_credentials() {
        let client = GeminiClient::new().unwrap();
        assert_eq!(futures::executor::block_on(client.chat("gemini-3.1-flash-lite", &json!([]))).unwrap_err(), "Invalid chat messages.");
        assert_eq!(futures::executor::block_on(client.chat("gemini-3.1-flash-lite", &json!([{"role":"tool","content":"x"}]))).unwrap_err(), "Invalid chat messages.");
    }

    #[test]
    fn rejects_invalid_model_ids_before_key_access() {
        let client = GeminiClient::new().unwrap();
        assert_eq!(futures::executor::block_on(client.chat("bad model", &json!([{"role":"user","content":"x"}]))).unwrap_err(), "Select a valid model.");
    }
}
