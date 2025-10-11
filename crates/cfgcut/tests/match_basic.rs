use std::path::{Path, PathBuf};

use assert_cmd::Command;
use predicates::prelude::*;

fn cfgcut_cmd() -> Command {
    let mut cmd = Command::cargo_bin("cfgcut").unwrap();
    cmd.current_dir(env!("CARGO_MANIFEST_DIR"));
    cmd
}

fn fixture_path(rel: &str) -> String {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    base.join("../../tests/fixtures")
        .join(rel)
        .to_string_lossy()
        .into_owned()
}

fn header(marker: &str, path: &str) -> String {
    let name = Path::new(path)
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string());
    format!("{marker} cfgcut matches for {name}")
}

#[test]
fn ios_full_interface_block() {
    let expected =
        "interface GigabitEthernet1\n ip address dhcp\n negotiation auto\n no mop enabled\n";
    let path = fixture_path("cisco_ios/sample.conf");
    let header_line = header("!", &path);

    let mut cmd = cfgcut_cmd();
    cmd.args(["-m", "interface GigabitEthernet1|>>|", &path])
        .assert()
        .success()
        .stdout(predicate::str::contains(&header_line))
        .stdout(predicate::str::contains(expected));
}

#[test]
fn ios_single_line_without_descend() {
    let path = fixture_path("cisco_ios/sample.conf");
    let header_line = header("!", &path);
    let expected_body = "interface GigabitEthernet1\n";
    let mut cmd = cfgcut_cmd();
    cmd.args(["-m", "interface GigabitEthernet1", &path])
        .assert()
        .success()
        .stdout(predicate::str::contains(&header_line))
        .stdout(predicate::str::diff(format!(
            "{header_line}\n{expected_body}"
        )));
}

#[test]
fn ios_descendant_match_includes_context() {
    let expected = "interface GigabitEthernet1\n ip address dhcp\n";
    let path = fixture_path("cisco_ios/sample.conf");
    let header_line = header("!", &path);

    let mut cmd = cfgcut_cmd();
    cmd.args(["-m", "interface GigabitEthernet1||ip address .*", &path])
        .assert()
        .success()
        .stdout(predicate::str::contains(&header_line))
        .stdout(predicate::str::contains(expected));
}

#[test]
fn nxos_interface_match_includes_children() {
    let path = fixture_path("cisco_nxos/sample.conf");
    let header_line = header("!", &path);
    let mut cmd = cfgcut_cmd();
    cmd.args(["-m", "interface Ethernet1/1|>>|", &path])
        .assert()
        .success()
        .stdout(predicate::str::contains(&header_line))
        .stdout(predicate::str::contains("interface Ethernet1/1"))
        .stdout(predicate::str::contains("no shutdown"));
}

#[test]
fn quiet_mode_produces_no_output() {
    let path = fixture_path("cisco_ios/sample.conf");
    let mut cmd = cfgcut_cmd();
    cmd.args(["-q", "-m", "interface GigabitEthernet1", &path])
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn no_match_sets_nonzero_exit() {
    let path = fixture_path("cisco_ios/sample.conf");
    let mut cmd = cfgcut_cmd();
    cmd.args(["-m", "interface Loopback0", &path])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("warning: no matches found in")
                .and(predicate::str::contains("sample.conf")),
        );
}

#[test]
fn ios_hostname_with_route_map_set() {
    let path = fixture_path("cisco_ios/route_map_set.conf");
    let header_line = header("!", &path);
    let mut cmd = cfgcut_cmd();
    cmd.args(["-m", "hostname .*", &path])
        .assert()
        .success()
        .stdout(predicate::str::contains(&header_line))
        .stdout(predicate::str::contains("hostname demo-ios-set"));
}

#[test]
fn eos_hostname_with_route_map_set() {
    let path = fixture_path("arista_eos/route_map_set.conf");
    let header_line = header("!", &path);
    let mut cmd = cfgcut_cmd();
    cmd.args(["-m", "hostname .*", &path])
        .assert()
        .success()
        .stdout(predicate::str::contains(&header_line))
        .stdout(predicate::str::contains("hostname demo-eos-set"));
}
