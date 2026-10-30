// Enemy character tab: search by name and list the character-sheet defenses and HP.
// Port of the enemy path in characters.js. Equipment/typed-effect defenses and import are not ported yet.

use crate::host;
use crate::i18n::{format_t, t};
use crate::state::AppState;
use leptos::prelude::*;
use leptos::task::spawn_local;
use serde_json::{json, Value};
use aion2_core::calculator::TargetMode;
use aion2_core::character_extraction::{character_hp, enemy_defenses, ENEMY_FIELDS};

const GLOBAL_REGIONS: [(&str, &str); 5] = [
    ("naw", "NA West"),
    ("nae", "NA East"),
    ("eu", "EU"),
    ("as", "Asia"),
    ("la", "SA"),
];

// Sheet stat name -> calculator field, and whether the field is a percentage.
const SHEET_FIELDS: [(&str, &str, bool); 20] = [
    ("Defense", "defense", false),
    ("Weapon Damage Tolerance", "weaponDamageTolerance", true),
    ("Damage Tolerance", "damageTolerance", true),
    ("PvE Damage Tolerance", "pveDamageTolerance", true),
    ("Species Damage Tolerance", "speciesDamageTolerance", true),
    ("Boss Damage Tolerance", "bossDamageTolerance", true),
    ("Critical Damage Tolerance", "criticalDamageTolerance", true),
    ("Front Attack Damage Tolerance", "frontDamageTolerance", true),
    ("Back Attack Damage Tolerance", "backDamageTolerance", true),
    ("Block", "block", false),
    ("Evasion", "evasion", false),
    ("Critical Hit Resist", "criticalHitResist", false),
    ("Double Resist", "doubleResist", true),
    ("Perfect Resist", "perfectResist", true),
    ("Multi-hit Resist", "multiHitResist", true),
    ("Endurance", "endurance", true),
    ("Elemental Resistance", "elementalResistance", false),
    ("Critical Damage Defense", "criticalDamageDefense", false),
    ("Front Defense", "frontDefense", false),
    ("Back Defense", "backDefense", false),
];

fn stat_value(profile: &Value, name: &str) -> Option<String> {
    profile
        .get("info")?
        .get("stat")?
        .get("statList")?
        .as_array()?
        .iter()
        .find(|s| s.get("name").and_then(Value::as_str) == Some(name))
        .and_then(|s| s.get("value"))
        .map(|v| v.to_string().trim_matches('"').trim().to_string())
}

// Mirrors characters.js sheetFields: percent fields take "12" or "12%", others plain numbers only.
fn sheet_defenses(profile: &Value) -> Vec<(&'static str, &'static str, f64)> {
    SHEET_FIELDS
        .iter()
        .filter_map(|(label, field, percent)| {
            let text = stat_value(profile, label)?;
            if !*percent && text.ends_with('%') {
                return None;
            }
            let value: f64 = text.strip_suffix('%').unwrap_or(&text).parse().ok()?;
            Some((*field, *label, value))
        })
        .collect()
}

