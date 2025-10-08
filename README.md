# cfgcut

`cfgcut` is a command-line tool that helps network engineers slice big configuration dumps into the exact snippets they need. Feed it IOS, NX-OS, EOS, or Junos configs and it will walk the hierarchy, keep the right indentation and context, and hand you clean output that is ready for change reviews, audits, and automation.

## Why cfgcut?

- **Precise matching** – Describe hierarchy with `||` separators and cfgcut anchors each segment for you. No more brittle `grep` pipelines.
- **Keep context** – Matched lines are returned with the parent structure intact so the snippet still pastes cleanly back into a device.
- **Batch friendly** – Point cfgcut at individual files, directories, or globs; it emits a non-zero exit code when nothing matches so it fits neatly into scripts and CI jobs.
- **Safe to share** – Optional anonymisation scrubs usernames, secrets, ASNs, and IPv4 addresses while keeping the rest of the config readable.
- **Token inventory** – Turn on JSON token output to export sensitive values (scrubbed or original) for post-processing.

## Supported platforms

| Vendor | Platform |
| --- | --- |
| Cisco | IOS |
| Cisco | NX-OS |
| Arista | EOS |
| Juniper | JunOS (brace syntax) |
| Juniper | JunOS (set syntax) |

More dialects are on the roadmap. See the issue template "Platform support request" if you want to help add one.

## Install

Grab a prebuilt binary from the [Releases](https://github.com/astral-sh/cfgcut/releases) page, or install from source with Cargo:

```bash
cargo install --git https://github.com/astral-sh/cfgcut cfgcut
```

The CLI is self-contained and runs anywhere Rust 1.90+ is available.

## Quick tour

Assume `tests/fixtures/cisco_ios/sample.conf` looks like a typical IOS edge switch. Here are a few things you can do:

### Pull a whole interface block

```bash
cfgcut -m 'interface GigabitEthernet1|>>|' tests/fixtures/cisco_ios/sample.conf
```

### Check for a shutdown uplink and exit quietly if it exists

```bash
cfgcut -q -m 'interface GigabitEthernet2||shutdown' tests/fixtures/cisco_ios/sample.conf
```

### Grab every interface subtree on a Junos device

```bash
cfgcut -m 'interfaces|>>|' tests/fixtures/juniper_junos/sample.conf
```

### Scrub secrets while exporting tokens

```bash
cfgcut -a --tokens -m '.*' tests/fixtures/cisco_ios/sample.conf
```

Use the return code (`echo $?`) inside scripts to determine whether a match was found.

## Match expressions in plain language

1. **Split levels with `||`.** Each segment represents a node in the configuration hierarchy: `protocols||bgp||neighbor .*` drills down from protocols to a specific neighbour.
2. **Segments are anchored.** `GigabitEthernet1` only matches that exact stanza—no need to add `^` or `$`.
3. **Descend with `|>>|`.** Place `|>>|` after a segment when you want the full subtree beneath it.
4. **Target comments** by prefixing a segment with `|#|` and using `-c`/`--with-comments` to print them.

### Inline match blocks

When you would rather bundle match expressions with a fixture, add a leading comment that looks like a Jinja block:

```
{# [
'hostname lab-edge',
'interfaces|>>|',
] #}
```

Place the block at the very top of the file (whitespace is fine) and list each expression in single or double quotes. `cfgcut` will peel the comment before parsing so nothing leaks into the output. Command-line `-m/--match` arguments still win; if both are present `cfgcut` uses the CLI values and prints a warning on stderr so you know the inline list was ignored.

The CLI keeps parent blocks in the output so pasted snippets remain valid configs.

## Anonymisation & tokens

- `-a` / `--anonymize` swaps usernames, passwords, ASNs, and IPv4 addresses with deterministic placeholders so you can share snippets safely.
- `--tokens` emits newline-delimited JSON describing each sensitive token; pair with `--tokens-out <FILE>` to write them directly to disk for follow-up processing.

## Python bindings (work in progress)

A PyO3-based module (`pycfgcut`) mirrors the CLI surface. You can build it locally today with `maturin develop`, but prebuilt wheels are not yet published. Packaging plans and API details live in the mdBook.

## Learn more

Detailed matcher behaviour, contribution guidelines, coverage expectations, and the roadmap are documented in our mdBook:

```
mdbook serve docs
```

Open `http://localhost:3000` to browse the full guide.

## For contributors

- Run `mise run check` before sending a PR (fmt, clippy, tests, cargo-deny, docs).
- Optional extras: `mise run coverage`, `mise run bench`, `mise run fuzz parser`.
- See `CONTRIBUTING.md` and `docs/dialect_guidelines.md` for coding standards and dialect expectations.

Happy slicing!
