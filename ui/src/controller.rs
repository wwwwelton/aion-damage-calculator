use wasm_bindgen::prelude::*;
use web_sys::{Document, Element, HtmlElement, HtmlInputElement, HtmlSelectElement, Window};
use serde_json::Value;
use std::sync::LazyLock;
use aion2_core::calculator::{calculate_damage, estimate_combat, estimate_assumptions, attribute_duels, apply_target_mode, Position, TargetMode, Values};
use aion2_core::recommendations::{rank_damage_upgrades, Stat};
use aion2_core::numeric_input::parse_numeric_input;

static I18N_RAW: LazyLock<Value> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../data/i18n.json")).expect("i18n.json")
});

static MATCHUPS_RAW: LazyLock<Value> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../data/matchups.json")).expect("matchups.json")
});
pub mod attributes;
pub mod companion;
pub mod ai;
pub mod characters;
pub mod bosses;
pub mod character_search;

pub(crate) fn window() -> Window { web_sys::window().expect("window") }
pub(crate) fn document() -> Document { window().document().expect("document") }

pub(crate) fn t(lang: &str, key: &str) -> String {
    let map = I18N_RAW.get(lang).and_then(Value::as_object);
    if let Some(val) = map.and_then(|m| m.get(key)).and_then(Value::as_str) {
        return val.to_string();
    }
    if let Some(val) = I18N_RAW["pt"].get(key).and_then(Value::as_str) {
        return val.to_string();
    }
    key.to_string()
}

pub(crate) fn stat_label(lang: &str, label: &str) -> String {
    if lang != "pt" { return label.to_string(); }
    let mut res = label.to_string();
    if let Some(reps) = I18N_RAW["ptStatReplacements"].as_array() {
        for pair in reps {
            if let (Some(from), Some(to)) = (pair[0].as_str(), pair[1].as_str()) {
                res = res.replace(from, to);
            }
        }
    }
    res
}

pub(crate) fn current_lang() -> String {
    if let Some(storage) = window().local_storage().ok().flatten() {
        if let Ok(Some(saved)) = storage.get_item("aion2_calculator_lang") {
            if saved == "en" || saved == "pt" { return saved; }
        }
    }
    "pt".to_string()
}

pub fn apply_translations(lang: &str) {
    let doc = document();
    let _ = doc.document_element().map(|el| el.set_attribute("lang", if lang == "pt" { "pt-BR" } else { "en" }));
    doc.set_title(&t(lang, "windowTitle"));

    if let Ok(nodes) = doc.query_selector_all("[data-i18n]") {
        for i in 0..nodes.length() {
            if let Some(el) = nodes.item(i).and_then(|n| n.dyn_into::<HtmlElement>().ok()) {
                if let Some(key) = el.get_attribute("data-i18n") {
                    el.set_text_content(Some(&t(lang, &key)));
                }
            }
        }
    }
    if let Ok(nodes) = doc.query_selector_all("[data-i18n-label]") {
        for i in 0..nodes.length() {
            if let Some(el) = nodes.item(i).and_then(|n| n.dyn_into::<Element>().ok()) {
                if let Some(key) = el.get_attribute("data-i18n-label") {
                    let translated = t(lang, &key);
                    if let Some(first_child) = el.first_child() {
                        if first_child.node_type() == 3 {
                            first_child.set_node_value(Some(&format!("{translated} ")));
                            continue;
                        }
                    }
                    if let Ok(text_node) = doc.create_text_node(&format!("{translated} ")).dyn_into::<web_sys::Node>() {
                        let _ = el.insert_before(&text_node, el.first_child().as_ref());
                    }
                }
            }
        }
    }
    if let Ok(nodes) = doc.query_selector_all("[data-i18n-placeholder]") {
        for i in 0..nodes.length() {
            if let Some(input) = nodes.item(i).and_then(|n| n.dyn_into::<HtmlInputElement>().ok()) {
                if let Some(key) = input.get_attribute("data-i18n-placeholder") {
                    input.set_placeholder(&t(lang, &key));
                }
            }
        }
    }
}

