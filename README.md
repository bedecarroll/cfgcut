# cfgcut

`cfgcut` is a Rust-based configuration parser designed to allow you to extract
various sections of configuration ready to use as text objects.

## Main use cases

- Extract sections of configuration for diffing between devices
- Ensure configuration commands are present in configuration

## Examples

Get all interfaces and just the commands that start with switchport.

```bash
cfgcut -m 'interface .*||switchport .*' cisco.cfg
```

Get all descendant objects.

```bash
$ cfgcut -m 'system|>>|' juniper.cfg
system {
  host-name test;
  domain-name testnet.net;
}
```

Extract an address from an interface.

```bash
$ cfgcut -m 'interfaces||ae1||unit 1||inet||address .*' -e
192.168.0.1/24
```

## Features

- Ability to provide multiple match criteria
- Non-zero exit code if there are no matches

## Match syntax

The match syntax is fairly simple. The configuration is parsed into a hierarchy,
and using `||` will allow you to descend down the hierarchy. You are allowed to
use regex inside each level match. If you want to match infinite depth below
a certain point, you can use the special delimiter `|>>|` that denotes the
return of all lower levels. This is useful in deeply nested configurations like
`JunOS`. If you simply want things at the current level you've nested down into,
you can use `.*` to give all commands but ignore anything with
a sublevel. This use case would be unusual, but if you wanted the system config
in Juniper but only top-level objects like host-name and not users, this would
allow you to do that. You can also do things like `|#|.*` to match only
comments. Comments are filtered out, but by using `|#|` you can match on
comments, and if you want them printed at the level they are parsed at, you can
use the flag `-c` or `--with-comments`.

Note that certain text will be ignored by the dialect parser. This is to
prevent things like configuration hash sums and other irrelevant system-generated
text from polluting your configuration.

## Supported configuration formats

Each vendor and platform pair is called a dialect. These dialects allow us to
parse the configuration and generate the hierarchy. The dialect is what allows
us to understand things like hierarchy, comments, or generated text.

| Vendor | Platform | Supported |
|---|---|---|
| Cisco | IOS | |
| Cisco | NX-OS | |
| Cisco | IOS-XE | |
| Cisco | IOS-XR | |
| Cisco | AireOS | |
| Arista | EOS | |
| Juniper | JunOS | |

### Experimental Features

#### Variable extraction

This feature attempts to pull objects like IP addresses, prefix list names, and
MAC addresses from commands. This allows you to use a match statement for
a specific level and then extract tokens that are at that level.

#### Python bindings

Python is the primary language of network engineers, so it is important to
provide the capability to call this tool as a library. This feature allows you
to work directly with the parsed text.

```python
from cfgcut import Cfg

with open('cisco.txt') as f:
  raw_config = f.read()

parsed = Cfg(raw_config, force_dialect="cisco_ios")
parsed.match("interface Gig.*|>>|")
print(bool(parsed))  # True
```

This feature uses `PyO3`.

## Installation
