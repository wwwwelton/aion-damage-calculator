use regex::Regex;
use std::sync::LazyLock;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MarkdownNode {
    Text(String),
    Element {
        tag: String,
        children: Vec<MarkdownNode>,
    },
}

impl MarkdownNode {
    pub fn text(s: impl Into<String>) -> Self {
        MarkdownNode::Text(s.into())
    }

    pub fn element(tag: impl Into<String>, children: Vec<MarkdownNode>) -> Self {
        MarkdownNode::Element {
            tag: tag.into(),
            children,
        }
    }
}

static INLINE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"`([^`]+)`|\*\*([^*]+)\*\*|\*([^*\s][^*]*)\*|\[([^\]]+)\]\(([^)\s]+)\)").unwrap()
});

static TABLE_SEP_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\|?\s*:?-+:?\s*(\|\s*:?-+:?\s*)*\|?$").unwrap()
});

static HEADING_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(#{1,6})\s+(.*)$").unwrap()
});

static HR_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(-{3,}|\*{3,}|_{3,})$").unwrap()
});

static BULLET_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[-*+]\s+(.*)$").unwrap()
});

static ORDERED_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\d+[.)]\s+(.*)$").unwrap()
});

pub fn parse_inline(text: &str) -> Vec<MarkdownNode> {
    let mut nodes = Vec::new();
    let mut last = 0;

    for cap in INLINE_RE.captures_iter(text) {
        let m = cap.get(0).unwrap();
        if m.start() > last {
            nodes.push(MarkdownNode::text(&text[last..m.start()]));
        }

        if let Some(c) = cap.get(1) {
            nodes.push(MarkdownNode::element("code", vec![MarkdownNode::text(c.as_str())]));
        } else if let Some(c) = cap.get(2) {
            nodes.push(MarkdownNode::element("strong", parse_inline(c.as_str())));
        } else if let Some(c) = cap.get(3) {
            nodes.push(MarkdownNode::element("em", parse_inline(c.as_str())));
        } else if let (Some(label), Some(url)) = (cap.get(4), cap.get(5)) {
            nodes.push(MarkdownNode::text(format!("{} ({})", label.as_str(), url.as_str())));
        }

        last = m.end();
    }

    if last < text.len() {
        nodes.push(MarkdownNode::text(&text[last..]));
    }

    nodes
}

fn split_row(line: &str) -> Vec<String> {
    let trimmed = line.trim();
    let without_leading = trimmed.strip_prefix('|').unwrap_or(trimmed);
    let without_trailing = without_leading.strip_suffix('|').unwrap_or(without_leading);
    without_trailing.split('|').map(|cell| cell.trim().to_string()).collect()
}

fn is_table_start(lines: &[&str], index: usize) -> bool {
    lines[index].contains('|')
        && index + 1 < lines.len()
        && TABLE_SEP_RE.is_match(lines[index + 1].trim())
}

fn starts_block(lines: &[&str], index: usize) -> bool {
    let trimmed = lines[index].trim();
    trimmed.starts_with("```")
        || HEADING_RE.is_match(trimmed)
        || HR_RE.is_match(trimmed)
        || BULLET_RE.is_match(trimmed)
        || ORDERED_RE.is_match(trimmed)
        || trimmed.starts_with('>')
        || is_table_start(lines, index)
}

fn parse_blocks(lines: &[&str]) -> Vec<MarkdownNode> {
    let mut blocks = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let trimmed = lines[i].trim();
        if trimmed.is_empty() {
            i += 1;
        } else if trimmed.starts_with("```") {
            let mut body = Vec::new();
            i += 1;
            while i < lines.len() && !lines[i].trim().starts_with("```") {
                body.push(lines[i]);
                i += 1;
            }
            if i < lines.len() {
                i += 1;
            }
            blocks.push(MarkdownNode::element(
                "pre",
                vec![MarkdownNode::element(
                    "code",
                    vec![MarkdownNode::text(body.join("\n"))],
                )],
            ));
        } else if let Some(caps) = HEADING_RE.captures(trimmed) {
            let hashes = caps.get(1).unwrap().as_str();
            let title = caps.get(2).unwrap().as_str();
            let level = (hashes.len() + 2).min(6);
            blocks.push(MarkdownNode::element(
                format!("h{}", level),
                parse_inline(title),
            ));
            i += 1;
        } else if HR_RE.is_match(trimmed) {
            blocks.push(MarkdownNode::element("hr", vec![]));
            i += 1;
        } else if is_table_start(lines, i) {
            let headers = split_row(lines[i]);
            let header_row = MarkdownNode::element(
                "tr",
                headers
                    .into_iter()
                    .map(|cell| MarkdownNode::element("th", parse_inline(&cell)))
                    .collect(),
            );
            let mut rows = vec![header_row];
            i += 2;
            while i < lines.len() && !lines[i].trim().is_empty() && lines[i].contains('|') {
                let cells = split_row(lines[i]);
                rows.push(MarkdownNode::element(
                    "tr",
                    cells
                        .into_iter()
                        .map(|cell| MarkdownNode::element("td", parse_inline(&cell)))
                        .collect(),
                ));
                i += 1;
            }
            blocks.push(MarkdownNode::element("table", rows));
        } else if BULLET_RE.is_match(trimmed) || ORDERED_RE.is_match(trimmed) {
            let is_bullet = BULLET_RE.is_match(trimmed);
            let mut items = Vec::new();
            while i < lines.len() {
                let line_trimmed = lines[i].trim();
                let matched = if is_bullet {
                    BULLET_RE.captures(line_trimmed)
                } else {
                    ORDERED_RE.captures(line_trimmed)
                };
                match matched {
                    Some(caps) => {
                        let content = caps.get(1).unwrap().as_str();
                        items.push(MarkdownNode::element("li", parse_inline(content)));
                        i += 1;
                    }
                    None => break,
                }
            }
            blocks.push(MarkdownNode::element(if is_bullet { "ul" } else { "ol" }, items));
        } else if trimmed.starts_with('>') {
            let mut quoted = Vec::new();
            while i < lines.len() && lines[i].trim().starts_with('>') {
                let line = lines[i].trim();
                let content = if let Some(stripped) = line.strip_prefix("> ") {
                    stripped
                } else if let Some(stripped) = line.strip_prefix('>') {
                    stripped
                } else {
                    line
                };
                quoted.push(content);
                i += 1;
            }
            blocks.push(MarkdownNode::element("blockquote", parse_blocks(&quoted)));
        } else {
            let mut paragraph = vec![trimmed];
            i += 1;
            while i < lines.len() && !lines[i].trim().is_empty() && !starts_block(lines, i) {
                paragraph.push(lines[i].trim());
                i += 1;
            }
            blocks.push(MarkdownNode::element("p", parse_inline(&paragraph.join(" "))));
        }
    }

    blocks
}

