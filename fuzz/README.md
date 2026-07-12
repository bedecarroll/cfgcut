# cfgcut fuzzing

This directory holds libFuzzer harnesses for exercising the parser and matcher.

## Quick start

1. Install tooling:
   ```bash
   cargo install cargo-fuzz
   rustup toolchain install nightly --profile minimal
   ```
2. Run a harness via mise (defaults to the `parser` target when no name is supplied):
   ```bash
   mise run fuzz
   mise run fuzz matcher -- -runs=1000
   ```

Seed corpora live under `fuzz/corpus/`. Add new samples that demonstrate tricky configuration
constructs when extending dialect coverage so fuzzing can reach them quickly.

The targets are compiled with the `fuzzing` feature, exposing internal APIs defined in
`crates/cfgcut`. Crash artifacts appear under `fuzz/artifacts/` and should be minimised before
filing issues or adding regression tests.