async fn fetch_profile_now(
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

fn import_into_calculator(state: AppState, data: &Value) -> String {
    let lang = state.lang.get_untracked();
    let defenses = enemy_defenses(data);
    let hp = character_hp(data);
    let name = data
        .get("info")
        .and_then(|i| i.get("profile"))
        .and_then(|p| p.get("characterName"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let inputs = state.inputs.get_value();
    let set = |field: &str, text: String| {
        if let Some((_, sig)) = inputs.iter().find(|(f, _)| *f == field) {
            sig.set(text);
        }
    };
    for (_, field, _) in ENEMY_FIELDS {
        set(field, "0".to_string());
    }
    set("enemyHp", "0".to_string());
    for (field, value) in &defenses {
        set(field, value.to_string());
    }
    if let Some(hp) = hp {
        set("enemyHp", hp.to_string());
    }
    state.target_mode.set(TargetMode::Player);
    state.target_name.set(name.clone());
    state.tab.set(crate::state::ActiveTab::Calculator);
    let hp_text = match hp {
        Some(hp) => format_t(lang, "enemyHpKnown", &[("hp", hp.to_string().as_str())]),
        None => t(lang, "enemyHpUnknown"),
    };
    let count = defenses.len().to_string();
    format_t(lang, "enemyImportResult", &[("count", count.as_str()), ("name", name.as_str()), ("hp", hp_text.as_str())])
}

#[component]
pub fn EnemyTab(state: AppState) -> impl IntoView {
    let lang = state.lang;
    let name = RwSignal::new(String::new());
    let region = RwSignal::new("ww".to_string());
    let global_region = RwSignal::new("naw".to_string());
    let status = RwSignal::new(String::new());
    let import_status = RwSignal::new(String::new());
    let results = RwSignal::new(Vec::<Value>::new());
    let profile = RwSignal::new(None::<Value>);
    let selected = RwSignal::new(None::<Value>);

    let pick = move |item: Value| {
        let mut selected_item = item.clone();
        selected_item["globalRegion"] = json!(global_region.get_untracked());
        selected.set(Some(selected_item));
        status.set(String::new());
        profile.set(None);
        spawn_local(fetch_profile_now(item, region, global_region, profile, status));
    };

    let search = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let query = name.get().trim().to_string();
        if query.is_empty() {
            return;
        }
        status.set(t(lang.get(), "searchingProfiles"));
        results.set(vec![]);
        profile.set(None);
        spawn_local(async move {
            let gr = global_region.get_untracked();
            match host::request("search", &region.get_untracked(), &gr, json!({ "name": query })).await {
                Ok(data) => {
                    let list = data.get("results").and_then(Value::as_array).cloned().unwrap_or_default();
                    if list.is_empty() {
                        status.set(t(lang.get_untracked(), "noCharacters"));
                    } else if list.len() == 1 {
                        let mut selected_item = list[0].clone();
                        selected_item["globalRegion"] = json!(gr);
                        selected.set(Some(selected_item));
                        status.set(String::new());
                        fetch_profile_now(list[0].clone(), region, global_region, profile, status).await;
                    } else {
                        status.set(String::new());
                        results.set(list);
                    }
                }
                Err(e) => status.set(e),
            }
        });
    };

    view! {
        <section class="card">
            <h2>{move || t(lang.get(), "findEnemyTitle")}</h2>
            <p>{move || t(lang.get(), "findEnemySubtitle")}</p>
            <form class="search-form" on:submit=search>
                <label>
                    {move || t(lang.get(), "searchEnemyLabel")}
                    <input type="search" maxlength="20" required
                        placeholder=move || t(lang.get(), "searchCharPlaceholder")
                        on:input=move |ev| name.set(event_target_value(&ev)) />
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
            <crate::profiles::Profiles state=state profile=profile selected=selected global_region=global_region />
            <ul class="character-results">
                {move || results.get().into_iter().map(|item| {
                    let label = format!(
                        "{} · {} · {}",
                        item.get("name").and_then(Value::as_str).unwrap_or(""),
                        item.get("serverName").and_then(Value::as_str).unwrap_or(""),
                        item.get("className").and_then(Value::as_str).unwrap_or(""),
                    );
                    let chosen = item.clone();
                    view! {
                        <li><button type="button" on:click=move |_| pick(chosen.clone())>{label}</button></li>
                    }
                }).collect_view()}
            </ul>
        </section>

        {move || profile.get().map(|data| {
            let p = data.get("info").and_then(|i| i.get("profile")).cloned().unwrap_or(Value::Null);
            let title = format!(
                "{} · {}",
                p.get("characterName").and_then(Value::as_str).unwrap_or(""),
                p.get("serverName").and_then(Value::as_str).unwrap_or("")
            );
            let meta = format!(
                "{} · {} {} · {}",
                p.get("className").and_then(Value::as_str).unwrap_or(""),
                t(lang.get(), "level"),
                p.get("characterLevel").map(|v| v.to_string()).unwrap_or_default(),
                p.get("raceName").and_then(Value::as_str).unwrap_or(""),
            );
            let mut rows: Vec<String> = sheet_defenses(&data)
                .into_iter()
                .map(|(_, label, value)| format!("{label}: {value}"))
                .collect();
            if let Some(hp) = character_hp(&data) {
                rows.insert(0, format!("HP: {hp}"));
            }
            let rows_view = if rows.is_empty() {
                view! { <li>{move || t(lang.get(), "noDefenses")}</li> }.into_any()
            } else {
                let source = t(lang.get(), "characterSheetSource");
                rows.into_iter()
                    .map(|row| view! { <li>{format!("{row} ({source})")}</li> })
                    .collect_view()
                    .into_any()
            };
            let import_data = data.clone();
            view! {
                <section class="card">
                    <h2>{title}</h2>
                    <p>{meta}</p>
                    <crate::dossier::Dossier state=state data=data.clone() />
                    <h3>{move || t(lang.get(), "availableEnemyStats")}</h3>
                    <ul>{rows_view}</ul>
                    <p class="import-note">{move || t(lang.get(), "importEnemyNote")}</p>
                    <button type="button" on:click=move |_| import_status.set(import_into_calculator(state, &import_data))>{move || t(lang.get(), "importEnemyBtn")}</button>
                    <p role="status">{move || import_status.get()}</p>
                </section>
            }
        })}
    }
}
