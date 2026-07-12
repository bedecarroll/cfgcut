use super::indent;
use super::shared::ParsedConfig;

pub(super) fn detect(text: &str) -> bool {
    text.lines()
        .map(str::trim_start)
        .any(|line| line.starts_with("feature ") || line.starts_with("hardware profile"))
}

pub(super) fn parse(text: &str) -> ParsedConfig {
    indent::parse(text)
}
