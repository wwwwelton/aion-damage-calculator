use wasm_bindgen::prelude::*;
use web_sys::{Document, Element, HtmlElement, HtmlInputElement, HtmlSelectElement};
use serde_json::{json, Value};
use std::cell::RefCell;
use crate::i18n::{format_t, t, Lang};

#[derive(Default)]
struct EnemySearchState {
    search_id: u32,
    profile_id: u32,
    results: Vec<Value>,
    loaded_profile: Option<Value>,
    loaded_item: Option<Value>,
}

thread_local! {
    static STATE: RefCell<EnemySearchState> = RefCell::new(EnemySearchState::default());
}

fn document() -> Document {
    web_sys::window().expect("window").document().expect("document")
}

fn current_lang() -> Lang {
    if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
        if let Ok(Some(saved)) = storage.get_item("aion2_calculator_lang") {
            if saved == "en" { return Lang::En; }
            if saved == "pt" { return Lang::Pt; }
        }
    }
    Lang::Pt
}

fn el(doc: &Document, tag: &str, class_name: &str) -> Element {
    let elem = doc.create_element(tag).unwrap();
    if !class_name.is_empty() {
        elem.set_class_name(class_name);
    }
    elem
}

fn el_text(doc: &Document, tag: &str, class_name: &str, text: &str) -> Element {
    let elem = el(doc, tag, class_name);
    elem.set_text_content(Some(text));
    elem
}

fn format_number(val: f64) -> String {
    let int_part = val.round() as i64;
    let s = int_part.abs().to_string();
    let mut result = String::new();
    let len = s.len();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    if int_part < 0 {
        format!("-{}", result)
    } else {
        result
    }
}

fn attribute_icon(name: &str) -> &'static str {
    let lower = name.to_lowercase();
    if lower.contains("hp") || lower.contains("health") || lower.contains("recovery") || lower.contains("constitution") {
        "health"
    } else if lower.contains("defense") || lower.contains("tolerance") || lower.contains("resist") || lower.contains("block") || lower.contains("evasion") || lower.contains("endurance") {
        "defense"
    } else if lower.contains("chance") || lower.contains("critical") || lower.contains("accuracy") || lower.contains("precision") || lower.contains("multi-hit") {
        "chance"
    } else if lower.contains("damage") || lower.contains("boost") || lower.contains("coefficient") {
        "damage"
    } else if lower.contains("attack") || lower.contains("penetration") || lower.contains("might") {
        "attack"
    } else {
        "stat"
    }
}

fn icon_element(doc: &Document, kind: &str, label: &str) -> Element {
    let normalized = label.trim().to_lowercase().replace(' ', "");
    const CLASS_ICONS: &[&str] = &[
        "gladiator", "templar", "assassin", "ranger", "sorcerer", "spiritmaster", "cleric", "chanter",
    ];
    if kind == "class" && CLASS_ICONS.contains(&normalized.as_str()) {
        let img = doc.create_element("img").unwrap();
        img.set_class_name("icon icon-class");
        let _ = img.set_attribute("src", &format!("assets/classes/{}.webp", normalized));
        let _ = img.set_attribute("alt", "");
        let _ = img.set_attribute("aria-hidden", "true");
        return img;
    }
    const SYMBOLS: &[&str] = &[
        "boss", "mob", "skill", "attack", "defense", "health", "chance", "damage", "stat",
    ];
    let symbol = if SYMBOLS.contains(&kind) { kind } else { "stat" };
    let svg = doc
        .create_element_ns(Some("http://www.w3.org/2000/svg"), "svg")
        .unwrap();
    svg.set_class_name(&format!("icon icon-{}", symbol));
    let _ = svg.set_attribute("aria-hidden", "true");
    let _ = svg.set_attribute("viewBox", "0 0 24 24");
    let use_elem = doc
        .create_element_ns(Some("http://www.w3.org/2000/svg"), "use")
        .unwrap();
    let _ = use_elem.set_attribute("href", &format!("#icon-{}", symbol));
    let _ = svg.append_child(&use_elem);
    svg
}

fn profile_image(doc: &Document, url: &str, kind: &str, label: &str) -> Element {
    if (url.starts_with("https://assets.playnccdn.com/static-aion2-gamedata/resources/")
        || url.starts_with("https://profileimg.plaync.com/game_profile_images/")
        || url.starts_with("https://aion2data.com/game/monsters/"))
        && url.starts_with("https://")
    {
        let img = doc.create_element("img").unwrap();
        img.set_class_name("icon remote-icon");
        let _ = img.set_attribute("alt", "");
        let _ = img.set_attribute("loading", "lazy");
        let _ = img.set_attribute("referrerpolicy", "no-referrer");
        let _ = img.set_attribute("src", url);
        return img;
    }
    icon_element(doc, kind, label)
}

