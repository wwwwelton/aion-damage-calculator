// Mobs & Bosses tab: offline catalog search, details, and load-into-calculator. Port of bosses.js.

use crate::i18n::{format_t, t};
use crate::state::{ActiveTab, AppState};
use aion2_core::calculator::TargetMode;
use leptos::prelude::*;
use serde::Deserialize;
use std::sync::LazyLock;

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct Attack {
    #[serde(rename = "type")]
    pub kind: String,
    pub reach: Option<serde_json::Value>,
    pub interval: Option<serde_json::Value>,
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct Skill {
    pub name: String,
    pub damage: String,
    pub cast: String,
    pub cooldown: String,
    pub range: String,
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct Monster {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub level: String,
    pub rank: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub group: Option<String>,
    pub location: Option<String>,
    pub subtitle: Option<String>,
    pub portrait: Option<String>,
    #[serde(rename = "spawnPoints")]
    pub spawn_points: Option<u32>,
    pub attack: Option<Attack>,
    #[serde(default)]
    pub skills: Vec<Skill>,
}

// Mobs have no group/skills/attack in the snapshot; serde defaults cover that.
static BOSSES: LazyLock<Vec<Monster>> =
    LazyLock::new(|| serde_json::from_str(include_str!("../data/bosses.json")).expect("bosses.json"));
static MOBS: LazyLock<Vec<Monster>> =
    LazyLock::new(|| serde_json::from_str(include_str!("../data/mobs.json")).expect("mobs.json"));

const BOSS_TYPES: [&str; 4] = ["Cogni", "Fera", "Natura", "Varian"];

fn text_of(v: &Option<serde_json::Value>) -> String {
    match v {
        Some(serde_json::Value::String(s)) => s.clone(),
        Some(other) => other.to_string(),
        None => "?".into(),
    }
}

#[component]
pub fn BossTab(state: AppState) -> impl IntoView {
    let is_boss = RwSignal::new(true);
    let group = RwSignal::new(String::new());
    let search = RwSignal::new(String::new());
    let choice = RwSignal::new(String::new());

    let catalog = move || if is_boss.get() { &*BOSSES } else { &*MOBS };

    let visible = Memo::new(move |_| {
        let needle = search.get().trim().to_lowercase();
        let g = group.get();
        let boss = is_boss.get();
        catalog()
            .iter()
            .filter(|m| !boss || g.is_empty() || m.group.as_deref() == Some(g.as_str()))
            .filter(|m| {
                format!("{} {} {}", m.name, m.location.as_deref().unwrap_or(""), m.subtitle.as_deref().unwrap_or(""))
                    .to_lowercase()
                    .contains(&needle)
            })
            .cloned()
            .collect::<Vec<Monster>>()
    });

    let selected = Memo::new(move |_| {
        let id = choice.get();
        visible.get().into_iter().find(|m| m.id == id)
    });

    let lang = state.lang;
    let count_text = move || {
        let kind = if is_boss.get() { t(lang.get(), "bosses") } else { t(lang.get(), "mobs") };
        format_t(
            lang.get(),
            "catalogCount",
            &[
                ("shown", &visible.get().len().to_string()),
                ("total", &catalog().len().to_string()),
                ("kind", &kind),
            ],
        )
    };

    let import = move |_| {
        let Some(m) = selected.get() else { return };
        let kind = if BOSS_TYPES.contains(&m.kind.as_str()) { m.kind.to_lowercase() } else { "special".into() };
        state.target_mode.set(if is_boss.get() { TargetMode::Boss } else { TargetMode::Monster });
        state.creature_type.set(if kind == "special" { None } else { Some(kind) });
        state.target_name.set(m.name.clone());
        state.tab.set(ActiveTab::Calculator);
    };

    view! {
        <section class="card">
            <h2>{move || t(lang.get(), "bossTitle")}</h2>
            <div class="boss-filters">
                <label>
                    {move || t(lang.get(), "opponentLabel")}
                    <select on:change=move |ev| {
                        is_boss.set(event_target_value(&ev) == "boss");
                        choice.set(String::new());
                    }>
                        <option value="boss">{move || t(lang.get(), "opponentBoss")}</option>
                        <option value="mob">{move || t(lang.get(), "opponentMob")}</option>
                    </select>
                </label>
                <label hidden=move || !is_boss.get()>
                    {move || t(lang.get(), "bossGroupLabel")}
                    <select on:change=move |ev| group.set(event_target_value(&ev))>
                        <option value="">{move || t(lang.get(), "bossGroupAll")}</option>
                        <option value="Field boss">{move || t(lang.get(), "bossGroupField")}</option>
                        <option value="Hero / legend">{move || t(lang.get(), "bossGroupHero")}</option>
                    </select>
                </label>
                <label>
                    {move || t(lang.get(), "bossSearchLabel")}
                    <input type="search"
                        placeholder=move || t(lang.get(), "bossSearchPlaceholder")
                        on:input=move |ev| { search.set(event_target_value(&ev)); choice.set(String::new()); } />
                </label>
                <label>
                    {move || t(lang.get(), "monsterLabel")}
                    <select prop:value=move || choice.get() on:change=move |ev| choice.set(event_target_value(&ev))>
                        <option value="">{move || t(lang.get(), "selectMonster")}</option>
                        {move || visible.get().into_iter().map(|m| {
                            let label = format!(
                                "{} · Lv {} · {} · {}",
                                m.name, m.level, m.kind, m.group.clone().unwrap_or(m.rank.clone())
                            );
                            view! { <option value=m.id.clone()>{label}</option> }
                        }).collect_view()}
                    </select>
                </label>
            </div>
            <p role="status">{count_text}</p>
        </section>

        <section class="card">
            <h2>{move || selected.get().map(|m| m.name).unwrap_or_else(|| t(lang.get(), "selectMonster"))}</h2>
            {move || selected.get().map(|m| {
                let facts = vec![
                    (t(lang.get(), "factLevel"), m.level.clone()),
                    (t(lang.get(), "factRank"), m.rank.clone()),
                    (t(lang.get(), "factCreatureType"), m.kind.clone()),
                    (t(lang.get(), "factEncounter"), m.group.clone().unwrap_or_else(|| t(lang.get(), "mobs"))),
                    (t(lang.get(), "factHp"), t(lang.get(), "unknownServerSide")),
                    (t(lang.get(), "factDefense"), t(lang.get(), "unknownServerSide")),
                ]
                .into_iter()
                .chain(m.spawn_points.map(|n| (t(lang.get(), "factSpawnPoints"), n.to_string())))
                .chain(m.attack.clone().map(|a| {
                    let kind = match a.kind.as_str() {
                        "magic" => t(lang.get(), "attackMagic"),
                        "physical" => t(lang.get(), "attackPhysical"),
                        _ => t(lang.get(), "attackUnknown"),
                    };
                    let summary = format_t(
                        lang.get(),
                        "basicAttackSummary",
                        &[("type", &kind), ("reach", &text_of(&a.reach)), ("interval", &text_of(&a.interval))],
                    );
                    (t(lang.get(), "factBasicAttack"), summary)
                }))
                .collect::<Vec<_>>();

                let details = format!(
                    "NPC {}{}{}",
                    m.id,
                    m.location.as_ref().map(|l| format!(" · {l}")).unwrap_or_default(),
                    m.subtitle.as_ref().map(|s| format!(" · {s}")).unwrap_or_default()
                );
                let source = format!("{}: https://aion2data.com/monsters/{}-{}", t(lang.get(), "source"), m.id, m.slug);
                let skills_title = format_t(lang.get(), "bossSkillsTitle", &[("count", &m.skills.len().to_string())]);
                let skills_view = if m.skills.is_empty() {
                    view! { <p class="import-note">{move || t(lang.get(), "noSkillData")}</p> }.into_any()
                } else {
                    let rows = m.skills.clone().into_iter().map(|s| {
                        let damage = match s.damage.as_str() {
                            "Magic" => t(lang.get(), "attackMagic"),
                            "Physical" => t(lang.get(), "attackPhysical"),
                            other => other.to_string(),
                        };
                        view! { <tr><td>{s.name}</td><td>{damage}</td><td>{s.cast}</td><td>{s.cooldown}</td><td>{s.range}</td></tr> }
                    }).collect_view();
                    view! {
                        <table class="boss-skill-table">
                            <thead><tr>
                                <th>{move || t(lang.get(), "skillName")}</th>
                                <th>{move || t(lang.get(), "skillDamage")}</th>
                                <th>{move || t(lang.get(), "skillCast")}</th>
                                <th>{move || t(lang.get(), "skillCooldown")}</th>
                                <th>{move || t(lang.get(), "skillRange")}</th>
                            </tr></thead>
                            <tbody>{rows}</tbody>
                        </table>
                        <p class="import-note">{move || t(lang.get(), "bossSkillsNote")}</p>
                    }.into_any()
                };

                view! {
                    <div class="boss-facts">
                        {facts.into_iter().map(|(label, value)| view! {
                            <div class="boss-fact"><small>{label}</small><strong>{value}</strong></div>
                        }).collect_view()}
                    </div>
                    <p class="import-note">{details}</p>
                    <h3>{skills_title}</h3>
                    {skills_view}
                    <p class="import-note">{source}</p>
                }
            })}
            <p class="import-note">{move || t(lang.get(), "bossDetailsNote")}</p>
            <button type="button" disabled=move || selected.get().is_none() on:click=import>
                {move || t(lang.get(), "importBossBtn")}
            </button>
        </section>
    }
}
