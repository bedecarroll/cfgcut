//! Core library API for parsing and extracting configuration snippets.
//!
//! The library exposes [`run`] along with supporting types for configuring a
//! single invocation.

use std::borrow::Cow;
use std::collections::{BTreeSet, VecDeque};
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use glob::{PatternError, glob};
use regex::Regex;
use serde::Serialize;

mod anonymize;
mod dialect;
mod inline_match;

use self::dialect::{DialectKind, LineKind, ParsedConfig};
use anonymize::{Anonymizer, TokenCapture, collect_plain_tokens};
use inline_match::{InlineMatchParse, parse_inline_matches};

/// Errors that can be returned while executing the cfgcut pipeline.
#[derive(Debug)]
#[non_exhaustive]
pub enum CfgcutError {
    /// File-system interaction failed for the given path.
    Io {
        /// The path that triggered the failure.
        path: PathBuf,
        /// The underlying I/O error.
        source: io::Error,
    },
    /// Compilation of a CLI match expression failed.
    Pattern(String, regex::Error),
    /// Parsing inline match expressions embedded in a configuration file failed.
    InlineMatches {
        /// The source file containing the inline block.
        path: PathBuf,
        /// A human-readable error message.
        message: String,
    },
    /// A regular expression inside an inline block could not be compiled.
    InlinePattern {
        /// The source file containing the inline block.
        path: PathBuf,
        /// The invalid pattern fragment.
        pattern: String,
        /// The underlying regular-expression error.
        source: regex::Error,
    },
    /// No input paths were provided to the run request.
    NoInputPaths,
    /// A glob pattern could not be parsed.
    GlobPatternInvalid {
        /// The glob pattern supplied by the caller.
        pattern: String,
        /// The underlying glob parser error.
        source: PatternError,
    },
    /// A glob pattern was valid but matched no files.
    GlobPatternNoMatches {
        /// The glob pattern supplied by the caller.
        pattern: String,
    },
    /// No match expressions were provided for the given configuration file.
    NoPatternsProvided {
        /// The configuration file missing match expressions.
        path: PathBuf,
    },
    /// A match expression contained no actionable segments.
    EmptyPattern {
        /// The raw pattern text supplied by the caller.
        raw: String,
    },
    /// The requested token destination is not supported.
    UnsupportedTokenDestination,
    /// Serializing structured output failed.
    Serialization {
        /// The underlying serialization error.
        source: serde_json::Error,
    },
}

impl fmt::Display for CfgcutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, source } => {
                write!(f, "failed to read '{}': {}", path.display(), source)
            }
            Self::Pattern(pattern, err) => {
                write!(f, "invalid match pattern '{pattern}': {err}")
            }
            Self::InlineMatches { path, message } => {
                write!(
                    f,
                    "failed to parse inline matches in '{}': {message}",
                    path.display()
                )
            }
            Self::InlinePattern {
                path,
                pattern,
                source,
            } => {
                write!(
                    f,
                    "invalid inline match pattern '{}' in '{}': {}",
                    pattern,
                    path.display(),
                    source
                )
            }
            Self::NoInputPaths => f.write_str("no input paths provided"),
            Self::GlobPatternInvalid { pattern, source } => {
                write!(f, "invalid glob pattern '{pattern}': {source}")
            }
            Self::GlobPatternNoMatches { pattern } => {
                write!(f, "glob pattern '{pattern}' matched no files")
            }
            Self::NoPatternsProvided { path } => write!(
                f,
                "no match patterns provided via CLI or inline block in '{}'",
                path.display()
            ),
            Self::EmptyPattern { raw } => {
                write!(f, "match pattern must not be empty (input: '{raw}')")
            }
            Self::UnsupportedTokenDestination => f.write_str("unsupported token destination"),
            Self::Serialization { source } => {
                write!(f, "failed to serialize token record: {source}")
            }
        }
    }
}

impl std::error::Error for CfgcutError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::Pattern(_, err) => Some(err),
            Self::InlinePattern { source, .. } => Some(source),
            Self::GlobPatternInvalid { source, .. } => Some(source),
            Self::Serialization { source } => Some(source),
            Self::InlineMatches { .. }
            | Self::NoInputPaths
            | Self::GlobPatternNoMatches { .. }
            | Self::NoPatternsProvided { .. }
            | Self::EmptyPattern { .. }
            | Self::UnsupportedTokenDestination => None,
        }
    }
}