fn set_display_icon(doc: &Document, container: &Element, kind: &str, label: &str) {
    container.set_inner_html("");
    let icon = icon_element(doc, kind, label);
    let _ = container.append_child(&icon);
    let normalized = label.trim().to_lowercase().replace(' ', "");
    const CLASS_ICONS: &[&str] = &[
        "gladiator", "templar", "assassin", "ranger", "sorcerer", "spiritmaster", "cleric", "chanter",
    ];
    let hide = kind == "class" && !CLASS_ICONS.contains(&normalized.as_str());
    if let Some(el) = container.dyn_ref::<HtmlElement>() {
        el.set_hidden(hide);
    }
}

fn sheet_defenses(profile: &Value) -> std::collections::HashSet<String> {
    let mut sheet = std::collections::HashSet::new();
    let stats = profile
        .get("info")
        .and_then(|i| i.get("stat"))
        .and_then(|s| s.get("statList"))
        .and_then(Value::as_array);

    let Some(stats) = stats else { return sheet };

    for &(label, field, is_percent) in aion2_core::character_extraction::ENEMY_FIELDS {
        if let Some(stat) = stats.iter().find(|s| s.get("name").and_then(Value::as_str) == Some(label)) {
            let val_str = stat.get("value").map(|v| v.to_string().trim_matches('"').trim().to_string()).unwrap_or_default();
            if !is_percent && val_str.ends_with('%') {
                continue;
            }
            let stripped = val_str.strip_suffix('%').unwrap_or(&val_str);
            if stripped.parse::<f64>().is_ok() {
                sheet.insert(field.to_string());
            }
        }
    }
    sheet
}

fn render_gear_card(doc: &Document, lang: Lang, item: &Value, detail: &Value) -> Element {
    let card = el(doc, "article", "gear-card");
    let head = el(doc, "div", "gear-card-head");
    let icon_url = item.get("icon").or_else(|| detail.get("icon")).and_then(Value::as_str).unwrap_or("");
    let _ = head.append_child(&profile_image(doc, icon_url, "stat", ""));
    let title_div = el(doc, "div", "");
    let name = item.get("name").or_else(|| detail.get("name")).and_then(Value::as_str).unwrap_or("");
    let _ = title_div.append_child(&el_text(doc, "strong", "", name));
    let slot = item.get("slotPosName").and_then(Value::as_str).unwrap_or("");
    let grade = item.get("grade").or_else(|| detail.get("grade")).and_then(Value::as_str).unwrap_or("");
    let enchant = item.get("enchantLevel").map(|v| format!("+{v}")).unwrap_or_default();
    let sub_info = [slot, grade, &enchant].into_iter().filter(|s| !s.is_empty()).collect::<Vec<_>>().join(" · ");
    let _ = title_div.append_child(&el_text(doc, "small", "", &sub_info));
    let _ = head.append_child(&title_div);
    let _ = card.append_child(&head);

    for (label_key, stat_key) in [
        ("baseOptions", "mainStats"),
        ("soulBind", "subStats"),
        ("manastones", "magicStoneStat"),
        ("theostones", "godStoneStat"),
    ] {
        if let Some(stats) = detail.get(stat_key).and_then(Value::as_array) {
            if stats.is_empty() { continue; }
            let block = el(doc, "div", "gear-options");
            let _ = block.append_child(&el_text(doc, "h4", "", &t(lang, label_key)));
            let list = el(doc, "ul", "option-list");
            for stat in stats {
                let s_name = stat.get("name").and_then(Value::as_str).unwrap_or("");
                let s_val = stat.get("value").or_else(|| stat.get("desc")).map(|v| v.to_string().trim_matches('"').to_string()).unwrap_or_default();
                let row = el(doc, "li", "");
                let _ = row.append_child(&icon_element(doc, attribute_icon(s_name), s_name));
                let _ = row.append_child(&el_text(doc, "span", "", &crate::i18n::stat_label(lang, s_name)));
                let _ = row.append_child(&el_text(doc, "strong", "value-flat", &s_val));
                let _ = list.append_child(&row);
            }
            let _ = block.append_child(&list);
            let _ = card.append_child(&block);
        }
    }
    card
}

