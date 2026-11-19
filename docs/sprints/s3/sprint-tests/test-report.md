# Sprint 3 Test Report

- **Verdict:** pass, with its gaps stated rather than claimed — they are
  listed under “What this phase cannot show”, and the two that touch an
  acceptance criterion are named here. Seven of
  [INT-0006](../../../intents/INT-0006-desktop-gui.md)'s eight acceptance
  criteria are proved by executed tests, CI logs or recorded observation. Of
  those seven, one half-case is proved a layer down rather than in the window:
  criterion 4 names "an unreachable **or unauthorised** URL", and the
  unauthorised case is proved at the engine and at the command line, not
  through the window. The **eighth criterion is not closed by this report**:
  the weasel is a human judgement, the user has chosen this design from
  rendered images, and the screenshot of the running window is in front of
  them for their reading of it.
  [INT-0002](../../../intents/INT-0002-git-and-remote-sources.md)'s remote
  half is proved; its git-ref half is **not in this sprint**, so that chapter
  stays `active`.
- **Tested head:** `77b0c00570c978b4d12ea2b8d40fa88816f285e2` (branch `dev`).
- **Runner:** `cargo test --all -- --nocapture`, the project's canonical suite.
- **CI:** [run 44561690290](https://github.com/crussella0129/MDeezl/actions/runs/44561690290)
  passed on `ubuntu-latest` (job 135692396143) and `windows-latest` (job
  135692395486). Both legs ran rustc 1.100.0, cargo 1.100.0 and clippy
  0.1.100, which the run's own update step installed over the image's 1.99.1,
  under the image's rustup 1.29.3 — the update steps pass
  `--no-self-update`, so no run ever installs rustup. git was 2.56.1 on Linux
  and 2.56.0.windows.4 on Windows.
- **Local host:** Windows 11, rustc 1.99.1, cargo 1.99.1, rustup 1.29.2, git
  2.56.0.windows.3.
- **Critique:** [critique.md](critique.md). Four rounds: `block`,
  `proceed-with-caveats`, `block`, `proceed-with-caveats`. Every round's
  concerns were answered rather than banked; the two `block` rounds are why
  this report says what it cannot show as plainly as what it can.

## Results

| Suite | Local (Windows) | `ubuntu-latest` | `windows-latest` | Detail |
|-------|-----------------|-----------------|------------------|--------|
| Unit | 79 passed | 79 passed | 79 passed | [unit-tests.md](unit-tests.md) |
| Integration | passed | passed | passed | [integration-tests.md](integration-tests.md) |
| End-to-end | 67 passed, 1 ignored | 67 passed, 1 ignored | 67 passed, 1 ignored | [e2e-tests.md](e2e-tests.md) |
| Desktop app | 31 passed | 31 passed | 31 passed | [e2e-tests.md](e2e-tests.md) |

That is 177 executed tests. The ignored one is `test_live_github_clone`, run
by hand at the tested head: 1 passed in 2.01 s.

**Every unit test that closed sprint 2 still passes, unedited**, and the
proof is a byte comparison rather than a count: the 47,456-byte `mod tests`
block of `24442f1:src/main.rs`, minus its closing brace, is still a byte
prefix of `src/lib.rs` at this head.

`tests/cli.rs` is insertions plus exactly two deletions — so **53 of its 54
tests are unedited, and one is not**: `test_ci_updates_installs_and_logs_in_order`
is INT-0005's guard, which pins the CI gate lines, so T-005 changed it in the
same commit as the lines themselves, and the critique rounds then strengthened
it twice more. That is the one test this sprint was always going to edit, and
the mutation table is what keeps the edit honest.

**Gates:** `cargo fmt --all --check` clean;
`cargo clippy --workspace --all-targets -- -D warnings` clean.

**The CLI is still dependency-free.** `cargo tree -p mdeezl` at the tested
head prints:

```
mdeezl v0.1.0 (C:\Users\charl\MDeezl)
```

Its automated counterpart, `test_manifest_dependencies_table_is_empty`, ran
unedited and passed.

## INT-0006, criterion by criterion

| Criterion | Proved by |
|-----------|-----------|
| One window, four controls | `test_app_has_exactly_the_controls` (exact widget counts by AccessKit role, every label, the placeholder), `test_app_defaults`, `test_dropped_folder_fills_source`, and the manual `Browse…` check: the dialog is the native `Select Folder`, and a chosen folder fills its field. |
| The CLI's own output | `test_bundle_matches_cli_output` for six option combinations, and `test_generate_writes_engine_bytes`, which compares the window's file against `mdeezl::bundle`. They meet in one function, so the two cannot drift. |
| Responsive while working | `test_generate_marks_working_before_returning`, `test_generate_requests_a_repaint`, `test_close_deferral_runs_in_logic_alone` (a control pass with no tick records nothing; the tick's own request is within 100 ms), `test_worker_wakes_the_window`, `test_generate_disabled_while_working`, `test_status_names_fetching_or_bundling`. Live: a 38.0 s fetch of `github.com/git/git` from the release window, which answered **152 of 152** UI Automation probes while it worked. |
| No partial files | `test_failed_job_keeps_existing_output` (both cases), `test_generate_unwritable_output_writes_nothing` (a real `fs::write` failure inside a valid folder), `test_generate_missing_folder_writes_nothing`, `test_generate_unreachable_url_writes_nothing`, `test_generate_rejects_an_output_that_is_not_a_folder`, `test_generate_applies_the_stem_rule_to_the_file`, `test_temp_name_carries_the_pid`. The unauthorised-URL half rides on the shared engine path; [e2e-tests.md](e2e-tests.md) says so explicitly. |
| The CLI stays dependency-free | `test_manifest_dependencies_table_is_empty` (unedited), `test_gui_manifest_dependencies`, and the `cargo tree` output above. |
| Tested in CI on both OSes | Both legs compiled `mdeezl-gui` and ran its 31 tests, with no package-install line anywhere in either log. |
| Documented | `test_readme_documents_desktop_app` — now needling the sentence `needs git on your PATH` rather than a bare `git`, which "Respect .gitignore" had been satisfying — and `test_readme_scopes_std_only_to_the_engine`. |
| The weasel is judged by a person | The app draws it from one shape list, with no image file and no image decoding: `test_weasel_shapes_fit_the_unit_box`, `test_hero_paints_weasel_shapes`, `test_icon_rasterises_the_weasel` (measured opaque share 0.223), `test_native_options_carry_the_icon`. The screenshot is [window.png](window.png), committed beside this report and sent to the user in this phase. |

### The user's words on the weasel

Across five rounds in T-006 the user asked for "two humps to look like an 'M'
for MDeezl"; a tail tip and a cream belly that "match the outline"; a front
leg "bumped over like half of its width to the right"; a raised rump; a neck
"extended out" and distinct from the back; and the sharp neck-to-back angle
"rounded out", after which they wrote "Nice work." Shown the final pair of
variants, they chose the throat-only one: **"just do this one and call it a
day."**

These words are the user's messages in the sprint 3 Test Phase conversation.
They are quoted identically in [e2e-tests.md](e2e-tests.md); no committed
artifact holds them, which is why both records carry them verbatim rather
than by reference.

That is a verdict on the design, given from rendered images. The window in
[window.png](window.png) is that design running at the tested head. Whether
the running window reads as a weasel is the user's call, and this phase puts
the screenshot in front of them to make it.

## INT-0002: what this sprint proves, and what it does not

**Proved**, for the remote half:

- a repository addressed by URL, fetched with a hardened shallow clone
  (`test_clone_command_is_hardened` pins the exact argument list and
  environment);
- the document records its source, with credentials stripped
  (`test_display_url_strips_credentials`, `test_cli_unreachable_url_exits_1`,
  which asserts `SECRET` never appears);
- failures give stderr, a non-zero exit and no partial document: missing
  (`test_fetch_bad_url_errs`), unreachable (`test_cli_unreachable_url_exits_1`),
  stalled (`test_stalled_server_fails_the_fetch`), a protocol outside the
  allow-list (`test_fetch_rejects_protocol_outside_allow_list`), git absent
  (`test_cli_url_without_git_exits_1`), and — new in this phase —
  **unauthorised**, twice over: a local socket answers 401 with a Basic
  challenge, and the engine fails in 0.25 s with `terminal prompts disabled`
  (`test_unauthorised_repository_fails_without_prompting`), while the real
  binary against the same socket gives exit 1, empty stdout and one
  `mdeezl: could not fetch …` line with no credentials in it
  (`test_cli_unauthorised_url_exits_1`). The criterion is written in
  command-line terms, so the engine-level assertion alone would not have met
  it;
- it never waits on a person: prompts, askpass and credential helpers off, a
  30-second stall limit, and the live clone that fails without prompting;
- the temporary copy is removed, asserted after every fetch test and observed
  after every manual run.

**Not in this sprint:** criterion 2, reading a git ref without mutating the
working tree. That is backlog T-103, as the locked test plan's traceability
map states. INT-0002 therefore stays `active` at this sprint's close.

**Covered only by a planning measurement:** an `insteadOf` rule that rewrites
an `https://` URL to ssh. The allow-list that stops it is asserted at the
fetch boundary, but the rewrite itself is not staged, because the clone runs
with an isolated empty global config by design.

## INT-0001 and INT-0005: not advanced, not regressed

- **INT-0001.** Local output is unchanged byte for byte: every sprint 0 and 1
  test ran unedited, and the byte-prefix check above is what proves
  "unedited". Two legs remain. The dependency property now reads as the
  `mdeezl` crate's, which the chapter already records.
- **INT-0005.** The guard test changed in the same commit as the gate lines it
  pins, and the mutation table was re-run: **49 rows, 0 not caught**.

  **This phase closes an observation sprint 2 left open.** The lag-correction
  half of INT-0005's criterion — that `rustup update stable` actually moves a
  lagging runner image forward — could only be recorded then as *not
  exercised*, because every update step reported `unchanged`. Three runs in
  this phase, the tested head's among them, report on **both** legs:

  ```
  stable-x86_64-unknown-linux-gnu updated - rustc 1.100.0 (0ef1bc386 2026-11-09) (from rustc 1.99.1 (58cf8f9c3 2026-10-13))
  stable-x86_64-pc-windows-msvc  updated - rustc 1.100.0 (0ef1bc386 2026-11-09) (from rustc 1.99.1 (58cf8f9c3 2026-10-13))
  ```

  The images carried 1.99.1, the update step moved each to 1.100.0, and the
  gates ran on 1.100.0. A fourth run the same day started on 1.100.0 already
  and reported `unchanged`: the image varies between runs, which is exactly
  why the workflow logs the reading before and after. Green under every
  combination, and green locally on 1.99.1, so this head is green on two
  different stables. Those logs belong to INT-0005 as evidence.

## Deviations from the locked plan

Each is a real change to what the plan said, made during this phase and
recorded here rather than folded in silently.

| Plan text | What is true now | Why |
|-----------|------------------|-----|
| T-002: "**WHEN** the source argument is not a URL — including … `git@host:x/y` … — **THEN** mdeezl **SHALL** treat it as a local path, as before." | Three shapes are refused rather than read as paths: any `<scheme>://` outside the three supported ones, an ssh address such as `git@host:x/y`, and a URL with no host, which now fails with `has no host` instead of after a DNS wait. A Windows path still works: `C:\x` has a colon but no `user@host`. | Driving the CLI by hand, each produced "The filename, directory name, or volume label syntax is incorrect" — the file system’s answer to a path nobody meant to type — and `https://` spent 7.5 s in DNS first. Pinned by `test_cli_explains_a_source_it_cannot_use` and `test_cli_windows_path_is_not_mistaken_for_an_address`. |
| T-001: "its stdout, its stderr … and its exit codes 0, 1 and 2 **SHALL** be unchanged" | Three changes under this one clause. A mistyped option is now named: `mdeezl --wrap-none` exits **2** with "unknown option", where the old binary made it the path and exited **1** with "The system cannot find the file specified. (os error 2)" — measured by building `24442f1` and running it. Given after a path, `mdeezl . --wrap-none`, it exited 2 as "unexpected second path" and still exits 2, with the better message. A missing source path is named: `<path>: <os error>`. A failed `-o` write is named too: `could not write <path>: <os error>`. | Each old message sent the reader somewhere other than the mistake: a path they never typed, and two OS errors with no subject. The exit-code change cost a `-`-leading folder its only spelling, so `--` was added, documented in `--help` and the README, and pinned by `test_cli_names_what_it_could_not_use`. |
| T-004: "write the document to a temporary sibling, `<output>/.<name>.md.mdeezl-tmp`" | The name carries the process id: `.<name>.md.<pid>.mdeezl-tmp`. | Two windows generating the same source into one folder shared that file and could delete each other's work mid-write. `test_temp_name_carries_the_pid` and `test_generate_unwritable_output_writes_nothing` pin it. |
| T-004's status messages (`Choose a folder or URL`, `Choose an output folder`) | A third: `Not a folder: <path>`, shown before any job starts. | A mistyped output folder was otherwise only reported after the whole bundle had been built, as a write failure. |
| The test plan's `test_generate_unwritable_output_writes_nothing`: "the output path is an existing *file*" | That case is now `test_generate_rejects_an_output_that_is_not_a_folder`; the original name drives a genuine write failure inside a valid folder. | The folder check above made the old arrangement stop before any write, so the test no longer reached the path it was traced to. |
| T-004's EARS for responsiveness: "each tick **SHALL** request a repaint within 100 ms" | `generate` also requests the first frame. | The clause never bootstraps: `tick` only runs when a frame runs. A live fetch drew **no frames for 27 seconds** until this was fixed (`b4b245f`), which contradicted INT-0006's "responsive while working" outright. `test_generate_requests_a_repaint` pins it. |

No CI step was added, so T-005's "any fix that adds a step is a plan
amendment" was not triggered — and that is now asserted rather than assumed:
the guard pins the job's step list exactly, every key of every step, and that
there is exactly one job (mutation rows 42–49).

## Manual checks

Run against `target/release/mdeezl-gui.exe` built from `c92ab73`. Nothing that ships has changed since: `gui/src/lib.rs`,
`gui/Cargo.toml`, `Cargo.toml` and `Cargo.lock` are untouched between that
commit and the tested head, and the shipped half of `src/lib.rs` — the
35,733 bytes above `#[cfg(test)] mod tests` — has the same SHA-256 at both.
The sole source change is one hunk inside the test module. The binary these
checks drove is the binary this head builds. They were driven
entirely through the app's own window — UI Automation for focus, buttons, the
native dialog and the status line, and posted key messages for typing — so
nothing global was injected and no window was pulled to the foreground.

| Check | Result |
|-------|--------|
| `Browse…` opens a native folder picker and fills its field | `Select Folder`, class `#32770`; the chosen folder appeared in `Source` exactly. |
| A cancelled picker leaves the field alone | Unchanged, app alive. |
| A live URL fetch stays responsive | `https://github.com/git/git`: `Fetching…` 0.4 s after Generate and throughout, Generate disabled, **152 of 152** probes answered, finished in **38.0 s** with `Wrote …\git.md (48.1 MB)`; first line `# https://github.com/git/git`; no temp folder or sibling left. |
| No console window flashes | Console-host windows sampled every 200 ms across that fetch: baseline 2, maximum 2. A flash shorter than the sampling interval would not be resolved; the mechanism is pinned by `test_git_commands_set_create_no_window`. |
| Closing during a fetch | Window stayed up reading `Closing when the current job finishes…`, answered **137 of 137** probes, exited **39.1 s** after Generate, wrote the file anyway, left **0** `mdeezl-*` folders in `%TEMP%`. |

## The exploratory run

The user asked for the app to be driven into a wall before any official suite
was run. A worktree on `explore/hammer` held a 47-case CLI sweep and three UI
Automation drivers. **Nine defects** were found by operating the thing — the
frozen window above, and eight more repaired together in `de66382` — and all
nine are listed with their repairs in [e2e-tests.md](e2e-tests.md). Three
regression tests on the engine and two on the window pin them.

The three critique rounds then raised 29 concerns about the *tests* and the
records rather than the code — fourteen, then eight, then seven. Nine were
mechanisms that no test would have missed if they had been deleted, which is
why each fix is recorded with the mutation that confirms it. Those are listed, with the mutation that confirms each fix,
in [critique.md](critique.md).

## What this phase cannot show

- Whether the running window reads as a weasel: that is the user's judgement,
  and the screenshot is in front of them.
- A console flash shorter than 200 ms.
- An `insteadOf` rewrite, as described above.
- Any guarantee about a machine without OpenGL 2+, which INT-0006 records as a
  consequence of the glow renderer.
