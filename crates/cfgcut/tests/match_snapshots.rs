use std::fs;
use std::path::PathBuf;

use assert_cmd::Command;
use insta::assert_snapshot;
use serde::Deserialize;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate dir has parent")
        .parent()
        .expect("workspace root exists")
        .to_path_buf()
}

fn cfgcut_cmd() -> Command {
    let mut cmd = Command::cargo_bin("cfgcut").expect("binary built");
    cmd.current_dir(env!("CARGO_MANIFEST_DIR"));
    cmd
}

#[derive(Debug, Deserialize)]
struct Manifest {
    #[serde(rename = "case")]
    cases: Vec<Case>,
}

#[derive(Debug, Deserialize)]
struct Case {
    name: String,
    matches: Vec<String>,
    input: String,
}

fn load_manifest() -> Manifest {
    let path = workspace_root().join("tests/fixtures/synthetic_manifest.toml");
    let raw = fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
    toml::from_str(&raw).unwrap_or_else(|err| panic!("failed to parse {}: {err}", path.display()))
}

#[test]
fn synthetic_snapshots() {
    let manifest = load_manifest();
    let root = workspace_root();

    for case in manifest.cases {
        let mut cmd = cfgcut_cmd();
        for pattern in &case.matches {
            cmd.args(["-m", pattern]);
        }
        cmd.arg(root.join(&case.input));

        let output = cmd.assert().success().get_output().stdout.clone();
        let stdout = String::from_utf8_lossy(&output);

        let mut snapshot = format!("input: {}\n", case.input);
        for pattern in &case.matches {
            snapshot.push_str("match: ");
            snapshot.push_str(pattern);
            snapshot.push('\n');
        }
        snapshot.push_str("---\n");
        snapshot.push_str(stdout.trim_end_matches('\n'));
        snapshot.push('\n');

        assert_snapshot!(case.name, snapshot);
    }
}