pub fn render_dossier(
    doc: &Document,
    container: &Element,
    lang: Lang,
    view: &aion2_core::character_extraction::ExtractedCharacter,
    _data: &Value,
) {
    let summary = el(doc, "div", "dossier-summary");
    let portrait = profile_image(doc, &view.portrait, "class", &view.class_name);
    let _ = portrait.class_list().add_1("dossier-portrait");
    let _ = summary.append_child(&portrait);
    let summary_text = el(doc, "div", "");
    let title_elem = el_text(doc, "strong", "", &format!("{} · {}", view.name, view.class_name));
    let _ = summary_text.append_child(&title_elem);
    let meta_str = format!(
        "{} {} · {} · {} · {}",
        t(lang, "level"),
        if view.level.is_empty() { "—" } else { &view.level },
        if view.race.is_empty() { &t(lang, "raceUnknown") } else { &view.race },
        view.gender,
        if view.server.is_empty() { &t(lang, "serverUnknown") } else { &view.server }
    );
    let meta_elem = el_text(doc, "small", "", &meta_str);
    let _ = summary_text.append_child(&meta_elem);
    let cp_text = view.combat_power.map(|cp| format!("{} {}", t(lang, "combatPower"), format_number(cp))).unwrap_or_default();
    let gs_text = view.gear_score.map(|gs| format!("{} {}", t(lang, "gearScore"), format_number(gs))).unwrap_or_default();
    let scores = [cp_text, gs_text].into_iter().filter(|s| !s.is_empty()).collect::<Vec<_>>().join(" · ");
    if !scores.is_empty() {
        let scores_elem = el_text(doc, "small", "", &scores);
        let _ = summary_text.append_child(&scores_elem);
    }
    if !view.title_name.is_empty() {
        let title_line = el_text(doc, "small", "", &format!("{} {}", t(lang, "equippedTitle"), view.title_name));
        let _ = summary_text.append_child(&title_line);
    }
    let _ = summary.append_child(&summary_text);

    let tabs = el(doc, "div", "dossier-tabs");
    let _ = tabs.set_attribute("role", "tablist");
    let _ = tabs.set_attribute("aria-label", &t(lang, "characterDetails"));

    let mut tab_buttons = Vec::new();
    let mut tab_sections = Vec::new();

    let tab_specs = [
        (t(lang, "tabEquipment"), 0),
        (t(lang, "tabSkills"), 1),
        (t(lang, "tabAttributes"), 2),
        (t(lang, "tabProgression"), 3),
    ];

    for (label, idx) in tab_specs {
        let btn = el_text(doc, "button", "dossier-tab", &label);
        let _ = btn.set_attribute("type", "button");
        let _ = btn.set_attribute("role", "tab");
        let _ = btn.set_attribute("aria-selected", if idx == 0 { "true" } else { "false" });
        let _ = tabs.append_child(&btn);
        tab_buttons.push(btn);

        let section = el(doc, "section", "dossier-pane");
        let _ = section.set_attribute("role", "tabpanel");
        if idx != 0 {
            if let Some(s) = section.dyn_ref::<HtmlElement>() {
                s.set_hidden(true);
            }
        }
        tab_sections.push(section);
    }

    // Section 0: Equipment
    {
        let sec0 = &tab_sections[0];
        let head0 = el(doc, "div", "dossier-section-head");
        let _ = head0.append_child(&el_text(doc, "h3", "", &t(lang, "equippedItems")));
        let _ = head0.append_child(&el_text(doc, "small", "", &format_t(lang, "piecesPublished", &[("count", &view.gear.len().to_string())])));
        let _ = sec0.append_child(&head0);

        let gear_grid = el(doc, "div", "gear-grid");
        for item_pair in &view.gear {
            let item = &item_pair["item"];
            let detail = &item_pair["detail"];
            let card = render_gear_card(doc, lang, item, detail);
            let _ = gear_grid.append_child(&card);
        }
        let _ = sec0.append_child(&gear_grid);

        let companions = el(doc, "div", "progression-grid");
        if let Some(pet) = &view.pet {
            if let Some(name) = pet.get("name").and_then(Value::as_str) {
                let card = el(doc, "article", "progression-card");
                let _ = card.append_child(&profile_image(doc, pet.get("icon").and_then(Value::as_str).unwrap_or(""), "stat", ""));
                let _ = card.append_child(&el_text(doc, "strong", "", name));
                let detail_str = pet.get("level").map(|l| format!("{} {}", t(lang, "level"), l)).unwrap_or_default();
                let _ = card.append_child(&el_text(doc, "small", "", &format!("{} · {}", t(lang, "mount"), detail_str)));
                let _ = companions.append_child(&card);
            }
        }
        if let Some(wing) = &view.wing {
            if let Some(name) = wing.get("name").and_then(Value::as_str) {
                let card = el(doc, "article", "progression-card");
                let _ = card.append_child(&profile_image(doc, wing.get("icon").and_then(Value::as_str).unwrap_or(""), "stat", ""));
                let _ = card.append_child(&el_text(doc, "strong", "", name));
                let grade = wing.get("grade").and_then(Value::as_str).unwrap_or("");
                let _ = card.append_child(&el_text(doc, "small", "", &format!("{} · {}", t(lang, "wings"), grade)));
                let _ = companions.append_child(&card);
            }
        }
        let _ = sec0.append_child(&companions);
    }

    // Section 1: Skills
    {
        let sec1 = &tab_sections[1];
        let _ = sec1.append_child(&el_text(doc, "p", "reference-note", &t(lang, "skillDataNote")));
        for (category, key) in [("Active", "activeSkills"), ("Passive", "passiveSkills"), ("Dp", "stigmaSkills")] {
            let skills_in_cat: Vec<&Value> = view.skills.iter().filter(|s| s.get("category").and_then(Value::as_str) == Some(category)).collect();
            if skills_in_cat.is_empty() { continue; }
            let _ = sec1.append_child(&el_text(doc, "h3", "", &t(lang, key)));
            let grid = el(doc, "div", "skill-grid");
            for skill in skills_in_cat {
                let card = el(doc, "article", "skill-card");
                let icon_url = skill.get("icon").and_then(Value::as_str).unwrap_or("");
                let _ = card.append_child(&profile_image(doc, icon_url, "skill", ""));
                let content = el(doc, "div", "");
                let name = skill.get("name").and_then(Value::as_str).unwrap_or("");
                let _ = content.append_child(&el_text(doc, "strong", "", name));
                let lvl = skill.get("skillLevel").map(|v| v.to_string()).unwrap_or_default();
                let _ = content.append_child(&el_text(doc, "small", "", &format!("{} {}", t(lang, "level"), lvl)));
                let _ = card.append_child(&content);
                let _ = grid.append_child(&card);
            }
            let _ = sec1.append_child(&grid);
        }
    }

    // Section 2: Attributes
    {
        let sec2 = &tab_sections[2];
        let head2 = el(doc, "div", "dossier-section-head");
        let _ = head2.append_child(&el_text(doc, "h3", "", &t(lang, "sheetAttributes")));
        let _ = head2.append_child(&el_text(doc, "small", "", &t(lang, "noInferredTotals")));
        let _ = sec2.append_child(&head2);
        let grid2 = el(doc, "div", "attribute-grid");
        for stat in &view.stats {
            let name = stat.get("name").and_then(Value::as_str).unwrap_or("");
            let val = stat.get("value").map(|v| v.to_string().trim_matches('"').to_string()).unwrap_or_default();
            let card = el(doc, "article", "attribute-card");
            let _ = card.append_child(&icon_element(doc, attribute_icon(name), name));
            let text_div = el(doc, "div", "");
            let _ = text_div.append_child(&el_text(doc, "span", "", &crate::i18n::stat_label(lang, name)));
            let _ = text_div.append_child(&el_text(doc, "strong", "value-flat", &val));
            if let Some(seconds) = stat.get("statSecondList").and_then(Value::as_array) {
                for sec in seconds {
                    if let Some(sec_str) = sec.as_str() {
                        let _ = text_div.append_child(&el_text(doc, "small", "effect", &crate::i18n::stat_label(lang, sec_str)));
                    }
                }
            }
            let _ = card.append_child(&text_div);
            let _ = grid2.append_child(&card);
        }
        let _ = sec2.append_child(&grid2);
    }

    // Section 3: Progression
    {
        let sec3 = &tab_sections[3];
        if !view.titles.is_empty() {
            let _ = sec3.append_child(&el_text(doc, "h3", "", &t(lang, "equippedTitles")));
            let title_grid = el(doc, "div", "progression-grid");
            for title in &view.titles {
                let name = title.get("name").and_then(Value::as_str).unwrap_or("");
                let cat = title.get("equipCategory").and_then(Value::as_str).unwrap_or("");
                let card = el(doc, "article", "progression-card");
                let _ = card.append_child(&el_text(doc, "strong", "", name));
                let _ = card.append_child(&el_text(doc, "small", "", cat));
                let _ = title_grid.append_child(&card);
            }
            let _ = sec3.append_child(&title_grid);
        }
        if !view.boards.is_empty() {
            let _ = sec3.append_child(&el_text(doc, "h3", "", &t(lang, "daevanionBoards")));
            let board_grid = el(doc, "div", "progression-grid");
            for board in &view.boards {
                let name = board.get("name").and_then(Value::as_str).unwrap_or("");
                let open_count = board.get("openNodeCount").map(|v| v.to_string()).unwrap_or_else(|| "0".into());
                let total_count = board.get("totalNodeCount").map(|v| v.to_string()).unwrap_or_else(|| "?".into());
                let card = el(doc, "article", "progression-card");
                let _ = card.append_child(&el_text(doc, "strong", "", name));
                let _ = card.append_child(&el_text(doc, "small", "", &format!("{} / {}", open_count, total_count)));
                let _ = board_grid.append_child(&card);
            }
            let _ = sec3.append_child(&board_grid);
        }
        if !view.arcana.is_empty() {
            let _ = sec3.append_child(&el_text(doc, "h3", "", &t(lang, "arcana")));
            let arcana_grid = el(doc, "div", "gear-grid");
            for item_pair in &view.arcana {
                let item = &item_pair["item"];
                let detail = &item_pair["detail"];
                let card = render_gear_card(doc, lang, item, detail);
                let _ = arcana_grid.append_child(&card);
            }
            let _ = sec3.append_child(&arcana_grid);
        }
    }

    for (btn_idx, btn) in tab_buttons.iter().enumerate() {
        let btns_clone = tab_buttons.clone();
        let secs_clone = tab_sections.clone();
        let click_cb = Closure::<dyn FnMut()>::new(move || {
            for (i, b) in btns_clone.iter().enumerate() {
                let active = i == btn_idx;
                let _ = b.set_attribute("aria-selected", if active { "true" } else { "false" });
                if let Some(sec) = secs_clone[i].dyn_ref::<HtmlElement>() {
                    sec.set_hidden(!active);
                }
            }
        });
        if let Some(b) = btn.dyn_ref::<HtmlElement>() {
            b.set_onclick(Some(click_cb.as_ref().unchecked_ref()));
        }
        click_cb.forget();
    }

    container.set_inner_html("");
    let _ = container.append_child(&summary);
    let _ = container.append_child(&tabs);
    for sec in &tab_sections {
        let _ = container.append_child(sec);
    }
}

