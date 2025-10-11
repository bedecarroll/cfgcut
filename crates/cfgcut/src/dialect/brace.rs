use super::shared::{LineKind, ParsedConfig};

pub(super) fn detect(text: &str) -> bool {
    text.lines().any(|line| {
        let trimmed = line.trim();
        trimmed.contains('{') || trimmed.starts_with('}')
    })
}

pub(super) fn parse(text: &str) -> ParsedConfig {
    let mut parsed = ParsedConfig::default();
    let mut stack: Vec<usize> = Vec::new();

    for line in text.lines() {
        let mut trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        while trimmed.starts_with('}') {
            trimmed = trimmed[1..].trim_start();
            if let Some(closed_idx) = stack.pop() {
                let indent_spaces = stack.len() * 2;
                parsed.push_line(
                    format!("{:indent$}}}", "", indent = indent_spaces),
                    None,
                    LineKind::Closing,
                    Some(closed_idx),
                );
            }
        }

        if trimmed.is_empty() {
            continue;
        }

        let indent_spaces = stack.len() * 2;
        let rendered = format!("{:indent$}{}", "", trimmed, indent = indent_spaces);

        let kind = if trimmed.starts_with("##") {
            LineKind::Comment
        } else {
            LineKind::Command
        };

        let match_text = if matches!(kind, LineKind::Comment) {
            Some(trimmed.trim_start_matches('#').trim().to_string())
        } else if let Some(pos) = trimmed.find('{') {
            Some(trimmed[..pos].trim().to_string())
        } else {
            Some(trimmed.trim_end_matches(';').trim().to_string())
        };

        let parent = stack.last().copied();
        let idx = parsed.push_line(rendered, match_text, kind, parent);

        let open_braces = trimmed.matches('{').count();
        let close_braces = trimmed.matches('}').count();

        if open_braces > close_braces {
            stack.push(idx);
        }

        let mut diff = close_braces.saturating_sub(open_braces);
        while diff > 0 {
            if let Some(closed_idx) = stack.pop() {
                let indent_spaces = stack.len() * 2;
                parsed.push_line(
                    format!("{:indent$}}}", "", indent = indent_spaces),
                    None,
                    LineKind::Closing,
                    Some(closed_idx),
                );
            }
            diff -= 1;
        }
    }

    while let Some(closed_idx) = stack.pop() {
        let indent_spaces = stack.len() * 2;
        parsed.push_line(
            format!("{:indent$}}}", "", indent = indent_spaces),
            None,
            LineKind::Closing,
            Some(closed_idx),
        );
    }

    parsed
}
