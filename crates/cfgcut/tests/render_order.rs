//! Tests for output rendering order.

use std::path::Path;

use assert_cmd::Command;
use predicates::prelude::*;

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

fn header(marker: &str, path: &str) -> String {
    let name = Path::new(path).file_name().map_or_else(
        || path.to_string(),
        |name| name.to_string_lossy().into_owned(),
    );
    format!("{marker} cfgcut matches for {name}")
}

#[test]
fn sort_by_path_orders_matches_hierarchically() {
    let path = fixture_path("cisco_ios/out_of_order.conf");
    let header_line = header("!", &path);

    let expected_body = "\
interface GigabitEthernet1
 description access-edge
 ip address 10.0.1.1 255.255.255.0
 no shutdown
interface GigabitEthernet2
 description uplink-to-core
 ip address 10.0.0.2 255.255.255.252
 no shutdown
";
    let expected = format!("{header_line}\n{expected_body}");

    let mut cmd = cfgcut_cmd();
    cmd.args([
        "-m",
        "interface GigabitEthernet.*|>>|",
        "--sort-by-path",
        &path,
    ])
    .assert()
    .success()
    .stdout(predicate::str::diff(expected));
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
