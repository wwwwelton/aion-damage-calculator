use wasm_bindgen::prelude::*;
use web_sys::{Document, Element, HtmlElement, HtmlInputElement, Window};
use serde_json::Value;
use std::sync::LazyLock;
use std::cell::RefCell;
use aion2_core::numeric_input::parse_numeric_input;
use aion2_core::stat_info::stat_info;
use crate::controller::{current_lang, document, stat_label, t, window};

static MATCHUPS_RAW: LazyLock<Value> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../../data/matchups.json")).expect("matchups.json")
});

#[derive(Clone, Copy, Debug)]
struct StatFlags {
    enemy: bool,
    percent: bool,
}

static MATCHUP_FLAGS: LazyLock<std::collections::HashMap<String, StatFlags>> = LazyLock::new(|| {
    let mut map = std::collections::HashMap::new();
    if let Some(arr) = MATCHUPS_RAW.as_array() {
        for group in arr {
            if let Some(rows) = group.get("rows").and_then(Value::as_array) {
                for row in rows {
                    let percent = row.get(4).and_then(Value::as_bool).unwrap_or(false);
                    if let Some(atk) = row.get(1).and_then(Value::as_str) {
                        map.insert(atk.to_string(), StatFlags { enemy: false, percent });
                    }
                    if let Some(def) = row.get(3).and_then(Value::as_str) {
                        map.insert(def.to_string(), StatFlags { enemy: true, percent });
                    }
                }
            }
        }
    }
    map
});

const LEAVES: &[(&str, &str, &str, Option<&str>, bool, bool)] = &[
    ("pureAttack", "attack", "treeHintPureAttack", Some("treeTipPureAttack"), false, false),
    ("attackIncrease", "attack", "treeHintAttackIncrease", None, true, false),
    ("penetration", "attack", "treeHintPenetration", Some("treeTipPenetration"), false, false),
    ("accuracy", "chance", "treeHintAccuracy", None, false, false),
    ("damageBoost", "damage", "treeHintDamageBoost", Some("treeTipDamageBoost"), true, false),
    ("weaponDamageBoost", "damage", "treeHintWeaponBoost", None, true, false),
    ("criticalHit", "chance", "treeHintCriticalHit", None, false, false),
    ("criticalDamageBoost", "damage", "treeHintCriticalDamageBoost", Some("treeTipCriticalDamageBoost"), true, false),
    ("defense", "defense", "treeHintDefense", Some("treeTipDefense"), false, true),
    ("damageTolerance", "defense", "treeHintDamageTolerance", Some("treeTipDamageTolerance"), true, true),
    ("criticalHitResist", "defense", "treeHintCriticalResist", None, false, true),
    ("criticalDamageTolerance", "defense", "treeHintCriticalTolerance", None, true, true),
    ("evasion", "chance", "treeHintEvasion", Some("treeTipEvasion"), false, true),
    ("block", "defense", "treeHintBlock", None, false, true),
    ("playerHp", "health", "treeHintPlayerHp", Some("treeTipPlayerHp"), false, false),
    ("endurance", "health", "treeHintEndurance", None, true, true),
];

const GENUS: &[&str] = &["cogni", "fera", "natura", "varian"];
const GENUS_FIELDS: &[&str] = &[
    "DamageBoost", "DamageTolerance", "Attack", "Defense", "Accuracy",
    "Evasion", "BlockPenetration", "Block", "CriticalHit", "CriticalHitResist",
];

struct TooltipState {
    active_button: Option<Element>,
    pinned: bool,
    active_title: String,
    active_text: String,
}

thread_local! {
    static TOOLTIP_STATE: RefCell<TooltipState> = RefCell::new(TooltipState {
        active_button: None,
        pinned: false,
        active_title: String::new(),
        active_text: String::new(),
    });

    static IMPORTED_CHARACTER: RefCell<Option<Value>> = const { RefCell::new(None) };
}

