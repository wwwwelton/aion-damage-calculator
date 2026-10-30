// Port of attribute-trees.js. The accepted flat tree and inline tip layout are retained.
use std::collections::HashMap;
use std::sync::LazyLock;
use crate::i18n::{t, Lang};
use crate::state::AppState;
use aion2_core::numeric_input::parse_numeric_input;
use aion2_core::stat_info::stat_info;
use leptos::ev;
use leptos::prelude::*;

struct Leaf {
    stat: &'static str,
    icon: &'static str,
    label: &'static str,
    hint: &'static str,
    tip: Option<&'static str>,
    percent: bool,
    profile: bool,
}

const fn leaf(
    stat: &'static str,
    icon: &'static str,
    label: &'static str,
    hint: &'static str,
    tip: Option<&'static str>,
    percent: bool,
    profile: bool,
) -> Leaf {
    Leaf { stat, icon, label, hint, tip, percent, profile }
}

static LEAVES: [Leaf; 16] = [
    leaf("pureAttack", "attack", "pureAttackLabel", "treeHintPureAttack", Some("treeTipPureAttack"), false, false),
    leaf("attackIncrease", "attack", "treeAttackIncrease", "treeHintAttackIncrease", None, true, false),
    leaf("penetration", "attack", "penetrationLabel", "treeHintPenetration", Some("treeTipPenetration"), false, false),
    leaf("accuracy", "chance", "treeAccuracy", "treeHintAccuracy", None, false, false),
    leaf("damageBoost", "damage", "damageBoostLabel", "treeHintDamageBoost", Some("treeTipDamageBoost"), true, false),
    leaf("weaponDamageBoost", "damage", "treeWeaponBoost", "treeHintWeaponBoost", None, true, false),
    leaf("criticalHit", "chance", "treeCriticalHit", "treeHintCriticalHit", None, false, false),
    leaf("criticalDamageBoost", "damage", "criticalDamageBoostLabel", "treeHintCriticalDamageBoost", Some("treeTipCriticalDamageBoost"), true, false),
    leaf("defense", "defense", "treeDefense", "treeHintDefense", Some("treeTipDefense"), false, true),
    leaf("damageTolerance", "defense", "treeTolerance", "treeHintDamageTolerance", Some("treeTipDamageTolerance"), true, true),
    leaf("criticalHitResist", "defense", "treeCriticalResist", "treeHintCriticalResist", None, false, true),
    leaf("criticalDamageTolerance", "defense", "treeCriticalTolerance", "treeHintCriticalTolerance", None, true, true),
    leaf("evasion", "chance", "treeEvasion", "treeHintEvasion", Some("treeTipEvasion"), false, true),
    leaf("block", "defense", "treeBlock", "treeHintBlock", None, false, true),
    leaf("playerHp", "health", "playerHpLabel", "treeHintPlayerHp", Some("treeTipPlayerHp"), false, false),
    leaf("endurance", "health", "treeEndurance", "treeHintEndurance", None, true, true),
];

#[derive(Clone, PartialEq)]
struct Tip {
    title: String,
    text: String,
}

fn icon_html(kind: &str) -> String {
    format!(
        r##"<svg class="icon icon-{kind}" viewBox="0 0 24 24" aria-hidden="true"><use href="#icon-{kind}"></use></svg>"##
    )
}

/// Port of Intl.NumberFormat with maximumFractionDigits 2 and locale grouping.
fn format_number(v: f64, pt: bool) -> String {
    let (group, decimal) = if pt { ('.', ',') } else { (',', '.') };
    let fixed = format!("{:.2}", v.abs());
    let (int_part, frac) = fixed.split_once('.').unwrap_or((&fixed, ""));
    let frac = frac.trim_end_matches('0');
    let mut grouped = String::new();
    for (i, c) in int_part.chars().enumerate() {
        if i > 0 && (int_part.len() - i) % 3 == 0 {
            grouped.push(group);
        }
        grouped.push(c);
    }
    let sign = if v < 0.0 { "-" } else { "" };
    if frac.is_empty() {
        format!("{sign}{grouped}")
    } else {
        format!("{sign}{grouped}{decimal}{frac}")
    }
}

fn input_value(state: AppState, name: &str) -> Option<f64> {
    state
        .inputs
        .get_value()
        .iter()
        .find(|(k, _)| *k == name)
        .and_then(|(_, sig)| parse_numeric_input(&sig.get(), f64::NEG_INFINITY))
}

