#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]
// The protocol and asset helpers are consumed by the window module, which compiles on Windows only.
#![cfg_attr(not(windows), allow(dead_code))]

mod assets;
mod guides;
mod official;
mod protocol;
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
    let script = assets::lookup("/characters.js").is_some();
    let model = assets::lookup("/assets/ai/models/external/onnx-community/Qwen3-0.6B-ONNX/onnx/model_quantized.onnx_data").is_some();
    let mut report = json!({ "embedded": { "index": pages, "characters_js": script, "model_data": model } });
    match official::OfficialClient::new() {
        Ok(client) => {
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
                    }),
                    Err(error) => json!({ "ok": false, "error": error }),
                };
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
