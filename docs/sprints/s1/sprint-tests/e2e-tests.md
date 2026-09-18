# Sprint 1 End-to-End Test Results

- **Status:** possible. All end-to-end coverage the test plan promised is implemented.
- **Tested head:** `b1755d6ad6bf49ed8c09ba9eafa5ba8dc59c1976` (branch `dev`)
- **Runner:** `cargo test --all -- --nocapture`
- **Local result:** 50 passed, 0 failed, 0 ignored, on Windows 11. The 50 are the 28 sprint 0 end-to-end tests, which ran **unedited**, plus 22 new ones. Two of the new tests printed a SKIP here; see Linux-only.
- **CI result:** passed on `ubuntu-latest` and `windows-latest`; see "CI confirmation".

These tests run the real binary against **real git repositories** that the tests create themselves with `git init`.

- **Isolation.** Every git fixture and every binary run goes through an isolation helper. It sets `GIT_CONFIG_NOSYSTEM`, `GIT_CONFIG_GLOBAL`, `XDG_CONFIG_HOME`, and `GIT_CEILING_DIRECTORIES`, and removes the fifteen repository-local git variables.
- **Staging.** Fixtures stage with `git add -f` and never commit, because CI runners have no git identity. The submodule source passes one explicitly.
- **Which directories fixtures ignore.** Fixtures ignore only directories absent from the built-in list, so every omission they observe is gitignore's doing.

## T-001

| Test | Verifies |
|------|----------|
| `test_cli_help_names_both_exclusion_sources` | `--help` names the ignore list, `.gitignore`, `--no-gitignore`, `PATH`, and the work tree |

## T-002 — verifiable before pruning existed

| Test | Verifies |
|------|----------|
| `test_cli_no_gitignore_does_not_invoke_git` | In a plain directory, the default run prints the skip notice. `--no-gitignore` prints nothing, which proves git was not consulted. The two stdouts are byte-identical. |
| `test_cli_plain_directory_degrades_with_reason` | exit 0; exactly one stderr line, starting `mdeezl: gitignore filtering skipped:` and naming "not a git repository"; the full expected scaffold |
| `test_cli_missing_git_degrades_with_reason` | With `PATH` set to empty, the run exits 0, prints "git was not found", and produces the full expected scaffold. `env_remove` would let glibc find `/usr/bin/git`, and `env_clear` would drop `SystemRoot`, so neither is used. |
| `test_cli_ignored_scan_root_degrades_with_reason` | Scanning an ignored `out/` gives the skip notice with "itself ignored", and every file is present — not a title over an empty tree. |
| `test_cli_ignores_ambient_git_environment` | `GIT_DIR` and `GIT_WORK_TREE` point at a nonexistent path, and the run gets exit 0 with an **empty** stderr. That proves the binary removes them: left in place, `GIT_DIR` would override `-C` and git would fail. |

## T-003 — pruning

