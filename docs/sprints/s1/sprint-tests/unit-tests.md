# Sprint 1 Unit Test Results

- **Tested head:** `0ff9f2a9785b0acdd5f152c6e5d019e5506e5276` (branch `dev`)
- **Runner:** `cargo test --all -- --nocapture`
- **Host:** Windows 11, `cargo 1.96.0` / `rustc 1.96.0`, `git 2.54.0.windows.1`. CI runs Rust 1.98.0 and git 2.55.0; see the test report.
- **Result:** 68 passed, 0 failed, 0 ignored. That is the 44 sprint 0 unit tests, which ran **unedited**, plus 24 new ones. Two tests skipped one case each, printing a reason; see Platform-gated.
- **Gates:** `cargo fmt --check` clean; `cargo clippy --all-targets -D warnings` clean; `cargo tree` reports `mdeezl v0.1.0` and nothing else.

## T-001 — off switch

| Test | EARS clause verified |
|------|----------------------|
| `test_args_gitignore_on_by_default` | no arguments → `use_gitignore` is true |
| `test_args_no_gitignore` | `--no-gitignore` → false, with every other option still at its default |

## T-002 — the batched query

| Test | EARS clause verified |
|------|----------------------|
| `test_nul_payload` | Checks the `./` prefix, bracket-class escaping, and NUL termination. The expected backslash class is written in a different notation from the implementation, with its length asserted as 4. The test also asserts that no backslash appears outside that class. |
| `test_parse_records_maps_by_position` | Four records — tracked (empty fields), match, negation, no match — map to `[false, true, false, false]`. A record count one short is `Err`, never a shifted mapping. |
| `test_gitignore_candidates_root_repo_and_nested_repo` | The scan root holds `.git` and is still walked in full. Neither a nested repository's contents are sent — whether its `.git` is a gitlink **file** or a **directory**, as in an ordinary clone — nor any empty entry. |
| `test_git_ignored_returns_ignored_set` | Returns exactly the set git reports, with no `.` for a root that is not ignored. |
| `test_git_ignored_nothing_ignored_is_empty` | Git exits 1 → `Ok` holding an empty set. |
| `test_git_ignored_full_syntax` | Negation, `**`, a nested `.gitignore`, `.git/info/exclude`, and repo-local `core.excludesFile`, all checked at T-002's own boundary. |
| `test_git_ignored_subdir_root` | Root-level rules still apply when the scan root is a subdirectory. |
| `test_git_ignored_reports_root_only_when_ignored` | `.` is reported for an ignored root holding only untracked files, and **not** for one holding a tracked file. |
| `test_git_ignored_glob_chars_are_literal` | A bracketed untracked name is reported. Not reported: the Windows-sensitive tracked `x[1].log`, the unrelated `out[1].txt`, and the tracked `pages/[id].tsx`. |
| `test_git_ignored_whitelist_reports_escaped_directory` | Under `*` / `!*/` / `!*.tsx`, git reports `app/[slug]` while its tracked `page.tsx` is not reported. That is **the directory-removal guard's precondition**, pinned against whichever git runs the tests. Planning measured it on git 2.54; CI runs 2.55. |
| `test_git_ignored_submodule_candidates_do_not_abort` | A real submodule (committed source, `protocol.file.allow=always`) does not abort the query. |
| `test_git_ignored_outside_work_tree_errs` | `Err`, naming the repository problem. |
| `test_git_ignored_tracked_file_not_reported` | A staged file matching `*.log` is not reported; an untracked one beside it is. |
| `test_git_ignored_large_list_no_deadlock` | 20,000 synthetic paths of **60 bytes each** (asserted). The inbound payload is asserted to exceed **1.2 MB**. Outbound is larger still, because each record repeats the path beside its matching rule. All paths return under a 60 s `recv_timeout`. |
| `test_git_ignored_large_list_outside_work_tree_errs_without_panic` | The same list against a plain directory: git exits during setup, the writer meets a broken pipe, and the result is `Err`, not a panic. |
| `test_git_local_env_vars_cover_gits_list` | Every variable `git rev-parse --local-env-vars` lists on the running git appears in the removal list. A git that adds one will fail here, instead of letting it silently redirect the query. The test also asserts the list names `GIT_DIR` and `GIT_INDEX_FILE`, so empty output cannot pass it vacuously. |
| `test_run_spawns_git_exactly_once` | The `#[cfg(test)]` spawn counter reads exactly 1 after a full `run`. The test **also reads the document** that run wrote: the gitignored `b.log` is absent and the nested `src/d/e.rs` is present. The count alone would pass even if the query had failed, because `run` returns `Ok` then. |

