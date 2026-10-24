// Opt-in retrieval of whitelisted AION 2 combat guides. Only a topic name is sent to the guide site.
use std::time::Duration;

use regex::Regex;
use serde_json::{json, Value};
use std::sync::OnceLock;

const EXCERPT_LIMIT: usize = 3000;
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AION2-Calculator";

const GUIDES: [(&str, &str); 5] = [
    ("attack", "https://aion2maps.com/guides/stats-explained/"),
    ("pve", "https://aion2maps.com/guides/how-damage-works/"),
    ("pvp", "https://aion2maps.com/guides/how-damage-works/"),
    ("chance", "https://aion2maps.com/guides/accuracy-and-crit/"),
    ("creature", "https://aion2maps.com/guides/stats-explained/"),
];

pub struct GuideClient {
    http: reqwest::Client,
}

impl GuideClient {
    pub fn new() -> Result<Self, String> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(12))
            .user_agent(USER_AGENT)
            .build()
            .map_err(|error| error.to_string())?;
        Ok(Self { http })
    }

    pub async fn fetch(&self, topic: &str) -> Result<Value, String> {
        let url = GUIDES
            .iter()
            .find(|(name, _)| *name == topic)
            .map(|(_, url)| *url)
            .ok_or_else(|| "Unknown guide topic.".to_string())?;
        let response = self.http.get(url).send().await.map_err(|error| error.to_string())?;
        let html = response
            .error_for_status()
            .map_err(|error| error.to_string())?
            .text()
            .await
            .map_err(|error| error.to_string())?;
        Ok(json!({ "url": url, "excerpt": article_excerpt(&html)? }))
    }
}

// Extracts the <article> body as plain text, limited to EXCERPT_LIMIT characters.
pub fn article_excerpt(html: &str) -> Result<String, String> {
    let lower = html.to_ascii_lowercase();
    let start = lower.find("<article").ok_or("Guide article was not found.")?;
    let end = start + lower[start..].find("</article>").ok_or("Guide article is incomplete.")?;
    let article = &html[start..end];

    static BLOCKS: OnceLock<Regex> = OnceLock::new();
    static TAGS: OnceLock<Regex> = OnceLock::new();
    let blocks = BLOCKS.get_or_init(|| Regex::new(r"(?is)<script\b[^>]*>.*?</script>|<style\b[^>]*>.*?</style>").expect("valid regex"));
    let tags = TAGS.get_or_init(|| Regex::new(r"<[^>]+>").expect("valid regex"));

    let without_blocks = blocks.replace_all(article, " ");
    let text = html_escape::decode_html_entities(&tags.replace_all(&without_blocks, " ")).into_owned();
    let collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");
    Ok(collapsed.chars().take(EXCERPT_LIMIT).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_article_text_without_scripts_or_markup() {
        let html = "<html><head><script>var x=1;</script></head><body><nav>menu</nav>\
            <article><style>p{}</style><h1>Stats &amp; Damage</h1><p>Attack   <b>up</b></p></article><footer>end</footer></body></html>";
        assert_eq!(article_excerpt(html).unwrap(), "Stats & Damage Attack up");
    }

    #[test]
    fn reports_missing_or_incomplete_articles() {
        assert!(article_excerpt("<p>none</p>").is_err());
        assert!(article_excerpt("<article>open only").is_err());
    }

    #[test]
    fn excerpt_is_limited_by_characters_not_bytes() {
        let body = "가".repeat(EXCERPT_LIMIT + 10);
        let html = format!("<article>{body}</article>");
        assert_eq!(article_excerpt(&html).unwrap().chars().count(), EXCERPT_LIMIT);
    }
}
