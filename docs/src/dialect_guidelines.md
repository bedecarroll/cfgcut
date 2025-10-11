# Dialect Contribution Guidelines

This project expects every vendor/platform parser to live under `crates/cfgcut/src/dialect/` in its own module. Follow these practices when adding a new dialect:

## Parser structure
- Reuse the shared helpers in `dialect::shared` whenever possible (comment detection, match text extraction, hierarchy wiring).
- Keep regexes and parsing rules limited to the minimum needed for correctness; avoid vendor-specific behaviour in the matcher.
- Collect children and closing nodes the same way existing dialects do so the matcher’s anchoring logic behaves consistently.

## Comment and ignored text handling
- Define comment markers in the dialect module so `-c/--with-comments` works uniformly. Indentation should not affect comment detection.
- Exclude device-generated boilerplate (hashes, timestamps) during parsing to avoid noise in matches. Document any ignored text in module comments or tests.

## Testing expectations
- Add snippet-based unit tests covering: comment detection, hierarchy/parent assignment, and closing-brace emission (for brace dialects).
- Extend `crates/cfgcut/tests` with integration scenarios using fixtures under `tests/fixtures/<vendor_platform>/`. New fixtures should be minimal but realistic.
- Ensure new fixtures are referenced in `tests/fixtures/README.md` with source attribution.

## Anonymizer and token extraction hooks
- Avoid building anonymization logic into dialect parsers. Use the shared anonymizer so token scrubbing remains consistent.
- When token extraction lands, surface dialect-specific token types via the shared trait rather than custom plumbing.

## Fuzzing & hardening
- Consider adding a seed corpus covering edge cases for the new dialect under `fuzz/corpus/` once fixtures exist.
- Run `mise run check` before opening a PR; it enforces formatting, clippy, tests, the dependency audit, and doc build.
- Capture coverage with `mise run coverage` when touching parser/matcher logic.

## Review checklist
- `cargo fmt`, `cargo clippy -- -D warnings`, `cargo nextest run --workspace --all-targets`, `cargo test --doc`, and (if installed) `cargo deny check` all succeed.
- Documentation: update README examples if the dialect adds user-visible syntax, and note any new fixtures in `tests/fixtures/README.md`.
