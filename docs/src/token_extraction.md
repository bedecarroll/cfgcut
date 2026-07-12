# Token Extraction Design Notes

## Objectives
- Reuse the anonymizer's deterministic token maps so anonymized output and extracted tokens stay in sync.
- Support both CLI and library workflows without changing existing stdout behaviour by default.
- Provide deterministic ordering of extracted tokens to keep diffs stable.

## Planned CLI surface
- Introduce a `--tokens` flag that prints newline-delimited JSON or key/value pairs describing matches.
- Allow `--tokens` to be combined with `--quiet` so automation can act on exit status and token payload without extra text.
- Support writing tokens to a file (`--tokens-out <PATH>`) in addition to stdout for larger captures.

## Data model
- Describe each token with fields: `dialect`, `path` (hierarchical segments), `kind` (ip, asn, username, secret, literal), and `value`.
- Preserve a reference to the anonymized value when anonymization is active.
- Capture positional metadata (line/column) to help IDE integrations.

## Implementation sketch
- Extend `MatchAccumulator` to optionally record token spans using the anonymizer maps.
- Add a `TokenSink` trait so new dialects can surface dialect-specific token types without coupling to core logic.
- Ensure comment handling obeys the same switches as existing output (`-c/--with-comments`).

## Testing strategy
- Add table-driven unit tests per dialect to cover common command patterns (interface addresses, BGP ASNs, login commands).
- Mirror the anonymizer integration tests with `--tokens` enabled to confirm consistent mappings.
- Extend fuzz targets to emit token metadata behind a feature gate to catch malformed spans early.

## Open questions
- Should tokens be grouped per match expression or per line? (Default proposal: per line.)
- How should overlapping token classes (e.g. passwords containing IP-like strings) be prioritised?
- Do we need opt-in redaction for custom patterns supplied via config files?
