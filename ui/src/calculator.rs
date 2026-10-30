// Calculator tab: numeric inputs -> build_scenario -> core damage and combat estimate.
use crate::calculator_form::build_scenario;
use crate::state::AppState;
use aion2_core::calculator::{calculate_damage, estimate_combat, Position, TargetMode};
use leptos::prelude::*;
use std::collections::HashMap;

// (raw key, default). Keys match build_scenario; percent fields are entered as percent.
pub const FIELDS: &[(&str, &str)] = &[
    ("pureAttack", "1000"),
    ("weaponDamageBoost", "100"),
    ("weaponDamageTolerance", "0"),
    ("multiHit", "100"),
    ("powerShard", "100"),
    ("attackBonus", "0"),
    ("attackIncrease", "100"),
    ("pveAttackPower", "0"),
    ("bossAttackPower", "0"),
    ("speciesAttackPower", "0"),
    ("frontBackAttackPower", "0"),
    ("skillCoefficient", "100"),
    ("skillAdditionalDamage", "0"),
    ("defense", "0"),
    ("penetration", "0"),
    ("skillSpecialtyEnhancement", "100"),
    ("damageBoost", "100"),
    ("damageTolerance", "0"),
    ("pveDamageBoost", "0"),
    ("pveDamageTolerance", "0"),
    ("speciesDamageBoost", "0"),
    ("speciesDamageTolerance", "0"),
    ("bossDamageBoost", "0"),
    ("bossDamageTolerance", "0"),
    ("criticalDamageBoost", "100"),
    ("criticalDamageTolerance", "0"),
    ("double", "100"),
    ("frontBoost", "100"),
    ("frontDamageTolerance", "0"),
    ("backBoost", "100"),
    ("backDamageTolerance", "0"),
    ("hitsPerSecond", "1"),
    ("enemyHp", "10000"),
];

// Blank or malformed input counts as 0, matching build_scenario's missing-key rule.
fn parse(s: &str) -> f64 {
    s.trim().parse::<f64>().ok().filter(|n| n.is_finite()).unwrap_or(0.0)
}

fn fmt(n: Option<f64>) -> String {
    n.map(|v| format!("{v:.1}")).unwrap_or_else(|| "-".into())
}

fn raw_of(inputs: &[(&'static str, RwSignal<String>)]) -> HashMap<String, f64> {
    inputs.iter().map(|(k, s)| (k.to_string(), parse(&s.get()))).collect()
}

#[component]
pub fn CalculatorTab(state: AppState) -> impl IntoView {
    let inputs = state.inputs.get_value();

    let damage = {
        let inputs = inputs.clone();
        Memo::new(move |_| {
            let s = build_scenario(
                &raw_of(&inputs),
                state.target_mode.get(),
                state.creature_type.get().as_deref(),
                state.critical.get(),
                state.double.get(),
                state.position.get(),
            );
            calculate_damage(&s.values)
        })
    };

    let combat = {
        let inputs = inputs.clone();
        Memo::new(move |_| {
            let r = raw_of(&inputs);
            estimate_combat(
                damage.get(),
                r.get("hitsPerSecond").copied().unwrap_or(0.0),
                r.get("enemyHp").copied().unwrap_or(0.0),
            )
        })
    };

    let inputs_view = inputs
        .into_iter()
        .map(|(k, sig)| {
            view! {
                <label class="calc-field">
                    <span>{k}</span>
                    <input
                        type="text"
                        inputmode="decimal"
                        prop:value=move || sig.get()
                        on:input=move |ev| sig.set(event_target_value(&ev))
                    />
                </label>
            }
        })
        .collect_view();

    view! {
        <section class="calculator">
            <div class="calc-inputs">{inputs_view}</div>
            <div class="calc-toggles">
                <label>
                    "critical"
                    <input type="checkbox" prop:checked=move || state.critical.get()
                        on:change=move |ev| state.critical.set(event_target_checked(&ev)) />
                </label>
                <label>
                    "double"
                    <input type="checkbox" prop:checked=move || state.double.get()
                        on:change=move |ev| state.double.set(event_target_checked(&ev)) />
                </label>
                <label>
                    "position"
                    <select on:change=move |ev| state.position.set(match event_target_value(&ev).as_str() {
                        "front" => Position::Front,
                        "back" => Position::Back,
                        _ => Position::None,
                    })>
                        <option value="none">"none"</option>
                        <option value="front">"front"</option>
                        <option value="back">"back"</option>
                    </select>
                </label>
                <label>
                    "target"
                    <select on:change=move |ev| state.target_mode.set(match event_target_value(&ev).as_str() {
                        "monster" => TargetMode::Monster,
                        "player" => TargetMode::Player,
                        _ => TargetMode::Boss,
                    })>
                        <option value="boss">"boss"</option>
                        <option value="monster">"monster"</option>
                        <option value="player">"player"</option>
                    </select>
                </label>
            </div>
            <output class="calc-result">
                <div>"damage: " {move || format!("{:.1}", damage.get())}</div>
                <div>"dps: " {move || fmt(combat.get().dps)}</div>
                <div>"seconds to defeat: " {move || fmt(combat.get().seconds_to_defeat)}</div>
            </output>
        </section>
    }
}
