// Maps raw calculator form numbers to the core Values, mirroring renderer.js update().

use aion2_core::calculator::{apply_target_mode, Position, TargetMode, Values};
use std::collections::HashMap;

// Fields the form stores as percent; the core expects ratios.
const PERCENT_FIELDS: &[&str] = &[
    "weaponDamageBoost", "multiHit", "powerShard", "attackIncrease", "skillCoefficient",
    "damageBoost", "pveDamageBoost", "bossDamageBoost", "bossDamageTolerance", "speciesDamageBoost",
    "criticalDamageBoost", "weaponDamageTolerance", "damageTolerance", "pveDamageTolerance",
    "speciesDamageTolerance", "criticalDamageTolerance", "frontDamageTolerance", "backDamageTolerance",
    "double", "frontBoost", "backBoost", "skillSpecialtyEnhancement",
];

pub struct Scenario {
    pub values: Values,
    pub mode: TargetMode,
    pub critical: bool,
    pub position: Position,
}

// `raw` holds parsed form numbers keyed by input name; missing names count as 0.
pub fn build_scenario(
    raw: &HashMap<String, f64>,
    mode: TargetMode,
    creature: Option<&str>,
    critical: bool,
    double_enabled: bool,
    position: Position,
) -> Scenario {
    let get = |name: &str| raw.get(name).copied().unwrap_or(0.0);
    let scaled = |name: &str| {
        let n = get(name);
        if PERCENT_FIELDS.contains(&name) { n / 100.0 } else { n }
    };
    let mut v = Values {
        pure_attack: get("pureAttack"),
        weapon_damage_boost: scaled("weaponDamageBoost"),
        weapon_damage_tolerance: scaled("weaponDamageTolerance"),
        multi_hit: scaled("multiHit"),
        power_shard: scaled("powerShard"),
        attack_bonus: get("attackBonus"),
        attack_increase: scaled("attackIncrease"),
        pve_attack_power: get("pveAttackPower"),
        boss_attack_power: get("bossAttackPower"),
        species_attack_power: get("speciesAttackPower"),
        front_back_attack_power: get("frontBackAttackPower"),
        skill_coefficient: scaled("skillCoefficient"),
        skill_additional_damage: get("skillAdditionalDamage"),
        defense: get("defense"),
        penetration: get("penetration"),
        skill_specialty_enhancement: scaled("skillSpecialtyEnhancement"),
        damage_boost: scaled("damageBoost"),
        damage_tolerance: scaled("damageTolerance"),
        pve_damage_boost: scaled("pveDamageBoost"),
        pve_damage_tolerance: scaled("pveDamageTolerance"),
        species_damage_boost: scaled("speciesDamageBoost"),
        species_damage_tolerance: scaled("speciesDamageTolerance"),
        boss_damage_boost: scaled("bossDamageBoost"),
        boss_damage_tolerance: scaled("bossDamageTolerance"),
        critical_damage_boost: scaled("criticalDamageBoost"),
        critical_damage_tolerance: scaled("criticalDamageTolerance"),
        double: scaled("double"),
        ..Values::default()
    };
    // Creature-type inputs are generated per type and stored raw (not in PERCENT_FIELDS).
    if let Some(t) = creature {
        v.species_damage_boost = get(&format!("{t}DamageBoost")) / 100.0;
        v.species_damage_tolerance = get(&format!("{t}DamageTolerance")) / 100.0;
    }
    if !critical {
        v.critical_damage_boost = 1.0;
        v.critical_damage_tolerance = 0.0;
    }
    if !double_enabled {
        v.double = 1.0;
    }
    let (fb_boost, fb_tol) = match position {
        Position::Front => (scaled("frontBoost"), scaled("frontDamageTolerance")),
        Position::Back => (scaled("backBoost"), scaled("backDamageTolerance")),
        Position::None => (1.0, 0.0),
    };
    v.front_back_damage_boost = fb_boost;
    v.front_back_damage_tolerance = fb_tol;
    let values = apply_target_mode(&v, mode);
    Scenario { values, mode, critical, position }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn raw(pairs: &[(&str, f64)]) -> HashMap<String, f64> {
        pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect()
    }

    #[test]
    fn percent_fields_become_ratios() {
        let s = build_scenario(&raw(&[("weaponDamageBoost", 150.0)]), TargetMode::Boss, None, false, false, Position::None);
        assert!((s.values.weapon_damage_boost - 1.5).abs() < 1e-12);
    }

    #[test]
    fn creature_type_boost_replaces_species_once_scaled() {
        let r = raw(&[("cogniDamageBoost", 20.0), ("cogniDamageTolerance", 10.0)]);
        let s = build_scenario(&r, TargetMode::Monster, Some("cogni"), false, false, Position::None);
        assert!((s.values.species_damage_boost - 0.2).abs() < 1e-12);
        assert!((s.values.species_damage_tolerance - 0.1).abs() < 1e-12);
    }

    #[test]
    fn inactive_critical_and_double_are_neutral() {
        let r = raw(&[("criticalDamageBoost", 150.0), ("criticalDamageTolerance", 30.0), ("double", 200.0)]);
        let s = build_scenario(&r, TargetMode::Boss, None, false, false, Position::None);
        assert_eq!(s.values.critical_damage_boost, 1.0);
        assert_eq!(s.values.critical_damage_tolerance, 0.0);
        assert_eq!(s.values.double, 1.0);
    }

    #[test]
    fn position_selects_front_or_back_terms_only() {
        let r = raw(&[("frontBoost", 120.0), ("backBoost", 130.0)]);
        let front = build_scenario(&r, TargetMode::Boss, None, false, false, Position::Front);
        assert!((front.values.front_back_damage_boost - 1.2).abs() < 1e-12);
        let none = build_scenario(&r, TargetMode::Boss, None, false, false, Position::None);
        assert_eq!(none.values.front_back_damage_boost, 1.0);
    }
}
