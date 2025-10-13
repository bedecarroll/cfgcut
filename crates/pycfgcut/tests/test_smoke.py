from importlib import metadata
from pathlib import Path

import pytest

from pycfgcut import __version__, run_cfg


def _fixture_path(relative: str) -> Path:
    root = Path(__file__).resolve().parents[3]
    return root / "tests" / "fixtures" / relative


def test_run_cfg_smoke():
    fixture = _fixture_path("juniper_junos/sample.conf")
    result = run_cfg(["interfaces|>>|"], [str(fixture)])

    assert result["matched"] is True
    assert isinstance(result["stdout"], str)
    assert isinstance(result["tokens"], list)


def test_with_comments_includes_comment_lines(tmp_path: Path):
    config = tmp_path / "sample.cfg"
    config.write_text(
        "## comment marker\nsystem {\n    host-name test;\n}\n",
        encoding="utf-8",
    )

    result = run_cfg(["|#|comment.*"], [str(config)], with_comments=True)

    assert "## comment marker" in result["stdout"]


def test_tokens_written_to_file(tmp_path: Path):
    fixture = _fixture_path("arista_eos/sample.conf")
    output = tmp_path / "tokens.jsonl"
    result = run_cfg(
        ["interface Management1|>>|"],
        [str(fixture)],
        anonymize=True,
        tokens=True,
        tokens_out=str(output),
    )

    assert result["matched"] is True
    assert result["tokens"], "token list should not be empty when anonymising"
    assert output.exists()
    lines = output.read_text(encoding="utf-8").splitlines()
    assert lines, "tokens_out should produce at least one JSON line"


@pytest.mark.parametrize("anonymize", (True, False))
def test_tokens_flag_followed(anonymize: bool):
    fixture = _fixture_path("arista_eos/sample.conf")
    result = run_cfg(
        ["interface Management1|>>|"],
        [str(fixture)],
        anonymize=anonymize,
        tokens=True,
    )

    if anonymize:
        assert result["tokens"], "anonymize should emit replacement tokens"
        assert all(token["anonymized"] for token in result["tokens"])
    else:
        assert result["tokens"], "token metadata still surfaces when anonymize is disabled"
        assert all(token["anonymized"] is None for token in result["tokens"])


def test_invalid_inputs_raise():
    fixture = _fixture_path("juniper_junos/sample.conf")

    with pytest.raises(RuntimeError):
        run_cfg([], [str(fixture)])

    with pytest.raises(RuntimeError):
        run_cfg(["interfaces|>>|"], [])


def test_version_matches_package_metadata():
    assert __version__ == metadata.version("pycfgcut")
