# cfgcut Roadmap

## Phase 1 hardening (near-term)
- Backfill integration tests once the vendor fixture drop arrives (multi-vendor, anonymize/tokens combinations, Junos set vs. brace parity).
- Shake out Junos `set` heuristics with real configs and extend pairing rules where necessary.
- Add smoke tests for the Python bridge so core scenarios (matches + token extraction) stay green.

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
