use super::indent;
use super::shared::ParsedConfig;

pub(super) fn detect(text: &str) -> bool {
    text.lines().any(|line| {
        let trimmed = line.trim();
        trimmed.contains("cEOS") || trimmed.contains("vEOS") || trimmed.contains("Arista")
    })
}

pub(super) fn parse(text: &str) -> ParsedConfig {
    indent::parse(text)
}