pub fn format_number(v: f64, pt: bool) -> String {
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

pub fn create_icon_svg(doc: &Document, symbol: &str) -> Element {
    let svg = doc.create_element_ns(Some("http://www.w3.org/2000/svg"), "svg").unwrap();
    let _ = svg.set_attribute("class", &format!("icon icon-{symbol}"));
    let _ = svg.set_attribute("aria-hidden", "true");
    let _ = svg.set_attribute("viewBox", "0 0 24 24");
    let u = doc.create_element_ns(Some("http://www.w3.org/2000/svg"), "use").unwrap();
    let _ = u.set_attribute("href", &format!("#icon-{symbol}"));
    let _ = svg.append_child(&u);
    svg
}

pub fn attribute_icon(name: &str) -> &'static str {
    let lower = name.to_lowercase();
    if lower.contains("hp") || lower.contains("health") || lower.contains("recovery") || lower.contains("constitution") {
        "health"
    } else if lower.contains("defense") || lower.contains("tolerance") || lower.contains("resist") || lower.contains("block") || lower.contains("evasion") || lower.contains("endurance") {
        "defense"
    } else if lower.contains("chance") || lower.contains("critical") || lower.contains("accuracy") || lower.contains("precision") || lower.contains("multi-hit") {
        "chance"
    } else if lower.contains("damage") || lower.contains("boost") || lower.contains("coefficient") {
        "damage"
    } else if lower.contains("attack") || lower.contains("penetration") || lower.contains("shard") || lower.contains("might") {
        "attack"
    } else {
        "stat"
    }
}

fn close_tree_tip() {
    let doc = document();
    if let Some(tip) = doc.get_element_by_id("tree-tip") {
        let _ = tip.set_attribute("hidden", "");
    }
    TOOLTIP_STATE.with(|state| {
        let mut st = state.borrow_mut();
        if let Some(btn) = st.active_button.take() {
            let _ = btn.remove_attribute("aria-describedby");
        }
        st.pinned = false;
        st.active_title.clear();
        st.active_text.clear();
    });
}

fn position_tree_tip(button: &Element) {
    let doc = document();
    let win = window();
    let Some(tip) = doc.get_element_by_id("tree-tip").and_then(|e| e.dyn_into::<HtmlElement>().ok()) else { return };
    let rect = button.get_bounding_client_rect();
    let width = tip.offset_width() as f64;
    let height = tip.offset_height() as f64;
    let win_w = win.inner_width().ok().and_then(|v| v.as_f64()).unwrap_or(1024.0);
    let win_h = win.inner_height().ok().and_then(|v| v.as_f64()).unwrap_or(768.0);

    let left = 12.0_f64.max((rect.left() + rect.width() / 2.0 - width / 2.0).min(win_w - width - 12.0));
    let above = rect.top() - height - 10.0;
    let top = if above >= 12.0 {
        above
    } else {
        (rect.bottom() + 10.0).min(win_h - height - 12.0)
    };
    let _ = tip.style().set_property("left", &format!("{}px", left.max(12.0)));
    let _ = tip.style().set_property("top", &format!("{}px", top.max(12.0)));
}

fn show_tree_tip(button: &Element, title: &str, text: &str) {
    let doc = document();
    if let Some(title_el) = doc.get_element_by_id("tree-tip-title") {
        title_el.set_text_content(Some(title));
    }
    if let Some(text_el) = doc.get_element_by_id("tree-tip-text") {
        text_el.set_text_content(Some(text));
    }
    if let Some(tip) = doc.get_element_by_id("tree-tip") {
        let _ = tip.remove_attribute("hidden");
    }
    let _ = button.set_attribute("aria-describedby", "tree-tip");
    position_tree_tip(button);

    TOOLTIP_STATE.with(|s| {
        let mut st = s.borrow_mut();
        st.active_title = title.to_string();
        st.active_text = text.to_string();
    });
}