fn render_enemy_stats(doc: &Document, lang: Lang, data: &Value, stats_elem: &Element, import_btn: &Element) {
    let defenses = aion2_core::character_extraction::enemy_defenses(data);
    let sheet = sheet_defenses(data);
    let hp = aion2_core::character_extraction::character_hp(data);

    stats_elem.set_inner_html("");
    let mut entries = Vec::new();

    for (field, value) in &defenses {
        let label = aion2_core::character_extraction::ENEMY_FIELDS
            .iter()
            .find(|(_, f, _)| f == field)
            .map(|(l, _, _)| *l)
            .unwrap_or(field.as_str());

        let is_sheet = sheet.contains(field);
        let source = if is_sheet {
            t(lang, "characterSheetSource")
        } else {
            t(lang, "equipmentPartialSource")
        };
        let label_translated = crate::i18n::stat_label(lang, label);
        let text = format!("{label_translated}: {value} ({source})");
        let icon_kind = attribute_icon(label);
        entries.push((icon_kind, text));
    }

    if let Some(hp_val) = hp {
        let source = t(lang, "characterSheetSource");
        entries.push(("health", format!("HP: {hp_val} ({source})")));
    }

    if entries.is_empty() {
        let li = el_text(doc, "li", "", &t(lang, "noDefenses"));
        let _ = stats_elem.append_child(&li);
        let _ = import_btn.set_attribute("disabled", "");
    } else {
        for (icon_kind, text) in entries {
            let li = el(doc, "li", "");
            let _ = li.append_child(&icon_element(doc, icon_kind, ""));
            let _ = li.append_child(&doc.create_text_node(&format!(" {text}")));
            let _ = stats_elem.append_child(&li);
        }
        let _ = import_btn.remove_attribute("disabled");
    }
}