pub fn parse_markdown(source: &str) -> Vec<MarkdownNode> {
    let normalized = source.replace("\r\n", "\n").replace('\r', "\n");
    let lines: Vec<&str> = normalized.split('\n').collect();
    parse_blocks(&lines)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(v: &str) -> MarkdownNode {
        MarkdownNode::text(v)
    }

    fn el(tag: &str, children: Vec<MarkdownNode>) -> MarkdownNode {
        MarkdownNode::element(tag, children)
    }

    #[test]
    fn headings_are_shifted_below_panel_title_and_keep_inline_emphasis() {
        assert_eq!(
            parse_markdown("# Report\n## Summary\n### Detail"),
            vec![
                el("h3", vec![text("Report")]),
                el("h4", vec![text("Summary")]),
                el("h5", vec![text("Detail")]),
            ]
        );
    }

    #[test]
    fn consecutive_lines_form_one_paragraph_and_unmatched_asterisks_stay_literal() {
        assert_eq!(
            parse_markdown("Hit is **high**\nwith 2 * 3 and *ok*"),
            vec![el(
                "p",
                vec![
                    text("Hit is "),
                    el("strong", vec![text("high")]),
                    text(" with 2 * 3 and "),
                    el("em", vec![text("ok")]),
                ]
            )]
        );
    }

    #[test]
    fn pipe_tables_become_header_and_body_rows_and_every_row_keeps_its_cells() {
        let nodes = parse_markdown("| Atributo | Você |\n| --- | --- |\n| Dano | `100%` |");
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            MarkdownNode::Element { tag, children } => {
                assert_eq!(tag, "table");
                assert_eq!(children.len(), 2);
                let th_tags: Vec<&str> = match &children[0] {
                    MarkdownNode::Element { tag: _, children } => children
                        .iter()
                        .map(|c| match c {
                            MarkdownNode::Element { tag, .. } => tag.as_str(),
                            _ => "",
                        })
                        .collect(),
                    _ => vec![],
                };
                assert_eq!(th_tags, vec!["th", "th"]);
                assert_eq!(
                    children[1],
                    el(
                        "tr",
                        vec![
                            el("td", vec![text("Dano")]),
                            el("td", vec![el("code", vec![text("100%")])]),
                        ]
                    )
                );
            }
            _ => panic!("Expected element"),
        }
    }

    #[test]
    fn bullet_and_numbered_lists_group_consecutive_items() {
        assert_eq!(
            parse_markdown("- one\n- two\n1. first"),
            vec![
                el("ul", vec![el("li", vec![text("one")]), el("li", vec![text("two")])]),
                el("ol", vec![el("li", vec![text("first")])]),
            ]
        );
    }

    #[test]
    fn fenced_code_keeps_its_content_verbatim_without_inline_parsing() {
        assert_eq!(
            parse_markdown("```\n**not bold**\n```"),
            vec![el("pre", vec![el("code", vec![text("**not bold**")])])]
        );
    }

    #[test]
    fn model_text_containing_html_never_produces_elements() {
        let nodes = parse_markdown("<img src=x onerror=alert(1)> and <b>bold</b>");
        assert_eq!(
            nodes,
            vec![el("p", vec![text("<img src=x onerror=alert(1)> and <b>bold</b>")])]
        );
    }

    #[test]
    fn links_keep_their_label_and_visible_url_as_plain_text() {
        assert_eq!(
            parse_markdown("[guia](https://example.com)"),
            vec![el("p", vec![text("guia (https://example.com)")])]
        );
    }
}
