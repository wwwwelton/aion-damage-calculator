//! Pure data and logic for the game companion panel (port of `game-companion.js`).
//! No DOM or JS APIs here, so it runs under native `cargo test`.

const HOUR_MS: f64 = 3_600_000.0;
const MINUTE_MS: f64 = 60_000.0;

pub struct MapInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub faction: &'static str,
    pub width: u32,
    pub height: u32,
    pub tiles: (u32, u32),
}

pub const MAPS: [MapInfo; 7] = [
    MapInfo { id: "World_L_Starter", name: "Poeta", faction: "Elyos", width: 4096, height: 4096, tiles: (4, 4) },
    MapInfo { id: "World_L_A", name: "Verteron / Eltnen", faction: "Elyos", width: 8192, height: 8192, tiles: (8, 8) },
    MapInfo { id: "World_D_Starter", name: "Ishalgen", faction: "Asmodian", width: 5120, height: 5120, tiles: (5, 5) },
    MapInfo { id: "World_D_A", name: "Altgard / Morheim", faction: "Asmodian", width: 8192, height: 8192, tiles: (8, 8) },
    MapInfo { id: "Abyss_Reshanta_A", name: "Reshanta (Lower)", faction: "Abyss", width: 4096, height: 4096, tiles: (4, 4) },
    MapInfo { id: "Abyss_Reshanta_B", name: "Reshanta (Middle)", faction: "Abyss", width: 2048, height: 2048, tiles: (2, 2) },
    MapInfo { id: "Abyss_Reshanta_C", name: "Reshanta (Upper)", faction: "Abyss", width: 1024, height: 1024, tiles: (1, 1) },
];

pub fn map_option_label(map: &MapInfo) -> String {
    format!("{} ({})", map.name, map.faction)
}

pub fn map_summary(map: &MapInfo) -> String {
    format!(
        "{} · {} · {}x{} px · {}x{} tiles",
        map.name, map.faction, map.width, map.height, map.tiles.0, map.tiles.1
    )
}

#[derive(Clone, Copy)]
pub enum Schedule {
    /// Fires every hour at `minute` past the hour (local time).
    Minute { minute: u32 },
    /// Fires at local hours `h` where `h % every_hours == offset_hours % every_hours`.
    Interval { every_hours: u32, offset_hours: u32 },
}

pub const RIFT: Schedule = Schedule::Interval { every_hours: 3, offset_hours: 2 };
pub const SHUGO_FESTA: Schedule = Schedule::Minute { minute: 0 };
pub const DIMENSIONAL_INVASION: Schedule = Schedule::Minute { minute: 30 };
pub const KAIRAH: Schedule = Schedule::Interval { every_hours: 3, offset_hours: 1 };

pub const TIMERS: [(&str, Schedule); 4] = [
    ("Fenda Espaço-Temporal", RIFT),
    ("Shugo Festa", SHUGO_FESTA),
    ("Invasão Dimensional", DIMENSIONAL_INVASION),
    ("Vigilante Kairah", KAIRAH),
];

/// Next occurrence of `schedule` strictly after `now_ms` (UTC epoch ms), returned as UTC epoch ms.
///
/// `tz_offset_min(t)` mirrors JS `Date.prototype.getTimezoneOffset` (UTC minus local, in minutes)
/// evaluated at UTC instant `t`. Local time is computed by shifting `now_ms` by that offset.
/// Known limit: candidate hours are computed on the shifted timeline with the offset at `now`,
/// so a DST change inside the search window can be off by one hour; upgrade by re-evaluating
/// the offset per candidate if this matters.
pub fn next_occurrence(schedule: Schedule, now_ms: f64, tz_offset_min: impl Fn(f64) -> f64) -> f64 {
    let local = now_ms - tz_offset_min(now_ms) * MINUTE_MS;
    let hour_start = local - local.rem_euclid(HOUR_MS);
    let next_local = match schedule {
        Schedule::Minute { minute } => {
            let minute = minute as f64;
            let current_minute = (local.rem_euclid(HOUR_MS) / MINUTE_MS).floor();
            let mut candidate = hour_start + minute * MINUTE_MS;
            if current_minute >= minute {
                candidate += HOUR_MS;
            }
            candidate
        }
        Schedule::Interval { every_hours, offset_hours } => {
            let target = offset_hours % every_hours;
            (1..=48)
                .map(|step| hour_start + step as f64 * HOUR_MS)
                .find(|candidate| {
                    let hour_of_day = (candidate / HOUR_MS).floor().rem_euclid(24.0) as u32;
                    hour_of_day % every_hours == target
                })
                .expect("interval schedules match within 24 hours")
        }
    };
    to_utc(next_local, &tz_offset_min)
}