fn bind_tree_tip<F>(button: &Element, on_select: F)
where
    F: Fn(&Element) + 'static,
{
    let on_select = std::rc::Rc::new(on_select);

    let open_action = {
        let button = button.clone();
        let on_select = on_select.clone();
        move || {
            TOOLTIP_STATE.with(|state| {
                let mut st = state.borrow_mut();
                if let Some(prev) = st.active_button.take() {
                    let _ = prev.remove_attribute("aria-describedby");
                }
                st.active_button = Some(button.clone());
            });
            on_select(&button);
        }
    };
    let open_rc = std::rc::Rc::new(open_action);

    // mouseenter
    {
        let open = open_rc.clone();
        let closure = Closure::<dyn FnMut()>::new(move || {
            let is_pinned = TOOLTIP_STATE.with(|s| s.borrow().pinned);
            if !is_pinned {
                open();
            }
        });
        let _ = button.add_event_listener_with_callback("mouseenter", closure.as_ref().unchecked_ref());
        closure.forget();
    }

    // focus
    {
        let open = open_rc.clone();
        let closure = Closure::<dyn FnMut()>::new(move || {
            let is_pinned = TOOLTIP_STATE.with(|s| s.borrow().pinned);
            if !is_pinned {
                open();
            }
        });
        let _ = button.add_event_listener_with_callback("focus", closure.as_ref().unchecked_ref());
        closure.forget();
    }

    // mouseleave
    {
        let btn = button.clone();
        let closure = Closure::<dyn FnMut()>::new(move || {
            let should_close = TOOLTIP_STATE.with(|s| {
                let st = s.borrow();
                !st.pinned && st.active_button.as_ref().is_some_and(|b| b == &btn)
            });
            if should_close {
                close_tree_tip();
            }
        });
        let _ = button.add_event_listener_with_callback("mouseleave", closure.as_ref().unchecked_ref());
        closure.forget();
    }

    // blur
    {
        let btn = button.clone();
        let closure = Closure::<dyn FnMut()>::new(move || {
            let should_close = TOOLTIP_STATE.with(|s| {
                let st = s.borrow();
                !st.pinned && st.active_button.as_ref().is_some_and(|b| b == &btn)
            });
            if should_close {
                close_tree_tip();
            }
        });
        let _ = button.add_event_listener_with_callback("blur", closure.as_ref().unchecked_ref());
        closure.forget();
    }

    // click
    {
        let btn = button.clone();
        let open = open_rc.clone();
        let closure = Closure::<dyn FnMut()>::new(move || {
            let (is_pinned, is_active) = TOOLTIP_STATE.with(|s| {
                let st = s.borrow();
                (st.pinned, st.active_button.as_ref().is_some_and(|b| b == &btn))
            });
            if is_pinned && is_active {
                close_tree_tip();
            } else {
                open();
                TOOLTIP_STATE.with(|s| s.borrow_mut().pinned = true);
            }
        });
        let _ = button.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref());
        closure.forget();
    }
}

fn tree_tip_text(stat: &str, lang: &str) -> String {
    let found = LEAVES.iter().find(|(st, _, _, _, _, _)| *st == stat);
    if let Some((_, _, _, Some(tip_key), _, _)) = found {
        t(lang, tip_key)
    } else {
        stat_info(stat, lang == "pt")
            .map(|i| i.desc)
            .unwrap_or_default()
    }
}

fn init_global_tip_listeners(doc: &Document, win: &Window) {
    // pointerdown: click outside active button closes tip
    let on_pointerdown = Closure::<dyn FnMut(web_sys::PointerEvent)>::new(move |event: web_sys::PointerEvent| {
        let target = event.target().and_then(|t| t.dyn_into::<web_sys::Node>().ok());
        let should_close = TOOLTIP_STATE.with(|s| {
            let st = s.borrow();
            if let (Some(btn), Some(t)) = (st.active_button.as_ref(), target.as_ref()) {
                !btn.contains(Some(t))
            } else {
                false
            }
        });
        if should_close {
            close_tree_tip();
        }
    });
    let _ = doc.add_event_listener_with_callback("pointerdown", on_pointerdown.as_ref().unchecked_ref());
    on_pointerdown.forget();

    // keydown: Escape closes tip
    let on_keydown = Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(move |event: web_sys::KeyboardEvent| {
        if event.key() == "Escape" {
            close_tree_tip();
        }
    });
    let _ = doc.add_event_listener_with_callback("keydown", on_keydown.as_ref().unchecked_ref());
    on_keydown.forget();

    // scroll & resize: reposition tip
    let on_reposition = Closure::<dyn FnMut()>::new(move || {
        TOOLTIP_STATE.with(|s| {
            if let Some(btn) = s.borrow().active_button.as_ref() {
                position_tree_tip(btn);
            }
        });
    });
    let _ = win.add_event_listener_with_callback("scroll", on_reposition.as_ref().unchecked_ref());
    let _ = win.add_event_listener_with_callback("resize", on_reposition.as_ref().unchecked_ref());
    on_reposition.forget();
}

