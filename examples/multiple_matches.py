"""Request multiple subtrees from a single Junos configuration."""

from pathlib import Path

from pycfgcut import run_cfg


def main() -> None:
    repo_root = Path(__file__).resolve().parents[1]
    fixture = repo_root / "tests" / "fixtures" / "juniper_junos" / "sample.conf"

    match_expressions = ["system|>>|", "protocols||ospf|>>|"]
    result = run_cfg(match_expressions, [str(fixture)])

    print("Matches:", ", ".join(match_expressions))
    print(result["stdout"], end="")


if __name__ == "__main__":
    main()
