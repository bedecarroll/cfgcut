# cfgcut Roadmap

## Phase 1 hardening (near-term)
- Backfill integration tests once the vendor fixture drop arrives (multi-vendor, anonymize/tokens combinations, Junos set vs. brace parity).
- Shake out Junos `set` heuristics with real configs and extend pairing rules where necessary.
- Add smoke tests for the Python bridge so core scenarios (matches + token extraction) stay green.
- Enforce `missing_docs = "deny"` once new documentation stabilises so future public API additions stay covered.
- Add focused rustdoc examples/doc-tests for the `RunRequest` builder, anonymization flow, and token output to lock in the documented surface.

## Dialect & fixture support
- Stage the fixture library Brian is sourcing and wire every fixture into the test suite.
- Document expectations for new fixture contributions (source attribution, minimal trimming) alongside the existing dialect guidelines.

## Tooling & fuzzing
- Grow and curate fuzz corpora; capture interesting crashing inputs as regression tests.
- Track CI runtimes and break out `cargo deny`/`cargo fuzz` installs into a cached tool image if necessary.

## Future enhancements (Phase 2+)
- Publish `pycfgcut` wheels (maturin/PyPI) and stabilise the Python API.
- Expose streaming/zero-copy parsing options for large configs and add corresponding Criterion benches.
- Support additional dialects (NX-OS variants, Junos `set` extras, vendor XML exports) once high-confidence fixtures land.
- Extend token extraction with custom token classes and user-defined scrubbing policies.
- Add IPv6 anonymization plus SNMP community string, MAC address, and certificate/key scrubbing to complete credential coverage.
- Extract `Pattern` construction and token accumulation helpers into dedicated modules for clearer ownership.
- Introduce property-based tests (proptest) around parser invariants, anonymizer stability, and pattern determinism.
- Expand Criterion benchmarks to cover end-to-end runs and anonymization hot paths; surface results in CI dashboards.
- Explore regex-automata or precompiled DFA backends for frequently used patterns when startup cost becomes a concern.
- Prototype optional parallel file processing (rayon) for large directory inputs while keeping deterministic output ordering.
- Schedule periodic CI jobs (e.g., weekly) for heavy tasks such as fuzzing, cargo-audit, and benchmark comparisons to keep the main pipeline fast.
