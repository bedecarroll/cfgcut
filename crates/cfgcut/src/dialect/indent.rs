use super::shared::{
    LineKind, ParsedConfig, dialect_comment_prefix, extract_match_text, is_comment,
};

pub(super) fn detect(text: &str) -> bool {
    text.lines().any(|line| {
        line.trim_start().starts_with('!') || line.trim_start().starts_with("interface ")
    })
}

pub(super) fn parse(text: &str) -> ParsedConfig {
    let mut parsed = ParsedConfig::default();
    let mut stack: Vec<(usize, usize)> = Vec::new();
    let mut multiline: Option<(usize, String)> = None;

    for line in text.lines() {
        let trimmed_end = line.trim_end();
        let trimmed = trimmed_end.trim();

        if let Some((parent_idx, delimiter)) = multiline.take() {
            let is_closing = trimmed == delimiter;
            let kind = if is_closing {
                LineKind::Closing
            } else {
                LineKind::Command
            };
            let match_text = if is_closing {
                None
            } else {
                Some(trimmed.to_string())
            };
            parsed.push_line(trimmed_end.to_string(), match_text, kind, Some(parent_idx));
            if !is_closing {
                multiline = Some((parent_idx, delimiter));
            }
            continue;
        }

        if trimmed.is_empty() {
            continue;
        }
        let indent = trimmed_end
            .chars()
            .take_while(|c| c.is_whitespace())
            .count();

        while let Some(&(prev_indent, _)) = stack.last() {
            if indent <= prev_indent {
                stack.pop();
            } else {
                break;
            }
        }

        let parent = stack.last().map(|&(_, idx)| idx);
        let mut match_text = Some(extract_match_text(
            trimmed_end,
            dialect_comment_prefix(trimmed_end),
        ));
        let kind = if is_comment(trimmed_end) {
            LineKind::Comment
        } else {
            LineKind::Command
        };
        if matches!(kind, LineKind::Command)
            && let Some(text) = banner_match_text(trimmed)
        {
            match_text = Some(text);
        }

        let idx = parsed.push_line(trimmed_end.to_string(), match_text, kind, parent);
        stack.push((indent, idx));

        if let Some(delimiter) = banner_delimiter(trimmed) {
            multiline = Some((idx, delimiter.to_string()));
        }
    }

    parsed
}

fn banner_delimiter(line: &str) -> Option<&str> {
    let mut parts = line.split_whitespace();
    match parts.next() {
        Some(word) if word.eq_ignore_ascii_case("banner") => {}
        _ => return None,
    }
    parts.next()?;
    let candidate = parts.next()?;
    if candidate.eq_ignore_ascii_case("file") || parts.next().is_some() {
        return None;
    }
    Some(candidate)
}

fn banner_match_text(line: &str) -> Option<String> {
    let mut parts = line.split_whitespace();
    let command = parts.next()?;
    if !command.eq_ignore_ascii_case("banner") {
        return None;
    }
    let banner_kind = parts.next()?;
    Some(format!("{command} {banner_kind}"))
}
