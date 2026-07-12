import os
import subprocess
import sys
from pathlib import Path

import pytest


_EXAMPLE_CASES = [
    ("basic_run.py", ["Matched: True"], [], False),
    (
        "multiple_matches.py",
        ["Matches: system|>>|, protocols||ospf|>>|"],
        [],
        False,
    ),
    (
        "include_comments.py",
        ["! cfgcut matches for full_lab.conf", "router bgp 65001"],
        [],
        False,
    ),
    (
        "anonymize_output.py",
        ["! cfgcut matches for full_lab.conf", "router bgp"],
        ["65001", "65000", "10.0.0.1", "10.10.0.0"],
        False,
    ),
    (
        "token_capture.py",
        ["Captured tokens:", "Wrote token log to"],
        [],
        True,
    ),
]


def _repo_root() -> Path:
    return Path(__file__).resolve().parents[3]


@pytest.mark.parametrize(
    ("script_name", "expected_snippets", "forbidden_snippets", "creates_token_log"),
    _EXAMPLE_CASES,
)
def test_example_scripts(
    script_name: str,
    expected_snippets: list[str],
    forbidden_snippets: list[str],
    creates_token_log: bool,
) -> None:
    repo_root = _repo_root()
    script_path = repo_root / "examples" / script_name
    assert script_path.exists(), f"example script {script_name} missing"

    tokens_path = None
    if creates_token_log:
        tokens_path = script_path.with_suffix(".tokens.jsonl")
        if tokens_path.exists():
            tokens_path.unlink()

    try:
        completed = subprocess.run(
            [sys.executable, str(script_path)],
            cwd=repo_root,
            env=os.environ.copy(),
            capture_output=True,
            text=True,
        )

        assert completed.returncode == 0, completed.stderr
        assert completed.stderr == ""
        for snippet in expected_snippets:
            assert snippet in completed.stdout
        for snippet in forbidden_snippets:
            assert snippet not in completed.stdout

        if tokens_path is not None:
            assert tokens_path.exists()
            assert tokens_path.stat().st_size > 0
    finally:
        if tokens_path is not None and tokens_path.exists():
            tokens_path.unlink()