fn profile_description(lang: Lang, stat: &serde_json::Value) -> String {
    let key = match stat["type"].as_str().unwrap_or("") {
        "STR" => "treeTipMight", "AGI" => "treeTipPrecision",
        "DEX" => "treeTipDexterity", "CON" => "treeTipConstitution",
        "Justice" => "treeTipJustice", "Freedom" => "treeTipFreedom",
        "Illusion" => "treeTipIllusion", "Life" => "treeTipLife",
        "Time" => "treeTipTime", "Destruction" => "treeTipDestruction",
        "Death" => "treeTipDeath", "Wisdom" => "treeTipWisdom",
        "Destiny" => "treeTipDestiny", "Space" => "treeTipSpace",
        _ => "",
    };
    if !key.is_empty() {
        return t(lang, key);
    }
    let name = stat["name"].as_str().unwrap_or("");
    let mut words = name.split(|c: char| !c.is_ascii_alphanumeric()).filter(|s| !s.is_empty());
    let mut field = words.next().unwrap_or("").to_ascii_lowercase();
    for word in words {
        field.push_str(&word[..1].to_ascii_uppercase());
        field.push_str(&word[1..]);
    }
    stat_info(name, matches!(lang, Lang::Pt))
        .or_else(|| stat_info(&field, matches!(lang, Lang::Pt)))
        .map(|info| info.desc)
        .unwrap_or_else(|| t(lang, "treeProfileNoMechanics"))
}

fn current_profile(state: AppState) -> Option<serde_json::Value> {
    state.imported_profile.get().filter(|profile| {
        profile.pointer("/info/profile/characterName").and_then(serde_json::Value::as_str)
            == Some(state.attacker_name.get().as_str())
            && profile.pointer("/info/profile/className").and_then(serde_json::Value::as_str)
                == Some(state.attacker_class.get().as_str())
    })
}

fn leaf_text(state: AppState, leaf: &Leaf) -> String {
    let lang = state.lang.get();
    let value = if leaf.profile {
        current_profile(state).and_then(|profile| {
            aion2_core::character_extraction::enemy_defenses(&profile).get(leaf.stat).copied()
        })
    } else {
        input_value(state, leaf.stat)
    };
    match value {
        Some(v) if !(leaf.stat == "playerHp" && v <= 0.0) => {
            let suffix = if leaf.percent { "%" } else { "" };
            format!("{}{suffix}", format_number(v, matches!(lang, Lang::Pt)))
        }
        _ => t(lang, "treeUnknown"),
    }
}

fn leaf_tip(lang: Lang, leaf: &Leaf) -> Tip {
    let text = match leaf.tip {
        Some(key) => t(lang, key),
        None => stat_info(leaf.stat, matches!(lang, Lang::Pt))
            .map(|i| i.desc)
            .unwrap_or_default(),
    };
    Tip { title: t(lang, leaf.label), text }
}

/// Enemy-side stats with no matchup row (markup-only inputs in index.html).
const MARKUP_ENEMY: [&str; 11] = [
    "enemyHp", "weaponDamageTolerance", "defense", "damageTolerance", "pveDamageTolerance",
    "bossDamageTolerance", "speciesDamageTolerance", "criticalDamageTolerance",
    "frontDamageTolerance", "backDamageTolerance", "elementalResistance",
];
const GENUS: [&str; 4] = ["cogni", "fera", "natura", "varian"];
const GENUS_FIELDS: [&str; 10] = [
    "DamageBoost", "DamageTolerance", "Attack", "Defense", "Accuracy", "Evasion",
    "BlockPenetration", "Block", "CriticalHit", "CriticalHitResist",
];

#[derive(Clone, Copy)]
struct Flags {
    enemy: bool,
    percent: bool,
}

/// Row shape in matchups.json: [attackLabel, attackField, defenseLabel, defenseField, percent?].
/// The defense side is the enemy stat; the optional trailing flag marks percentages.
static MATCHUP_FLAGS: LazyLock<HashMap<String, Flags>> = LazyLock::new(|| {
    let groups: serde_json::Value =
        serde_json::from_str(include_str!("../data/matchups.json")).expect("matchups.json");
    let mut flags = HashMap::new();
    for group in groups.as_array().into_iter().flatten() {
        for row in group["rows"].as_array().into_iter().flatten() {
            let percent = row.get(4) == Some(&serde_json::Value::Bool(true));
            if let Some(field) = row[1].as_str() {
                flags.insert(field.to_string(), Flags { enemy: false, percent });
            }
            if let Some(field) = row[3].as_str() {
                flags.insert(field.to_string(), Flags { enemy: true, percent });
            }
        }
    }
    flags
});

