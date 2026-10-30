// Public character lookups from the official AION 2 site (aion2.plaync.com), using the same
// search and character APIs as its /en-us/characters pages. Only user-initiated lookups use this client.
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use futures::future::join_all;
use percent_encoding::percent_decode_str;
use reqwest::Url;
use serde_json::{json, Map, Value};
use tokio::sync::Semaphore;

const SEARCH_ENDPOINT: &str = "https://api-search.plaync.com/aion2global/search/v2/character";
const CHARACTER_ENDPOINT: &str = "https://aion2.plaync.com/api/character/";
// Imported stat labels are matched in English, so the official data is always requested in en-US.
const LANGUAGE: &str = "en-US";
const GLOBAL_REGIONS: [&str; 5] = ["as", "naw", "nae", "eu", "la"];
const MAX_CACHE_ENTRIES: usize = 128;
const ITEM_CONCURRENCY: usize = 6;
const SEARCH_TTL: Duration = Duration::from_secs(120);
const PROFILE_TTL: Duration = Duration::from_secs(300);
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AION2-Calculator";

pub struct OfficialClient {
    http: reqwest::Client,
    cache: Mutex<HashMap<String, CacheEntry>>,
    items: Semaphore,
}

struct CacheEntry {
    expires_at: Instant,
    data: Value,
}

impl OfficialClient {
    pub fn new() -> Result<Self, String> {
        let http = reqwest::Client::builder()
            // Fail fast: a healthy official response arrives in a few seconds.
            .timeout(Duration::from_secs(20))
            .user_agent(USER_AGENT)
            .default_headers({
                let mut headers = reqwest::header::HeaderMap::new();
                headers.insert(reqwest::header::ACCEPT, "application/json".parse().map_err(|error| format!("{error}"))?);
                headers.insert(reqwest::header::REFERER, "https://aion2.plaync.com/en-us/characters/index".parse().map_err(|error| format!("{error}"))?);
                headers
            })
            .build()
            .map_err(|error| error.to_string())?;
        Ok(Self {
            http,
            cache: Mutex::new(HashMap::new()),
            items: Semaphore::new(ITEM_CONCURRENCY),
        })
    }

    pub async fn search(&self, name: &str, region: &str, global_region: &str) -> Result<Value, String> {
        if name.trim().is_empty() || name.chars().count() > 20 {
            return Err("Enter a character name (up to 20 characters).".into());
        }
        require_global_region(region, global_region)?;
        let url = Url::parse_with_params(
            SEARCH_ENDPOINT,
            &[("keyword", name.trim()), ("region", global_region), ("localeInfo", LANGUAGE), ("size", "40")],
        )
        .map_err(|error| error.to_string())?;
        let key = url.to_string();
        if let Some(cached) = self.cached(&key) {
            return Ok(cached);
        }
        let response = self.get_json(url).await?;
        let results: Vec<Value> = response
            .get("list")
            .and_then(Value::as_array)
            .map(|list| list.iter().map(|entry| json!({
                "name": strip_tags(text_field(entry, "name")),
                "characterId": text_field(entry, "characterId"),
                "serverId": entry.get("serverId").and_then(Value::as_i64).unwrap_or(0),
                "serverName": text_field(entry, "serverName"),
                "level": entry.get("level").and_then(Value::as_i64).unwrap_or(0),
                "className": class_name_from_pcid(entry.get("pcId").and_then(Value::as_i64).unwrap_or(0)),
                "combatPower": entry.get("combatPower").and_then(Value::as_i64),
                "region": global_region,
            })).collect())
            .unwrap_or_default();
        let data = json!({ "results": results, "complete": true });
        self.store(key, &data, SEARCH_TTL);
        Ok(data)
    }

    pub async fn profile(&self, server_id: i64, character_id: &str, region: &str, global_region: &str) -> Result<Value, String> {
        self.profile_request(server_id, character_id, region, global_region, false).await
    }

    pub async fn refresh(&self, server_id: i64, character_id: &str, region: &str, global_region: &str) -> Result<Value, String> {
        self.profile_request(server_id, character_id, region, global_region, true).await
    }

