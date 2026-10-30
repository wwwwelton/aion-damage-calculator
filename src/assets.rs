// Web UI and data assets embedded in the executable.
// Only the files the page loads are embedded; tests and tooling stay out of the binary.
use std::borrow::Cow;

use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "ui/dist/"]
#[include = "index.html"]
#[include = "style.css"]
#[include = "aion2_ui.js"]
#[include = "aion2_ui_bg.wasm"]
struct Embedded;

#[derive(RustEmbed)]
#[folder = "."]
#[include = "assets/classes/*.webp"]
#[include = "assets/genus/*.webp"]
struct Images;

pub struct Asset {
    pub data: Cow<'static, [u8]>,
    pub content_type: String,
    pub cache_control: &'static str,
}

// Returns the embedded file for a request path such as "/index.html" or "/assets/classes/cleric.webp".
pub fn lookup(request_path: &str) -> Option<Asset> {
    let name = match request_path.trim_start_matches('/') {
        "" => "index.html",
        name => name,
    };
    if name.split('/').any(|part| part == ".." || part.is_empty()) {
        return None;
    }
    let file = Embedded::get(name).or_else(|| Images::get(name))?;
    let mime = mime_guess::from_path(name).first_or_octet_stream();
    let content_type = if mime.type_() == mime_guess::mime::TEXT {
        format!("{mime}; charset=utf-8")
    } else {
        mime.to_string()
    };
    let cache_control = if name.starts_with("assets/classes/") && name.ends_with(".webp") {
        "public, max-age=31536000, immutable"
    } else {
        "no-store"
    };
    Some(Asset { data: file.data, content_type, cache_control })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serves_the_home_page_and_scripts_with_text_types() {
        let page = lookup("/").expect("index.html is embedded");
        assert!(page.content_type.starts_with("text/html"));
        assert_eq!(page.cache_control, "no-store");
        assert!(lookup("/aion2_ui.js").expect("Wasm loader is embedded").content_type.starts_with("text/javascript"));
        let wasm = lookup("/aion2_ui_bg.wasm").expect("Wasm binary is embedded");
        assert_eq!(wasm.content_type, "application/wasm");
        assert!(wasm.data.starts_with(b"\0asm"));
    }

    #[test]
    fn rejects_paths_outside_the_embedded_set() {
        assert!(lookup("/../Cargo.toml").is_none());
        assert!(lookup("/Cargo.toml").is_none());
        assert!(lookup("/characters.test.js").is_none());
    }

    #[test]
    fn serves_genus_symbols_as_webp_assets() {
        for genus in ["cogni", "fera", "natura", "varian", "special"] {
            let path = format!("/assets/genus/{genus}.webp");
            let asset = lookup(&path).unwrap_or_else(|| panic!("{path} is not embedded"));
            assert_eq!(asset.content_type, "image/webp");
            assert!(asset.data.starts_with(b"RIFF"));
            assert_eq!(asset.data.get(8..12), Some(b"WEBP".as_slice()));
        }
    }

}
