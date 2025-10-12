# Changelog

## 0.2.0 - 2025-10-12

- make `RunRequest` fields private and provide a builder for construction
- mark `CfgcutError`, `TokenDestination`, and `TokenKind` as `#[non_exhaustive]` to allow future expansion without breaking downstream code
- remove backward compatibility guarantees present in `0.1.x`; downstream crates must update to the 0.2 APIs
