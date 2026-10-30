use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use serde_json::{json, Value};

pub struct ProfileCache {
    pub(crate) path: PathBuf,
    entries: Mutex<Vec<Value>>,
}

impl ProfileCache {
    pub fn open(path: PathBuf) -> Result<Self, String> {
        let entries = match std::fs::read(&path) {
            Ok(bytes) => serde_json::from_slice(&bytes).map_err(|error| format!("Cannot read saved profiles: {error}"))?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Vec::new(),
            Err(error) => return Err(format!("Cannot read saved profiles: {error}")),
        };
        Ok(Self { path, entries: Mutex::new(entries) })
    }

    pub fn all(&self) -> Result<Vec<Value>, String> {
        self.entries.lock().map(|entries| entries.clone()).map_err(|_| "Profile cache is unavailable.".into())
    }

    pub fn get(&self, server: i64, character: &str, region: &str) -> Option<Value> {
        self.entries.lock().ok()?.iter().find(|entry| entry["serverId"] == server && entry["characterId"] == character && entry["globalRegion"] == region).cloned()
    }

    pub fn save(&self, server: i64, character: &str, region: &str, profile: &Value) -> Result<Value, String> {
        let mut entries = self.entries.lock().map_err(|_| "Profile cache is unavailable.")?;
        let mut updated = entries.clone();
        updated.retain(|entry| !(entry["serverId"] == server && entry["characterId"] == character && entry["globalRegion"] == region));
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).map_err(|error| error.to_string())?.as_secs();
        let entry = json!({ "serverId": server, "characterId": character, "globalRegion": region, "fetchedAt": timestamp, "profile": profile });
        updated.push(entry.clone());
        if updated.len() > 20 { updated.remove(0); }
        let parent = self.path.parent().ok_or("Invalid profile cache path.")?;
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        let temporary = self.path.with_extension("tmp");
        std::fs::write(&temporary, serde_json::to_vec(&updated).map_err(|error| error.to_string())?).map_err(|error| error.to_string())?;
        std::fs::rename(&temporary, &self.path).map_err(|error| error.to_string())?;
        *entries = updated;
        Ok(entry)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saves_profiles_across_restarts_and_preserves_region_identity() {
        let directory = std::env::temp_dir().join(format!("aion2-cache-{}-{}", std::process::id(), SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()));
        let path = directory.join("profiles.json");
        let cache = ProfileCache::open(path.clone()).unwrap();
        cache.save(1401, "hero", "la", &json!({"name":"first"})).unwrap();
        cache.save(1401, "hero", "eu", &json!({"name":"other"})).unwrap();
        cache.save(1401, "hero", "la", &json!({"name":"updated"})).unwrap();
        let restarted = ProfileCache::open(path).unwrap();
        assert_eq!(restarted.all().unwrap().len(), 2);
        assert_eq!(restarted.get(1401, "hero", "la").unwrap()["profile"]["name"], "updated");
        assert_eq!(restarted.get(1401, "hero", "eu").unwrap()["profile"]["name"], "other");
        std::fs::remove_dir_all(directory).unwrap();
    }
}
