use std::collections::HashMap;
use std::sync::LazyLock;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lang {
    En,
    Pt,
}

impl Default for Lang {
    fn default() -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
                if let Ok(Some(saved)) = storage.get_item("aion2_calculator_lang") {
                    if saved == "en" { return Lang::En; }
                }
            }
        }
        Lang::Pt
    }
}

impl Lang {
    pub fn as_str(self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::Pt => "pt",
        }
    }

    pub fn toggle(self) -> Self {
        match self {
            Lang::En => Lang::Pt,
            Lang::Pt => Lang::En,
        }
    }
}

struct I18nTables {
    en: HashMap<String, String>,
    pt: HashMap<String, String>,
    pt_replacements: Vec<(String, String)>,
}

static TABLES: LazyLock<I18nTables> = LazyLock::new(|| {
    let raw: serde_json::Value = serde_json::from_str(include_str!("../data/i18n.json"))
        .expect("ui/data/i18n.json must be valid");

    let parse_map = |key: &str| -> HashMap<String, String> {
        raw.get(key)
            .and_then(serde_json::Value::as_object)
            .map(|obj| {
                obj.iter()
                    .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string()))
                    .collect()
            })
            .unwrap_or_default()
    };

    let pt_replacements = raw
        .get("ptStatReplacements")
        .and_then(serde_json::Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|pair| {
                    let p = pair.as_array()?;
                    if p.len() >= 2 {
                        Some((p[0].as_str()?.to_string(), p[1].as_str()?.to_string()))
                    } else {
                        None
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    I18nTables {
        en: parse_map("en"),
        pt: parse_map("pt"),
        pt_replacements,
    }
});

pub fn t(lang: Lang, key: &str) -> String {
    let map = match lang {
        Lang::En => &TABLES.en,
        Lang::Pt => &TABLES.pt,
    };
    if let Some(val) = map.get(key) {
        return val.clone();
    }
    if let Some(val) = TABLES.en.get(key) {
        return val.clone();
    }
    key.to_string()
}

pub fn format_t(lang: Lang, key: &str, replacements: &[(&str, &str)]) -> String {
    let mut text = t(lang, key);
    for &(k, v) in replacements {
        let pattern = format!("{{{}}}", k);
        text = text.replace(&pattern, v);
    }
    text
}

pub fn stat_label(lang: Lang, english_label: &str) -> String {
    match lang {
        Lang::En => english_label.to_string(),
        Lang::Pt => {
            let mut result = english_label.to_string();
            for (from, to) in &TABLES.pt_replacements {
                result = result.replace(from, to);
            }
            result
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translates_known_keys_in_both_languages() {
        assert_eq!(t(Lang::En, "appTitle"), "Damage Calculator");
        assert_eq!(t(Lang::Pt, "appTitle"), "Calculadora de Dano");
    }

    #[test]
    fn falls_back_to_en_then_to_raw_key() {
        assert_eq!(t(Lang::Pt, "nonExistentKey"), "nonExistentKey");
    }

    #[test]
    fn formats_template_variables() {
        let text = format_t(Lang::En, "charactersFound", &[("count", "3"), ("extra", "")]);
        assert!(text.contains('3'));
    }

    #[test]
    fn translates_stat_labels_in_portuguese() {
        assert_eq!(stat_label(Lang::En, "Dexterity"), "Dexterity");
        assert_eq!(stat_label(Lang::Pt, "Dexterity"), "Destreza");
    }
}
