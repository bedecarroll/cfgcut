# Releasing cfgcut and pycfgcut

This repository publishes both the Rust CLI (`cfgcut`) and the Python bindings (`pycfgcut`). The release automation builds native binaries, produces wheels and an sdist, publishes them to PyPI, and attaches everything to a GitHub release.

## Pre-release checklist

1. Bump versions in `Cargo.toml` as needed (`crates/cfgcut` and `crates/pycfgcut`) and update the changelog.
2. Regenerate the Python bindings with `mise run pytest` to ensure the wheel continues to pass its test suite locally.
3. Commit and push the changes to `main`.

## Cutting a release

1. Create an annotated tag that matches the desired semantic version, for example:
   ```bash
   git tag -a v0.3.2 -m "cfgcut v0.3.2"
   git push origin v0.3.2
   ```
2. The `Release` workflow builds:
   - cfgcut binaries for Linux (GNU + musl), macOS (x86_64 + arm64), and Windows.
   - pycfgcut wheels for Linux (manylinux x86_64), macOS (x86_64 + arm64), and Windows, plus a source distribution.
   - A pytest smoke test for each built wheel and the sdist.
3. Once all artifacts are available, the workflow publishes the Python artifacts to PyPI (using trusted publishing) and then creates a GitHub release that bundles both the Rust and Python deliverables.

## PyPI trusted publishing setup

1. Create the `pycfgcut` project on [PyPI](https://pypi.org) and mark it as owned by the maintainer account.
2. On PyPI, add a new *GitHub Actions* trusted publisher scoped to the `bedecarroll/cfgcut` repository.
3. In this repository, add an environment named `pypi` (repository settings → Environments) and require review if you want extra safety.
4. Tag a release; the `publish-pypi` job will use OIDC to upload the wheels and sdist automatically.

If a publish needs to be retried (for example, when PyPI is unavailable), re-run the `publish-pypi` job for the relevant tag once the issue is resolved.
