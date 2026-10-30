use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Document, Element, HtmlElement, HtmlInputElement, HtmlSelectElement};
use serde_json::{json, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use aion2_core::character_extraction::{
    extract_character, equipment_bonuses, offensive_references,
    character_hp, enemy_defenses, sheet_modifiers, formula_import_coverage,
    ExtractedCharacter,
};

fn document() -> Document {
    web_sys::window().expect("window").document().expect("document")
}

struct CharacterSearchState {
    current_search: u64,
    current_profile: u64,
    results: Vec<Value>,
    loaded: Option<Value>,
    loaded_item: Option<Value>,
    previous_import: HashMap<String, f64>,
    previous_ref_import: HashMap<String, f64>,
}

thread_local! {
    static STATE: RefCell<CharacterSearchState> = RefCell::new(CharacterSearchState {
        current_search: 0,
        current_profile: 0,
        results: Vec::new(),
        loaded: None,
        loaded_item: None,
        previous_import: HashMap::new(),
        previous_ref_import: HashMap::new(),
    });
}

fn profile_key(item: &Value) -> String {
    let gr = item.get("queryGlobalRegion").and_then(Value::as_str).unwrap_or("naw");
    let server_id = item.get("serverId").map(|v| v.to_string().trim_matches('"').to_string()).unwrap_or_default();
    let char_id = item.get("characterId").and_then(Value::as_str).unwrap_or("");
    format!("{gr}|{server_id}|{char_id}")
}

fn get_favorites() -> Vec<String> {
    if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
        if let Ok(Some(json_str)) = storage.get_item("aion2Favorites") {
            if let Ok(arr) = serde_json::from_str::<Vec<String>>(&json_str) {
                return arr.into_iter().take(20).collect();
            }
        }
    }
    Vec::new()
}

fn save_favorites(favs: &[String]) {
    if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
        if let Ok(json_str) = serde_json::to_string(favs) {
            let _ = storage.set_item("aion2Favorites", &json_str);
        }
    }
}

fn get_recent_searches() -> Vec<String> {
    if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
        if let Ok(Some(json_str)) = storage.get_item("aion2RecentSearches") {
            if let Ok(arr) = serde_json::from_str::<Vec<String>>(&json_str) {
                return arr.into_iter().filter(|s| !s.is_empty() && s.len() <= 20).take(10).collect();
            }
        }
    }
    Vec::new()
}

fn save_recent_searches(recents: &[String]) {
    if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
        if let Ok(json_str) = serde_json::to_string(recents) {
            let _ = storage.set_item("aion2RecentSearches", &json_str);
        }
    }
}

fn render_recent_searches(doc: &Document) {
    let recents = get_recent_searches();
    let datalist = if let Some(dl) = doc.get_element_by_id("recent-character-searches") {
        dl
    } else {
        let dl = doc.create_element("datalist").unwrap();
        dl.set_id("recent-character-searches");
        if let Some(name_inp) = doc.get_element_by_id("character-name") {
            let _ = name_inp.set_attribute("list", "recent-character-searches");
        }
        if let Some(form) = doc.get_element_by_id("character-search") {
            let _ = form.append_child(&dl);
        }
        dl
    };
    datalist.set_inner_html("");
    for name in recents {
        let opt = doc.create_element("option").unwrap();
        let _ = opt.set_attribute("value", &name);
        let _ = datalist.append_child(&opt);
    }
}

fn format_number(val: f64) -> String {
    let int_val = val.round() as i64;
    let s = int_val.to_string();
    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();
    for (i, c) in chars.into_iter().enumerate() {
        if i > 0 && (len - i) % 3 == 0 && c != '-' {
            result.push(',');
        }
        result.push(c);
    }
    result
}

fn format_date(timestamp_secs: u64, lang: &str) -> String {
    let js_date = js_sys::Date::new(&JsValue::from_f64((timestamp_secs as f64) * 1000.0));
    let locale = if lang == "pt" { "pt-BR" } else { "en-US" };
    js_date.to_locale_string(locale, &JsValue::UNDEFINED).as_string().unwrap_or_default()
}

fn icon_element(doc: &Document, kind: &str, label: &str) -> Element {
    let normalized = label.trim().to_lowercase().replace(' ', "");
    let class_icons = ["gladiator", "templar", "assassin", "ranger", "sorcerer", "spiritmaster", "cleric", "chanter"];
    if kind == "class" && class_icons.contains(&normalized.as_str()) {
        let img = doc.create_element("img").unwrap();
        let _ = img.set_attribute("class", "icon icon-class");
        let _ = img.set_attribute("src", &format!("assets/classes/{normalized}.webp"));
        let _ = img.set_attribute("alt", "");
        let _ = img.set_attribute("aria-hidden", "true");
        return img;
    }
    let symbols = ["boss", "mob", "skill", "attack", "defense", "health", "chance", "damage", "stat"];
    let symbol = if symbols.contains(&kind) { kind } else { "stat" };
    let svg = doc.create_element_ns(Some("http://www.w3.org/2000/svg"), "svg").unwrap();
    let _ = svg.set_attribute("class", &format!("icon icon-{symbol}"));
    let _ = svg.set_attribute("aria-hidden", "true");
    let _ = svg.set_attribute("viewBox", "0 0 24 24");
    let u = doc.create_element_ns(Some("http://www.w3.org/2000/svg"), "use").unwrap();
    let _ = u.set_attribute("href", &format!("#icon-{symbol}"));
    let _ = svg.append_child(&u);
    svg
}

fn profile_image(doc: &Document, url: &str, kind: &str, label: &str) -> Element {
    let safe = url.starts_with("https://") || url.starts_with("assets/");
    if safe {
        let img = doc.create_element("img").unwrap();
        let _ = img.set_attribute("class", "icon remote-icon");
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
    let class_icons = ["gladiator", "templar", "assassin", "ranger", "sorcerer", "spiritmaster", "cleric", "chanter"];
    let hidden = kind == "class" && !class_icons.contains(&normalized.as_str());
    if let Some(html_el) = container.dyn_ref::<HtmlElement>() {
        html_el.set_hidden(hidden);
    }
}

fn attribute_icon(name: &str) -> &'static str {
    let lower = name.to_lowercase();
    if lower.contains("hp") || lower.contains("health") || lower.contains("recovery") || lower.contains("regeneration") || lower.contains("constitution") {
        "health"
    } else if lower.contains("defense") || lower.contains("tolerance") || lower.contains("resist") || lower.contains("block") || lower.contains("evasion") || lower.contains("endurance") {
        "defense"
    } else if lower.contains("chance") || lower.contains("critical hit") || lower.contains("accuracy") || lower.contains("precision") || lower.contains("perfect") || lower.contains("multi-hit") {
        "chance"
    } else if lower.contains("damage") || lower.contains("boost") || lower.contains("coefficient") || lower.contains("specialty") {
        "damage"
    } else if lower.contains("attack") || lower.contains("penetration") || lower.contains("power shard") || lower.contains("might") {
        "attack"
    } else {
        "stat"
    }
}

fn dossier_node(doc: &Document, tag: &str, class_name: &str, text: Option<&str>) -> Element {
    let el = doc.create_element(tag).unwrap();
    if !class_name.is_empty() {
        let _ = el.set_attribute("class", class_name);
    }
    if let Some(txt) = text {
        el.set_text_content(Some(txt));
    }
    el
}

fn dossier_header(doc: &Document, name: &str, note: &str) -> Element {
    let header = dossier_node(doc, "div", "dossier-section-head", None);
    let h3 = dossier_node(doc, "h3", "", Some(name));
    let small = dossier_node(doc, "small", "", Some(note));
    let _ = header.append_child(&h3);
    let _ = header.append_child(&small);
    header
}

fn stat_list(doc: &Document, entries: &[Value], lang: crate::i18n::Lang, class_name: &str) -> Element {
    let ul = dossier_node(doc, "ul", class_name, None);
    for entry in entries {
        let name = entry.get("name").and_then(Value::as_str).unwrap_or("");
        if name.is_empty() { continue; }
        let li = dossier_node(doc, "li", "", None);
        let sheet_total = aion2_core::character_extraction::option_name(entry).is_none();
        if sheet_total {
            let _ = li.class_list().add_1("sheet-total");
        }
        let icon = icon_element(doc, attribute_icon(name), "");
        let _ = li.append_child(&icon);
        let span = dossier_node(doc, "span", "", Some(&crate::i18n::stat_label(lang, name)));
        let _ = li.append_child(&span);
        let val_opt = entry.get("value").and_then(Value::as_str).or_else(|| entry.get("desc").and_then(Value::as_str));
        if let Some(val) = val_opt {
            let strong = dossier_node(doc, "strong", "", Some(val));
            let is_pct = val.trim().ends_with('%');
            let _ = strong.class_list().add_1(if is_pct { "value-percent" } else { "value-flat" });
            let _ = li.append_child(&strong);
        }
        if let Some(extra) = entry.get("extra").and_then(|v| v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse::<f64>().ok()))) {
            if extra != 0.0 {
                let suffix = crate::i18n::t(lang, "bonusSuffix");
                let sm = dossier_node(doc, "small", "", Some(&format!("+{extra} {suffix}")));
                let _ = li.append_child(&sm);
            }
        }
        if sheet_total {
            let sm = dossier_node(doc, "small", "badge badge-display", Some(&crate::i18n::t(lang, "sheetTotalIncluded")));
            let _ = li.append_child(&sm);
        }
        let _ = ul.append_child(&li);
    }
    ul
}

