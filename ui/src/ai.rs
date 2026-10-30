use crate::{host, i18n::{t, format_t, Lang}, state::AppState};
use leptos::{prelude::*, task::spawn_local};
use serde_json::{json, Value};
use aion2_core::markdown::{parse_markdown, MarkdownNode};

fn markdown(nodes: Vec<MarkdownNode>) -> AnyView {
    nodes.into_iter().map(|node| match node {
        MarkdownNode::Text(text) => text.into_any(),
        MarkdownNode::Element { tag, children } => {
            let children = markdown(children);
            match tag.as_str() {
                "p" => view! { <p>{children}</p> }.into_any(),
                "strong" => view! { <strong>{children}</strong> }.into_any(),
                "em" => view! { <em>{children}</em> }.into_any(),
                "code" => view! { <code>{children}</code> }.into_any(),
                "pre" => view! { <pre>{children}</pre> }.into_any(),
                "ul" => view! { <ul>{children}</ul> }.into_any(),
                "ol" => view! { <ol>{children}</ol> }.into_any(),
                "li" => view! { <li>{children}</li> }.into_any(),
                "h2" => view! { <h2>{children}</h2> }.into_any(),
                "h3" => view! { <h3>{children}</h3> }.into_any(),
                "h4" => view! { <h4>{children}</h4> }.into_any(),
                "table" => view! { <table>{children}</table> }.into_any(),
                "thead" => view! { <thead>{children}</thead> }.into_any(),
                "tbody" => view! { <tbody>{children}</tbody> }.into_any(),
                "tr" => view! { <tr>{children}</tr> }.into_any(),
                "th" => view! { <th>{children}</th> }.into_any(),
                "td" => view! { <td>{children}</td> }.into_any(),
                _ => children,
            }
        }
    }).collect_view().into_any()
}

