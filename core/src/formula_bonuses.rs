// Port of collectFormulaBonuses in characters.js: sums gear substats and sheet attack-increase
// effects into calculator fields, each offset by its baseline and rounded to 4 decimals.

use regex::Regex;
use serde_json::Value;
use std::collections::BTreeMap;
use std::sync::LazyLock;

// Gear option id -> display name; ids are stable across client languages.
const GEAR_OPTION_NAMES: [(&str, &str); 20] = [
    ("WeaponFixingDamage", "Attack"),
    ("WeaponAccuracy", "Accuracy"),
    ("Critical", "Critical Hit"),
    ("Block", "Block"),
    ("ArmorDefense", "Defense"),
    ("ArmorEvasion", "Evasion"),
    ("CriticalResist", "Critical Hit Resist"),
    ("AmplifyAllDamage", "Damage Boost"),
    ("AmplifyWeaponDamage", "Weapon Damage Boost"),
    ("PvEAmplifyDamage", "PvE Damage Boost"),
    ("PvEDecreaseDamage", "PvE Damage Tolerance"),
    ("DefensePierce", "Penetration"),
    ("AdditionalHitRate", "Multi-hit Chance"),
    ("Perfect", "Perfect Chance"),
    ("CriticalAddDamage", "Critical Attack"),
    ("FrontAttackDefense", "Front Defense"),
    ("Restoration", "Regeneration"),
    ("HPMax", "HP"),
    ("MPMax", "MP"),
    ("SealStoneAddDamage", "Power Shard Damage Bonus"),
];

// Primary stats and divinities are already in the sheet totals, so gear never sums them.
const SHEET_ONLY_IDS: [&str; 16] = [
    "STR", "DEX", "CON", "INT", "AGI", "WIS",
    "Justice", "Freedom", "Illusion", "Life", "Time", "Destruction", "Death", "Wisdom", "Destiny", "Space",
];

// Display name -> (calculator field, baseline, percentage input). Order matches characters.js.
const BONUS_FIELDS: [(&str, &str, f64, bool); 14] = [
    ("Attack Bonus", "attackBonus", 0.0, false),
    ("PvE Attack", "pveAttackPower", 0.0, false),
    ("PvE Attack Power", "pveAttackPower", 0.0, false),
    ("Boss Attack", "bossAttackPower", 0.0, false),
    ("Penetration", "penetration", 0.0, false),
    ("Boss Attack Power", "bossAttackPower", 0.0, false),
    ("Weapon Damage Boost", "weaponDamageBoost", 100.0, true),
    ("Damage Boost", "damageBoost", 100.0, true),
    ("PvE Damage Boost", "pveDamageBoost", 0.0, true),
    ("Boss Damage Boost", "bossDamageBoost", 0.0, true),
    ("Attack Increase", "attackIncrease", 100.0, true),
    ("Critical Damage Boost", "criticalDamageBoost", 150.0, true),
    ("Front Attack Damage Boost", "frontBoost", 100.0, true),
    ("Back Attack Damage Boost", "backBoost", 100.0, true),
];

fn bonus_by_name(name: &str) -> Option<(&'static str, f64, bool)> {
    BONUS_FIELDS.iter().find(|(n, ..)| *n == name).map(|(_, f, b, p)| (*f, *b, *p))
}

fn baseline_of(field: &str) -> f64 {
    BONUS_FIELDS.iter().find(|(_, f, ..)| *f == field).map_or(0.0, |(_, _, b, _)| *b)
}

// Sheet stats whose "Attack increase" effect feeds attackIncrease (value / 10 = percent).
const ATTACK_INCREASE_TYPES: [&str; 2] = ["STR", "Destruction"];

static PERCENT_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\d+(?:\.\d+)?%$").unwrap());
static PLAIN_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\d+(?:\.\d+)?$").unwrap());

fn text_of(v: Option<&Value>) -> String {
    match v {
        Some(Value::String(s)) => s.trim().to_string(),
        Some(Value::Number(n)) => n.to_string(),
        _ => String::new(),
    }
}

fn option_field(option: &Value) -> Option<(&'static str, bool)> {
    let id = option.get("id").and_then(Value::as_str).unwrap_or("");
    if SHEET_ONLY_IDS.contains(&id) {
        return None;
    }
    let name = GEAR_OPTION_NAMES.iter().find(|(k, _)| *k == id).map(|(_, v)| *v)?;
    bonus_by_name(name).map(|(f, _, p)| (f, p))
}