fn gear_card(doc: &Document, pair: &Value, lang: crate::i18n::Lang) -> Element {
    let card = dossier_node(doc, "article", "gear-card", None);
    let item = &pair["item"];
    let detail = &pair["detail"];
    let head = dossier_node(doc, "div", "gear-card-head", None);
    let icon_url = item.get("icon").and_then(Value::as_str).or_else(|| detail.get("icon").and_then(Value::as_str)).unwrap_or("");
    let img = profile_image(doc, icon_url, "stat", "");
    let _ = head.append_child(&img);

    let title = dossier_node(doc, "div", "", None);
    let unknown_str = crate::i18n::t(lang, "unknownItem");
    let item_name = item.get("name").and_then(Value::as_str).or_else(|| detail.get("name").and_then(Value::as_str)).unwrap_or(&unknown_str);
    let strong = dossier_node(doc, "strong", "", Some(item_name));
    let eq_str = crate::i18n::t(lang, "equipment");
    let slot = item.get("slotPosName").and_then(Value::as_str).unwrap_or(&eq_str);
    let grade_unknown = crate::i18n::t(lang, "gradeUnknown");
    let grade = item.get("grade").and_then(Value::as_str).or_else(|| detail.get("grade").and_then(Value::as_str)).unwrap_or(&grade_unknown);
    let enchant = item.get("enchantLevel").and_then(|v| v.as_i64()).map(|l| format!(" · +{l}")).unwrap_or_default();
    let exceed = item.get("exceedLevel").and_then(|v| v.as_i64()).map(|l| format!(" · exceed {l}")).unwrap_or_default();
    let small = dossier_node(doc, "small", "", Some(&format!("{slot} · {grade}{enchant}{exceed}")));
    let _ = title.append_child(&strong);
    let _ = title.append_child(&small);
    let _ = head.append_child(&title);
    let _ = card.append_child(&head);

    let sections: [(&str, &str); 4] = [
        ("baseOptions", "mainStats"),
        ("soulBind", "subStats"),
        ("manastones", "magicStoneStat"),
        ("theostones", "godStoneStat"),
    ];
    for (label_key, stat_key) in sections {
        if let Some(arr) = detail.get(stat_key).and_then(Value::as_array) {
            if !arr.is_empty() {
                let block = dossier_node(doc, "div", "gear-options", None);
                if stat_key == "godStoneStat" {
                    let _ = block.class_list().add_1("theostone-options");
                }
                let h4 = dossier_node(doc, "h4", "", Some(&crate::i18n::t(lang, label_key)));
                let _ = block.append_child(&h4);
                let list = stat_list(doc, arr, lang, "option-list");
                let _ = block.append_child(&list);
                let _ = card.append_child(&block);
            }
        }
    }

    if let Some(sub_skills) = detail.get("subSkills").and_then(Value::as_array) {
        if !sub_skills.is_empty() {
            let block = dossier_node(doc, "div", "gear-options", None);
            let h4 = dossier_node(doc, "h4", "", Some(&crate::i18n::t(lang, "soulSkills")));
            let _ = block.append_child(&h4);
            let ul = dossier_node(doc, "ul", "option-list", None);
            let sk_default = crate::i18n::t(lang, "skill");
            for skill in sub_skills {
                let li = dossier_node(doc, "li", "", None);
                let icon = profile_image(doc, skill.get("icon").and_then(Value::as_str).unwrap_or(""), "skill", "");
                let span = dossier_node(doc, "span", "", Some(skill.get("name").and_then(Value::as_str).unwrap_or(&sk_default)));
                let lvl = skill.get("level").and_then(|v| v.as_i64()).unwrap_or(0);
                let strg = dossier_node(doc, "strong", "", Some(&format!("+{lvl}")));
                let _ = li.append_child(&icon);
                let _ = li.append_child(&span);
                let _ = li.append_child(&strg);
                let _ = ul.append_child(&li);
            }
            let _ = block.append_child(&ul);
            let _ = card.append_child(&block);
        }
    }

    if card.children().length() == 1 {
        let p = dossier_node(doc, "p", "reference-note", Some(&crate::i18n::t(lang, "detailsUnavailable")));
        let _ = card.append_child(&p);
    }
    card
}

fn skill_state(skill: &Value) -> &'static str {
    let acquired = skill.get("acquired").and_then(|v| v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse().ok()))).unwrap_or(0.0) > 0.0;
    let lvl = skill.get("skillLevel").and_then(|v| v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse().ok()))).unwrap_or(0.0);
    if !acquired || lvl <= 0.0 {
        return "skillNotAcquired";
    }
    let equipped = skill.get("equip").and_then(|v| v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse().ok()))).unwrap_or(0.0) > 0.0;
    if equipped { "equipped" } else { "skillAcquired" }
}

fn skill_card(doc: &Document, skill: &Value, lang: crate::i18n::Lang) -> Element {
    let card = dossier_node(doc, "article", "skill-card", None);
    let icon = profile_image(doc, skill.get("icon").and_then(Value::as_str).unwrap_or(""), "skill", "");
    let _ = card.append_child(&icon);
    let content = dossier_node(doc, "div", "", None);
    let name = skill.get("name").and_then(Value::as_str).unwrap_or("");
    let strong = dossier_node(doc, "strong", "", Some(name));
    let acquired = skill.get("acquired").and_then(|v| v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse().ok()))).unwrap_or(0.0) > 0.0;
    let lvl = skill.get("skillLevel").map(|v| v.to_string().trim_matches('"').to_string()).unwrap_or_default();
    let equipped = skill.get("equip").and_then(|v| v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse().ok()))).unwrap_or(0.0) > 0.0;
    let sub = if acquired {
        let eq_str = if equipped { format!(" · {}", crate::i18n::t(lang, "equipped")) } else { String::new() };
        format!("{} {lvl}{eq_str}", crate::i18n::t(lang, "level"))
    } else {
        let need_lvl = skill.get("needLevel").map(|v| v.to_string().trim_matches('"').to_string()).unwrap_or_default();
        let need_str = if !need_lvl.is_empty() { format!(" · {} {need_lvl}", crate::i18n::t(lang, "level")) } else { String::new() };
        format!("{}{need_str}", crate::i18n::t(lang, "skillNotAcquired"))
    };
    let small = dossier_node(doc, "small", "", Some(&sub));
    let _ = content.append_child(&strong);
    let _ = content.append_child(&small);
    let _ = card.append_child(&content);
    card
}

fn stat_group(stat: &Value) -> &'static str {
    const PRIMARY_STAT_TYPES: &[&str] = &["STR", "DEX", "CON", "INT", "AGI", "WIS"];
    const DIVINITY_TYPES: &[&str] = &["Justice", "Freedom", "Illusion", "Life", "Time", "Destruction", "Death", "Wisdom", "Destiny", "Space"];
    let type_str = stat.get("type").and_then(Value::as_str).unwrap_or("");
    if PRIMARY_STAT_TYPES.contains(&type_str) {
        "primary"
    } else if DIVINITY_TYPES.contains(&type_str) {
        "divinity"
    } else {
        "other"
    }
}

fn effect_role(text: &str) -> &'static str {
    const FORMULA_EFFECT_LABELS: &[&str] = &[
        "Attack increase", "Double Chance", "Double Resist", "Perfect Chance",
        "Perfect Resist", "Endurance Penetration", "Endurance",
        "Regeneration Penetration", "Regeneration",
    ];
    if FORMULA_EFFECT_LABELS.iter().any(|label| text.starts_with(label)) {
        "formula"
    } else {
        "display"
    }
}

fn character_ranking_rows(ranking: &Value) -> Vec<(String, String, String, String)> {
    let scopes: [(&str, &str, &str, &str); 3] = [
        ("rankOverall", "overall_rank", "overall_total", "overall_percentile"),
        ("rankClass", "job_rank", "job_total", "job_percentile"),
        ("rankServer", "server_rank", "server_total", "server_percentile"),
    ];
    let mut rows = Vec::new();
    for (scope, r_key, tot_key, pct_key) in scopes {
        let r = ranking.get(r_key).map(|v| v.to_string().trim_matches('"').to_string()).unwrap_or_default();
        let tot = ranking.get(tot_key).map(|v| v.to_string().trim_matches('"').to_string()).unwrap_or_default();
        let pct = ranking.get(pct_key).and_then(|v| v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse().ok())));
        if r.is_empty() || tot.is_empty() || pct.is_none() {
            continue;
        }
        let pct_str = format!("{:.2}", pct.unwrap());
        rows.push((scope.to_string(), r, tot, pct_str));
    }
    rows
}

