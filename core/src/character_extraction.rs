use serde_json::Value;
use std::collections::{BTreeMap, HashSet};
use std::sync::LazyLock;

static EMPTY_OBJECT: LazyLock<Value> = LazyLock::new(|| Value::Object(serde_json::Map::new()));

const GEAR_OPTION_NAMES: &[(&str, &str)] = &[
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

const PRIMARY_STAT_TYPES: &[&str] = &["STR", "DEX", "CON", "INT", "AGI", "WIS"];
const DIVINITY_TYPES: &[&str] = &[
    "Justice", "Freedom", "Illusion", "Life", "Time", "Destruction", "Death", "Wisdom", "Destiny", "Space",
];

pub fn option_name(option: &Value) -> Option<String> {
    let id = option.get("id").and_then(Value::as_str);
    if let Some(id_str) = id {
        if PRIMARY_STAT_TYPES.contains(&id_str) || DIVINITY_TYPES.contains(&id_str) {
            return None;
        }
        for &(key, val) in GEAR_OPTION_NAMES {
            if key == id_str {
                return Some(val.to_string());
            }
        }
    }
    option.get("name").and_then(Value::as_str).map(|s| s.to_string())
}

struct BonusMapping {
    field: &'static str,
    percentage: bool,
}

fn bonus_mapping(name: &str) -> Option<BonusMapping> {
    match name {
        "Attack Bonus" => Some(BonusMapping { field: "attackBonus", percentage: false }),
        "PvE Attack" | "PvE Attack Power" => Some(BonusMapping { field: "pveAttackPower", percentage: false }),
        "Boss Attack" | "Boss Attack Power" => Some(BonusMapping { field: "bossAttackPower", percentage: false }),
        "Penetration" => Some(BonusMapping { field: "penetration", percentage: false }),
        "Weapon Damage Boost" => Some(BonusMapping { field: "weaponDamageBoost", percentage: true }),
        "Damage Boost" => Some(BonusMapping { field: "damageBoost", percentage: true }),
        "PvE Damage Boost" => Some(BonusMapping { field: "pveDamageBoost", percentage: true }),
        "Boss Damage Boost" => Some(BonusMapping { field: "bossDamageBoost", percentage: true }),
        "Attack Increase" => Some(BonusMapping { field: "attackIncrease", percentage: true }),
        "Critical Damage Boost" => Some(BonusMapping { field: "criticalDamageBoost", percentage: true }),
        "Front Attack Damage Boost" => Some(BonusMapping { field: "frontBoost", percentage: true }),
        "Back Attack Damage Boost" => Some(BonusMapping { field: "backBoost", percentage: true }),
        "Skill Specialty Enhancement" => Some(BonusMapping { field: "skillSpecialtyEnhancement", percentage: true }),
        _ => None,
    }
}

pub struct EquippedItem<'a> {
    pub item: &'a Value,
    pub detail: &'a Value,
}

pub fn equipped_details(profile: &Value) -> Vec<EquippedItem<'_>> {
    let equipped = profile
        .get("equipment")
        .and_then(|e| e.get("equipment"))
        .and_then(|e| e.get("equipmentList"))
        .and_then(Value::as_array);

    let details = profile.get("itemDetails");

    match equipped {
        Some(list) => list
            .iter()
            .filter(|item| {
                item.get("slotPosName")
                    .and_then(Value::as_str)
                    .map(|s| !s.starts_with("Arcana"))
                    .unwrap_or(true)
            })
            .map(|item| {
                let slot = item
                    .get("slotPos")
                    .map(|s| if let Some(n) = s.as_i64() { n.to_string() } else { s.as_str().unwrap_or("").to_string() })
                    .unwrap_or_default();
                let detail = details.and_then(|d| d.get(&slot)).unwrap_or(&*EMPTY_OBJECT);
                EquippedItem { item, detail }
            })
            .collect(),
        None => Vec::new(),
    }
}

fn parse_stat_number(text: &str, is_percent: bool) -> Option<f64> {
    let trimmed = text.trim();
    if is_percent {
        let stripped = trimmed.strip_suffix('%')?;
        stripped.parse::<f64>().ok()
    } else {
        if trimmed.ends_with('%') {
            return None;
        }
        trimmed.parse::<f64>().ok()
    }
}

