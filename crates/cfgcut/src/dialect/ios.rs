use super::indent;
use super::shared::ParsedConfig;

pub(super) fn detect(text: &str) -> bool {
    indent::detect(text)
}

pub(super) fn parse(text: &str) -> ParsedConfig {
    indent::parse(text)
}