| Test | Verifies |
|------|----------|
| `test_cli_gitignore_omits_from_both_halves` | With `out/`, `*.log`, `!keep.log`: ignored entries are absent from the scaffold and the contents; kept ones are present in both |
| `test_cli_gitignore_honours_full_syntax` | negation, `**/generated/`, a nested `.gitignore`, `.git/info/exclude`, and repo-local `core.excludesFile` (set with a forward-slash path), each end to end |
| `test_cli_scan_subdir_applies_root_rules` | scanning `repo/sub` still obeys the repository-root `*.log` |
| `test_cli_tracked_file_matching_pattern_is_bundled` | a staged `tracked.log` under `*.log` is present, body included |
| `test_cli_tracked_file_inside_ignored_directory` | A tracked `out/keep.txt` under an ignored `out/` is kept, and the untracked `out/a.o` goes. Scanning `out/` directly gives **no** skip notice, with the same result. |
| `test_cli_no_gitignore_restores_paths` | `--no-gitignore` restores every gitignored path |
| `test_cli_include_beats_gitignore` | `--include out` restores `out/` **and** `out/a.o` with its body; other rules still apply |
| `test_cli_include_dot_star_with_gitignore` | `--include ".*"` restores a gitignored `.cache/` and its contents, shows `.git`, and prints no skip notice |
| `test_cli_submodule_contents_not_filtered` | With a real submodule, the query survives (empty stderr). `top.log` goes, and `mod/x.log` stays. |
| `test_cli_tracked_file_in_escaped_directory_is_kept` | Under the whitelist idiom `*` / `!*/` / `!*.tsx`, the tracked `app/[slug]/page.tsx` is present, body included, and the untracked `notes.md` goes. This is the directory-removal guard end to end. |
| `test_cli_glob_char_filename_is_literal` | an untracked `a[1].log` goes and a tracked `a1.log` stays; runs on both platforms |
| `test_cli_colon_prefixed_filename_is_a_path` | **Linux only:** `:secret` is kept and `:x.log` goes |
| `test_cli_newline_in_filename_is_matched` | **Linux only:** a `.log` name containing a newline goes |
| `test_cli_large_repo_completes` | 5,000 real files, half ignored, written to `-o` under a 60 s deadline. Exactly 2,500 `File: data/` sections remain, and no `.o` file survives. |
| `test_cli_gitignore_output_is_byte_identical_across_runs` | two runs over the gitignore fixture succeed, are non-empty, and are identical |

## T-004

| Test | Verifies |
|------|----------|
| `test_readme_documents_gitignore` | All eleven documented items are present, and "No gitignore support yet" is gone. Two needles first failed — one on capitalisation, one assuming a line break the README does not have. The test was corrected; the README was not bent to fit. |

## Self-check against this repository

- **Plain run.** `mdeezl .` exits 0 in 0.2 s with an **empty** stderr, so the query ran against a real work tree. Its output is byte-identical to `--no-gitignore`. That is expected: this repository's `.gitignore` names nothing the built-in list does not already prune.
- **Probe run.** To show removal on the real repository, an untracked `selfcheck-probe.tmp` and `selfcheck-probe.txt` were created. The `.tmp` probe was omitted by the repo's own `*.tmp` rule and the `.txt` probe kept. `--no-gitignore` restored both. The probes were then deleted.

## Linux-only

Four cases printed a SKIP on this Windows host and are proved only by the `ubuntu-latest` CI leg:

- `test_cli_colon_prefixed_filename_is_a_path`
- `test_cli_newline_in_filename_is_matched`
- the backslash case of `test_git_ignored_glob_chars_are_literal`
- sprint 0's `test_unreadable_dir_marked_and_walk_continues`

## CI confirmation

Run <https://github.com/crussella0129/MDeezl/actions/runs/35325608285>, on the tested head, **passed on both legs**:

| Leg | Result | Linux-only cases |
|-----|--------|------------------|
| `rust (ubuntu-latest)` | 66 unit + 50 end-to-end, all passed | **ran and passed — no SKIP line in the log** |
| `rust (windows-latest)` | 66 unit + 50 end-to-end, all passed | printed their four SKIP reasons, as designed |

Checked per test in the log rather than inferred from the green badge. On `ubuntu-latest`, each of these four reports `ok` with no SKIP line:

- `test_cli_colon_prefixed_filename_is_a_path`
- `test_cli_newline_in_filename_is_matched`
- `test_git_ignored_glob_chars_are_literal`, including its backslash case, the one git-level check of the four-byte `[\\]` class
- `test_unreadable_dir_marked_and_walk_continues`

So the colon-pathspec, newline-protocol, and backslash-class claims are verified at git level on Linux, not just asserted.

**This head is the second attempt.** The first push, `d286b96`, failed CI at the clippy step on both legs, so no tests ran. The cause was toolchain drift: the runners use Rust 1.98.0, whose clippy extends `byte_char_slices` to an array literal in `test_nul_payload` that the local 1.96.0 accepts. The literal duplicated a variable defined a few lines above, and reusing it fixed the lint. The test report records the drift as a finding.
