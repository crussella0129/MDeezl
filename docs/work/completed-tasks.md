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
- **Commit:** `1bf6c31ed8f1b22c765f61022f23e8700375d38c`

## T-005 (sprint 0)

- **Intent:** [INT-0001](../intents/INT-0001-markdown-repo-context-bundle.md)
- **Completed:** 2026-09-17
- **Touched:** `src/main.rs`, `tests/cli.rs`, `Cargo.lock`
- **Summary:** Document assembled in memory, then written through one
  `BufWriter` over stdout or the `-o` file, so a failure cannot leave a partial
  document on stdout. Exit 0/2/1 for success, usage error, and I/O failure.
  Created `tests/cli.rs`, the integration target carrying all 23 end-to-end
  tests. Self-check: running `mdeezl .` against this repository produces a
  140 KB bundle whose fences balance, with a 5-backtick opening fence where a
  sprint plan contains a 4-backtick run — the case a fixed fence would corrupt.
- **Commit:** `b87a0eaa7a83419563997a5d2850d8c81fc9a06e`

## T-006 (sprint 0)

- **Intent:** [INT-0001](../intents/INT-0001-markdown-repo-context-bundle.md)
- **Completed:** 2026-09-17
- **Touched:** `.github/workflows/sprint-loops-ci.yml`, `tests/cli.rs`
- **Summary:** Re-ran the bundle's `scaffold-ci.sh` now that `Cargo.toml`
  exists — substrate convergence had generated nothing, because it ran before
  any Rust did. The generated workflow was Linux-only, so it now runs a
  `ubuntu-latest` + `windows-latest` matrix with `fail-fast: false`, and
  `cargo test -- --nocapture` so a platform-gated SKIP is visible in the log
  rather than passing silently. `test_ci_workflow_runs_tests_on_both_platforms`
  asserts all three properties.
- **Commit:** `0c4f43294b6449fb45cb0e40d284d02c21fb267b`

## T-007 (sprint 0)

- **Intent:** [INT-0001](../intents/INT-0001-markdown-repo-context-bundle.md)
- **Completed:** 2026-09-17
- **Touched:** `README.md`, `tests/cli.rs`
- **Summary:** Rewrote `README.md` — previously one line of framing with no
  usage — to document every flag, all four pattern forms, the pre-populated
  ignore list, the three wrap modes, exit codes, and the degraded cases, plus
  the deferred INT-0002/INT-0003 scope. `test_readme_documents_cli_surface`
  asserts all four elements the criterion names. The worked example is fenced
  with four backticks so its inner fences render, which is the same technique
  the tool applies to file bodies.
- **Commit:** `a5d03e4eb46380f5e36043066501a42c9c3c72c3`