fn show_enemy_profile(doc: &Document, lang: Lang, data: Value) {
    let enemy_char = aion2_core::character_extraction::extract_character(&data);

    if let Some(title) = doc.get_element_by_id("enemy-title") {
        title.set_text_content(Some(&format!("{} · {}", enemy_char.name, enemy_char.server)));
    }

    let cp_str = enemy_char.combat_power.map(|cp| format!("CP {}", format_number(cp))).unwrap_or_default();
    let gs_str = enemy_char.gear_score.map(|gs| format!("GS {}", format_number(gs))).unwrap_or_default();
    let scores = [cp_str, gs_str].into_iter().filter(|s| !s.is_empty()).collect::<Vec<_>>().join(" · ");
    let mut meta_parts = vec![
        enemy_char.class_name.clone(),
        format!("{} {}", t(lang, "level"), enemy_char.level),
        enemy_char.race.clone(),
    ];
    if !scores.is_empty() {
        meta_parts.push(scores);
    }
    if let Some(meta) = doc.get_element_by_id("enemy-meta") {
        meta.set_text_content(Some(&meta_parts.join(" · ")));
    }

    if let Some(class_icon) = doc.get_element_by_id("enemy-class-icon") {
        set_display_icon(doc, &class_icon, "class", &enemy_char.class_name);
    }

    if let Some(dossier_elem) = doc.get_element_by_id("enemy-dossier") {
        render_dossier(doc, &dossier_elem, lang, &enemy_char, &data);
    }

    if let (Some(stats_elem), Some(import_btn)) = (doc.get_element_by_id("enemy-stats"), doc.get_element_by_id("import-enemy")) {
        render_enemy_stats(doc, lang, &data, &stats_elem, &import_btn);
    }

    if let Some(status) = doc.get_element_by_id("enemy-profile-status") {
        let is_partial = data.get("complete") == Some(&Value::Bool(false));
        let base = t(lang, if is_partial { "enemyPartialProfile" } else { "enemyProfileLoaded" });
        let date_str = data.pointer("/cacheMeta/fetchedAt").and_then(Value::as_str);
        let text = if let Some(d) = date_str {
            let is_cached = data.pointer("/cacheMeta/cached") == Some(&Value::Bool(true));
            let key = if is_cached { "cachedProfileDate" } else { "profileDate" };
            format!("{} · {}", base, format_t(lang, key, &[("date", d)]))
        } else {
            base
        };
        status.set_text_content(Some(&text));
    }

    STATE.with(|s| {
        s.borrow_mut().loaded_profile = Some(data);
    });
}

