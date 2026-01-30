use std::path::{Path, PathBuf};

use assert_cmd::Command;
use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

fn cfgcut_cmd() -> Command {
    let mut cmd = cargo_bin_cmd!("cfgcut");
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
    let name = Path::new(path).file_name().map_or_else(
        || path.to_string(),
        |name| name.to_string_lossy().into_owned(),
    );
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
fn ios_banner_wildcard_captures_all_types() {
    let expected_classic = "\
banner login ^C
#####
Authorized access only
Disconnect now if you are not permitted.
#####
^C
";
    let expected_dollar = "\
banner login $
dollar banner
$
";
    let expected_console = "\
banner console ^C
This
  has
   indents
^C
";
    let expected_file = "banner login file flash:/motd.txt\n";
    let path = fixture_path("cisco_ios/banner.conf");
    let header_line = header("!", &path);

    let mut cmd = cfgcut_cmd();
    cmd.args(["-m", "banner .*|>>|", &path])
        .assert()
        .success()
        .stdout(predicate::str::contains(&header_line))
        .stdout(predicate::str::contains(expected_classic))
        .stdout(predicate::str::contains(expected_dollar))
        .stdout(predicate::str::contains(expected_console))
        .stdout(predicate::str::contains(expected_file))
        .stdout(predicate::str::contains("line vty 0 4").not());
}

#[test]
fn ios_banner_console_preserves_indents() {
    let expected = "\
banner console ^C
This
  has
   indents
^C
";
    let path = fixture_path("cisco_ios/banner.conf");
    let header_line = header("!", &path);

    let mut cmd = cfgcut_cmd();
    cmd.args(["-m", "banner console|>>|", &path])
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
fn nxos_banner_wildcard_matches_all() {
    let expected = "\
banner motd EOF
Welcome to NX-OS
Authorized access only.
EOF
";
    let expected_file = "banner motd file bootflash:nx-motd.txt\n";
    let path = fixture_path("cisco_nxos/sample.conf");
    let header_line = header("!", &path);

    let mut cmd = cfgcut_cmd();
    cmd.args(["-m", "banner .*|>>|", &path])
        .assert()
        .success()
        .stdout(predicate::str::contains(&header_line))
        .stdout(predicate::str::contains(expected))
        .stdout(predicate::str::contains(expected_file))
        .stdout(predicate::str::contains("interface Vlan10").not());
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
