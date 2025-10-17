# Changelog

## 0.3.2 - 2025-10-17

- fix Cisco banner parsing across IOS, IOS-XE, IOS-XR, NX-OS, and other indent dialects by tracking custom delimiters

## 0.3.1 - 2025-10-13

- align crate metadata with the v0.3.1 release tag used by cargo-dist

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
