#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]
// The protocol and asset helpers are consumed by the window module, which compiles on Windows only.
#![cfg_attr(not(windows), allow(dead_code))]

mod assets;
mod dns;
mod gemini;
mod guides;
mod official;
mod protocol;
mod profile_cache;
mod profile_sync;
#[cfg(windows)]
mod window;

use std::path::Path;

use serde_json::{json, Value};

fn main() {
    let runtime = tokio::runtime::Runtime::new().expect("tokio runtime starts");
    // Headless check used to verify the embedded assets and the official lookup on Windows
    // without opening a window: AION2_SELFTEST=<file> writes a JSON report to that file.
    if let Some(report) = std::env::var_os("AION2_SELFTEST") {
        let outcome = runtime.block_on(self_test());
        let _ = std::fs::write(Path::new(&report), outcome.to_string());
        return;
    }
    #[cfg(windows)]
    window::run(runtime);
    #[cfg(not(windows))]
    {
        let _ = runtime;
        eprintln!("The calculator window runs on Windows only.");
        std::process::exit(1);
    }
}

async fn self_test() -> Value {
    let pages = assets::lookup("/").is_some();
    let ui_js = assets::lookup("/aion2_ui.js").is_some();
    let ui_wasm = assets::lookup("/aion2_ui_bg.wasm").is_some();
    let mut report = json!({ "embedded": { "index": pages, "ui_js": ui_js, "ui_wasm": ui_wasm } });
    match official::OfficialClient::new() {
        Ok(client) => {
            let client = std::sync::Arc::new(client);
            let search = client.search("Avatarnoia", "ww", "la").await;
            let first = search.as_ref().ok().and_then(|data| data["results"].as_array()).and_then(|list| list.first().cloned());
            report["search"] = match &search {
                Ok(data) => json!({ "ok": true, "count": data["results"].as_array().map(Vec::len).unwrap_or(0) }),
                Err(error) => json!({ "ok": false, "error": error }),
            };
            if let Some(entry) = first {
                let profile = client
                    .profile(entry["serverId"].as_i64().unwrap_or(0), entry["characterId"].as_str().unwrap_or(""), "ww", "la")
                    .await;
                report["profile"] = match &profile {
                    Ok(data) => json!({
                        "ok": true,
                        "name": data["info"]["profile"]["characterName"],
                        "gearDetails": data["itemDetails"].as_object().map(|map| map.len()).unwrap_or(0),
                        "complete": data["complete"],
                        "externalRanking": data["externalRanking"],
                    }),
                    Err(error) => json!({ "ok": false, "error": error }),
                };
                if let Ok(data) = &profile {
                    report["profileCache"] = cache_self_test(client.clone(), &entry, data).await;
                }
            }
        }
        Err(error) => report["client"] = json!({ "ok": false, "error": error }),
    }
    let guide = guides::GuideClient::new();
    report["guide"] = match guide {
        Ok(client) => match client.fetch("attack").await {
            Ok(data) => json!({ "ok": true, "excerptChars": data["excerpt"].as_str().map(|text| text.chars().count()).unwrap_or(0) }),
            Err(error) => json!({ "ok": false, "error": error }),
        },
        Err(error) => json!({ "ok": false, "error": error }),
    };
    report
}

async fn cache_self_test(client: std::sync::Arc<official::OfficialClient>, entry: &Value, data: &Value) -> Value {
    let directory = std::env::temp_dir().join(format!("aion2-smoke-{}", std::process::id()));
    let path = directory.join("profiles.json");
    let server = entry["serverId"].as_i64().unwrap_or(0);
    let character = entry["characterId"].as_str().unwrap_or("");
    let result = async {
        let cache = profile_cache::ProfileCache::open(path.clone())?;
        cache.save(server, character, "la", data)?;
        let restarted = profile_cache::ProfileCache::open(path)?;
        let persisted = restarted.get(server, character, "la").ok_or("Saved profile missing after restart")?;
        let sync = profile_sync::ProfileSync::new(restarted, client)?;
        let replies = std::sync::Mutex::new(Vec::new());
        let request = json!({"id": 1, "kind":"profile", "serverId":server, "characterId":character, "region":"ww", "globalRegion":"la"});
        sync.respond(&request.to_string(), |reply| replies.lock().unwrap().push(reply)).await;
        let replies: Vec<Value> = replies.into_inner().unwrap().iter().filter_map(|reply| serde_json::from_str(reply).ok()).collect();
        Ok::<_, String>(json!({
            "persisted": persisted["profile"]["info"]["profile"]["characterName"],
            "cachedReplyFirst": replies.first().is_some_and(|reply| reply["data"]["cacheMeta"]["cached"] == true),
            "backgroundRefresh": replies.last().is_some_and(|reply| reply["type"] == "profile_sync" && reply["data"]["cacheMeta"]["cached"] == false),
            "events": replies.len(),
        }))
    }.await;
    let _ = std::fs::remove_dir_all(directory);
    match result { Ok(report) => report, Err(error) => json!({"error":error}) }
}