impl From<serde_json::Error> for CfgcutError {
    fn from(source: serde_json::Error) -> Self {
        Self::Serialization { source }
    }
}

/// Describes how a single cfgcut invocation should behave.
#[derive(Debug, Clone)]
pub struct RunRequest {
    matches: Vec<String>,
    comment_handling: CommentHandling,
    output_mode: OutputMode,
    anonymization: Anonymization,
    inputs: Vec<PathBuf>,
    token_output: Option<TokenDestination>,
}

/// Controls whether comments are included in the rendered output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommentHandling {
    /// Strip comment-only lines from the results.
    Exclude,
    /// Emit comment-only lines alongside matched commands.
    Include,
}

/// Determines how verbose standard output should be.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    /// Emit headings and matched configuration lines.
    Normal,
    /// Suppress stdout entirely.
    Quiet,
}

/// Whether anonymization of sensitive tokens is enabled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Anonymization {
    /// Emit raw configuration content.
    Disabled,
    /// Scrub sensitive tokens and optionally capture them for audit use.
    Enabled,
}

/// Defines where captured token data should be written.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum TokenDestination {
    /// Emit each token record as JSON to stdout.
    Stdout,
    /// Append token records as JSON lines to the provided file.
    File(PathBuf),
}

/// Enumerates the types of sensitive values that can be anonymized.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[non_exhaustive]
#[serde(rename_all = "snake_case")]
pub enum TokenKind {
    /// Username or account identifier.
    Username,
    /// A secret such as a password or shared key.
    Secret,
    /// An autonomous system number.
    Asn,
    /// An IPv4 address.
    Ip,
}

impl TokenKind {
    /// Returns the canonical string representation for a token kind.
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

/// Captures anonymized token information emitted during a run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TokenRecord {
    /// The dialect of configuration from which the token originated.
    pub dialect: DialectKind,
    /// The hierarchical path to the matched command.
    pub path: Vec<String>,
    /// The type of token that was captured.
    pub kind: TokenKind,
    /// The original text discovered in the configuration.
    pub original: String,
    /// The anonymized replacement, if anonymization was enabled.
    pub anonymized: Option<String>,
    /// The line number in the source file.
    pub line: usize,
}

/// Describes the outcome of executing [`run`].
#[derive(Debug, Default, PartialEq, Eq)]
pub struct RunOutput {
    /// Whether any patterns matched across the provided inputs.
    pub matched: bool,
    /// The rendered configuration output destined for stdout.
    pub stdout: String,
    /// Any token records collected during anonymization.
    pub tokens: Vec<TokenRecord>,
    /// Warnings generated during processing, such as missing inline patterns.
    pub warnings: Vec<String>,
}

impl RunRequest {
    /// Create a builder used to construct a [`RunRequest`].
    #[must_use]
    pub fn builder() -> RunRequestBuilder {
        RunRequestBuilder::default()
    }

    /// The destination used to emit captured token data, if configured.
    #[must_use]
    pub fn token_output(&self) -> Option<&TokenDestination> {
        self.token_output.as_ref()
    }

    /// The CLI match expressions that should be applied to each input file.
    #[must_use]
    pub fn matches(&self) -> &[String] {
        &self.matches
    }

    /// The input paths gathered for this run.
    #[must_use]
    pub fn inputs(&self) -> &[PathBuf] {
        &self.inputs
    }

    /// Whether commands originating from comments should be included in output.
    #[must_use]
    pub fn comment_handling(&self) -> CommentHandling {
        self.comment_handling
    }

    /// The style of stdout emission for the current run.
    #[must_use]
    pub fn output_mode(&self) -> OutputMode {
        self.output_mode
    }

    /// Whether sensitive values should be anonymized while rendering output.
    #[must_use]
    pub fn anonymization(&self) -> Anonymization {
        self.anonymization
    }
}