#[component]
pub fn AiPanel(state: AppState) -> impl IntoView {
    let settings = RwSignal::new(false);
    let status = RwSignal::new(String::new());
    let key = RwSignal::new(String::new());
    let models = RwSignal::new(Vec::<String>::new());
    let scenario = Memo::new(move |_| {
        let raw = state.inputs.get_value().iter().map(|(name, signal)| {
            (name.to_string(), aion2_core::numeric_input::parse_numeric_input(&signal.get(), f64::NEG_INFINITY).unwrap_or(0.0))
        }).collect();
        crate::calculator_form::build_scenario(&raw, state.target_mode.get(), state.creature_type.get().as_deref(), state.critical.get(), state.double.get(), state.position.get()).values
    });
    let storage = web_sys::window().and_then(|w| w.local_storage().ok().flatten());
    let selected = storage.as_ref().and_then(|s| s.get_item("geminiModel").ok().flatten()).unwrap_or_else(|| "gemini-3.1-flash-lite".into());
    let model = RwSignal::new(selected);
    let question = RwSignal::new(String::new());
    let history = RwSignal::new(Vec::<Value>::new());
    let messages = RwSignal::new(Vec::<(String, String)>::new());
    let busy = RwSignal::new(false);
    let source = RwSignal::new(String::new());
    let configure = move |_| {
        settings.set(!settings.get_untracked());
        key.set(String::new());
        if !settings.get_untracked() { return; }
        status.set(t(state.lang.get_untracked(), "aiCheckingSettings"));
        spawn_local(async move {
            let result = async {
                let configured = host::request("ai_key_status", "", "", json!({})).await?;
                let list = host::request("ai_models", "", "", json!({})).await?;
                let list: Vec<String> = serde_json::from_value(list).map_err(|e| e.to_string())?;
                if !list.contains(&model.get_untracked()) { model.set(list.first().cloned().unwrap_or_default()); }
                models.set(list);
                Ok::<_, String>(configured["configured"].as_bool().unwrap_or(false))
            }.await;
            status.set(match result { Ok(configured) => t(state.lang.get_untracked(), if configured { "aiKeyConfigured" } else { "aiKeyMissing" }), Err(e) => e });
        });
    };
    let save = move |_| spawn_local(async move {
        let secret = key.get_untracked();
        key.set(String::new());
        let result = host::request("ai_key_save", "", "", json!({"key":secret})).await;
        status.set(match result { Ok(_) => t(state.lang.get_untracked(), "aiKeyConfigured"), Err(e) => e });
    });
    let remove = move |_| spawn_local(async move {
        key.set(String::new());
        let result = host::request("ai_key_delete", "", "", json!({})).await;
        status.set(match result { Ok(_) => t(state.lang.get_untracked(), "aiKeyMissing"), Err(e) => e });
    });
    let send = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        let q = question.get_untracked().trim().to_owned();
        if q.is_empty() || busy.get_untracked() { return; }
        question.set(String::new());
        busy.set(true);
        messages.update(|m| m.push(("user".into(), q.clone())));
        let inputs = state.inputs.get_value().iter().map(|(name, signal)| format!("{name}: {}", signal.get_untracked())).collect::<Vec<_>>().join(", ");
        let values = scenario.get_untracked();
        let damage = aion2_core::calculator::calculate_damage(&values);
        let upgrades = aion2_core::recommendations::rank_damage_upgrades(&values, state.critical.get_untracked(), state.position.get_untracked());
        let context = format!("Character: {}; class: {}; target: {}; mode: {:?}; type: {:?}; stats: {}; scenario: {:?}; calculated hit: {}; +1% upgrades: {:?}", state.attacker_name.get_untracked(), state.attacker_class.get_untracked(), state.target_name.get_untracked(), state.target_mode.get_untracked(), state.creature_type.get_untracked(), inputs, values, damage, upgrades);
        let language = if state.lang.get_untracked() == Lang::Pt { "Portuguese" } else { "English" };
        spawn_local(async move {
            let lower = q.to_lowercase();
            let topic = if ["creature", "species", "cogni", "fera", "natura", "varian"].iter().any(|s| lower.contains(s)) { "creature" }
                else if ["critical", "block", "evasion", "chance", "resist", "double"].iter().any(|s| lower.contains(s)) { "chance" }
                else if matches!(state.target_mode.get_untracked(), aion2_core::calculator::TargetMode::Player) { "pvp" }
                else if ["attack", "weapon", "penetration"].iter().any(|s| lower.contains(s)) { "attack" } else { "pve" };
            let guide = match host::request("research", "", "", json!({"topic":topic})).await {
                Ok(data) => {
                    source.set(format_t(state.lang.get_untracked(), "aiGuide", &[("url", data["url"].as_str().unwrap_or(""))]));
                    data["excerpt"].as_str().unwrap_or("").chars().take(280).collect::<String>()
                }
                Err(e) => { source.set(format_t(state.lang.get_untracked(), "aiGuideUnavailable", &[("error", &e)])); String::new() }
            };
            let mut request = vec![json!({"role":"system", "content":format!("You are an AION 2 combat analyst. Answer in {language}. Distinguish Combat Power, per-hit damage, estimated sustained DPS and measured rotation DPS. Do not invent unverified mechanics or unavailable data; state missing inputs. Calculator: {context}\nGuide: {guide}")})];
            request.extend(history.get_untracked().into_iter().rev().take(6).collect::<Vec<_>>().into_iter().rev());
            request.push(json!({"role":"user","content":q}));
            match host::request("ai_chat", "", "", json!({"model":model.get_untracked(),"messages":request})).await {
                Ok(data) => {
                    let answer = data["text"].as_str().unwrap_or("").to_owned();
                    history.update(|h| { h.push(json!({"role":"user","content":q})); h.push(json!({"role":"assistant","content":answer})); if h.len()>8 { h.drain(..h.len()-8); } });
                    messages.update(|m| m.push(("assistant".into(), answer)));
                }
                Err(e) => messages.update(|m| m.push(("assistant".into(), format_t(state.lang.get_untracked(), "aiUnavailable", &[("error", &e)])))),
            }
            busy.set(false);
        });
    };
    view! {
        <section class="card ai-panel">
            <h2>{move || t(state.lang.get(), "aiChatHeader")}</h2>
            <p>{move || t(state.lang.get(), "aiPrivacyNote")}</p>
            <button type="button" on:click=configure>{move || t(state.lang.get(), "aiConfigure")}</button>
            <section hidden=move || !settings.get()>
                <h3>{move || t(state.lang.get(), "aiSettingsTitle")}</h3>
                <label>{move || t(state.lang.get(), "aiKeyLabel")}<input type="password" autocomplete="off" prop:value=move || key.get() on:input=move |ev| key.set(event_target_value(&ev)) /></label>
                <button type="button" on:click=save>{move || t(state.lang.get(), "aiSaveKey")}</button>
                <button type="button" on:click=remove>{move || t(state.lang.get(), "aiRemoveKey")}</button>
                <label>{move || t(state.lang.get(), "aiModelLabel")}<select prop:value=move || model.get() on:change=move |ev| {
                    let value = event_target_value(&ev);
                    if let Some(storage) = &storage { if let Err(_) = storage.set_item("geminiModel", &value) { status.set("Could not save model preference.".into()); } }
                    model.set(value);
                }>{move || models.get().into_iter().map(|id| {
                    let value = id.clone();
                    view! { <option value=value>{id}</option> }
                }).collect_view()}</select></label>
                <p role="status">{move || status.get()}</p>
            </section>
            <div id="ai-chat-dialog" aria-live="polite">{move || messages.get().into_iter().map(|(role, text)| {
                let author = if role == "user" { t(state.lang.get(), "aiYou") } else { "Gemini".into() };
                view! { <div class=format!("ai-msg ai-msg-{role}")><strong>{author}</strong>{markdown(parse_markdown(&text))}</div> }
            }).collect_view()}</div>
            <p hidden=move || !busy.get()>{move || t(state.lang.get(), "aiGenerating")}</p>
            <p>{move || source.get()}</p>
            {move || aion2_core::recommendations::rank_damage_upgrades(&scenario.get(), state.critical.get(), state.position.get()).first().map(|upgrade| {
                use aion2_core::recommendations::Stat;
                let field = match upgrade.stat {
                    Stat::WeaponDamageBoost => "weaponDamageBoost",
                    Stat::DamageBoost => "damageBoost",
                    Stat::CriticalDamageBoost => "criticalDamageBoost",
                    Stat::FrontBackDamageBoost => if matches!(state.position.get(), aion2_core::calculator::Position::Back) { "backBoost" } else { "frontBoost" },
                };
                let gain = format!("{:.2}%", upgrade.percent);
                let label = format_t(state.lang.get(), "aiApplyDynamic", &[("label", upgrade.label), ("gain", &gain)]);
                view! { <button type="button" on:click=move |_| {
                    state.inputs.with_value(|inputs| {
                        if let Some((_, signal)) = inputs.iter().find(|(name, _)| *name == field) {
                            if let Some(value) = aion2_core::numeric_input::parse_numeric_input(&signal.get_untracked(), 0.0) {
                                signal.set((value + 1.0).to_string());
                            }
                        }
                    });
                }>{label}</button> }
            })}
            <form on:submit=send><input aria-label=move || t(state.lang.get(), "aiQuestionPlaceholder") placeholder=move || t(state.lang.get(), "aiQuestionPlaceholder") prop:value=move || question.get() on:input=move |ev| question.set(event_target_value(&ev)) />
                <button type="submit" disabled=move || busy.get()>{move || t(state.lang.get(), "aiAskBtn")}</button>
            </form>
        </section>
    }
}
