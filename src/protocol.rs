// Routes messages from the page to the native services and builds the reply the page expects.
// Each reply carries the request id, so the page can match it to the waiting request.
use serde_json::{json, Value};

use crate::gemini::GeminiClient;
use crate::guides::GuideClient;
use crate::official::OfficialClient;

pub const APP_ORIGIN_HOST: &str = "aion2.localhost";

// Returns the JSON reply for a page message, or None when the message has no usable id.
pub async fn respond(message: &str, official: &OfficialClient, guides: &GuideClient, ai: &GeminiClient) -> Option<String> {
    let request: Value = serde_json::from_str(message).ok()?;
    let id = request.get("id").and_then(Value::as_i64)?;
    let outcome = match request.get("kind").and_then(Value::as_str) {
        Some("search") => {
            let name = text(&request, "name");
            official.search(name, text(&request, "region"), text(&request, "globalRegion")).await
        }
        Some("profile") => match (request.get("serverId").and_then(Value::as_i64), request.get("characterId").and_then(Value::as_str)) {
            (Some(server_id), Some(character_id)) => {
                official.profile(server_id, character_id, text(&request, "region"), text(&request, "globalRegion")).await
            }
            _ => Err("Select a valid search result.".to_string()),
        },
        Some("research") => guides.fetch(text(&request, "topic")).await,
        Some("legion_roster") => {
            let server_name = text(&request, "serverName");
            let legion_name = text(&request, "legionName");
            let server_id = request.get("serverId").and_then(Value::as_i64).unwrap_or(0);
            official.legion_roster(server_name, legion_name, server_id).await
        }
        Some("ai_models") => ai.models().await,
        Some("ai_key_status") => Ok(json!({ "configured": ai.key_status() })),
        Some("ai_key_save") => ai.save_key(text(&request, "key")).map(|()| json!({ "configured": true })),
        Some("ai_key_delete") => ai.delete_key().map(|()| json!({ "configured": false })),
        Some("ai_chat") => ai.chat(text(&request, "model"), request.get("messages").unwrap_or(&Value::Null)).await,
        _ => Err("Unknown character request.".to_string()),
    };
    let reply = match outcome {
        Ok(data) => json!({ "id": id, "data": data }),
        Err(error) => json!({ "id": id, "error": error }),
    };
    Some(reply.to_string())
}

fn text<'a>(request: &'a Value, key: &str) -> &'a str {
    request.get(key).and_then(Value::as_str).unwrap_or("")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn unknown_kinds_reply_with_the_request_id() {
        let official = OfficialClient::new().unwrap();
        let guides = GuideClient::new().unwrap();
        let ai = GeminiClient::new().unwrap();
        let reply = respond(r#"{"id":7,"kind":"nope"}"#, &official, &guides, &ai).await.unwrap();
        let value: Value = serde_json::from_str(&reply).unwrap();
        assert_eq!(value["id"], 7);
        assert_eq!(value["error"], "Unknown character request.");
    }

    #[tokio::test]
    async fn messages_without_an_id_get_no_reply() {
        let official = OfficialClient::new().unwrap();
        let guides = GuideClient::new().unwrap();
        let ai = GeminiClient::new().unwrap();
        assert!(respond(r#"{"kind":"search"}"#, &official, &guides, &ai).await.is_none());
        assert!(respond("not json", &official, &guides, &ai).await.is_none());
    }

    #[tokio::test]
    async fn non_global_regions_are_rejected_before_any_network_call() {
        let official = OfficialClient::new().unwrap();
        let guides = GuideClient::new().unwrap();
        let ai = GeminiClient::new().unwrap();
        let reply = respond(r#"{"id":1,"kind":"search","name":"Kanon","region":"kr","globalRegion":""}"#, &official, &guides, &ai)
            .await
            .unwrap();
        let value: Value = serde_json::from_str(&reply).unwrap();
        assert!(value["error"].as_str().unwrap().contains("Global regions only"));
    }
}
