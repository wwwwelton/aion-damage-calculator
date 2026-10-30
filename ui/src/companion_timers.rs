//! Companion timers, interactive map, class meta builds, leaderboard snapshot,
//! and game database directory. Port of game-companion.js using web_sys.

use wasm_bindgen::prelude::*;
use web_sys::{Document, HtmlInputElement, HtmlSelectElement, Window};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MapMeta {
    pub id: &'static str,
    pub name: &'static str,
    pub faction: &'static str,
    pub width: u32,
    pub height: u32,
    pub tiles: (u32, u32),
}

pub const MAP_METADATA: &[MapMeta] = &[
    MapMeta { id: "World_L_Starter", name: "Poeta", faction: "Elyos", width: 4096, height: 4096, tiles: (4, 4) },
    MapMeta { id: "World_L_A", name: "Verteron / Eltnen", faction: "Elyos", width: 8192, height: 8192, tiles: (8, 8) },
    MapMeta { id: "World_D_Starter", name: "Ishalgen", faction: "Asmodian", width: 5120, height: 5120, tiles: (5, 5) },
    MapMeta { id: "World_D_A", name: "Altgard / Morheim", faction: "Asmodian", width: 8192, height: 8192, tiles: (8, 8) },
    MapMeta { id: "Abyss_Reshanta_A", name: "Reshanta (Lower)", faction: "Abyss", width: 4096, height: 4096, tiles: (4, 4) },
    MapMeta { id: "Abyss_Reshanta_B", name: "Reshanta (Middle)", faction: "Abyss", width: 2048, height: 2048, tiles: (2, 2) },
    MapMeta { id: "Abyss_Reshanta_C", name: "Reshanta (Upper)", faction: "Abyss", width: 1024, height: 1024, tiles: (1, 1) },
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Schedule {
    Minute {
        minute: u32,
        duration_minutes: u32,
    },
    Interval {
        interval_hours: u32,
        offset_hours: u32,
        duration_minutes: u32,
    },
}

pub struct ScheduledEvent {
    pub next: js_sys::Date,
    pub duration_minutes: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClassBuild {
    pub class_name: &'static str,
    pub pve_weapon: &'static str,
    pub pvp_weapon: &'static str,
}

pub const CLASS_BUILDS: &[ClassBuild] = &[
    ClassBuild {
        class_name: "Gladiator",
        pve_weapon: "Montante do Lorde Dragão Sombrio Brilhante +15",
        pvp_weapon: "Montante do General Guardião +15",
    },
    ClassBuild {
        class_name: "Templar",
        pve_weapon: "Espada Longa do Lorde Dragão Sombrio Brilhante +15",
        pvp_weapon: "Espada Longa do General Guardião +15",
    },
    ClassBuild {
        class_name: "Ranger",
        pve_weapon: "Arco do Lorde Dragão Sombrio Brilhante +15",
        pvp_weapon: "Arco do General Guardião +15",
    },
    ClassBuild {
        class_name: "Assassin",
        pve_weapon: "Adaga do Lorde Dragão Sombrio Brilhante +15",
        pvp_weapon: "Adaga do General Guardião +15",
    },
    ClassBuild {
        class_name: "Sorcerer",
        pve_weapon: "Grimório do Lorde Dragão Sombrio Brilhante +15",
        pvp_weapon: "Grimório do General Guardião +15",
    },
    ClassBuild {
        class_name: "Spiritmaster",
        pve_weapon: "Orbe do Lorde Dragão Sombrio Brilhante +15",
        pvp_weapon: "Orbe do General Guardião +15",
    },
    ClassBuild {
        class_name: "Cleric",
        pve_weapon: "Maça do Lorde Dragão Sombrio Brilhante +15",
        pvp_weapon: "Maça do General Guardião +15",
    },
    ClassBuild {
        class_name: "Chanter",
        pve_weapon: "Cajado do Lorde Dragão Sombrio Brilhante +15",
        pvp_weapon: "Cajado do General Guardião +15",
    },
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LeaderboardEntry {
    pub rank: u32,
    pub name: &'static str,
    pub class_name: &'static str,
    pub server: &'static str,
    pub cp: u64,
}

pub const LEADERBOARD_SNAPSHOT: &[LeaderboardEntry] = &[
    LeaderboardEntry { rank: 1, name: "Asulgard", class_name: "Ranger", server: "Siel (EU)", cp: 287070 },
    LeaderboardEntry { rank: 2, name: "dieyoung", class_name: "Ranger", server: "Israphel (EU)", cp: 286732 },
    LeaderboardEntry { rank: 3, name: "gigawhaleYT", class_name: "Chanter", server: "Israphel (EU)", cp: 286058 },
    LeaderboardEntry { rank: 4, name: "白菜菜", class_name: "Chanter", server: "Israphel (EU)", cp: 284746 },
    LeaderboardEntry { rank: 5, name: "kbbb", class_name: "Gladiator", server: "Israphel (Asia)", cp: 283814 },
    LeaderboardEntry { rank: 6, name: "小小时光", class_name: "Ranger", server: "Israphel (EU)", cp: 282417 },
    LeaderboardEntry { rank: 7, name: "DaBaiCai", class_name: "Ranger", server: "Israphel (EU)", cp: 282330 },
    LeaderboardEntry { rank: 8, name: "LeOz", class_name: "Ranger", server: "Siel (Asia)", cp: 281614 },
    LeaderboardEntry { rank: 9, name: "まつびさいき", class_name: "Sorcerer", server: "Zikel (Asia)", cp: 281476 },
    LeaderboardEntry { rank: 10, name: "ZZ", class_name: "Cleric", server: "Zikel (NA East)", cp: 280882 },
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DatabaseEntry {
    pub kind: &'static str,
    pub name: &'static str,
    pub category: &'static str,
    pub detail: &'static str,
}

pub const DATABASE_ENTRIES: &[DatabaseEntry] = &[
    DatabaseEntry { kind: "item", name: "Montante do Lorde Dragão Sombrio Brilhante", category: "Arma", detail: "Espada de duas mãos · Nível 102 · Heroico" },
    DatabaseEntry { kind: "item", name: "Asas do Guerreiro", category: "Asas", detail: "Grau Lendário · Nível de encantamento +0" },
    DatabaseEntry { kind: "skill", name: "Postura de Assalto", category: "Habilidade", detail: "Estigma ativa · Gladiador · Aumento ofensivo" },
    DatabaseEntry { kind: "dungeon", name: "Abyss Outland", category: "Conteúdo", detail: "Battle royale do Abismo · Partidas a cada 30 minutos" },
    DatabaseEntry { kind: "monster", name: "Vigilante Kairah", category: "Chefe", detail: "Chefe de campo do Abismo Inferior · Ciclo de 3 horas" },
    DatabaseEntry { kind: "title", name: "Treading on Black Wings", category: "Título", detail: "Título de ataque · Bônus de Ataque PvP" },
];

fn window() -> Window {
    web_sys::window().expect("global window")
}

fn document() -> Document {
    window().document().expect("document on window")
}

pub fn format_duration(seconds: i64) -> String {
    if seconds <= 0 {
        return "00:00".to_string();
    }
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let rem = seconds % 60;
    if hours > 0 {
        format!("{hours}h {minutes:02}m")
    } else {
        format!("{minutes:02}:{rem:02}")
    }
}

pub fn next_scheduled_event(schedule: &Schedule, now: &js_sys::Date) -> Option<ScheduledEvent> {
    match *schedule {
        Schedule::Minute { minute, duration_minutes } => {
            let next = js_sys::Date::new(&now.clone().into());
            next.set_seconds(0);
            next.set_milliseconds(0);
            if next.get_minutes() >= minute {
                next.set_hours(next.get_hours() + 1);
            }
            next.set_minutes(minute);
            Some(ScheduledEvent { next, duration_minutes })
        }
        Schedule::Interval { interval_hours, offset_hours, duration_minutes } => {
            let cycle = interval_hours;
            let offset = offset_hours % cycle;
            let next = js_sys::Date::new(&now.clone().into());
            next.set_minutes(0);
            next.set_seconds(0);
            next.set_milliseconds(0);
            let mut step = 0;
            while step < 48 {
                if (next.get_hours() % cycle == offset) && next.get_time() > now.get_time() {
                    break;
                }
                next.set_hours(next.get_hours() + 1);
                step += 1;
            }
            Some(ScheduledEvent { next, duration_minutes })
        }
    }
}

pub fn render_timers(doc: &Document) {
    let Some(timer_cards) = doc.get_element_by_id("timer-cards") else {
        return;
    };
    let now = js_sys::Date::new_0();
    let items: [(&str, Schedule); 4] = [
        ("Fenda Espaço-Temporal", Schedule::Interval { interval_hours: 3, offset_hours: 2, duration_minutes: 60 }),
        ("Shugo Festa", Schedule::Minute { minute: 0, duration_minutes: 10 }),
        ("Invasão Dimensional", Schedule::Minute { minute: 30, duration_minutes: 15 }),
        ("Vigilante Kairah", Schedule::Interval { interval_hours: 3, offset_hours: 1, duration_minutes: 30 }),
    ];

    timer_cards.set_inner_html("");
    for (title, schedule) in &items {
        if let Ok(card) = doc.create_element("article") {
            card.set_class_name("progression-card");
            if let Ok(strong) = doc.create_element("strong") {
                strong.set_text_content(Some(title));
                let _ = card.append_child(&strong);
            }
            if let Ok(small) = doc.create_element("small") {
                let ev = next_scheduled_event(schedule, &now);
                let text = match &ev {
                    Some(e) => {
                        let diff = ((e.next.get_time() - now.get_time()) / 1000.0).max(0.0) as i64;
                        let time_str = String::from(e.next.to_locale_time_string("default"));
                        let dur_str = format_duration(diff);
                        format!("Próximo: {time_str} (em {dur_str})")
                    }
                    None => "Próximo: — (em 00:00)".to_string(),
                };
                small.set_text_content(Some(&text));
                let _ = card.append_child(&small);
            }
            let _ = timer_cards.append_child(&card);
        }
    }
}

pub fn init_timer_cards(doc: &Document) {
    if doc.get_element_by_id("timer-cards").is_none() {
        return;
    }
    render_timers(doc);
    let closure = Closure::<dyn FnMut()>::new(move || {
        let doc = document();
        render_timers(&doc);
    });
    let _ = window().set_interval_with_callback_and_timeout_and_arguments_0(
        closure.as_ref().unchecked_ref(),
        10_000,
    );
    closure.forget();
}

pub fn init_map_select(doc: &Document) {
    let Some(select) = doc.get_element_by_id("map-select").and_then(|el| el.dyn_into::<HtmlSelectElement>().ok()) else {
        return;
    };
    select.set_inner_html("");
    for map in MAP_METADATA {
        if let Ok(opt) = doc.create_element("option") {
            let _ = opt.set_attribute("value", map.id);
            opt.set_text_content(Some(&format!("{} ({})", map.name, map.faction)));
            let _ = select.append_child(&opt);
        }
    }

    fn update_summary(doc: &Document, map_id: &str) {
        if let Some(summary) = doc.get_element_by_id("map-summary") {
            let chosen = MAP_METADATA.iter().find(|m| m.id == map_id).unwrap_or(&MAP_METADATA[0]);
            summary.set_text_content(Some(&format!(
                "{} · {} · {}x{} px · {}x{} tiles",
                chosen.name, chosen.faction, chosen.width, chosen.height, chosen.tiles.0, chosen.tiles.1
            )));
        }
    }

    update_summary(doc, &select.value());

    let select_clone = select.clone();
    let on_change = Closure::<dyn FnMut()>::new(move || {
        let doc = document();
        update_summary(&doc, &select_clone.value());
    });
    select.set_onchange(Some(on_change.as_ref().unchecked_ref()));
    on_change.forget();
}

pub fn populate_build_cards(doc: &Document) {
    let Some(build_cards) = doc.get_element_by_id("build-cards") else {
        return;
    };
    build_cards.set_inner_html("");
    for build in CLASS_BUILDS {
        if let Ok(card) = doc.create_element("article") {
            card.set_class_name("progression-card");
            if let Ok(strong) = doc.create_element("strong") {
                strong.set_text_content(Some(build.class_name));
                let _ = card.append_child(&strong);
            }
            if let Ok(pve) = doc.create_element("small") {
                pve.set_text_content(Some(&format!("PvE: {}", build.pve_weapon)));
                let _ = card.append_child(&pve);
            }
            if let Ok(pvp) = doc.create_element("small") {
                pvp.set_text_content(Some(&format!("PvP: {}", build.pvp_weapon)));
                let _ = card.append_child(&pvp);
            }
            let _ = build_cards.append_child(&card);
        }
    }
}

pub fn populate_leaderboard(doc: &Document) {
    let Some(leaderboard_list) = doc.get_element_by_id("leaderboard-list") else {
        return;
    };
    if let Some(summary) = doc.get_element_by_id("leaderboard-summary") {
        let sum: u64 = LEADERBOARD_SNAPSHOT.iter().map(|e| e.cp).sum();
        let median = (sum as f64 / LEADERBOARD_SNAPSHOT.len() as f64).round() as u64;
        let median_str = String::from(js_sys::Number::from(median as f64).to_locale_string("default"));
        summary.set_text_content(Some(&format!(
            "Poder de combate mediano (Top 10): {median_str} · Limite top 10: 280.882 CP"
        )));
    }

    leaderboard_list.set_inner_html("");
    for entry in LEADERBOARD_SNAPSHOT {
        if let Ok(li) = doc.create_element("li") {
            let cp_str = String::from(js_sys::Number::from(entry.cp as f64).to_locale_string("default"));
            li.set_text_content(Some(&format!(
                "{} · {} · {} · {} CP",
                entry.name, entry.class_name, entry.server, cp_str
            )));
            let _ = leaderboard_list.append_child(&li);
        }
    }
}

pub fn render_database(doc: &Document, query: &str) {
    let Some(db_results) = doc.get_element_by_id("db-search-results") else {
        return;
    };
    let q = query.trim().to_lowercase();
    db_results.set_inner_html("");
    for entry in DATABASE_ENTRIES {
        if q.is_empty()
            || entry.name.to_lowercase().contains(&q)
            || entry.category.to_lowercase().contains(&q)
        {
            if let Ok(card) = doc.create_element("article") {
                card.set_class_name("progression-card");
                if let Ok(strong) = doc.create_element("strong") {
                    strong.set_text_content(Some(&format!("{} ({})", entry.name, entry.category)));
                    let _ = card.append_child(&strong);
                }
                if let Ok(small) = doc.create_element("small") {
                    small.set_text_content(Some(entry.detail));
                    let _ = card.append_child(&small);
                }
                let _ = db_results.append_child(&card);
            }
        }
    }
}

pub fn init_database_search(doc: &Document) {
    let Some(db_input) = doc.get_element_by_id("db-search-input").and_then(|el| el.dyn_into::<HtmlInputElement>().ok()) else {
        return;
    };
    render_database(doc, &db_input.value());

    let db_input_clone = db_input.clone();
    let on_input = Closure::<dyn FnMut()>::new(move || {
        let doc = document();
        render_database(&doc, &db_input_clone.value());
    });
    db_input.set_oninput(Some(on_input.as_ref().unchecked_ref()));
    on_input.forget();
}

pub fn init_companion_timers() {
    init();
}

pub fn init() {
    let doc = document();
    init_map_select(&doc);
    init_timer_cards(&doc);
    populate_build_cards(&doc);
    populate_leaderboard(&doc);
    init_database_search(&doc);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duration_formatting() {
        assert_eq!(format_duration(0), "00:00");
        assert_eq!(format_duration(45), "00:45");
        assert_eq!(format_duration(3599), "59:59");
        assert_eq!(format_duration(3600), "1h 00m");
        assert_eq!(format_duration(3665), "1h 01m");
    }

    #[test]
    fn metadata_invariants() {
        assert_eq!(MAP_METADATA.len(), 7);
        for m in MAP_METADATA {
            assert!(m.width > 0 && m.height > 0);
            assert!(m.tiles.0 > 0 && m.tiles.1 > 0);
        }
        assert_eq!(CLASS_BUILDS.len(), 8);
        assert_eq!(LEADERBOARD_SNAPSHOT.len(), 10);
        assert_eq!(DATABASE_ENTRIES.len(), 6);
    }
}
