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
