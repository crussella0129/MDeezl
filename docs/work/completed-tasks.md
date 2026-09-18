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
- **Commit:** PENDING