/// Local shifted time back to UTC; the offset is re-read at the first guess for DST accuracy.
fn to_utc(local: f64, tz_offset_min: &impl Fn(f64) -> f64) -> f64 {
    let first_guess = local + tz_offset_min(local) * MINUTE_MS;
    local + tz_offset_min(first_guess) * MINUTE_MS
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

pub struct Loadout {
    pub weapon: &'static str,
    pub stigmas: &'static [&'static str],
}

pub struct ClassBuild {
    pub class_name: &'static str,
    pub pve: Loadout,
    pub pvp: Loadout,
}

pub const CLASS_BUILDS: [ClassBuild; 8] = [
    ClassBuild {
        class_name: "Gladiator",
        pve: Loadout { weapon: "Montante do Lorde Dragão Sombrio Brilhante +15", stigmas: &["Postura de Assalto", "Bênção de Zikel", "Explosão de Ira", "Perseverança", "Bloqueio Concentrado", "Arremesso de Lâmina"] },
        pvp: Loadout { weapon: "Montante do General Guardião +15", stigmas: &["Postura de Assalto", "Bênção de Zikel", "Arremesso de Lâmina", "Perseverança", "Salto Devastador", "Fúria Destrutiva"] },
    },
    ClassBuild {
        class_name: "Templar",
        pve: Loadout { weapon: "Espada Longa do Lorde Dragão Sombrio Brilhante +15", stigmas: &["Bandeira do Campo de Batalha", "Escudo da Perdição", "Armadura Nobre", "Provocação", "Apreensão", "Escudo da Proteção", "Punição do Senhor Empíreo"] },
        pvp: Loadout { weapon: "Espada Longa do General Guardião +15", stigmas: &["Escudo da Proteção", "Apreensão", "Punição Divina", "Armadura Nobre", "Golpe do Escudo Inabalável", "Julgamento"] },
    },
    ClassBuild {
        class_name: "Ranger",
        pve: Loadout { weapon: "Arco do Lorde Dragão Sombrio Brilhante +15", stigmas: &["Soberania de Vaizel", "Arco da Bênção", "Fogo de Supressão", "Flecha de Grifo", "Sopro da Mãe Natureza", "Chute Surpresa"] },
        pvp: Loadout { weapon: "Arco do General Guardião +15", stigmas: &["Soberania de Vaizel", "Olho de Águia", "Flecha de Grifo", "Fogo de Supressão", "Flecha Silenciadora", "Retirada Tática"] },
    },
    ClassBuild {
        class_name: "Assassin",
        pve: Loadout { weapon: "Adaga do Lorde Dragão Sombrio Brilhante +15", stigmas: &["Pacto da Celeridade", "Clone Ilusório", "Lançamento de Espada Sombria", "Presa da Besta", "Punhal de Triniel", "Caminhada nas Sombras", "Postura de Evasão"] },
        pvp: Loadout { weapon: "Adaga do General Guardião +15", stigmas: &["Pacto da Celeridade", "Caminhada nas Sombras", "Punhal de Triniel", "Golpe Sombrio", "Postura de Evasão", "Lançamento de Espada Sombria"] },
    },
    ClassBuild {
        class_name: "Sorcerer",
        pve: Loadout { weapon: "Grimório do Lorde Dragão Sombrio Brilhante +15", stigmas: &["Aprimoramento Elemental", "Barreira de Fogo", "Explosão Tardia", "Tempestade Gélida", "Escudo Protetivo de Aço", "Armadura Ártica", "Hibernação"] },
        pvp: Loadout { weapon: "Grimório do General Guardião +15", stigmas: &["Aprimoramento Elemental", "Hibernação", "Chama do Caos", "Escudo Protetivo de Aço", "Prisão Gélida", "Teleporte"] },
    },
    ClassBuild {
        class_name: "Spiritmaster",
        pve: Loadout { weapon: "Orbe do Lorde Dragão Sombrio Brilhante +15", stigmas: &["Invocação: Espírito Ancestral", "Ataque Conjunto: Corrosão", "Aprimoramento: Bendição do Espírito", "Bênção do Fogo Ardente", "Comando: Substituto", "Sucção"] },
        pvp: Loadout { weapon: "Orbe do General Guardião +15", stigmas: &["Maldição dos Espíritos", "Invocação: Espírito Ancestral", "Comando: Substituto", "Aprimoramento: Bendição do Espírito", "Nuvem Aflitiva", "Sucção"] },
    },
    ClassBuild {
        class_name: "Cleric",
        pve: Loadout { weapon: "Maça do Lorde Dragão Sombrio Brilhante +15", stigmas: &["Punição da Terra", "Oração da Amplificação", "Luz da Proteção", "Aura Nobre", "Salvação", "Ressurreição por Invocação", "Soberania de Yustiel"] },
        pvp: Loadout { weapon: "Maça do General Guardião +15", stigmas: &["Luz da Proteção", "Punição da Terra", "Graça Divina", "Salvação", "Correntes do Julgamento", "Oração da Amplificação"] },
    },
    ClassBuild {
        class_name: "Chanter",
        pve: Loadout { weapon: "Cajado do Lorde Dragão Sombrio Brilhante +15", stigmas: &["Mantra do Imbatível", "Soberania do Vendaval", "Bênção da Guarda", "Fúria de Marchutan", "Mantra da Disparada", "Defesa Concentrada"] },
        pvp: Loadout { weapon: "Cajado do General Guardião +15", stigmas: &["Mantra do Imbatível", "Soberania do Vendaval", "Fúria de Marchutan", "Golpe Desestabilizador", "Defesa Concentrada", "Mantra da Vitória"] },
    },
];