fn render_equipment_tab(doc: &Document, sec: &Element, view: &ExtractedCharacter, lang: crate::i18n::Lang) {
    let count_str = view.gear.len().to_string();
    let note = crate::i18n::format_t(lang, "piecesPublished", &[("count", &count_str)]);
    let head = dossier_header(doc, &crate::i18n::t(lang, "equippedItems"), &note);
    let _ = sec.append_child(&head);

    let grid = dossier_node(doc, "div", "gear-grid", None);
    for pair in &view.gear {
        let card = gear_card(doc, pair, lang);
        let _ = grid.append_child(&card);
    }
    let _ = sec.append_child(&grid);

    let companions = dossier_node(doc, "div", "progression-grid", None);
    if let Some(pet) = &view.pet {
        if let Some(name) = pet.get("name").and_then(Value::as_str) {
            let card = dossier_node(doc, "article", "progression-card", None);
            let icon_url = pet.get("icon").and_then(Value::as_str).unwrap_or("");
            let img = profile_image(doc, icon_url, "stat", "");
            let strong = dossier_node(doc, "strong", "", Some(name));
            let lvl = pet.get("level").map(|v| v.to_string().trim_matches('"').to_string()).unwrap_or_default();
            let detail = if lvl.is_empty() {
                crate::i18n::t(lang, "mount")
            } else {
                format!("{} · {} {lvl}", crate::i18n::t(lang, "mount"), crate::i18n::t(lang, "level"))
            };
            let small = dossier_node(doc, "small", "", Some(&detail));
            let _ = card.append_child(&img);
            let _ = card.append_child(&strong);
            let _ = card.append_child(&small);
            let _ = companions.append_child(&card);
        }
    }
    if let Some(wing) = &view.wing {
        if let Some(name) = wing.get("name").and_then(Value::as_str) {
            let card = dossier_node(doc, "article", "progression-card", None);
            let icon_url = wing.get("icon").and_then(Value::as_str).unwrap_or("");
            let img = profile_image(doc, icon_url, "stat", "");
            let strong = dossier_node(doc, "strong", "", Some(name));
            let grade = wing.get("grade").and_then(Value::as_str).unwrap_or("");
            let enchant = wing.get("enchantLevel").and_then(|v| v.as_i64()).map(|l| format!("+{l}")).unwrap_or_default();
            let mut wing_details = Vec::new();
            if !grade.is_empty() { wing_details.push(grade); }
            if !enchant.is_empty() { wing_details.push(enchant.as_str()); }
            let detail = if wing_details.is_empty() {
                crate::i18n::t(lang, "wings")
            } else {
                format!("{} · {}", crate::i18n::t(lang, "wings"), wing_details.join(" · "))
            };
            let small = dossier_node(doc, "small", "", Some(&detail));
            let _ = card.append_child(&img);
            let _ = card.append_child(&strong);
            let _ = card.append_child(&small);
            let _ = companions.append_child(&card);
        }
    }
    let _ = sec.append_child(&companions);
}

fn render_skills_tab(doc: &Document, sec: &Element, view: &ExtractedCharacter, lang: crate::i18n::Lang) {
    let p = dossier_node(doc, "p", "reference-note", Some(&crate::i18n::t(lang, "skillDataNote")));
    let _ = sec.append_child(&p);

    let categories = [
        ("Active", "activeSkills"),
        ("Passive", "passiveSkills"),
        ("Dp", "stigmaSkills"),
    ];

    for (cat, title_key) in categories {
        let cat_head = dossier_header(doc, &crate::i18n::t(lang, title_key), "");
        let _ = sec.append_child(&cat_head);

        for state in ["equipped", "skillAcquired", "skillNotAcquired"] {
            let matching: Vec<&Value> = view.skills.iter().filter(|s| {
                s.get("category").and_then(Value::as_str) == Some(cat) && skill_state(s) == state
            }).collect();

            if !matching.is_empty() {
                let sub_head = dossier_header(doc, &crate::i18n::t(lang, state), &matching.len().to_string());
                let _ = sec.append_child(&sub_head);

                let grid = dossier_node(doc, "div", "skill-grid", None);
                for skill in matching {
                    let card = skill_card(doc, skill, lang);
                    let _ = grid.append_child(&card);
                }
                let _ = sec.append_child(&grid);
            }
        }
    }
}

fn render_attributes_tab(doc: &Document, sec: &Element, view: &ExtractedCharacter, lang: crate::i18n::Lang) {
    let head = dossier_header(doc, &crate::i18n::t(lang, "sheetAttributes"), &crate::i18n::t(lang, "noInferredTotals"));
    let _ = sec.append_child(&head);

    let groups = [
        ("primary", "statsPrimary"),
        ("divinity", "statsDivinities"),
        ("other", "statsOther"),
    ];

    for (grp, title_key) in groups {
        let matching: Vec<&Value> = view.stats.iter().filter(|s| stat_group(s) == grp).collect();
        if !matching.is_empty() {
            let count_str = matching.len().to_string();
            let note = crate::i18n::format_t(lang, "publishedCount", &[("count", &count_str)]);
            let grp_head = dossier_header(doc, &crate::i18n::t(lang, title_key), &note);
            let _ = sec.append_child(&grp_head);

            let grid = dossier_node(doc, "div", "attribute-grid", None);
            for stat in matching {
                let card = dossier_node(doc, "article", "attribute-card", None);
                let name = stat.get("name").and_then(Value::as_str).unwrap_or("");
                let icon = icon_element(doc, attribute_icon(name), "");
                let _ = card.append_child(&icon);

                let text_div = dossier_node(doc, "div", "", None);
                let val_str = stat.get("value").map(|v| v.to_string().trim_matches('"').to_string()).unwrap_or_default();
                let amount = dossier_node(doc, "strong", "", Some(&val_str));
                let _ = amount.class_list().add_1(if val_str.trim().ends_with('%') { "value-percent" } else { "value-flat" });

                let is_item_level = stat.get("type").and_then(Value::as_str) == Some("ItemLevel");
                let label_text = if is_item_level { crate::i18n::t(lang, "itemLevel") } else { crate::i18n::stat_label(lang, name) };
                let span = dossier_node(doc, "span", "", Some(&label_text));
                let _ = text_div.append_child(&span);
                let _ = text_div.append_child(&amount);

                if let Some(second_list) = stat.get("statSecondList").and_then(Value::as_array) {
                    for effect_val in second_list {
                        let effect = effect_val.as_str().unwrap_or("");
                        let role = effect_role(effect);
                        let line = dossier_node(doc, "small", &format!("effect effect-{role}"), Some(&crate::i18n::stat_label(lang, effect)));
                        let space = doc.create_text_node(" ");
                        let badge_text = crate::i18n::t(lang, if role == "formula" { "usedInFormula" } else { "displayOnly" });
                        let badge = dossier_node(doc, "span", &format!("badge badge-{role}"), Some(&badge_text));
                        let _ = line.append_child(&space);
                        let _ = line.append_child(&badge);
                        let _ = text_div.append_child(&line);
                    }
                }
                let _ = card.append_child(&text_div);
                let _ = grid.append_child(&card);
            }
            let _ = sec.append_child(&grid);
        }
    }
}

fn render_progression_tab(doc: &Document, sec: &Element, view: &ExtractedCharacter, lang: crate::i18n::Lang) {
    let count_str = view.titles.len().to_string();
    let note = crate::i18n::format_t(lang, "publishedCount", &[("count", &count_str)]);
    let head = dossier_header(doc, &crate::i18n::t(lang, "equippedTitles"), &note);
    let _ = sec.append_child(&head);

    let title_grid = dossier_node(doc, "div", "progression-grid", None);
    let title_default = crate::i18n::t(lang, "title");
    for title in &view.titles {
        let card = dossier_node(doc, "article", "progression-card", None);
        let title_name = title.get("name").and_then(Value::as_str).unwrap_or("");
        let strong = dossier_node(doc, "strong", "", Some(title_name));
        let equip_cat = title.get("equipCategory").and_then(Value::as_str).unwrap_or(&title_default);
        let grade = title.get("grade").and_then(Value::as_str).unwrap_or("");
        let mut details = Vec::new();
        if !equip_cat.is_empty() { details.push(equip_cat); }
        if !grade.is_empty() { details.push(grade); }
        let small = dossier_node(doc, "small", "", Some(&details.join(" · ")));
        let _ = card.append_child(&strong);
        let _ = card.append_child(&small);

        if let Some(equip_stats) = title.get("equipStatList").and_then(Value::as_array) {
            for stat in equip_stats {
                if let Some(desc) = stat.get("desc").and_then(Value::as_str) {
                    let p = dossier_node(doc, "p", "", Some(desc));
                    let _ = card.append_child(&p);
                }
            }
        }
        if let Some(stats) = title.get("statList").and_then(Value::as_array) {
            for stat in stats {
                if let Some(desc) = stat.get("desc").and_then(Value::as_str) {
                    let owned_bonus = crate::i18n::t(lang, "titleOwnedBonus");
                    let p = dossier_node(doc, "p", "title-owned-bonus", Some(&format!("{owned_bonus}: {desc}")));
                    let _ = card.append_child(&p);
                }
            }
        }
        let _ = title_grid.append_child(&card);
    }
    let _ = sec.append_child(&title_grid);

    // Daevanion boards
    let b_count_str = view.boards.len().to_string();
    let b_note = crate::i18n::format_t(lang, "publishedCount", &[("count", &b_count_str)]);
    let b_head = dossier_header(doc, &crate::i18n::t(lang, "daevanionBoards"), &b_note);
    let _ = sec.append_child(&b_head);

    let board_grid = dossier_node(doc, "div", "progression-grid", None);
    let board_default = crate::i18n::t(lang, "board");
    for board in &view.boards {
        let card = dossier_node(doc, "article", "progression-card", None);
        let icon_url = board.get("icon").and_then(Value::as_str).unwrap_or("");
        let img = profile_image(doc, icon_url, "stat", "");
        let b_name = board.get("name").and_then(Value::as_str).unwrap_or(&board_default);
        let strong = dossier_node(doc, "strong", "", Some(b_name));
        let open_str = board.get("openNodeCount").map(|v| v.to_string()).unwrap_or_else(|| "0".into());
        let tot_str = board.get("totalNodeCount").map(|v| v.to_string()).unwrap_or_else(|| "?".into());
        let node_note = crate::i18n::format_t(lang, "nodesOpen", &[("open", &open_str), ("total", &tot_str)]);
        let small = dossier_node(doc, "small", "", Some(&node_note));
        let _ = card.append_child(&img);
        let _ = card.append_child(&strong);
        let _ = card.append_child(&small);
        let _ = board_grid.append_child(&card);
    }
    let _ = sec.append_child(&board_grid);

    // Arcana
    let a_count_str = view.arcana.len().to_string();
    let a_note = crate::i18n::format_t(lang, "equippedCount", &[("count", &a_count_str)]);
    let a_head = dossier_header(doc, &crate::i18n::t(lang, "arcana"), &a_note);
    let _ = sec.append_child(&a_head);

    let arcana_grid = dossier_node(doc, "div", "gear-grid", None);
    for pair in &view.arcana {
        let card = gear_card(doc, pair, lang);
        let _ = arcana_grid.append_child(&card);
    }
    let _ = sec.append_child(&arcana_grid);
}

