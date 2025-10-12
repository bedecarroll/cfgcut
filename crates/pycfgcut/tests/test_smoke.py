from pathlib import Path

from pycfgcut import run_cfg


def _fixture_path(relative: str) -> Path:
    root = Path(__file__).resolve().parents[3]
    return root / "tests" / "fixtures" / relative


def test_run_cfg_smoke():
    fixture = _fixture_path("juniper_junos/sample.conf")
    result = run_cfg(["interfaces|>>|"], [str(fixture)])

    assert result["matched"] is True
    assert isinstance(result["stdout"], str)
    assert isinstance(result["tokens"], list)