/// Case-insensitive class match; unknown or blank names fall back to Gladiator (as the JS did).
pub fn resolve_class_build(attacker_class: &str) -> &'static ClassBuild {
    let wanted = attacker_class.trim().to_lowercase();
    CLASS_BUILDS
        .iter()
        .find(|build| build.class_name.to_lowercase() == wanted)
        .unwrap_or(&CLASS_BUILDS[0])
}

pub struct LeaderboardEntry {
    pub name: &'static str,
    pub class_name: &'static str,
    pub server: &'static str,
    pub cp: u64,
}

pub const LEADERBOARD: [LeaderboardEntry; 10] = [
    LeaderboardEntry { name: "Asulgard", class_name: "Ranger", server: "Siel (EU)", cp: 287070 },
    LeaderboardEntry { name: "dieyoung", class_name: "Ranger", server: "Israphel (EU)", cp: 286732 },
    LeaderboardEntry { name: "gigawhaleYT", class_name: "Chanter", server: "Israphel (EU)", cp: 286058 },
    LeaderboardEntry { name: "白菜菜", class_name: "Chanter", server: "Israphel (EU)", cp: 284746 },
    LeaderboardEntry { name: "kbbb", class_name: "Gladiator", server: "Israphel (Asia)", cp: 283814 },
    LeaderboardEntry { name: "小小时光", class_name: "Ranger", server: "Israphel (EU)", cp: 282417 },
    LeaderboardEntry { name: "DaBaiCai", class_name: "Ranger", server: "Israphel (EU)", cp: 282330 },
    LeaderboardEntry { name: "LeOz", class_name: "Ranger", server: "Siel (Asia)", cp: 281614 },
    LeaderboardEntry { name: "まつびさいき", class_name: "Sorcerer", server: "Zikel (Asia)", cp: 281476 },
    LeaderboardEntry { name: "ZZ", class_name: "Cleric", server: "Zikel (NA East)", cp: 280882 },
];

/// Mean CP of the snapshot, rounded. The JS label calls this "mediano" (median) but it is a mean;
/// kept as-is to match the original behavior.
pub fn leaderboard_mean_cp(entries: &[LeaderboardEntry]) -> u64 {
    let sum: u64 = entries.iter().map(|entry| entry.cp).sum();
    (sum as f64 / entries.len() as f64).round() as u64
}

pub struct DatabaseEntry {
    pub name: &'static str,
    pub category: &'static str,
    pub detail: &'static str,
}

pub const DATABASE: [DatabaseEntry; 6] = [
    DatabaseEntry { name: "Montante do Lorde Dragão Sombrio Brilhante", category: "Arma", detail: "Espada de duas mãos · Nível 102 · Heroico" },
    DatabaseEntry { name: "Asas do Guerreiro", category: "Asas", detail: "Grau Lendário · Nível de encantamento +0" },
    DatabaseEntry { name: "Postura de Assalto", category: "Habilidade", detail: "Estigma ativa · Gladiador · Aumento ofensivo" },
    DatabaseEntry { name: "Abyss Outland", category: "Conteúdo", detail: "Battle royale do Abismo · Partidas a cada 30 minutos" },
    DatabaseEntry { name: "Vigilante Kairah", category: "Chefe", detail: "Chefe de campo do Abismo Inferior · Ciclo de 3 horas" },
    DatabaseEntry { name: "Treading on Black Wings", category: "Título", detail: "Título de ataque · Bônus de Ataque PvP" },
];

