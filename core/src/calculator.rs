// Damage formula ported from calculator.js. Multipliers are ratios (1 = 100%); attack values are absolute.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TargetMode {
    Boss,
    Monster,
    Player,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Position {
    None,
    Front,
    Back,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Values {
    pub pure_attack: f64,
    pub weapon_damage_boost: f64,
    pub weapon_damage_tolerance: f64,
    pub multi_hit: f64,
    pub power_shard: f64,
    pub attack_bonus: f64,
    pub attack_increase: f64,
    pub pve_attack_power: f64,
    pub boss_attack_power: f64,
    pub species_attack_power: f64,
    pub front_back_attack_power: f64,
    pub skill_coefficient: f64,
    pub skill_additional_damage: f64,
    pub defense: f64,
    pub penetration: f64,
    pub defense_scale: f64,
    pub damage_boost: f64,
    pub damage_tolerance: f64,
    pub pve_damage_boost: f64,
    pub pve_damage_tolerance: f64,
    pub species_damage_boost: f64,
    pub species_damage_tolerance: f64,
    pub boss_damage_boost: f64,
    pub boss_damage_tolerance: f64,
    pub critical_damage_boost: f64,
    pub critical_damage_tolerance: f64,
    pub double: f64,
    pub front_back_damage_boost: f64,
    pub front_back_damage_tolerance: f64,
    pub skill_specialty_enhancement: f64,
}

// Defaults match the calculator form's neutral starting values.
impl Default for Values {
    fn default() -> Self {
        Self {
            pure_attack: 1000.0,
            weapon_damage_boost: 1.0,
            weapon_damage_tolerance: 0.0,
            multi_hit: 1.0,
            power_shard: 1.0,
            attack_bonus: 0.0,
            attack_increase: 1.0,
            pve_attack_power: 0.0,
            boss_attack_power: 0.0,
            species_attack_power: 0.0,
            front_back_attack_power: 0.0,
            skill_coefficient: 1.0,
            skill_additional_damage: 0.0,
            defense: 0.0,
            penetration: 0.0,
            defense_scale: 0.1,
            damage_boost: 1.0,
            damage_tolerance: 0.0,
            pve_damage_boost: 0.0,
            pve_damage_tolerance: 0.0,
            species_damage_boost: 0.0,
            species_damage_tolerance: 0.0,
            boss_damage_boost: 0.0,
            boss_damage_tolerance: 0.0,
            critical_damage_boost: 1.0,
            critical_damage_tolerance: 0.0,
            double: 1.0,
            front_back_damage_boost: 1.0,
            front_back_damage_tolerance: 0.0,
            skill_specialty_enhancement: 1.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Combat {
    pub dps: Option<f64>,
    pub seconds_to_defeat: Option<f64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DuelKind {
    Multiplier,
    Defense,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DuelRow {
    pub id: &'static str,
    pub label_key: &'static str,
    pub kind: DuelKind,
    pub you: f64,
    pub target: f64,
    pub net: f64,
}

// Returns values with the stats that do not apply to the target mode zeroed and the mode's defense scale set.
pub fn apply_target_mode(values: &Values, mode: TargetMode) -> Values {
    let mut selected = values.clone();
    if mode == TargetMode::Player {
        selected.pve_attack_power = 0.0;
        selected.pve_damage_boost = 0.0;
        selected.pve_damage_tolerance = 0.0;
    }
    if mode != TargetMode::Boss {
        selected.boss_attack_power = 0.0;
        selected.boss_damage_boost = 0.0;
        selected.boss_damage_tolerance = 0.0;
    }
    if mode == TargetMode::Player {
        selected.species_attack_power = 0.0;
        selected.species_damage_boost = 0.0;
        selected.species_damage_tolerance = 0.0;
    }
    selected.defense_scale = if mode == TargetMode::Player { 0.01 } else { 0.1 };
    if mode == TargetMode::Player && selected.double > 1.0 {
        selected.double = 1.0 + (selected.double - 1.0) * 0.5;
    }
    if mode == TargetMode::Player && selected.critical_damage_boost > 1.0 {
        selected.critical_damage_boost = (selected.critical_damage_boost - 0.25).max(1.0);
    }
    selected
}

// One landed hit after mitigation and every multiplier stage, clamped at zero per stage.
pub fn calculate_damage(v: &Values) -> f64 {
    let weapon_multiplier = (v.weapon_damage_boost - v.weapon_damage_tolerance).max(0.0);
    let attack = (v.pure_attack * weapon_multiplier * v.multi_hit * v.power_shard + v.attack_bonus)
        * v.attack_increase
        + v.pve_attack_power
        + v.boss_attack_power
        + v.species_attack_power
        + v.front_back_attack_power;
    let skill = attack * v.skill_coefficient + v.skill_additional_damage;
    let mitigated = (skill - (v.defense - v.penetration).max(0.0) * v.defense_scale).max(0.0);
    let damage_multiplier = (v.damage_boost + v.pve_damage_boost + v.species_damage_boost + v.boss_damage_boost
        - v.damage_tolerance
        - v.pve_damage_tolerance
        - v.boss_damage_tolerance
        - v.species_damage_tolerance)
        .max(0.0);
    let critical_multiplier = (v.critical_damage_boost - v.critical_damage_tolerance).max(0.0);
    let positional_multiplier = (v.front_back_damage_boost - v.front_back_damage_tolerance).max(0.0);
    mitigated * damage_multiplier * critical_multiplier * v.double * positional_multiplier * v.skill_specialty_enhancement
}

// DPS and time-to-defeat; both need a positive hit rate and damage, and time also needs a known target HP.
pub fn estimate_combat(damage: f64, hits_per_second: f64, target_hp: f64) -> Combat {
    let raw = (damage > 0.0 && hits_per_second > 0.0).then(|| damage * hits_per_second);
    let dps = raw.filter(|value| value.is_finite());
    let seconds_to_defeat = match raw {
        Some(value) if value > 0.0 && target_hp > 0.0 => Some(target_hp / value),
        _ => None,
    };
    Combat { dps, seconds_to_defeat }
}

// Keys describing what the displayed hit and estimate take for granted. The hit itself is deterministic;
// everything probabilistic (crit, procs, position uptime, cadence) is assumed, not modelled.
pub fn estimate_assumptions(hits_per_second: f64, critical: bool, double: bool, position: Position) -> Vec<&'static str> {
    let mut keys = Vec::new();
    if critical {
        keys.push("assumeCritLands");
    }
    if double {
        keys.push("assumeDoubleProcs");
    }
    keys.push(if position == Position::None { "positionComparisonHint" } else { "positionUptimeHint" });
    if hits_per_second > 0.0 {
        keys.push("assumeConstantRate");
    }
    keys.push("resistNotModelled");
    keys.push("roundingNotModelled");
    keys
}

// Pairs each attacker stat with the target stat it meets, using the same clamping as calculate_damage.
// Multiplier rows report the net ratio (1 = 100%); the defense row reports absolute defense left after penetration.
pub fn attribute_duels(v: &Values, mode: TargetMode, critical: bool, position: Position) -> Vec<DuelRow> {
    let multiplier = |id, label_key, you: f64, target: f64, active: bool| {
        active.then(|| DuelRow { id, label_key, kind: DuelKind::Multiplier, you, target, net: (you - target).max(0.0) })
    };
    [
        multiplier("weapon", "duelWeapon", v.weapon_damage_boost, v.weapon_damage_tolerance, true),
        Some(DuelRow {
            id: "penetration",
            label_key: "duelPenetration",
            kind: DuelKind::Defense,
            you: v.penetration,
            target: v.defense,
            net: (v.defense - v.penetration).max(0.0),
        }),
        multiplier("damage", "duelDamage", v.damage_boost, v.damage_tolerance, true),
        multiplier("pve", "duelPve", v.pve_damage_boost, v.pve_damage_tolerance, mode != TargetMode::Player),
        multiplier("boss", "duelBoss", v.boss_damage_boost, v.boss_damage_tolerance, mode == TargetMode::Boss),
        multiplier("species", "duelSpecies", v.species_damage_boost, v.species_damage_tolerance, mode != TargetMode::Player),
        multiplier("critical", "duelCritical", v.critical_damage_boost, v.critical_damage_tolerance, critical),
        multiplier("position", "duelPosition", v.front_back_damage_boost, v.front_back_damage_tolerance, position != Position::None),
    ]
    .into_iter()
    .flatten()
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Values {
        Values {
            pure_attack: 1000.0,
            weapon_damage_boost: 1.0,
            multi_hit: 1.0,
            power_shard: 1.0,
            attack_increase: 1.0,
            skill_coefficient: 1.0,
            damage_boost: 1.0,
            critical_damage_boost: 1.0,
            double: 1.0,
            front_back_damage_boost: 1.0,
            skill_specialty_enhancement: 1.0,
            ..Values::default()
        }
    }

    fn close(actual: f64, expected: f64) -> bool {
        (actual - expected).abs() < 1e-9
    }

    #[test]
    fn combines_attack_boosts_skill_mitigation_and_multipliers_in_order() {
        let values = Values {
            pure_attack: 200.0,
            weapon_damage_boost: 1.5,
            multi_hit: 2.0,
            power_shard: 1.25,
            attack_bonus: 50.0,
            attack_increase: 1.1,
            pve_attack_power: 10.0,
            boss_attack_power: 20.0,
            species_attack_power: 30.0,
            front_back_attack_power: 40.0,
            skill_coefficient: 2.0,
            skill_additional_damage: 15.0,
            defense: 300.0,
            penetration: 100.0,
            damage_boost: 1.1,
            pve_damage_boost: 0.2,
            species_damage_boost: 0.3,
            critical_damage_boost: 1.5,
            double: 2.0,
            front_back_damage_boost: 1.2,
            skill_specialty_enhancement: 1.1,
            ..base()
        };
        // ((200*1.5*2*1.25+50)*1.1+10+20+30+40)*2+15-(300-100)*0.1 = 1955
        assert!(close(calculate_damage(&values), 1955.0 * 1.6 * 1.5 * 2.0 * 1.2 * 1.1));
    }

    #[test]
    fn penetration_cannot_reduce_effective_defense_below_zero() {
        let values = Values { defense: 50.0, penetration: 150.0, ..base() };
        assert_eq!(calculate_damage(&values), 1000.0);
    }

    #[test]
    fn enemy_tolerances_reduce_their_matching_stages_without_changing_neutral_defaults() {
        let values = Values {
            weapon_damage_boost: 1.5,
            critical_damage_boost: 1.5,
            front_back_damage_boost: 1.2,
            pve_damage_boost: 0.3,
            species_damage_boost: 0.2,
            weapon_damage_tolerance: 0.1,
            damage_tolerance: 0.1,
            pve_damage_tolerance: 0.05,
            species_damage_tolerance: 0.05,
            critical_damage_tolerance: 0.2,
            front_back_damage_tolerance: 0.1,
            ..base()
        };
        assert!(close(calculate_damage(&values), 1400.0 * 1.3 * 1.3 * 1.1));
        assert_eq!(calculate_damage(&base()), 1000.0);
    }

    #[test]
    fn switching_target_mode_excludes_inapplicable_terms_without_losing_inputs() {
        let values = Values {
            pve_attack_power: 100.0,
            boss_attack_power: 200.0,
            species_attack_power: 300.0,
            pve_damage_boost: 0.1,
            species_damage_boost: 0.2,
            pve_damage_tolerance: 0.05,
            species_damage_tolerance: 0.02,
            ..base()
        };
        assert!(close(calculate_damage(&apply_target_mode(&values, TargetMode::Boss)), 1600.0 * 1.23));
        assert!(close(calculate_damage(&apply_target_mode(&values, TargetMode::Monster)), 1400.0 * 1.23));
        assert_eq!(calculate_damage(&apply_target_mode(&values, TargetMode::Player)), 1000.0);
    }

    #[test]
    fn boss_boost_and_tolerance_only_apply_to_bosses() {
        let values = Values { boss_damage_boost: 0.3, boss_damage_tolerance: 0.1, ..base() };
        assert_eq!(calculate_damage(&apply_target_mode(&values, TargetMode::Boss)), 1200.0);
        assert_eq!(calculate_damage(&apply_target_mode(&values, TargetMode::Monster)), 1000.0);
        assert_eq!(calculate_damage(&apply_target_mode(&values, TargetMode::Player)), 1000.0);
    }

    #[test]
    fn pvp_defense_scales_by_one_percent_and_double_proc_has_half_the_pve_bonus() {
        let values = Values { defense: 100.0, double: 2.0, ..base() };
        assert_eq!(calculate_damage(&apply_target_mode(&values, TargetMode::Boss)), 1980.0);
        assert_eq!(calculate_damage(&apply_target_mode(&values, TargetMode::Player)), 1498.5);
        let penetrating = Values { penetration: 200.0, ..values };
        assert_eq!(calculate_damage(&apply_target_mode(&penetrating, TargetMode::Player)), 1500.0);
    }

    #[test]
    fn pvp_critical_hits_use_the_1_25_base_multiplier_instead_of_pve_1_5() {
        let values = Values { critical_damage_boost: 1.5, ..base() };
        assert_eq!(calculate_damage(&apply_target_mode(&values, TargetMode::Boss)), 1500.0);
        assert_eq!(calculate_damage(&apply_target_mode(&values, TargetMode::Player)), 1250.0);
    }

    #[test]
    fn damage_stages_cannot_turn_a_landed_hit_into_negative_damage() {
        assert_eq!(calculate_damage(&Values { defense: 20000.0, ..base() }), 0.0);
        assert_eq!(calculate_damage(&Values { damage_tolerance: 2.0, ..base() }), 0.0);
        assert_eq!(calculate_damage(&Values { critical_damage_tolerance: 2.0, ..base() }), 0.0);
    }

    #[test]
    fn dps_and_defeat_time_need_a_positive_hit_rate_and_actual_target_hp() {
        assert_eq!(estimate_combat(1200.0, 2.5, 9000.0), Combat { dps: Some(3000.0), seconds_to_defeat: Some(3.0) });
        assert_eq!(estimate_combat(1200.0, 2.5, 0.0), Combat { dps: Some(3000.0), seconds_to_defeat: None });
        assert_eq!(estimate_combat(1200.0, 0.0, 9000.0), Combat { dps: None, seconds_to_defeat: None });
        assert_eq!(estimate_combat(-100.0, 2.0, 9000.0), Combat { dps: None, seconds_to_defeat: None });
    }

    #[test]
    fn assumptions_name_only_the_conditions_the_toggles_take_for_granted() {
        assert_eq!(
            estimate_assumptions(0.0, false, false, Position::None),
            vec!["positionComparisonHint", "resistNotModelled", "roundingNotModelled"]
        );
        assert_eq!(
            estimate_assumptions(2.0, true, true, Position::Back),
            vec![
                "assumeCritLands",
                "assumeDoubleProcs",
                "positionUptimeHint",
                "assumeConstantRate",
                "resistNotModelled",
                "roundingNotModelled",
            ]
        );
    }

    #[test]
    fn duels_clamp_each_pair_at_zero_and_show_penetration_against_defense() {
        let values = Values { weapon_damage_tolerance: 1.5, defense: 300.0, penetration: 100.0, ..base() };
        let rows = attribute_duels(&values, TargetMode::Boss, false, Position::None);
        let weapon = rows.iter().find(|row| row.id == "weapon").unwrap();
        let penetration = rows.iter().find(|row| row.id == "penetration").unwrap();
        assert_eq!(weapon.net, 0.0);
        assert_eq!(penetration.net, 200.0);
        let over = Values { defense: 100.0, penetration: 500.0, ..base() };
        let rows = attribute_duels(&over, TargetMode::Boss, false, Position::None);
        assert_eq!(rows.iter().find(|row| row.id == "penetration").unwrap().net, 0.0);
    }

    #[test]
    fn duels_show_only_the_target_type_rows_that_apply_to_the_mode() {
        let ids = |mode| -> Vec<&'static str> {
            attribute_duels(&base(), mode, false, Position::None).iter().map(|row| row.id).collect()
        };
        let player = ids(TargetMode::Player);
        assert!(!player.contains(&"pve") && !player.contains(&"boss") && !player.contains(&"species"));
        assert!(ids(TargetMode::Boss).contains(&"boss"));
        assert!(!ids(TargetMode::Monster).contains(&"boss"));
        assert!(ids(TargetMode::Monster).contains(&"species"));
    }

    #[test]
    fn duels_include_critical_and_positional_rows_only_when_in_play() {
        let off: Vec<&str> = attribute_duels(&base(), TargetMode::Boss, false, Position::None).iter().map(|row| row.id).collect();
        let on: Vec<&str> = attribute_duels(&base(), TargetMode::Boss, true, Position::Back).iter().map(|row| row.id).collect();
        assert!(!off.contains(&"critical") && !off.contains(&"position"));
        assert!(on.contains(&"critical") && on.contains(&"position"));
    }
}