pub fn render_dossier(doc: &Document, container: &Element, data: &Value) {
    container.set_inner_html("");
    let lang = crate::i18n::Lang::default();
    let view = extract_character(data);

    // Summary
    let summary = dossier_node(doc, "div", "dossier-summary", None);
    let portrait = profile_image(doc, &view.portrait, "class", &view.class_name);
    let _ = portrait.class_list().add_1("dossier-portrait");
    let _ = summary.append_child(&portrait);

    let summary_text = dossier_node(doc, "div", "", None);
    let title_strong = dossier_node(doc, "strong", "", Some(&format!("{} · {}", view.name, view.class_name)));
    let _ = summary_text.append_child(&title_strong);

    let level_str = if view.level.is_empty() { "—".into() } else { format!("{} {}", crate::i18n::t(lang, "level"), view.level) };
    let race_str = if view.race.is_empty() { crate::i18n::t(lang, "raceUnknown") } else { view.race.clone() };
    let server_str = if view.server.is_empty() { crate::i18n::t(lang, "serverUnknown") } else { view.server.clone() };
    let meta_parts = vec![level_str, race_str, view.gender.clone(), server_str];
    let meta_small = dossier_node(doc, "small", "", Some(&meta_parts.into_iter().filter(|s| !s.is_empty()).collect::<Vec<_>>().join(" · ")));
    let _ = summary_text.append_child(&meta_small);

    // CP, GS and Legion row
    let cp_str = view.combat_power.map(|cp| format!("{} {}", crate::i18n::t(lang, "combatPower"), format_number(cp))).unwrap_or_default();
    let gs_str = view.gear_score.map(|gs| format!("{} {}", crate::i18n::t(lang, "gearScore"), format_number(gs))).unwrap_or_default();
    let scores_row = dossier_node(doc, "small", "", None);
    let mut parts = Vec::new();
    if !cp_str.is_empty() { parts.push(cp_str); }
    if !gs_str.is_empty() { parts.push(gs_str); }
    if !parts.is_empty() {
        let text_node = doc.create_text_node(&parts.join(" · "));
        let _ = scores_row.append_child(&text_node);
    }
    if !view.legion.is_empty() {
        if !parts.is_empty() {
            let sep = doc.create_text_node(" · ");
            let _ = scores_row.append_child(&sep);
        }
        let legion_btn = dossier_node(doc, "button", "legion-link-btn", Some(&format!("{} {}", crate::i18n::t(lang, "legion"), view.legion)));
        let _ = legion_btn.set_attribute("type", "button");
        let _ = legion_btn.set_attribute("data-legion-name", &view.legion);
        let _ = legion_btn.set_attribute("data-server-name", &view.server);
        let _ = scores_row.append_child(&legion_btn);
    }
    let _ = summary_text.append_child(&scores_row);

    // Titles info
    if !view.title_name.is_empty() || view.titles_owned.is_some() {
        let mut t_parts = Vec::new();
        if !view.title_name.is_empty() {
            t_parts.push(format!("{} {}", crate::i18n::t(lang, "equippedTitle"), view.title_name));
        }
        if let Some(owned) = view.titles_owned {
            let total = view.titles_total.map(|n| n.to_string()).unwrap_or_else(|| "?".into());
            t_parts.push(crate::i18n::format_t(lang, "titlesOwned", &[("owned", &owned.to_string()), ("total", &total)]));
        }
        let title_sm = dossier_node(doc, "small", "", Some(&t_parts.join(" · ")));
        let _ = summary_text.append_child(&title_sm);
    }

    // Ranking info
    if let Some(ranking) = &view.ranking {
        let rows = character_ranking_rows(ranking);
        let mut rank_lines = Vec::new();
        for (scope, rank, total, pct) in rows {
            rank_lines.push(crate::i18n::format_t(lang, "rankLine", &[
                ("scope", &crate::i18n::t(lang, &scope)),
                ("rank", &rank),
                ("total", &total),
                ("percent", &pct),
            ]));
        }
        let rank_text = if rank_lines.is_empty() {
            crate::i18n::t(lang, "rankingUnavailable")
        } else {
            rank_lines.join(" · ")
        };
        let rank_sm = dossier_node(doc, "small", "", Some(&format!("{}: {rank_text}", crate::i18n::t(lang, "rankAtool"))));
        let _ = summary_text.append_child(&rank_sm);
    }

    let _ = summary.append_child(&summary_text);
    let _ = container.append_child(&summary);

    // Tabs container
    let tabs_nav = dossier_node(doc, "div", "dossier-tabs", None);
    let _ = tabs_nav.set_attribute("role", "tablist");
    let _ = tabs_nav.set_attribute("aria-label", &crate::i18n::t(lang, "characterDetails"));

    let tab_specs = [
        ("tabEquipment", 0),
        ("tabSkills", 1),
        ("tabAttributes", 2),
        ("tabProgression", 3),
    ];

    let mut sections = Vec::new();
    let mut tab_buttons = Vec::new();

    for (label_key, idx) in tab_specs {
        let btn = dossier_node(doc, "button", "dossier-tab", Some(&crate::i18n::t(lang, label_key)));
        let _ = btn.set_attribute("type", "button");
        let _ = btn.set_attribute("role", "tab");
        let _ = btn.set_attribute("aria-selected", if idx == 0 { "true" } else { "false" });
        let _ = tabs_nav.append_child(&btn);

        let sec = dossier_node(doc, "section", "dossier-pane", None);
        let _ = sec.set_attribute("role", "tabpanel");
        if let Some(html_sec) = sec.dyn_ref::<HtmlElement>() {
            html_sec.set_hidden(idx != 0);
        }

        match idx {
            0 => render_equipment_tab(doc, &sec, &view, lang),
            1 => render_skills_tab(doc, &sec, &view, lang),
            2 => render_attributes_tab(doc, &sec, &view, lang),
            3 => render_progression_tab(doc, &sec, &view, lang),
            _ => {}
        }

        tab_buttons.push(btn);
        sections.push(sec);
    }

    let _ = container.append_child(&tabs_nav);
    for sec in &sections {
        let _ = container.append_child(sec);
    }

    // Attach click listeners to tabs
    for (i, btn) in tab_buttons.iter().enumerate() {
        let tab_buttons_clone = tab_buttons.clone();
        let sections_clone = sections.clone();
        let on_click = Closure::<dyn FnMut()>::new(move || {
            for (j, b) in tab_buttons_clone.iter().enumerate() {
                let active = i == j;
                let _ = b.set_attribute("aria-selected", if active { "true" } else { "false" });
                if let Some(sec) = sections_clone.get(j).and_then(|s| s.dyn_ref::<HtmlElement>()) {
                    sec.set_hidden(!active);
                }
            }
        });
        btn.dyn_ref::<HtmlElement>().unwrap().set_onclick(Some(on_click.as_ref().unchecked_ref()));
        on_click.forget();
    }
}

fn update_profile_status(doc: &Document, data: &Value, lang: crate::i18n::Lang) {
    let is_complete = data.get("complete") != Some(&Value::Bool(false));
    let base_status = crate::i18n::t(lang, if is_complete { "profileLoaded" } else { "partialProfile" });
    let fetched_at = data.pointer("/cacheMeta/fetchedAt").and_then(Value::as_u64);
    let cached = data.pointer("/cacheMeta/cached").and_then(Value::as_bool).unwrap_or(false);

    let status_text = if let Some(ts) = fetched_at {
        let date_str = format_date(ts, lang.as_str());
        let date_msg = crate::i18n::format_t(lang, if cached { "cachedProfileDate" } else { "profileDate" }, &[("date", &date_str)]);
        format!("{base_status} · {date_msg}")
    } else {
        base_status
    };

    if let Some(st) = doc.get_element_by_id("profile-status") {
        st.set_text_content(Some(&status_text));
    }
}

fn show_profile(data: &Value) {
    let doc = document();
    let lang = crate::i18n::Lang::default();
    let view = extract_character(data);

    if let Some(title) = doc.get_element_by_id("profile-title") {
        title.set_text_content(Some(&format!("{} · {}", view.name, view.server)));
    }
    let cp_display = view.combat_power.map(|cp| format!("CP {}", format_number(cp))).unwrap_or_default();
    let gs_display = view.gear_score.map(|gs| format!("GS {}", format_number(gs))).unwrap_or_default();
    let mut scores = Vec::new();
    if !cp_display.is_empty() { scores.push(cp_display); }
    if !gs_display.is_empty() { scores.push(gs_display); }
    let score_str = if scores.is_empty() { String::new() } else { format!(" · {}", scores.join(" · ")) };

    if let Some(meta) = doc.get_element_by_id("profile-meta") {
        meta.set_text_content(Some(&format!(
            "{} · {} {} · {}{score_str}",
            view.class_name,
            crate::i18n::t(lang, "level"),
            view.level,
            view.race
        )));
    }
    if let Some(class_icon) = doc.get_element_by_id("profile-class-icon") {
        set_display_icon(&doc, &class_icon, "class", &view.class_name);
    }
    if let Some(dossier) = doc.get_element_by_id("profile-dossier") {
        render_dossier(&doc, &dossier, data);
    }
    if let Some(import_btn) = doc.get_element_by_id("import-character") {
        let has_bonuses = !equipment_bonuses(data).is_empty();
        let has_refs = !offensive_references(data).is_empty();
        let has_hp = character_hp(data).is_some();
        let can_import = has_bonuses || has_refs || has_hp;
        if can_import {
            let _ = import_btn.remove_attribute("disabled");
        } else {
            let _ = import_btn.set_attribute("disabled", "");
        }
    }

    update_profile_status(&doc, data, lang);
}