/// Case-insensitive substring match on name or category; an empty query matches everything.
pub fn database_matches(query: &str) -> Vec<&'static DatabaseEntry> {
    let query = query.trim().to_lowercase();
    DATABASE
        .iter()
        .filter(|entry| {
            query.is_empty()
                || entry.name.to_lowercase().contains(&query)
                || entry.category.to_lowercase().contains(&query)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const HOUR: f64 = HOUR_MS;
    const UTC: fn(f64) -> f64 = |_| 0.0;
    // Brazil (UTC-3): getTimezoneOffset() is +180.
    const BRT: fn(f64) -> f64 = |_| 180.0;

    #[test]
    fn format_duration_under_hour_is_mm_ss() {
        assert_eq!(format_duration(0), "00:00");
        assert_eq!(format_duration(-5), "00:00");
        assert_eq!(format_duration(59), "00:59");
        assert_eq!(format_duration(75), "01:15");
    }

    #[test]
    fn format_duration_over_hour_is_h_mm() {
        assert_eq!(format_duration(3600), "1h 00m");
        assert_eq!(format_duration(3725), "1h 02m");
    }

    #[test]
    fn minute_schedule_same_hour_and_rollover() {
        // 10:10 UTC: :30 is still this hour (10:30); :00 already passed (11:00).
        let now = 10.0 * HOUR + 10.0 * MINUTE_MS;
        assert_eq!(next_occurrence(SHUGO_FESTA, now, UTC), 11.0 * HOUR);
        assert_eq!(next_occurrence(DIMENSIONAL_INVASION, now, UTC), 10.5 * HOUR);
    }

    #[test]
    fn minute_schedule_at_exact_minute_rolls_to_next_hour() {
        // Exactly 10:30:00 UTC: :30 already passed, next is 11:30.
        let now = 10.5 * HOUR;
        assert_eq!(next_occurrence(DIMENSIONAL_INVASION, now, UTC), 11.5 * HOUR);
    }

    #[test]
    fn interval_schedule_uses_local_hour() {
        // 11:00 UTC is 08:00 in Brazil. Rift fires at local hours 2,5,8,11,...
        // Strictly after 08:00 local -> 11:00 local = 14:00 UTC.
        let now = 11.0 * HOUR;
        assert_eq!(next_occurrence(RIFT, now, BRT), 14.0 * HOUR);
    }

    #[test]
    fn interval_schedule_is_strictly_after_now() {
        // Kairah fires at UTC hours 1,4,7,...; exactly 01:00 UTC must skip to 04:00.
        assert_eq!(next_occurrence(KAIRAH, 1.0 * HOUR, UTC), 4.0 * HOUR);
        assert_eq!(next_occurrence(KAIRAH, 0.0, UTC), 1.0 * HOUR);
    }

    #[test]
    fn leaderboard_mean_is_rounded() {
        // Sum 2_837_139 / 10 = 283_713.9 -> 283_714.
        assert_eq!(leaderboard_mean_cp(&LEADERBOARD), 283_714);
    }

    #[test]
    fn class_build_matches_case_insensitively_and_defaults_to_gladiator() {
        assert_eq!(resolve_class_build("  ranger ").class_name, "Ranger");
        assert_eq!(resolve_class_build("Unknown").class_name, "Gladiator");
        assert_eq!(resolve_class_build("").class_name, "Gladiator");
    }

    #[test]
    fn database_empty_query_returns_all() {
        assert_eq!(database_matches("").len(), DATABASE.len());
        assert_eq!(database_matches("   ").len(), DATABASE.len());
    }

    #[test]
    fn database_matches_name_or_category_case_insensitively() {
        let by_category = database_matches("  ARMA ");
        assert_eq!(by_category.len(), 1);
        assert_eq!(by_category[0].name, "Montante do Lorde Dragão Sombrio Brilhante");

        let by_name = database_matches("kairah");
        assert_eq!(by_name.len(), 1);
        assert_eq!(by_name[0].category, "Chefe");

        assert!(database_matches("inexistente").is_empty());
    }

    #[test]
    fn map_summary_matches_original_format() {
        assert_eq!(map_summary(&MAPS[0]), "Poeta · Elyos · 4096x4096 px · 4x4 tiles");
        assert_eq!(map_option_label(&MAPS[6]), "Reshanta (Upper) (Abyss)");
    }
}