fn load_enemy(index: usize) {
    let doc = document();
    let lang = current_lang();

    let (item, profile_id) = STATE.with(|s| {
        let mut st = s.borrow_mut();
        st.profile_id += 1;
        let item = st.results.get(index).cloned();
        st.loaded_item = item.clone();
        st.loaded_profile = None;
        (item, st.profile_id)
    });

    let Some(item) = item else { return };

    if let Some(panel) = doc.get_element_by_id("enemy-panel").and_then(|e| e.dyn_into::<HtmlElement>().ok()) {
        panel.set_hidden(false);
    }
    if let Some(btn) = doc.get_element_by_id("import-enemy") {
        let _ = btn.set_attribute("disabled", "");
    }

    let initial_class = item.get("className").and_then(Value::as_str).filter(|c| *c != "Unknown").unwrap_or("");
    let initial_cp = item.get("combatPower").and_then(Value::as_f64).map(|cp| format!("CP {}", format_number(cp))).unwrap_or_default();
    let level_str = item.get("level").map(|l| format!("{} {}", t(lang, "level"), l.to_string().trim_matches('"'))).unwrap_or_default();
    let initial_details = [initial_class, &level_str, &initial_cp].into_iter().filter(|s| !s.is_empty()).collect::<Vec<_>>().join(" · ");

    if let Some(title) = doc.get_element_by_id("enemy-title") {
        title.set_text_content(Some(&format!("{} · {}", item.get("name").and_then(Value::as_str).unwrap_or(""), item.get("serverName").and_then(Value::as_str).unwrap_or(""))));
    }
    if let Some(meta) = doc.get_element_by_id("enemy-meta") {
        meta.set_text_content(Some(&initial_details));
    }
    if !initial_class.is_empty() {
        if let Some(class_icon) = doc.get_element_by_id("enemy-class-icon") {
            set_display_icon(&doc, &class_icon, "class", initial_class);
        }
    }
    if let Some(status) = doc.get_element_by_id("enemy-profile-status") {
        status.set_text_content(Some(&t(lang, "loadingDefenses")));
    }
    if let Some(dossier) = doc.get_element_by_id("enemy-dossier") {
        dossier.set_inner_html("");
    }
    if let Some(stats) = doc.get_element_by_id("enemy-stats") {
        stats.set_inner_html("");
    }

    let server_id = item.get("serverId").cloned().unwrap_or(Value::Null);
    let char_id = item.get("characterId").cloned().unwrap_or(Value::Null);
    let region = item.get("queryRegion").and_then(Value::as_str).unwrap_or("ww").to_string();
    let global_region = item.get("queryGlobalRegion").and_then(Value::as_str).unwrap_or("naw").to_string();

    wasm_bindgen_futures::spawn_local(async move {
        let res = crate::host::request(
            "profile",
            &region,
            &global_region,
            json!({ "serverId": server_id, "characterId": char_id }),
        )
        .await;

        let is_current = STATE.with(|s| s.borrow().profile_id == profile_id);
        if !is_current { return; }

        let doc = document();
        let lang = current_lang();
        match res {
            Ok(data) => {
                show_enemy_profile(&doc, lang, data);
            }
            Err(err) => {
                if let Some(status) = doc.get_element_by_id("enemy-profile-status") {
                    status.set_text_content(Some(&format_t(lang, "profileFailed", &[("error", &err)])));
                }
            }
        }
    });
}

fn reset_enemy_search() {
    let doc = document();
    let lang = current_lang();

    STATE.with(|s| {
        let mut st = s.borrow_mut();
        st.search_id += 1;
        st.profile_id += 1;
        st.results.clear();
        st.loaded_profile = None;
        st.loaded_item = None;
    });

    if let Some(lbl) = doc.get_element_by_id("enemy-choice-label").and_then(|e| e.dyn_into::<HtmlElement>().ok()) {
        lbl.set_hidden(true);
    }
    if let Some(panel) = doc.get_element_by_id("enemy-panel").and_then(|e| e.dyn_into::<HtmlElement>().ok()) {
        panel.set_hidden(true);
    }
    if let Some(btn) = doc.get_element_by_id("import-enemy") {
        let _ = btn.set_attribute("disabled", "");
    }
    if let Some(status) = doc.get_element_by_id("enemy-search-status") {
        status.set_text_content(Some(&t(lang, "searchAgain")));
    }
}

