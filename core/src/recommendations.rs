use crate::calculator::{calculate_damage, Position, Values};

// A stat that an upgrade candidate raises by one percentage point.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stat {
    WeaponDamageBoost,
    DamageBoost,
    CriticalDamageBoost,
    FrontBackDamageBoost,
}

impl Stat {
    fn slot(self, values: &mut Values) -> &mut f64 {
        match self {
            Stat::WeaponDamageBoost => &mut values.weapon_damage_boost,
            Stat::DamageBoost => &mut values.damage_boost,
            Stat::CriticalDamageBoost => &mut values.critical_damage_boost,
            Stat::FrontBackDamageBoost => &mut values.front_back_damage_boost,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Upgrade {
    pub label: &'static str,
    pub stat: Stat,
    pub percent: f64,
}

// Compares equal +1 percentage-point changes to the stats the formula models, largest gain first.
pub fn rank_damage_upgrades(values: &Values, critical: bool, position: Position) -> Vec<Upgrade> {
    let baseline = calculate_damage(values);
    if !baseline.is_finite() || baseline <= 0.0 {
        return Vec::new();
    }

    let mut candidates = vec![
        ("Weapon Damage Boost", Stat::WeaponDamageBoost),
        ("Damage Boost", Stat::DamageBoost),
    ];
    if critical {
        candidates.push(("Critical Damage Boost", Stat::CriticalDamageBoost));
    }
    match position {
        Position::None => {}
        Position::Front => candidates.push(("Front Attack Damage Boost", Stat::FrontBackDamageBoost)),
        Position::Back => candidates.push(("Back Attack Damage Boost", Stat::FrontBackDamageBoost)),
    }

    let mut upgrades: Vec<Upgrade> = candidates
        .into_iter()
        .filter_map(|(label, stat)| {
            let mut bumped = values.clone();
            *stat.slot(&mut bumped) += 0.01;
            let percent = (calculate_damage(&bumped) - baseline) / baseline * 100.0;
            (percent.is_finite() && percent > 0.0).then_some(Upgrade { label, stat, percent })
        })
        .collect();
    upgrades.sort_by(|a, b| b.percent.total_cmp(&a.percent));
    upgrades
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Values {
        Values { pure_attack: 1000.0, weapon_damage_boost: 1.5, damage_boost: 2.0, ..Values::default() }
    }

    fn labels(upgrades: &[Upgrade]) -> Vec<&'static str> {
        upgrades.iter().map(|item| item.label).collect()
    }

    #[test]
    fn ranks_equal_point_changes_by_current_damage_not_a_fixed_tier_list() {
        let upgrades = rank_damage_upgrades(&base(), false, Position::None);
        assert_eq!(labels(&upgrades), ["Weapon Damage Boost", "Damage Boost"]);
        assert!((upgrades[0].percent - 100.0 / 150.0).abs() < 1e-9);
        assert!((upgrades[1].percent - 0.5).abs() < 1e-9);
    }

    #[test]
    fn includes_critical_and_the_selected_positional_boost_only_when_they_apply() {
        let values = Values { critical_damage_boost: 1.5, front_back_damage_boost: 1.1, ..base() };
        let upgrades = rank_damage_upgrades(&values, true, Position::Back);
        assert_eq!(
            labels(&upgrades),
            ["Back Attack Damage Boost", "Weapon Damage Boost", "Critical Damage Boost", "Damage Boost"]
        );
        assert!((upgrades[0].percent - 100.0 / 110.0).abs() < 1e-9);
    }

    #[test]
    fn omits_positional_and_critical_candidates_when_not_selected() {
        let values = Values { critical_damage_boost: 1.5, front_back_damage_boost: 1.1, ..base() };
        let upgrades = rank_damage_upgrades(&values, false, Position::None);
        assert!(upgrades.iter().all(|item| item.stat != Stat::CriticalDamageBoost && item.stat != Stat::FrontBackDamageBoost));
    }

    #[test]
    fn claims_no_percentage_gains_from_a_non_positive_baseline() {
        assert!(rank_damage_upgrades(&Values { damage_boost: 0.0, ..base() }, false, Position::None).is_empty());
    }
}