    async fn profile_request(&self, server_id: i64, character_id: &str, region: &str, global_region: &str, force: bool) -> Result<Value, String> {
        if server_id <= 0 || character_id.trim().is_empty() || character_id.chars().count() > 128 {
            return Err("Select a valid search result.".into());
        }
        require_global_region(region, global_region)?;
        let key = format!("profile|{server_id}|{character_id}|{global_region}");
        if !force {
            if let Some(cached) = self.cached(&key) {
                return Ok(cached);
            }
        }
        // Search results already carry the encoded ID; decode once so it is encoded exactly once below.
        let id = percent_decode_str(character_id).decode_utf8_lossy().into_owned();
        let shared = vec![
            ("region", global_region.to_string()),
            ("lang", LANGUAGE.to_string()),
            ("characterId", id),
            ("serverId", server_id.to_string()),
        ];
        let info_url = character_url("info", &shared, &[])?;
        let equipment_url = character_url("equipment", &shared, &[])?;
        let (info, equipment) = tokio::try_join!(self.get_json(info_url), self.get_json(equipment_url))?;

        let items: Vec<Value> = equipment
            .pointer("/equipment/equipmentList")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let (details, external_ranking) = tokio::join!(
            join_all(items.iter().map(|item| self.item_detail(item, &shared))),
            self.community_ranking(&info, server_id),
        );

        let mut item_details = Map::new();
        let mut complete = true;
        for detail in details {
            match detail {
                Some((slot, value)) => {
                    item_details.insert(slot, value);
                }
                None => complete = false,
            }
        }
        let data = json!({
            "info": info,
            "equipment": equipment,
            "itemDetails": item_details,
            "externalRanking": external_ranking,
            "complete": complete,
        });
        if complete {
            self.store(key, &data, PROFILE_TTL);
        }
        Ok(data)
    }

    // Atool has the useful public combat-power ranking; its failure must not block official profile data.
    async fn community_ranking(&self, info: &Value, server_id: i64) -> Option<Value> {
        let profile = info.get("profile")?;
        let name = text_field(profile, "characterName");
        let server_name = text_field(profile, "serverName");
        let region = server_region(server_id)?;
        let race = match profile.get("raceId").and_then(Value::as_i64)? {
            1 => "천족",
            2 => "마족",
            _ => return None,
        };
        if name.is_empty() || server_name.is_empty() {
            return None;
        }
        let server = format!("{server_name} ({region})");
        let url = Url::parse_with_params(
            "https://aion2tool.com/global/api/character/combat-score-rankings",
            &[("nickname", name), ("server", &server), ("race", race)],
        )
        .ok()?;
        let response = self
            .http
            .get(url)
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .ok()?
            .error_for_status()
            .ok()?;
        let bytes = response.bytes().await.ok()?;
        let payload: Value = serde_json::from_slice(&bytes).ok()?;
        community_ranking_data(payload)
    }

    pub async fn legion_roster(&self, server_name: &str, legion_name: &str, server_id: i64) -> Result<Value, String> {
        if server_name.trim().is_empty() || legion_name.trim().is_empty() {
            return Err("Invalid legion or server.".into());
        }
        let region = server_region(server_id).unwrap_or("SA");
        let server_full = format!("{server_name} ({region})");
        let encoded_server = percent_encoding::utf8_percent_encode(&server_full, percent_encoding::NON_ALPHANUMERIC).to_string();
        let encoded_guild = percent_encoding::utf8_percent_encode(legion_name, percent_encoding::NON_ALPHANUMERIC).to_string();
        let url_str = format!("https://aion2tool.com/global/api/region/{encoded_server}/{encoded_guild}?page=1&per_page=50");
        let url = Url::parse(&url_str).map_err(|e| e.to_string())?;
        let response = self.http.get(url)
            .header("Referer", format!("https://aion2tool.com/global/region/{encoded_server}/{encoded_guild}"))
            .header("Accept", "application/json, text/plain, */*")
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .timeout(Duration::from_secs(8))
            .send()
            .await
            .map_err(|e| format!("Failed to reach legion service: {e}"))?;
        let response = response.error_for_status().map_err(|e| format!("Legion service HTTP error: {e}"))?;
        let data: Value = response.json().await.map_err(|e| format!("Invalid JSON from legion service: {e}"))?;
        Self::checked_legion_roster(data)
    }

    fn checked_legion_roster(data: Value) -> Result<Value, String> {
        if data["success"] != true {
            return Err("Legion not found in public database.".into());
        }
        Ok(data)
    }

    // A failed item detail is reported as a partial profile instead of failing the whole lookup.
    async fn item_detail(&self, item: &Value, shared: &[(&str, String)]) -> Option<(String, Value)> {
        let slot = item.get("slotPos")?.as_i64()?;
        let id = item.get("id")?.as_i64()?;
        let enchant = item.get("enchantLevel")?.as_i64()?;
        let _permit = self.items.acquire().await.ok()?;
        let url = character_url(
            "equipment/item",
            shared,
            &[("id", id.to_string()), ("enchantLevel", enchant.to_string()), ("slotPos", slot.to_string())],
        )
        .ok()?;
        let detail = self.get_json(url).await.ok()?;
        Some((slot.to_string(), detail))
    }


    async fn get_json(&self, url: Url) -> Result<Value, String> {
        let response = self.http.get(url).send().await.map_err(|error| error.to_string())?;
        let response = response.error_for_status().map_err(|error| error.to_string())?;
        let bytes = response.bytes().await.map_err(|error| error.to_string())?;
        serde_json::from_slice(&bytes).map_err(|error| error.to_string())
    }

