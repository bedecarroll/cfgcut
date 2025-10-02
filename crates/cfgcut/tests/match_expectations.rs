use std::path::PathBuf;

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

#[test]
fn ios_interface_block_matches_expected() {
    let expected =
        "interface GigabitEthernet1\n ip address dhcp\n negotiation auto\n no mop enabled\n";
    cfgcut_cmd()
        .args([
            "-m",
            "interface GigabitEthernet1|>>|",
            &fixture_path("cisco_ios/sample.conf"),
        ])
        .assert()
        .success()
        .stdout(predicate::str::diff(expected));
}

#[test]
fn ios_route_map_set_block_matches_expected() {
    let expected = "route-map next-hop-self permit 10\n set ip next-hop peer-address\n";
    cfgcut_cmd()
        .args([
            "-m",
            "route-map next-hop-self permit 10|>>|",
            &fixture_path("cisco_ios/route_map_set.conf"),
        ])
        .assert()
        .success()
        .stdout(predicate::str::diff(expected));
}

#[test]
fn eos_interface_block_matches_expected() {
    let expected = "interface Ethernet1\n description to-core\n switchport mode trunk\n";
    cfgcut_cmd()
        .args([
            "-m",
            "interface Ethernet1|>>|",
            &fixture_path("arista_eos/route_map_set.conf"),
        ])
        .assert()
        .success()
        .stdout(predicate::str::diff(expected));
}

#[test]
fn eos_route_map_block_matches_expected() {
    let expected = "route-map RM-EDGE permit 10\n set ip next-hop peer-address\n";
    cfgcut_cmd()
        .args([
            "-m",
            "route-map RM-EDGE permit 10|>>|",
            &fixture_path("arista_eos/route_map_set.conf"),
        ])
        .assert()
        .success()
        .stdout(predicate::str::diff(expected));
}

#[test]
fn nxos_interface_block_matches_expected() {
    let expected = "interface Ethernet1/1\n description server-link\n no shutdown\n switchport\n";
    cfgcut_cmd()
        .args([
            "-m",
            "interface Ethernet1/1|>>|",
            &fixture_path("cisco_nxos/sample.conf"),
        ])
        .assert()
        .success()
        .stdout(predicate::str::diff(expected));
}

#[test]
fn nxos_feature_line_matches_expected() {
    let expected = "feature interface-vlan\n";
    cfgcut_cmd()
        .args([
            "-m",
            "feature interface-vlan",
            &fixture_path("cisco_nxos/sample.conf"),
        ])
        .assert()
        .success()
        .stdout(predicate::str::diff(expected));
}

#[test]
fn junos_brace_subtree_matches_expected() {
    let expected = "interfaces {\n  ge-0/0/0 {\n    unit 0 {\n      family inet {\n        dhcp;\n      }\n    }\n  }\n}\n";
    cfgcut_cmd()
        .args([
            "-m",
            "interfaces||ge-0/0/0|>>|",
            &fixture_path("juniper_junos/sample.conf"),
        ])
        .assert()
        .success()
        .stdout(predicate::str::diff(expected));
}

#[test]
fn junos_set_subtree_matches_expected() {
    let expected = "set interfaces\nset interfaces ge-0/0/0\nset interfaces ge-0/0/0 unit 0\nset interfaces ge-0/0/0 unit 0 family inet\nset interfaces ge-0/0/0 unit 0 family inet address 10.0.0.1/24\nset interfaces ge-0/0/0 unit 0 description Uplink to core\n";
    cfgcut_cmd()
        .args([
            "-m",
            "interfaces||ge-0/0/0|>>|",
            &fixture_path("juniper_junos_set/sample.set"),
        ])
        .assert()
        .success()
        .stdout(predicate::str::diff(expected));
}