pub fn load_character(index: usize, force: bool) {
    let doc = document();
    let lang = crate::i18n::Lang::default();
    let item_opt = STATE.with(|s| s.borrow().results.get(index).cloned());
    let Some(item) = item_opt else { return };

    let key = profile_key(&item);
    let is_fav = get_favorites().contains(&key);
    if let Some(fav_btn) = doc.get_element_by_id("favorite-profile") {
        let _ = fav_btn.set_attribute("aria-pressed", if is_fav { "true" } else { "false" });
    }

    let profile_id = STATE.with(|s| {
        let mut st = s.borrow_mut();
        st.loaded_item = Some(item.clone());
        st.loaded = None;
        st.current_profile += 1;
        st.current_profile
    });

    if let Some(panel) = doc.get_element_by_id("profile-panel").and_then(|e| e.dyn_into::<HtmlElement>().ok()) {
        panel.set_hidden(false);
    }
    if let Some(imp) = doc.get_element_by_id("import-character") {
        let _ = imp.set_attribute("disabled", "");
    }

    let char_name = item.get("name").and_then(Value::as_str).unwrap_or("");
    let srv_name = item.get("serverName").and_then(Value::as_str).unwrap_or("");
    let cls_name = item.get("className").and_then(Value::as_str).unwrap_or("");
    let lvl_str = item.get("level").map(|v| v.to_string().trim_matches('"').to_string()).unwrap_or_default();
    let race_name = item.get("raceName").and_then(Value::as_str).unwrap_or("");
    let cp_val = item.get("combatPower").map(|v| format!(" · CP {v}")).unwrap_or_default();

    if let Some(title) = doc.get_element_by_id("profile-title") {
        title.set_text_content(Some(&format!("{char_name} · {srv_name}")));
    }
    if let Some(meta) = doc.get_element_by_id("profile-meta") {
        meta.set_text_content(Some(&format!("{cls_name} · {} {lvl_str} · {race_name}{cp_val}", crate::i18n::t(lang, "level"))));
    }
    if let Some(class_icon) = doc.get_element_by_id("profile-class-icon") {
        set_display_icon(&doc, &class_icon, "class", cls_name);
    }
    crate::controller::show_loading(&doc, "profile-status", &crate::i18n::t(lang, "loadingProfile"));
    if let Some(dossier) = doc.get_element_by_id("profile-dossier") {
        dossier.set_inner_html("");
    }

    let server_id = item.get("serverId").cloned().unwrap_or(Value::Null);
    let character_id = item.get("characterId").and_then(Value::as_str).unwrap_or("").to_string();
    let query_region = item.get("queryRegion").and_then(Value::as_str).unwrap_or("ww").to_string();
    let query_gr = item.get("queryGlobalRegion").and_then(Value::as_str).unwrap_or("naw").to_string();

    wasm_bindgen_futures::spawn_local(async move {
        let doc = document();
        let lang = crate::i18n::Lang::default();
        let kind = if force { "refresh_profile" } else { "profile" };
        let params = json!({ "serverId": server_id, "characterId": character_id });
        match crate::host::request(kind, &query_region, &query_gr, params).await {
            Ok(data) => {
                let still_active = STATE.with(|s| s.borrow().current_profile == profile_id);
                if !still_active { return; }

                STATE.with(|s| {
                    s.borrow_mut().loaded = Some(data.clone());
                });

                show_profile(&data);
                refresh_saved_profiles();
            }
            Err(err) => {
                let still_active = STATE.with(|s| s.borrow().current_profile == profile_id);
                if still_active {
                    let err_msg = crate::i18n::format_t(lang, "profileFailed", &[("error", &err)]);
                    if let Some(st) = doc.get_element_by_id("profile-status") {
                        st.set_text_content(Some(&err_msg));
                    }
                }
            }
        }
    });
}

fn handle_search_submit() {
    let doc = document();
    let lang = crate::i18n::Lang::default();
    let name_inp = doc.get_element_by_id("character-name").and_then(|e| e.dyn_into::<HtmlInputElement>().ok());
    let name = name_inp.map(|i| i.value().trim().to_string()).unwrap_or_default();
    if name.is_empty() { return; }

    let region = doc.get_element_by_id("character-region").and_then(|e| e.dyn_into::<HtmlSelectElement>().ok()).map(|s| s.value()).unwrap_or_else(|| "ww".into());
    let gr = doc.get_element_by_id("global-region").and_then(|e| e.dyn_into::<HtmlSelectElement>().ok()).map(|s| s.value()).unwrap_or_else(|| "naw".into());

    let search_id = STATE.with(|s| {
        let mut st = s.borrow_mut();
        st.current_search += 1;
        st.current_profile += 1;
        st.results.clear();
        st.loaded = None;
        st.loaded_item = None;
        st.current_search
    });

    if let Some(lbl) = doc.get_element_by_id("character-choice-label").and_then(|e| e.dyn_into::<HtmlElement>().ok()) {
        lbl.set_hidden(true);
    }
    if let Some(panel) = doc.get_element_by_id("profile-panel").and_then(|e| e.dyn_into::<HtmlElement>().ok()) {
        panel.set_hidden(true);
    }
    if let Some(imp) = doc.get_element_by_id("import-character") {
        let _ = imp.set_attribute("disabled", "");
    }
    crate::controller::show_loading(&doc, "search-status", &crate::i18n::t(lang, "searchingProfiles"));

    wasm_bindgen_futures::spawn_local(async move {
        let doc = document();
        let lang = crate::i18n::Lang::default();
        match crate::host::request("search", &region, &gr, json!({ "name": name })).await {
            Ok(data) => {
                let still_active = STATE.with(|s| s.borrow().current_search == search_id);
                if !still_active { return; }

                let list = data.get("results").and_then(Value::as_array).cloned().unwrap_or_default();
                let mut results_with_query = Vec::new();
                for mut item in list {
                    if let Some(obj) = item.as_object_mut() {
                        obj.insert("queryRegion".into(), Value::from(region.clone()));
                        obj.insert("queryGlobalRegion".into(), Value::from(gr.clone()));
                    }
                    results_with_query.push(item);
                }

                if results_with_query.is_empty() {
                    let is_processing = data.get("complete") == Some(&Value::Bool(false));
                    let msg = crate::i18n::t(lang, if is_processing { "searchStillProcessing" } else { "noCharacters" });
                    if let Some(st) = doc.get_element_by_id("search-status") {
                        st.set_text_content(Some(&msg));
                    }
                    return;
                }

                // Add to recent searches
                let mut recents = get_recent_searches();
                recents.retain(|s| s != &name);
                recents.insert(0, name.clone());
                if recents.len() > 10 { recents.truncate(10); }
                save_recent_searches(&recents);
                render_recent_searches(&doc);

                STATE.with(|s| {
                    s.borrow_mut().results = results_with_query.clone();
                });

                let count_str = results_with_query.len().to_string();
                let is_partial = data.get("complete") == Some(&Value::Bool(false));
                let extra_str = if is_partial { crate::i18n::t(lang, "moreMayAppear") } else { String::new() };
                let found_msg = crate::i18n::format_t(lang, "charactersFound", &[
                    ("count", &count_str),
                    ("extra", &extra_str),
                ]);
                if let Some(st) = doc.get_element_by_id("search-status") {
                    st.set_text_content(Some(&found_msg));
                }

                // Populate choice select
                if let Some(choice) = doc.get_element_by_id("character-choice").and_then(|e| e.dyn_into::<HtmlSelectElement>().ok()) {
                    choice.set_inner_html("");
                    let default_opt = doc.create_element("option").unwrap();
                    let _ = default_opt.set_attribute("value", "");
                    default_opt.set_text_content(Some(&crate::i18n::t(lang, "selectCharacter")));
                    let _ = choice.append_child(&default_opt);

                    for (idx, item) in results_with_query.iter().enumerate() {
                        let opt = doc.create_element("option").unwrap();
                        let _ = opt.set_attribute("value", &idx.to_string());
                        let char_name = item.get("name").and_then(Value::as_str).unwrap_or("");
                        let srv_name = item.get("serverName").and_then(Value::as_str).unwrap_or("");
                        let cls_name = item.get("className").and_then(Value::as_str).filter(|c| *c != "Unknown").unwrap_or("");
                        let cp_str = item.get("combatPower").and_then(|v| v.as_f64()).map(|cp| format!("CP {}", format_number(cp))).unwrap_or_default();
                        let mut details = Vec::new();
                        if !srv_name.is_empty() { details.push(srv_name); }
                        if !cls_name.is_empty() { details.push(cls_name); }
                        if !cp_str.is_empty() { details.push(cp_str.as_str()); }
                        let detail_str = if details.is_empty() { String::new() } else { format!(" · {}", details.join(" · ")) };
                        opt.set_text_content(Some(&format!("{char_name}{detail_str}")));
                        let _ = choice.append_child(&opt);
                    }

                    if results_with_query.len() == 1 {
                        load_character(0, false);
                    } else if let Some(lbl) = doc.get_element_by_id("character-choice-label").and_then(|e| e.dyn_into::<HtmlElement>().ok()) {
                        lbl.set_hidden(false);
                    }
                }
            }
            Err(err) => {
                let still_active = STATE.with(|s| s.borrow().current_search == search_id);
                if still_active {
                    let err_msg = crate::i18n::format_t(lang, "searchFailed", &[("error", &err)]);
                    if let Some(st) = doc.get_element_by_id("search-status") {
                        st.set_text_content(Some(&err_msg));
                    }
                }
            }
        }
    });
}

