use aion2_core::calculator::{Position, TargetMode, Values};
use crate::calculator::FIELDS;
use crate::i18n::Lang;
use leptos::prelude::*;
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ActiveTab {
    #[default]
    Calculator,
    Character,
    Enemy,
    Boss,
    Attributes,
    Timers,
}

impl ActiveTab {
    pub fn id(self) -> &'static str {
        match self {
            ActiveTab::Calculator => "calculator-view",
            ActiveTab::Character => "character-view",
            ActiveTab::Enemy => "enemy-view",
            ActiveTab::Boss => "boss-view",
            ActiveTab::Attributes => "attributes-view",
            ActiveTab::Timers => "timers-view",
        }
    }
    pub fn tab_btn_id(self) -> &'static str {
        match self {
            ActiveTab::Calculator => "calculator-tab",
            ActiveTab::Character => "character-tab",
            ActiveTab::Enemy => "enemy-tab",
            ActiveTab::Boss => "boss-tab",
            ActiveTab::Attributes => "attributes-tab",
            ActiveTab::Timers => "timers-tab",
        }
    }


    pub fn i18n_key(self) -> &'static str {
        match self {
            ActiveTab::Calculator => "tabCalculator",
            ActiveTab::Character => "tabCharacter",
            ActiveTab::Enemy => "tabEnemy",
            ActiveTab::Boss => "tabBoss",
            ActiveTab::Attributes => "tabAttributes",
            ActiveTab::Timers => "tabTimers",
        }
    }
}

pub const ALL_TABS: [ActiveTab; 6] = [
    ActiveTab::Calculator,
    ActiveTab::Character,
    ActiveTab::Enemy,
    ActiveTab::Boss,
    ActiveTab::Attributes,
    ActiveTab::Timers,
];

#[derive(Clone, Copy, Debug)]
pub struct AppState {
    pub lang: RwSignal<Lang>,
    pub tab: RwSignal<ActiveTab>,
    pub values: RwSignal<Values>,
    pub target_mode: RwSignal<TargetMode>,
    pub critical: RwSignal<bool>,
    pub double: RwSignal<bool>,
    pub position: RwSignal<Position>,
    pub hits_per_second: RwSignal<f64>,
    pub target_hp: RwSignal<Option<f64>>,
    pub creature_type: RwSignal<Option<String>>,
    pub attacker_name: RwSignal<String>,
    pub attacker_class: RwSignal<String>,
    pub target_name: RwSignal<String>,
    /// Calculator inputs keyed by form name; shared so the Character import can write them.
    pub inputs: StoredValue<Vec<(&'static str, RwSignal<String>)>>,
    /// Values written by the last character import, so a re-import can revert stale ones.
    pub import_previous: RwSignal<BTreeMap<String, f64>>,
    pub imported_profile: RwSignal<Option<serde_json::Value>>,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            lang: RwSignal::new(Lang::default()),
            tab: RwSignal::new(ActiveTab::default()),
            values: RwSignal::new(Values::default()),
            target_mode: RwSignal::new(TargetMode::Boss),
            critical: RwSignal::new(false),
            double: RwSignal::new(false),
            position: RwSignal::new(Position::None),
            hits_per_second: RwSignal::new(1.0),
            target_hp: RwSignal::new(None),
            creature_type: RwSignal::new(None),
            attacker_name: RwSignal::new(String::new()),
            attacker_class: RwSignal::new(String::new()),
            target_name: RwSignal::new(String::new()),
            inputs: StoredValue::new(
                FIELDS.iter().map(|(k, d)| (*k, RwSignal::new(d.to_string()))).collect(),
            ),
            import_previous: RwSignal::new(BTreeMap::new()),
            imported_profile: RwSignal::new(None),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
