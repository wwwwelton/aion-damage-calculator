use crate::{i18n::t, state::AppState};
use leptos::prelude::*;
use serde_json::Value;
use std::sync::LazyLock;

static DATA: LazyLock<Value> = LazyLock::new(|| serde_json::from_str(include_str!("../data/companion.json")).expect("companion dataset"));

fn next_event(now: &js_sys::Date, minute: u32, interval: Option<(u32, u32)>) -> js_sys::Date {
    let next = js_sys::Date::new(&now.clone().into());
    next.set_seconds(0);
    next.set_milliseconds(0);
    if let Some((cycle, offset)) = interval {
        next.set_minutes(0);
        while next.get_time() <= now.get_time() || next.get_hours() % cycle != offset % cycle {
            next.set_hours(next.get_hours() + 1);
        }
    } else {
        if next.get_minutes() >= minute {
            next.set_hours(next.get_hours() + 1);
        }
        next.set_minutes(minute);
    }
    next
}

fn countdown(seconds: u64) -> String {
    if seconds >= 3600 { format!("{}h {:02}m", seconds / 3600, seconds % 3600 / 60) }
    else { format!("{:02}:{:02}", seconds / 60, seconds % 60) }
}

#[component]
pub fn TimersTab(state: AppState) -> impl IntoView {
    let now = RwSignal::new(js_sys::Date::now());
    let interval = set_interval_with_handle(move || now.set(js_sys::Date::now()), std::time::Duration::from_secs(10)).expect("timer interval");
    on_cleanup(move || interval.clear());
    let map = RwSignal::new(0usize);
    let query = RwSignal::new(String::new());
    view! {
        <section class="card">
            <h2>{move || t(state.lang.get(), "timersTitle")}</h2>
            <p>{move || t(state.lang.get(), "timersSubtitle")}</p>
            <div id="timer-cards" class="progression-grid">{move || {
                let date = js_sys::Date::new(&now.get().into());
                [("Fenda Espaço-Temporal", 0, Some((3, 2))), ("Shugo Festa", 0, None), ("Invasão Dimensional", 30, None), ("Vigilante Kairah", 0, Some((3, 1)))].into_iter().map(|(title, minute, interval)| {
                    let next = next_event(&date, minute, interval);
                    let seconds = ((next.get_time() - date.get_time()) / 1000.0).max(0.0) as u64;
                    let label = format!("Próximo: {:02}:{:02}:{:02} (em {})", next.get_hours(), next.get_minutes(), next.get_seconds(), countdown(seconds));
                    view! { <article class="progression-card"><strong>{title}</strong><small>{label}</small></article> }
                }).collect_view()
            }}</div>
        </section>
        <section class="card">
            <h2>{move || t(state.lang.get(), "mapTitle")}</h2>
            <p>{move || t(state.lang.get(), "mapSubtitle")}</p>
            <label>{move || t(state.lang.get(), "mapSelectLabel")}
                <select id="map-select" on:change=move |ev| map.set(event_target_value(&ev).parse().unwrap_or(0))>
                    {DATA["MAP_METADATA"].as_array().unwrap().iter().enumerate().map(|(i, m)| view! {
                        <option value=i.to_string()>{format!("{} ({})", m["name"].as_str().unwrap(), m["faction"].as_str().unwrap())}</option>
                    }).collect_view()}
                </select>
            </label>
            <p id="map-summary">{move || {
                let m = &DATA["MAP_METADATA"][map.get()];
                format!("{} · {} · {}x{} px · {}x{} tiles", m["name"].as_str().unwrap(), m["faction"].as_str().unwrap(), m["width"], m["height"], m["tiles"][0], m["tiles"][1])
            }}</p>
        </section>
        <section class="card">
            <h2>{move || t(state.lang.get(), "buildsTitle")}</h2>
            <p>{move || t(state.lang.get(), "buildsSubtitle")}</p>
            <div id="build-cards" class="progression-grid">{DATA["CLASS_BUILDS"].as_object().unwrap().iter().map(|(name, build)| view! {
                <article class="progression-card"><strong>{name.clone()}</strong>
                    <small>{format!("PvE: {}", build["pve"]["weapon"].as_str().unwrap())}</small>
                    <small>{format!("PvP: {}", build["pvp"]["weapon"].as_str().unwrap())}</small>
                </article>
            }).collect_view()}</div>
        </section>
        <section class="card">
            <h2>{move || t(state.lang.get(), "leaderboardTitle")}</h2>
            <p>{move || t(state.lang.get(), "leaderboardSubtitle")}</p>
            <p>{format!("Poder de combate mediano (Top 10): {} · Limite top 10: 280.882 CP", (DATA["LEADERBOARD_SNAPSHOT"].as_array().unwrap().iter().map(|e| e["cp"].as_u64().unwrap()).sum::<u64>() as f64 / 10.0).round())}</p>
            <ol id="leaderboard-list">{DATA["LEADERBOARD_SNAPSHOT"].as_array().unwrap().iter().map(|entry| view! {
                <li>{format!("{} · {} · {} · {} CP", entry["name"].as_str().unwrap(), entry["className"].as_str().unwrap(), entry["server"].as_str().unwrap(), entry["cp"])}</li>
            }).collect_view()}</ol>
        </section>
        <section class="card">
            <h2>{move || t(state.lang.get(), "dbTitle")}</h2>
            <p>{move || t(state.lang.get(), "dbSubtitle")}</p>
            <input id="db-search-input" type="search" placeholder=move || t(state.lang.get(), "dbSearchPlaceholder") on:input=move |ev| query.set(event_target_value(&ev)) />
            <div id="db-search-results" class="progression-grid">{move || {
                let q = query.get().trim().to_lowercase();
                DATA["DATABASE_ENTRIES"].as_array().unwrap().iter().filter(|entry| {
                    entry["name"].as_str().unwrap().to_lowercase().contains(&q) || entry["category"].as_str().unwrap().to_lowercase().contains(&q)
                }).map(|entry| view! {
                    <article class="progression-card"><strong>{format!("{} ({})", entry["name"].as_str().unwrap(), entry["category"].as_str().unwrap())}</strong><small>{entry["detail"].as_str().unwrap().to_owned()}</small></article>
                }).collect_view()
            }}</div>
        </section>
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn countdown_boundaries() {
        assert_eq!(super::countdown(0), "00:00");
        assert_eq!(super::countdown(3599), "59:59");
        assert_eq!(super::countdown(3660), "1h 01m");
    }
}