pub fn collect_formula_bonuses(profile: &Value) -> (BTreeMap<String, f64>, BTreeMap<String, String>) {
    let mut sums: BTreeMap<String, f64> = BTreeMap::new();
    let mut sources: BTreeMap<String, String> = BTreeMap::new();

    for equipped in equipped_details(profile) {
        let main_stats = equipped.detail.get("mainStats").and_then(Value::as_array);
        let sub_stats = equipped.detail.get("subStats").and_then(Value::as_array);
        let all_stats = main_stats.into_iter().flatten().chain(sub_stats.into_iter().flatten());

        for stat in all_stats {
            let opt_name = match option_name(stat) {
                Some(name) => name,
                None => continue,
            };
            let mapping = match bonus_mapping(&opt_name) {
                Some(m) => m,
                None => continue,
            };

            let text = stat
                .get("value")
                .map(|v| if let Some(s) = v.as_str() { s.to_string() } else { v.to_string() })
                .unwrap_or_default();

            if let Some(num) = parse_stat_number(&text, mapping.percentage) {
                *sums.entry(mapping.field.to_string()).or_default() += num;
                sources.insert(mapping.field.to_string(), "gear".to_string());
            }
        }
    }

    if let Some(stats) = profile.get("info").and_then(|i| i.get("stat")).and_then(|s| s.get("statList")).and_then(Value::as_array) {
        for stat in stats {
            let stat_type = stat.get("type").and_then(Value::as_str).unwrap_or("");
            if stat_type == "STR" || stat_type == "Destruction" {
                let amount = stat
                    .get("value")
                    .and_then(|v| v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse::<f64>().ok())))
                    .unwrap_or(0.0);
                if amount > 0.0 {
                    *sums.entry("attackIncrease".to_string()).or_default() += amount / 10.0;
                    let prev = sources.get("attackIncrease").map(String::as_str);
                    let new_src = match prev {
                        Some("gear") | Some("sheet+gear") => "sheet+gear",
                        _ => "sheet",
                    };
                    sources.insert("attackIncrease".to_string(), new_src.to_string());
                }
            }
        }
    }

    let mut values = BTreeMap::new();
    for (field, amount) in sums {
        let baseline = match field.as_str() {
            "weaponDamageBoost" | "damageBoost" | "attackIncrease" | "frontBoost" | "backBoost"
            | "skillSpecialtyEnhancement" => 100.0,
            "criticalDamageBoost" => 150.0,
            _ => 0.0,
        };
        let rounded = ((baseline + amount) * 10000.0).round() / 10000.0;
        values.insert(field, rounded);
    }

    (values, sources)
}

pub fn equipment_bonuses(profile: &Value) -> BTreeMap<String, f64> {
    collect_formula_bonuses(profile).0
}

