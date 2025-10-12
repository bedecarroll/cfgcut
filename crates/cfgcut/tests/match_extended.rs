use std::fs;
use std::path::{Path, PathBuf};

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

fn cfgcut_cmd() -> Command {
    let mut cmd = Command::cargo_bin("cfgcut").unwrap();
    cmd.current_dir(env!("CARGO_MANIFEST_DIR"));
    cmd
}

fn fixture_path(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures")
        .join(rel)
}

fn fixture_str(rel: &str) -> String {
    fixture_path(rel).to_string_lossy().into_owned()
}

fn header(marker: &str, path: &Path) -> String {
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string());
    format!("{marker} cfgcut matches for {name}")
}

#[test]
fn junos_descend_all_returns_subtree() {
    let path = fixture_path("juniper_junos/sample.conf");
    let header_line = header("##", &path);
    let path_str = path.to_string_lossy().into_owned();
    let mut cmd = cfgcut_cmd();
    cmd.args(["-m", "system|>>|", path_str.as_str()])
        .assert()
        .success()
        .stdout(predicate::str::contains(&header_line))
        .stdout(predicate::str::contains("system {"))
        .stdout(predicate::str::contains("host-name vsrx;"))
        .stdout(predicate::str::contains("}"))
        .stdout(predicate::str::contains("version").not());
}

#[test]
fn junos_interfaces_descend_all_limits_scope() {
    let path = fixture_path("juniper_junos/sample.conf");
    let header_line = header("##", &path);
    let path_str = path.to_string_lossy().into_owned();
    let mut cmd = cfgcut_cmd();
    cmd.args(["-m", "interfaces|>>|", path_str.as_str()])
        .assert()
        .success()
        .stdout(predicate::str::contains(&header_line))
        .stdout(predicate::str::contains("interfaces {"))
        .stdout(predicate::str::contains("ge-0/0/0"))
        .stdout(predicate::str::contains("system {").not())
        .stdout(predicate::str::contains("version").not());
}

#[test]
fn junos_specific_interface_descend_all_is_anchored() {
    let path = fixture_path("juniper_junos/sample.conf");
    let header_line = header("##", &path);
    let path_str = path.to_string_lossy().into_owned();
    let mut cmd = cfgcut_cmd();
    cmd.args(["-m", "interfaces||ge-.*0|>>|", path_str.as_str()])
        .assert()
        .success()
        .stdout(predicate::str::contains(&header_line))
        .stdout(predicate::str::contains("interfaces {"))
        .stdout(predicate::str::contains("ge-0/0/0"))
        .stdout(predicate::str::contains("ge-0/0/1").not())
        .stdout(predicate::str::contains("system {").not());
}

#[test]
fn junos_unit_match_without_descend_returns_line_and_path() {
    let path = fixture_path("juniper_junos/sample.conf");
    let header_line = header("##", &path);
    let path_str = path.to_string_lossy().into_owned();
    let mut cmd = cfgcut_cmd();
    cmd.args(["-m", "interfaces||ge-.*||unit 0", path_str.as_str()])
        .assert()
        .success()
        .stdout(predicate::str::contains(&header_line))
        .stdout(predicate::str::contains("interfaces {"))
        .stdout(predicate::str::contains("ge-0/0/0"))
        .stdout(predicate::str::contains("ge-0/0/1"))
        .stdout(predicate::str::contains("family inet").not());
}

#[test]
fn with_comments_flag_includes_comment_lines() {
    let tmp = tempdir().unwrap();
    let path = tmp.path().join("sample.cfg");
    fs::write(
        &path,
        "## comment marker\nsystem {\n    host-name test;\n}\n",
    )
    .unwrap();
    let path_str = path.to_string_lossy().into_owned();

    let mut without_comments = cfgcut_cmd();
    without_comments
        .args(["-m", "|#|comment.*", path_str.as_str()])
        .assert()
        .success()
        .stdout(predicate::str::is_empty());

    let mut with_comments = cfgcut_cmd();
    with_comments
        .args(["-c", "-m", "|#|comment.*", path_str.as_str()])
        .assert()
        .success()
        .stdout(predicate::str::contains(header("##", &path)))
        .stdout(predicate::str::contains("## comment marker"));
}