    fn cached(&self, key: &str) -> Option<Value> {
        let mut cache = self.cache.lock().ok()?;
        match cache.get(key) {
            Some(entry) if entry.expires_at > Instant::now() => Some(entry.data.clone()),
            Some(_) => {
                cache.remove(key);
                None
            }
            None => None,
        }
    }

    fn store(&self, key: String, data: &Value, lifetime: Duration) {
        let Ok(mut cache) = self.cache.lock() else { return };
        if cache.len() >= MAX_CACHE_ENTRIES && !cache.contains_key(&key) {
            let oldest = cache.iter().min_by_key(|(_, entry)| entry.expires_at).map(|(key, _)| key.clone());
            if let Some(oldest) = oldest {
                cache.remove(&oldest);
            }
        }
        cache.insert(key, CacheEntry { expires_at: Instant::now() + lifetime, data: data.clone() });
    }
}

fn server_region(server_id: i64) -> Option<&'static str> {
    match server_id / 100 {
        11 | 21 => Some("NA-E"),
        12 | 22 => Some("NA-W"),
        13 | 23 => Some("EU"),
        14 | 24 => Some("SA"),
        15 | 25 => Some("AS"),
        _ => None,
    }
}

fn community_ranking_data(payload: Value) -> Option<Value> {
    (payload.get("success").and_then(Value::as_bool) == Some(true))
        .then(|| payload.get("data").cloned())
        .flatten()
}

fn character_url(path: &str, shared: &[(&str, String)], extra: &[(&str, String)]) -> Result<Url, String> {
    let pairs: Vec<(&str, &str)> = shared.iter().chain(extra.iter()).map(|(key, value)| (*key, value.as_str())).collect();
    Url::parse_with_params(&format!("{CHARACTER_ENDPOINT}{path}"), &pairs).map_err(|error| error.to_string())
}

fn require_global_region(region: &str, global_region: &str) -> Result<(), String> {
    if region != "ww" || !GLOBAL_REGIONS.contains(&global_region) {
        return Err("Official lookup is available for Global regions only. Select a Global region.".into());
    }
    Ok(())
}

fn text_field<'a>(value: &'a Value, key: &str) -> &'a str {
    value.get(key).and_then(Value::as_str).unwrap_or("")
}

// Search names may contain highlight markup such as <strong>; keep only the text.
fn strip_tags(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    let mut inside_tag = false;
    for character in text.chars() {
        match character {
            '<' => inside_tag = true,
            '>' if inside_tag => inside_tag = false,
            _ if !inside_tag => output.push(character),
            _ => {}
        }
    }
    output
}
fn class_name_from_pcid(pc_id: i64) -> &'static str {
    match pc_id {
        1..=4 => "Warrior",
        5..=8 => "Gladiator",
        9..=12 => "Templar",
        13..=16 => "Ranger",
        17..=20 => "Assassin",
        21..=24 => "Spiritmaster",
        25..=28 => "Sorcerer",
        29..=32 => "Cleric",
        33..=36 => "Chanter",
        _ => "Unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_search_highlight_markup() {
        assert_eq!(strip_tags("<strong>Avatarnoia</strong>"), "Avatarnoia");
        assert_eq!(strip_tags("plain"), "plain");
    }

    #[test]
    fn only_global_regions_are_accepted() {
        assert!(require_global_region("ww", "la").is_ok());
        assert!(require_global_region("kr", "la").is_err());
        assert!(require_global_region("ww", "xx").is_err());
    }

    #[test]
    fn maps_global_servers_and_accepts_only_successful_community_rankings() {
        assert_eq!(server_region(1401), Some("SA"));
        assert_eq!(server_region(1301), Some("EU"));
        assert_eq!(server_region(1201), Some("NA-W"));
        assert_eq!(server_region(9999), None);

        let data = json!({ "overall_rank": 320 });
        assert_eq!(
            community_ranking_data(json!({ "success": true, "data": data.clone() })),
            Some(data)
        );
        assert_eq!(community_ranking_data(json!({ "success": false })), None);
    }

    #[test]
    fn character_id_is_encoded_once() {
        let id = percent_decode_str("9p064LqT7tlY1gPpQykuPS9QTy0nrWUblFDbE5MGhyY%3D").decode_utf8_lossy().into_owned();
        let shared = vec![("characterId", id)];
        let url = character_url("info", &shared, &[]).unwrap();
        assert!(url.query().unwrap().contains("characterId=9p064LqT7tlY1gPpQykuPS9QTy0nrWUblFDbE5MGhyY%3D"));
    }

    #[test]
    fn legion_roster_accepts_only_successful_responses() {
        let fixture = json!({
            "success": true,
            "guild": "GAURUS",
            "members": [{"name": "A"}, {"name": "B"}, {"name": "C"}]
        });
        let data = OfficialClient::checked_legion_roster(fixture.clone()).unwrap();
        assert_eq!(data, fixture);
        assert_eq!(
            OfficialClient::checked_legion_roster(json!({"success": false})).unwrap_err(),
            "Legion not found in public database."
        );
    }
}
