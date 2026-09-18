Finalized - DO NOT EDIT

# Sprint 1 Test Plan

## Intent Traceability

One row per EARS clause, in build-plan order. Every clause appears once in the clause column; every named test below appears in the verification column. A test is listed under the task whose boundary can first verify it.

| Intent | Acceptance criterion | Build task / EARS clause | Verification |
|--------|----------------------|--------------------------|--------------|
| [INT-0004](../../../intents/INT-0004-gitignore-aware-exclusion.md) | On by default | T-001 / WHEN no arguments THEN `use_gitignore` SHALL be true | `test_args_gitignore_on_by_default` |
| [INT-0004](../../../intents/INT-0004-gitignore-aware-exclusion.md) | Can be switched off for a run | T-001 / WHEN `--no-gitignore` THEN `use_gitignore` SHALL be false, other options default | `test_args_no_gitignore` |
| [INT-0004](../../../intents/INT-0004-gitignore-aware-exclusion.md) | `--help` says where an omission came from | T-001 / WHEN `--help` THEN it SHALL name both sources, `--no-gitignore`, `PATH`, and the work tree | `test_cli_help_names_both_exclusion_sources` |
| [INT-0004](../../../intents/INT-0004-gitignore-aware-exclusion.md) | `./`-prefixed paths, glob characters bracket-class escaped | T-002 / WHEN `nul_payload` is given paths THEN each SHALL be `./`-prefixed, each of `*?[\` wrapped in a bracket class, no backslash escaping, NUL-terminated | `test_nul_payload`, `test_git_ignored_glob_chars_are_literal` |
| [INT-0004](../../../intents/INT-0004-gitignore-aware-exclusion.md) | Results mapped by position | T-002 / WHEN `parse_records` is given `-v -n` output THEN one boolean per path in order; count mismatch SHALL be `Err` | `test_parse_records_maps_by_position` |
| [INT-0004](../../../intents/INT-0004-gitignore-aware-exclusion.md) | No empty entry; not inside nested repositories or submodules other than the scan root | T-002 / WHEN `gitignore_candidates` collects THEN `.`, every other `rel`, no empty entry, nothing beneath a non-root directory holding `.git` | `test_gitignore_candidates_root_repo_and_nested_repo`, `test_git_ignored_submodule_candidates_do_not_abort` |
| [INT-0004](../../../intents/INT-0004-gitignore-aware-exclusion.md) | All of gitignore; one process; isolated from ambient git variables | T-002 / WHEN `git_ignored` is called in a work tree THEN it SHALL spawn once with local variables removed and return exactly the marked candidates | `test_git_ignored_returns_ignored_set`, `test_git_ignored_nothing_ignored_is_empty`, `test_git_ignored_full_syntax`, `test_git_ignored_subdir_root`, `test_git_ignored_reports_root_only_when_ignored`, `test_cli_ignores_ambient_git_environment` |
| [INT-0004](../../../intents/INT-0004-gitignore-aware-exclusion.md) | Degradation: git unavailable, not a work tree, or the query fails | T-002 / WHEN git cannot be spawned THEN `Err` "not found"; WHEN git exits other than 0/1 THEN `Err` with git's stderr | `test_git_ignored_outside_work_tree_errs`, `test_cli_missing_git_degrades_with_reason` |
| [INT-0004](../../../intents/INT-0004-gitignore-aware-exclusion.md) | A tracked file is included even when it matches | T-002 / WHEN a tracked file matches THEN it SHALL NOT be in the set | `test_git_ignored_tracked_file_not_reported` |
| [INT-0004](../../../intents/INT-0004-gitignore-aware-exclusion.md) | Survives a list larger than the pipe buffer | T-002 / WHEN the list exceeds the pipe buffer THEN no deadlock; write errors ignored; join without panicking | `test_git_ignored_large_list_no_deadlock`, `test_git_ignored_large_list_outside_work_tree_errs_without_panic` |
| [INT-0004](../../../intents/INT-0004-gitignore-aware-exclusion.md) | One subprocess per run; degrade with a reason; skip when git reports the root ignored | T-002 / WHEN `use_gitignore` THEN `run` SHALL call `git_ignored` once; WHEN `Err` or `.` reported THEN one stderr line, complete document, exit 0 | `test_run_spawns_git_exactly_once`, `test_cli_plain_directory_degrades_with_reason`, `test_cli_ignored_scan_root_degrades_with_reason` |
| [INT-0004](../../../intents/INT-0004-gitignore-aware-exclusion.md) | Switched off means git is not consulted | T-002 / WHEN `use_gitignore` is false THEN `run` SHALL NOT invoke git and SHALL write nothing to stderr | `test_cli_no_gitignore_does_not_invoke_git` |
| [INT-0004](../../../intents/INT-0004-gitignore-aware-exclusion.md) | Zero-dependency property preserved | T-002 / WHEN built THEN no dependencies; `cargo tree` reports only this crate | `test_manifest_dependencies_table_is_empty` (sprint 0, unedited) plus the `cargo tree` gate, whose literal output the test report records |
| [INT-0004](../../../intents/INT-0004-gitignore-aware-exclusion.md) | Gitignored paths omitted; a pruned directory takes its subtree | T-003 / WHEN a node is in the set and matches no include THEN removed with its subtree, subject to the guard | `test_prune_gitignored_removes_subtree` |
| [INT-0004](../../../intents/INT-0004-gitignore-aware-exclusion.md) | A reported directory is removed wholesale only when every queried entry beneath it is reported | T-003 / WHEN a directory is in the set but a queried entry beneath it is not THEN keep it and judge its children individually | `test_prune_gitignored_guard_keeps_unreported_descendant`, `test_cli_tracked_file_in_escaped_directory_is_kept` |
| [INT-0004](../../../intents/INT-0004-gitignore-aware-exclusion.md) | Including a directory git reports ignored exempts its subtree | T-003 / WHEN a node is in the set and matches an include pattern THEN kept, subtree exempt | `test_prune_gitignored_include_exempts_subtree`, `test_cli_include_beats_gitignore`, `test_cli_include_dot_star_with_gitignore` |
| [INT-0004](../../../intents/INT-0004-gitignore-aware-exclusion.md) | …but only when the included directory is itself gitignored | T-003 / WHEN a node matches include but is not in the set THEN kept without exemption | `test_prune_gitignored_include_of_unignored_dir_gives_no_exemption` |
| [INT-0004](../../../intents/INT-0004-gitignore-aware-exclusion.md) | An include cannot rescue an entry under a pruned ancestor | T-003 / WHEN an ancestor directory is pruned THEN every entry beneath it SHALL be pruned, include or not | `test_prune_gitignored_ancestor_blocks_include` |
| [INT-0004](../../../intents/INT-0004-gitignore-aware-exclusion.md) | Both halves governed identically; all of gitignore; tracked files kept; glob-literal names; no filtering inside submodules | T-003 / WHEN `git_ignored` returns `Ok` and the root is not reported ignored THEN `run` SHALL prune before rendering | `test_cli_gitignore_omits_from_both_halves`, `test_cli_gitignore_honours_full_syntax`, `test_cli_scan_subdir_applies_root_rules`, `test_cli_tracked_file_matching_pattern_is_bundled`, `test_cli_tracked_file_inside_ignored_directory`, `test_cli_no_gitignore_restores_paths`, `test_cli_submodule_contents_not_filtered`, `test_cli_glob_char_filename_is_literal`, `test_cli_colon_prefixed_filename_is_a_path`, `test_cli_newline_in_filename_is_matched`, `test_cli_large_repo_completes` |
| [INT-0004](../../../intents/INT-0004-gitignore-aware-exclusion.md) | Byte-identical for the same tree and git state | T-003 / WHEN run twice on an unchanged repository THEN byte-identical | `test_cli_gitignore_output_is_byte_identical_across_runs` |
| [INT-0004](../../../intents/INT-0004-gitignore-aware-exclusion.md) | The README says where an omission came from | T-004 / WHEN read THEN it SHALL document the nine listed items | `test_readme_documents_gitignore` |
| [INT-0004](../../../intents/INT-0004-gitignore-aware-exclusion.md) | The README no longer claims no support | T-004 / WHEN read THEN it SHALL NOT contain "No gitignore support yet" | `test_readme_documents_gitignore` |
| [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md) | All realized criteria — no regression | none; INT-0001 is constrained, not advanced | all 72 sprint 0 tests, which **must pass without being edited** |

Twenty-two EARS clauses, twenty-two clause rows. Forty-four new named tests plus one reused. Every test appears in exactly one row, with one deliberate exception: `test_readme_documents_gitignore` verifies both T-004 clauses — what the README says and what it no longer says — and so appears in both T-004 rows.

## Test environment isolation

Every test that creates a git fixture, and every test that runs the binary against one, goes through a helper that:

- **Clears git's repository variables.** It removes every variable listed by `git rev-parse --local-env-vars`, so a test run from inside a git hook cannot redirect git. The single exception is `test_cli_ignores_ambient_git_environment`, which sets them deliberately, because it tests that the *binary* removes them. The helper must not remove them first, or that test would prove nothing.
- **Isolates git configuration.** It sets `GIT_CONFIG_NOSYSTEM=1` and points `GIT_CONFIG_GLOBAL` at an empty file, so a developer's or runner's global `core.excludesFile` cannot change a result.
- **Stops repository discovery.** It sets `GIT_CEILING_DIRECTORIES` to the temp root, so a temp directory inside some work tree is never taken to be part of it.
- **Stages but never commits.** It stages with `git add -f`. Planning verified that a staged-only file already counts as tracked. Committing would need a git identity, and GitHub-hosted runners have none. The one exception is the submodule fixture, whose source repository must have a commit; it passes identity explicitly with `-c user.name=… -c user.email=…`.
- **Creates directories on disk.** It creates every directory that a directory-only pattern names. Planning verified that `out/` does not match a directory that does not exist.

The 72 sprint 0 tests cannot be edited. They build fixtures in plain temp directories. Those are not work trees on this host (verified) or on standard CI runners, so the tests take the degradation path and print the skip notice. None of them asserts an empty stderr on success; the two sprint 0 assertions on stderr both require it to be non-empty. So the notice cannot break them.

## Unit Tests

### T-001 unit tests
- **Intent:** [INT-0004](../../../intents/INT-0004-gitignore-aware-exclusion.md)
- `test_args_gitignore_on_by_default`: no arguments → `use_gitignore` is true.
- `test_args_no_gitignore`: `["--no-gitignore"]` → `use_gitignore` is false, and root, sink, wrap, ignore list, and include list all still equal their defaults.
- Stubs: none.

### T-002 unit tests
- **Intent:** [INT-0004](../../../intents/INT-0004-gitignore-aware-exclusion.md)
- `test_nul_payload`: the input `["a", "b/c", ":x", "p*[1]?\z"]` must produce, byte for byte, `./a`, `./b/c`, `./:x`, and `./p[*][[]1][?][\\]z`, each followed by one NUL. The assertion also checks that **no backslash appears except inside the `[\\]` class**. That is the property that keeps the payload safe on Windows, where git turns a bare `\` into `/`. The test is a pure string transform, so it runs on every platform, including the `\` case that only Unix filenames can reach.
- `test_parse_records_maps_by_position`: hand-built `-v -n` output for four inputs, one record each: tracked (all fields empty), matched `*.log`, matched the negation `!keep.log`, and no match. The result must be `[false, true, false, false]`. Separately, a record count one short of the input count → `Err`.
- `test_gitignore_candidates_root_repo_and_nested_repo`: a real temp tree whose **root holds a `.git` directory**, plus `a.txt`, `sub/b.txt`, and `nested/`, which holds a `.git` file and `nested/c.txt`. The result contains `.`, `a.txt`, `sub`, `sub/b.txt`, and `nested`. It does **not** contain `nested/c.txt`, and it contains no empty string. With the root holding `.git`, an implementation that mistook the scan root for a nested repository would collect only `.` and fail.
- `test_git_ignored_returns_ignored_set`: a fixture repository whose `.gitignore` holds `out/` and `*.log`, with `out/` created on disk. Querying `[".", "out", "out/a.o", "app.log", "keep.txt"]` returns exactly `{"out", "out/a.o", "app.log"}`. It contains no `.`, because this root is not ignored.
- `test_git_ignored_nothing_ignored_is_empty`: a repository with no `.gitignore` → `Ok` holding an empty set, not `Err`.
- `test_git_ignored_full_syntax`: the returned set covers each case a hand-rolled matcher would get wrong:
  - negation (`!keep.log`);
  - `**` (`**/generated/`);
  - a nested `.gitignore`;
  - `.git/info/exclude`;
  - a `core.excludesFile` set with `git config`, using a forward-slash absolute path.

  This verifies the syntax criterion at T-002's own boundary, before pruning exists.
- `test_git_ignored_subdir_root`: called with a root one level below the repository root, where only the repository-root `.gitignore` holds `*.log`. `x.log` is in the set and `keep.txt` is not.
- `test_git_ignored_reports_root_only_when_ignored`: `out/` is ignored and contains only untracked files. Called with `out/` as the root, the set contains `.`. With a force-added `out/keep.txt`, the same call does **not** contain `.`, `a.o` is in the set, and `keep.txt` is not. That second half is git's tracked-content rule, which the root-skip criterion now states.
- `test_git_ignored_glob_chars_are_literal`: runs on both platforms, because Windows permits `[` in filenames. `.gitignore` holds `*.log` and `out/`, with `out/` on disk. Four cases:
  - `a1.log` is staged and `a[1].log` is untracked. The set contains `a[1].log` and not `a1.log`. Unescaped, the glob would match the tracked `a1.log` and git would suppress it.
  - **A tracked `x[1].log` is not in the set.** This is the Windows-sensitive case: under backslash escaping, Git for Windows reads `./x\[1].log` as `x/[1].log` and reports the tracked file ignored.
  - **An untracked `out[1].txt`, beside the ignored `out/`, is not in the set.** Also Windows-sensitive: backslash escaping reads it as `out/[1].txt`, which the `out/` rule matches.
  - A tracked `pages/[id].tsx` is not in the set — the common framework case.
  - **Unix only** — Windows forbids `\` in filenames, so this case prints its SKIP reason there: a tracked `x\y.log` is not in the set. This is the one git-level check of the four-byte `\` class. The transform test above could share a transcription slip with the implementation; git cannot, so a wrong class here makes git miss the index entry and fail the case on the Linux CI leg.

  Both Windows-sensitive cases were measured to fail under backslash escaping and pass under bracket-class escaping on this host, so on the Windows CI leg these would catch a regression to the wrong scheme.
- `test_git_ignored_submodule_candidates_do_not_abort`: a repository with a real submodule `mod/` (source committed with explicit `-c` identity, added with `-c protocol.file.allow=always`), walked and collected, then queried → `Ok`. Without the candidate rule this query would exit 128.
- `test_git_ignored_outside_work_tree_errs`: a plain temp directory → `Err` whose text names the repository problem.
- `test_git_ignored_tracked_file_not_reported`: `tracked.log` staged with `git add -f` under a `*.log` rule is absent from the set, while an untracked `free.log` beside it is present.
- `test_git_ignored_large_list_no_deadlock`: in a fixture repository whose `.gitignore` holds the file glob `*.o`, query **20,000 synthetic paths of about 60 bytes**, each ending `.o`. For a file glob, check-ignore matches paths that do not exist, so no files are created. With four fields per record, that is over 1.2 MiB inbound and more outbound. That is far past any pipe buffer, in both directions, which is the combination that deadlocks a write-then-read implementation. The call runs on a spawned thread and the test waits with `recv_timeout(60 s)`, so a deadlock fails the test rather than hanging the suite. Asserts all 20,000 are returned.
- `test_git_ignored_large_list_outside_work_tree_errs_without_panic`: the same 20,000 paths against a plain directory. Git exits 128 during setup, before reading stdin, so the writer hits a broken pipe. Asserts `Err` and that no panic occurred.
- `test_run_spawns_git_exactly_once`: calls `run` directly with `Sink::File` against a fixture repository with several directories and files, then asserts that the `#[cfg(test)]` thread-local spawn counter is exactly 1.
- **Git-absent gate:** each test that needs git first runs `git --version`. If that fails, it prints `SKIP <name>: git not on PATH` and returns. Git is present on this host and on both CI runners, so none should skip there.
- Stubs: none. Fixtures are real repositories, and the existing `Tmp` guard removes them even when an assertion panics.

### T-003 unit tests
- **Intent:** [INT-0004](../../../intents/INT-0004-gitignore-aware-exclusion.md)
- **Shared fixture.** The constructed tree is `root/{out/{a.o, deep/b.o}, src/{gen.o, main.rs}, top.o, .cache/{c.bin}, keep.txt}`. The set is what git would actually return for the rules `out/`, `*.o`, `.cache/`, with every descendant of an ignored directory included: `{"out", "out/a.o", "out/deep", "out/deep/b.o", "src/gen.o", "top.o", ".cache", ".cache/c.bin"}`.
- `test_prune_gitignored_removes_subtree`: with no include, the result is exactly `root/{src/{main.rs}, keep.txt}`.
- `test_prune_gitignored_include_exempts_subtree`: one case per pattern form. Each case asserts the exact surviving tree.
  - `out` (exact name, in the set) → the whole `out/` subtree survives. `top.o` and `src/gen.o` are still pruned.
  - `*.o` (file type) → `top.o` and `src/gen.o` survive. `out/` is still pruned, together with both `.o` files inside it.
  - `src/gen.o` (a path) → `src/gen.o` survives, and `top.o` is pruned.
  - `.*` (in the set, via `.cache`) → `.cache/` and `.cache/c.bin` survive.
- `test_prune_gitignored_include_of_unignored_dir_gives_no_exemption`: include `src` → `src/` is kept (it was never in the set), but `src/gen.o` is **still pruned**. This pins the narrowed rule: including a directory git does not report ignored does not switch off the repository's rules inside it.
- `test_prune_gitignored_ancestor_blocks_include`: include `out/deep/b.o`, whose ancestors `out` and `out/deep` are ignored and not included → `out/deep/b.o` is **absent**.
- `test_prune_gitignored_guard_keeps_unreported_descendant`: a separate constructed tree, `root/{app/{[slug]/{page.tsx, notes.md}}, nested/{inner.txt}}`. The sets mirror what git returns for the rules `*`, `!*/`, `!*.tsx`, **followed by `nested/`**. The first three are the whitelist idiom measured in planning, which reports `app/[slug]`. The fourth is needed because `nested` is an ordinary name: `!*/` would re-include it on its own, and the nested-repository half of this test needs `nested` reported.
  - **Queried:** `.`, `app`, `app/[slug]`, `app/[slug]/page.tsx`, `app/[slug]/notes.md`, `nested`. `nested/inner.txt` is not queried, because `nested` holds a nested repository.
  - **Ignored:** `{"app/[slug]", "app/[slug]/notes.md", "nested"}`.

  The expected results:
  - `app/[slug]` is **kept**, because its queried child `page.tsx` is not in the set.
  - `page.tsx` is present and `notes.md` is pruned.
  - `nested/` is **pruned** with `inner.txt`. That child was never queried, so it does not trigger the guard — a nested repository inside an ignored directory is still removed.
- Stubs: the pruner takes a constructed `Node` tree and a constructed set, so these touch neither the filesystem nor git.

## Integration Tests

### Query + pruning + rendering
- **Intents:** [INT-0004](../../../intents/INT-0004-gitignore-aware-exclusion.md), [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md)
- `test_run_spawns_git_exactly_once` composes T-002's query with `run` in-crate.
- Pruning composed with both renderers is verified by `test_cli_gitignore_omits_from_both_halves`, which drives the real binary against a real repository. A separate in-crate composition test would re-verify the same path with less fidelity.

## End-to-End Tests

- **Status:** possible.
- **Fixtures.** Fixture repositories ignore only directories absent from the built-in list: `out/` and `generated/`, never `build/` or `target/`.

### Landing in T-001
- `test_cli_help_names_both_exclusion_sources`: `--help` names the ignore list, `.gitignore`, `--no-gitignore`, `PATH`, and the work-tree requirement.

### Landing in T-002 (verifiable before pruning exists)
- `test_cli_no_gitignore_does_not_invoke_git`: run in a plain directory, where the default run prints the skip notice.
  - With `--no-gitignore`, stderr is **empty**. That proves git was not consulted.
  - Its stdout is **byte-identical** to the default run's stdout. That proves the document equals what the built-in list alone produces.
- `test_cli_plain_directory_degrades_with_reason`: a plain directory.
  - Exits 0.
  - Stderr is exactly one line, beginning `mdeezl: gitignore filtering skipped:` and naming the repository problem.
  - Stdout's scaffold equals an expected literal block.
- `test_cli_missing_git_degrades_with_reason`: runs with `.env("PATH", "")`, and `current_dir` set to the fixture.
  - **Not** `env_remove("PATH")`: glibc then falls back to `/bin:/usr/bin` and finds git.
  - **Not** `env_clear()`: on Windows that also drops `SystemRoot`.
  - On Windows, std searches the application directory, System32, and the Windows directory before `PATH`. None of these hold git on a standard runner.
  - Asserts exit 0, a stderr line stating git was not found, and the expected scaffold.
- `test_cli_ignored_scan_root_degrades_with_reason`: in a fixture that ignores `out/`, which holds only untracked files, scan `out/` itself. Asserts the skip notice, and that every file in `out/` is present.
- `test_cli_ignores_ambient_git_environment`: runs the binary on a fixture repository with `GIT_DIR` and `GIT_WORK_TREE` pointed at a nonexistent path.
  - If the binary did not remove them, git would fail and the skip notice would appear.
  - Asserts exit 0 and an **empty** stderr.

### Landing in T-003 (require pruning)
- `test_cli_gitignore_omits_from_both_halves`: `.gitignore` holds `out/`, `*.log`, and `!keep.log`.
  - `out/` with its file, and `app.log`, are absent from the scaffold **and** from the contents.
  - `keep.log` and `keep.txt` are present in both.
- `test_cli_gitignore_honours_full_syntax`: the same five syntax cases as the unit test, verified end to end. Each target is absent, and each control is present.
- `test_cli_scan_subdir_applies_root_rules`: scan `repo/sub`. `sub/x.log` is absent and `sub/keep.txt` is present.
- `test_cli_tracked_file_matching_pattern_is_bundled`: a `.log` file staged under a `*.log` rule is present, body included.
- `test_cli_tracked_file_inside_ignored_directory`: `out/` is ignored; `out/keep.txt` is force-added and `out/a.o` is untracked.
  - Scanning the repository root: `out/keep.txt` is present and `out/a.o` is absent.
  - Scanning `out/` itself: **no** skip notice, `keep.txt` present, `a.o` absent.
- `test_cli_no_gitignore_restores_paths`: on the gitignore fixture, `--no-gitignore` restores every gitignored path.
- `test_cli_include_beats_gitignore`: `--include out` → `out/` **and** `out/a.o`, with its body, are present.
- `test_cli_include_dot_star_with_gitignore`: the fixture's `.gitignore` holds `.cache/`, and `.cache/c.bin` exists. With `--include ".*"`:
  - `.cache/` and `.cache/c.bin` are **present**. `.cache` is in the ignored set and matches the include, so this exercises the subtree-exemption clause end to end rather than only asserting the absence of a failure.
  - `.git` is present as before.
  - The run exits 0 with **no** skip notice.
- `test_cli_submodule_contents_not_filtered`: a real submodule `mod/` containing `x.log`, plus a top-level `top.log`, under `*.log`.
  - Exits 0 with no notice.
  - `top.log` is absent.
  - `mod/x.log` is present.
- `test_cli_tracked_file_in_escaped_directory_is_kept`: runs on both platforms. The fixture uses the whitelist idiom — `.gitignore` holds `*`, `!*/`, `!*.tsx`, `!.gitignore` — with a staged `app/[slug]/page.tsx` and an untracked `app/[slug]/notes.md`.
  - `app/[slug]/page.tsx` is **present**, body included.
  - `notes.md` is absent.

  Planning measured that git reports `app/[slug]` ignored here. Without the guard, the tracked file would be dropped.
- `test_cli_glob_char_filename_is_literal`: `a1.log` is staged and `a[1].log` is untracked, under `*.log`. `a[1].log` is absent and `a1.log` is present. Runs on both platforms.
- `test_cli_colon_prefixed_filename_is_a_path`: **Unix only**; prints its SKIP reason on Windows, which forbids `:`. `.gitignore` holds `secret` and `*.log`.
  - `:secret` is present, because as a path it does not match `secret`.
  - `:x.log` is absent.
- `test_cli_newline_in_filename_is_matched`: **Unix only**, for the same reason. A file whose name contains a newline, matched by `*.log`, is absent.
- `test_cli_large_repo_completes`: 5,000 real files under `data/`, half ending `.o`, with `*.o` ignored.
  - The document goes to `-o`, so the harness never has to drain a large stdout.
  - Deadline 60 s, after which the child is killed and the test fails.
  - This is an end-to-end smoke check. The deadlock property itself is pinned by the 1.2 MiB unit test.
- `test_cli_gitignore_output_is_byte_identical_across_runs`: two runs over the gitignore fixture both succeed, produce non-empty output, and are byte-identical.

### Landing in T-004
- `test_readme_documents_gitignore`: `README.md` documents:
  - `.gitignore` as an exclusion source;
  - `--no-gitignore`;
  - the git and work-tree requirement;
  - the degradation behaviour;
  - the tracked-file rule;
  - the directory-include exemption and its limit;
  - the ancestor rule;
  - the submodule limitation;
  - that names containing glob characters are matched only partially, including the case where an include cannot rescue untracked content in such a directory.

  It no longer contains "No gitignore support yet".
