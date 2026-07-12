"""Preserve IOS comment delimiters alongside matched output."""

from pathlib import Path

from pycfgcut import run_cfg


def main() -> None:
    repo_root = Path(__file__).resolve().parents[1]
    fixture = repo_root / "tests" / "fixtures" / "cisco_ios" / "full_lab.conf"

    result = run_cfg(
        ["router bgp 65001|>>|"],
        [str(fixture)],
        with_comments=True,
    )

    print(result["stdout"], end="")


if __name__ == "__main__":
    main()
