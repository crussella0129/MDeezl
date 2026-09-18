# Sprint 1 End-to-End Test Results

- **Status:** possible. All end-to-end coverage the test plan promised is implemented.
- **Tested head:** `1a4fb986939feba1ffd8dadcffaa5b97d174c5d1` (branch `dev`)
- **Runner:** `cargo test --all -- --nocapture`
- **Local result:** 51 passed, 0 failed, 0 ignored, on Windows 11. That is the 28 sprint 0 end-to-end tests, run **unedited**, plus 23 new. Two new tests printed a SKIP here; see Linux-only.
- **CI result:** passed on `ubuntu-latest` and `windows-latest` with git 2.55.0; see "CI confirmation".

These tests run the real binary against **real git repositories**, which the tests create with `git init`.

- **Isolation.** Every **sprint 1** fixture and binary run goes through an isolation helper. It sets `GIT_CONFIG_NOSYSTEM`, `GIT_CONFIG_GLOBAL`, `XDG_CONFIG_HOME` and `GIT_CEILING_DIRECTORIES`, and removes the fifteen repository-local git variables.
- **The sprint 0 tests are not isolated, and cannot be.** They must stay unedited, so they still run the binary without the helper, and now spawn git with the host's configuration. They pass because their plain temp directories are not work trees. That holds on this host (verified) and on standard CI runners, and the locked plan states it as an assumption. An earlier version of this record said *every* binary run was isolated. That was wrong.
- **Staging.** Fixtures stage with `git add -f` and never commit, because CI runners have no git identity. The submodule source passes one explicitly.
- **What fixtures ignore.** Fixtures ignore only directories absent from the built-in list, so any omission they see comes from gitignore.
- **Proving gitignore actually ran.** Tests that could otherwise pass with gitignore skipped assert an empty stderr, which rules out a skip notice. They also assert an ignored control is absent.

## T-001

| Test | Verifies |
|------|----------|
| `test_cli_help_names_both_exclusion_sources` | `--help` contains text that only the sprint 1 help has: `--include outranks both`, `git on your PATH`, `git work tree`, `.gitignore`, `--no-gitignore`. The sprint 0 help already said "PATH" and "the ignore list", so the earlier needles proved nothing. |

## T-002 — verifiable before pruning existed

| Test | Verifies |
|------|----------|
| `test_cli_no_gitignore_does_not_invoke_git` | In a plain directory, the default run prints the skip notice. `--no-gitignore` prints nothing, which proves git was not consulted. Both runs produce byte-identical stdout. |
| `test_cli_plain_directory_degrades_with_reason` | Exit 0, **exactly one** stderr line naming "not a git repository", and the full expected scaffold. |
| `test_cli_missing_git_degrades_with_reason` | Run with an empty `PATH`: exit 0, **exactly one** stderr line reading "git was not found", and the full scaffold. |
| `test_cli_ignored_scan_root_degrades_with_reason` | Scanning an ignored `out/`: **exactly one** stderr line reading "itself ignored", and every file is present. |
| `test_cli_ignores_ambient_git_environment` | Runs with `GIT_DIR`, `GIT_WORK_TREE` and `GIT_INDEX_FILE` all pointed at a nonexistent path. It checks exit 0, an empty stderr, that `app.log` is absent (so the scan root's rules applied), and that the staged `tracked.log` is **present** (so the scan root's index applied). If `GIT_INDEX_FILE` were not removed, the run would fail *silently*: the tracked file would be dropped with nothing on stderr. The mutation check confirms that this test catches it. |

## T-003 — pruning

