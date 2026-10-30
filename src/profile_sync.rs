use std::sync::Arc;
use serde_json::{json, Value};
use crate::{official::OfficialClient, profile_cache::ProfileCache};

pub struct ProfileSync {
    pub cache: ProfileCache,
    pub official: Arc<OfficialClient>,
    active: tokio::sync::Mutex<()>,
    settings: std::sync::Mutex<Value>,
    settings_path: std::path::PathBuf,
}

impl ProfileSync {
    pub fn new(cache: ProfileCache, official: Arc<OfficialClient>) -> Result<Self, String> {
        let settings_path = cache.path.with_file_name("sync-settings.json");
        let settings = match std::fs::read(&settings_path) {
            Ok(bytes) => serde_json::from_slice(&bytes).map_err(|error| format!("Cannot read sync settings: {error}"))?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => json!({"enabled":true,"interval":600}),
            Err(error) => return Err(error.to_string()),
        };
        validate_settings(&settings)?;
        Ok(Self { cache, official, active: tokio::sync::Mutex::new(()), settings: std::sync::Mutex::new(settings), settings_path })
    }

    pub fn settings(&self) -> Result<Value, String> {
        self.settings.lock().map(|settings| settings.clone()).map_err(|_| "Sync settings unavailable.".into())
    }

    fn configure(&self, request: &Value) -> Result<Value, String> {
        if request.get("enabled").is_none() { return self.settings(); }
        let next = json!({"enabled": request["enabled"], "interval": request["interval"]});
        validate_settings(&next)?;
        let mut settings = self.settings.lock().map_err(|_| "Sync settings unavailable.")?;
        std::fs::create_dir_all(self.settings_path.parent().ok_or("Invalid settings path.")?).map_err(|error| error.to_string())?;
        let temporary = self.settings_path.with_extension("tmp");
        std::fs::write(&temporary, next.to_string()).map_err(|error| error.to_string())?;
        std::fs::rename(temporary, &self.settings_path).map_err(|error| error.to_string())?;
        *settings = next.clone();
        Ok(next)
    }

    pub async fn refresh(&self, server: i64, character: &str, region: &str) -> Result<Value, String> {
        let _guard = self.active.lock().await;
        let profile = self.official.refresh(server, character, "ww", region).await?;
        let entry = self.cache.save(server, character, region, &profile)?;
        Ok(entry)
    }

    pub async fn respond(&self, message: &str, send: impl Fn(String)) -> bool {
        let Ok(request) = serde_json::from_str::<Value>(message) else { return false };
        let Some(id) = request["id"].as_i64() else { return false };
        match request["kind"].as_str() {
            Some("sync_settings") => {
                let reply = match self.configure(&request) {
                    Ok(settings) => json!({"id":id,"data":settings}),
                    Err(error) => json!({"id":id,"error":error}),
                };
                send(reply.to_string());
                true
            }
            Some("saved_profiles") => {
                let reply = match self.cache.all() {
                    Ok(entries) => json!({"id": id, "data": entries}),
                    Err(error) => json!({"id": id, "error": error}),
                };
                send(reply.to_string());
                true
            }
            Some("profile" | "refresh_profile") => {
                let Some(server) = request["serverId"].as_i64().filter(|server| *server > 0) else { return false };
                let Some(character) = request["characterId"].as_str().filter(|value| !value.is_empty() && value.len() <= 384) else { return false };
                let Some(region) = request["globalRegion"].as_str().filter(|value| ["la", "eu", "nae", "naw", "as"].contains(value)) else { return false };
                if request["region"] != "ww" { return false; }
                let cached = self.cache.get(server, character, region);
                if let Some(entry) = &cached {
                    let mut profile = entry["profile"].clone();
                    profile["cacheMeta"] = json!({"fetchedAt": entry["fetchedAt"], "cached": true});
                    send(json!({"id": id, "data": profile}).to_string());
                }
                if cached.is_some() && request["kind"] != "refresh_profile" && self.settings().is_ok_and(|settings| settings["enabled"] == false) {
                    return true;
                }
                let result = self.refresh(server, character, region).await;
                match result {
                    Ok(entry) => {
                        let mut profile = entry["profile"].clone();
                        profile["cacheMeta"] = json!({"fetchedAt": entry["fetchedAt"], "cached": false});
                        if cached.is_some() {
                            send(json!({"type":"profile_sync", "serverId":server, "characterId":character, "globalRegion":region, "data":profile}).to_string());
                        } else {
                            send(json!({"id":id, "data":profile}).to_string());
                        }
                    }
                    Err(error) if cached.is_some() => send(json!({"type":"profile_sync", "serverId":server, "characterId":character, "globalRegion":region, "error":error}).to_string()),
                    Err(error) => send(json!({"id":id,"error":error}).to_string()),
                }
                true
            }
            _ => false,
        }
    }

    pub async fn refresh_all(&self, send: impl Fn(String)) {
        let Ok(entries) = self.cache.all() else { return };
        for entry in entries {
            let Some(server) = entry["serverId"].as_i64() else { continue };
            let Some(character) = entry["characterId"].as_str() else { continue };
            let Some(region) = entry["globalRegion"].as_str() else { continue };
            let mut event = json!({"type":"profile_sync", "serverId":server,"characterId":character,"globalRegion":region});
            match self.refresh(server, character, region).await {
                Ok(updated) => {
                    event["data"] = updated["profile"].clone();
                    event["data"]["cacheMeta"] = json!({"fetchedAt":updated["fetchedAt"],"cached":false});
                }
                Err(error) => event["error"] = json!(error),
            }
            send(event.to_string());
        }
    }
}

fn validate_settings(settings: &Value) -> Result<(), String> {
    if settings["enabled"].as_bool().is_none() || !settings["interval"].as_u64().is_some_and(|seconds| (600..=3600).contains(&seconds)) {
        return Err("Sync interval must be between 600 and 3600 seconds.".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn disabled_sync_serves_saved_profiles_without_a_network_request_and_survives_restart() {
        let directory = std::env::temp_dir().join(format!("aion2-sync-settings-{}-{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let path = directory.join("profiles.json");
        let official = Arc::new(OfficialClient::new().unwrap());
        let cache = ProfileCache::open(path.clone()).unwrap();
        cache.save(1401, "saved-character", "la", &json!({"complete":true,"info":{"profile":{"characterName":"Saved"}}})).unwrap();
        let sync = ProfileSync::new(cache, official.clone()).unwrap();
        sync.configure(&json!({"enabled":false,"interval":900})).unwrap();
        assert!(sync.configure(&json!({"enabled":true,"interval":599})).is_err());
        assert_eq!(sync.settings().unwrap()["enabled"], false);
        let restarted = ProfileSync::new(ProfileCache::open(path).unwrap(), official).unwrap();
        assert_eq!(restarted.settings().unwrap(), json!({"enabled":false,"interval":900}));
        let replies = std::sync::Mutex::new(Vec::new());
        let handled = restarted.respond(r#"{"id":1,"kind":"profile","serverId":1401,"characterId":"saved-character","region":"ww","globalRegion":"la"}"#, |reply| replies.lock().unwrap().push(reply)).await;
        assert!(handled);
        let replies = replies.into_inner().unwrap();
        assert_eq!(replies.len(), 1);
        let first: Value = serde_json::from_str(&replies[0]).unwrap();
        assert_eq!(first["data"]["info"]["profile"]["characterName"], "Saved");
        assert_eq!(first["data"]["cacheMeta"]["cached"], true);
        std::fs::remove_dir_all(directory).unwrap();
    }
}
