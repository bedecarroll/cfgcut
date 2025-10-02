# Contributing to cfgcut

Thanks for helping improve cfgcut! This document outlines the workflow and quality bars we expect for every change.

## Getting started

1. Install [Rust](https://www.rust-lang.org/tools/install) 1.90 or newer. The repository pins toolchain components in `rust-toolchain.toml`.
2. Install [mise](https://github.com/jdx/mise) so the project tasks are available (`mise --version`).
3. Fetch helper binaries as needed:
   - `cargo install cargo-deny`
   - `cargo install cargo-nextest --locked`
   - `cargo install cargo-llvm-cov`
   - `cargo install cargo-fuzz`
   - `rustup toolchain install nightly --profile minimal`
   - `cargo install typos-cli`

## Development workflow

- **Format & lint**: `mise run check` (runs fmt, clippy with pedantic/nursery, cargo-nextest-powered tests + doc tests, dependency audit, docs build).
- **Spell check**: `typos` to catch documentation/config spelling slips (config in `.typos.toml`).
- **Coverage**: `mise run coverage` emits HTML + LCOV artefacts under `target/llvm-cov/` and enforces an 80 % line threshold.
- **Benchmarks**: `mise run bench` executes the Criterion suite for regression tracking.
- **Fuzzing**: `mise run fuzz parser` / `mise run fuzz matcher` (requires nightly + `cargo-fuzz`). Add new corpora under `fuzz/corpus/` when extending behaviour.

Pushes and PRs must pass `mise run check`. Feature work should include tests covering new matcher/dialect behaviour as well as anonymizer/token extraction changes where relevant.

## Coding guidelines

- Follow the rules in `AGENTS.md` and `docs/dialect_guidelines.md`.
- Keep dialect-specific logic isolated; shared behaviour belongs in common modules.
- Prefer deterministic transforms—scrubbing/anonymizing should be reproducible within a run.
- Surface new flags and behaviour in `README.md` and the mdBook (`docs/`).

## Opening a pull request

1. Confirm `mise run check`, `mise run coverage`, and (if applicable) `mise run fuzz …` all succeed locally.
2. Update CHANGELOG or roadmap entries when scope changes.
3. Include rationale for any new lint allowances in `clippy.toml` if you need to relax rules.
4. Link real-world config additions in `tests/fixtures/README.md` with licensing details.

We appreciate every contribution—thank you for helping build a robust network configuration toolkit!
