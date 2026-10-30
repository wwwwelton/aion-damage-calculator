use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Document, Element, HtmlElement, HtmlInputElement, HtmlSelectElement, Window};
use serde_json::{json, Value};
use aion2_core::markdown::{parse_markdown, MarkdownNode};
use aion2_core::numeric_input::parse_numeric_input;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::LazyLock;

static I18N_RAW: LazyLock<Value> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../../data/i18n.json")).expect("i18n.json")
});

thread_local! {
    static CONVERSATION_HISTORY: RefCell<Vec<Value>> = const { RefCell::new(Vec::new()) };
    static GUIDE_CACHE: RefCell<HashMap<String, (String, String)>> = RefCell::new(HashMap::new());
}

fn window() -> Window {
    web_sys::window().expect("window")
}

fn document() -> Document {
    window().document().expect("document")
}

fn current_lang() -> String {
    if let Some(storage) = window().local_storage().ok().flatten() {
        if let Ok(Some(saved)) = storage.get_item("aion2_calculator_lang") {
            if saved == "en" || saved == "pt" {
                return saved;
            }
        }
    }
    "pt".to_string()
}

fn t(lang: &str, key: &str) -> String {
    let map = I18N_RAW.get(lang).and_then(Value::as_object);
    if let Some(val) = map.and_then(|m| m.get(key)).and_then(Value::as_str) {
        return val.to_string();
    }
    if let Some(val) = I18N_RAW["pt"].get(key).and_then(Value::as_str) {
        return val.to_string();
    }
    key.to_string()
}

fn format_t(lang: &str, key: &str, replacements: &[(&str, &str)]) -> String {
    let mut text = t(lang, key);
    for (name, val) in replacements {
        text = text.replace(&format!("{{{name}}}"), val);
    }
    text
}

fn scroll_chat_to_bottom(chat: &Element) {
    chat.set_scroll_top(chat.scroll_height());
}

fn append_chat_message(doc: &Document, chat: &Element, role: &str, author: &str, text: &str) -> Option<Element> {
    let msg = doc.create_element("div").ok()?;
    let _ = msg.set_attribute("class", &format!("ai-msg ai-msg-{role}"));

    let author_span = doc.create_element("span").ok()?;
    let _ = author_span.set_attribute("class", "ai-msg-author");
    author_span.set_text_content(Some(author));

    let body = doc.create_element("div").ok()?;
    let _ = body.set_attribute("class", "ai-msg-body");
    body.set_text_content(Some(text));

    let _ = msg.append_child(&author_span);
    let _ = msg.append_child(&body);
    let _ = chat.append_child(&msg);

    scroll_chat_to_bottom(chat);
    Some(body)
}

fn node_to_dom(doc: &Document, node: &MarkdownNode) -> web_sys::Node {
    match node {
        MarkdownNode::Text(text) => {
            let t = doc.create_text_node(text);
            t.unchecked_into::<web_sys::Node>()
        }
        MarkdownNode::Element { tag, children } => {
            let el = doc.create_element(tag).unwrap_or_else(|_| doc.create_element("span").unwrap());
            for child in children {
                let child_node = node_to_dom(doc, child);
                let _ = el.append_child(&child_node);
            }
            el.unchecked_into::<web_sys::Node>()
        }
    }
}

fn render_markdown_into(doc: &Document, target: &Element, source: &str) {
    target.set_inner_html("");
    let nodes = parse_markdown(source);
    for node in &nodes {
        let dom_node = node_to_dom(doc, node);
        let _ = target.append_child(&dom_node);
    }
}

fn guide_topic(doc: &Document, question: &str) -> &'static str {
    let lower = question.to_lowercase();
    if ["creature", "species", "cogni", "fera", "natura", "varian"]
        .iter()
        .any(|s| lower.contains(s))
    {
        return "creature";
    }
    if ["critical", "block", "evasion", "chance", "resist", "double"]
        .iter()
        .any(|s| lower.contains(s))
    {
        return "chance";
    }
    let mode = doc.get_element_by_id("target-mode")
        .and_then(|e| e.dyn_into::<HtmlSelectElement>().ok())
        .map(|s| s.value())
        .unwrap_or_default();
    if mode == "player" {
        return "pvp";
    }
    if ["attack", "weapon", "penetration"]
        .iter()
        .any(|s| lower.contains(s))
    {
        return "attack";
    }
    "pve"
}

