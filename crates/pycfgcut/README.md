# pycfgcut

Python bindings for [`cfgcut`](https://cfgcut.bedecarroll.com), the deterministic configuration slicer for network engineers.

## Installation

```bash
pip install pycfgcut
```

`pycfgcut` ships prebuilt wheels for CPython 3.9 and newer on Linux and macOS. If a wheel is unavailable for your platform `pip` falls back to building from source; make sure a compatible Rust toolchain is installed (`rustup` recommended).

## Usage

```python
from pathlib import Path

from pycfgcut import run_cfg

fixture = Path("sample.conf")
result = run_cfg(["interfaces|>>|"], [str(fixture)], anonymize=True, tokens=True)

if result["matched"]:
    print(result["stdout"])
```

Refer to the [cfgcut documentation](https://cfgcut.bedecarroll.com) for matcher semantics, anonymisation behaviour, and CLI parity guarantees.

## License

`pycfgcut` is distributed under the terms of the MIT License. See the top-level `LICENSE` file in this repository for the full text.