fn handle_favorite_click() {
    let item_opt = STATE.with(|s| s.borrow().loaded_item.clone());
    let Some(item) = item_opt else { return };
    let key = profile_key(&item);

    let mut favorites = get_favorites();
    let is_fav = if favorites.contains(&key) {
        favorites.retain(|f| f != &key);
        false
    } else {
        favorites.push(key);
        if favorites.len() > 20 {
            favorites.remove(0);
        }
        true
    };
    save_favorites(&favorites);

    let doc = document();
    if let Some(fav_btn) = doc.get_element_by_id("favorite-profile") {
        let _ = fav_btn.set_attribute("aria-pressed", if is_fav { "true" } else { "false" });
    }
    refresh_saved_profiles();
}

fn handle_refresh_click() {
    let has_item = STATE.with(|s| s.borrow().loaded_item.is_some());
    if has_item {
        load_character(0, true);
    }
}

const BONUS_FIELD_DEFAULTS: &[(&str, f64)] = &[
    ("attackBonus", 0.0),
    ("pveAttackPower", 0.0),
    ("bossAttackPower", 0.0),
    ("penetration", 0.0),
    ("weaponDamageBoost", 100.0),
    ("damageBoost", 100.0),
    ("pveDamageBoost", 0.0),
    ("bossDamageBoost", 0.0),
    ("attackIncrease", 100.0),
    ("criticalDamageBoost", 150.0),
    ("frontBoost", 100.0),
    ("backBoost", 100.0),
    ("skillSpecialtyEnhancement", 100.0),
];

pub fn apply_character_import(profile: &Value) {
    let doc = document();
    let lang = crate::i18n::Lang::default();
    let bonuses = equipment_bonuses(profile);
    let references = offensive_references(profile);

    STATE.with(|s| {
        let mut state = s.borrow_mut();
        for (field, prev) in &state.previous_import {
            if !bonuses.contains_key(field) {
                if let Some(inp) = doc.query_selector(&format!("#calculator input[name=\"{field}\"]")).ok().flatten().and_then(|n| n.dyn_into::<HtmlInputElement>().ok()) {
                    if aion2_core::numeric_input::parse_numeric_input(&inp.value(), f64::NEG_INFINITY) == Some(*prev) {
                        let def = BONUS_FIELD_DEFAULTS.iter().find(|(k, _)| *k == field.as_str()).map(|(_, v)| *v).unwrap_or(0.0);
                        inp.set_value(&def.to_string());
                    }
                }
            }
        }
        for (field, prev) in &state.previous_ref_import {
            if !references.contains_key(field) {
                if let Some(inp) = doc.query_selector(&format!("#calculator input[name=\"{field}\"]")).ok().flatten().and_then(|n| n.dyn_into::<HtmlInputElement>().ok()) {
                    if aion2_core::numeric_input::parse_numeric_input(&inp.value(), f64::NEG_INFINITY) == Some(*prev) {
                        inp.set_value("0");
                    }
                }
            }
        }

        let mut new_prev_ref = HashMap::new();
        for (field, value) in &references {
            if let Some(inp) = doc.query_selector(&format!("#calculator input[name=\"{field}\"]")).ok().flatten().and_then(|n| n.dyn_into::<HtmlInputElement>().ok()) {
                inp.set_value(&value.to_string());
                new_prev_ref.insert(field.clone(), *value);
            }
        }
        state.previous_ref_import = new_prev_ref;

        let mut new_prev_bonuses = HashMap::new();
        for (field, value) in &bonuses {
            if let Some(inp) = doc.query_selector(&format!("#calculator input[name=\"{field}\"]")).ok().flatten().and_then(|n| n.dyn_into::<HtmlInputElement>().ok()) {
                inp.set_value(&value.to_string());
                new_prev_bonuses.insert(field.clone(), *value);
            }
        }
        state.previous_import = new_prev_bonuses;
    });

    let attacker = extract_character(profile);
    if let Some(inp) = doc.get_element_by_id("attacker-name").and_then(|e| e.dyn_into::<HtmlInputElement>().ok()) {
        inp.set_value(&attacker.name);
    }
    if let Some(inp) = doc.get_element_by_id("attacker-class").and_then(|e| e.dyn_into::<HtmlInputElement>().ok()) {
        inp.set_value(&attacker.class_name);
    }
    if let Some(hp_inp) = doc.query_selector("#calculator input[name=\"playerHp\"]").ok().flatten().and_then(|n| n.dyn_into::<HtmlInputElement>().ok()) {
        let hp_val = character_hp(profile).unwrap_or(0.0);
        hp_inp.set_value(&hp_val.to_string());
    }

    if let Some(calc) = doc.get_element_by_id("calculator") {
        let init = web_sys::CustomEventInit::new();
        init.set_bubbles(true);
        if let Ok(ev) = web_sys::CustomEvent::new_with_event_init_dict("input", &init) {
            let _ = calc.dispatch_event(&ev);
        }
    }

    let defenses = enemy_defenses(profile);
    let stats = &attacker.stats;
    let modifiers = sheet_modifiers(profile);
    let detail_val = json!({
        "name": attacker.name,
        "defenses": defenses,
        "stats": stats,
        "modifiers": modifiers,
    });
    let detail_js = js_sys::JSON::parse(&detail_val.to_string()).unwrap_or(JsValue::NULL);
    let custom_init = web_sys::CustomEventInit::new();
    custom_init.set_detail(&detail_js);
    if let Ok(ev) = web_sys::CustomEvent::new_with_event_init_dict("character-imported", &custom_init) {
        let _ = doc.dispatch_event(&ev);
    }

    let coverage = formula_import_coverage(profile);
    let mut manual_labels = Vec::new();
    for field in &coverage.manual {
        if let Some(inp) = doc.query_selector(&format!("#calculator input[name=\"{field}\"]")).ok().flatten() {
            if let Some(parent) = inp.parent_element() {
                let text = parent.text_content().unwrap_or_default().trim().to_string();
                if !text.is_empty() {
                    manual_labels.push(text);
                    continue;
                }
            }
        }
        manual_labels.push(field.clone());
    }
    let manual_str = if manual_labels.is_empty() {
        crate::i18n::t(lang, "importManualFallback")
    } else {
        manual_labels.join(", ")
    };
    let gear_count = coverage.sources.values().filter(|s| s.as_str() == "gear").count();
    let sheet_count = coverage.sources.len().saturating_sub(gear_count);
    let coverage_text = crate::i18n::format_t(lang, "importCoverage", &[
        ("formula", &coverage.imported.len().to_string()),
        ("sheet", &sheet_count.to_string()),
        ("gear", &gear_count.to_string()),
        ("references", &references.len().to_string()),
        ("manual", &manual_str),
    ]);

    if let Some(st) = doc.get_element_by_id("profile-status") {
        st.set_text_content(Some(&coverage_text));
    }
    if let Some(warn) = doc.get_element_by_id("import-warning") {
        warn.set_text_content(Some(&coverage_text));
        if let Some(el) = warn.dyn_ref::<HtmlElement>() {
            el.set_hidden(false);
        }
    }

    show_tab(&doc, "calculator-view");
    if let Some(enc) = doc.query_selector(".encounter").ok().flatten() {
        enc.scroll_into_view();
    }
}

fn show_tab(doc: &Document, tab_id: &str) {
    if let Ok(tabs) = doc.query_selector_all(".tabs .tab") {
        for i in 0..tabs.length() {
            if let Some(el) = tabs.item(i).and_then(|n| n.dyn_into::<HtmlElement>().ok()) {
                let active = el.get_attribute("data-tab").as_deref() == Some(tab_id);
                let _ = el.class_list().toggle_with_force("active", active);
                let _ = el.set_attribute("aria-selected", if active { "true" } else { "false" });
            }
        }
    }
    if let Ok(panels) = doc.query_selector_all(".tab-panel") {
        for i in 0..panels.length() {
            if let Some(el) = panels.item(i).and_then(|n| n.dyn_into::<HtmlElement>().ok()) {
                let matches = el.id() == tab_id;
                el.set_hidden(!matches);
            }
        }
    }
}

