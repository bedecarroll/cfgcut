//! Tests for CLI help output.

use assert_cmd::Command;
use predicates::prelude::*;
use predicates::str::contains;

fn cfgcut_cmd() -> Command {
    let mut cmd = Command::new(cfgcut_bin());
    cmd.current_dir(manifest_dir());
    cmd
}

fn fixture_path(rel: &str) -> String {
    let base = manifest_dir();
    base.join("../../tests/fixtures")
        .join(rel)
        .to_string_lossy()
        .into_owned()
}

#[test]
fn help_shows_usage() {
    let mut cmd = cfgcut_cmd();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(contains("cfgcut").and(contains("Usage")));
}

#[test]
fn help_mentions_matching_defaults() {
    let mut cmd = cfgcut_cmd();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(contains("implicitly anchored"))
        .stdout(contains("--within"))
        .stdout(contains("--require"))
        .stdout(contains("-a, --anonymize"))
        .stdout(contains("--tokens"));
}

#[test]
fn version_reports_semver() {
    let mut cmd = cfgcut_cmd();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(contains("cfgcut").and(contains("0.4.0")));
}

#[test]
fn missing_input_is_an_error() {
    let mut cmd = cfgcut_cmd();
    cmd.assert().failure().stderr(contains("Usage"));
}

#[test]
fn no_matches_still_report_to_stderr() {
    let mut cmd = cfgcut_cmd();
    cmd.args([
        "-m",
        "nonexistent pattern",
        &fixture_path("cisco_ios/sample.conf"),
    ])
    .assert()
    .failure()
    .stderr(
        predicate::str::contains("warning: no matches found in")
            .and(predicate::str::contains("sample.conf")),
    );
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
