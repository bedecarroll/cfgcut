"""Minimal run_cfg invocation capturing a Junos interfaces subtree."""

from pathlib import Path

from pycfgcut import run_cfg


def main() -> None:
    repo_root = Path(__file__).resolve().parents[1]
    fixture = repo_root / "tests" / "fixtures" / "juniper_junos" / "sample.conf"

    result = run_cfg(["interfaces|>>|"], [str(fixture)])

    print(result["stdout"], end="")
    print(f"Matched: {result['matched']}")


if __name__ == "__main__":
    main()
