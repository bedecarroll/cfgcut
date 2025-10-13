use std::path::PathBuf;

use assert_cmd::Command;
use predicates::prelude::*;
use predicates::str::contains;

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
        .stdout(contains("-a/--anonymize"))
        .stdout(contains("--tokens"));
}

#[test]
fn version_reports_semver() {
    let mut cmd = cfgcut_cmd();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(contains("cfgcut").and(contains("0.3.1")));
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
