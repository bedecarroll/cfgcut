//! Tests for inline match syntax.

use assert_cmd::Command;
use predicates::prelude::*;
use std::path::Path;
use std::path::PathBuf;

fn cfgcut_cmd() -> Command {
    let mut cmd = Command::new(cfgcut_bin());
    cmd.current_dir(manifest_dir());
    cmd
}

fn fixture(rel: &str) -> PathBuf {
    manifest_dir().join("../../tests/fixtures").join(rel)
}

fn expected_with_header(marker: &str, path: &Path, body: &str) -> String {
    let name = path.file_name().map_or_else(
        || path.display().to_string(),
        |name| name.to_string_lossy().into_owned(),
    );
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

#[test]
fn inline_matches_emit_warning_when_scoped_cli_provided() {
    let path = fixture("cisco_ios/inline.conf");
    let body = "interface GigabitEthernet0/1\n description uplink\n";
    let expected = expected_with_header("!", &path, body);
    cfgcut_cmd()
        .args([
            "--within",
            "interface .*",
            "--require",
            "description uplink",
            "-m",
            "description uplink",
            path.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::diff(expected))
        .stderr(predicate::str::contains("ignoring inline matches"));
}

fn cfgcut_bin() -> std::path::PathBuf {
    // Absolute under cargo; runfiles-relative under Bazel, so canonicalize
    // (tests chdir before spawning).
    let bin = std::path::PathBuf::from(env!("CARGO_BIN_EXE_cfgcut"));
    bin.canonicalize().unwrap_or(bin)
}

fn manifest_dir() -> std::path::PathBuf {
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    dir.canonicalize().unwrap_or(dir)
}