fn open_legion_dialog(legion_name: &str, server_name: &str) {
    let doc = document();
    let Some(dialog) = doc.get_element_by_id("legion-dialog") else { return };
    let Some(list) = doc.get_element_by_id("legion-members-list") else { return };
    if let Some(title) = doc.get_element_by_id("legion-dialog-title") {
        title.set_text_content(Some(&format!("Legion: {legion_name}")));
    }
    let srv = if server_name.is_empty() { "ativo" } else { server_name };
    crate::controller::show_loading(&doc, "legion-dialog-summary", &format!("Consultando integrantes da legião no servidor {srv}…"));
    list.set_inner_html("");

    let _ = js_sys::Reflect::get(&dialog, &"showModal".into()).ok().and_then(|f| f.dyn_into::<js_sys::Function>().ok()).map(|f| f.call0(&dialog));

    let legion = legion_name.to_string();
    let server = if server_name.is_empty() { "Siel".to_string() } else { server_name.to_string() };
    let server_id = STATE.with(|s| {
        s.borrow().loaded_item.as_ref().and_then(|item| item.get("serverId").and_then(|v| v.as_i64())).unwrap_or(1401)
    });

    wasm_bindgen_futures::spawn_local(async move {
        let doc = document();
        let params = json!({
            "serverName": server,
            "legionName": legion,
            "serverId": server_id
        });
        let remote_res = crate::host::request("legion_roster", "ww", "", params).await;
        let mut shown_remote = false;
        if let Ok(ref data) = remote_res {
            if let Some(members) = data.get("members").and_then(Value::as_array) {
                if !members.is_empty() {
                    shown_remote = true;
                    if let Some(summary) = doc.get_element_by_id("legion-dialog-summary") {
                        let total = data.get("total_members").map(|v| v.to_string()).unwrap_or_else(|| members.len().to_string());
                        let cp2 = data.get("avg_combat_power2").and_then(|v| v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse().ok()))).unwrap_or(0.0);
                        let cp = data.get("avg_combat_power").and_then(|v| v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse().ok()))).unwrap_or(0.0);
                        let rank = data.get("server_rank").map(|v| v.to_string()).unwrap_or_else(|| "—".into());
                        summary.set_text_content(Some(&format!(
                            "Membros registrados: {total} · Média CP: {} · Média GS: {} · Rank do Servidor: #{rank}",
                            format_number(cp2),
                            format_number(cp)
                        )));
                    }
                    if let Some(list) = doc.get_element_by_id("legion-members-list") {
                        list.set_inner_html("");
                        for m in members {
                            let card = dossier_node(&doc, "article", "progression-card", None);
                            let strong = dossier_node(&doc, "strong", "", Some(&format!("{} · {}", m.get("nickname").and_then(Value::as_str).unwrap_or(""), m.get("job").and_then(Value::as_str).unwrap_or(""))));
                            let race = m.get("race").and_then(Value::as_str).unwrap_or("");
                            let cp_val = m.get("combat_power2").and_then(|v| v.as_f64()).map(|v| format!("CP {}", format_number(v))).unwrap_or_default();
                            let gs_val = m.get("combat_power").and_then(|v| v.as_f64()).map(|v| format!("GS {}", format_number(v))).unwrap_or_default();
                            let srv = m.get("server").and_then(Value::as_str).unwrap_or("");
                            let parts: Vec<&str> = [race, cp_val.as_str(), gs_val.as_str(), srv].into_iter().filter(|s| !s.is_empty()).collect();
                            let small = dossier_node(&doc, "small", "", Some(&parts.join(" · ")));
                            let _ = card.append_child(&strong);
                            let _ = card.append_child(&small);
                            let _ = list.append_child(&card);
                        }
                    }
                }
            }
        }

        if !shown_remote {
            // Fallback to locally saved profiles
            if let Ok(res) = crate::host::request("saved_profiles", "ww", "", json!({})).await {
                let mut local_members = Vec::new();
                if let Some(arr) = res.as_array() {
                    for entry in arr {
                        let v = aion2_core::character_extraction::extract_character(&entry["profile"]);
                        if v.legion.eq_ignore_ascii_case(&legion) {
                            local_members.push(v);
                        }
                    }
                }
                if !local_members.is_empty() {
                    if let Some(summary) = doc.get_element_by_id("legion-dialog-summary") {
                        summary.set_text_content(Some(&format!("Membros registrados localmente ({}) no servidor {server}.", local_members.len())));
                    }
                    if let Some(list) = doc.get_element_by_id("legion-members-list") {
                        list.set_inner_html("");
                        for m in local_members {
                            let card = dossier_node(&doc, "article", "progression-card", None);
                            let strong = dossier_node(&doc, "strong", "", Some(&format!("{} · {}", m.name, m.class_name)));
                            let cp_val = m.combat_power.map(|cp| format!(" · CP {}", format_number(cp))).unwrap_or_default();
                            let gs_val = m.gear_score.map(|gs| format!(" · GS {}", format_number(gs))).unwrap_or_default();
                            let small = dossier_node(&doc, "small", "", Some(&format!("Nível {}{cp_val}{gs_val}", m.level)));
                            let _ = card.append_child(&strong);
                            let _ = card.append_child(&small);
                            let _ = list.append_child(&card);
                        }
                    }
                } else {
                    if let Some(summary) = doc.get_element_by_id("legion-dialog-summary") {
                        summary.set_text_content(Some("Consulta remota indisponível no momento."));
                    }
                    if let Some(list) = doc.get_element_by_id("legion-members-list") {
                        list.set_inner_html("<p class=\"reference-note\">Não foi possível carregar a lista de integrantes remota e nenhum outro membro desta legião foi consultado localmente ainda.</p>");
                    }
                }
            }
        }
    });
}

pub fn refresh_saved_profiles() {
    wasm_bindgen_futures::spawn_local(async {
        let doc = document();
        let lang = crate::i18n::Lang::default();
        if let Ok(res) = crate::host::request("saved_profiles", "ww", "", json!({})).await {
            if let Some(arr) = res.as_array() {
                render_saved_profiles(&doc, arr, lang);
            }
        }
    });
}

fn render_saved_profiles(doc: &Document, entries: &[Value], lang: crate::i18n::Lang) {
    let Some(container) = doc.get_element_by_id("saved-profiles") else { return };
    container.set_inner_html("");
    let favorites = get_favorites();
    let mut sorted = entries.to_vec();
    sorted.sort_by(|a, b| {
        let key_a = format!("{}|{}|{}", a.get("globalRegion").and_then(Value::as_str).unwrap_or(""), a.get("serverId").map(|v| v.to_string()).unwrap_or_default(), a.get("characterId").and_then(Value::as_str).unwrap_or(""));
        let key_b = format!("{}|{}|{}", b.get("globalRegion").and_then(Value::as_str).unwrap_or(""), b.get("serverId").map(|v| v.to_string()).unwrap_or_default(), b.get("characterId").and_then(Value::as_str).unwrap_or(""));
        let fav_a = favorites.contains(&key_a);
        let fav_b = favorites.contains(&key_b);
        fav_b.cmp(&fav_a).then_with(|| {
            let time_a = a.get("fetchedAt").and_then(Value::as_u64).unwrap_or(0);
            let time_b = b.get("fetchedAt").and_then(Value::as_u64).unwrap_or(0);
            time_b.cmp(&time_a)
        })
    });

    for entry in sorted {
        let view = extract_character(&entry["profile"]);
        let gr = entry.get("globalRegion").and_then(Value::as_str).unwrap_or("naw").to_string();
        let server_id = entry.get("serverId").cloned().unwrap_or(Value::Null);
        let character_id = entry.get("characterId").and_then(Value::as_str).unwrap_or("").to_string();
        let key = format!("{gr}|{server_id}|{character_id}");
        let is_fav = favorites.contains(&key);
        let star = if is_fav { "★ " } else { "" };
        let date_str = entry.get("fetchedAt").and_then(Value::as_u64).map(|ts| format_date(ts, lang.as_str())).unwrap_or_default();
        let date_part = if date_str.is_empty() { String::new() } else { format!(" · {date_str}") };

        let btn = dossier_node(doc, "button", "", Some(&format!("{star}{} · {}{date_part}", view.name, view.server)));
        let _ = btn.set_attribute("type", "button");

        let item = json!({
            "name": view.name,
            "serverName": view.server,
            "className": view.class_name,
            "level": view.level,
            "raceName": view.race,
            "combatPower": view.combat_power,
            "serverId": server_id,
            "characterId": character_id,
            "queryRegion": "ww",
            "queryGlobalRegion": gr,
        });

        let on_click = Closure::<dyn FnMut()>::new(move || {
            let item_clone = item.clone();
            STATE.with(|s| {
                s.borrow_mut().results = vec![item_clone];
            });
            load_character(0, false);
        });
        btn.dyn_ref::<HtmlElement>().unwrap().set_onclick(Some(on_click.as_ref().unchecked_ref()));
        on_click.forget();
        let _ = container.append_child(&btn);
    }
}

fn reset_search(doc: &Document) {
    let lang = crate::i18n::Lang::default();
    STATE.with(|s| {
        let mut st = s.borrow_mut();
        st.current_search += 1;
        st.current_profile += 1;
        st.results.clear();
        st.loaded = None;
        st.loaded_item = None;
    });
    if let Some(lbl) = doc.get_element_by_id("character-choice-label").and_then(|e| e.dyn_into::<HtmlElement>().ok()) {
        lbl.set_hidden(true);
    }
    if let Some(panel) = doc.get_element_by_id("profile-panel").and_then(|e| e.dyn_into::<HtmlElement>().ok()) {
        panel.set_hidden(true);
    }
    if let Some(st) = doc.get_element_by_id("search-status") {
        st.set_text_content(Some(&crate::i18n::t(lang, "searchAgain")));
    }
}

