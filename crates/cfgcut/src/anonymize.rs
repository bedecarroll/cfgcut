use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::sync::LazyLock;

use regex::{Captures, Regex};

use crate::TokenKind;

static USERNAME_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(\buser(?:name)?\s+)([A-Za-z0-9._-]+)").expect("valid username regex")
});

static PASSWORD_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(\b(?:password|secret)\s+)(?:(\d+)\s+)?(\S+)").expect("valid password regex")
});

static KEYSTRING_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)(encrypted-password\s+")([^"]+)(")"#).expect("valid encrypted password regex")
});

static SSH_KEY_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"(?i)(ssh-rsa\s+")([^"]+)(")"#).expect("valid ssh key regex"));

static ASN_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(\b(?:asn|as-number|autonomous-system|local-as|remote-as|peer-as)\s+)(\d+)")
        .expect("valid asn regex")
});

static ROUTER_BGP_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)(\brouter\s+bgp\s+)(\d+)").expect("valid router bgp regex"));

static IP_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?P<ip>\b(?:\d{1,3}\.){3}\d{1,3})(?:/(?P<prefix>\d{1,2}))?")
        .expect("valid ip regex")
});

#[derive(Debug, Clone)]
pub struct TokenCapture {
    pub kind: TokenKind,
    pub original: String,
    pub anonymized: Option<String>,
}

impl TokenCapture {
    #[expect(
        clippy::missing_const_for_fn,
        reason = "constructor requires heap allocation and cannot be const"
    )]
    pub fn new(kind: TokenKind, original: String, anonymized: Option<String>) -> Self {
        Self {
            kind,
            original,
            anonymized,
        }
    }
}

#[derive(Default)]
pub struct Anonymizer {
    username_map: HashMap<String, String>,
    password_map: HashMap<String, String>,
    asn_map: HashMap<String, String>,
    ip_map: HashMap<String, String>,
    username_counter: usize,
    password_counter: usize,
    asn_counter: u32,
    ip_counter: u32,
}

impl Anonymizer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn scrub(&mut self, input: &str) -> String {
        self.scrub_internal(input, None)
    }

    pub fn scrub_with_tokens(&mut self, input: &str, sink: &mut Vec<TokenCapture>) -> String {
        self.scrub_internal(input, Some(sink))
    }

    #[expect(
        clippy::too_many_lines,
        reason = "token scrubbing logic is intentionally centralized for performance"
    )]
    fn scrub_internal(&mut self, input: &str, mut sink: Option<&mut Vec<TokenCapture>>) -> String {
        let mut line = input.to_string();

        line = USERNAME_RE
            .replace_all(&line, |caps: &Captures| {
                let replacement = self.map_username(&caps[2]);
                if let Some(sink) = sink.as_deref_mut() {
                    sink.push(TokenCapture::new(
                        TokenKind::Username,
                        caps[2].to_string(),
                        Some(replacement.clone()),
                    ));
                }
                format!("{}{}", &caps[1], replacement)
            })
            .into_owned();

        line = PASSWORD_RE
            .replace_all(&line, |caps: &Captures| {
                let prefix = &caps[1];
                let algo_part = caps
                    .get(2)
                    .map(|m| format!("{} ", m.as_str()))
                    .unwrap_or_default();
                let original = caps.get(3).unwrap().as_str().to_string();
                let replacement = self.map_password(&original);
                if let Some(sink) = sink.as_deref_mut() {
                    sink.push(TokenCapture::new(
                        TokenKind::Secret,
                        original,
                        Some(replacement.clone()),
                    ));
                }
                format!("{prefix}{algo_part}{replacement}")
            })
            .into_owned();

        line = KEYSTRING_RE
            .replace_all(&line, |caps: &Captures| {
                let original = caps[2].to_string();
                let replacement = self.map_password(&original);
                if let Some(sink) = sink.as_deref_mut() {
                    sink.push(TokenCapture::new(
                        TokenKind::Secret,
                        original,
                        Some(replacement.clone()),
                    ));
                }
                format!("{}{}{}", &caps[1], replacement, &caps[3])
            })
            .into_owned();

        line = SSH_KEY_RE
            .replace_all(&line, |caps: &Captures| {
                let original = caps[2].to_string();
                let replacement = self.map_password(&original);
                if let Some(sink) = sink.as_deref_mut() {
                    sink.push(TokenCapture::new(
                        TokenKind::Secret,
                        original,
                        Some(replacement.clone()),
                    ));
                }
                format!("{}{}{}", &caps[1], replacement, &caps[3])
            })
            .into_owned();

        line = ASN_RE
            .replace_all(&line, |caps: &Captures| {
                let original = caps[2].to_string();
                let replacement = self.map_asn(&original);
                if let Some(sink) = sink.as_deref_mut() {
                    sink.push(TokenCapture::new(
                        TokenKind::Asn,
                        original,
                        Some(replacement.clone()),
                    ));
                }
                format!("{}{}", &caps[1], replacement)
            })
            .into_owned();

        line = ROUTER_BGP_RE
            .replace_all(&line, |caps: &Captures| {
                let original = caps[2].to_string();
                let replacement = self.map_asn(&original);
                if let Some(sink) = sink.as_deref_mut() {
                    sink.push(TokenCapture::new(
                        TokenKind::Asn,
                        original,
                        Some(replacement.clone()),
                    ));
                }
                format!("{}{}", &caps[1], replacement)
            })
            .into_owned();

        line = IP_RE
            .replace_all(&line, |caps: &Captures| {
                let ip = &caps["ip"];
                let prefix = caps
                    .name("prefix")
                    .map(|m| format!("/{}", m.as_str()))
                    .unwrap_or_default();
                let mut original = ip.to_string();
                original.push_str(&prefix);
                let replacement = self.map_ip(ip);
                if let Some(sink) = sink.as_deref_mut() {
                    sink.push(TokenCapture::new(
                        TokenKind::Ip,
                        original,
                        Some(replacement.clone()),
                    ));
                }
                format!("{replacement}{prefix}")
            })
            .into_owned();

        line
    }

    fn map_username(&mut self, original: &str) -> String {
        if original.is_empty() {
            return String::from("user0");
        }
        self.username_map
            .entry(original.to_string())
            .or_insert_with(|| {
                self.username_counter += 1;
                format!("user{}", self.username_counter)
            })
            .clone()
    }

    fn map_password(&mut self, original: &str) -> String {
        self.password_map
            .entry(original.to_string())
            .or_insert_with(|| {
                self.password_counter += 1;
                format!("scrambled{}", self.password_counter)
            })
            .clone()
    }

    fn map_asn(&mut self, original: &str) -> String {
        self.asn_map
            .entry(original.to_string())
            .or_insert_with(|| {
                self.asn_counter += 1;
                let base = 64512u32;
                format!("{}", base + self.asn_counter)
            })
            .clone()
    }

    fn map_ip(&mut self, original: &str) -> String {
        if let Some(mapped) = self.ip_map.get(original) {
            return mapped.clone();
        }

        if original.parse::<Ipv4Addr>().is_ok() {
            let (third, fourth) = self.next_ip_octets();
            let replacement = Ipv4Addr::new(203, 0, third, fourth).to_string();
            self.ip_map
                .insert(original.to_string(), replacement.clone());
            return replacement;
        }

        original.to_string()
    }

    #[expect(
        clippy::missing_const_for_fn,
        reason = "generator mutates internal counters on each call"
    )]
    fn next_ip_octets(&mut self) -> (u8, u8) {
        let idx = self.ip_counter;
        self.ip_counter = self.ip_counter.saturating_add(1);
        let third = 113 + ((idx / 254) % 10) as u8;
        let fourth = 1 + (idx % 254) as u8;
        (third, fourth)
    }
}

