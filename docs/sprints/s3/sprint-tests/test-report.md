# Sprint 3 Test Report

- **Verdict:** pass. Every acceptance criterion of
  [INT-0006](../../../intents/INT-0006-desktop-gui.md) is proved by executed
  tests, CI logs or recorded observation, except the eighth, which is a human
  judgement and is recorded as the user's own.
  [INT-0002](../../../intents/INT-0002-git-and-remote-sources.md)'s remote
  half is proved; its git-ref half is **not in this sprint**, so that chapter
  stays `active`.
- **Tested head:** `cb11527c73cba51b630c32bafe5912c1d378bbfe` (branch `dev`).
- **Runner:** `cargo test --all -- --nocapture`, the project's canonical suite.
- **CI:** [run 44554527551](https://github.com/crussella0129/MDeezl/actions/runs/44554527551)
  passed on `ubuntu-latest` (job 135666884830) and `windows-latest` (job
  135666884423). Both legs ran rustc 1.100.0, cargo 1.100.0, clippy 0.1.100
  and rustup 1.29.3, installed by the run's own update step; git was 2.56.1 on
  Linux and 2.56.0.windows.4 on Windows.
- **Local host:** Windows 11, rustc 1.99.1, cargo 1.99.1, rustup 1.29.2, git
  2.56.0.windows.3.
- **Critique:** [critique.md](critique.md). Two rounds: `block`, then
  `proceed-with-caveats`, whose caveats were closed rather than banked.

## Results

| Suite | Local (Windows) | `ubuntu-latest` | `windows-latest` | Detail |
|-------|-----------------|-----------------|------------------|--------|
| Unit | 79 passed | 79 passed | 79 passed | [unit-tests.md](unit-tests.md) |
| Integration | passed | passed | passed | [integration-tests.md](integration-tests.md) |
| End-to-end | 66 passed, 1 ignored | 66 passed, 1 ignored | 66 passed, 1 ignored | [e2e-tests.md](e2e-tests.md) |
| Desktop app | 31 passed | 31 passed | 31 passed | [e2e-tests.md](e2e-tests.md) |

That is 176 executed tests. The ignored one is `test_live_github_clone`, run
by hand at the tested head: 1 passed in 2.01 s.

**Every test that closed sprint 2 still passes, unedited.** The proof is a
byte comparison, not a count: the 47,456-byte `mod tests` block of
`24442f1:src/main.rs`, minus its closing brace, is still a byte prefix of
`src/lib.rs` at this head. `tests/cli.rs` gained insertions plus exactly two
deletions, the two gate lines T-005 replaced.

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
"rounded out" — then: "so much better… Nice work." Shown the final pair of
variants, they chose the throat-only one: **"just do this one and call it a
day."**

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
  **unauthorised** (`test_unauthorised_repository_fails_without_prompting`: a
  local socket answers 401 with a Basic challenge, and the fetch fails in
  0.25 s with `terminal prompts disabled`);
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
  pins, and the mutation table was re-run: **44 rows, 0 not caught**.

  **This sprint closes an observation sprint 2 left open.** The lag-correction
  half of INT-0005's criterion — that `rustup update stable` actually moves a
  lagging runner image forward — could only be recorded then as *not
  exercised*, because every update step reported `unchanged`. The tested
  head's run reports, on **both** legs:

  ```
  stable-x86_64-unknown-linux-gnu updated - rustc 1.100.0 (0ef1bc386 2026-11-09) (from rustc 1.99.1 (58cf8f9c3 2026-10-13))
  stable-x86_64-pc-windows-msvc  updated - rustc 1.100.0 (0ef1bc386 2026-11-09) (from rustc 1.99.1 (58cf8f9c3 2026-10-13))
  ```

  The images carried 1.99.1, the update step moved them to 1.100.0, and the
  gates ran on 1.100.0. The same head passes locally on 1.99.1, so it is green
  on two different stables. That log belongs to INT-0005 as evidence.

## Deviations from the locked plan

Each is a real change to what the plan said, made during this phase and
recorded here rather than folded in silently.

| Plan text | What is true now | Why |
|-----------|------------------|-----|
| T-002: "**WHEN** the source argument is not a URL — including … `git@host:x/y` … — **THEN** mdeezl **SHALL** treat it as a local path, as before." | An ssh address is refused with "`git@host:x/y` is an ssh address, which is not supported", and the three supported schemes are named. | Driving the CLI by hand, it produced "The filename, directory name, or volume label syntax is incorrect" — the file system's answer to a path nobody meant to type. |
| T-001: "its stdout, its stderr … and its exit codes 0, 1 and 2 **SHALL** be unchanged" | A mistyped option (`--wrap-none`) now exits **2** with "unknown option", where it used to exit 1 as "unexpected second path". | The old message sent the reader looking for a path they never typed. The cost is that a `-`-leading folder needed an escape, so `--` was added, documented in `--help` and the README, and pinned by `test_cli_names_what_it_could_not_use`. |
| T-004: "write the document to a temporary sibling, `<output>/.<name>.md.mdeezl-tmp`" | The name carries the process id: `.<name>.md.<pid>.mdeezl-tmp`. | Two windows generating the same source into one folder shared that file and could delete each other's work mid-write. `test_temp_name_carries_the_pid` and `test_generate_unwritable_output_writes_nothing` pin it. |
| T-004's status messages (`Choose a folder or URL`, `Choose an output folder`) | A third: `Not a folder: <path>`, shown before any job starts. | A mistyped output folder was otherwise only reported after the whole bundle had been built, as a write failure. |
| The test plan's `test_generate_unwritable_output_writes_nothing`: "the output path is an existing *file*" | That case is now `test_generate_rejects_an_output_that_is_not_a_folder`; the original name drives a genuine write failure inside a valid folder. | The folder check above made the old arrangement stop before any write, so the test no longer reached the path it was traced to. |
| T-004's EARS for responsiveness: "each tick **SHALL** request a repaint within 100 ms" | `generate` also requests the first frame. | The clause never bootstraps: `tick` only runs when a frame runs. A live fetch drew **no frames for 27 seconds** until this was fixed (`b4b245f`), which contradicted INT-0006's "responsive while working" outright. `test_generate_requests_a_repaint` pins it. |

No CI step was added, so T-005's "any fix that adds a step is a plan
amendment" was not triggered — and that is now asserted rather than assumed:
the guard pins the job's step list exactly, every key of every step, and that
there is exactly one job (mutation rows 42–44).

## Manual checks

Run against `target/release/mdeezl-gui.exe` built from the tested head, driven
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

The two critique rounds then found fourteen more problems in the *tests*
rather than the code: seven mechanisms that no test would have missed if they
had been deleted. Those are listed, with the mutation that confirms each fix,
in [critique.md](critique.md).

## What this phase cannot show

- Whether the running window reads as a weasel: that is the user's judgement,
  and the screenshot is in front of them.
- A console flash shorter than 200 ms.
- An `insteadOf` rewrite, as described above.
- Any guarantee about a machine without OpenGL 2+, which INT-0006 records as a
  consequence of the glow renderer.
