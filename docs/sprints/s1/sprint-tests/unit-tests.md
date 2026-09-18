# Sprint 1 Unit Test Results

- **Tested head:** `b1755d6ad6bf49ed8c09ba9eafa5ba8dc59c1976` (branch `dev`)
- **Runner:** `cargo test --all -- --nocapture`
- **Host:** Windows 11, `cargo 1.96.0` / `rustc 1.96.0`, `git 2.54.0.windows.1`
- **Result:** 66 passed, 0 failed, 0 ignored. The 66 include the 44 sprint 0 unit tests, which ran **unedited**, and 22 new ones. Two tests self-skipped a case with a printed reason; see Platform-gated.
- **Gates:** `cargo fmt --check` clean; `cargo clippy --all-targets -D warnings` clean; `cargo tree` reports `mdeezl v0.1.0` and nothing else.

## T-001 — off switch

| Test | EARS clause verified |
|------|----------------------|
| `test_args_gitignore_on_by_default` | no arguments → `use_gitignore` true |
| `test_args_no_gitignore` | `--no-gitignore` → false, every other option still at its default |

## T-002 — the batched query

| Test | EARS clause verified |
|------|----------------------|
| `test_nul_payload` | `./` prefix, bracket-class escaping, NUL termination. The expected backslash class is built in a different notation from the implementation, with its length asserted as 4, and the test asserts no backslash appears outside that class. |
| `test_parse_records_maps_by_position` | one boolean per record in input order — tracked (empty fields), a match, a negation, and no match give `[false, true, false, false]`. A record count one short is `Err`, never a shifted mapping. |
| `test_gitignore_candidates_root_repo_and_nested_repo` | the scan root holds `.git` and is still walked in full. The contents of a nested repository are not sent, and no entry is empty. |
| `test_git_ignored_returns_ignored_set` | exactly the set git reports; `.` absent for a root that is not ignored |
| `test_git_ignored_nothing_ignored_is_empty` | exit 1 → `Ok` holding an empty set |
| `test_git_ignored_full_syntax` | negation, `**`, a nested `.gitignore`, `.git/info/exclude`, and repo-local `core.excludesFile`, at T-002's own boundary |
| `test_git_ignored_subdir_root` | root-level rules apply when the scan root is a subdirectory |
| `test_git_ignored_reports_root_only_when_ignored` | `.` reported for an ignored root holding only untracked files, and **not** for one holding a tracked file |
| `test_git_ignored_glob_chars_are_literal` | a bracketed untracked name is reported. The Windows-sensitive tracked `x[1].log` and the unrelated `out[1].txt` are **not**, and neither is the tracked `pages/[id].tsx`. |
| `test_git_ignored_submodule_candidates_do_not_abort` | a real submodule (committed source, `protocol.file.allow=always`) does not abort the query |
| `test_git_ignored_outside_work_tree_errs` | `Err` naming the repository problem |
| `test_git_ignored_tracked_file_not_reported` | a staged file matching `*.log` is not reported; an untracked one beside it is |
| `test_git_ignored_large_list_no_deadlock` | 20,000 synthetic paths of about 60 bytes — over 1.2 MiB each way — all returned under a 60 s `recv_timeout`. The whole unit suite ran in about 1.6 s. |
| `test_git_ignored_large_list_outside_work_tree_errs_without_panic` | the same list against a plain directory: git exits during setup, the writer meets a broken pipe, and the result is `Err`, not a panic |
| `test_run_spawns_git_exactly_once` | the `#[cfg(test)]` spawn counter reads exactly 1 after a full `run` over a repository with nested directories |

## T-003 — pruning

The shared fixture uses the ignored set git actually returns — every descendant of an ignored directory listed — rather than a set containing only the directory. The plan critique found the latter had hidden a real defect.

| Test | EARS clause verified |
|------|----------------------|
| `test_prune_gitignored_removes_subtree` | each ignored directory goes with its subtree in one step |
| `test_prune_gitignored_include_exempts_subtree` | all four pattern forms: a name keeps `out/` whole; `*.o` keeps the top-level and `src/` object files while `out/` still goes; a path keeps one file; `.*` keeps `.cache/` whole |
| `test_prune_gitignored_include_of_unignored_dir_gives_no_exemption` | including `src`, which git does not ignore, leaves `src/gen.o` pruned |
| `test_prune_gitignored_ancestor_blocks_include` | an included file under a pruned ancestor is still pruned |
| `test_prune_gitignored_guard_keeps_unreported_descendant` | the reported `app/[slug]` is kept because its tracked `page.tsx` was not reported; `notes.md` goes; `nested/` goes because its child was never queried |

## Platform-gated

Two cases printed a SKIP on this Windows host. The Linux CI leg is authoritative for both; the test report records the run.

- **`test_unreadable_dir_marked_and_walk_continues`** (sprint 0): Windows' `set_permissions` cannot block `read_dir`.
- **The backslash case of `test_git_ignored_glob_chars_are_literal`**: Windows forbids `\` in filenames. This case is the only git-level check of the four-byte `[\\]` class. Every other case in the same test ran and passed here.

## Isolation limit of the in-process tests

The T-002 unit tests call `git_ignored` in-process, so they cannot isolate git from the host's global configuration. Doing that would need `std::env::set_var`, which is `unsafe` in edition 2024 and races with parallel tests.

Setting `GIT_CONFIG_GLOBAL` would not have been enough in any case. Git's default excludes file is `$XDG_CONFIG_HOME/git/ignore`, and `GIT_CONFIG_GLOBAL` does not disable it.

This host has such a file. It names only `.claude/settings.local.json`, so it cannot affect any fixture name. The end-to-end harness *is* fully isolated: it sets `XDG_CONFIG_HOME` as well as `GIT_CONFIG_GLOBAL` and `GIT_CONFIG_NOSYSTEM`. That goes beyond the locked plan's isolation list and was added because of this finding.
