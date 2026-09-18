# Sprint 1 Test Report

- **Verdict:** pass. All eighteen INT-0004 acceptance criteria are proved by executed tests or, in one case, by recorded inspection (see Intent verification).
- **Tested head:** `0ff9f2a9785b0acdd5f152c6e5d019e5506e5276` (branch `dev`).
- **Runner:** `cargo test --all -- --nocapture`, the project's canonical suite.
- **CI:** [run 35327617895](https://github.com/crussella0129/MDeezl/actions/runs/35327617895) passed on `ubuntu-latest` and `windows-latest`, with Rust 1.98.0 and git 2.55.0.
- **Local host:** Windows 11, Rust 1.96.0, git 2.54.0.
- **Critique:** [critique.md](critique.md). The final verdict is `proceed-with-caveats` after three rounds; round one returned `block`.

## Results

| Suite | Local (Windows) | `ubuntu-latest` | `windows-latest` | Detail |
|-------|-----------------|-----------------|------------------|--------|
| Unit | 68 passed | 68 passed | 68 passed | [unit-tests.md](unit-tests.md) |
| Integration | passed | passed | passed | [integration-tests.md](integration-tests.md) |
| End-to-end | 51 passed | 51 passed | 51 passed | [e2e-tests.md](e2e-tests.md) |

That is 119 tests. **All 72 sprint 0 tests ran unedited and passed**, which was the plan's regression bar for INT-0001.

Four cases skip on Windows, each printing its reason. On `ubuntu-latest` all four ran and passed, and that log shows **no SKIP lines**:

- the colon-prefixed filename;
- the newline in a filename;
- the backslash class;
- the unreadable directory.

## Gates

| Gate | Result |
|------|--------|
| `cargo fmt --check` | clean |
| `cargo clippy --all-targets -- -D warnings` | clean, locally on 1.96.0 and in CI on 1.98.0 |
| `cargo tree` | exactly `mdeezl v0.1.0 (C:\Users\charl\MDeezl)` and nothing else |

The literal `cargo tree` output is recorded above because it is a toolchain query rather than a property of the code. `test_manifest_dependencies_table_is_empty`, a sprint 0 test that ran unedited, is its automated counterpart.

## Intent verification

The locked build plan's header says INT-0004 has "twenty" acceptance criteria. **That is a miscount: the chapter has eighteen.** The plan is locked and cannot be corrected, so the count is recorded here. Every one of the eighteen is mapped below.

| # | INT-0004 acceptance criterion | Verified by |
|---|------------------------------|-------------|
| 1 | Ignored paths are omitted in addition to the built-in list, and `--include` outranks both | `test_cli_gitignore_omits_from_both_halves` (both sources, on a run where git answered); `test_cli_include_beats_gitignore` |
| 2 | Including a directory that git reports ignored exempts its subtree, and only then | `test_prune_gitignored_include_exempts_subtree`; `test_prune_gitignored_include_of_unignored_dir_gives_no_exemption`; `test_cli_include_beats_gitignore`; `test_cli_include_dot_star_with_gitignore` |
| 3 | An include cannot rescue an entry whose ancestor directory is pruned | `test_prune_gitignored_ancestor_blocks_include`; the `*.o` case of `test_prune_gitignored_include_exempts_subtree` |
| 4 | The scaffold and the contents are governed identically; a pruned directory takes its subtree with it | `test_cli_gitignore_omits_from_both_halves`; `test_prune_gitignored_removes_subtree`; `test_run_spawns_git_exactly_once` (reads the rendered document) |
| 5 | Gitignore filtering can be switched off for a run | `test_args_no_gitignore`; `test_cli_no_gitignore_restores_paths`; `test_cli_no_gitignore_does_not_invoke_git` |
| 6 | The supported syntax is all of gitignore | `test_git_ignored_full_syntax`; `test_cli_gitignore_honours_full_syntax` |
| 7 | A tracked file is kept even when it matches a rule | `test_git_ignored_tracked_file_not_reported`; `test_cli_tracked_file_matching_pattern_is_bundled`; `test_cli_tracked_file_inside_ignored_directory` |
| 8 | The directory-removal guard keeps tracked content under escaped names | `test_prune_gitignored_guard_keeps_unreported_descendant`; `test_git_ignored_whitelist_reports_escaped_directory` (its precondition, **on git 2.55.0 in CI**); `test_cli_tracked_file_in_escaped_directory_is_kept` |
| 9 | Degrades when git is missing, the root is not a work tree, or the query fails | `test_git_ignored_outside_work_tree_errs`; `test_cli_plain_directory_degrades_with_reason`; `test_cli_missing_git_degrades_with_reason`; `test_git_ignored_large_list_outside_work_tree_errs_without_panic` |
| 10 | When git reports the scan root itself ignored, filtering is skipped with a notice | `test_git_ignored_reports_root_only_when_ignored`; `test_cli_ignored_scan_root_degrades_with_reason`; `test_cli_tracked_file_inside_ignored_directory` (no skip when the root holds tracked files) |
| 11 | Nested repositories and submodules are not filtered | `test_gitignore_candidates_root_repo_and_nested_repo` (gitlink file **and** `.git` directory); `test_git_ignored_submodule_candidates_do_not_abort`; `test_cli_submodule_contents_not_filtered`; `test_cli_nested_repository_not_filtered` |
| 12 | Paths are `./`-prefixed and escaped with bracket classes | `test_nul_payload`; `test_git_ignored_glob_chars_are_literal` (including the Windows-sensitive cases, and the backslash case on Linux); `test_cli_glob_char_filename_is_literal`; `test_cli_colon_prefixed_filename_is_a_path` (Linux) |
| 13 | Results are mapped to paths by position | `test_parse_records_maps_by_position`; `test_cli_newline_in_filename_is_matched` (Linux) |
| 14 | The query always concerns the scan root, never an ambient repository | `test_cli_ignores_ambient_git_environment` (`GIT_DIR`, `GIT_WORK_TREE`, and the silently failing `GIT_INDEX_FILE`); `test_git_local_env_vars_cover_gits_list` (**on git 2.55.0 in CI**); `test_git_ignored_subdir_root`; `test_cli_scan_subdir_applies_root_rules` |
| 15 | Output is byte-identical for the same tree and the same git state | `test_cli_gitignore_output_is_byte_identical_across_runs` (on a run where git answered) |
| 16 | INT-0004 is the authority for the second source; INT-0001's single-list criterion describes the built-in mechanism | **Recorded inspection, not a test.** This criterion allocates authority between documents and has no runtime behaviour to execute. Inspection found that the README's "What is omitted" section names both sources, with `--include` outranking both, and that `--help` does the same. `test_readme_documents_gitignore` and `test_cli_help_names_both_exclusion_sources` guard that the text stays present. INT-0001's transition history records the reading. |
| 17 | The zero-dependency property is preserved | the `cargo tree` gate above; `test_manifest_dependencies_table_is_empty` |
| 18 | One git subprocess per run | `test_run_spawns_git_exactly_once` (spawn counter reads exactly 1); `test_git_ignored_large_list_no_deadlock` (20,000 paths through one process) |

## Self-check

The self-check ran `mdeezl` against this repository.

- **Plain run:** exit 0 in 0.2 s, with an empty stderr, so the query ran against a real work tree.
- **Probe run:** an untracked `.tmp` probe was omitted by the repository's own `*.tmp` rule, and a `.txt` probe was kept. `--no-gitignore` restored both.

## Findings

- **Toolchain drift between this host and CI.** CI uses the runner's floating stable Rust, 1.98.0; this host has 1.96.0. The first push failed at clippy because 1.98's `byte_char_slices` also flags an array literal that 1.96 accepts. Consequence: local `clippy -D warnings` is not a reliable predictor of CI. Pinning a toolchain with `rust-toolchain.toml` would remove the drift, at the cost of CI no longer tracking new lints. That is a project decision, so it is recorded here rather than made in this sprint.
- **Git version drift.** CI runs git 2.55.0, while planning measured every git behaviour on 2.54.0. The two behaviours most likely to shift are now pinned by tests that run on whatever git executes them, and both passed on 2.55.0:
  - the escaped-directory report that the directory-removal guard depends on;
  - the repository-local environment list.
- **Updater intake crossed the boundary unfetched.** Dependabot PR #2 was merged into `dev` 46 seconds before sprint 1 initialized, which is correct boundary intake. It was not fetched locally until the first sprint 1 push was rejected. It was then merged in, not rebased, so the commit SHAs already recorded as evidence stay valid; all of them were confirmed reachable. It touched only the CI workflow's `actions/checkout` version.
- **The in-process test isolation was initially claimed impossible.** It was not: a `#[cfg(test)]` block on the git child command isolates it with no process-wide `set_var`. That block also sets `XDG_CONFIG_HOME`, which the locked plan's isolation list omitted. Git's default excludes file lives there, and `GIT_CONFIG_GLOBAL` does not disable it.

## Notable defects caught before release

Across five plan-critique rounds and three test-critique rounds, the following were caught before they could ship:

- **Gitignore would have been skipped on every run.** The root node's empty path made git exit 128.
- **`mdeezl .` would have filtered nothing.** The nested-repository rule would have mistaken the scan root for a nested repository.
- **`--include out` would have restored an empty `out/`.** Git reports every descendant of an ignored directory, and the include check covered only the directory itself.
- **One submodule would have aborted the whole query.** A path inside a submodule makes git exit 128 for the entire batch.
- **Git for Windows would have dropped tracked files.** It turns a backslash escape into a path separator; this was measured reporting a tracked `x[1].log` as ignored.
- **The whitelist idiom would have dropped a tracked file.** An escaped directory name defeats git's tracked-content check, so under that idiom it would have lost a tracked `app/[slug]/page.tsx`.

The test critique found no implementation defect. It found tests that would have passed through real regressions, and evidence that claimed more than it showed.
