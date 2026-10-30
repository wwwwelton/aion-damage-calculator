// Character search tab: name search and profile summary over the native host bridge.
// Port of the search and summary path in characters.js. Dossier, sync and import are not ported yet.

use crate::host;
use crate::i18n::t;
use crate::state::AppState;
use leptos::prelude::*;
use leptos::task::spawn_local;
use serde_json::{json, Value};

const REGIONS: [(&str, &str); 1] = [("ww", "Global")];
const GLOBAL_REGIONS: [(&str, &str); 5] = [
    ("naw", "NA West"),
    ("nae", "NA East"),
    ("eu", "EU"),
    ("as", "Asia"),
    ("la", "SA"),
];

fn num(v: &Value, key: &str) -> String {
    v.get(key).map(|x| x.to_string().trim_matches('"').to_string()).unwrap_or_default()
}

#[component]
pub fn CharacterTab(state: AppState) -> impl IntoView {
    let lang = state.lang;
    let name = RwSignal::new(String::new());
    let region = RwSignal::new("ww".to_string());
    let global_region = RwSignal::new("naw".to_string());
    let status = RwSignal::new(String::new());
    let results = RwSignal::new(Vec::<Value>::new());
    let profile = RwSignal::new(None::<Value>);

    let load_profile = move |item: Value| {
        let server_id = item.get("serverId").cloned().unwrap_or(Value::Null);
        let character_id = item.get("characterId").cloned().unwrap_or(Value::Null);
        let gr = global_region.get();
        status.set(String::new());
        spawn_local(async move {
            let params = json!({ "serverId": server_id, "characterId": character_id });
            match host::request("profile", &region.get(), &gr, params).await {
                Ok(data) => profile.set(Some(data)),
                Err(e) => status.set(e),
            }
        });
    };

    let search = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let query = name.get().trim().to_string();
        if query.is_empty() {
            return;
        }
        let gr = global_region.get();
        status.set(String::new());
        results.set(vec![]);
        profile.set(None);
        spawn_local(async move {
            match host::request("search", &region.get(), &gr, json!({ "name": query })).await {
                Ok(data) => {
                    let list = data.get("results").and_then(Value::as_array).cloned().unwrap_or_default();
                    if list.len() == 1 {
                        results.set(list.clone());
                        // Single match loads straight away, like the JS path.
                        load_profile_now(list[0].clone(), region, global_region, profile, status).await;
                    } else {
                        results.set(list);
                    }
                }
                Err(e) => status.set(e),
            }
        });
    };

    view! {
        <section class="card">
            <h2>{move || t(lang.get(), "findCharTitle")}</h2>
            <p>{move || t(lang.get(), "findCharSubtitle")}</p>
            <form class="search-form" on:submit=search>
                <label>
                    {move || t(lang.get(), "searchCharLabel")}
                    <input type="search" maxlength="20" required
                        placeholder=move || t(lang.get(), "searchCharPlaceholder")
                        on:input=move |ev| name.set(event_target_value(&ev)) />
                </label>
                <label>
                    {move || t(lang.get(), "regionLabel")}
                    <select on:change=move |ev| region.set(event_target_value(&ev))>
                        {REGIONS.iter().map(|(v, l)| view! { <option value=*v>{*l}</option> }).collect_view()}
                    </select>
                </label>
                <label>
                    {move || t(lang.get(), "globalRegionLabel")}
                    <select on:change=move |ev| global_region.set(event_target_value(&ev))>
                        {GLOBAL_REGIONS.iter().map(|(v, l)| view! { <option value=*v>{*l}</option> }).collect_view()}
                    </select>
                </label>
                <button type="submit">{move || t(lang.get(), "searchBtn")}</button>
            </form>
            <p role="status">{move || status.get()}</p>
            <ul class="character-results">
                {move || results.get().into_iter().map(|item| {
                    let label = format!(
                        "{} · {} · {} {}",
                        item.get("name").and_then(Value::as_str).unwrap_or(""),
                        item.get("serverName").and_then(Value::as_str).unwrap_or(""),
                        t(lang.get(), "level"),
                        num(&item, "level"),
                    );
                    let pick = item.clone();
                    view! {
                        <li><button type="button" on:click=move |_| load_profile(pick.clone())>{label}</button></li>
                    }
                }).collect_view()}
            </ul>
        </section>

        <Show when=move || profile.get().is_some() fallback=|| ()>
            {move || profile.get().map(|data| {
                let p = data.get("info").and_then(|i| i.get("profile")).cloned().unwrap_or(Value::Null);
                let title = format!(
                    "{} · {}",
                    p.get("characterName").and_then(Value::as_str).unwrap_or(""),
                    p.get("serverName").and_then(Value::as_str).unwrap_or("")
                );
                let cp = num(&p, "combatPower");
                let meta = format!(
                    "{} · {} {} · {}{}",
                    p.get("className").and_then(Value::as_str).unwrap_or(""),
                    t(lang.get(), "level"),
                    num(&p, "characterLevel"),
                    p.get("raceName").and_then(Value::as_str).unwrap_or(""),
                    if cp.is_empty() { String::new() } else { format!(" · CP {cp}") },
                );
                view! {
                    <section class="card">
                        <h2>{title}</h2>
                        <p>{meta}</p>
                    </section>
                }
            })}
        </Show>
    }
}

// Helper for the single-result path: awaits the same profile request the click handler uses.
async fn load_profile_now(
    item: Value,
    region: RwSignal<String>,
    global_region: RwSignal<String>,
    profile: RwSignal<Option<Value>>,
    status: RwSignal<String>,
) {
    let params = json!({
        "serverId": item.get("serverId").cloned().unwrap_or(Value::Null),
        "characterId": item.get("characterId").cloned().unwrap_or(Value::Null),
    });
    match host::request("profile", &region.get(), &global_region.get(), params).await {
        Ok(data) => profile.set(Some(data)),
        Err(e) => status.set(e),
    }
}
