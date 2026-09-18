# Sprint 0 Unit Test Results

- **Tested head:** `2e48a1cb5e7380631092e2e74371f28792afb583` (branch `dev`)
- **Runner:** `cargo test --all -- --nocapture`
- **Host:** Windows 11, `cargo 1.96.0` / `rustc 1.96.0`
- **Result:** 44 passed, 0 failed, 0 ignored. 1 test self-skipped with a
  printed reason (see Platform-gated below).
- **Gates:** `cargo fmt --check` clean; `cargo clippy --all-targets -D warnings`
  clean; `cargo tree` reports `mdeezl v0.1.0` and nothing else.

Unit tests live in `src/main.rs` under `#[cfg(test)] mod tests`, which lets
them reach private functions directly.

## T-001 — argument parsing and manifest

| Test | EARS clause verified |
|------|----------------------|
| `test_args_defaults` | no arguments -> root `.`, stdout, `Fence`, the exact six-entry ignore list |
| `test_args_positional_root` | a bare positional argument becomes the scan root |
| `test_args_accumulates_exclude_and_include` | `--exclude`/`--include` repeat and retain declaration order |
| `test_args_rejects_unknown_wrap` | an unknown `--wrap` value is a usage error naming the value |
| `test_args_rejects_missing_flag_value` | all four of `-o`, `--wrap`, `--exclude`, `--include` error when the value is missing |
| `test_args_rejects_second_positional` | a second path is a usage error, not a silent drop |
| `test_args_help` | `-h` and `--help` both return the help request |
| `test_manifest_dependencies_table_is_empty` | `[dependencies]`, `[dev-dependencies]` and `[build-dependencies]` all hold no entry lines — the dev table is the only way a test suite could add a crate |
| `test_manifest_declares_bin_name_and_license` | bin target `mdeezl` (which `CARGO_BIN_EXE_mdeezl` resolves by) and `license = "Apache-2.0"` |

## T-002 — walk, patterns, ignore/include

| Test | EARS clause verified |
|------|----------------------|
| `test_pattern_dot_star` | `.*` matches dot-prefixed names only |
| `test_pattern_extension` | `*.png` matches by extension, not by prefix or substring |
| `test_pattern_relative_path` | a pattern containing `/` compares against the root-relative path |
| `test_pattern_exact_name` | any other pattern is an exact name match |
| `test_pattern_name_matches_at_depth` | name patterns are depth-independent |
| `test_include_outranks_ignore` | matching both ignore and include keeps the entry |
| `test_include_dot_star_readmits_git` | pins the `--include ".*"` re-admits `.git` consequence |
| `test_walk_sorts_children` | children are sorted by name, giving deterministic output |
| `test_walk_does_not_follow_symlink` | a symlink is recorded and never descended |
| `test_unreadable_dir_marked_and_walk_continues` | an unlistable directory is marked, traversal continues past it (Unix; see Platform-gated) |
| `test_walk_children_marks_unreadable_path` | host-independent proof of the same failure branch: `read_dir` on a regular file is an error, so the path is marked unreadable and yields no children |
| `test_walk_rejects_missing_root` | a missing root errors rather than yielding an empty tree |
| `test_walk_rejects_file_as_root` | the other half of the same clause: a root that exists but is a regular file leaves `walk` through a different branch, and errors with "not a directory" |

## T-003 — scaffold tree

| Test | EARS clause verified |
|------|----------------------|
| `test_tree_branch_symbols` | non-final entries use the branch symbol, the final one the last-branch symbol |
| `test_tree_prefix_composition` | continuation past a non-last entry, four spaces past a last one |
| `test_tree_directory_suffix` | directories carry a trailing `/`, files do not |
| `test_tree_unreadable_marker` | an unlistable directory carries the trailing marker |
| `test_tree_fence_wrap` | `fence` puts the tree in exactly one fenced block |
| `test_tree_inline_wrap` | `inline` wraps each line in single backticks, symbols untouched |
| `test_tree_none_wrap` | `none` emits no fence and no backticks on any line |

## T-004 — content sections

| Test | EARS clause verified |
|------|----------------------|
| `test_file_header_format` | the inherited `---` / `File: <path>` / `---` header, asserted as literal lines |
| `test_section_separator_blank_line` | a blank line precedes each header, so `---` never underlines the previous body line |
| `test_fence_len_plain_body` | no backticks -> 3 |
| `test_fence_len_body_with_triple_backticks` | a triple run -> 4 |
| `test_fence_len_body_with_quad_backticks` | a quad run -> 5 |
| `test_fence_len_counts_longest_run_not_total` | many single backticks -> still 3 |
| `test_body_fence_survives_inner_fence` | a body containing a fence opens with a longer one |
| `test_binary_body_elided` | non-UTF-8 -> byte-count marker, no raw bytes |
| `test_unreadable_file_body_marked` | a read error is rendered in place, carrying the error |
| `test_unreadable_file_does_not_abort_document` | the second half of that clause: a file removed after the walk recorded it is marked, **and the next file's section still renders**. Host-independent, so unlike the directory case it needs no skip |
| `test_language_hint_known_and_unknown` | known extensions map, unknown ones yield no info string and no error |
| `test_relative_path_uses_forward_slashes` | a real nested walk through `join_rel` produces `/`-separated `File:` paths with no `\` |

## T-005 — document assembly

| Test | EARS clause verified |
|------|----------------------|
| `test_document_assembles_in_order` | title, then `## Structure`, then `## Contents` |

## Platform-gated

`test_unreadable_dir_marked_and_walk_continues` **skipped on this Windows host**
and printed its reason. `std::fs::set_permissions` on Windows can only toggle
the read-only attribute, which does not block `read_dir`, and std exposes no ACL
API. The `ubuntu-latest` leg of the CI matrix is its authoritative run.

The skip decision probes `fs::read_dir(...).is_err()` rather than whether the
`chmod` call returned `Ok`. That distinction matters: running as root — the
default in many containers — the `chmod` succeeds while `read_dir` still works,
and a test keyed on the `chmod` result would have *failed* there instead of
skipping. Permissions are also restored immediately after the walk and before
any assertion, so a failing assertion cannot strand a `0o000` directory that
nothing can subsequently remove.

Because that test is Linux-authoritative and CI has not yet run, its EARS clause
would otherwise have no executed evidence anywhere.
`test_walk_children_marks_unreadable_path` closes **half** that gap, and only
half: it calls `walk_children` on a regular file, which `read_dir` rejects on
every platform, and asserts the unreadable marking and the empty child list. It
runs and passes here.

It does **not** cover the clause's second half — "SHALL continue traversing the
remainder of the tree" — because it never places an unreadable node inside a
tree and never checks that a sibling survives. That half is covered only by
`test_unreadable_dir_marked_and_walk_continues`, which skipped on this host, so
**it remains unexecuted anywhere pending the Linux leg of the CI matrix.**

`test_walk_does_not_follow_symlink` **ran and passed** on this host — Developer
Mode is enabled, so `symlink_dir` succeeded. It self-skips with a printed reason
where it is not, and the Linux leg is authoritative there too.

No test passes silently when its platform cannot exercise it: both print SKIP,
and CI runs with `--nocapture` so the line is visible in the log.
