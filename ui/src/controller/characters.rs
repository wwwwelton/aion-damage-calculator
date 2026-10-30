use wasm_bindgen::prelude::*;
use web_sys::{HtmlElement, HtmlInputElement, HtmlSelectElement};
use serde_json::{json, Value};
use aion2_core::character_extraction::{extract_character, character_hp, enemy_defenses};

pub use crate::controller::character_search::*;

fn document() -> web_sys::Document { web_sys::window().unwrap().document().unwrap() }

pub fn init_characters() {
    init_character_search();
    init_enemy();
}
fn init_enemy() {
    let doc = document();
    if let Some(form) = doc.get_element_by_id("enemy-search").and_then(|e| e.dyn_into::<HtmlElement>().ok()) {
        let on_submit = Closure::<dyn FnMut(web_sys::Event)>::new(move |ev: web_sys::Event| {
            ev.prevent_default();
            let doc = document();
            let query = doc.get_element_by_id("enemy-name").and_then(|e| e.dyn_into::<HtmlInputElement>().ok()).map(|i| i.value().trim().to_string()).unwrap_or_default();
            if query.is_empty() { return; }
            let region = doc.get_element_by_id("enemy-region").and_then(|e| e.dyn_into::<HtmlSelectElement>().ok()).map(|s| s.value()).unwrap_or_else(|| "ww".into());
            let gr = doc.get_element_by_id("enemy-global-region").and_then(|e| e.dyn_into::<HtmlSelectElement>().ok()).map(|s| s.value()).unwrap_or_else(|| "naw".into());
            wasm_bindgen_futures::spawn_local(async move {
                let _doc = document();
                if let Ok(data) = crate::host::request("search", &region, &gr, json!({ "name": query })).await {
                    let list = data.get("results").and_then(Value::as_array).cloned().unwrap_or_default();
                    if list.len() == 1 {
                        let server_id = list[0]["serverId"].as_i64().unwrap_or(0);
                        let char_id = list[0]["characterId"].as_str().unwrap_or("");
                        if let Ok(p_data) = crate::host::request("profile", &region, &gr, json!({ "serverId": server_id, "characterId": char_id })).await {
                            render_enemy_profile(&p_data);
                        }
                    }
                }
            });
        });
        form.set_onsubmit(Some(on_submit.as_ref().unchecked_ref()));
        on_submit.forget();
    }
}

fn render_enemy_profile(data: &Value) {
    let doc = document();
    let view = extract_character(data);
    if let Some(panel) = doc.get_element_by_id("enemy-panel").and_then(|e| e.dyn_into::<HtmlElement>().ok()) {
        panel.set_hidden(false);
    }
    if let Some(title) = doc.get_element_by_id("enemy-title") {
        title.set_text_content(Some(&format!("{} · {}", view.name, view.server)));
    }
    if let Some(meta) = doc.get_element_by_id("enemy-meta") {
        meta.set_text_content(Some(&format!("{} · Nível {} · {}", view.class_name, view.level, view.race)));
    }
    if let Some(btn) = doc.get_element_by_id("import-enemy").and_then(|e| e.dyn_into::<HtmlElement>().ok()) {
        let p_copy = data.clone();
        let on_click = Closure::<dyn FnMut()>::new(move || {
            let doc = document();
            let view = extract_character(&p_copy);
            if let Some(inp) = doc.get_element_by_id("target-name").and_then(|e| e.dyn_into::<HtmlInputElement>().ok()) {
                inp.set_value(&view.name);
            }
            if let Some(sel) = doc.get_element_by_id("target-mode").and_then(|e| e.dyn_into::<HtmlSelectElement>().ok()) {
                sel.set_value("player");
            }
            if let Some(hp) = character_hp(&p_copy) {
                if let Some(inp) = doc.query_selector("input[name=\"enemyHp\"]").ok().flatten().and_then(|n| n.dyn_into::<HtmlInputElement>().ok()) {
                    inp.set_value(&hp.to_string());
                }
            }
            for (k, v) in enemy_defenses(&p_copy) {
                if let Some(inp) = doc.query_selector(&format!("input[name=\"{k}\"]")).ok().flatten().and_then(|n| n.dyn_into::<HtmlInputElement>().ok()) {
                    inp.set_value(&v.to_string());
                }
            }
            if let Some(tab) = doc.get_element_by_id("calculator-tab").and_then(|e| e.dyn_into::<HtmlElement>().ok()) {
                tab.click();
            }
            crate::controller::recalculate();
        });
        btn.set_onclick(Some(on_click.as_ref().unchecked_ref()));
        on_click.forget();
        let _ = btn.remove_attribute("disabled");
    }
}