#[test]
fn multiple_match_patterns_union_results() {
    let path = fixture_path("cisco_ios/sample.conf");
    let header_line = header("!", &path);
    let path_str = path.to_string_lossy().into_owned();
    let mut cmd = cfgcut_cmd();
    cmd.args([
        "-m",
        "interface GigabitEthernet1",
        "-m",
        "interface GigabitEthernet2",
        path_str.as_str(),
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains(&header_line))
    .stdout(predicate::str::contains("interface GigabitEthernet1"))
    .stdout(predicate::str::contains("interface GigabitEthernet2"));
}

#[test]
fn directory_input_collects_files() {
    let tmp = tempdir().unwrap();
    let dir = tmp.path();
    let src = fixture_path("cisco_ios/sample.conf");
    let dst = dir.join("config.cfg");
    fs::copy(&src, &dst).unwrap();
    let dir_str = dir.to_string_lossy().into_owned();
    let header_line = header("!", &dst);

    let mut cmd = cfgcut_cmd();
    cmd.args(["-m", "interface GigabitEthernet1", dir_str.as_str()])
        .assert()
        .success()
        .stdout(predicate::str::contains(&header_line))
        .stdout(predicate::str::contains("interface GigabitEthernet1"));
}

#[test]
fn glob_pattern_collects_files() {
    let tmp = tempdir().unwrap();
    let dir = tmp.path();
    let src_ios = fixture_path("cisco_ios/sample.conf");
    let src_eos = fixture_path("arista_eos/sample.conf");
    fs::copy(&src_ios, dir.join("ios.cfg")).unwrap();
    fs::copy(&src_eos, dir.join("eos.cfg")).unwrap();

    let pattern = format!("{}/*.cfg", dir.to_string_lossy());
    let ios_header = header("!", &dir.join("ios.cfg"));
    let eos_header = header("!", &dir.join("eos.cfg"));

    let mut cmd = cfgcut_cmd();
    cmd.args([
        "-m",
        "interface GigabitEthernet1",
        "-m",
        "interface Ethernet1",
        pattern.as_str(),
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains(&ios_header))
    .stdout(predicate::str::contains(&eos_header))
    .stdout(predicate::str::contains("interface GigabitEthernet1"))
    .stdout(predicate::str::contains("interface Ethernet1"));
}

#[test]
fn anonymize_scrambles_sensitive_tokens() {
    let tmp = tempdir().unwrap();
    let path = tmp.path().join("sensitive.cfg");
    fs::write(
        &path,
        "username admin password 0 adminpass\nrouter bgp 65000\n neighbor 192.0.2.1 remote-as 65001\n",
    )
    .unwrap();
    let path_str = path.to_string_lossy().into_owned();
    let header_line = header("!", &path);

    let mut cmd = cfgcut_cmd();
    cmd.args(["-a", "-m", ".*", path_str.as_str()])
        .assert()
        .success()
        .stdout(predicate::str::contains(&header_line))
        .stdout(predicate::str::contains("admin").not())
        .stdout(predicate::str::contains("adminpass").not())
        .stdout(predicate::str::contains("65000").not())
        .stdout(predicate::str::contains("65001").not())
        .stdout(predicate::str::contains("192.0.2.1").not())
        .stdout(predicate::str::contains("user1"))
        .stdout(predicate::str::contains("scrambled1"));
}

#[test]
fn tokens_flag_emits_json_lines() {
    let tmp = tempdir().unwrap();
    let path = tmp.path().join("tokens.cfg");
    fs::write(
        &path,
        "username admin password 0 adminpass\nrouter bgp 65000\n neighbor 192.0.2.1 remote-as 65001\n",
    )
    .unwrap();
    let path_str = path.to_string_lossy().into_owned();

    let mut cmd = cfgcut_cmd();
    cmd.args([
        "--tokens",
        "-m",
        "username .*",
        "-m",
        "router bgp .*|>>|",
        path_str.as_str(),
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains(header("!", &path)))
    .stdout(predicate::str::contains("\"kind\":\"username\""))
    .stdout(predicate::str::contains("\"original\":\"admin\""))
    .stdout(predicate::str::contains("\"kind\":\"asn\""))
    .stdout(predicate::str::contains("\"kind\":\"ip\""));
}

#[test]
fn tokens_out_writes_file() {
    let tmp = tempdir().unwrap();
    let input = tmp.path().join("tokens.cfg");
    fs::write(&input, "neighbor 192.0.2.1 remote-as 65001\n").unwrap();
    let output = tmp.path().join("tokens.json");

    let mut cmd = cfgcut_cmd();
    cmd.args([
        "--tokens-out",
        output.to_string_lossy().as_ref(),
        "-q",
        "-m",
        ".*",
        input.to_string_lossy().as_ref(),
    ])
    .assert()
    .success()
    .stdout(predicate::str::is_empty());

    let written = fs::read_to_string(&output).unwrap();
    assert!(written.contains("\"kind\":\"ip\""));
    assert!(written.contains("\"kind\":\"asn\""));
}

#[test]
fn junos_set_hierarchy_matches() {
    let path = fixture_path("juniper_junos_set/sample.set");
    let header_line = header("#", &path);
    let path_str = path.to_string_lossy().into_owned();
    let mut cmd = cfgcut_cmd();
    cmd.args([
        "-m",
        "interfaces||ge-0/0/0||unit 0||family inet||address .*",
        path_str.as_str(),
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains(&header_line))
    .stdout(predicate::str::contains(
        "set interfaces ge-0/0/0 unit 0 family inet address 10.0.0.1/24",
    ));
}

#[test]
fn junos_set_subtree_expansion() {
    let path = fixture_path("juniper_junos_set/sample.set");
    let header_line = header("#", &path);
    let path_str = path.to_string_lossy().into_owned();
    let mut cmd = cfgcut_cmd();
    cmd.args(["-m", "interfaces||ge-0/0/0|>>|", path_str.as_str()])
        .assert()
        .success()
        .stdout(predicate::str::contains(&header_line))
        .stdout(predicate::str::contains(
            "set interfaces ge-0/0/0 unit 0 family inet address",
        ))
        .stdout(predicate::str::contains(
            "set interfaces ge-0/0/0 unit 0 description",
        ));
}

#[test]
fn quiet_mode_with_anonymize_retains_success() {
    let mut cmd = cfgcut_cmd();
    cmd.args([
        "-q",
        "-a",
        "-m",
        "interface GigabitEthernet1",
        &fixture_str("cisco_ios/sample.conf"),
    ])
    .assert()
    .success()
    .stdout(predicate::str::is_empty());
}

#[test]
fn anonymize_with_comments_scrubs_comment_tokens() {
    let tmp = tempdir().unwrap();
    let path = tmp.path().join("comments.cfg");
    fs::write(&path, "## IP 192.0.2.1\n").unwrap();
    let path_str = path.to_string_lossy().into_owned();

    let mut cmd = cfgcut_cmd();
    cmd.args(["-a", "-c", "-m", "|#|IP .*", path_str.as_str()])
        .assert()
        .success()
        .stdout(predicate::str::contains(header("!", &path)))
        .stdout(predicate::str::contains("203.0.113."))
        .stdout(predicate::str::contains("192.0.2.1").not());
}

#[test]
fn warns_when_file_lacks_matches() {
    let tmp = tempdir().unwrap();
    let matched = tmp.path().join("matched.cfg");
    let unmatched = tmp.path().join("unmatched.cfg");
    fs::write(&matched, "interface GigabitEthernet1\n").unwrap();
    fs::write(&unmatched, "hostname test\n").unwrap();

    let mut cmd = cfgcut_cmd();
    cmd.args([
        "-m",
        "interface GigabitEthernet1",
        matched.to_string_lossy().as_ref(),
        unmatched.to_string_lossy().as_ref(),
    ])
    .assert()
    .success()
    .stderr(
        predicate::str::contains("warning: no matches found in")
            .and(predicate::str::contains("unmatched.cfg")),
    )
    .stdout(predicate::str::contains("interface GigabitEthernet1"));
}

#[test]
fn junos_unit_descend_returns_full_subtree() {
    let path = fixture_path("juniper_junos/sample.conf");
    let expected = "interfaces {\n  ge-0/0/0 {\n    unit 0 {\n      family inet {\n        dhcp;\n      }\n    }\n  }\n  ge-0/0/1 {\n    unit 0;\n  }\n}\n";
    let expected = format!("{}\n{expected}", header("##", &path));
    let path_str = path.to_string_lossy().into_owned();
    let mut cmd = cfgcut_cmd();
    cmd.args(["-m", "interfaces||ge-.*||unit 0|>>|", path_str.as_str()])
        .assert()
        .success()
        .stdout(predicate::str::diff(expected));
}
