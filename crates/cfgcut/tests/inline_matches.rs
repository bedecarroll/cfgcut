use assert_cmd::Command;
use predicates::prelude::*;
use std::path::PathBuf;

fn cfgcut_cmd() -> Command {
    let mut cmd = Command::cargo_bin("cfgcut").unwrap();
    cmd.current_dir(env!("CARGO_MANIFEST_DIR"));
    cmd
}

fn fixture(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures")
        .join(rel)
}

#[test]
fn inline_matches_without_cli_arguments() {
    let expected = "hostname lab-inline\ninterface GigabitEthernet0/1\n description uplink\n";
    cfgcut_cmd()
        .arg(fixture("cisco_ios/inline.conf"))
        .assert()
        .success()
        .stdout(predicate::str::diff(expected))
        .stderr(predicate::str::is_empty());
}

#[test]
fn inline_matches_emit_warning_when_cli_provided() {
    let expected = "interface GigabitEthernet0/2\n shutdown\n";
    cfgcut_cmd()
        .args([
            "-m",
            "interface GigabitEthernet0/2|>>|",
            fixture("cisco_ios/inline.conf").to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::diff(expected))
        .stderr(predicate::str::contains("ignoring inline matches"));
}
