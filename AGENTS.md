# Engineering Guidelines

- Favor clarity over cleverness. Keep control flow explicit and data structures simple.
- Treat every pattern segment as anchored to the trimmed line; require `|>>|` for subtree output.
- Keep dialect-specific parsing isolated; once lines are normalised, behaviour must be identical across vendors.
- Prefer deterministic transforms. Anonymisation must map inputs to consistent replacements within a run.
- Default to zero stderr output. Communicate failure via exit codes and let callers decide how to surface errors.
- Tests first. Every new capability needs coverage (unit + integration) exercising each flag combination.
- Clippy pedantic/nursery and `cargo fmt` must pass before landing changes.
- Keep public APIs minimal; expose enums or types only when they communicate intent.
- When in doubt, document assumptions in code comments, but avoid restating the obvious.
- Any deferred work, ignored issue, or stub must carry a `TODO(code): <one line description>` comment so follow-ups are easy to track.
- Plan ahead. Update the roadmap whenever scope shifts so future work stays visible.