pub fn init_attributes() {
    let doc = document();
    let win = window();

    init_global_tip_listeners(&doc, &win);

    // Augment .tree-node in .attribute-trees
    let Ok(nodes) = doc.query_selector_all(".attribute-trees .tree-node, #attributes-view .tree-node") else { return };
    for i in 0..nodes.length() {
        let Some(node) = nodes.item(i).and_then(|n| n.dyn_into::<HtmlElement>().ok()) else { continue };
        let parent = node.parent_element();
        let stat = parent.as_ref().and_then(|p| p.get_attribute("data-tree-stat"))
            .or_else(|| node.get_attribute("data-tree-stat"));
        let icon_attr = parent.as_ref().and_then(|p| p.get_attribute("data-tree-icon"))
            .or_else(|| node.get_attribute("data-tree-icon"))
            .unwrap_or_else(|| "stat".into());
        let hint_attr = parent.as_ref().and_then(|p| p.get_attribute("data-tree-hint"))
            .or_else(|| node.get_attribute("data-tree-hint"))
            .unwrap_or_default();

        let (icon, hint_key) = if let Some(s) = &stat {
            let found = LEAVES.iter().find(|(st, _, _, _, _, _)| st == s);
            (
                found.map(|(_, ic, _, _, _, _)| *ic).unwrap_or("stat"),
                found.map(|(_, _, h, _, _, _)| *h).unwrap_or("treeHintAttack"),
            )
        } else {
            (icon_attr.as_str(), hint_attr.as_str())
        };

        let circle = doc.create_element(if stat.is_some() { "button" } else { "span" }).unwrap();
        let _ = circle.set_attribute("class", "tree-circle");
        if let Some(s) = &stat {
            let _ = circle.set_attribute("type", "button");
            let _ = circle.set_attribute("data-tree-stat", s);
        }
        let svg = create_icon_svg(&doc, icon);
        let _ = circle.append_child(&svg);

        if let Some(p) = node.parent_element() {
            let _ = p.insert_before(&circle, Some(&node));
        }
        let _ = node.class_list().remove_1("tree-node");
        let _ = node.class_list().add_1("tree-caption");

        let hint = doc.create_element("small").unwrap();
        let _ = hint.set_attribute("class", "tree-hint");
        let _ = hint.set_attribute("data-hint-key", hint_key);

        if let Some(s) = &stat {
            let value_span = doc.create_element("span").unwrap();
            let _ = value_span.set_attribute("class", "tree-value");

            if let Some(p) = node.parent_element() {
                let next = node.next_sibling();
                let _ = p.insert_before(&value_span, next.as_ref());
                let _ = p.insert_before(&hint, next.as_ref());
            }

            let stat_name = s.clone();
            bind_tree_tip(&circle, move |btn| {
                let lang = current_lang();
                let title = btn.parent_element()
                    .and_then(|p| p.query_selector(".tree-caption").ok().flatten())
                    .and_then(|c| c.text_content())
                    .unwrap_or_else(|| stat_name.clone());
                let tip = tree_tip_text(&stat_name, &lang);
                show_tree_tip(btn, title.trim(), &tip);
            });
        } else {
            if let Some(p) = node.parent_element() {
                let next = node.next_sibling();
                let _ = p.insert_before(&hint, next.as_ref());
            }
        }
    }

    // Populate catalog from ui/data/matchups.json
    if let Some(catalog_container) = doc.get_element_by_id("tree-catalog") {
        if catalog_container.children().length() == 0 {
            populate_catalog(&doc, &catalog_container);
        }
    }

    // Character import listener
    let on_imported = Closure::<dyn FnMut(web_sys::CustomEvent)>::new(move |event: web_sys::CustomEvent| {
        close_tree_tip();
        if let Ok(detail_str) = js_sys::JSON::stringify(&event.detail()) {
            let detail_string: String = detail_str.into();
            if let Ok(mut val) = serde_json::from_str::<Value>(&detail_string) {
                let doc = document();
                let class_val = doc.get_element_by_id("attacker-class")
                    .and_then(|e| e.dyn_into::<HtmlInputElement>().ok())
                    .map(|i| i.value())
                    .unwrap_or_default();
                val["class"] = serde_json::Value::String(class_val);
                IMPORTED_CHARACTER.with(|ic| *ic.borrow_mut() = Some(val));
            }
        }
        update_attribute_values();
        render_profile_stats();
    });
    let _ = doc.add_event_listener_with_callback("character-imported", on_imported.as_ref().unchecked_ref());
    on_imported.forget();

    update_attribute_values();
    render_profile_stats();
}