// Returns calculator field values and, per field, whether it came from "gear", "sheet" or "sheet+gear".
pub fn collect_formula_bonuses(profile: &Value) -> (BTreeMap<String, f64>, BTreeMap<String, &'static str>) {
    let mut sums: BTreeMap<String, f64> = BTreeMap::new();
    let mut sources: BTreeMap<String, &'static str> = BTreeMap::new();

    let equipped = profile
        .pointer("/equipment/equipment/equipmentList")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    for item in &equipped {
        let slot_name = item.get("slotPosName").and_then(Value::as_str).unwrap_or("");
        if slot_name.starts_with("Arcana") {
            continue;
        }
        let slot = text_of(item.get("slotPos"));
        let Some(detail) = profile.get("itemDetails").and_then(|d| d.get(&slot)) else { continue };
        let options = ["mainStats", "subStats"]
            .iter()
            .filter_map(|key| detail.get(*key).and_then(Value::as_array))
            .flatten();
        for option in options {
            let Some((field, percentage)) = option_field(option) else { continue };
            let text = text_of(option.get("value"));
            let valid = if percentage { PERCENT_RE.is_match(&text) } else { PLAIN_RE.is_match(&text) };
            if !valid {
                continue;
            }
            let number: f64 = text.trim_end_matches('%').parse().unwrap_or(0.0);
            *sums.entry(field.to_string()).or_insert(0.0) += number;
            sources.insert(field.to_string(), "gear");
        }
    }

    let stats = profile.pointer("/info/stat/statList").and_then(Value::as_array).cloned().unwrap_or_default();
    for stat in &stats {
        let kind = stat.get("type").and_then(Value::as_str).unwrap_or("");
        if !ATTACK_INCREASE_TYPES.contains(&kind) {
            continue;
        }
        let amount: f64 = text_of(stat.get("value")).parse().unwrap_or(0.0);
        if !amount.is_finite() || amount <= 0.0 {
            continue;
        }
        *sums.entry("attackIncrease".to_string()).or_insert(0.0) += amount / 10.0;
        let merged = matches!(sources.get("attackIncrease"), Some(&"gear") | Some(&"sheet+gear"));
        sources.insert("attackIncrease".to_string(), if merged { "sheet+gear" } else { "sheet" });
    }

    let values = sums
        .into_iter()
        .map(|(name, amount)| {
            let baseline = baseline_of(&name);
            (name, ((baseline + amount) * 10000.0).round() / 10000.0)
        })
        .collect();
    (values, sources)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn weapon(options: Value) -> Value {
        json!({
            "equipment": { "equipment": { "equipmentList": [ { "slotPos": 1, "slotPosName": "Weapon" } ] } },
            "itemDetails": { "1": options }
        })
    }

    #[test]
    fn sums_gear_percent_and_applies_baseline() {
        let profile = weapon(json!({ "mainStats": [ { "id": "AmplifyAllDamage", "value": "10%" } ],
            "subStats": [ { "id": "AmplifyAllDamage", "value": "2.5%" } ] }));
        let (values, sources) = collect_formula_bonuses(&profile);
        assert_eq!(values["damageBoost"], 112.5);
        assert_eq!(sources["damageBoost"], "gear");
    }

    #[test]
    fn rejects_percent_without_sign_and_sheet_only_ids() {
        let profile = weapon(json!({ "mainStats": [
            { "id": "AmplifyAllDamage", "value": "10" },
            { "id": "STR", "value": "50" }
        ] }));
        assert!(collect_formula_bonuses(&profile).0.is_empty());
    }

    #[test]
    fn sheet_attack_increase_merges_with_gear() {
        let mut profile = weapon(json!({ "mainStats": [ { "id": "AmplifyWeaponDamage", "value": "5%" } ] }));
        profile["info"] = json!({ "stat": { "statList": [ { "type": "STR", "value": "200" } ] } });
        let (values, sources) = collect_formula_bonuses(&profile);
        assert_eq!(values["attackIncrease"], 120.0);
        assert_eq!(sources["attackIncrease"], "sheet");
        assert_eq!(values["weaponDamageBoost"], 105.0);
    }
}
