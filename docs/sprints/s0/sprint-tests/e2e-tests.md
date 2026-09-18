# Sprint 0 End-to-End Test Results

- **Status:** possible — all end-to-end coverage the test plan promised is
  implemented and passing locally. One criterion is **not** verified: see
  "Pending CI" at the foot of this file.
- **Tested head:** `2e48a1cb5e7380631092e2e74371f28792afb583` (branch `dev`)
- **Runner:** `cargo test --all -- --nocapture`
- **Result:** 28 passed, 0 failed, 0 ignored.

These run the real binary, located with `env!("CARGO_BIN_EXE_mdeezl")` — a
Cargo built-in for integration targets under `tests/`, which needs no library
target and adds no dependency. Each test builds its own fixture tree under a
unique temp directory and removes it afterwards.

The shared fixture holds, deliberately in one tree: nested directories, a
`target/` build directory, a `.secret` dot-file, a `.github/` directory, a
4-byte non-UTF-8 file, a `.log` file, a `.rs` file, two same-named `one.txt`
files at different depths, a text file whose last line is ordinary prose, and a
Markdown file containing a triple-backtick fence.

| Test | Verifies |
|------|----------|
| `test_cli_writes_document_to_stdout` | a Markdown document on stdout, exit 0, and no new file in the scanned directory |
| `test_cli_no_path_scans_current_directory` | invoked with no positional argument and `current_dir` set to the fixture, the title names that directory — the only test exercising the "no path" half of the criterion |
| `test_fences_balanced_detects_imbalance` | the fence checker itself rejects an unclosed fence and a too-short closer, so the balance assertion above is not vacuous |
| `test_cli_document_section_order` | title, `## Structure`, `## Contents` in that order, matched as exact lines |
| `test_cli_tree_shape` | the **whole** scaffold compared literally with `assert_eq!` against a nine-line expected block, so wrong ordering, a duplicated subtree, or a dropped entry all fail |
| `test_cli_file_header_format` | the exact `\n---\nFile: src/main.rs\n---\n` header |
| `test_cli_default_ignores_target_and_dotfiles` | `target/` and `.secret` absent with their contents absent, **and** positive controls that the non-ignored siblings and their bodies are still present — four negative assertions alone would pass on an empty document |
| `test_cli_include_readmits_dot_entry` | `--include .github` re-admits it while other dot-entries stay out |
| `test_cli_exclude_adds_pattern` | `--exclude *.log` drops the file type; siblings survive |
| `test_cli_exclude_single_file_by_path` | `--exclude sub/one.txt` drops exactly that file and leaves the root `one.txt` |
| `test_cli_binary_file_elided` | the elision marker appears and byte `0xff` never reaches stdout |
| `test_cli_markdown_body_does_not_close_its_own_fence` | a body holding a triple-backtick run opens with a four-backtick fence; a fence **state machine** confirms every fence is closed by a long-enough closer; the body's own fence lines survive inside the block; and the opening fence is asserted to be **strictly longer than the longest run inside the body** — the invariant balance alone cannot see, because a too-short opener closes early, re-opens on the next fence line, and still balances |
| `test_cli_fence_mode_tree_in_one_block` | the default mode wraps the scaffold in exactly one fenced block |
| `test_cli_rust_body_carries_language_hint` | a `.rs` body's fence carries `rust` |
| `test_cli_inline_mode_keeps_bodies_fenced` | `--wrap inline` backtick-wraps tree lines while bodies stay fenced |
| `test_cli_inline_mode_body_carries_language_hint` | the hint applies in `inline` too, the mode easy to overlook |
| `test_cli_wrap_none_has_no_fences` | `--wrap none` leaves no fence and no backtick in the tree, **and** a named body is asserted to follow its header with no fence — the tree slice alone cannot see bodies |
| `test_cli_wrap_none_preserves_section_separation` | in `none` mode a body's last line is followed by a blank line, so the next `---` is a thematic break and not a setext underline |
| `test_cli_output_flag_writes_file` | `-o` writes the file and leaves stdout empty |
| `test_cli_nested_paths_use_forward_slashes` | no `\` in any `File:` header, on this Windows host |
| `test_cli_output_is_byte_identical_across_runs` | two runs produce identical, **non-empty** bytes and both exited 0 — two equally-empty failures would otherwise compare equal |
| `test_cli_usage_error_exits_2` | exit 2, message on stderr, empty stdout |
| `test_cli_missing_root_exits_1_with_empty_stdout` | exit 1, no partial document on stdout |
| `test_cli_unwritable_output_exits_1` | the second exit-1 trigger: an unwritable `-o` target gives exit 1, not a panic's 101 |
| `test_cli_file_as_root_exits_1` | a regular file as the root gives exit 1 with a "not a directory" message and no document on stdout |
| `test_cli_help_documents_patterns_and_caveats` | `--help` names all four pattern forms and both caveats |
| `test_ci_workflow_runs_tests_on_both_platforms` | **a static read of the workflow file**: that it runs `cargo test` and names both runners. It proves the file says so, not that any run has happened — see "Pending CI" |
| `test_readme_documents_cli_surface` | `README.md` documents every flag, all four pattern forms, every pre-populated ignore entry, and all three wrap modes |

## Self-check against this repository

Beyond the fixtures, the binary was run against its own repository — the
strongest available end-to-end signal, because this repo is exactly the
backtick-heavy Markdown that motivates the adaptive fence.

- Output: 164,836 bytes.
- 28 fences opened, and a fence **state machine** confirms every one is closed
  by a closer at least as long as its opener.
- Longest opening fence: **5 backticks**, produced where the sprint plans
  contain four-backtick runs. A fixed three-backtick wrapper would have
  corrupted the document at exactly that point.
- `target/` and `.git/` absent from the scaffold, as the default ignore list
  requires.

An earlier version of this record claimed balance from an *even count of
fence-only lines*. That inference was invalid — the count mixed real delimiters
with the backtick lines inside Markdown bodies, and excluded every opening fence
carrying a language hint. Both the claim and the test that made it have been
replaced with the state machine above.

## Pending CI (resolved 2026-09-18)

**Resolved.** The checkpoint CI run confirmed this criterion; see the CI
confirmation section of [test-report.md](test-report.md). The original text is
kept below as the record of what was unverified at sprint close.

**The INT-0001 criterion "Verification runs on both Linux and Windows" was not
verified at sprint close.** Branch `dev` has never been pushed, so
`.github/workflows/sprint-loops-ci.yml` has never executed on any runner. The
only evidence is `test_ci_workflow_runs_tests_on_both_platforms`, a static read
of the YAML committed alongside it; it would pass just as happily on a workflow
that never triggers.

Two further items depend on that first run and are likewise unverified:

- the "continues traversing the remainder of the tree" half of the
  unreadable-directory clause, whose only full test skips on Windows;
- every result in this file on a non-Windows host — all local runs were on
  Windows 11.

This was recorded as a known gap rather than a pass. It closed on
2026-09-18 when the checkpoint opened and the matrix ran green on both runners.