/// Builder for [`RunRequest`].
#[derive(Debug, Clone)]
pub struct RunRequestBuilder {
    matches: Vec<String>,
    comment_handling: CommentHandling,
    output_mode: OutputMode,
    anonymization: Anonymization,
    inputs: Vec<PathBuf>,
    token_output: Option<TokenDestination>,
}

impl Default for RunRequestBuilder {
    fn default() -> Self {
        Self {
            matches: Vec::new(),
            comment_handling: CommentHandling::Exclude,
            output_mode: OutputMode::Normal,
            anonymization: Anonymization::Disabled,
            inputs: Vec::new(),
            token_output: None,
        }
    }
}

impl RunRequestBuilder {
    /// Replace any existing CLI match expressions.
    #[must_use]
    pub fn matches(mut self, matches: Vec<String>) -> Self {
        self.matches = matches;
        self
    }

    /// Configure how comments should be treated in the rendered output.
    #[must_use]
    pub fn comment_handling(mut self, handling: CommentHandling) -> Self {
        self.comment_handling = handling;
        self
    }

    /// Configure how verbose stdout should be during execution.
    #[must_use]
    pub fn output_mode(mut self, mode: OutputMode) -> Self {
        self.output_mode = mode;
        self
    }

    /// Enable or disable token anonymization in the rendered output.
    #[must_use]
    pub fn anonymization(mut self, anonymization: Anonymization) -> Self {
        self.anonymization = anonymization;
        self
    }

    /// Provide the inputs that cfgcut should scan.
    #[must_use]
    pub fn inputs(mut self, inputs: Vec<PathBuf>) -> Self {
        self.inputs = inputs;
        self
    }

    /// Configure where token data should be written.
    #[must_use]
    pub fn token_output(mut self, token_output: Option<TokenDestination>) -> Self {
        self.token_output = token_output;
        self
    }

    /// Finalize the builder and produce a [`RunRequest`].
    #[must_use]
    pub fn build(self) -> RunRequest {
        RunRequest {
            matches: self.matches,
            comment_handling: self.comment_handling,
            output_mode: self.output_mode,
            anonymization: self.anonymization,
            inputs: self.inputs,
            token_output: self.token_output,
        }
    }
}

fn compile_cli_patterns(matches: &[String]) -> Result<Option<Vec<Pattern>>, CfgcutError> {
    if matches.is_empty() {
        return Ok(None);
    }

    let mut compiled = Vec::with_capacity(matches.len());
    for raw in matches {
        compiled.push(Pattern::parse(raw)?);
    }
    Ok(Some(compiled))
}

struct ParsedFile {
    inline_matches: Option<Vec<String>>,
    parsed: ParsedConfig,
    dialect_kind: DialectKind,
}

fn parse_config_file(path: &Path) -> Result<ParsedFile, CfgcutError> {
    let raw_content = fs::read_to_string(path).map_err(|source| CfgcutError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let InlineMatchParse { matches, body } =
        parse_inline_matches(&raw_content).map_err(|err| CfgcutError::InlineMatches {
            path: path.to_path_buf(),
            message: err.to_string(),
        })?;
    let content = body.into_owned();
    let (dialect_kind, parsed) = dialect::parse_with_detect(&content);
    Ok(ParsedFile {
        inline_matches: matches,
        parsed,
        dialect_kind,
    })
}

fn resolve_patterns<'a>(
    cli_patterns: Option<&'a [Pattern]>,
    inline_strings: Option<&[String]>,
    path: &Path,
) -> Result<(Cow<'a, [Pattern]>, Option<String>), CfgcutError> {
    if let Some(patterns) = cli_patterns {
        let warning = if inline_strings.is_some() {
            Some(format!(
                "{}: ignoring inline matches because CLI patterns were provided",
                path.display()
            ))
        } else {
            None
        };
        return Ok((Cow::Borrowed(patterns), warning));
    }

    let inline_strings = inline_strings.ok_or_else(|| CfgcutError::NoPatternsProvided {
        path: path.to_path_buf(),
    })?;

    let compiled = compile_inline_patterns(path, inline_strings)?;
    Ok((Cow::Owned(compiled), None))
}

