// Port of attribute-trees.js. Tree values come from calculator inputs; profile
// (imported defenses) leaves show "unknown" until the Character import is ported.
// Deviation: the nested tree is a flat list of the 17 leaves; the tip is shown
// below the list instead of anchored to the button.
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

const LEAVES: [Leaf; 16] = [
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

fn leaf_text(state: AppState, leaf: &Leaf) -> String {
    let lang = state.lang.get();
    if leaf.profile {
        return t(lang, "treeUnknown");
    }
    match input_value(state, leaf.stat) {
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
            view! {
                <div class="tree-leaf">
                    <button
                        type="button"
                        class="tree-circle"
                        on:mouseenter=move |_| {
                            if !pinned.get() {
                                tip.set(Some(leaf_tip(state.lang.get(), leaf)));
                            }
                        }
                        on:focus=move |_| {
                            if !pinned.get() {
                                tip.set(Some(leaf_tip(state.lang.get(), leaf)));
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
                            let current = leaf_tip(state.lang.get(), leaf);
                            if pinned.get() && tip.get().as_ref() == Some(&current) {
                                pinned.set(false);
                                tip.set(None);
                            } else {
                                pinned.set(true);
                                tip.set(Some(current));
                            }
                        }
                    >
                        <span inner_html=icon></span>
                    </button>
                    <div class="tree-leaf-text">
                        <strong>{label}</strong>
                        <small>{hint}</small>
                        <span class="tree-value">{text}</span>
                    </div>
                </div>
            }
        })
        .collect_view();

    view! {
        <section class="attributes-view">
            <div class="tree-list">{leaves}</div>
            <div class="tree-tip" role="tooltip" hidden=move || tip.get().is_none()>
                {move || tip.get().map(|tp| view! {
                    <strong>{tp.title}</strong>
                    <p>{tp.text}</p>
                })}
            </div>
        </section>
    }
}