fn strip_parens(title: &str) -> &str {
    match title.find(" (") {
        Some(i) if title.ends_with(')') => &title[..i],
        _ => title,
    }
}

pub fn populate_catalog(doc: &Document, container: &Element) {
    container.set_inner_html("");
    let lang = current_lang();
    let is_pt = lang == "pt";

    let genus_names: Vec<String> = GENUS.iter()
        .flat_map(|g| GENUS_FIELDS.iter().map(move |f| format!("{g}{f}")))
        .collect();
    let genus_refs: Vec<&str> = genus_names.iter().map(|s| s.as_str()).collect();

    let groups: &[(&str, &[&str])] = &[
        ("treeGroupHit", &["accuracy", "evasion", "blockPenetration", "block", "pveAccuracy", "pvpAccuracy", "pvpEvasion", "pvpBlockPenetration", "pvpBlock"]),
        ("treeGroupAttack", &["pureAttack", "attackBonus", "attackIncrease", "weaponDamageBoost", "weaponDamageTolerance", "powerShard", "multiHit", "pveAttackPower", "bossAttackPower", "speciesAttackPower", "frontBackAttackPower", "pvpAttack"]),
        ("treeGroupSkill", &["skillCoefficient", "skillAdditionalDamage", "skillSpecialtyEnhancement", "penetration", "defense", "pvpDefense"]),
        ("treeGroupBoost", &["damageBoost", "damageTolerance", "pveDamageBoost", "pveDamageTolerance", "bossDamageBoost", "bossDamageTolerance", "speciesDamageBoost", "speciesDamageTolerance", "pvpDamageBoost", "pvpDamageTolerance"]),
        ("treeGroupCritical", &["criticalHit", "criticalHitResist", "criticalDamageBoost", "criticalDamageTolerance", "criticalAttack", "criticalDamageDefense", "pvpCriticalHit", "pvpCriticalHitResist"]),
        ("treeGroupProc", &["doubleChance", "doubleResist", "double", "perfectChance", "perfectResist", "multiHitChance", "multiHitResist", "endurancePenetration", "endurance", "regenerationPenetration", "regeneration"]),
        ("treeGroupPosition", &["frontBoost", "frontDamageTolerance", "backBoost", "backDamageTolerance", "frontAttack", "frontDefense", "backAttack", "backDefense", "frontCriticalHit", "frontCriticalHitResist", "backCriticalHit", "backCriticalHitResist"]),
        ("treeGroupElemental", &["elementalResistance", "fireBoost", "fireTolerance", "waterBoost", "waterTolerance", "windBoost", "windTolerance", "earthBoost", "earthTolerance"]),
        ("treeGroupGenus", &genus_refs),
        ("treeGroupHp", &["playerHp", "enemyHp", "hitsPerSecond"]),
    ];

    for (i, (key, names)) in groups.iter().enumerate() {
        let details = doc.create_element("details").unwrap();
        let _ = details.set_attribute("class", "tree-catalog-group");
        if i == 0 {
            let _ = details.set_attribute("open", "");
        }
        let summary = doc.create_element("summary").unwrap();
        summary.set_text_content(Some(&format!("{} ({})", t(&lang, key), names.len())));

        let ul = doc.create_element("ul").unwrap();
        let _ = ul.set_attribute("class", "tree-profile-list");

        for name in *names {
            let flags = MATCHUP_FLAGS.get(*name).copied();
            let is_enemy = flags.map(|f| f.enemy).unwrap_or(false);
            let percent = flags.map(|f| f.percent).unwrap_or(false);

            let info = stat_info(name, is_pt);
            let raw_title = info.as_ref().map(|i| strip_parens(&i.title)).unwrap_or(name);
            let localized_title = stat_label(&lang, raw_title);
            let final_title = if is_enemy {
                format!("{} · {}", t(&lang, "treeCatalogEnemy"), localized_title)
            } else {
                localized_title
            };

            let desc = info.as_ref().map(|i| i.desc.as_str()).unwrap_or("");
            let icon = attribute_icon(name);
            let val = get_catalog_value(doc, name, percent, &lang);

            let li = doc.create_element("li").unwrap();
            let btn = doc.create_element("button").unwrap();
            let _ = btn.set_attribute("type", "button");
            let _ = btn.set_attribute("data-stat-name", name);
            let _ = btn.set_attribute("data-stat-percent", if percent { "true" } else { "false" });
            let _ = btn.set_attribute("data-stat-title", &final_title);
            let _ = btn.set_attribute("data-stat-desc", desc);
            let _ = btn.set_attribute("aria-label", &format!("{final_title}: {val}. {desc}"));

            let svg = create_icon_svg(doc, icon);
            let span = doc.create_element("span").unwrap();
            span.set_text_content(Some(&final_title));
            let strong = doc.create_element("strong").unwrap();
            strong.set_text_content(Some(&val));

            let _ = btn.append_child(&svg);
            let _ = btn.append_child(&span);
            let _ = btn.append_child(&strong);

            let btn_title = final_title.clone();
            let btn_desc = desc.to_string();
            bind_tree_tip(&btn, move |b| {
                show_tree_tip(b, &btn_title, &btn_desc);
            });

            let _ = li.append_child(&btn);
            let _ = ul.append_child(&li);
        }

        let _ = details.append_child(&summary);
        let _ = details.append_child(&ul);
        let _ = container.append_child(&details);
    }
}

