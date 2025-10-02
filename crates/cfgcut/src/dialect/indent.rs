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

    for line in text.lines() {
        let trimmed_end = line.trim_end();
        if trimmed_end.trim().is_empty() {
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
        let match_text = Some(extract_match_text(
            trimmed_end,
            dialect_comment_prefix(trimmed_end),
        ));
        let kind = if is_comment(trimmed_end) {
            LineKind::Comment
        } else {
            LineKind::Command
        };

        let idx = parsed.push_line(trimmed_end.to_string(), match_text, kind, parent);
        stack.push((indent, idx));
    }

    parsed
}
