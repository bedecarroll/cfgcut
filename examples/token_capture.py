"""Collect token metadata for automated post-processing."""

from pathlib import Path

from pycfgcut import run_cfg


def main() -> None:
    repo_root = Path(__file__).resolve().parents[1]
    fixture = repo_root / "tests" / "fixtures" / "cisco_ios" / "full_lab.conf"
    tokens_path = Path(__file__).with_suffix(".tokens.jsonl")

    result = run_cfg(
        ["interface GigabitEthernet0/0|>>|"],
        [str(fixture)],
        tokens=True,
        tokens_out=str(tokens_path),
    )

    print("Captured tokens:", len(result["tokens"]))
    for record in result["tokens"][:3]:
        print(
            f"{record['dialect']} {record['path']} "
            f"{record['kind']} -> {record['anonymized'] or record['original']}"
        )

    if tokens_path.exists():
        print(f"Wrote token log to {tokens_path}")


if __name__ == "__main__":
    main()
