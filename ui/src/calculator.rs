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
    ("criticalDamageBoost", "150"),
    ("criticalDamageTolerance", "0"),
    ("double", "200"),
    ("frontBoost", "100"),
    ("frontDamageTolerance", "0"),
    ("backBoost", "100"),
    ("backDamageTolerance", "0"),
    ("hitsPerSecond", "0"),
    ("enemyHp", "0"),
    ("playerHp", "0"),
    ("pveAccuracy", "0"),
    ("elementalResistance", "0"),
    ("accuracy", "0"),
    ("evasion", "0"),
    ("blockPenetration", "0"),
    ("block", "0"),
    ("pvpAccuracy", "0"),
    ("pvpEvasion", "0"),
    ("pvpBlockPenetration", "0"),
    ("pvpBlock", "0"),
    ("criticalHit", "0"),
    ("criticalHitResist", "0"),
    ("frontCriticalHit", "0"),
    ("frontCriticalHitResist", "0"),
    ("backCriticalHit", "0"),
    ("backCriticalHitResist", "0"),
    ("criticalAttack", "0"),
    ("criticalDamageDefense", "0"),
    ("frontAttack", "0"),
    ("frontDefense", "0"),
    ("backAttack", "0"),
    ("backDefense", "0"),
    ("pvpCriticalHit", "0"),
    ("pvpCriticalHitResist", "0"),
    ("doubleChance", "0"),
    ("doubleResist", "0"),
    ("perfectChance", "0"),
    ("perfectResist", "0"),
    ("multiHitChance", "0"),
    ("multiHitResist", "0"),
    ("endurancePenetration", "0"),
    ("endurance", "0"),
    ("regenerationPenetration", "0"),
    ("regeneration", "0"),
    ("pvpDamageBoost", "0"),
    ("pvpDamageTolerance", "0"),
    ("pvpAttack", "0"),
    ("pvpDefense", "0"),
    ("fireBoost", "0"),
    ("fireTolerance", "0"),
    ("waterBoost", "0"),
    ("waterTolerance", "0"),
    ("windBoost", "0"),
    ("windTolerance", "0"),
    ("earthBoost", "0"),
    ("earthTolerance", "0"),
    ("cogniDamageBoost", "0"),
    ("cogniDamageTolerance", "0"),
    ("cogniAttack", "0"),
    ("cogniDefense", "0"),
    ("cogniAccuracy", "0"),
    ("cogniEvasion", "0"),
    ("cogniBlockPenetration", "0"),
    ("cogniBlock", "0"),
    ("cogniCriticalHit", "0"),
    ("cogniCriticalHitResist", "0"),
    ("feraDamageBoost", "0"),
    ("feraDamageTolerance", "0"),
    ("feraAttack", "0"),
    ("feraDefense", "0"),
    ("feraAccuracy", "0"),
    ("feraEvasion", "0"),
    ("feraBlockPenetration", "0"),
    ("feraBlock", "0"),
    ("feraCriticalHit", "0"),
    ("feraCriticalHitResist", "0"),
    ("naturaDamageBoost", "0"),
    ("naturaDamageTolerance", "0"),
    ("naturaAttack", "0"),
    ("naturaDefense", "0"),
    ("naturaAccuracy", "0"),
    ("naturaEvasion", "0"),
    ("naturaBlockPenetration", "0"),
    ("naturaBlock", "0"),
    ("naturaCriticalHit", "0"),
    ("naturaCriticalHitResist", "0"),
    ("varianDamageBoost", "0"),
    ("varianDamageTolerance", "0"),
    ("varianAttack", "0"),
    ("varianDefense", "0"),
    ("varianAccuracy", "0"),
    ("varianEvasion", "0"),
    ("varianBlockPenetration", "0"),
    ("varianBlock", "0"),
    ("varianCriticalHit", "0"),
    ("varianCriticalHitResist", "0"),
];

// Blank or malformed input counts as 0, matching build_scenario's missing-key rule.
fn parse(s: &str) -> f64 {
    aion2_core::numeric_input::parse_numeric_input(s, 0.0).unwrap_or(0.0)
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
                    <span>{move || aion2_core::stat_info::stat_info(k, state.lang.get() == crate::i18n::Lang::Pt).map(|i| crate::i18n::stat_label(state.lang.get(), &i.title)).unwrap_or_else(|| k.to_owned())}</span>
                    <input
                        type="text"
                        inputmode="decimal"
                        name=k
                        title=move || aion2_core::stat_info::stat_info(k, state.lang.get() == crate::i18n::Lang::Pt).map(|i| i.desc).unwrap_or_default()
                        prop:value=move || sig.get()
                        on:input=move |ev| sig.set(event_target_value(&ev))
                    />
                </label>
            }
        })
        .collect_view();

    view! {
        <section class="calculator">
            <div class="combatants">
                <label>{move || crate::i18n::t(state.lang.get(), "charNameLabel")}
                    <input id="attacker-name" prop:value=move || state.attacker_name.get() on:input=move |ev| state.attacker_name.set(event_target_value(&ev)) />
                </label>
                <label>{move || crate::i18n::t(state.lang.get(), "classLabel")}
                    <input id="attacker-class" prop:value=move || state.attacker_class.get() on:input=move |ev| state.attacker_class.set(event_target_value(&ev)) />
                </label>
                <label>{move || crate::i18n::t(state.lang.get(), "targetNameLabel")}
                    <input id="target-name" prop:value=move || state.target_name.get() on:input=move |ev| state.target_name.set(event_target_value(&ev)) />
                </label>
                <label>{move || crate::i18n::t(state.lang.get(), "creatureTypeLabel")}
                    <select prop:value=move || state.creature_type.get().unwrap_or_default() on:change=move |ev| {
                        let value = event_target_value(&ev);
                        state.creature_type.set(if value.is_empty() { None } else { Some(value) });
                    }>
                        <option value="">{move || crate::i18n::t(state.lang.get(), "creatureManualOption")}</option>
                        <option value="cogni">"Cogni"</option><option value="fera">"Fera"</option>
                        <option value="natura">"Natura"</option><option value="varian">"Varian"</option>
                    </select>
                </label>
            </div>
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
