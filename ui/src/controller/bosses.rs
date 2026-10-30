use wasm_bindgen::prelude::*;
use web_sys::{Document, Element, HtmlElement, HtmlInputElement, HtmlSelectElement};
use serde::Deserialize;
use std::sync::LazyLock;
use crate::i18n::{format_t, t, Lang};

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct Attack {
    #[serde(rename = "type")]
    pub kind: String,
    pub reach: Option<serde_json::Value>,
    pub interval: Option<serde_json::Value>,
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct Skill {
    pub name: String,
    pub damage: String,
    pub cast: String,
    pub cooldown: String,
    pub range: String,
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct Monster {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub level: String,
    pub rank: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub group: Option<String>,
    pub location: Option<String>,
    pub subtitle: Option<String>,
    pub portrait: Option<String>,
    #[serde(rename = "spawnPoints")]
    pub spawn_points: Option<u32>,
    pub attack: Option<Attack>,
    #[serde(default)]
    pub skills: Vec<Skill>,
}

static BOSSES: LazyLock<Vec<Monster>> =
    LazyLock::new(|| serde_json::from_str(include_str!("../../data/bosses.json")).expect("bosses.json"));
static MOBS: LazyLock<Vec<Monster>> =
    LazyLock::new(|| serde_json::from_str(include_str!("../../data/mobs.json")).expect("mobs.json"));

const BOSS_TYPES: [&str; 4] = ["Cogni", "Fera", "Natura", "Varian"];

fn document() -> Document {
    web_sys::window().expect("window").document().expect("document")
}

fn current_lang() -> Lang {
    Lang::default()
}

fn value_to_str(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        _ => v.to_string(),
    }
}

fn create_icon_svg(doc: &Document, kind: &str) -> Element {
    let svg = doc.create_element_ns(Some("http://www.w3.org/2000/svg"), "svg").unwrap();
    let _ = svg.set_attribute("class", &format!("icon icon-{}", kind));
    let _ = svg.set_attribute("aria-hidden", "true");
    let _ = svg.set_attribute("viewBox", "0 0 24 24");
    let u = doc.create_element_ns(Some("http://www.w3.org/2000/svg"), "use").unwrap();
    let _ = u.set_attribute("href", &format!("#icon-{}", kind));
    let _ = svg.append_child(&u);
    svg
}

fn show_tab(tab_id: &str) {
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
}

pub fn filter_bosses() {
    let doc = document();
    let lang = current_lang();

    let search = doc.get_element_by_id("boss-search")
        .and_then(|e| e.dyn_into::<HtmlInputElement>().ok())
        .map(|i| i.value().trim().to_lowercase())
        .unwrap_or_default();

    let is_boss = doc.get_element_by_id("opponent-kind")
        .and_then(|e| e.dyn_into::<HtmlSelectElement>().ok())
        .map(|s| s.value() == "boss")
        .unwrap_or(true);

    let group = doc.get_element_by_id("boss-group")
        .and_then(|e| e.dyn_into::<HtmlSelectElement>().ok())
        .map(|s| s.value())
        .unwrap_or_default();

    let catalog: &[Monster] = if is_boss { &*BOSSES } else { &*MOBS };

    let visible: Vec<&Monster> = catalog
        .iter()
        .filter(|boss| {
            if is_boss && !group.is_empty() && boss.group.as_deref() != Some(&group) {
                return false;
            }
            if !search.is_empty() {
                let text = format!(
                    "{} {} {}",
                    boss.name,
                    boss.location.as_deref().unwrap_or(""),
                    boss.subtitle.as_deref().unwrap_or("")
                )
                .to_lowercase();
                if !text.contains(&search) {
                    return false;
                }
            }
            true
        })
        .collect();

    let Some(choice) = doc.get_element_by_id("boss-choice").and_then(|e| e.dyn_into::<HtmlSelectElement>().ok()) else {
        return;
    };
    choice.set_inner_html("");

    let default_opt = doc.create_element("option").unwrap();
    default_opt.set_text_content(Some(&t(lang, "selectMonster")));
    let _ = default_opt.set_attribute("value", "");
    let _ = choice.append_child(&default_opt);

    for boss in &visible {
        let opt = doc.create_element("option").unwrap();
        let _ = opt.set_attribute("value", &boss.id);
        let group_or_rank = boss.group.as_deref().unwrap_or(&boss.rank);
        opt.set_text_content(Some(&format!(
            "{} · Lv {} · {} · {}",
            boss.name, boss.level, boss.kind, group_or_rank
        )));
        let _ = choice.append_child(&opt);
    }

    if let Some(cnt_el) = doc.get_element_by_id("boss-count") {
        let kind_text = t(lang, if is_boss { "bosses" } else { "mobs" });
        let count_str = format_t(
            lang,
            "catalogCount",
            &[
                ("shown", &visible.len().to_string()),
                ("total", &catalog.len().to_string()),
                ("kind", &kind_text),
            ],
        );
        cnt_el.set_text_content(Some(&count_str));
    }

    show_boss();
}

pub fn show_boss() {
    let doc = document();
    let lang = current_lang();

    let is_boss = doc.get_element_by_id("opponent-kind")
        .and_then(|e| e.dyn_into::<HtmlSelectElement>().ok())
        .map(|s| s.value() == "boss")
        .unwrap_or(true);

    let choice_val = doc.get_element_by_id("boss-choice")
        .and_then(|e| e.dyn_into::<HtmlSelectElement>().ok())
        .map(|s| s.value())
        .unwrap_or_default();

    let catalog: &[Monster] = if is_boss { &*BOSSES } else { &*MOBS };
    let boss = if choice_val.is_empty() {
        None
    } else {
        catalog.iter().find(|item| item.id == choice_val)
    };

    // Update #import-boss button disabled state
    if let Some(import_btn) = doc.get_element_by_id("import-boss").and_then(|e| e.dyn_into::<HtmlElement>().ok()) {
        if boss.is_some() {
            let _ = import_btn.remove_attribute("disabled");
        } else {
            let _ = import_btn.set_attribute("disabled", "");
        }
    }

    // Update #boss-title
    if let Some(title) = doc.get_element_by_id("boss-title") {
        let title_text = boss.map(|b| b.name.clone()).unwrap_or_else(|| t(lang, "selectMonster"));
        title.set_text_content(Some(&title_text));
    }

    // Update #boss-icon
    if let Some(icon_holder) = doc.get_element_by_id("boss-icon").and_then(|e| e.dyn_into::<HtmlElement>().ok()) {
        icon_holder.set_inner_html("");
        if let Some(portrait) = boss.and_then(|b| b.portrait.as_deref()) {
            let img = doc.create_element("img").unwrap();
            let _ = img.set_attribute("class", "icon remote-icon");
            let _ = img.set_attribute("alt", "");
            let _ = img.set_attribute("loading", "lazy");
            let _ = img.set_attribute("referrerpolicy", "no-referrer");
            let _ = img.set_attribute("src", &format!("https://aion2data.com{}", portrait));
            let _ = icon_holder.append_child(&img);
            icon_holder.set_hidden(false);
        } else {
            let icon_kind = if is_boss { "boss" } else { "mob" };
            let svg = create_icon_svg(&doc, icon_kind);
            let _ = icon_holder.append_child(&svg);
        }
    }

    // Update #boss-facts
    if let Some(facts) = doc.get_element_by_id("boss-facts").and_then(|e| e.dyn_into::<HtmlElement>().ok()) {
        facts.set_hidden(boss.is_none());
        facts.set_inner_html("");
        if let Some(b) = boss {
            let attack_summary = b.attack.as_ref().map(|atk| {
                let type_label = match atk.kind.as_str() {
                    "magic" => t(lang, "attackMagic"),
                    "physical" => t(lang, "attackPhysical"),
                    _ => t(lang, "attackUnknown"),
                };
                let reach = atk.reach.as_ref().map(value_to_str).unwrap_or_else(|| "?".to_string());
                let interval = atk.interval.as_ref().map(value_to_str).unwrap_or_else(|| "?".to_string());
                format_t(
                    lang,
                    "basicAttackSummary",
                    &[
                        ("type", &type_label),
                        ("reach", &reach),
                        ("interval", &interval),
                    ],
                )
            });

            let mut fact_pairs: Vec<(String, String)> = vec![
                (t(lang, "factLevel"), b.level.clone()),
                (t(lang, "factRank"), b.rank.clone()),
                (t(lang, "factCreatureType"), b.kind.clone()),
                (t(lang, "factEncounter"), b.group.clone().unwrap_or_else(|| t(lang, "mobs"))),
                (t(lang, "factHp"), t(lang, "unknownServerSide")),
                (t(lang, "factDefense"), t(lang, "unknownServerSide")),
            ];
            if let Some(sp) = b.spawn_points {
                fact_pairs.push((t(lang, "factSpawnPoints"), sp.to_string()));
            }
            if let Some(atk) = attack_summary {
                fact_pairs.push((t(lang, "factBasicAttack"), atk));
            }

            for (label, value) in fact_pairs {
                let card = doc.create_element("div").unwrap();
                card.set_class_name("boss-fact");
                let caption = doc.create_element("small").unwrap();
                caption.set_text_content(Some(&label));
                let detail = doc.create_element("strong").unwrap();
                detail.set_text_content(Some(&value));
                let _ = card.append_child(&caption);
                let _ = card.append_child(&detail);
                let _ = facts.append_child(&card);
            }
        }
    }

    // Update #boss-details
    if let Some(details) = doc.get_element_by_id("boss-details") {
        let details_text = if let Some(b) = boss {
            format!(
                "NPC {}{}{}",
                b.id,
                b.location.as_ref().map(|l| format!(" · {}", l)).unwrap_or_default(),
                b.subtitle.as_ref().map(|s| format!(" · {}", s)).unwrap_or_default(),
            )
        } else {
            String::new()
        };
        details.set_text_content(Some(&details_text));
    }

    // Update #boss-skills
    if let Some(skills) = doc.get_element_by_id("boss-skills").and_then(|e| e.dyn_into::<HtmlElement>().ok()) {
        skills.set_hidden(boss.is_none());
        skills.set_inner_html("");
        if let Some(b) = boss {
            let heading = doc.create_element("h3").unwrap();
            let count_str = b.skills.len().to_string();
            heading.set_text_content(Some(&format_t(lang, "bossSkillsTitle", &[("count", &count_str)])));
            let _ = skills.append_child(&heading);

            if !b.skills.is_empty() {
                let table = doc.create_element("table").unwrap();
                table.set_class_name("boss-skill-table");
                let thead = doc.create_element("thead").unwrap();
                let head_row = doc.create_element("tr").unwrap();
                for key in ["skillName", "skillDamage", "skillCast", "skillCooldown", "skillRange"] {
                    let cell = doc.create_element("th").unwrap();
                    cell.set_text_content(Some(&t(lang, key)));
                    let _ = head_row.append_child(&cell);
                }
                let _ = thead.append_child(&head_row);
                let _ = table.append_child(&thead);

                let tbody = doc.create_element("tbody").unwrap();
                for skill in &b.skills {
                    let row = doc.create_element("tr").unwrap();
                    let damage = match skill.damage.as_str() {
                        "Magic" => t(lang, "attackMagic"),
                        "Physical" => t(lang, "attackPhysical"),
                        _ => skill.damage.clone(),
                    };
                    for value in [&skill.name, &damage, &skill.cast, &skill.cooldown, &skill.range] {
                        let cell = doc.create_element("td").unwrap();
                        cell.set_text_content(Some(value));
                        let _ = row.append_child(&cell);
                    }
                    let _ = tbody.append_child(&row);
                }
                let _ = table.append_child(&tbody);
                let _ = skills.append_child(&table);

                let note = doc.create_element("p").unwrap();
                note.set_class_name("import-note");
                note.set_text_content(Some(&t(lang, "bossSkillsNote")));
                let _ = skills.append_child(&note);
            } else {
                let empty = doc.create_element("p").unwrap();
                empty.set_class_name("import-note");
                empty.set_text_content(Some(&t(lang, "noSkillData")));
                let _ = skills.append_child(&empty);
            }
        }
    }

    // Update #boss-source
    if let Some(source) = doc.get_element_by_id("boss-source").and_then(|e| e.dyn_into::<HtmlElement>().ok()) {
        source.set_hidden(boss.is_none());
        if let Some(b) = boss {
            source.set_text_content(Some(&format!("{}: https://aion2data.com/monsters/{}-{}", t(lang, "source"), b.id, b.slug)));
        } else {
            source.set_text_content(None);
        }
    }
}

pub fn init_bosses() {
    let doc = document();

    // Event listener for opponent-kind: toggle boss-group-label and filter
    if let Some(kind) = doc.get_element_by_id("opponent-kind").and_then(|e| e.dyn_into::<HtmlSelectElement>().ok()) {
        let on_kind_change = Closure::<dyn FnMut()>::new(move || {
            let doc = document();
            let is_mob = doc.get_element_by_id("opponent-kind")
                .and_then(|e| e.dyn_into::<HtmlSelectElement>().ok())
                .map(|s| s.value() == "mob")
                .unwrap_or(false);
            if let Some(group_lbl) = doc.get_element_by_id("boss-group-label").and_then(|e| e.dyn_into::<HtmlElement>().ok()) {
                group_lbl.set_hidden(is_mob);
            }
            filter_bosses();
        });
        kind.set_onchange(Some(on_kind_change.as_ref().unchecked_ref()));
        on_kind_change.forget();
    }

    // Event listener for boss-search
    if let Some(srch) = doc.get_element_by_id("boss-search").and_then(|e| e.dyn_into::<HtmlInputElement>().ok()) {
        let on_search = Closure::<dyn FnMut()>::new(move || {
            filter_bosses();
        });
        srch.set_oninput(Some(on_search.as_ref().unchecked_ref()));
        on_search.forget();
    }

    // Event listener for boss-group
    if let Some(grp) = doc.get_element_by_id("boss-group").and_then(|e| e.dyn_into::<HtmlSelectElement>().ok()) {
        let on_group = Closure::<dyn FnMut()>::new(move || {
            filter_bosses();
        });
        grp.set_onchange(Some(on_group.as_ref().unchecked_ref()));
        on_group.forget();
    }

    // Event listener for boss-choice
    if let Some(choice) = doc.get_element_by_id("boss-choice").and_then(|e| e.dyn_into::<HtmlSelectElement>().ok()) {
        let on_choice = Closure::<dyn FnMut()>::new(move || {
            show_boss();
        });
        choice.set_onchange(Some(on_choice.as_ref().unchecked_ref()));
        on_choice.forget();
    }

    // Event listener for import-boss
    if let Some(import_btn) = doc.get_element_by_id("import-boss").and_then(|e| e.dyn_into::<HtmlElement>().ok()) {
        let on_import = Closure::<dyn FnMut()>::new(move || {
            let doc = document();
            let choice_val = doc.get_element_by_id("boss-choice")
                .and_then(|e| e.dyn_into::<HtmlSelectElement>().ok())
                .map(|s| s.value())
                .unwrap_or_default();
            if choice_val.is_empty() { return; }

            let is_boss = doc.get_element_by_id("opponent-kind")
                .and_then(|e| e.dyn_into::<HtmlSelectElement>().ok())
                .map(|s| s.value() == "boss")
                .unwrap_or(true);

            let catalog: &[Monster] = if is_boss { &*BOSSES } else { &*MOBS };
            let Some(boss) = catalog.iter().find(|m| m.id == choice_val) else { return };

            // Reset enemy inputs
            if let Ok(enemy_inputs) = doc.query_selector_all("#calculator .enemy input") {
                for i in 0..enemy_inputs.length() {
                    if let Some(inp) = enemy_inputs.item(i).and_then(|n| n.dyn_into::<HtmlInputElement>().ok()) {
                        inp.set_value("0");
                    }
                }
            }

            // Set target mode
            let target_mode_val = if is_boss { "boss" } else { "monster" };
            if let Some(tm) = doc.get_element_by_id("target-mode").and_then(|e| e.dyn_into::<HtmlSelectElement>().ok()) {
                tm.set_value(target_mode_val);
                let _ = tm.dispatch_event(&web_sys::CustomEvent::new("change").unwrap());
            }
            if let Some(view) = doc.get_element_by_id("calculator-view") {
                let _ = view.set_attribute("data-mode", target_mode_val);
            }
            if let Some(l) = doc.get_element_by_id("target-kind-label").and_then(|e| e.dyn_into::<HtmlElement>().ok()) {
                l.set_hidden(false);
            }
            if let Some(l) = doc.get_element_by_id("target-class-label").and_then(|e| e.dyn_into::<HtmlElement>().ok()) {
                l.set_hidden(true);
            }

            // Creature type
            let is_standard_type = BOSS_TYPES.iter().any(|&bt| bt.eq_ignore_ascii_case(&boss.kind));
            let type_val = if is_standard_type { boss.kind.to_lowercase() } else { "special".to_string() };
            let creature_type_val = if type_val == "special" { "" } else { &type_val };

            if let Some(ct) = doc.get_element_by_id("creature-type").and_then(|e| e.dyn_into::<HtmlSelectElement>().ok()) {
                ct.set_value(creature_type_val);
            }
            if let Some(cth) = doc.get_element_by_id("creature-type-header").and_then(|e| e.dyn_into::<HtmlSelectElement>().ok()) {
                cth.set_value(&type_val);
            }
            if let Some(tn) = doc.get_element_by_id("target-name").and_then(|e| e.dyn_into::<HtmlInputElement>().ok()) {
                tn.set_value(&boss.name);
            }

            // Dispatch input event to calculator and recalculate
            if let Some(calc) = doc.get_element_by_id("calculator") {
                let init = web_sys::CustomEventInit::new();
                init.set_bubbles(true);
                if let Ok(ev) = web_sys::CustomEvent::new_with_event_init_dict("input", &init) {
                    let _ = calc.dispatch_event(&ev);
                }
            }
            crate::controller::recalculate();

            // Set warning
            let lang = current_lang();
            if let Some(warning) = doc.get_element_by_id("import-warning").and_then(|e| e.dyn_into::<HtmlElement>().ok()) {
                warning.set_hidden(false);
                let target_selected_str = format_t(
                    lang,
                    "targetSelected",
                    &[
                        ("name", &boss.name),
                        ("level", &boss.level),
                        ("type", &boss.kind),
                        ("kind", if is_boss { "boss" } else { "mob" }),
                    ],
                );
                let bonus_key = if is_standard_type { "creatureBonusesApplied" } else { "specialCreatureManual" };
                let bonus_msg = t(lang, bonus_key).replace("{type}", &boss.kind);
                warning.set_text_content(Some(&format!("{} {}", target_selected_str, bonus_msg)));
            }

            // Switch tab to calculator
            show_tab("calculator-view");
            if let Ok(Some(enc)) = doc.query_selector(".encounter") {
                if let Some(el) = enc.dyn_into::<HtmlElement>().ok() {
                    el.scroll_into_view();
                }
            }
        });
        import_btn.set_onclick(Some(on_import.as_ref().unchecked_ref()));
        on_import.forget();
    }

    // Initial filter and render
    filter_bosses();
}

pub fn refresh_boss_ui() {
    let doc = document();
    let selected = doc.get_element_by_id("boss-choice")
        .and_then(|e| e.dyn_into::<HtmlSelectElement>().ok())
        .map(|s| s.value())
        .unwrap_or_default();
    filter_bosses();
    if !selected.is_empty() {
        if let Some(choice) = doc.get_element_by_id("boss-choice").and_then(|e| e.dyn_into::<HtmlSelectElement>().ok()) {
            choice.set_value(&selected);
        }
    }
    show_boss();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_catalog_loaded() {
        assert_eq!(BOSSES.len(), 521);
        assert_eq!(MOBS.len(), 3072);
        let gartua = BOSSES.iter().find(|b| b.id == "2101074").expect("Gartua found");
        assert_eq!(gartua.name, "Eternal Gartua");
        assert_eq!(gartua.kind, "Natura");
        assert_eq!(gartua.level, "51");
        assert!(!gartua.skills.is_empty());
        assert_eq!(gartua.skills[0].name, "Interstellar Flight");
    }
}
