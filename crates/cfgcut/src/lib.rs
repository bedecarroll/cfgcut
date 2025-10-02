#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

use std::collections::{BTreeSet, VecDeque};
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use glob::glob;
use regex::Regex;
use serde::Serialize;

mod anonymize;
mod dialect;

use self::dialect::{DialectKind, LineKind, ParsedConfig};
use anonymize::{Anonymizer, TokenCapture, collect_plain_tokens};

#[derive(Debug)]
pub enum CfgcutError {
    Io { path: PathBuf, source: io::Error },
    InvalidArguments(String),
    Pattern(String, regex::Error),
}

impl fmt::Display for CfgcutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, source } => {
                write!(f, "failed to read '{}': {}", path.display(), source)
            }
            Self::InvalidArguments(msg) => f.write_str(msg),
            Self::Pattern(pattern, err) => {
                write!(f, "invalid match pattern '{pattern}': {err}")
            }
        }
    }
}

impl std::error::Error for CfgcutError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::Pattern(_, err) => Some(err),
            Self::InvalidArguments(_) => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RunRequest {
    pub matches: Vec<String>,
    pub comment_handling: CommentHandling,
    pub output_mode: OutputMode,
    pub anonymization: Anonymization,
    pub inputs: Vec<PathBuf>,
    pub token_output: Option<TokenDestination>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommentHandling {
    Exclude,
    Include,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    Normal,
    Quiet,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Anonymization {
    Disabled,
    Enabled,
}

#[derive(Debug, Clone)]
pub enum TokenDestination {
    Stdout,
    File(PathBuf),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TokenKind {
    Username,
    Secret,
    Asn,
    Ip,
}

impl TokenKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Username => "username",
            Self::Secret => "secret",
            Self::Asn => "asn",
            Self::Ip => "ip",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TokenRecord {
    pub dialect: DialectKind,
    pub path: Vec<String>,
    pub kind: TokenKind,
    pub original: String,
    pub anonymized: Option<String>,
    pub line: usize,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct RunOutput {
    pub matched: bool,
    pub stdout: String,
    pub tokens: Vec<TokenRecord>,
}

/// Execute `cfgcut` over the provided inputs.
///
/// # Errors
/// Returns an error when match patterns are invalid, input paths cannot be
/// resolved, or files cannot be read from disk.
pub fn run(request: &RunRequest) -> Result<RunOutput, CfgcutError> {
    if request.matches.is_empty() {
        return Err(CfgcutError::InvalidArguments(
            "at least one -m/--match pattern is required".to_string(),
        ));
    }

    let files = collect_files(&request.inputs)?;

    let patterns = request
        .matches
        .iter()
        .map(|raw| Pattern::parse(raw))
        .collect::<Result<Vec<_>, _>>()?;

    let mut output = String::new();
    let mut matched_any = false;
    let include_comments = matches!(request.comment_handling, CommentHandling::Include);
    let anonymize = matches!(request.anonymization, Anonymization::Enabled);

    let mut anonymizer = anonymize.then(Anonymizer::new);
    let mut tokens = Vec::new();

    for path in files {
        let content = fs::read_to_string(&path).map_err(|source| CfgcutError::Io {
            path: path.clone(),
            source,
        })?;
        let (dialect_kind, parsed) = dialect::parse_with_detect(&content);
        let mut token_accumulator = request
            .token_output
            .as_ref()
            .map(|_| TokenAccumulator::new(dialect_kind));

        let mut indices = BTreeSet::new();
        let mut matched_file = false;

        for pattern in &patterns {
            let mut accumulator = MatchAccumulator::new(&parsed);
            pattern.apply(&parsed, &mut accumulator);
            if accumulator.matched {
                matched_any = true;
                matched_file = true;
            }
            indices.extend(accumulator.indices);
        }

        if matched_file {
            let rendered = render_output(
                &parsed,
                &indices,
                include_comments,
                anonymizer.as_mut(),
                token_accumulator.as_mut(),
            );
            if !rendered.is_empty() {
                if !output.is_empty() && !output.ends_with('\n') {
                    output.push('\n');
                }
                output.push_str(&rendered);
                if !rendered.ends_with('\n') {
                    output.push('\n');
                }
            }
        }

        if let Some(accumulator) = token_accumulator {
            tokens.extend(accumulator.finish());
        }
    }

    Ok(RunOutput {
        matched: matched_any,
        stdout: output,
        tokens,
    })
}

fn collect_files(inputs: &[PathBuf]) -> Result<Vec<PathBuf>, CfgcutError> {
    if inputs.is_empty() {
        return Err(CfgcutError::InvalidArguments(
            "no input paths provided".to_string(),
        ));
    }

    let mut files = Vec::new();
    for input in inputs {
        if let Some(pattern) = glob_pattern(input) {
            let mut matched_any = false;
            let paths = glob(&pattern).map_err(|err| {
                CfgcutError::InvalidArguments(format!("invalid glob pattern '{pattern}': {err}"))
            })?;

            for entry in paths {
                matched_any = true;
                let path = match entry {
                    Ok(path) => path,
                    Err(err) => {
                        let error = err.error();
                        return Err(CfgcutError::Io {
                            path: err.path().to_path_buf(),
                            source: io::Error::new(error.kind(), error.to_string()),
                        });
                    }
                };

                if path.is_dir() {
                    gather_dir(&path, &mut files)?;
                } else if path.is_file() {
                    files.push(path);
                }
            }

            if !matched_any {
                return Err(CfgcutError::InvalidArguments(format!(
                    "glob pattern '{pattern}' matched no files"
                )));
            }

            continue;
        }

        if input.is_file() {
            files.push(input.clone());
        } else if input.is_dir() {
            gather_dir(input, &mut files)?;
        } else {
            return Err(CfgcutError::Io {
                path: input.clone(),
                source: io::Error::new(io::ErrorKind::NotFound, "input path not found"),
            });
        }
    }

    files.sort();
    files.dedup();
    Ok(files)
}

fn glob_pattern(path: &Path) -> Option<String> {
    let text = path.to_string_lossy();
    if text.contains('*') || text.contains('?') || text.contains('[') {
        Some(text.into_owned())
    } else {
        None
    }
}

fn gather_dir(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), CfgcutError> {
    let mut entries = std::fs::read_dir(dir)
        .map_err(|source| CfgcutError::Io {
            path: dir.to_path_buf(),
            source,
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|source| CfgcutError::Io {
            path: dir.to_path_buf(),
            source,
        })?;

    entries.sort_by_key(std::fs::DirEntry::path);

    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            gather_dir(&path, files)?;
        } else if path.is_file() {
            files.push(path);
        }
    }
    Ok(())
}

#[derive(Debug, Clone)]
struct Pattern {
    segments: Vec<PatternSegment>,
}

impl Pattern {
    fn parse(raw: &str) -> Result<Self, CfgcutError> {
        let mut segments = Vec::new();
        for base in raw.split("||") {
            let mut remainder = base;
            loop {
                if remainder.is_empty() {
                    break;
                }

                if let Some(pos) = remainder.find("|>>|") {
                    let before = &remainder[..pos];
                    if !before.trim().is_empty() {
                        segments.push(create_segment(raw, before, MatchTarget::Command)?);
                    }
                    segments.push(PatternSegment::DescendAll);
                    remainder = &remainder[pos + 4..];
                    continue;
                }

                if let Some(stripped) = remainder.strip_prefix("|#|") {
                    segments.push(create_segment(raw, stripped, MatchTarget::Comment)?);
                } else if !remainder.trim().is_empty() {
                    segments.push(create_segment(raw, remainder, MatchTarget::Command)?);
                }
                break;
            }
        }

        if segments.is_empty() {
            return Err(CfgcutError::InvalidArguments(
                "match patterns must not be empty".to_string(),
            ));
        }

        Ok(Self { segments })
    }

    fn apply(&self, config: &ParsedConfig, accumulator: &mut MatchAccumulator) {
        if self.segments.is_empty() {
            return;
        }

        let roots: Vec<usize> = config
            .lines
            .iter()
            .enumerate()
            .filter_map(|(idx, line)| {
                if line.parent.is_none() {
                    Some(idx)
                } else {
                    None
                }
            })
            .collect();

        for root in roots {
            self.walk(config, root, 0, accumulator);
        }
    }

    fn walk(
        &self,
        config: &ParsedConfig,
        node_idx: usize,
        segment_idx: usize,
        accumulator: &mut MatchAccumulator,
    ) {
        if segment_idx >= self.segments.len() {
            return;
        }

        match &self.segments[segment_idx] {
            PatternSegment::DescendAll => {
                accumulator.record_full(node_idx);
                accumulator.matched = true;
            }
            PatternSegment::Match { regex, target } => {
                let line = &config.lines[node_idx];
                if !target.matches(line.kind) {
                    return;
                }
                if let Some(candidate) = line.match_text.as_deref() {
                    if !regex.is_match(candidate) {
                        return;
                    }
                } else {
                    return;
                }

                if segment_idx + 1 == self.segments.len() {
                    accumulator.record_match(node_idx);
                } else if matches!(self.segments[segment_idx + 1], PatternSegment::DescendAll) {
                    self.walk(config, node_idx, segment_idx + 1, accumulator);
                } else {
                    for &child in &config.children[node_idx] {
                        self.walk(config, child, segment_idx + 1, accumulator);
                    }
                }
            }
        }
    }
}

fn create_segment(
    raw: &str,
    pattern: &str,
    target: MatchTarget,
) -> Result<PatternSegment, CfgcutError> {
    let regex = compile_pattern(raw, pattern)?;
    Ok(PatternSegment::Match { regex, target })
}

fn compile_pattern(raw: &str, fragment: &str) -> Result<Regex, CfgcutError> {
    let mut pattern = fragment.trim().to_string();
    let anchored_start = pattern.starts_with('^');
    let anchored_end = pattern.ends_with('$');

    if !anchored_start {
        pattern = format!("^(?:{pattern})");
    }
    if !anchored_end {
        pattern.push('$');
    }

    Regex::new(&pattern).map_err(|err| CfgcutError::Pattern(raw.to_string(), err))
}

#[derive(Debug, Clone)]
enum PatternSegment {
    Match { regex: Regex, target: MatchTarget },
    DescendAll,
}

#[derive(Debug, Clone, Copy)]
enum MatchTarget {
    Command,
    Comment,
}

impl MatchTarget {
    const fn matches(self, kind: LineKind) -> bool {
        matches!(
            (self, kind),
            (Self::Command, LineKind::Command | LineKind::Closing)
                | (Self::Comment, LineKind::Comment)
        )
    }
}

struct MatchAccumulator<'a> {
    config: &'a ParsedConfig,
    pub matched: bool,
    pub indices: BTreeSet<usize>,
}

impl<'a> MatchAccumulator<'a> {
    #[allow(clippy::missing_const_for_fn)]
    fn new(config: &'a ParsedConfig) -> Self {
        Self {
            config,
            matched: false,
            indices: BTreeSet::new(),
        }
    }

    fn record_full(&mut self, node_idx: usize) {
        self.add_ancestors(node_idx);
        self.add_subtree(node_idx);
        self.matched = true;
    }

    fn record_match(&mut self, node_idx: usize) {
        self.add_ancestors(node_idx);
        self.indices.insert(node_idx);
        self.add_node_closing(node_idx);
        self.matched = true;
    }

    fn add_ancestors(&mut self, mut idx: usize) {
        while let Some(parent_idx) = self.config.lines[idx].parent {
            self.indices.insert(parent_idx);
            if let Some(children) = self.config.children.get(parent_idx) {
                for &child in children {
                    if matches!(self.config.lines[child].kind, LineKind::Closing) {
                        self.indices.insert(child);
                    }
                }
            }
            idx = parent_idx;
        }
    }

    fn add_subtree(&mut self, root_idx: usize) {
        let mut queue = VecDeque::from([root_idx]);
        while let Some(idx) = queue.pop_front() {
            self.indices.insert(idx);
            for &child in &self.config.children[idx] {
                queue.push_back(child);
            }
        }
    }

    fn add_node_closing(&mut self, idx: usize) {
        if let Some(children) = self.config.children.get(idx) {
            for &child in children {
                if matches!(self.config.lines[child].kind, LineKind::Closing) {
                    self.indices.insert(child);
                }
            }
        }
    }
}

#[cfg(feature = "fuzzing")]
pub fn fuzz_parse(text: &str) {
    let _ = ParsedConfig::from_text(text);
}

#[cfg(feature = "fuzzing")]
pub fn fuzz_matcher(pattern: &str, text: &str) {
    if let Ok(pattern) = Pattern::parse(pattern) {
        let parsed = ParsedConfig::from_text(text);
        let mut accumulator = MatchAccumulator::new(&parsed);
        pattern.apply(&parsed, &mut accumulator);
    }
}

struct TokenAccumulator {
    dialect: DialectKind,
    entries: Vec<TokenRecord>,
}

impl TokenAccumulator {
    const fn new(dialect: DialectKind) -> Self {
        Self {
            dialect,
            entries: Vec::new(),
        }
    }

    fn record(&mut self, config: &ParsedConfig, idx: usize, captures: &[TokenCapture]) {
        if captures.is_empty() {
            return;
        }
        let path = line_path(config, idx);
        let line_no = idx + 1;
        for capture in captures {
            self.entries.push(TokenRecord {
                dialect: self.dialect,
                path: path.clone(),
                kind: capture.kind,
                original: capture.original.clone(),
                anonymized: capture.anonymized.clone(),
                line: line_no,
            });
        }
    }

    fn finish(self) -> Vec<TokenRecord> {
        self.entries
    }
}

fn line_path(config: &ParsedConfig, idx: usize) -> Vec<String> {
    let mut path = Vec::new();
    let mut current = Some(idx);
    while let Some(i) = current {
        if let Some(text) = &config.lines[i].match_text {
            path.push(text.clone());
        }
        current = config.lines[i].parent;
    }
    path.reverse();
    path
}

fn render_output(
    config: &ParsedConfig,
    indices: &BTreeSet<usize>,
    with_comments: bool,
    mut anonymizer: Option<&mut Anonymizer>,
    mut tokens: Option<&mut TokenAccumulator>,
) -> String {
    let mut buf = String::new();
    for &idx in indices {
        let line = &config.lines[idx];
        if matches!(line.kind, LineKind::Comment) && !with_comments {
            continue;
        }

        let mut captures = Vec::new();
        let text = match (anonymizer.as_mut(), tokens.as_ref(), line.kind) {
            (Some(tool), Some(_), LineKind::Command) => {
                tool.scrub_with_tokens(&line.raw, &mut captures)
            }
            (Some(tool), _, _) => tool.scrub(&line.raw),
            (None, Some(_), LineKind::Command) => {
                captures = collect_plain_tokens(&line.raw);
                line.raw.clone()
            }
            (None, _, _) => line.raw.clone(),
        };

        if let Some(tokens) = tokens.as_deref_mut()
            && !captures.is_empty()
        {
            tokens.record(config, idx, &captures);
        }

        buf.push_str(&text);
        buf.push('\n');
    }
    buf
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dialect::{self, DialectKind};

    #[test]
    fn detect_brace_dialect() {
        let text = "system {\n    services {\n        ssh;\n    }\n}";
        let (kind, _) = dialect::parse_with_detect(text);
        assert!(matches!(kind, DialectKind::JuniperJunos));
    }

    #[test]
    fn detect_indent_dialect() {
        let text = "interface GigabitEthernet1\n ip address dhcp";
        let (kind, _) = dialect::parse_with_detect(text);
        assert!(matches!(
            kind,
            DialectKind::CiscoIos | DialectKind::AristaEos
        ));
    }

    #[test]
    fn comment_pattern_matches() {
        let text = "## Last changed: today\nsystem {\n}\n";
        let config = ParsedConfig::from_text(text);
        let pattern = Pattern::parse("|#|Last changed: .*").unwrap();
        let mut accumulator = MatchAccumulator::new(&config);
        pattern.apply(&config, &mut accumulator);
        assert!(accumulator.matched);
    }
}