fn get_catalog_value(doc: &Document, name: &str, percent: bool, lang: &str) -> String {
    let input = doc.query_selector(&format!("#calculator input[name=\"{name}\"]")).ok().flatten()
        .and_then(|n| n.dyn_into::<HtmlInputElement>().ok());
    let val_str = input.as_ref().map(|i| i.value()).unwrap_or_default();
    let min = input.as_ref().and_then(|i| {
        let m = i.min();
        if m.is_empty() { None } else { m.parse::<f64>().ok() }
    }).unwrap_or(f64::NEG_INFINITY);

    if let Some(num) = parse_numeric_input(&val_str, min) {
        let formatted = format_number(num, lang == "pt");
        format!("{}{}", formatted, if percent { "%" } else { "" })
    } else {
        t(lang, "treeUnknown")
    }
}

pub fn update_attribute_values() {
    let doc = document();
    let lang = current_lang();
    let is_pt = lang == "pt";

    let character_name = doc.get_element_by_id("attacker-name")
        .and_then(|e| e.dyn_into::<HtmlInputElement>().ok())
        .map(|i| i.value())
        .unwrap_or_default();
    let character_class = doc.get_element_by_id("attacker-class")
        .and_then(|e| e.dyn_into::<HtmlInputElement>().ok())
        .map(|i| i.value())
        .unwrap_or_default();

    let (same_profile, imported_name, imported_defenses) = IMPORTED_CHARACTER.with(|ic| {
        if let Some(val) = &*ic.borrow() {
            let name = val.get("name").and_then(Value::as_str).unwrap_or("");
            let class = val.get("class").and_then(Value::as_str).unwrap_or("");
            let same = !name.is_empty() && name == character_name && (class.is_empty() || class == character_class);
            let defenses = val.get("defenses").cloned().unwrap_or(Value::Null);
            (same, name.to_string(), defenses)
        } else {
            (false, String::new(), Value::Null)
        }
    });

    // Update tree nodes
    let Ok(leaves) = doc.query_selector_all(".attribute-trees li[data-tree-stat], #attributes-view li[data-tree-stat]") else { return };
    for i in 0..leaves.length() {
        let Some(li) = leaves.item(i).and_then(|n| n.dyn_into::<HtmlElement>().ok()) else { continue };
        let Some(stat) = li.get_attribute("data-tree-stat") else { continue };

        let field_info = LEAVES.iter().find(|(st, _, _, _, _, _)| *st == stat);
        let (is_percent, is_profile) = field_info.map(|(_, _, _, _, p, pr)| (*p, *pr)).unwrap_or((false, false));

        let num_opt = if is_profile && same_profile {
            imported_defenses.get(&stat).and_then(|v| {
                if let Some(n) = v.as_f64() {
                    Some(n)
                } else if let Some(s) = v.as_str() {
                    parse_numeric_input(s, f64::NEG_INFINITY)
                } else {
                    None
                }
            })
        } else {
            let input_val = doc.query_selector(&format!("#calculator input[name=\"{stat}\"]"))
                .ok().flatten()
                .and_then(|n| n.dyn_into::<HtmlInputElement>().ok())
                .map(|inp| inp.value());
            input_val.as_deref().and_then(|v| parse_numeric_input(v, f64::NEG_INFINITY))
        };

        let label = match num_opt {
            Some(num) if !(stat == "playerHp" && num <= 0.0) => {
                let formatted = format_number(num, is_pt);
                format!("{}{}", formatted, if is_percent { "%" } else { "" })
            }
            _ => t(&lang, "treeUnknown"),
        };

        if let Some(val_span) = li.query_selector(".tree-value").ok().flatten().and_then(|n| n.dyn_into::<HtmlElement>().ok()) {
            val_span.set_text_content(Some(&label));
        }

        let caption = li.query_selector(".tree-caption").ok().flatten()
            .and_then(|c| c.text_content())
            .unwrap_or_else(|| stat.clone());
        let tip = tree_tip_text(&stat, &lang);
        if let Some(btn) = li.query_selector("button").ok().flatten() {
            let _ = btn.set_attribute("aria-label", &format!("{}: {label}. {tip}", caption.trim()));
        }
    }

    // Update hints
    if let Ok(hints) = doc.query_selector_all(".attribute-trees .tree-hint, #attributes-view .tree-hint") {
        for i in 0..hints.length() {
            if let Some(hint_el) = hints.item(i).and_then(|n| n.dyn_into::<HtmlElement>().ok()) {
                if let Some(key) = hint_el.get_attribute("data-hint-key") {
                    hint_el.set_text_content(Some(&t(&lang, &key)));
                }
            }
        }
    }

    // Update value note
    if let Some(note) = doc.get_element_by_id("tree-value-note") {
        let note_text = if same_profile {
            let template = t(&lang, "treeImportedFrom");
            template.replace("{name}", &imported_name)
        } else {
            t(&lang, "treeValueNote")
        };
        note.set_text_content(Some(&note_text));
    }

    // Update catalog buttons
    if let Ok(buttons) = doc.query_selector_all("#tree-catalog button[data-stat-name]") {
        for i in 0..buttons.length() {
            let Some(btn) = buttons.item(i).and_then(|n| n.dyn_into::<HtmlElement>().ok()) else { continue };
            let Some(name) = btn.get_attribute("data-stat-name") else { continue };
            let percent = btn.get_attribute("data-stat-percent").as_deref() == Some("true");
            let val = get_catalog_value(&doc, &name, percent, &lang);

            if let Some(strong) = btn.query_selector("strong").ok().flatten() {
                strong.set_text_content(Some(&val));
            }

            let title = btn.get_attribute("data-stat-title").unwrap_or_default();
            let desc = btn.get_attribute("data-stat-desc").unwrap_or_default();
            let _ = btn.set_attribute("aria-label", &format!("{title}: {val}. {desc}"));
        }
    }
}