## T-003 — pruning

The shared fixture uses the ignored set git actually returns, with every descendant of an ignored directory listed, rather than just the directory.

| Test | EARS clause verified |
|------|----------------------|
| `test_prune_gitignored_removes_subtree` | Each ignored directory goes with its subtree in one step. |
| `test_prune_gitignored_include_exempts_subtree` | Covers all four pattern forms. A name keeps `out/` whole. `*.o` keeps the top-level and `src/` object files, but `out/` still goes. A path keeps one file. `.*` keeps `.cache/` whole. |
| `test_prune_gitignored_include_of_unignored_dir_gives_no_exemption` | Including `src`, which git does not ignore, still leaves `src/gen.o` pruned. |
| `test_prune_gitignored_ancestor_blocks_include` | An included file under a pruned ancestor is still pruned. |
| `test_prune_gitignored_guard_keeps_unreported_descendant` | The reported `app/[slug]` is kept, because its tracked `page.tsx` was not reported. `notes.md` goes. `nested/` goes too, because its child was never queried. |

## Mutation check

The test critique found several assertions that could not fail. After tightening them, two regressions the critique named were introduced into the implementation deliberately, one at a time. The source was restored byte-identical after each, and each was caught:

| Mutation | Unit test | End-to-end test |
|----------|-----------|-----------------|
| `.git` detection regresses from `.exists()` to `.is_file()` | `test_gitignore_candidates_root_repo_and_nested_repo` **failed** | `test_cli_nested_repository_not_filtered` **failed** |
| `GIT_INDEX_FILE` dropped from the removal list | `test_git_local_env_vars_cover_gits_list` **failed** | `test_cli_ignores_ambient_git_environment` **failed**, at "the scan root's index applied" — the silent failure, where a tracked file is dropped with nothing on stderr |

The end-to-end results were run with `--no-fail-fast`. A plain `cargo test` stops after the first failing test binary, so without that flag the end-to-end tests would never have run.

## Platform-gated

Two tests skipped one case each on this Windows host, printing a SKIP. The Linux CI leg is the authoritative run for both, and the test report records it.

- **`test_unreadable_dir_marked_and_walk_continues`** (sprint 0): Windows' `set_permissions` cannot block `read_dir`.
- **The backslash case of `test_git_ignored_glob_chars_are_literal`:** Windows forbids `\` in filenames. This case is the only git-level check of the four-byte `[\\]` class.

## Isolation

Every in-process test that queries `git_ignored` is isolated. A `#[cfg(test)]` block in `git_ignored` sets these on the git **child**:

- `GIT_CONFIG_NOSYSTEM`
- `GIT_CONFIG_GLOBAL` (pointing at an empty file)
- `XDG_CONFIG_HOME`
- `GIT_CEILING_DIRECTORIES`

These match what the end-to-end harness sets. No process-wide `set_var` is needed.

One test spawns git outside that path: `test_git_local_env_vars_cover_gits_list` runs `git rev-parse --local-env-vars` directly. That command prints a fixed list compiled into git, independent of configuration and repository, so isolation would not change its result. The test also asserts the list contains `GIT_DIR` and `GIT_INDEX_FILE`, so empty output cannot pass it vacuously.

An earlier version of this record claimed the in-process tests *could not* be isolated without `set_var`. That was wrong; the critique pointed out the child-command approach. Two details matter:

- **`XDG_CONFIG_HOME` is necessary.** Git's default excludes file is `$XDG_CONFIG_HOME/git/ignore`, and `GIT_CONFIG_GLOBAL` alone does not disable it. This host has such a file.
- **`GIT_CEILING_DIRECTORIES` is necessary.** Without it, the two outside-a-work-tree tests would fail on any machine whose temp directory sits inside a repository.
