use std::collections::HashMap;

use super::shared::{LineKind, ParsedConfig};

const KEYWORDS_WITH_VALUE: &[&str] = &[
    "unit",
    "family",
    "address",
    "group",
    "neighbor",
    "policy",
    "term",
    "interface",
    "area",
    "level",
    "apply-groups",
    "host-name",
    "peer-as",
    "community",
    "description",
    "set",
    "match",
    "then",
    "from",
    "router-id",
];
const KEYWORDS_REST: &[&str] = &["description"]; // join rest of line

pub(super) fn detect(text: &str) -> bool {
    let mut saw_set = false;

    for raw in text.lines() {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with('#') {
            continue;
        }
        if trimmed.starts_with("set ") {
            saw_set = true;
            continue;
        }
        // Encountered a meaningful line that does not begin with `set`, so this cannot
        // be Junos set syntax.
        return false;
    }

    saw_set
}

pub(super) fn parse(text: &str) -> ParsedConfig {
    let mut parsed = ParsedConfig::default();
    let mut node_map: HashMap<Vec<String>, usize> = HashMap::new();

    for raw_line in text.lines() {
        let trimmed = raw_line.trim_end();
        if trimmed.trim().is_empty() {
            continue;
        }
        let trimmed_start = trimmed.trim_start();
        if trimmed_start.starts_with('#') {
            let idx = parsed.push_line(
                trimmed_start.to_string(),
                Some(trimmed_start.trim_start_matches('#').trim().to_string()),
                LineKind::Comment,
                None,
            );
            node_map.insert(vec![format!("comment:{idx}")], idx);
            continue;
        }
        if !trimmed_start.starts_with("set ") {
            continue;
        }
        let body = trimmed_start.trim_start_matches("set ");
        let segments = split_segments(body);
        if segments.is_empty() {
            continue;
        }

        let mut path = Vec::new();
        let mut parent = None;
        for (idx, segment) in segments.iter().enumerate() {
            path.push(segment.clone());
            if let Some(&existing) = node_map.get(&path) {
                parent = Some(existing);
                continue;
            }

            let raw = if idx + 1 == segments.len() {
                trimmed_start.to_string()
            } else {
                format!("set {}", path.join(" "))
            };
            let match_text = Some(segment.clone());
            let node_idx = parsed.push_line(raw, match_text, LineKind::Command, parent);
            node_map.insert(path.clone(), node_idx);
            parent = Some(node_idx);
        }
    }

    parsed
}

fn split_segments(body: &str) -> Vec<String> {
    let mut segments = Vec::new();
    let mut tokens = body.split_whitespace();
    while let Some(token) = tokens.next() {
        if KEYWORDS_REST.contains(&token) {
            let mut segment = token.to_string();
            let remainder = tokens.collect::<Vec<_>>();
            if !remainder.is_empty() {
                segment.push(' ');
                segment.push_str(&remainder.join(" "));
            }
            segments.push(segment);
            break;
        }

        if KEYWORDS_WITH_VALUE.contains(&token)
            && let Some(next) = tokens.next()
        {
            segments.push(format!("{token} {next}"));
            continue;
        }

        segments.push(token.to_string());
    }
    segments
}
