use reqwest::Client;
use serde_json::{json, Value};

const API: &str = "https://api.aionlabs.ai/v1";
const KEY_TARGET: &str = "AION2-Damage-Calculator/AionLabsApiKey";

pub struct AionLabsClient {
    http: Client,
}

impl AionLabsClient {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            http: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .map_err(|error| error.to_string())?,
        })
    }

    pub async fn models(&self) -> Result<Value, String> {
        let response = self.http.get(format!("{API}/models")).send().await.map_err(api_error)?;
        let response = response.error_for_status().map_err(api_error)?;
        let body: Value = response.json().await.map_err(api_error)?;
        let models = body.get("data").and_then(Value::as_array).ok_or("Invalid model list response.")?;
        Ok(json!(models.iter().filter_map(|model| model.get("id").and_then(Value::as_str)).collect::<Vec<_>>()))
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
            .json(&json!({ "model": model, "messages": messages, "max_tokens": 2400, "reasoning_effort": "low" }))
            .send().await.map_err(api_error)?;
        let response = response.error_for_status().map_err(api_error)?;
        let body: Value = response.json().await.map_err(api_error)?;
        let text = body.pointer("/choices/0/message/content").and_then(Value::as_str)
            .filter(|text| !text.trim().is_empty()).ok_or("AionLabs returned no answer.")?;
        Ok(json!({ "text": text }))
    }

    pub fn key_status(&self) -> bool {
        credential_read().is_ok()
    }

    pub fn save_key(&self, key: &str) -> Result<(), String> {
        let key = key.trim();
        if !(key.starts_with("alv2_") && (20..=512).contains(&key.len()) && key.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')) {
            return Err("Enter a valid AionLabs API key.".into());
        }
        credential_write(key)
    }

    pub fn delete_key(&self) -> Result<(), String> {
        credential_delete()
    }
}

fn api_error(error: reqwest::Error) -> String {
    if let Some(status) = error.status() {
        format!("AionLabs API returned HTTP {status}.")
    } else {
        format!("AionLabs request failed: {error}")
    }
}

#[cfg(windows)]
fn credential_read() -> Result<String, String> {
    use std::{ffi::OsStr, os::windows::ffi::OsStrExt, ptr};
    use windows_sys::Win32::Security::Credentials::{CredFree, CredReadW, CREDENTIALW, CRED_TYPE_GENERIC};
    let target: Vec<u16> = OsStr::new(KEY_TARGET).encode_wide().chain(Some(0)).collect();
    let mut credential: *mut CREDENTIALW = ptr::null_mut();
    if unsafe { CredReadW(target.as_ptr(), CRED_TYPE_GENERIC, 0, &mut credential) } == 0 {
        return Err("AionLabs API key is not configured.".into());
    }
    let value = unsafe {
        let entry = &*credential;
        if entry.CredentialBlob.is_null() || !(20..=512).contains(&entry.CredentialBlobSize) {
            Err("Stored AionLabs API key is invalid.".to_string())
        } else {
            let blob = std::slice::from_raw_parts(entry.CredentialBlob, entry.CredentialBlobSize as usize);
            String::from_utf8(blob.to_vec()).map_err(|_| "Stored AionLabs API key is invalid.".to_string())
        }
    };
    unsafe { CredFree(credential.cast()) };
    value
}

#[cfg(not(windows))]
fn credential_read() -> Result<String, String> {
    Err("AionLabs API keys are stored in Windows Credential Manager; run this in the Windows app.".into())
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
    Err("AionLabs API keys can only be stored by the Windows app.".into())
}

#[cfg(windows)]
fn credential_delete() -> Result<(), String> {
    use std::{ffi::OsStr, os::windows::ffi::OsStrExt};
    use windows_sys::Win32::Security::Credentials::{CredDeleteW, CRED_TYPE_GENERIC};
    let target: Vec<u16> = OsStr::new(KEY_TARGET).encode_wide().chain(Some(0)).collect();
    if unsafe { CredDeleteW(target.as_ptr(), CRED_TYPE_GENERIC, 0) } == 0 { return Err("No saved AionLabs API key was found.".into()); }
    Ok(())
}

#[cfg(not(windows))]
fn credential_delete() -> Result<(), String> {
    Err("AionLabs API keys can only be managed by the Windows app.".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_validation_rejects_unknown_roles_and_empty_messages_without_credentials() {
        let client = AionLabsClient::new().unwrap();
        assert_eq!(futures::executor::block_on(client.chat("aion-labs/aion-3.5-mini", &json!([]))).unwrap_err(), "Invalid chat messages.");
        assert_eq!(futures::executor::block_on(client.chat("aion-labs/aion-3.5-mini", &json!([{"role":"tool","content":"x"}]))).unwrap_err(), "Invalid chat messages.");
    }

    #[test]
    fn rejects_invalid_model_ids_before_key_access() {
        let client = AionLabsClient::new().unwrap();
        assert_eq!(futures::executor::block_on(client.chat("bad model", &json!([{"role":"user","content":"x"}]))).unwrap_err(), "Select a valid model.");
    }
}