fn current_ai_context(doc: &Document) -> String {
    let attacker = doc.get_element_by_id("attacker-name")
        .and_then(|e| e.dyn_into::<HtmlInputElement>().ok())
        .map(|i| i.value())
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| "Player".to_string());

    let a_class = doc.get_element_by_id("attacker-class")
        .and_then(|e| e.dyn_into::<HtmlInputElement>().ok())
        .map(|i| i.value())
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| "Unspecified class".to_string());

    let target = doc.get_element_by_id("target-name")
        .and_then(|e| e.dyn_into::<HtmlInputElement>().ok())
        .map(|i| i.value())
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| "Target".to_string());

    let mode = doc.get_element_by_id("target-mode")
        .and_then(|e| e.dyn_into::<HtmlSelectElement>().ok())
        .map(|s| s.value())
        .unwrap_or_else(|| "boss".to_string());

    let c_type = doc.get_element_by_id("creature-type-header")
        .and_then(|e| e.dyn_into::<HtmlSelectElement>().ok())
        .map(|s| s.value())
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| "Unknown".to_string());

    let target_class = if mode == "player" {
        doc.get_element_by_id("target-class")
            .and_then(|e| e.dyn_into::<HtmlInputElement>().ok())
            .map(|i| i.value())
            .filter(|v| !v.trim().is_empty())
            .unwrap_or_else(|| "Unspecified class".to_string())
    } else {
        String::new()
    };

    let mut character_stats = Vec::new();
    let mut enemy_stats = Vec::new();

    if let Ok(inputs) = doc.query_selector_all("#calculator input[name]") {
        for idx in 0..inputs.length() {
            let Some(node) = inputs.item(idx) else { continue };
            let Some(input) = node.dyn_into::<HtmlInputElement>().ok() else { continue };

            if input.closest("[hidden]").ok().flatten().is_some() {
                continue;
            }
            if mode == "player" && input.closest(".pve-only, .boss-only").ok().flatten().is_some() {
                continue;
            }
            if mode == "monster" && input.closest(".boss-only, .player-only").ok().flatten().is_some() {
                continue;
            }
            if mode == "boss" && input.closest(".player-only").ok().flatten().is_some() {
                continue;
            }

            let val = input.value();
            if val.is_empty() || val == "0" || val == input.default_value() {
                continue;
            }

            let is_enemy = input.closest("label.enemy").ok().flatten().is_some();
            let entry = format!("{}: {}", input.name(), val);
            if is_enemy {
                enemy_stats.push(entry);
            } else {
                character_stats.push(entry);
            }
        }
    }

    let upgrades = if let Ok(list) = doc.query_selector_all("#upgrade-list li") {
        let mut u_vec = Vec::new();
        for i in 0..list.length().min(3) {
            if let Some(item) = list.item(i).and_then(|n| n.text_content()) {
                let trimmed = item.trim().to_string();
                if !trimmed.is_empty() {
                    u_vec.push(trimmed);
                }
            }
        }
        if u_vec.is_empty() { "None".to_string() } else { u_vec.join(" | ") }
    } else {
        "None".to_string()
    };

    let assumptions = if let Ok(list) = doc.query_selector_all("#estimate-assumptions li") {
        let mut a_vec = Vec::new();
        for i in 0..list.length() {
            if let Some(item) = list.item(i).and_then(|n| n.text_content()) {
                a_vec.push(item);
            }
        }
        a_vec.join(" ")
    } else {
        String::new()
    };

    let result = doc.get_element_by_id("result")
        .and_then(|e| e.text_content())
        .unwrap_or_default();

    let dps = doc.get_element_by_id("estimated-dps")
        .and_then(|e| e.text_content())
        .unwrap_or_default();

    let target_class_suffix = if !target_class.is_empty() {
        format!("; class: {target_class}")
    } else {
        String::new()
    };

    format!(
        "Character: {attacker}; class: {a_class}\n\
         Target: {target}; mode: {mode}; type: {c_type}{target_class_suffix}\n\
         Character stats: {}\n\
         Target stats: {}\n\
         Calculated hit: {result}; estimated DPS: {dps}\n\
         Calculated +1% upgrades: {upgrades}\n\
         Estimate assumptions: {assumptions}",
        character_stats.join(", "),
        enemy_stats.join(", ")
    )
}