pub fn render_profile_stats() {
    let doc = document();
    let lang = current_lang();
    let is_pt = lang == "pt";

    let character_name = doc.get_element_by_id("attacker-name")
        .and_then(|e| e.dyn_into::<HtmlInputElement>().ok())
        .map(|i| i.value())
        .unwrap_or_default();
    let character_class = doc.get_element_by_id("attacker-class")
        .and_then(|e| e.dyn_into::<HtmlInputElement>().ok())
        .map(|i| i.value())
        .unwrap_or_default();

    let (_same_profile, stats, modifiers) = IMPORTED_CHARACTER.with(|ic| {
        if let Some(val) = &*ic.borrow() {
            let name = val.get("name").and_then(Value::as_str).unwrap_or("");
            let class = val.get("class").and_then(Value::as_str).unwrap_or("");
            let same = !name.is_empty() && name == character_name && (class.is_empty() || class == character_class);
            let stats = if same { val.get("stats").and_then(Value::as_array).cloned().unwrap_or_default() } else { vec![] };
            let modifiers = if same { val.get("modifiers").cloned().unwrap_or(Value::Null) } else { Value::Null };
            (same, stats, modifiers)
        } else {
            (false, vec![], Value::Null)
        }
    });

    if let Some(empty_el) = doc.get_element_by_id("tree-profile-empty") {
        let _ = empty_el.set_attribute("hidden", if stats.is_empty() { "false" } else { "true" });
        if !stats.is_empty() {
            let _ = empty_el.set_attribute("hidden", "");
        } else {
            let _ = empty_el.remove_attribute("hidden");
        }
    }

    if let Some(list) = doc.get_element_by_id("tree-profile-list") {
        list.set_inner_html("");
        for stat in &stats {
            let name = stat.get("name").and_then(Value::as_str).unwrap_or("");
            let val_str = stat.get("value").map(|v| v.to_string().trim_matches('"').to_string()).unwrap_or_default();
            let is_percent = val_str.ends_with('%');
            let num_clean = val_str.trim_end_matches('%');
            let val_display = if let Ok(num) = num_clean.parse::<f64>() {
                format!("{}{}", format_number(num, is_pt), if is_percent { "%" } else { "" })
            } else {
                val_str.clone()
            };

            let title = stat_label(&lang, name);
            let info = stat_info(name, is_pt);
            let desc = info.as_ref().map(|i| i.desc.as_str()).unwrap_or("");
            let icon = attribute_icon(name);

            let li = doc.create_element("li").unwrap();
            let btn = doc.create_element("button").unwrap();
            let _ = btn.set_attribute("type", "button");
            let _ = btn.set_attribute("aria-label", &format!("{title}: {val_display}. {desc}"));

            let svg = create_icon_svg(&doc, icon);
            let span = doc.create_element("span").unwrap();
            span.set_text_content(Some(&title));
            let strong = doc.create_element("strong").unwrap();
            strong.set_text_content(Some(&val_display));

            let _ = btn.append_child(&svg);
            let _ = btn.append_child(&span);
            let _ = btn.append_child(&strong);

            let btn_title = title.clone();
            let btn_desc = desc.to_string();
            bind_tree_tip(&btn, move |b| {
                show_tree_tip(b, &btn_title, &btn_desc);
            });

            let _ = li.append_child(&btn);
            let _ = list.append_child(&li);
        }
    }

    // Modifiers
    if let Some(mod_block) = doc.get_element_by_id("tree-modifier-block") {
        let mod_map = modifiers.as_object();
        let has_mods = mod_map.map(|m| !m.is_empty()).unwrap_or(false);
        if has_mods {
            let _ = mod_block.remove_attribute("hidden");
        } else {
            let _ = mod_block.set_attribute("hidden", "");
        }

        if let (Some(m_list), Some(map)) = (doc.get_element_by_id("tree-modifier-list"), mod_map) {
            m_list.set_inner_html("");
            for (field, amount_val) in map {
                let amount = amount_val.as_f64().unwrap_or(0.0);
                let mut cap = field.clone();
                if let Some(first) = cap.get_mut(0..1) {
                    first.make_ascii_uppercase();
                }
                let title = t(&lang, &format!("modifier{cap}"));
                let desc = t(&lang, "sheetModifiersNote");
                let val_display = format!("{}%", format_number(amount, is_pt));
                let icon = attribute_icon(field);

                let li = doc.create_element("li").unwrap();
                let btn = doc.create_element("button").unwrap();
                let _ = btn.set_attribute("type", "button");
                let _ = btn.set_attribute("aria-label", &format!("{title}: {val_display}. {desc}"));

                let svg = create_icon_svg(&doc, icon);
                let span = doc.create_element("span").unwrap();
                span.set_text_content(Some(&title));
                let strong = doc.create_element("strong").unwrap();
                strong.set_text_content(Some(&val_display));

                let _ = btn.append_child(&svg);
                let _ = btn.append_child(&span);
                let _ = btn.append_child(&strong);

                let btn_title = title.clone();
                let btn_desc = desc.clone();
                bind_tree_tip(&btn, move |b| {
                    show_tree_tip(b, &btn_title, &btn_desc);
                });

                let _ = li.append_child(&btn);
                let _ = m_list.append_child(&li);
            }
        }
    }
}
