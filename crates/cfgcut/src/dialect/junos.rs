use super::brace;
use super::shared::ParsedConfig;

pub(super) fn detect(text: &str) -> bool {
    brace::detect(text)
}

pub(super) fn parse(text: &str) -> ParsedConfig {
    brace::parse(text)
}