pub fn init_tabs() {
    let doc = document();
    let Ok(tabs) = doc.query_selector_all(".tabs .tab") else { return };
    for i in 0..tabs.length() {
        if let Some(tab_btn) = tabs.item(i).and_then(|n| n.dyn_into::<HtmlElement>().ok()) {
            let tab_id = tab_btn.get_attribute("data-tab").unwrap_or_default();
            let closure = Closure::<dyn FnMut()>::new(move || {
                let doc = document();
                let Ok(all_tabs) = doc.query_selector_all(".tabs .tab") else { return };
                for j in 0..all_tabs.length() {
                    if let Some(b) = all_tabs.item(j).and_then(|n| n.dyn_into::<HtmlElement>().ok()) {
                        let target = b.get_attribute("data-tab").unwrap_or_default();
                        let active = target == tab_id;
                        let _ = b.class_list().toggle_with_force("active", active);
                        let _ = b.set_attribute("aria-selected", if active { "true" } else { "false" });
                        if let Some(panel) = doc.get_element_by_id(&target).and_then(|p| p.dyn_into::<HtmlElement>().ok()) {
                            panel.set_hidden(!active);
                        }
                    }
                }
            });
            tab_btn.set_onclick(Some(closure.as_ref().unchecked_ref()));
            closure.forget();
        }
    }
}

fn attribute_icon(name: &str) -> &'static str {
    let lower = name.to_lowercase();
    if lower.contains("hp") || lower.contains("health") || lower.contains("recovery") || lower.contains("constitution") { "health" }
    else if lower.contains("defense") || lower.contains("tolerance") || lower.contains("resist") || lower.contains("block") || lower.contains("evasion") || lower.contains("endurance") { "defense" }
    else if lower.contains("chance") || lower.contains("critical") || lower.contains("accuracy") || lower.contains("precision") || lower.contains("multi-hit") { "chance" }
    else if lower.contains("damage") || lower.contains("boost") || lower.contains("coefficient") { "damage" }
    else if lower.contains("attack") || lower.contains("penetration") || lower.contains("shard") || lower.contains("might") { "attack" }
    else { "stat" }
}

pub fn init_calculator() {
    let doc = document();
    let Ok(labels) = doc.query_selector_all("#calculator label") else { return };
    for i in 0..labels.length() {
        if let Some(label) = labels.item(i).and_then(|n| n.dyn_into::<HtmlElement>().ok()) {
            if label.query_selector(".icon").ok().flatten().is_some() { continue; }
            if let Some(input) = label.query_selector("input").ok().flatten().and_then(|n| n.dyn_into::<HtmlInputElement>().ok()) {
                let name = input.name();
                let icon = attribute_icon(&name);
                let svg = doc.create_element_ns(Some("http://www.w3.org/2000/svg"), "svg").unwrap();
                let _ = svg.set_attribute("class", &format!("icon icon-{icon}"));
                let _ = svg.set_attribute("aria-hidden", "true");
                let _ = svg.set_attribute("viewBox", "0 0 24 24");
                let u = doc.create_element_ns(Some("http://www.w3.org/2000/svg"), "use").unwrap();
                let _ = u.set_attribute("href", &format!("#icon-{icon}"));
                let _ = svg.append_child(&u);
                let _ = label.append_child(&svg);
            }
        }
    }

    if let Some(creature_container) = doc.get_element_by_id("creature-matchups") {
        if creature_container.children().length() == 0 {
            populate_matchup_groups(&doc, &creature_container, true);
        }
    }
    if let Some(ref_container) = doc.get_element_by_id("reference-matchups") {
        if ref_container.children().length() == 0 {
            populate_matchup_groups(&doc, &ref_container, false);
        }
    }

    let update = Closure::<dyn FnMut()>::new(move || {
        recalculate();
    });
    if let Some(form) = doc.get_element_by_id("calculator").and_then(|e| e.dyn_into::<HtmlElement>().ok()) {
        form.set_oninput(Some(update.as_ref().unchecked_ref()));
        form.set_onchange(Some(update.as_ref().unchecked_ref()));
    }
    if let Some(select) = doc.get_element_by_id("target-mode").and_then(|e| e.dyn_into::<HtmlSelectElement>().ok()) {
        let mode_change = Closure::<dyn FnMut()>::new(move || {
            let doc = document();
            let mode = doc.get_element_by_id("target-mode").and_then(|e| e.dyn_into::<HtmlSelectElement>().ok()).map(|s| s.value()).unwrap_or_else(|| "boss".into());
            let is_player = mode == "player";
            if let Some(view) = doc.get_element_by_id("calculator-view") {
                let _ = view.set_attribute("data-mode", &mode);
            }
            if let Some(l) = doc.get_element_by_id("target-kind-label").and_then(|e| e.dyn_into::<HtmlElement>().ok()) {
                l.set_hidden(is_player);
            }
            if let Some(l) = doc.get_element_by_id("target-class-label").and_then(|e| e.dyn_into::<HtmlElement>().ok()) {
                l.set_hidden(!is_player);
            }
            recalculate();
        });
        select.set_onchange(Some(mode_change.as_ref().unchecked_ref()));
        mode_change.forget();
    }
    update.forget();
    recalculate();
}

