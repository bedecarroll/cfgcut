# Changelog

## 0.3.0 - 2025-10-13

- add multi-platform Python wheel builds and sdist verification to CI and Release workflows
- enable trusted publishing from GitHub Actions to PyPI for tagged releases
- document the end-to-end release process for maintainers
- advertise and test Python support through 3.14

## 0.2.0 - 2025-10-12

- make `RunRequest` fields private and provide a builder for construction
- mark `CfgcutError`, `TokenDestination`, and `TokenKind` as `#[non_exhaustive]` to allow future expansion without breaking downstream code
- remove backward compatibility guarantees present in `0.1.x`; downstream crates must update to the 0.2 APIs
- ship `pycfgcut` wheels on PyPI with documented installation instructions and packaging metadata
