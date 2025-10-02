# CI Integration Notes

## Required steps
- Run `mise run check` on every pull request to cover `cargo fmt`, `cargo clippy`, `cargo nextest run` (plus doc tests), the dependency audit, and the docs build.
- Publish coverage artefacts with `mise run coverage` (requires `cargo-llvm-cov`).
- Cache the cargo home directory to speed up repeated lint/test runs.

## Recommended extras
- Nightly or scheduled job executing `mise run fuzz parser -- -runs=1000` and `mise run fuzz matcher -- -runs=1000` (after installing a nightly toolchain) to exercise the seed corpora without impacting PR latency.
- Upload crash artifacts from fuzz runs as CI artifacts for quick triage.
- Track execution time; fail the fuzz job only on crashes/timeouts to avoid flakiness.
- Optional benchmarking job (`mise run bench`) on a dedicated runner to spot large regressions.

## Future hooks
- Once token extraction ships, add integration tests that execute with the new flags and ensure they run as part of the PR suite.
- When additional dialect fixtures land, extend CI to validate that each dialect parser has coverage (unit + integration).

## Example GitHub Actions outline
```yaml
name: ci

on:
  pull_request:
  push:

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          toolchain: 1.90
          components: clippy,rustfmt
      - uses: jdx/mise-action@v2
        with:
          version: latest
      - run: cargo install cargo-deny
      - run: cargo install mdbook --locked
      - run: cargo install cargo-llvm-cov
      - run: cargo install cargo-nextest --locked
      - run: mise run check
      - run: mise run coverage

  fuzz-smoke:
    if: github.event_name == 'schedule'
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          toolchain: 1.90
      - uses: jdx/mise-action@v2
        with:
          version: latest
      - run: cargo install cargo-fuzz
      - run: rustup toolchain install nightly --profile minimal
      - run: mise run fuzz parser -- -runs=1000
      - run: mise run fuzz matcher -- -runs=1000
```
