// Bridge to the native host: post a request with an integer id, resolve when the reply with that id arrives.
// Mirrors request() in characters.js and the reply shape from src/protocol.rs.

use serde_json::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

type Settle = (js_sys::Function, js_sys::Function);

thread_local! {
    static NEXT_ID: RefCell<u64> = const { RefCell::new(0) };
    static PENDING: RefCell<HashMap<String, Settle>> = RefCell::new(HashMap::new());
    static LISTENING: RefCell<bool> = const { RefCell::new(false) };
}

fn host() -> Result<JsValue, String> {
    let window = web_sys::window().ok_or("no window")?;
    let host = js_sys::Reflect::get(&window, &"aion2Host".into()).map_err(|_| "no host")?;
    if host.is_undefined() || host.is_null() {
        return Err("Requires the Windows app.".into());
    }
    Ok(host)
}

// Installs the single reply listener the first time a request is made.
fn ensure_listener(host: &JsValue) -> Result<(), String> {
    if LISTENING.with(|l| *l.borrow()) {
        return Ok(());
    }
    let on_message = js_sys::Reflect::get(host, &"onMessage".into()).map_err(|_| "no onMessage")?;
    let on_message: js_sys::Function = on_message.dyn_into().map_err(|_| "onMessage not a function")?;
    let listener = Closure::<dyn FnMut(JsValue)>::new(|event: JsValue| {
        let data = js_sys::Reflect::get(&event, &"data".into()).unwrap_or(JsValue::UNDEFINED);
        let Some(text) = data.as_string() else { return };
        let Ok(reply) = serde_json::from_str::<Value>(&text) else { return };
        let Some(id) = reply.get("id").map(|v| v.to_string()) else { return };
        let Some((resolve, reject)) = PENDING.with(|p| p.borrow_mut().remove(&id)) else { return };
        if let Some(error) = reply.get("error").and_then(Value::as_str) {
            let _ = reject.call1(&JsValue::NULL, &JsValue::from_str(error));
        } else {
            let data = reply.get("data").cloned().unwrap_or(Value::Null);
            let _ = resolve.call1(&JsValue::NULL, &JsValue::from_str(&data.to_string()));
        }
    });
    on_message
        .call1(host, listener.as_ref())
        .map_err(|_| "onMessage failed")?;
    // The host keeps one listener for the page lifetime.
    listener.forget();
    LISTENING.with(|l| *l.borrow_mut() = true);
    Ok(())
}

// Sends `kind` with `params` merged in, and returns the reply `data` as JSON.
pub async fn request(kind: &str, region: &str, global_region: &str, params: Value) -> Result<Value, String> {
    let host = host()?;
    ensure_listener(&host)?;
    let id = NEXT_ID.with(|n| {
        let mut n = n.borrow_mut();
        *n += 1;
        *n
    });
    let mut message = params;
    let obj = message.as_object_mut().ok_or("params must be an object")?;
    obj.insert("id".into(), Value::from(id));
    obj.insert("kind".into(), Value::from(kind));
    obj.insert("region".into(), Value::from(region));
    obj.insert("globalRegion".into(), Value::from(global_region));

    let key = id.to_string();
    let promise = js_sys::Promise::new(&mut |resolve, reject| {
        PENDING.with(|p| p.borrow_mut().insert(key.clone(), (resolve, reject)));
    });
    let post = js_sys::Reflect::get(&host, &"post".into()).map_err(|_| "no post")?;
    let post: js_sys::Function = post.dyn_into().map_err(|_| "post not a function")?;
    let js_message = js_sys::JSON::parse(&message.to_string()).map_err(|_| "bad json")?;
    post.call1(&host, &js_message).map_err(|_| "post failed")?;

    let reply = JsFuture::from(promise)
        .await
        .map_err(|e| e.as_string().unwrap_or_else(|| "request failed".into()))?;
    let text = reply.as_string().ok_or("bad reply")?;
    serde_json::from_str(&text).map_err(|e| e.to_string())
}
