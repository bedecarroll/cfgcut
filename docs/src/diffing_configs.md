# Diffing Configurations

Extracting only the sections you care about makes it much easier to compare large device configurations. The `--sort-by-path` flag keeps the rendered output grouped by the hierarchical path of each match, so reordering blocks (a common behavior on Palo Alto and similar platforms) no longer appears as a change.

## Normalize The Output

Use the same match expressions for both configurations and include `--sort-by-path` to stabilize the order. Enable additional switches such as `--anonymize` or `--with-comments` as needed for your review.

```bash
cfgcut --sort-by-path \
  -m "set network.virtual-router .*|>>|" \
  old.conf
```

The command above emits the matched configuration with blocks ordered by their hierarchical path instead of their position within the source file.

## Compare Two Files

Any external diff tool works once the output is normalized. Classic Unix `diff` with process substitution is a convenient option on macOS and Linux:

```bash
diff -u <(cfgcut --sort-by-path -m "address-group .*|>>|" before.conf) \
        <(cfgcut --sort-by-path -m "address-group .*|>>|" after.conf)
```

Because both invocations sort by path, identical blocks that only moved between stanzas no longer show up as additions or deletions.

### Tips

- Add `--anonymize` when sharing diffs externally so sensitive values are replaced consistently.
- When comparing different dialects or vendors, keep the match expression vendor-specific but reuse the `--sort-by-path` switch.
- For GUI diff tools, direct the output to temporary files: `cfgcut --sort-by-path … > /tmp/before.txt` and `… > /tmp/after.txt`, then point the tool at those files.