fn flags_for(name: &str) -> Flags {
    let matchup = MATCHUP_FLAGS.get(name).copied();
    Flags {
        enemy: matchup.is_some_and(|f| f.enemy) || MARKUP_ENEMY.contains(&name),
        percent: matchup.is_some_and(|f| f.percent)
            || stat_info(name, false).is_some_and(|i| i.tag.contains("(%)")),
    }
}

fn names(list: &[&str]) -> Vec<String> {
    list.iter().map(|n| n.to_string()).collect()
}

/// Port of catalogGroups in attribute-trees.js, in the same order.
fn catalog_groups() -> Vec<(&'static str, Vec<String>)> {
    let genus = GENUS
        .iter()
        .flat_map(|g| GENUS_FIELDS.iter().map(move |f| format!("{g}{f}")))
        .collect();
    vec![
        ("treeGroupHit", names(&["accuracy", "evasion", "blockPenetration", "block", "pveAccuracy", "pvpAccuracy", "pvpEvasion", "pvpBlockPenetration", "pvpBlock"])),
        ("treeGroupAttack", names(&["pureAttack", "attackBonus", "attackIncrease", "weaponDamageBoost", "weaponDamageTolerance", "powerShard", "multiHit", "pveAttackPower", "bossAttackPower", "speciesAttackPower", "frontBackAttackPower", "pvpAttack"])),
        ("treeGroupSkill", names(&["skillCoefficient", "skillAdditionalDamage", "skillSpecialtyEnhancement", "penetration", "defense", "pvpDefense"])),
        ("treeGroupBoost", names(&["damageBoost", "damageTolerance", "pveDamageBoost", "pveDamageTolerance", "bossDamageBoost", "bossDamageTolerance", "speciesDamageBoost", "speciesDamageTolerance", "pvpDamageBoost", "pvpDamageTolerance"])),
        ("treeGroupCritical", names(&["criticalHit", "criticalHitResist", "criticalDamageBoost", "criticalDamageTolerance", "criticalAttack", "criticalDamageDefense", "pvpCriticalHit", "pvpCriticalHitResist"])),
        ("treeGroupProc", names(&["doubleChance", "doubleResist", "double", "perfectChance", "perfectResist", "multiHitChance", "multiHitResist", "endurancePenetration", "endurance", "regenerationPenetration", "regeneration"])),
        ("treeGroupPosition", names(&["frontBoost", "frontDamageTolerance", "backBoost", "backDamageTolerance", "frontAttack", "frontDefense", "backAttack", "backDefense", "frontCriticalHit", "frontCriticalHitResist", "backCriticalHit", "backCriticalHitResist"])),
        ("treeGroupElemental", names(&["elementalResistance", "fireBoost", "fireTolerance", "waterBoost", "waterTolerance", "windBoost", "windTolerance", "earthBoost", "earthTolerance"])),
        ("treeGroupGenus", genus),
        ("treeGroupHp", names(&["playerHp", "enemyHp", "hitsPerSecond"])),
    ]
}

fn strip_parens(title: &str) -> &str {
    match title.find(" (") {
        Some(i) if title.ends_with(')') => &title[..i],
        _ => title,
    }
}

fn catalog_title(name: &str, lang: Lang, enemy: bool) -> String {
    // Like localizedStatInfo in stat-tooltips.js: translate the title, then drop the parenthesis.
    let base = stat_info(name, matches!(lang, Lang::Pt))
        .map(|i| strip_parens(&crate::i18n::stat_label(lang, &i.title)).to_string())
        .unwrap_or_else(|| name.to_string());
    if enemy {
        format!("{} · {base}", t(lang, "treeCatalogEnemy"))
    } else {
        base
    }
}

fn catalog_tip(lang: Lang, name: &str, enemy: bool) -> Tip {
    let text = stat_info(name, matches!(lang, Lang::Pt))
        .map(|i| i.desc)
        .unwrap_or_default();
    Tip { title: catalog_title(name, lang, enemy), text }
}

fn catalog_value(state: AppState, name: &str, percent: bool) -> String {
    let lang = state.lang.get();
    match input_value(state, name) {
        Some(v) => format!(
            "{}{}",
            format_number(v, matches!(lang, Lang::Pt)),
            if percent { "%" } else { "" }
        ),
        None => t(lang, "treeUnknown"),
    }
}