struct RecommendedUpgrade {
    field: &'static str,
    label: String,
    gain: String,
}

fn recommended_upgrade(doc: &Document) -> Option<RecommendedUpgrade> {
    let first = doc.query_selector("#upgrade-list li").ok().flatten()?.text_content().unwrap_or_default();
    let mut parts = first.splitn(2, ':');
    let label = parts.next()?.trim();
    let gain = parts.next()?.trim();
    let field = match label {
        "Weapon Damage Boost" | "Aumento de Dano da Arma" => "weaponDamageBoost",
        "Damage Boost" | "Aumento de Dano" => "damageBoost",
        "Critical Damage Boost" | "Aumento de Dano Crítico" => "criticalDamageBoost",
        "Front Attack Damage Boost" | "Aumento de Dano Frontal" => "frontBoost",
        "Back Attack Damage Boost" | "Aumento de Dano Pelas Costas" => "backBoost",
        _ => return None,
    };
    let selector = format!("#calculator input[name=\"{}\"]", field);
    if doc.query_selector(&selector).ok().flatten().is_some() {
        Some(RecommendedUpgrade {
            field,
            label: label.to_string(),
            gain: gain.to_string(),
        })
    } else {
        None
    }
}

pub fn update_apply_button() {
    let doc = document();
    let Some(btn) = doc.get_element_by_id("ai-apply").and_then(|e| e.dyn_into::<HtmlElement>().ok()) else { return };
    let suggestion = recommended_upgrade(&doc);
    btn.set_hidden(suggestion.is_none());
    if let Some(s) = suggestion {
        let lang = current_lang();
        btn.set_text_content(Some(&format_t(&lang, "aiApplyDynamic", &[("label", &s.label), ("gain", &s.gain)])));
    }
}