fn populate_matchup_groups(doc: &Document, container: &Element, is_pve: bool) {
    let lang = current_lang();
    if let Some(groups) = MATCHUPS_RAW.as_array() {
        for group in groups {
            let mode = group.get("mode").and_then(Value::as_str).unwrap_or("");
            if (is_pve && mode != "pve") || (!is_pve && mode == "pve") { continue; }
            let details = doc.create_element("details").unwrap();
            let class_name = if mode.is_empty() { "reference-group" } else if mode == "pve" { "reference-group pve-only" } else { "reference-group player-only" };
            let _ = details.set_attribute("class", class_name);
            if group.get("open").and_then(Value::as_bool).unwrap_or(false) {
                let _ = details.set_attribute("open", "");
            }
            let summary = doc.create_element("summary").unwrap();
            let title = if lang == "pt" { group["titlePt"].as_str().unwrap_or("") } else { group["title"].as_str().unwrap_or("") };
            summary.set_text_content(Some(title));
            let note = doc.create_element("p").unwrap();
            let note_text = if lang == "pt" { group["notePt"].as_str().unwrap_or("") } else { group["note"].as_str().unwrap_or("") };
            note.set_text_content(Some(note_text));
            let grid = doc.create_element("div").unwrap();
            let _ = grid.set_attribute("class", "matchup-grid");

            if let Some(rows) = group.get("rows").and_then(Value::as_array) {
                for row in rows {
                    let a_label = row[0].as_str().unwrap_or("");
                    let a_field = row[1].as_str().unwrap_or("");
                    let d_label = row[2].as_str().unwrap_or("");
                    let d_field = row[3].as_str().unwrap_or("");
                    let pct = row.get(4) == Some(&Value::Bool(true));

                    let pair = doc.create_element("div").unwrap();
                    let _ = pair.set_attribute("class", "matchup");

                    for (label, field, is_enemy) in [(a_label, a_field, false), (d_label, d_field, true)] {
                        let wrapper = doc.create_element("label").unwrap();
                        if is_enemy { let _ = wrapper.set_attribute("class", "enemy"); }
                        let pfx = if is_enemy { if lang == "pt" { "Inimigo " } else { "Enemy " } } else { "" };
                        let sfx = if pct { " (%)" } else { "" };
                        let lbl_text = format!("{pfx}{}{sfx}", stat_label(&lang, label));
                        wrapper.set_text_content(Some(&lbl_text));
                        let input = doc.create_element("input").unwrap();
                        let _ = input.set_attribute("name", field);
                        let _ = input.set_attribute("type", "number");
                        let _ = input.set_attribute("value", "0");
                        let _ = input.set_attribute("step", "any");
                        let _ = wrapper.append_child(&input);
                        let _ = pair.append_child(&wrapper);
                    }
                    let _ = grid.append_child(&pair);
                }
            }
            let _ = details.append_child(&summary);
            let _ = details.append_child(&note);
            let _ = details.append_child(&grid);
            let _ = container.append_child(&details);
        }
    }
}

