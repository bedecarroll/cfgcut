# cfgcut

`cfgcut` is a deterministic slicer for network configuration files. It understands common vendor dialects, lets you lift specific subtrees with declarative match expressions, and can anonymise sensitive tokens as it slices.

## Installation

Install from crates.io once published, or build directly from this repository:

```bash
cargo install --path crates/cfgcut
```

## Documentation

Full matcher syntax, anonymisation behaviour, and contribution guidelines live at [cfgcut.bedecarroll.com](https://cfgcut.bedecarroll.com).

## License

`cfgcut` is available under the terms of either the MIT License or the Apache License 2.0.
