use std::borrow::Cow;
use std::fmt;

#[derive(Debug)]
pub struct InlineMatchParse<'a> {
    pub matches: Option<Vec<String>>,
    pub body: Cow<'a, str>,
}

#[derive(Debug)]
pub struct InlineMatchError {
    message: String,
}

impl InlineMatchError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for InlineMatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for InlineMatchError {}

pub fn parse_inline_matches(text: &str) -> Result<InlineMatchParse<'_>, InlineMatchError> {
    let index = first_non_ws(text);
    if index.is_none() {
        return Ok(InlineMatchParse {
            matches: None,
            body: Cow::Borrowed(text),
        });
    }
    let start = index.unwrap();
    if !text[start..].starts_with("{#") {
        return Ok(InlineMatchParse {
            matches: None,
            body: Cow::Borrowed(text),
        });
    }

    let end_offset = text[start + 2..]
        .find("#}")
        .ok_or_else(|| InlineMatchError::new("unterminated inline match block"))?;
    let end = start + 2 + end_offset;
    let block = &text[start + 2..end];
    let matches = parse_match_list(block)?;

    let mut body = String::with_capacity(text.len() - (end + 2 - start));
    body.push_str(&text[..start]);
    body.push_str(&text[end + 2..]);

    Ok(InlineMatchParse {
        matches: Some(matches),
        body: Cow::Owned(body),
    })
}

fn parse_match_list(block: &str) -> Result<Vec<String>, InlineMatchError> {
    let trimmed = block.trim();
    if trimmed.is_empty() {
        return Err(InlineMatchError::new("inline match block is empty"));
    }
    if !trimmed.starts_with('[') || !trimmed.ends_with(']') {
        return Err(InlineMatchError::new(
            "inline match block must contain a bracketed list",
        ));
    }

    let inner = &trimmed[1..trimmed.len() - 1];
    let mut chars = inner.chars().peekable();
    let mut matches = Vec::new();

    while let Some(&ch) = chars.peek() {
        if ch.is_whitespace() {
            chars.next();
            continue;
        }
        if ch == ',' {
            chars.next();
            continue;
        }
        if ch == '\'' || ch == '"' {
            chars.next();
            let quote = ch;
            let mut value = String::new();
            let mut closed = false;
            while let Some(next) = chars.next() {
                if next == '\\' {
                    if let Some(escaped) = chars.next() {
                        value.push(escaped);
                    } else {
                        return Err(InlineMatchError::new(
                            "unterminated escape sequence in inline match",
                        ));
                    }
                    continue;
                }
                if next == quote {
                    closed = true;
                    break;
                }
                value.push(next);
            }
            if !closed {
                return Err(InlineMatchError::new(
                    "unterminated string literal in inline match list",
                ));
            }
            matches.push(value);
            continue;
        }
        return Err(InlineMatchError::new(format!(
            "unexpected character '{ch}' in inline match list"
        )));
    }

    if matches.is_empty() {
        return Err(InlineMatchError::new(
            "inline match list must contain at least one pattern",
        ));
    }

    Ok(matches)
}

fn first_non_ws(text: &str) -> Option<usize> {
    text.char_indices()
        .find(|&(_, ch)| !is_inline_ws(ch))
        .map(|(idx, _)| idx)
}

const fn is_inline_ws(ch: char) -> bool {
    ch.is_whitespace() || ch == '\u{feff}'
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(text: &str) -> InlineMatchParse<'_> {
        parse_inline_matches(text).expect("inline matches should parse")
    }

    #[test]
    fn detects_single_line_block() {
        let parsed = parse("{# ['foo'] #}\nrest");
        assert_eq!(parsed.matches.unwrap(), vec!["foo".to_string()]);
        assert_eq!(parsed.body, "\nrest");
    }

    #[test]
    fn trims_leading_whitespace() {
        let parsed = parse("\n \t{# ['foo'] #}\nbody");
        assert_eq!(parsed.matches.unwrap(), vec!["foo".to_string()]);
        assert_eq!(parsed.body, "\n \t\nbody");
    }

    #[test]
    fn parses_multiline_block_with_mixed_quotes() {
        let text = "{# [\n'alpha',\n\"beta|>>|\",\n] #}\nconfig";
        let parsed = parse(text);
        assert_eq!(
            parsed.matches.unwrap(),
            vec!["alpha".to_string(), "beta|>>|".to_string()]
        );
        assert_eq!(parsed.body, "\nconfig");
    }

    #[test]
    fn ignores_body_without_block() {
        let parsed = parse_inline_matches("hostname router\n").unwrap();
        assert!(parsed.matches.is_none());
        assert_eq!(parsed.body, "hostname router\n");
    }

    #[test]
    fn block_must_be_first_meaningful_line() {
        let text = "hostname router\n{# ['foo'] #}\n";
        let parsed = parse_inline_matches(text).unwrap();
        assert!(parsed.matches.is_none());
        assert_eq!(parsed.body, text);
    }

    #[test]
    fn errors_on_unterminated_block() {
        let err = parse_inline_matches("{# ['foo']\nhostname").unwrap_err();
        assert_eq!(err.to_string(), "unterminated inline match block");
    }

    #[test]
    fn errors_on_empty_list() {
        let err = parse_inline_matches("{# [ ] #}\n").unwrap_err();
        assert_eq!(
            err.to_string(),
            "inline match list must contain at least one pattern"
        );
    }

    #[test]
    fn errors_on_missing_brackets() {
        let err = parse_inline_matches("{# 'foo' #}\n").unwrap_err();
        assert_eq!(
            err.to_string(),
            "inline match block must contain a bracketed list"
        );
    }

    #[test]
    fn supports_escaped_quotes() {
        let parsed = parse("{# [\"in\\\"line\"] #}\nbody");
        assert_eq!(parsed.matches.unwrap(), vec!["in\"line".to_string()]);
    }
}
