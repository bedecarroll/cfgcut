#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineKind {
    Command,
    Comment,
    Closing,
}

#[derive(Debug, Clone)]
pub struct LineEntry {
    pub raw: String,
    pub match_text: Option<String>,
    pub kind: LineKind,
    pub parent: Option<usize>,
}

#[derive(Debug, Default, Clone)]
pub struct ParsedConfig {
    pub lines: Vec<LineEntry>,
    pub children: Vec<Vec<usize>>,
}

impl ParsedConfig {
    pub fn push_line(
        &mut self,
        raw: String,
        match_text: Option<String>,
        kind: LineKind,
        parent: Option<usize>,
    ) -> usize {
        let idx = self.lines.len();
        self.lines.push(LineEntry {
            raw,
            match_text,
            kind,
            parent,
        });
        self.children.push(Vec::new());
        if let Some(parent_idx) = parent
            && let Some(children) = self.children.get_mut(parent_idx)
        {
            children.push(idx);
        }
        idx
    }

    #[cfg_attr(not(any(test, feature = "fuzzing")), allow(dead_code))]
    pub fn from_text(text: &str) -> Self {
        let (_, parsed) = super::parse_with_detect(text);
        parsed
    }
}

pub fn extract_match_text(line: &str, comment_prefix: Option<&str>) -> String {
    if let Some(prefix) = comment_prefix {
        let trimmed = line.trim_start();
        return trimmed.trim_start_matches(prefix).trim().to_string();
    }
    line.trim().to_string()
}

pub fn dialect_comment_prefix(line: &str) -> Option<&'static str> {
    let trimmed = line.trim_start();
    if trimmed.starts_with('!') {
        Some("!")
    } else if trimmed.starts_with('#') {
        Some("#")
    } else {
        None
    }
}

pub fn is_comment(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with('!') || trimmed.starts_with('#')
}