/// Circle/row button that shows a tip on hover/focus and pins it on click (closeTreeTip semantics).
fn tip_button(
    tip: RwSignal<Option<Tip>>,
    pinned: RwSignal<bool>,
    state: AppState,
    make: impl Fn(Lang) -> Tip + Clone + 'static,
    class: &'static str,
    inner: AnyView,
) -> impl IntoView {
    let (enter, focus, click) = (make.clone(), make.clone(), make);
    view! {
        <button
            type="button"
            class=class
            on:mouseenter=move |_| {
                if !pinned.get() {
                    tip.set(Some(enter(state.lang.get())));
                }
            }
            on:focus=move |_| {
                if !pinned.get() {
                    tip.set(Some(focus(state.lang.get())));
                }
            }
            on:mouseleave=move |_| {
                if !pinned.get() {
                    tip.set(None);
                }
            }
            on:blur=move |_| {
                if !pinned.get() {
                    tip.set(None);
                }
            }
            on:click=move |_| {
                let current = click(state.lang.get());
                if pinned.get() && tip.get().as_ref() == Some(&current) {
                    pinned.set(false);
                    tip.set(None);
                } else {
                    pinned.set(true);
                    tip.set(Some(current));
                }
            }
        >
            {inner}
        </button>
    }
}

#[component]
pub fn AttributesTab(state: AppState) -> impl IntoView {
    let tip = RwSignal::new(None::<Tip>);
    let pinned = RwSignal::new(false);

    // Escape closes a pinned tip, matching closeTreeTip().
    window_event_listener(ev::keydown, move |e| {
        if e.key() == "Escape" {
            pinned.set(false);
            tip.set(None);
        }
    });

    let leaves = LEAVES
        .iter()
        .map(|leaf| {
            let text = move || leaf_text(state, leaf);
            let label = move || t(state.lang.get(), leaf.label);
            let hint = move || t(state.lang.get(), leaf.hint);
            let icon = icon_html(leaf.icon);
            let inner = view! { <span inner_html=icon></span> }.into_any();
            view! {
                <div class="tree-leaf">
                    {tip_button(tip, pinned, state, move |lang| leaf_tip(lang, leaf), "tree-circle", inner)}
                    <div class="tree-leaf-text">
                        <strong>{label}</strong>
                        <small>{hint}</small>
                        <span class="tree-value">{text}</span>
                    </div>
                </div>
            }
        })
        .collect_view();

    let catalog = catalog_groups()
        .into_iter()
        .enumerate()
        .map(|(i, (key, group_names))| {
            let count = group_names.len();
            let open = i == 0;
            let items = group_names
                .into_iter()
                .map(|name| {
                    let flags = flags_for(&name);
                    let (value_name, title_name, tip_name) = (name.clone(), name.clone(), name);
                    let value = move || catalog_value(state, &value_name, flags.percent);
                    let title = move || catalog_title(&title_name, state.lang.get(), flags.enemy);
                    let inner = view! {
                        <span class="tree-catalog-name">{title}</span>
                        <strong class="tree-value">{value}</strong>
                    }
                    .into_any();
                    view! {
                        <li>{tip_button(tip, pinned, state, move |lang| catalog_tip(lang, &tip_name, flags.enemy), "tree-catalog-item", inner)}</li>
                    }
                })
                .collect_view();
            let summary = move || format!("{} ({count})", t(state.lang.get(), key));
            view! {
                <details class="tree-catalog-group" open=open>
                    <summary>{summary}</summary>
                    <ul class="tree-profile-list">{items}</ul>
                </details>
            }
        })
        .collect_view();

    view! {
        <section class="attributes-view">
            <div class="tree-list">{leaves}</div>
            <p>{move || current_profile(state).map(|_| crate::i18n::format_t(
                state.lang.get(), "treeImportedFrom", &[("name", &state.attacker_name.get())],
            )).unwrap_or_else(|| t(state.lang.get(), "treeValueNote"))}</p>
            <h3>{move || t(state.lang.get(), "treeProfileTitle")}</h3>
            {move || {
                let lang = state.lang.get();
                let profile = current_profile(state);
                let stats = profile.as_ref().map(|p| aion2_core::character_extraction::extract_character(p).stats).unwrap_or_default();
                let empty = stats.is_empty();
                let rows = stats.into_iter().map(|stat| {
                    let name = stat["name"].as_str().unwrap_or("").to_owned();
                    let raw = stat["value"].as_str().map(str::to_owned).unwrap_or_else(|| stat["value"].to_string());
                    let percent = raw.trim().ends_with('%');
                    let value = raw.trim().trim_end_matches('%').parse::<f64>().ok()
                        .map(|v| format!("{}{}", format_number(v, matches!(lang, Lang::Pt)), if percent { "%" } else { "" }))
                        .unwrap_or(raw);
                    let title = crate::i18n::stat_label(lang, &name);
                    let inner = view! { <span>{title}</span><strong>{value}</strong> }.into_any();
                    view! { <li>{tip_button(tip, pinned, state, move |lang| Tip {
                        title: crate::i18n::stat_label(lang, &name),
                        text: profile_description(lang, &stat),
                    }, "tree-catalog-item", inner)}</li> }
                }).collect_view();
                let modifiers = profile.as_ref().map(aion2_core::character_extraction::sheet_modifiers).unwrap_or_default();
                let has_modifiers = !modifiers.is_empty();
                let modifiers = modifiers.into_iter().map(|(field, amount)| {
                    let key = format!("modifier{}{}", field[..1].to_ascii_uppercase(), &field[1..]);
                    let title = t(lang, &key);
                    let inner = view! { <span>{title}</span><strong>{format!("{}%", format_number(amount, matches!(lang, Lang::Pt)))}</strong> }.into_any();
                    view! { <li>{tip_button(tip, pinned, state, move |lang| Tip {
                        title: t(lang, &key), text: t(lang, "sheetModifiersNote"),
                    }, "tree-catalog-item", inner)}</li> }
                }).collect_view();
                view! {
                    <p hidden=!empty>{t(lang, "treeProfileEmpty")}</p>
                    <ul id="tree-profile-list" class="tree-profile-list">{rows}</ul>
                    <section hidden=!has_modifiers>
                        <h3>{t(lang, "sheetModifiersTitle")}</h3>
                        <p>{t(lang, "sheetModifiersNote")}</p>
                        <ul id="tree-modifier-list" class="tree-profile-list">{modifiers}</ul>
                    </section>
                }
            }}
            <div class="tree-tip" role="tooltip" hidden=move || tip.get().is_none()>
                {move || tip.get().map(|tp| view! {
                    <strong>{tp.title}</strong>
                    <p>{tp.text}</p>
                })}
            </div>
            <div class="tree-catalog" id="tree-catalog">{catalog}</div>
        </section>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn imported_defenses_follow_identity_and_never_replace_target_inputs() {
        Owner::new().with(|| {
            let state = AppState::new();
            state.attacker_name.set("Alice".into());
            state.attacker_class.set("Ranger".into());
            state.imported_profile.set(Some(serde_json::json!({
                "info": {
                    "profile": {"characterName": "Alice", "className": "Ranger"},
                    "stat": {"statList": [{"name": "Defense", "value": "1400"}]}
                }
            })));
            let defense = LEAVES.iter().find(|leaf| leaf.stat == "defense").unwrap();
            assert_eq!(leaf_text(state, defense), "1,400");
            assert_eq!(input_value(state, "defense"), Some(0.0));
            state.attacker_class.set("Gladiator".into());
            assert_eq!(leaf_text(state, defense), "Not available");
            state.attacker_class.set("Ranger".into());
            state.attacker_name.set("Bob".into());
            assert_eq!(leaf_text(state, defense), "Not available");
            state.attacker_name.set("Alice".into());
            state.lang.set(Lang::Pt);
            assert_eq!(leaf_text(state, defense), "1.400");
        });
    }

    #[test]
    fn catalog_names_follow_the_selected_language_like_the_js_catalog() {
        assert_eq!(catalog_title("evasion", Lang::En, true), "Enemy · Evasion");
        assert_eq!(catalog_title("accuracy", Lang::Pt, false), "Precisão");
        assert_eq!(catalog_title("evasion", Lang::Pt, true), "Inimigo · Evasão");
        assert_eq!(catalog_title("blockPenetration", Lang::Pt, false), "Penetração de Bloqueio");
        assert_eq!(catalog_title("block", Lang::Pt, false), "Bloqueio");
        assert_eq!(catalog_title("cogniBlock", Lang::Pt, false), "Cogni Bloqueio");
    }
}
