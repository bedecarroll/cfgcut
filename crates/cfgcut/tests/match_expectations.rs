use std::path::{Path, PathBuf};

use assert_cmd::Command;
use predicates::prelude::*;

fn cfgcut_cmd() -> Command {
    let mut cmd = Command::cargo_bin("cfgcut").unwrap();
    cmd.current_dir(env!("CARGO_MANIFEST_DIR"));
    cmd
}

fn fixture_path(rel: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures")
        .join(rel)
        .to_string_lossy()
        .into_owned()
}

fn expected_with_header(marker: &str, path: &str, body: &str) -> String {
    let name = Path::new(path)
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string());
    let mut output = format!("{marker} cfgcut matches for {name}\n");
    output.push_str(body);
    output
}

#[test]
fn ios_interface_block_matches_expected() {
    let path = fixture_path("cisco_ios/sample.conf");
    let body = "interface GigabitEthernet1\n ip address dhcp\n negotiation auto\n no mop enabled\n";
    let expected = expected_with_header("!", &path, body);
    cfgcut_cmd()
        .args(["-m", "interface GigabitEthernet1|>>|", &path])
        .assert()
        .success()
        .stdout(predicate::str::diff(expected));
}

#[test]
fn ios_route_map_set_block_matches_expected() {
    let path = fixture_path("cisco_ios/route_map_set.conf");
    let body = "route-map next-hop-self permit 10\n set ip next-hop peer-address\n";
    let expected = expected_with_header("!", &path, body);
    cfgcut_cmd()
        .args(["-m", "route-map next-hop-self permit 10|>>|", &path])
        .assert()
        .success()
        .stdout(predicate::str::diff(expected));
}

#[test]
fn eos_interface_block_matches_expected() {
    let path = fixture_path("arista_eos/route_map_set.conf");
    let body = "interface Ethernet1\n description to-core\n switchport mode trunk\n";
    let expected = expected_with_header("!", &path, body);
    cfgcut_cmd()
        .args(["-m", "interface Ethernet1|>>|", &path])
        .assert()
        .success()
        .stdout(predicate::str::diff(expected));
}

#[test]
fn eos_route_map_block_matches_expected() {
    let path = fixture_path("arista_eos/route_map_set.conf");
    let body = "route-map RM-EDGE permit 10\n set ip next-hop peer-address\n";
    let expected = expected_with_header("!", &path, body);
    cfgcut_cmd()
        .args(["-m", "route-map RM-EDGE permit 10|>>|", &path])
        .assert()
        .success()
        .stdout(predicate::str::diff(expected));
}

#[test]
fn nxos_interface_block_matches_expected() {
    let path = fixture_path("cisco_nxos/sample.conf");
    let body = "interface Ethernet1/1\n description server-link\n no shutdown\n switchport\n";
    let expected = expected_with_header("!", &path, body);
    cfgcut_cmd()
        .args(["-m", "interface Ethernet1/1|>>|", &path])
        .assert()
        .success()
        .stdout(predicate::str::diff(expected));
}

#[test]
fn nxos_feature_line_matches_expected() {
    let path = fixture_path("cisco_nxos/sample.conf");
    let body = "feature interface-vlan\n";
    let expected = expected_with_header("!", &path, body);
    cfgcut_cmd()
        .args(["-m", "feature interface-vlan", &path])
        .assert()
        .success()
        .stdout(predicate::str::diff(expected));
}

#[test]
fn junos_brace_subtree_matches_expected() {
    let path = fixture_path("juniper_junos/sample.conf");
    let expected = "interfaces {\n  ge-0/0/0 {\n    unit 0 {\n      family inet {\n        dhcp;\n      }\n    }\n  }\n}\n";
    let expected = expected_with_header("##", &path, expected);
    cfgcut_cmd()
        .args(["-m", "interfaces||ge-0/0/0|>>|", &path])
        .assert()
        .success()
        .stdout(predicate::str::diff(expected));
}

#[test]
fn junos_set_subtree_matches_expected() {
    let path = fixture_path("juniper_junos_set/sample.set");
    let expected = "set interfaces\nset interfaces ge-0/0/0\nset interfaces ge-0/0/0 unit 0\nset interfaces ge-0/0/0 unit 0 family inet\nset interfaces ge-0/0/0 unit 0 family inet address 10.0.0.1/24\nset interfaces ge-0/0/0 unit 0 description Uplink to core\n";
    let expected = expected_with_header("#", &path, expected);
    cfgcut_cmd()
        .args(["-m", "interfaces||ge-0/0/0|>>|", &path])
        .assert()
        .success()
        .stdout(predicate::str::diff(expected));
}