fn compile_inline_patterns(
    path: &Path,
    inline_strings: &[String],
) -> Result<Vec<Pattern>, CfgcutError> {
    let mut patterns = Vec::with_capacity(inline_strings.len());
    for raw in inline_strings {
        match Pattern::parse(raw) {
            Ok(pattern) => patterns.push(pattern),
            Err(CfgcutError::Pattern(pattern_str, source)) => {
                return Err(CfgcutError::InlinePattern {
                    path: path.to_path_buf(),
                    pattern: pattern_str,
                    source,
                });
            }
            Err(CfgcutError::EmptyPattern { raw }) => {
                return Err(CfgcutError::InlineMatches {
                    path: path.to_path_buf(),
                    message: if raw.trim().is_empty() {
                        "inline match pattern is empty".to_string()
                    } else {
                        format!("inline match pattern is empty: '{raw}'")
                    },
                });
            }
            Err(err) => return Err(err),
        }
    }
    Ok(patterns)
}

/// Execute `cfgcut` over the provided inputs.
///
/// # Errors
/// Returns an error when input files cannot be read, when patterns fail to
/// compile, when glob arguments are invalid, or when outputs cannot be
/// serialized.
pub fn run(request: &RunRequest) -> Result<RunOutput, CfgcutError> {
    let files = collect_files(&request.inputs)?;
    let cli_patterns = compile_cli_patterns(&request.matches)?;
    let include_comments = matches!(request.comment_handling, CommentHandling::Include);
    let anonymize = matches!(request.anonymization, Anonymization::Enabled);

    let mut output = String::new();
    let mut matched_any = false;
    let mut anonymizer = anonymize.then(Anonymizer::new);
    let mut tokens = Vec::new();
    let mut warnings = Vec::new();

    for path in files {
        let ParsedFile {
            inline_matches,
            parsed,
            dialect_kind,
        } = parse_config_file(&path)?;

        let mut token_accumulator = request
            .token_output
            .as_ref()
            .map(|_| TokenAccumulator::new(dialect_kind));

        let (pattern_set, warning) =
            resolve_patterns(cli_patterns.as_deref(), inline_matches.as_deref(), &path)?;
        if let Some(message) = warning {
            warnings.push(message);
        }

        let mut indices = BTreeSet::new();
        let mut matched_file = false;

        for pattern in pattern_set.iter() {
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
                let label = file_label(&path);
                output.push_str(comment_marker_for(dialect_kind));
                output.push_str(" cfgcut matches for ");
                output.push_str(&label);
                output.push('\n');
                output.push_str(&rendered);
                if !rendered.ends_with('\n') {
                    output.push('\n');
                }
            }
        } else {
            warnings.push(format!("warning: no matches found in {}", path.display()));
        }

        if let Some(accumulator) = token_accumulator {
            tokens.extend(accumulator.finish());
        }
    }

    Ok(RunOutput {
        matched: matched_any,
        stdout: output,
        tokens,
        warnings,
    })
}

fn collect_files(inputs: &[PathBuf]) -> Result<Vec<PathBuf>, CfgcutError> {
    if inputs.is_empty() {
        return Err(CfgcutError::NoInputPaths);
    }

    let mut files = Vec::new();
    for input in inputs {
        if let Some(pattern) = glob_pattern(input) {
            let mut matched_any = false;
            let paths = glob(&pattern).map_err(|err| CfgcutError::GlobPatternInvalid {
                pattern: pattern.clone(),
                source: err,
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
                return Err(CfgcutError::GlobPatternNoMatches {
                    pattern: pattern.clone(),
                });
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

const fn comment_marker_for(dialect: DialectKind) -> &'static str {
    match dialect {
        DialectKind::CiscoIos | DialectKind::CiscoNxos | DialectKind::AristaEos => "!",
        DialectKind::JuniperJunos => "##",
        DialectKind::JuniperJunosSet => "#",
    }
}

fn file_label(path: &Path) -> String {
    path.file_name().map_or_else(
        || path.display().to_string(),
        |name| name.to_string_lossy().into_owned(),
    )
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
            return Err(CfgcutError::EmptyPattern {
                raw: raw.to_string(),
            });
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
    #[expect(
        clippy::missing_const_for_fn,
        reason = "const constructors cannot accept runtime borrow parameters"
    )]
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
