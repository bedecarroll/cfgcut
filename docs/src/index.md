# Welcome

`cfgcut` is a CLI for carving meaningful slices out of large network configuration files. It parses vendor-specific syntax into a common tree so you can match by hierarchy, anonymise sensitive fields, and feed the results into automation tooling.

## Installing

Until official releases are published, build from source using Rust 1.90 or newer:

```bash
cargo install --path crates/cfgcut
```

Python users can install the bindings after compiling the Rust core:

```bash
cargo build -p pycfgcut --release
```

Both commands place the binaries and extension module under the standard cargo target directory.

## Quick start

1. Grab a configuration file (the `tests/fixtures/` directory ships realistic examples).
2. Call `cfgcut` with a match expression consisting of hierarchical regex segments.

```bash
cfgcut -m 'interfaces||ge-0/0/0|>>|' tests/fixtures/juniper_junos/sample.conf
```

The example prints the entire `ge-0/0/0` subtree. Every segment is implicitly anchored, so `ge-0/0/0` will not accidentally match similarly named interfaces.

Add `-a` to anonymise usernames, secrets, ASNs, and IPv4 addresses, or `-q` to run in check-only mode where the exit status signals whether a match was found.

Continue to [CLI Usage](./usage.md) for the full command reference and matcher behaviour.