pub fn recalculate() {
    let doc = document();
    let lang = current_lang();
    let pt = lang == "pt";
    let Ok(inputs) = doc.query_selector_all("#calculator input") else { return };
    let mut raw = std::collections::HashMap::new();
    for i in 0..inputs.length() {
        if let Some(inp) = inputs.item(i).and_then(|n| n.dyn_into::<HtmlInputElement>().ok()) {
            let name = inp.name();
            if !name.is_empty() {
                let val = parse_numeric_input(&inp.value(), f64::NEG_INFINITY).unwrap_or(0.0);
                let pct = ["weaponDamageBoost", "multiHit", "powerShard", "attackIncrease", "skillCoefficient", "damageBoost", "pveDamageBoost", "bossDamageBoost", "bossDamageTolerance", "speciesDamageBoost", "criticalDamageBoost", "weaponDamageTolerance", "damageTolerance", "pveDamageTolerance", "speciesDamageTolerance", "criticalDamageTolerance", "frontDamageTolerance", "backDamageTolerance", "double", "frontBoost", "backBoost", "skillSpecialtyEnhancement"].contains(&name.as_str());
                raw.insert(name, if pct { val / 100.0 } else { val });
            }
        }
    }
    let get = |k: &str| raw.get(k).copied().unwrap_or(0.0);

    let mode_str = doc.get_element_by_id("target-mode").and_then(|e| e.dyn_into::<HtmlSelectElement>().ok()).map(|s| s.value()).unwrap_or_else(|| "boss".into());
    let mode = match mode_str.as_str() { "monster" => TargetMode::Monster, "player" => TargetMode::Player, _ => TargetMode::Boss };
    let crit = doc.get_element_by_id("critical").and_then(|e| e.dyn_into::<HtmlSelectElement>().ok()).map(|s| s.value() == "yes").unwrap_or(false);
    let double_en = doc.get_element_by_id("doubleEnabled").and_then(|e| e.dyn_into::<HtmlSelectElement>().ok()).map(|s| s.value() == "yes").unwrap_or(false);
    let pos = match doc.get_element_by_id("position").and_then(|e| e.dyn_into::<HtmlSelectElement>().ok()).map(|s| s.value()).as_deref() {
        Some("front") => Position::Front, Some("back") => Position::Back, _ => Position::None
    };

    let mut v = Values {
        pure_attack: get("pureAttack"), weapon_damage_boost: get("weaponDamageBoost"), weapon_damage_tolerance: get("weaponDamageTolerance"),
        multi_hit: get("multiHit"), power_shard: get("powerShard"), attack_bonus: get("attackBonus"), attack_increase: get("attackIncrease"),
        pve_attack_power: get("pveAttackPower"), boss_attack_power: get("bossAttackPower"), species_attack_power: get("speciesAttackPower"),
        front_back_attack_power: get("frontBackAttackPower"), skill_coefficient: get("skillCoefficient"), skill_additional_damage: get("skillAdditionalDamage"),
        defense: get("defense"), penetration: get("penetration"), skill_specialty_enhancement: get("skillSpecialtyEnhancement"),
        damage_boost: get("damageBoost"), damage_tolerance: get("damageTolerance"), pve_damage_boost: get("pveDamageBoost"),
        pve_damage_tolerance: get("pveDamageTolerance"), species_damage_boost: get("speciesDamageBoost"), species_damage_tolerance: get("speciesDamageTolerance"),
        boss_damage_boost: get("bossDamageBoost"), boss_damage_tolerance: get("bossDamageTolerance"),
        critical_damage_boost: if crit { get("criticalDamageBoost") } else { 1.0 },
        critical_damage_tolerance: if crit { get("criticalDamageTolerance") } else { 0.0 },
        double: if double_en { get("double") } else { 1.0 },
        front_back_damage_boost: match pos { Position::Front => get("frontBoost"), Position::Back => get("backBoost"), Position::None => 1.0 },
        front_back_damage_tolerance: match pos { Position::Front => get("frontDamageTolerance"), Position::Back => get("backDamageTolerance"), Position::None => 0.0 },
        ..Values::default()
    };
    if let Some(t_type) = doc.get_element_by_id("creature-type").and_then(|e| e.dyn_into::<HtmlSelectElement>().ok()).map(|s| s.value()).filter(|s| !s.is_empty()) {
        v.species_damage_boost = get(&format!("{t_type}DamageBoost"));
        v.species_damage_tolerance = get(&format!("{t_type}DamageTolerance"));
    }

    let scenario = apply_target_mode(&v, mode);
    let damage = calculate_damage(&scenario);
    let hps = get("hitsPerSecond");
    let hp = get("enemyHp");
    let combat = estimate_combat(damage, hps, hp);

    let fmt_num = |val: f64| {
        let (grp, dec) = if pt { ('.', ',') } else { (',', '.') };
        let fixed = format!("{val:.2}");
        let (int_p, fr) = fixed.split_once('.').unwrap_or((&fixed, ""));
        let mut grouped = String::new();
        for (idx, c) in int_p.chars().enumerate() {
            if idx > 0 && (int_p.len() - idx) % 3 == 0 { grouped.push(grp); }
            grouped.push(c);
        }
        if fr == "00" { grouped } else { format!("{grouped}{dec}{fr}") }
    };

    if let Some(res) = doc.get_element_by_id("result") {
        res.set_text_content(Some(&fmt_num(damage)));
    }
    if let Some(dps) = doc.get_element_by_id("estimated-dps") {
        dps.set_text_content(Some(&combat.dps.map(fmt_num).unwrap_or_else(|| "—".into())));
    }
    if let Some(ttd) = doc.get_element_by_id("estimated-time") {
        ttd.set_text_content(Some(&combat.seconds_to_defeat.map(|s| format!("{} s", fmt_num(s))).unwrap_or_else(|| "—".into())));
    }

    if let Some(upgrades_list) = doc.get_element_by_id("upgrade-list") {
        upgrades_list.set_inner_html("");
        let upgrades = rank_damage_upgrades(&scenario, crit, pos);
        for u in upgrades.iter().take(3) {
            let li = doc.create_element("li").unwrap();
            let label = match u.stat {
                Stat::WeaponDamageBoost => t(&lang, "weaponDamageBoostLabel"),
                Stat::DamageBoost => t(&lang, "damageBoostLabel"),
                Stat::CriticalDamageBoost => t(&lang, "criticalDamageBoostLabel"),
                Stat::FrontBackDamageBoost => if pos == Position::Back { t(&lang, "backBoostLabel") } else { t(&lang, "frontBoostLabel") },
            };
            let label_clean = label.trim_end_matches(" (%)").trim();
            li.set_text_content(Some(&format!("{label_clean}: +{:.2}% de dano", u.percent)));
            let _ = upgrades_list.append_child(&li);
        }
    }

    if let Some(duels_tbody) = doc.get_element_by_id("duel-rows") {
        duels_tbody.set_inner_html("");
        let duels = attribute_duels(&scenario, mode, crit, pos);
        for d in duels {
            let tr = doc.create_element("tr").unwrap();
            let th = doc.create_element("th").unwrap();
            let _ = th.set_attribute("scope", "row");
            th.set_text_content(Some(&t(&lang, d.label_key)));
            let td1 = doc.create_element("td").unwrap();
            let td2 = doc.create_element("td").unwrap();
            let td3 = doc.create_element("td").unwrap();
            let fmt_cell = |val: f64| if matches!(d.kind, aion2_core::calculator::DuelKind::Defense) { fmt_num(val) } else { format!("{}%", fmt_num(val * 100.0)) };
            td1.set_text_content(Some(&fmt_cell(d.you)));
            td2.set_text_content(Some(&fmt_cell(d.target)));
            td3.set_text_content(Some(&fmt_cell(d.net)));
            let _ = tr.append_child(&th);
            let _ = tr.append_child(&td1);
            let _ = tr.append_child(&td2);
            let _ = tr.append_child(&td3);
            let _ = duels_tbody.append_child(&tr);
        }
    }

    if let Some(assump_list) = doc.get_element_by_id("estimate-assumptions") {
        assump_list.set_inner_html("");
        for key in estimate_assumptions(hps, crit, double_en, pos) {
            let li = doc.create_element("li").unwrap();
            li.set_text_content(Some(&t(&lang, key)));
            let _ = assump_list.append_child(&li);
        }
    }
    attributes::update_attribute_values();
}

pub fn init_all() {
    let lang = current_lang();
    if let Some(selector) = document().get_element_by_id("lang-select").and_then(|e| e.dyn_into::<HtmlSelectElement>().ok()) {
        selector.set_value(&lang);
        let closure = Closure::<dyn FnMut()>::new(move || {
            let doc = document();
            if let Some(sel) = doc.get_element_by_id("lang-select").and_then(|e| e.dyn_into::<HtmlSelectElement>().ok()) {
                let next = sel.value();
                if let Some(storage) = window().local_storage().ok().flatten() {
                    let _ = storage.set_item("aion2_calculator_lang", &next);
                }
                apply_translations(&next);
                bosses::refresh_boss_ui();
                recalculate();
            }
        });
        selector.set_onchange(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }
    apply_translations(&lang);
    init_tabs();
    attributes::init_attributes();
    companion::init_companion();
    ai::init_ai();
    characters::init_characters();
    bosses::init_bosses();
    init_calculator();
}
