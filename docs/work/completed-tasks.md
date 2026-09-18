# Completed Tasks Log (Append-Only)

## T-001 (sprint 0)

- **Intent:** [INT-0001](../intents/INT-0001-markdown-repo-context-bundle.md)
- **Completed:** 2026-09-17
- **Touched:** `Cargo.toml`, `src/main.rs`
- **Summary:** Cargo skeleton with an empty `[dependencies]` table, the shared
  `Options`/`Wrap`/`Sink` types the later tasks consume, hand-written argument
  parsing over an iterator, and the help text documenting the four pattern
  forms and both caveats. `cargo clippy -D warnings` clean; 9 unit tests pass.
- **Commit:** `01ce91a8e4330d52359c2483b9c2450f40a83852`

## T-002 (sprint 0)

- **Intent:** [INT-0001](../intents/INT-0001-markdown-repo-context-bundle.md)
- **Completed:** 2026-09-17
- **Touched:** `src/main.rs`
- **Summary:** Single symlink-safe walk building the shared `Node` tree, with
  the four pattern forms and include-outranks-ignore resolution. Entries are
  sorted per level for byte-identical output; `DirEntry::file_type` classifies
  without following links; an unlistable directory is marked and traversal
  continues. 11 new unit tests. On this Windows host the symlink test ran and
  passed; the unlistable-directory test skipped with its reason, as planned,
  and the Linux CI leg is its authoritative run.
- **Commit:** `3697fc1dd56670cf7fa10ccba6c562206107568c`

## T-003 (sprint 0)

- **Intent:** [INT-0001](../intents/INT-0001-markdown-repo-context-bundle.md)
- **Completed:** 2026-09-17
- **Touched:** `src/main.rs`
- **Summary:** Scaffold renderer over the shared `Node` tree using the exact
  box-drawing symbols from `Scaffolding symbols generator.md`, with prefix
  composition (`│   ` past a non-last entry, four spaces past a last one),
  trailing `/` on directories, an `[unreadable]` marker, and all three wrap
  modes. 7 new unit tests, including the `none` case that asserts no backticks
  leak onto any line.
- **Commit:** `471366a4c21e821d02d54bda37871c1f2be1e370`

## T-004 (sprint 0)

- **Intent:** [INT-0001](../intents/INT-0001-markdown-repo-context-bundle.md)
- **Completed:** 2026-09-17
- **Touched:** `src/main.rs`
- **Summary:** Content renderer emitting the inherited `---` / `File: <path>` /
  `---` header preceded by the blank line the awk one-liner produces, adaptive
  fence length (`max(3, longest_backtick_run + 1)`), a small extension-to-
  language table, and in-place markers for non-UTF-8 and unreadable files.
  Also the composed both-halves property this task owns. 14 new tests,
  including the two integration tests proving an omitted entry is absent from
  the scaffold *and* the contents, and a re-admitted one present in both.
- **Commit:** PENDING