| Test | Verifies |
|------|----------|
| `test_cli_gitignore_omits_from_both_halves` | Rules `out/`, `*.log`, `!keep.log`: ignored entries are absent from both the scaffold and the contents, and kept entries are present in both. |
| `test_cli_gitignore_honours_full_syntax` | Negation, `**/generated/`, a nested `.gitignore`, `.git/info/exclude`, and repo-local `core.excludesFile`, each checked end to end. |
| `test_cli_scan_subdir_applies_root_rules` | Scanning `repo/sub` still obeys the repository-root `*.log`. |
| `test_cli_tracked_file_matching_pattern_is_bundled` | A staged `tracked.log` is present, body included. An untracked `free.log` under the same rule is **absent**, and stderr is empty — so the rule really ran and the tracked file was kept anyway. |
| `test_cli_tracked_file_inside_ignored_directory` | A tracked `out/keep.txt` is kept and `out/a.o` goes. Scanning `out/` directly gives the same result with **no** skip notice. |
| `test_cli_no_gitignore_restores_paths` | `--no-gitignore` restores every gitignored path. |
| `test_cli_include_beats_gitignore` | `--include out` restores `out/` **and** `out/a.o` with its body; other rules still apply. |
| `test_cli_include_dot_star_with_gitignore` | `--include ".*"` restores a gitignored `.cache/` and its contents, and shows `.git`, with no skip notice. The gitignored non-dot `out/` is **still pruned** — a control proving pruning ran, so `.cache` survived because of the include. |
| `test_cli_submodule_contents_not_filtered` | With a real submodule, the query survives (empty stderr): `top.log` goes and `mod/x.log` stays. |
| `test_cli_nested_repository_not_filtered` | An ordinary nested clone, whose `.git` is a **directory** (asserted as a precondition): the enclosing `*.log` removes `top.log`, but not `inner/x.log`. Before the critique, every nested-repository fixture used a gitlink *file*, so an `.exists()` → `.is_file()` regression passed all 116 tests. The mutation check confirms this test now catches it. |
| `test_cli_tracked_file_in_escaped_directory_is_kept` | Under the whitelist idiom, the tracked `app/[slug]/page.tsx` is present with its body, and the untracked `notes.md` goes. This exercises the directory-removal guard end to end. The unit test `test_git_ignored_whitelist_reports_escaped_directory` pins the precondition on the running git. |
| `test_cli_glob_char_filename_is_literal` | An untracked `a[1].log` goes and a tracked `a1.log` stays. Runs on both platforms. |
| `test_cli_colon_prefixed_filename_is_a_path` | **Linux only:** `:secret` is kept and `:x.log` goes. |
| `test_cli_newline_in_filename_is_matched` | **Linux only:** a `.log` name containing a newline goes. |
| `test_cli_large_repo_completes` | 5,000 real files, half of them ignored, written to `-o` under a 60 s deadline. Exactly 2,500 `File: data/` sections remain. |
| `test_cli_gitignore_output_is_byte_identical_across_runs` | Two runs produce byte-identical output. The test also checks stderr is empty and `app.log` is absent, because two *degraded* runs would be identical too. |

## T-004

| Test | Verifies |
|------|----------|
| `test_readme_documents_gitignore` | All twelve documented items are present, including the case where an include cannot rescue untracked content, and "No gitignore support yet" is gone. |

## Self-check against this repository

- `mdeezl .` exits 0 in 0.2 s with an **empty** stderr, so the query ran against a real work tree. Its output is byte-identical to `--no-gitignore`, as expected: this repository's `.gitignore` names nothing the built-in list does not already prune.
- A probe run showed removal. Untracked `selfcheck-probe.tmp` and `selfcheck-probe.txt` files were created. The `.tmp` probe was omitted by the repository's own `*.tmp` rule and the `.txt` probe was kept. `--no-gitignore` restored both. The probes were then deleted.

## Linux-only

Four cases printed a SKIP on this Windows host. They are proved only by the `ubuntu-latest` CI leg:

- `test_cli_colon_prefixed_filename_is_a_path`
- `test_cli_newline_in_filename_is_matched`
- the backslash case of `test_git_ignored_glob_chars_are_literal`
- sprint 0's `test_unreadable_dir_marked_and_walk_continues`

## CI confirmation

Run <https://github.com/crussella0129/MDeezl/actions/runs/35326727838>, on the tested head, **passed on both legs**. Both runners used **git 2.55.0**; every behaviour this sprint relies on was measured in planning on 2.54.

| Leg | Result | Linux-only cases |
|-----|--------|------------------|
| `rust (ubuntu-latest)` | 68 unit + 51 end-to-end, all passed | **ran and passed, with no SKIP line in the log** |
| `rust (windows-latest)` | 68 unit + 51 end-to-end, all passed | printed their four SKIP reasons, as designed |

Each result below was checked per test in the log, not inferred from the green badge:

- **Linux-only cases.** On `ubuntu-latest`, the colon-pathspec, newline-name, backslash-class (`[\\]`) and unreadable-directory cases each report `ok`, with no SKIP.
- **Guard precondition.** `test_git_ignored_whitelist_reports_escaped_directory` passed on 2.55.0 on both legs. So git 2.55 still reports the escaped `app/[slug]` as ignored while leaving its tracked `page.tsx` unreported. The directory-removal guard is therefore exercised on the git CI runs, not just on the 2.54 it was designed against.
- **Environment list.** `test_git_local_env_vars_cover_gits_list` passed on 2.55.0: its `--local-env-vars` list is covered by the fifteen hard-coded names.

**Earlier attempts.**

- `d286b96` failed at clippy on toolchain drift (runner Rust 1.98.0, local 1.96.0).
- `b1755d6` passed with the pre-critique tests.
- This head is the first with the tightened tests.
