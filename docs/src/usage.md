# CLI Usage

## Command reference

`cfgcut` accepts one or more `-m/--match` expressions and a list of files or directories. Directories are expanded using glob semantics, so you can point the tool at an entire configuration dump.

| Option | Description |
| --- | --- |
| `-m, --match <MATCH>` | Hierarchical regex segments (anchored). Repeat the flag for multiple patterns. |
| `-c, --with-comments` | Include comment lines recognised by the active dialect. |
| `-q, --quiet` | Suppress stdout; rely on exit status to detect matches. |
| `-a, --anonymize` | Scramble usernames, secrets, ASNs, and IPv4 addresses deterministically. |
| `--tokens` | Emit newline-delimited JSON token records for every match. |
| `--tokens-out <PATH>` | Write token records to a file instead of stdout. |
| `--help` | Display the full usage text with examples. |

Combine flags as needed. For example, run a check that exits with status 0 only when a BGP neighbour exists:

```bash
cfgcut -q -m 'protocols||bgp||group CUSTOMERS||neighbor 198\.51\.100\.10' router.conf
```

## Match semantics

Configurations are parsed into a hierarchy. Use `||` to move down levels and place `|>>|` after a segment to include the entire subtree underneath that node.

- Every segment is wrapped with `^...$` automatically. `ge-.*` targets individual interfaces rather than matching a partial line.
- Matches print their ancestor context so output remains valid configuration. Without `|>>|`, only the matched line plus its parents are shown.
- Comment markers are normalised per dialect (for example `!` on IOS, `#` on Junos). Opt into printing them with `-c/--with-comments`.

Example: fetch every trunk interface from a Cisco IOS device while keeping parent context.

```bash
cfgcut -m 'interface .*||switchport trunk allowed vlan .*' tests/fixtures/cisco_ios/sample.conf
```

To grab an entire Junos subtree:

```bash
cfgcut -m 'interfaces||ae1|>>|' tests/fixtures/juniper_junos/sample.conf
```

## Anonymisation and token output

Enabling `-a/--anonymize` replaces sensitive fields with stable placeholders that remain consistent within a single run. The original values are still available through the token stream produced by `--tokens` or `--tokens-out`.

Token payloads include the dialect, hierarchical path, kind, original value, anonymised value (when available), and source line. See [Token Extraction Design Notes](./token_extraction.md) for the data model and ongoing work.