pub fn init_ai() {
    let doc = document();

    // Configure button -> showModal
    if let (Some(btn), Some(_dlg)) = (
        doc.get_element_by_id("ai-configure").and_then(|e| e.dyn_into::<HtmlElement>().ok()),
        doc.get_element_by_id("ai-settings"),
    ) {
        let on_click = Closure::<dyn FnMut()>::new(move || {
            let doc = document();
            if let Some(dlg) = doc.get_element_by_id("ai-settings") {
                let _ = js_sys::Reflect::get(&dlg, &"showModal".into())
                    .ok()
                    .and_then(|f| f.dyn_into::<js_sys::Function>().ok())
                    .map(|f| f.call0(&dlg));
            }
            let lang = current_lang();
            if let Some(st) = doc.get_element_by_id("ai-key-status") {
                st.set_text_content(Some(&t(&lang, "aiCheckingSettings")));
            }
            wasm_bindgen_futures::spawn_local(async move {
                let doc = document();
                let lang = current_lang();

                let status_res = crate::host::request("ai_key_status", "", "", json!({})).await;
                let models_res = crate::host::request("ai_models", "", "", json!({})).await;

                match (status_res, models_res) {
                    (Ok(status), Ok(models)) => {
                        let configured = status.get("configured").and_then(|v| v.as_bool()).unwrap_or(false);
                        if let Some(st) = doc.get_element_by_id("ai-key-status") {
                            st.set_text_content(Some(&t(&lang, if configured { "aiKeyConfigured" } else { "aiKeyMissing" })));
                        }
                        if let Some(sel) = doc.get_element_by_id("ai-model").and_then(|e| e.dyn_into::<HtmlSelectElement>().ok()) {
                            sel.set_inner_html("");
                            let selected = window()
                                .local_storage()
                                .ok()
                                .flatten()
                                .and_then(|s| s.get_item("geminiModel").ok().flatten())
                                .unwrap_or_else(|| "gemini-3.1-flash-lite".to_string());

                            let mut model_list = Vec::new();
                            if let Some(arr) = models.as_array() {
                                for m in arr {
                                    if let Some(name) = m.as_str() {
                                        model_list.push(name.to_string());
                                        let opt = doc.create_element("option").unwrap();
                                        let _ = opt.set_attribute("value", name);
                                        opt.set_text_content(Some(name));
                                        let _ = sel.append_child(&opt);
                                    }
                                }
                            }
                            if model_list.contains(&selected) {
                                sel.set_value(&selected);
                            } else if let Some(first) = model_list.first() {
                                sel.set_value(first);
                            }
                            sel.set_disabled(model_list.is_empty());
                        }
                    }
                    (Err(err), _) | (_, Err(err)) => {
                        if let Some(st) = doc.get_element_by_id("ai-key-status") {
                            st.set_text_content(Some(&format_t(&lang, "aiUnavailable", &[("error", &err)])));
                        }
                    }
                }
            });
        });
        btn.set_onclick(Some(on_click.as_ref().unchecked_ref()));
        on_click.forget();
    }

    // Save key
    if let Some(btn) = doc.get_element_by_id("ai-save-key").and_then(|e| e.dyn_into::<HtmlElement>().ok()) {
        let on_click = Closure::<dyn FnMut()>::new(move || {
            let doc = document();
            let key = doc.get_element_by_id("ai-api-key")
                .and_then(|e| e.dyn_into::<HtmlInputElement>().ok())
                .map(|i| i.value())
                .unwrap_or_default();
            wasm_bindgen_futures::spawn_local(async move {
                let doc = document();
                let lang = current_lang();
                match crate::host::request("ai_key_save", "", "", json!({ "key": key })).await {
                    Ok(_) => {
                        if let Some(field) = doc.get_element_by_id("ai-api-key").and_then(|e| e.dyn_into::<HtmlInputElement>().ok()) {
                            field.set_value("");
                        }
                        if let Some(st) = doc.get_element_by_id("ai-key-status") {
                            st.set_text_content(Some(&t(&lang, "aiKeyConfigured")));
                        }
                    }
                    Err(err) => {
                        if let Some(st) = doc.get_element_by_id("ai-key-status") {
                            st.set_text_content(Some(&err));
                        }
                    }
                }
            });
        });
        btn.set_onclick(Some(on_click.as_ref().unchecked_ref()));
        on_click.forget();
    }

    // Remove key
    if let Some(btn) = doc.get_element_by_id("ai-remove-key").and_then(|e| e.dyn_into::<HtmlElement>().ok()) {
        let on_click = Closure::<dyn FnMut()>::new(move || {
            wasm_bindgen_futures::spawn_local(async move {
                let doc = document();
                let lang = current_lang();
                match crate::host::request("ai_key_delete", "", "", json!({})).await {
                    Ok(_) => {
                        if let Some(st) = doc.get_element_by_id("ai-key-status") {
                            st.set_text_content(Some(&t(&lang, "aiKeyMissing")));
                        }
                    }
                    Err(err) => {
                        if let Some(st) = doc.get_element_by_id("ai-key-status") {
                            st.set_text_content(Some(&err));
                        }
                    }
                }
            });
        });
        btn.set_onclick(Some(on_click.as_ref().unchecked_ref()));
        on_click.forget();
    }

    // Settings dialog close -> clear input
    if let Some(dlg) = doc.get_element_by_id("ai-settings") {
        let on_close = Closure::<dyn FnMut()>::new(move || {
            let doc = document();
            if let Some(inp) = doc.get_element_by_id("ai-api-key").and_then(|e| e.dyn_into::<HtmlInputElement>().ok()) {
                inp.set_value("");
            }
        });
        let _ = dlg.add_event_listener_with_callback("close", on_close.as_ref().unchecked_ref());
        on_close.forget();
    }

    // Model select change -> localStorage
    if let Some(sel) = doc.get_element_by_id("ai-model").and_then(|e| e.dyn_into::<HtmlSelectElement>().ok()) {
        let on_change = Closure::<dyn FnMut()>::new(move || {
            let doc = document();
            if let Some(s) = doc.get_element_by_id("ai-model").and_then(|e| e.dyn_into::<HtmlSelectElement>().ok()) {
                let val = s.value();
                if let Some(storage) = window().local_storage().ok().flatten() {
                    let _ = storage.set_item("geminiModel", &val);
                }
            }
        });
        sel.set_onchange(Some(on_change.as_ref().unchecked_ref()));
        on_change.forget();
    }

    // Send question click
    if let Some(btn) = doc.get_element_by_id("ai-send").and_then(|e| e.dyn_into::<HtmlElement>().ok()) {
        let on_click = Closure::<dyn FnMut()>::new(move || {
            send_chat_message();
        });
        btn.set_onclick(Some(on_click.as_ref().unchecked_ref()));
        on_click.forget();
    }

    // Question input keydown (Enter)
    if let Some(input) = doc.get_element_by_id("ai-question").and_then(|e| e.dyn_into::<HtmlElement>().ok()) {
        let on_keydown = Closure::<dyn FnMut(JsValue)>::new(move |event: JsValue| {
            if let Ok(key) = js_sys::Reflect::get(&event, &"key".into()) {
                if key.as_string().as_deref() == Some("Enter") {
                    if let Ok(prevent_default) = js_sys::Reflect::get(&event, &"preventDefault".into()) {
                        if let Ok(func) = prevent_default.dyn_into::<js_sys::Function>() {
                            let _ = func.call0(&event);
                        }
                    }
                    send_chat_message();
                }
            }
        });
        let _ = input.add_event_listener_with_callback("keydown", on_keydown.as_ref().unchecked_ref());
        on_keydown.forget();
    }

    // Apply button click
    if let Some(btn) = doc.get_element_by_id("ai-apply").and_then(|e| e.dyn_into::<HtmlElement>().ok()) {
        let on_click = Closure::<dyn FnMut()>::new(move || {
            let doc = document();
            let Some(suggestion) = recommended_upgrade(&doc) else { return };
            let selector = format!("#calculator input[name=\"{}\"]", suggestion.field);
            let Some(input) = doc.query_selector(&selector).ok().flatten().and_then(|e| e.dyn_into::<HtmlInputElement>().ok()) else { return };
            let val = parse_numeric_input(&input.value(), f64::NEG_INFINITY).unwrap_or(0.0);
            input.set_value(&(val + 1.0).to_string());
            let init = web_sys::CustomEventInit::new();
            init.set_bubbles(true);
            if let Ok(event) = web_sys::CustomEvent::new_with_event_init_dict("input", &init) {
                let _ = input.dispatch_event(&event);
            }
            update_apply_button();
        });
        btn.set_onclick(Some(on_click.as_ref().unchecked_ref()));
        on_click.forget();
    }

    // Inputs/selects change -> update apply button
    for selector in [
        "#attacker-name",
        "#attacker-class",
        "#target-name",
        "#target-class",
        "#target-mode",
        "#creature-type-header",
        "#critical",
        "#position",
    ] {
        if let Some(el) = doc.query_selector(selector).ok().flatten() {
            let on_input = Closure::<dyn FnMut()>::new(|| {
                update_apply_button();
            });
            let _ = el.add_event_listener_with_callback("input", on_input.as_ref().unchecked_ref());
            let _ = el.add_event_listener_with_callback("change", on_input.as_ref().unchecked_ref());
            on_input.forget();
        }
    }

    update_apply_button();
}

