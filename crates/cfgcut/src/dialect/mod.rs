use serde::Serialize;

mod brace;
mod eos;
mod indent;
mod ios;
mod junos;
mod junos_set;
mod nxos;
pub mod shared;

pub use shared::{LineKind, ParsedConfig};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum DialectKind {
    CiscoIos,
    CiscoNxos,
    AristaEos,
    JuniperJunos,
    JuniperJunosSet,
}

struct DialectDescriptor {
    kind: DialectKind,
    detect: fn(&str) -> bool,
    parse: fn(&str) -> ParsedConfig,
}

const DIALECTS: &[DialectDescriptor] = &[
    DialectDescriptor {
        kind: DialectKind::JuniperJunosSet,
        detect: junos_set::detect,
        parse: junos_set::parse,
    },
    DialectDescriptor {
        kind: DialectKind::JuniperJunos,
        detect: junos::detect,
        parse: junos::parse,
    },
    DialectDescriptor {
        kind: DialectKind::AristaEos,
        detect: eos::detect,
        parse: eos::parse,
    },
    DialectDescriptor {
        kind: DialectKind::CiscoNxos,
        detect: nxos::detect,
        parse: nxos::parse,
    },
    DialectDescriptor {
        kind: DialectKind::CiscoIos,
        detect: ios::detect,
        parse: ios::parse,
    },
];

pub fn parse_with_detect(text: &str) -> (DialectKind, ParsedConfig) {
    for descriptor in DIALECTS {
        if (descriptor.detect)(text) {
            return (descriptor.kind, (descriptor.parse)(text));
        }
    }
    (DialectKind::CiscoIos, ios::parse(text))
}

#[cfg(test)]
mod tests {
    use super::shared::LineKind;
    use super::*;

    #[test]
    fn ios_parse_assigns_parents_and_comments() {
        let text = "interface GigabitEthernet1\n description Uplink\n ! maintenance comment\n ip address dhcp\n";
        let parsed = ios::parse(text);
        assert_eq!(parsed.lines.len(), 4);
        assert!(matches!(parsed.lines[0].kind, LineKind::Command));
        assert_eq!(
            parsed.lines[0].match_text.as_deref(),
            Some("interface GigabitEthernet1")
        );
        assert_eq!(parsed.lines[1].parent, Some(0));
        assert_eq!(
            parsed.lines[1].match_text.as_deref(),
            Some("description Uplink")
        );
        assert!(matches!(parsed.lines[2].kind, LineKind::Comment));
        assert_eq!(
            parsed.lines[2].match_text.as_deref(),
            Some("maintenance comment")
        );
    }

    #[test]
    fn eos_detect_recognises_platform_markers() {
        let text = "! device: Arista cEOSLAB\ninterface Ethernet1\n";
        assert!(eos::detect(text));
        assert!(!eos::detect("interface Ethernet1\n"));
    }

    #[test]
    fn nxos_detects_feature_lines() {
        let text = "feature interface-vlan\ninterface Ethernet1/1\n";
        assert!(nxos::detect(text));
    }

    #[test]
    fn junos_set_detects_set_syntax() {
        let text = "set system host-name vsrx\nset interfaces ge-0/0/0 unit 0 family inet address 192.0.2.1/24\n";
        assert!(junos_set::detect(text));
    }

    #[test]
    fn junos_parse_emits_closing_nodes_for_blocks() {
        let text = "system {\n    services {\n        ssh;\n    }\n}\n";
        let parsed = junos::parse(text);
        let closing_count = parsed
            .lines
            .iter()
            .filter(|line| matches!(line.kind, LineKind::Closing))
            .count();
        assert!(closing_count >= 2);

        let services_idx = parsed
            .lines
            .iter()
            .position(|line| line.match_text.as_deref() == Some("services"))
            .expect("services block present");
        let closing_idx = parsed.children[services_idx]
            .iter()
            .copied()
            .find(|&idx| matches!(parsed.lines[idx].kind, LineKind::Closing))
            .expect("closing brace child present");
        assert_eq!(parsed.lines[closing_idx].raw.trim(), "}");
        assert_eq!(parsed.lines[closing_idx].parent, Some(services_idx));
    }
}