pub fn init_character_search() {
    let doc = document();

    render_recent_searches(&doc);
    refresh_saved_profiles();

    wasm_bindgen_futures::spawn_local(async {
        let doc = document();
        if let Ok(settings) = crate::host::request("sync_settings", "ww", "", json!({})).await {
            if let Some(enabled) = settings.get("enabled").and_then(Value::as_bool) {
                if let Some(inp) = doc.get_element_by_id("sync-enabled").and_then(|e| e.dyn_into::<HtmlInputElement>().ok()) {
                    inp.set_checked(enabled);
                }
            }
            if let Some(interval) = settings.get("interval").and_then(Value::as_u64) {
                if let Some(inp) = doc.get_element_by_id("sync-interval").and_then(|e| e.dyn_into::<HtmlInputElement>().ok()) {
                    inp.set_value(&interval.to_string());
                }
            }
        }
    });

    if let Some(sync_chk) = doc.get_element_by_id("sync-enabled").and_then(|e| e.dyn_into::<HtmlInputElement>().ok()) {
        let on_change = Closure::<dyn FnMut()>::new(move || {
            let doc = document();
            let enabled = doc.get_element_by_id("sync-enabled").and_then(|e| e.dyn_into::<HtmlInputElement>().ok()).map(|i| i.checked()).unwrap_or(true);
            let interval = doc.get_element_by_id("sync-interval").and_then(|e| e.dyn_into::<HtmlInputElement>().ok()).and_then(|i| i.value().parse::<u64>().ok()).unwrap_or(600);
            wasm_bindgen_futures::spawn_local(async move {
                let _ = crate::host::request("sync_settings", "ww", "", json!({ "enabled": enabled, "interval": interval })).await;
            });
        });
        sync_chk.set_onchange(Some(on_change.as_ref().unchecked_ref()));
        on_change.forget();
    }
    if let Some(sync_int) = doc.get_element_by_id("sync-interval").and_then(|e| e.dyn_into::<HtmlInputElement>().ok()) {
        let on_change = Closure::<dyn FnMut()>::new(move || {
            let doc = document();
            let enabled = doc.get_element_by_id("sync-enabled").and_then(|e| e.dyn_into::<HtmlInputElement>().ok()).map(|i| i.checked()).unwrap_or(true);
            let interval = doc.get_element_by_id("sync-interval").and_then(|e| e.dyn_into::<HtmlInputElement>().ok()).and_then(|i| i.value().parse::<u64>().ok()).unwrap_or(600);
            wasm_bindgen_futures::spawn_local(async move {
                let _ = crate::host::request("sync_settings", "ww", "", json!({ "enabled": enabled, "interval": interval })).await;
            });
        });
        sync_int.set_onchange(Some(on_change.as_ref().unchecked_ref()));
        on_change.forget();
    }

    if let Some(region_sel) = doc.get_element_by_id("character-region").and_then(|e| e.dyn_into::<HtmlSelectElement>().ok()) {
        let on_change = Closure::<dyn FnMut()>::new(move || {
            let doc = document();
            let region = doc.get_element_by_id("character-region").and_then(|e| e.dyn_into::<HtmlSelectElement>().ok()).map(|s| s.value()).unwrap_or_else(|| "ww".into());
            if let Some(gr_lbl) = doc.get_element_by_id("global-region-label").and_then(|e| e.dyn_into::<HtmlElement>().ok()) {
                gr_lbl.set_hidden(region != "ww");
            }
            reset_search(&doc);
        });
        region_sel.set_onchange(Some(on_change.as_ref().unchecked_ref()));
        on_change.forget();
    }
    if let Some(gr_sel) = doc.get_element_by_id("global-region").and_then(|e| e.dyn_into::<HtmlSelectElement>().ok()) {
        let on_change = Closure::<dyn FnMut()>::new(move || {
            let doc = document();
            reset_search(&doc);
        });
        gr_sel.set_onchange(Some(on_change.as_ref().unchecked_ref()));
        on_change.forget();
    }

    if let Some(form) = doc.get_element_by_id("character-search").and_then(|e| e.dyn_into::<HtmlElement>().ok()) {
        let on_submit = Closure::<dyn FnMut(web_sys::Event)>::new(move |ev: web_sys::Event| {
            ev.prevent_default();
            handle_search_submit();
        });
        form.set_onsubmit(Some(on_submit.as_ref().unchecked_ref()));
        on_submit.forget();
    }

    if let Some(choice) = doc.get_element_by_id("character-choice").and_then(|e| e.dyn_into::<HtmlSelectElement>().ok()) {
        let on_change = Closure::<dyn FnMut()>::new(move || {
            let doc = document();
            if let Some(sel) = doc.get_element_by_id("character-choice").and_then(|e| e.dyn_into::<HtmlSelectElement>().ok()) {
                let val = sel.value();
                if !val.is_empty() {
                    if let Ok(idx) = val.parse::<usize>() {
                        load_character(idx, false);
                    }
                }
            }
        });
        choice.set_onchange(Some(on_change.as_ref().unchecked_ref()));
        on_change.forget();
    }

    if let Some(fav_btn) = doc.get_element_by_id("favorite-profile").and_then(|e| e.dyn_into::<HtmlElement>().ok()) {
        let on_click = Closure::<dyn FnMut()>::new(move || {
            handle_favorite_click();
        });
        fav_btn.set_onclick(Some(on_click.as_ref().unchecked_ref()));
        on_click.forget();
    }

    if let Some(ref_btn) = doc.get_element_by_id("refresh-profile").and_then(|e| e.dyn_into::<HtmlElement>().ok()) {
        let on_click = Closure::<dyn FnMut()>::new(move || {
            handle_refresh_click();
        });
        ref_btn.set_onclick(Some(on_click.as_ref().unchecked_ref()));
        on_click.forget();
    }

    if let Some(imp_btn) = doc.get_element_by_id("import-character").and_then(|e| e.dyn_into::<HtmlElement>().ok()) {
        let on_click = Closure::<dyn FnMut()>::new(move || {
            let profile = STATE.with(|s| s.borrow().loaded.clone());
            if let Some(p) = profile {
                apply_character_import(&p);
            }
        });
        imp_btn.set_onclick(Some(on_click.as_ref().unchecked_ref()));
        on_click.forget();
    }

    let doc_click = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(move |ev: web_sys::MouseEvent| {
        if let Some(target) = ev.target().and_then(|t| t.dyn_into::<Element>().ok()) {
            if let Ok(Some(btn)) = target.closest(".legion-link-btn") {
                let legion_name = btn.get_attribute("data-legion-name").unwrap_or_default();
                let server_name = btn.get_attribute("data-server-name").unwrap_or_default();
                if !legion_name.is_empty() {
                    open_legion_dialog(&legion_name, &server_name);
                }
            }
        }
    });
    let _ = doc.add_event_listener_with_callback("click", doc_click.as_ref().unchecked_ref());
    doc_click.forget();

    if let Some(window) = web_sys::window() {
        let sync_listener = Closure::<dyn FnMut(web_sys::CustomEvent)>::new(move |event: web_sys::CustomEvent| {
            let Ok(text) = js_sys::JSON::stringify(&event.detail()) else { return };
            let Some(text) = text.as_string() else { return };
            let Ok(update) = serde_json::from_str::<Value>(&text) else { return };

            STATE.with(|s| {
                let mut state = s.borrow_mut();
                if let Some(loaded_item) = &state.loaded_item {
                    let s_server_id = loaded_item.get("serverId").map(|v| v.to_string());
                    let s_char_id = loaded_item.get("characterId").and_then(Value::as_str);
                    let s_gr = loaded_item.get("queryGlobalRegion").and_then(Value::as_str);

                    let u_server_id = update.get("serverId").map(|v| v.to_string());
                    let u_char_id = update.get("characterId").and_then(Value::as_str);
                    let u_gr = update.get("globalRegion").and_then(Value::as_str);

                    if s_server_id == u_server_id && s_char_id == u_char_id && s_gr == u_gr {
                        let doc = document();
                        let lang = crate::i18n::Lang::default();
                        if let Some(err) = update.get("error").and_then(Value::as_str) {
                            if let Some(st) = doc.get_element_by_id("profile-status") {
                                st.set_text_content(Some(&crate::i18n::format_t(lang, "profileSyncFailed", &[("error", err)])));
                            }
                        } else if let Some(data) = update.get("data") {
                            state.loaded = Some(data.clone());
                            show_profile(data);
                        }
                    }
                }
            });
            refresh_saved_profiles();
        });
        let _ = window.add_event_listener_with_callback("aion2-profile-sync", sync_listener.as_ref().unchecked_ref());
        sync_listener.forget();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_key() {
        let item = json!({
            "queryGlobalRegion": "naw",
            "serverId": 1401,
            "characterId": "abc123"
        });
        assert_eq!(profile_key(&item), "naw|1401|abc123");
    }

    #[test]
    fn test_stat_group() {
        assert_eq!(stat_group(&json!({ "type": "STR" })), "primary");
        assert_eq!(stat_group(&json!({ "type": "Destruction" })), "divinity");
        assert_eq!(stat_group(&json!({ "type": "ItemLevel" })), "other");
    }

    #[test]
    fn test_effect_role() {
        assert_eq!(effect_role("Attack increase +0.6%"), "formula");
        assert_eq!(effect_role("Perfect Chance +5.8%"), "formula");
        assert_eq!(effect_role("Accuracy increase +2.3%"), "display");
        assert_eq!(effect_role("Cooldown -1.1%"), "display");
    }

    #[test]
    fn test_skill_state() {
        let not_acq = json!({ "acquired": 0, "skillLevel": 0, "equip": 0 });
        assert_eq!(skill_state(&not_acq), "skillNotAcquired");

        let acq = json!({ "acquired": 1, "skillLevel": 5, "equip": 0 });
        assert_eq!(skill_state(&acq), "skillAcquired");

        let eq = json!({ "acquired": 1, "skillLevel": 5, "equip": 1 });
        assert_eq!(skill_state(&eq), "equipped");
    }

    #[test]
    fn test_character_ranking_rows() {
        let ranking = json!({
            "overall_rank": 10,
            "overall_total": 500,
            "overall_percentile": 2.0,
            "job_rank": 2,
            "job_total": 80,
            "job_percentile": 2.5,
            "server_rank": 1,
            "server_total": 40,
            "server_percentile": 2.5
        });
        let rows = character_ranking_rows(&ranking);
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].0, "rankOverall");
        assert_eq!(rows[0].1, "10");
        assert_eq!(rows[0].2, "500");
        assert_eq!(rows[0].3, "2.00");
    }

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(1234567.0), "1,234,567");
        assert_eq!(format_number(42.0), "42");
        assert_eq!(format_number(1000.0), "1,000");
    }

    #[test]
    fn test_attribute_icon() {
        assert_eq!(attribute_icon("HP"), "health");
        assert_eq!(attribute_icon("Defense"), "defense");
        assert_eq!(attribute_icon("Critical Hit"), "chance");
        assert_eq!(attribute_icon("Attack"), "attack");
        assert_eq!(attribute_icon("Damage Boost"), "damage");
        assert_eq!(attribute_icon("Random"), "stat");
    }
}