fn send_chat_message() {
    let doc = document();
    let Some(input) = doc.get_element_by_id("ai-question").and_then(|e| e.dyn_into::<HtmlInputElement>().ok()) else { return };
    let question = input.value().trim().to_string();
    if question.is_empty() { return; }

    let Some(send_btn) = doc.get_element_by_id("ai-send").and_then(|e| e.dyn_into::<HtmlElement>().ok()) else { return };
    if send_btn.get_attribute("disabled").is_some() { return; }

    input.set_value("");

    let Some(chat) = doc.get_element_by_id("ai-chat-dialog") else { return };
    let lang = current_lang();

    let _ = append_chat_message(&doc, &chat, "user", &t(&lang, "aiYou"), &question);
    let Some(asst_body) = append_chat_message(&doc, &chat, "assistant", "Gemini", &t(&lang, "aiGenerating")) else { return };

    let _ = send_btn.set_attribute("disabled", "");
    if let Some(source_el) = doc.get_element_by_id("ai-source") {
        source_el.set_text_content(Some(""));
    }

    wasm_bindgen_futures::spawn_local(async move {
        dispatch_ai_request(question, asst_body).await;
    });
}

async fn dispatch_ai_request(question: String, asst_body: Element) {
    let doc = document();
    let lang = current_lang();
    let topic = guide_topic(&doc, &question);

    let cached_guide = GUIDE_CACHE.with(|cache| cache.borrow().get(topic).cloned());
    let guide_result = match cached_guide {
        Some(entry) => Ok(entry),
        None => match crate::host::request("research", "", "", json!({ "topic": topic })).await {
            Ok(data) => {
                let excerpt = data.get("excerpt").and_then(|v| v.as_str()).unwrap_or("").chars().take(280).collect::<String>();
                let url = data.get("url").and_then(|v| v.as_str()).unwrap_or("").to_string();
                GUIDE_CACHE.with(|cache| cache.borrow_mut().insert(topic.to_string(), (excerpt.clone(), url.clone())));
                Ok((excerpt, url))
            }
            Err(e) => Err(e),
        },
    };

    let guide_excerpt = match &guide_result {
        Ok((excerpt, url)) => {
            if let Some(source_el) = doc.get_element_by_id("ai-source") {
                source_el.set_text_content(Some(&format_t(&lang, "aiGuide", &[("url", url)])));
            }
            if excerpt.is_empty() { "none".to_string() } else { excerpt.clone() }
        }
        Err(e) => {
            if let Some(source_el) = doc.get_element_by_id("ai-source") {
                source_el.set_text_content(Some(&format_t(&lang, "aiGuideUnavailable", &[("error", e)])));
            }
            "none".to_string()
        }
    };

    let model = window()
        .local_storage()
        .ok()
        .flatten()
        .and_then(|s| s.get_item("geminiModel").ok().flatten())
        .filter(|m| !m.trim().is_empty())
        .unwrap_or_else(|| "gemini-3.1-flash-lite".to_string());

    let context = current_ai_context(&doc);
    let language = if lang == "pt" { "Portuguese" } else { "English" };

    let system_prompt = format!(
        "You are the expert combat analyst for AION 2. All responses are generated directly by this remote model without any local inference. Deliver authoritative, high-level, mathematically sound tactical advice in {language}. Explain exact formula mechanics, stat priorities, marginal gains, and combat matchups clearly and concisely. Clearly differentiate between raw Combat Power (CP), per-hit damage, sustained estimated DPS, and actual measured rotation DPS. Never hallucinate unavailable data or invent unverified game mechanics; explicitly state which stats or rotation logs are required when data is incomplete. Calculator data:\n{context}\nOnline guide:\n{guide_excerpt}"
    );

    let mut messages = vec![json!({ "role": "system", "content": system_prompt })];

    let history_slice = CONVERSATION_HISTORY.with(|hist| {
        let h = hist.borrow();
        let len = h.len();
        let start = if len > 6 { len - 6 } else { 0 };
        h[start..].to_vec()
    });
    messages.extend(history_slice);
    messages.push(json!({ "role": "user", "content": question }));

    match crate::host::request("ai_chat", "", "", json!({ "model": model, "messages": messages })).await {
        Ok(res) => {
            let text = res.get("text").and_then(|t| t.as_str()).unwrap_or("");
            render_markdown_into(&doc, &asst_body, text);
            CONVERSATION_HISTORY.with(|hist| {
                let mut h = hist.borrow_mut();
                h.push(json!({ "role": "user", "content": question }));
                h.push(json!({ "role": "assistant", "content": text }));
                if h.len() > 8 {
                    let excess = h.len() - 8;
                    h.drain(..excess);
                }
            });
        }
        Err(err) => {
            asst_body.set_text_content(Some(&format_t(&lang, "aiUnavailable", &[("error", &err)])));
        }
    }

    if let Some(send_btn) = doc.get_element_by_id("ai-send") {
        let _ = send_btn.remove_attribute("disabled");
    }

    if let Some(chat) = doc.get_element_by_id("ai-chat-dialog") {
        scroll_chat_to_bottom(&chat);
    }
}