pub fn character_hp(profile: &Value) -> Option<f64> {
    let stat_list = profile
        .get("info")
        .and_then(|i| i.get("stat"))
        .and_then(|s| s.get("statList"))
        .and_then(Value::as_array)?;

    let hp_stat = stat_list.iter().find(|s| s.get("name").and_then(Value::as_str) == Some("HP"))?;
    let val = hp_stat.get("value")?;
    let num = if let Some(n) = val.as_f64() {
        n
    } else if let Some(s) = val.as_str() {
        s.trim().replace(',', "").parse::<f64>().ok()?
    } else {
        return None;
    };
    (num > 0.0).then_some(num)
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExtractedCharacter {
    pub name: String,
    pub server: String,
    pub class_name: String,
    pub level: String,
    pub combat_power: Option<f64>,
    pub race: String,
    pub gender: String,
    pub title_name: String,
    pub item_level: Option<f64>,
    pub gear_score: Option<f64>,
    pub titles_owned: Option<i64>,
    pub titles_total: Option<i64>,
    pub legion: String,
    pub portrait: String,
    pub ranking: Option<Value>,
    pub stats: Vec<Value>,
    pub gear: Vec<Value>,
    pub arcana: Vec<Value>,
    pub skills: Vec<Value>,
    pub titles: Vec<Value>,
    pub boards: Vec<Value>,
    pub pet: Option<Value>,
    pub wing: Option<Value>,
}

pub fn extract_character(data: &Value) -> ExtractedCharacter {
    let empty_obj = Value::Object(serde_json::Map::new());
    let profile = data.get("info").and_then(|i| i.get("profile")).unwrap_or(&empty_obj);
    let stats = data
        .get("info")
        .and_then(|i| i.get("stat"))
        .and_then(|s| s.get("statList"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let combat_power = profile
        .get("combatPower")
        .and_then(|v| v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse::<f64>().ok())));

    let item_level = stats
        .iter()
        .find(|s| s.get("type").and_then(Value::as_str) == Some("ItemLevel"))
        .and_then(|s| s.get("value"))
        .and_then(|v| v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse::<f64>().ok())));

    let valid_stats: Vec<Value> = stats
        .into_iter()
        .filter(|s| {
            let name = s.get("name").and_then(Value::as_str).unwrap_or("");
            let val_str = s
                .get("value")
                .map(|v| if let Some(st) = v.as_str() { st.to_string() } else { v.to_string() })
                .unwrap_or_default();
            let trimmed = val_str.trim();
            if name.is_empty() || trimmed.is_empty() {
                return false;
            }
            let stripped = trimmed.strip_suffix('%').unwrap_or(trimmed);
            stripped.parse::<f64>().is_ok()
        })
        .collect();

    let equipped_raw = data
        .get("equipment")
        .and_then(|e| e.get("equipment"))
        .and_then(|e| e.get("equipmentList"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let details_raw = data.get("itemDetails").unwrap_or(&empty_obj);

    let mut gear = Vec::new();
    let mut arcana = Vec::new();

    for item in equipped_raw {
        let slot_name = item.get("slotPosName").and_then(Value::as_str).unwrap_or("");
        let slot = item
            .get("slotPos")
            .map(|s| if let Some(n) = s.as_i64() { n.to_string() } else { s.as_str().unwrap_or("").to_string() })
            .unwrap_or_default();
        let detail = details_raw.get(&slot).cloned().unwrap_or(empty_obj.clone());

        let pair = serde_json::json!({ "item": item, "detail": detail });
        if slot_name.starts_with("Arcana") {
            arcana.push(pair);
        } else {
            gear.push(pair);
        }
    }

    let skills: Vec<Value> = data
        .get("equipment")
        .and_then(|e| e.get("skill"))
        .and_then(|s| s.get("skillList"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|s| s.get("name").and_then(Value::as_str).map(|n| !n.is_empty()).unwrap_or(false))
        .collect();

    let titles: Vec<Value> = data
        .get("info")
        .and_then(|i| i.get("title"))
        .and_then(|t| t.get("titleList"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|t| t.get("name").and_then(Value::as_str).map(|n| !n.is_empty()).unwrap_or(false))
        .collect();

    let boards = data
        .get("info")
        .and_then(|i| i.get("daevanion"))
        .and_then(|d| d.get("boardList"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    ExtractedCharacter {
        name: profile.get("characterName").and_then(Value::as_str).unwrap_or("").to_string(),
        server: profile.get("serverName").and_then(Value::as_str).unwrap_or("").to_string(),
        class_name: profile.get("className").and_then(Value::as_str).unwrap_or("").to_string(),
        level: profile
            .get("characterLevel")
            .map(|v| if let Some(s) = v.as_str() { s.to_string() } else { v.to_string() })
            .unwrap_or_default(),
        combat_power,
        race: profile.get("raceName").and_then(Value::as_str).unwrap_or("").to_string(),
        gender: profile.get("genderName").and_then(Value::as_str).unwrap_or("").to_string(),
        title_name: profile.get("titleName").and_then(Value::as_str).unwrap_or("").to_string(),
        item_level,
        gear_score: item_level,
        titles_owned: data.get("info").and_then(|i| i.get("title")).and_then(|t| t.get("ownedCount")).and_then(Value::as_i64),
        titles_total: data.get("info").and_then(|i| i.get("title")).and_then(|t| t.get("totalCount")).and_then(Value::as_i64),
        legion: profile.get("regionName").and_then(Value::as_str).unwrap_or("").to_string(),
        portrait: profile.get("profileImage").and_then(Value::as_str).unwrap_or("").to_string(),
        ranking: data.get("externalRanking").cloned(),
        stats: valid_stats,
        gear,
        arcana,
        skills,
        titles,
        boards,
        pet: data.get("equipment").and_then(|e| e.get("petwing")).and_then(|p| p.get("pet")).cloned(),
        wing: data.get("equipment").and_then(|e| e.get("petwing")).and_then(|p| p.get("wing")).cloned(),
    }
}

const ENEMY_FIELDS: &[(&str, &str, bool)] = &[
    ("Defense", "defense", false),
    ("Weapon Damage Tolerance", "weaponDamageTolerance", true),
    ("Damage Tolerance", "damageTolerance", true),
    ("PvE Damage Tolerance", "pveDamageTolerance", true),
    ("Species Damage Tolerance", "speciesDamageTolerance", true),
    ("Boss Damage Tolerance", "bossDamageTolerance", true),
    ("Critical Damage Tolerance", "criticalDamageTolerance", true),
    ("Front Attack Damage Tolerance", "frontDamageTolerance", true),
    ("Back Attack Damage Tolerance", "backDamageTolerance", true),
    ("Block", "block", false),
    ("Evasion", "evasion", false),
    ("Critical Hit Resist", "criticalHitResist", false),
    ("Double Resist", "doubleResist", true),
    ("Perfect Resist", "perfectResist", true),
    ("Multi-hit Resist", "multiHitResist", true),
    ("Endurance", "endurance", true),
    ("Elemental Resistance", "elementalResistance", false),
    ("Critical Damage Defense", "criticalDamageDefense", false),
    ("Front Defense", "frontDefense", false),
    ("Back Defense", "backDefense", false),
];

const TYPED_DEFENSIVE_EFFECTS: &[(&str, &str)] = &[
    ("Time", "doubleResist"),
    ("Destruction", "perfectResist"),
    ("Destiny", "endurance"),
    ("Life", "regeneration"),
];

const TYPED_OFFENSIVE_EFFECTS: &[(&str, &str)] = &[
    ("Wisdom", "doubleChance"),
    ("Illusion", "endurancePenetration"),
    ("Death", "regenerationPenetration"),
    ("Justice", "perfectChance"),
];

fn extract_fields(
    profile: &Value,
    fields: &[(&str, &str, bool)],
    typed_effects: &[(&str, &str)],
) -> BTreeMap<String, f64> {
    let mut values = BTreeMap::new();

    // 1. Gear fields
    for equipped in equipped_details(profile) {
        let main = equipped.detail.get("mainStats").and_then(Value::as_array);
        let sub = equipped.detail.get("subStats").and_then(Value::as_array);
        for stat in main.into_iter().flatten().chain(sub.into_iter().flatten()) {
            let name = match option_name(stat) {
                Some(n) => n,
                None => continue,
            };
            if let Some(&(_, target_field, is_percent)) = fields.iter().find(|(k, _, _)| *k == name.as_str()) {
                let text = stat.get("value").map(|v| if let Some(s) = v.as_str() { s.to_string() } else { v.to_string() }).unwrap_or_default();
                if let Some(num) = parse_stat_number(&text, is_percent) {
                    let entry = values.entry(target_field.to_string()).or_insert(0.0);
                    *entry = ((*entry + num) * 10000.0).round() / 10000.0;
                }
            }
        }
    }

    // 2. Typed percent effects
    if let Some(stats) = profile.get("info").and_then(|i| i.get("stat")).and_then(|s| s.get("statList")).and_then(Value::as_array) {
        for stat in stats {
            let stat_type = stat.get("type").and_then(Value::as_str).unwrap_or("");
            if let Some(&(_, target_field)) = typed_effects.iter().find(|(k, _)| *k == stat_type) {
                let amount = stat
                    .get("value")
                    .and_then(|v| v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse::<f64>().ok())))
                    .unwrap_or(0.0) / 10.0;
                if amount > 0.0 {
                    values.insert(target_field.to_string(), amount);
                }
            }
        }
    }

    // 3. Direct sheet fields override or supplement gear
    if let Some(stats) = profile.get("info").and_then(|i| i.get("stat")).and_then(|s| s.get("statList")).and_then(Value::as_array) {
        for stat in stats {
            let stat_name = stat.get("name").and_then(Value::as_str).unwrap_or("");
            if let Some(&(_, target_field, is_percent)) = fields.iter().find(|(k, _, _)| *k == stat_name) {
                let text = stat.get("value").map(|v| if let Some(s) = v.as_str() { s.to_string() } else { v.to_string() }).unwrap_or_default();
                let stripped = text.trim();
                let parsed = if is_percent {
                    stripped.strip_suffix('%').unwrap_or(stripped).parse::<f64>().ok()
                } else {
                    stripped.parse::<f64>().ok()
                };
                if let Some(num) = parsed {
                    values.insert(target_field.to_string(), num);
                }
            }
        }
    }

    values
}

pub fn enemy_defenses(profile: &Value) -> BTreeMap<String, f64> {
    extract_fields(profile, ENEMY_FIELDS, TYPED_DEFENSIVE_EFFECTS)
}

const REFERENCE_FIELDS: &[(&str, &str, bool)] = &[
    ("Accuracy", "accuracy", false),
    ("Critical Hit", "criticalHit", false),
    ("Block Penetration", "blockPenetration", false),
    ("Double Chance", "doubleChance", true),
    ("Cogni Damage Boost", "cogniDamageBoost", true),
    ("Fera Damage Boost", "feraDamageBoost", true),
    ("Natura Damage Boost", "naturaDamageBoost", true),
    ("Varian Damage Boost", "varianDamageBoost", true),
    ("Fire Boost", "fireBoost", true),
    ("Water Boost", "waterBoost", true),
    ("Wind Boost", "windBoost", true),
    ("Earth Boost", "earthBoost", true),
    ("PvE Accuracy", "pveAccuracy", false),
];

pub fn offensive_references(profile: &Value) -> BTreeMap<String, f64> {
    extract_fields(profile, REFERENCE_FIELDS, TYPED_OFFENSIVE_EFFECTS)
}

#[derive(Clone, Debug, PartialEq)]
pub struct FormulaImportCoverage {
    pub imported: Vec<String>,
    pub manual: Vec<String>,
    pub sources: BTreeMap<String, String>,
}

const IMPORTABLE_FORMULA_FIELDS: &[&str] = &[
    "pureAttack", "weaponDamageBoost", "multiHit", "powerShard", "attackBonus",
    "attackIncrease", "pveAttackPower", "bossAttackPower", "frontBackAttackPower",
    "skillCoefficient", "skillAdditionalDamage", "penetration", "skillSpecialtyEnhancement",
    "damageBoost", "pveDamageBoost", "bossDamageBoost", "criticalDamageBoost",
    "frontBoost", "backBoost", "hitsPerSecond"
];

pub fn formula_import_coverage(profile: &Value) -> FormulaImportCoverage {
    let (mut values, mut sources) = collect_formula_bonuses(profile);
    if let Some(hp) = character_hp(profile) {
        values.insert("playerHp".to_string(), hp);
        sources.insert("playerHp".to_string(), "sheet".to_string());
    }
    let imported_keys: HashSet<String> = values.keys().cloned().collect();
    let imported: Vec<String> = values.into_keys().collect();
    let manual: Vec<String> = IMPORTABLE_FORMULA_FIELDS
        .iter()
        .filter(|&&f| !imported_keys.contains(f))
        .map(|&s| s.to_string())
        .collect();

    FormulaImportCoverage {
        imported,
        manual,
        sources,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn test_profile() -> Value {
        json!({
            "info": {
                "profile": {
                    "characterName": "Kanon",
                    "serverName": "Siel",
                    "className": "Cleric",
                    "characterLevel": 45,
                    "combatPower": 90000
                },
                "stat": {
                    "statList": [{ "name": "Might", "value": 12 }]
                }
            },
            "equipment": {
                "equipment": {
                    "equipmentList": [{ "slotPos": 1, "slotPosName": "MainHand", "name": "Mace" }]
                },
                "skill": {
                    "skillList": [
                        { "name": "Bolt", "skillLevel": 12, "acquired": 1, "category": "Active" },
                        { "name": "Locked", "skillLevel": 0, "acquired": 0 }
                    ]
                }
            },
            "itemDetails": {
                "1": {
                    "subStats": [
                        { "name": "Damage Boost", "value": "5.6%" },
                        { "name": "PvE Attack", "value": "20" },
                        { "name": "Weapon Damage Boost", "value": "3%" },
                        { "name": "Attack", "value": "251" },
                        { "name": "Critical Damage Boost", "value": "2%" }
                    ],
                    "mainStats": [
                        { "name": "Attack", "value": "323", "minValue": "251", "extra": "119" }
                    ]
                }
            }
        })
    }

    #[test]
    fn extracts_character_core_fields() {
        let view = extract_character(&test_profile());
        assert_eq!(view.name, "Kanon");
        assert_eq!(view.server, "Siel");
        assert_eq!(view.class_name, "Cleric");
        assert_eq!(view.combat_power, Some(90000.0));
        assert_eq!(view.gear.len(), 1);
        assert_eq!(view.gear[0]["item"]["name"], "Mace");
        assert_eq!(view.gear[0]["detail"]["subStats"][0]["value"], "5.6%");
        assert_eq!(view.skills.len(), 2);
    }

    #[test]
    fn imports_only_explicit_numeric_gear_bonuses() {
        let bonuses = equipment_bonuses(&test_profile());
        assert_eq!(bonuses.get("damageBoost"), Some(&105.6));
        assert_eq!(bonuses.get("pveAttackPower"), Some(&20.0));
        assert_eq!(bonuses.get("weaponDamageBoost"), Some(&103.0));
        assert_eq!(bonuses.get("criticalDamageBoost"), Some(&152.0));
    }

    #[test]
    fn imports_enemy_defenses() {
        let mut enemy = test_profile();
        enemy["info"]["stat"]["statList"] = json!([
            { "name": "Defense", "value": "825" },
            { "name": "Damage Tolerance", "value": "12.5%" },
            { "name": "PvE Damage Tolerance", "value": "4%" },
            { "name": "Weapon Damage Tolerance", "value": "3%" },
            { "name": "Critical Damage Tolerance", "value": "5%" },
            { "name": "Back Attack Damage Tolerance", "value": "7%" },
            { "name": "Block", "value": "162" },
            { "name": "Evasion", "value": "22" },
            { "name": "Double Resist", "value": "8%" },
            { "name": "Endurance", "value": "3%" }
        ]);
        let defs = enemy_defenses(&enemy);
        assert_eq!(defs.get("defense"), Some(&825.0));
        assert_eq!(defs.get("damageTolerance"), Some(&12.5));
        assert_eq!(defs.get("pveDamageTolerance"), Some(&4.0));
        assert_eq!(defs.get("weaponDamageTolerance"), Some(&3.0));
        assert_eq!(defs.get("criticalDamageTolerance"), Some(&5.0));
        assert_eq!(defs.get("backDamageTolerance"), Some(&7.0));
        assert_eq!(defs.get("block"), Some(&162.0));
        assert_eq!(defs.get("evasion"), Some(&22.0));
        assert_eq!(defs.get("doubleResist"), Some(&8.0));
        assert_eq!(defs.get("endurance"), Some(&3.0));
    }

    #[test]
    fn imports_offensive_references() {
        let mut player = test_profile();
        player["info"]["stat"]["statList"] = json!([
            { "name": "Accuracy", "value": "1200" },
            { "name": "Critical Hit", "value": "475" }
        ]);
        player["itemDetails"]["1"]["subStats"].as_array_mut().unwrap().extend(vec![
            json!({ "name": "Block Penetration", "value": "28" }),
            json!({ "name": "Double Chance", "value": "3.5%" }),
            json!({ "name": "Cogni Damage Boost", "value": "4%" }),
            json!({ "name": "Fire Boost", "value": "2.5%" })
        ]);
        let refs = offensive_references(&player);
        assert_eq!(refs.get("accuracy"), Some(&1200.0));
        assert_eq!(refs.get("criticalHit"), Some(&475.0));
        assert_eq!(refs.get("blockPenetration"), Some(&28.0));
        assert_eq!(refs.get("doubleChance"), Some(&3.5));
        assert_eq!(refs.get("cogniDamageBoost"), Some(&4.0));
        assert_eq!(refs.get("fireBoost"), Some(&2.5));
    }
}
