use assert_cmd::Command;
use predicates::prelude::*;
use std::path::{Path, PathBuf};

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

fn expected_with_header(marker: &str, path: &Path, body: &str) -> String {
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string());
    let mut output = format!("{marker} cfgcut matches for {name}\n");
    output.push_str(body);
    output
}

#[test]
fn inline_matches_without_cli_arguments() {
    let path = fixture("cisco_ios/inline.conf");
    let body = "hostname lab-inline\ninterface GigabitEthernet0/1\n description uplink\n";
    let expected = expected_with_header("!", &path, body);
    cfgcut_cmd()
        .arg(path)
        .assert()
        .success()
        .stdout(predicate::str::diff(expected))
        .stderr(predicate::str::is_empty());
}

#[test]
fn inline_matches_emit_warning_when_cli_provided() {
    let path = fixture("cisco_ios/inline.conf");
    let body = "interface GigabitEthernet0/2\n shutdown\n";
    let expected = expected_with_header("!", &path, body);
    cfgcut_cmd()
        .args([
            "-m",
            "interface GigabitEthernet0/2|>>|",
            path.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::diff(expected))
        .stderr(predicate::str::contains("ignoring inline matches"));
}