pub fn collect_plain_tokens(input: &str) -> Vec<TokenCapture> {
    let mut tokens = Vec::new();

    for caps in USERNAME_RE.captures_iter(input) {
        tokens.push(TokenCapture::new(
            TokenKind::Username,
            caps[2].to_string(),
            None,
        ));
    }

    for caps in PASSWORD_RE.captures_iter(input) {
        tokens.push(TokenCapture::new(
            TokenKind::Secret,
            caps.get(3).unwrap().as_str().to_string(),
            None,
        ));
    }

    for caps in KEYSTRING_RE.captures_iter(input) {
        tokens.push(TokenCapture::new(
            TokenKind::Secret,
            caps[2].to_string(),
            None,
        ));
    }

    for caps in SSH_KEY_RE.captures_iter(input) {
        tokens.push(TokenCapture::new(
            TokenKind::Secret,
            caps[2].to_string(),
            None,
        ));
    }

    for caps in ASN_RE.captures_iter(input) {
        tokens.push(TokenCapture::new(TokenKind::Asn, caps[2].to_string(), None));
    }

    for caps in ROUTER_BGP_RE.captures_iter(input) {
        tokens.push(TokenCapture::new(TokenKind::Asn, caps[2].to_string(), None));
    }

    for caps in IP_RE.captures_iter(input) {
        let mut original = caps["ip"].to_string();
        if let Some(prefix) = caps.name("prefix") {
            original.push('/');
            original.push_str(prefix.as_str());
        }
        tokens.push(TokenCapture::new(TokenKind::Ip, original, None));
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::{Anonymizer, collect_plain_tokens};
    use crate::TokenKind;

    #[test]
    fn anonymizer_scrubs_ip_and_asn() {
        let mut anon = Anonymizer::new();
        let line = "neighbor 192.0.2.1 remote-as 65001;";
        let scrubbed = anon.scrub(line);
        assert!(!scrubbed.contains("192.0.2.1"));
        assert!(!scrubbed.contains("65001"));
        assert!(scrubbed.contains("neighbor"));
        assert!(scrubbed.contains("203.0.113."));
    }

    #[test]
    fn anonymizer_reuses_mappings() {
        let mut anon = Anonymizer::new();
        let first = anon.scrub("username admin password 0 secret");
        let second = anon.scrub("username admin password 0 secret");
        assert_eq!(first, second);

        let third = anon.scrub("neighbor 192.0.2.1 remote-as 65001");
        let fourth = anon.scrub("neighbor 192.0.2.1 remote-as 65001");
        assert_eq!(third, fourth);
    }

    #[test]
    fn collect_plain_tokens_extracts_values() {
        let tokens = collect_plain_tokens(
            "username admin password 0 secret123 neighbor 192.0.2.1 remote-as 65001",
        );
        assert!(
            tokens
                .iter()
                .any(|t| matches!(t.kind, TokenKind::Username) && t.original == "admin")
        );
        assert!(
            tokens
                .iter()
                .any(|t| matches!(t.kind, TokenKind::Secret) && t.original == "secret123")
        );
        assert!(
            tokens
                .iter()
                .any(|t| matches!(t.kind, TokenKind::Ip) && t.original == "192.0.2.1")
        );
        assert!(
            tokens
                .iter()
                .any(|t| matches!(t.kind, TokenKind::Asn) && t.original == "65001")
        );
    }
}