pub fn init_enemy_search() {
    let doc = document();

    // 1. Enemy search form submit
    if let Some(form) = doc.get_element_by_id("enemy-search").and_then(|e| e.dyn_into::<HtmlElement>().ok()) {
        let on_submit = Closure::<dyn FnMut(web_sys::Event)>::new(move |ev: web_sys::Event| {
            ev.prevent_default();
            let doc = document();
            let lang = current_lang();

            let name = doc
                .get_element_by_id("enemy-name")
                .and_then(|e| e.dyn_into::<HtmlInputElement>().ok())
                .map(|i| i.value().trim().to_string())
                .unwrap_or_default();

            if name.is_empty() { return; }

            let region = doc
                .get_element_by_id("enemy-region")
                .and_then(|e| e.dyn_into::<HtmlSelectElement>().ok())
                .map(|s| s.value())
                .unwrap_or_else(|| "ww".into());

            let global_region = doc
                .get_element_by_id("enemy-global-region")
                .and_then(|e| e.dyn_into::<HtmlSelectElement>().ok())
                .map(|s| s.value())
                .unwrap_or_else(|| "naw".into());

            let search_id = STATE.with(|s| {
                let mut st = s.borrow_mut();
                st.search_id += 1;
                st.profile_id += 1;
                st.results.clear();
                st.loaded_profile = None;
                st.loaded_item = None;
                st.search_id
            });

            if let Some(lbl) = doc.get_element_by_id("enemy-choice-label").and_then(|e| e.dyn_into::<HtmlElement>().ok()) {
                lbl.set_hidden(true);
            }
            if let Some(panel) = doc.get_element_by_id("enemy-panel").and_then(|e| e.dyn_into::<HtmlElement>().ok()) {
                panel.set_hidden(true);
            }
            if let Some(btn) = doc.get_element_by_id("import-enemy") {
                let _ = btn.set_attribute("disabled", "");
            }
            if let Some(status) = doc.get_element_by_id("enemy-search-status") {
                status.set_text_content(Some(&t(lang, "searchingProfiles")));
            }

            wasm_bindgen_futures::spawn_local(async move {
                let res = crate::host::request(
                    "search",
                    &region,
                    &global_region,
                    json!({ "name": name }),
                )
                .await;

                let is_current = STATE.with(|s| s.borrow().search_id == search_id);
                if !is_current { return; }

                let doc = document();
                let lang = current_lang();

                match res {
                    Ok(data) => {
                        let raw_results = data.get("results").and_then(Value::as_array).cloned().unwrap_or_default();
                        let enriched: Vec<Value> = raw_results.into_iter().map(|mut item| {
                            if let Some(obj) = item.as_object_mut() {
                                obj.insert("queryRegion".into(), Value::String(region.clone()));
                                obj.insert("queryGlobalRegion".into(), Value::String(global_region.clone()));
                            }
                            item
                        }).collect();

                        STATE.with(|s| {
                            s.borrow_mut().results = enriched.clone();
                        });

                        if enriched.is_empty() {
                            let msg = if data.get("complete") == Some(&Value::Bool(false)) {
                                t(lang, "searchStillProcessing")
                            } else {
                                t(lang, "noCharacters")
                            };
                            if let Some(status) = doc.get_element_by_id("enemy-search-status") {
                                status.set_text_content(Some(&msg));
                            }
                            return;
                        }

                        if let Some(status) = doc.get_element_by_id("enemy-search-status") {
                            status.set_text_content(Some(&format_t(
                                lang,
                                "charactersFound",
                                &[("count", &enriched.len().to_string()), ("extra", "")],
                            )));
                        }

                        if let Some(choice) = doc.get_element_by_id("enemy-choice").and_then(|e| e.dyn_into::<HtmlSelectElement>().ok()) {
                            choice.set_inner_html("");
                            let default_opt = doc.create_element("option").unwrap();
                            let _ = default_opt.set_attribute("value", "");
                            default_opt.set_text_content(Some(&t(lang, "selectCharacter")));
                            let _ = choice.append_child(&default_opt);

                            for (idx, item) in enriched.iter().enumerate() {
                                let opt = doc.create_element("option").unwrap();
                                let _ = opt.set_attribute("value", &idx.to_string());
                                let item_name = item.get("name").and_then(Value::as_str).unwrap_or("");
                                let server = item.get("serverName").and_then(Value::as_str).unwrap_or("");
                                let class_name = item.get("className").and_then(Value::as_str).filter(|c| *c != "Unknown").unwrap_or("");
                                let cp_str = item.get("combatPower").and_then(Value::as_f64).map(|cp| format!("CP {}", format_number(cp))).unwrap_or_default();
                                let details = [server, class_name, &cp_str].into_iter().filter(|s| !s.is_empty()).collect::<Vec<_>>().join(" · ");
                                let text = if details.is_empty() {
                                    item_name.to_string()
                                } else {
                                    format!("{item_name} · {details}")
                                };
                                opt.set_text_content(Some(&text));
                                let _ = choice.append_child(&opt);
                            }
                        }

                        if enriched.len() == 1 {
                            load_enemy(0);
                        } else if let Some(lbl) = doc.get_element_by_id("enemy-choice-label").and_then(|e| e.dyn_into::<HtmlElement>().ok()) {
                            lbl.set_hidden(false);
                        }
                    }
                    Err(err) => {
                        if let Some(status) = doc.get_element_by_id("enemy-search-status") {
                            status.set_text_content(Some(&format_t(lang, "searchFailed", &[("error", &err)])));
                        }
                    }
                }
            });
        });
        form.set_onsubmit(Some(on_submit.as_ref().unchecked_ref()));
        on_submit.forget();
    }

    // 2. Region / Global Region change
    let reset_fn = Closure::<dyn FnMut()>::new(move || {
        reset_enemy_search();
    });
    if let Some(region) = doc.get_element_by_id("enemy-region").and_then(|e| e.dyn_into::<HtmlSelectElement>().ok()) {
        let r_clone = region.clone();
        let change_cb = Closure::<dyn FnMut()>::new(move || {
            let doc = document();
            if let Some(lbl) = doc.get_element_by_id("enemy-global-region-label").and_then(|e| e.dyn_into::<HtmlElement>().ok()) {
                lbl.set_hidden(r_clone.value() != "ww");
            }
            reset_enemy_search();
        });
        region.set_onchange(Some(change_cb.as_ref().unchecked_ref()));
        change_cb.forget();
    }
    if let Some(global_region) = doc.get_element_by_id("enemy-global-region").and_then(|e| e.dyn_into::<HtmlSelectElement>().ok()) {
        global_region.set_onchange(Some(reset_fn.as_ref().unchecked_ref()));
        reset_fn.forget();
    }

    // 3. Enemy choice change
    if let Some(choice) = doc.get_element_by_id("enemy-choice").and_then(|e| e.dyn_into::<HtmlSelectElement>().ok()) {
        let choice_clone = choice.clone();
        let on_change = Closure::<dyn FnMut()>::new(move || {
            let val = choice_clone.value();
            if !val.is_empty() {
                if let Ok(idx) = val.parse::<usize>() {
                    load_enemy(idx);
                }
            }
        });
        choice.set_onchange(Some(on_change.as_ref().unchecked_ref()));
        on_change.forget();
    }

    // 4. Import enemy button click
    if let Some(btn) = doc.get_element_by_id("import-enemy").and_then(|e| e.dyn_into::<HtmlElement>().ok()) {
        let on_import = Closure::<dyn FnMut()>::new(move || {
            let doc = document();
            let lang = current_lang();

            let loaded = STATE.with(|s| s.borrow().loaded_profile.clone());
            let Some(enemy_loaded) = loaded else { return };

            let defenses = aion2_core::character_extraction::enemy_defenses(&enemy_loaded);
            let hp = aion2_core::character_extraction::character_hp(&enemy_loaded);
            let enemy_char = aion2_core::character_extraction::extract_character(&enemy_loaded);

            if let Some(sel) = doc.get_element_by_id("target-mode").and_then(|e| e.dyn_into::<HtmlSelectElement>().ok()) {
                sel.set_value("player");
                let _ = sel.dispatch_event(&web_sys::Event::new("change").unwrap());
            }

            if let Ok(inputs) = doc.query_selector_all("#calculator .enemy input") {
                for i in 0..inputs.length() {
                    if let Some(inp) = inputs.item(i).and_then(|n| n.dyn_into::<HtmlInputElement>().ok()) {
                        inp.set_value("0");
                    }
                }
            }

            for (field, value) in &defenses {
                if let Ok(Some(inp)) = doc.query_selector(&format!("#calculator input[name=\"{field}\"]")) {
                    if let Some(inp) = inp.dyn_into::<HtmlInputElement>().ok() {
                        inp.set_value(&value.to_string());
                    }
                }
            }

            if let Some(hp_val) = hp {
                if let Ok(Some(inp)) = doc.query_selector("#calculator input[name=\"enemyHp\"]") {
                    if let Some(inp) = inp.dyn_into::<HtmlInputElement>().ok() {
                        inp.set_value(&hp_val.to_string());
                    }
                }
            }

            if let Some(inp) = doc.get_element_by_id("target-name").and_then(|e| e.dyn_into::<HtmlInputElement>().ok()) {
                inp.set_value(&enemy_char.name);
            }
            if let Some(inp) = doc.get_element_by_id("target-class").and_then(|e| e.dyn_into::<HtmlInputElement>().ok()) {
                let class_name = if !enemy_char.class_name.is_empty() {
                    enemy_char.class_name.clone()
                } else if let Some(meta) = doc.get_element_by_id("enemy-meta").and_then(|e| e.text_content()) {
                    meta.split(" · ").next().unwrap_or("").to_string()
                } else {
                    String::new()
                };
                inp.set_value(&class_name);
            }

            if let Some(calc) = doc.get_element_by_id("calculator") {
                let init = web_sys::EventInit::new();
                init.set_bubbles(true);
                if let Ok(ev) = web_sys::Event::new_with_event_init_dict("input", &init) {
                    let _ = calc.dispatch_event(&ev);
                }
            }

            if let Some(warning) = doc.get_element_by_id("import-warning").and_then(|e| e.dyn_into::<HtmlElement>().ok()) {
                warning.set_hidden(false);
                let hp_text = match hp {
                    Some(hp_val) => format_t(lang, "enemyHpKnown", &[("hp", &hp_val.to_string())]),
                    None => t(lang, "enemyHpUnknown"),
                };
                let msg = format_t(
                    lang,
                    "enemyImportResult",
                    &[
                        ("count", &defenses.len().to_string()),
                        ("name", &enemy_char.name),
                        ("hp", &hp_text),
                    ],
                );
                warning.set_text_content(Some(&msg));
            }

            if let Some(tab_btn) = doc.get_element_by_id("calculator-tab").and_then(|e| e.dyn_into::<HtmlElement>().ok()) {
                tab_btn.click();
            }

            if let Ok(Some(encounter)) = doc.query_selector(".encounter") {
                if let Some(el) = encounter.dyn_into::<HtmlElement>().ok() {
                    el.scroll_into_view();
                }
            }

            crate::controller::recalculate();
        });
        btn.set_onclick(Some(on_import.as_ref().unchecked_ref()));
        on_import.forget();
    }
}
