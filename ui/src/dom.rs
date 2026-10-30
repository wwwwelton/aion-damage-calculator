//! Browser mount points for the companion panels (wasm only). Data and logic live in `companion`.

use crate::companion::{
    database_matches, format_duration, leaderboard_mean_cp, map_option_label, map_summary,
    next_occurrence, resolve_class_build, Schedule, CLASS_BUILDS, LEADERBOARD, MAPS, TIMERS,
};
use leptos::mount::mount_to;
use leptos::prelude::*;
use std::time::Duration;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{HtmlElement, HtmlInputElement, HtmlSelectElement};

const TIMER_REFRESH: Duration = Duration::from_secs(10);

fn now_ms() -> f64 {
    js_sys::Date::now()
}

/// UTC minus local, in minutes, at a UTC instant (same sign as JS `getTimezoneOffset`).
fn tz_offset_min(utc_ms: f64) -> f64 {
    js_sys::Date::new(&JsValue::from_f64(utc_ms)).get_timezone_offset()
}

fn locale() -> String {
    web_sys::window()
        .and_then(|w| w.navigator().language())
        .unwrap_or_else(|| "en-US".to_string())
}

fn format_number(n: u64) -> String {
    js_sys::Number::from(n as f64).to_locale_string(&locale()).into()
}

fn format_time(utc_ms: f64) -> String {
    js_sys::Date::new(&JsValue::from_f64(utc_ms))
        .to_locale_time_string(&locale())
        .into()
}

fn element<T: JsCast>(id: &str) -> Option<T> {
    web_sys::window()?
        .document()?
        .get_element_by_id(id)?
        .dyn_into::<T>()
        .ok()
}

fn timer_text(schedule: Schedule, now: f64) -> String {
    let next = next_occurrence(schedule, now, tz_offset_min);
    let remaining = ((next - now) / 1000.0).floor().max(0.0) as i64;
    format!("Próximo: {} (em {})", format_time(next), format_duration(remaining))
}

fn timer_cards(now: ReadSignal<f64>) -> impl IntoView {
    TIMERS
        .into_iter()
        .map(|(title, schedule)| {
            view! {
                <article class="progression-card">
                    <strong>{title}</strong>
                    <small>{move || timer_text(schedule, now.get())}</small>
                </article>
            }
        })
        .collect_view()
}

fn build_cards() -> impl IntoView {
    CLASS_BUILDS
        .into_iter()
        .map(|build| {
            view! {
                <article class="progression-card">
                    <strong>{build.class_name}</strong>
                    <small>{format!("PvE: {}", build.pve.weapon)}</small>
                    <small>{format!("PvP: {}", build.pvp.weapon)}</small>
                </article>
            }
        })
        .collect_view()
}

fn set_prop(obj: &js_sys::Object, key: &str, value: &JsValue) {
    let _ = js_sys::Reflect::set(obj, &JsValue::from_str(key), value);
}

/// `{ className, weapon, stigmas }` for the combat report in renderer.js. Unknown classes fall back to Gladiator.
#[wasm_bindgen]
pub fn class_build(attacker_class: &str, pvp: bool) -> JsValue {
    let build = resolve_class_build(attacker_class);
    let loadout = if pvp { &build.pvp } else { &build.pve };
    let stigmas = js_sys::Array::new();
    for stigma in loadout.stigmas {
        stigmas.push(&JsValue::from_str(stigma));
    }
    let obj = js_sys::Object::new();
    set_prop(&obj, "className", &JsValue::from_str(build.class_name));
    set_prop(&obj, "weapon", &JsValue::from_str(loadout.weapon));
    set_prop(&obj, "stigmas", &stigmas);
    obj.into()
}

fn leaderboard_summary() -> String {
    format!(
        "Poder de combate mediano (Top 10): {} · Limite top 10: 280.882 CP",
        format_number(leaderboard_mean_cp(&LEADERBOARD))
    )
}

fn leaderboard_rows() -> impl IntoView {
    LEADERBOARD
        .into_iter()
        .map(|entry| {
            view! {
                <li>{format!(
                    "{} · {} · {} · {} CP",
                    entry.name,
                    entry.class_name,
                    entry.server,
                    format_number(entry.cp)
                )}</li>
            }
        })
        .collect_view()
}

fn database_results(query: ReadSignal<String>) -> impl IntoView {
    move || {
        database_matches(&query.get())
            .into_iter()
            .map(|entry| {
                view! {
                    <article class="progression-card">
                        <strong>{format!("{} ({})", entry.name, entry.category)}</strong>
                        <small>{entry.detail}</small>
                    </article>
                }
            })
            .collect_view()
    }
}

fn mount_map(select: &HtmlSelectElement, summary: HtmlElement) {
    let options_parent: HtmlElement = select.clone().unchecked_into();
    std::mem::forget(mount_to(options_parent, || {
        MAPS.into_iter()
            .map(|map| view! { <option>{map_option_label(&map)}</option> })
            .collect_view()
    }));
    summary.set_text_content(Some(&map_summary(&MAPS[0])));

    let picker = select.clone();
    let on_change = Closure::<dyn FnMut()>::new(move || {
        let picked = usize::try_from(picker.selected_index())
            .ok()
            .and_then(|i| MAPS.get(i));
        if let Some(map) = picked {
            summary.set_text_content(Some(&map_summary(map)));
        }
    });
    select
        .add_event_listener_with_callback("change", on_change.as_ref().unchecked_ref())
        .ok();
    on_change.forget();
}

/// Mounts every companion panel whose container exists. Missing containers are skipped.
#[wasm_bindgen]
pub fn mount_game_companion() {
    let owner = Owner::new();
    owner.with(|| {
        let (now, set_now) = signal(now_ms());
        let (query, set_query) = signal(String::new());

        if let Some(el) = element::<HtmlElement>("timer-cards") {
            set_interval(move || set_now.set(now_ms()), TIMER_REFRESH);
            std::mem::forget(mount_to(el, move || timer_cards(now)));
        }
        if let Some(el) = element::<HtmlElement>("build-cards") {
            std::mem::forget(mount_to(el, build_cards));
        }
        if let Some(el) = element::<HtmlElement>("leaderboard-summary") {
            el.set_text_content(Some(&leaderboard_summary()));
        }
        if let Some(el) = element::<HtmlElement>("leaderboard-list") {
            std::mem::forget(mount_to(el, leaderboard_rows));
        }
        if let (Some(select), Some(summary)) = (
            element::<HtmlSelectElement>("map-select"),
            element::<HtmlElement>("map-summary"),
        ) {
            mount_map(&select, summary);
        }
        if let (Some(input), Some(results)) = (
            element::<HtmlInputElement>("db-search-input"),
            element::<HtmlElement>("db-search-results"),
        ) {
            let field = input.clone();
            let on_input =
                Closure::<dyn FnMut()>::new(move || set_query.set(field.value()));
            input
                .add_event_listener_with_callback("input", on_input.as_ref().unchecked_ref())
                .ok();
            on_input.forget();
            std::mem::forget(mount_to(results, move || database_results(query)));
        }
    });
    std::mem::forget(owner);
}
