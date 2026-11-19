# Sprint 3 End-to-End Test Results

- **Status:** possible. Everything the test plan promised is implemented, and
  the parts a headless runner cannot reach were driven by hand against the
  release window.
- **Tested head:** `29ff2703f83ee48d2008c0bad0486c5859f935ab` (branch `dev`)
- **Runner:** `cargo test --all -- --nocapture`
- **Local result:** **67 passed, 0 failed, 1 ignored** in `tests/cli.rs`, and
  **31 passed, 0 failed** in `gui/tests/app.rs`. The ignored one is the live
  clone, run separately below. Sprint 2 closed with 54 end-to-end tests. **53
  of them ran unedited**; the one exception is
  `test_ci_updates_installs_and_logs_in_order`, INT-0005's guard, which pins
  the CI gate lines and so changed in the same commit as the lines themselves,
  then twice more from the critique. The mutation table is what keeps that
  edit honest.
- **CI result:** the same counts on both legs; see
  [integration-tests.md](integration-tests.md).

## The command line

### New in T-001 and T-002

| Test | EARS clause verified |
|------|----------------------|
| `test_main_rs_only_delegates_to_cli` | `src/main.rs`, with comments and blank lines removed, is at most 3 lines; it holds `mdeezl::cli(` and `std::process::exit(`, and no `use` or other `fn`. |
| `test_cli_file_url_source_bundles_the_clone` | Exit 0, heading is the URL, the committed file present and the untracked one absent, and the test-owned `TMP`/`TEMP`/`TMPDIR` folder empty afterwards. |
| `test_cli_unreachable_url_exits_1` | `https://user:SECRET@127.0.0.1:1/x.git` gives exit 1, empty stdout, and exactly one stderr line beginning `mdeezl: could not fetch https://127.0.0.1:1/x.git: ` which does not contain `SECRET`. Localhost only, so no network is used. The temp folder is empty afterwards. |
| `test_cli_unauthorised_url_exits_1` | A local socket answers 401 with a Basic challenge, as a private repository does to an anonymous client. The binary gives exit 1, empty stdout, exactly one line beginning `mdeezl: could not fetch http://127.0.0.1:PORT/private.git: `, no `SECRET` from the URL's userinfo, a refusal inside 20 s, and an empty temp folder. This is INT-0002's criterion in its own command-line terms; the engine-level half is `test_unauthorised_repository_fails_without_prompting`. |
| `test_cli_url_without_git_exits_1` | An empty `PATH` with a `file://` URL gives exit 1, empty stdout, and stderr ending `git was not found on PATH`. |
| `test_cli_help_documents_url_sources` | `--help` contains `URL`, `https://`, `shallow`, `temporary`, `removed`, `never prompts for credentials` and `credential helper`. |
| `test_readme_documents_url_sources` | The README carries the same URL-specific needles, its exit-code paragraph counts a failed fetch, and its "Not yet" section no longer contains `remote repositor`. |

### The Test Phase's own repair tests

These pin the behaviour the exploratory run changed, and are the assertions
behind two of the deviations in the [test report](test-report.md).

| Test | What it pins |
|------|--------------|
| `test_cli_explains_a_source_it_cannot_use` | `ssh://127.0.0.1/x` and `ftp://host/x` say which schemes are supported; `git@github.com:o/r.git` says it is an ssh address; `https://` with no host says `has no host`. Each exits 1 with empty stdout, names `https://, http:// and file://`, and **returns within 5 s** — the bound that catches the 7.5 s DNS wait the bare `https://` used to take. |
| `test_cli_windows_path_is_not_mistaken_for_an_address` | A Windows path bundles normally: the `user@host` rule must not fire on `C:\x`, which has a colon and no `@`. |
| `test_cli_names_what_it_could_not_use` | A missing path, a mistyped option (exit 2), a failed `-o` write, the `--` escape for a `-`-leading folder, and that `--help` and the README document that escape and what a mistyped option does. |

### New in T-007

| Test | EARS clause verified |
|------|----------------------|
| `test_readme_scopes_std_only_to_the_engine` | The introduction, the text before `## Install`, scopes "standard library only" to the `mdeezl` crate and names the desktop app's separate crate. |
| `test_readme_documents_desktop_app` | The `## Desktop app` section gives `cargo run -p mdeezl-gui --release`, `cargo install --path gui`, every control name, `.md`, `replac`, and — after the critique — the sentence `needs git on your PATH` rather than a bare `git`, which "Respect .gitignore" already satisfied. |

### The live clone

`test_live_github_clone` is `#[ignore]`d and was run by hand at the tested
head: `cargo test --test cli -- --ignored --nocapture` → **1 passed in 2.01 s**.
It clones `https://github.com/octocat/Hello-World` (heading and a `README`
section), fails on `https://github.com/octocat/definitely-not-a-repo-mdeezl`
within 10 s with exit 1 and no prompt, and leaves the temp folder empty after
both.

### Platform SKIPs

The Windows run printed the four platform SKIPs sprint 1 recorded. The Linux
leg printed one, this sprint's own: `SKIP test_git_commands_set_create_no_window: Windows only`.

## The window, driven headlessly

Most tests below hold the real `App` through `Harness::new_eframe`, which
calls the real `eframe::App::logic` and then `ui`, exactly as eframe does.
That is what proves the `logic → tick` wiring: `wait_idle` polls until the app
stops working, so a `tick` that was not wired into `logic` would hang every
harness test that waits for a result — `test_generate_writes_engine_bytes`
first among them.

**Three are deliberately not harness tests**:
`test_close_deferral_runs_in_logic_alone`, `test_generate_requests_a_repaint`
and `test_worker_wakes_the_window` drive a bare `egui::Context`, because each
observes something eframe's own pass would mask — a logic pass with no `ui`,
a request made before any pass, and a wake-up with no pass at all. The locked
test plan named the first as the one exception; this phase added the other
two, for the repairs described below.

Tests wait with the `step()` polling loop, never `run()`, and any test that
needs a working app injects a gated job with a test-owned output folder.

| Test | EARS clause verified |
|------|----------------------|
| `test_app_has_exactly_the_controls` | Exactly 2 `TextInput`, 3 `Button`, 3 `RadioButton`, 1 `CheckBox`; the labels `MDeezl`, `Source`, `Output folder`, `fence`, `inline`, `none`, `Respect .gitignore`, `Generate`; the source field's placeholder `Folder path or https:// URL`; two `Browse…` buttons. |
| `test_app_defaults` | `fence` selected, checkbox checked, output field holding the home folder. |
| `test_dropped_folder_fills_source` | A dropped folder becomes the source after one `step()`. |
| `test_generate_marks_working_before_returning` | With a gated job, `generate()` returns already working; the polling loop later sees the result. |
| `test_generate_requests_a_repaint` | `generate()` asks for the frame that starts the polling, so a window with no input still shows the result. |
| `test_generate_disabled_while_working` | `Generate`'s AccessKit node is disabled while working, the root viewport's `repaint_delay` is at most 100 ms, and the button is enabled again afterwards. The frame's own animations can satisfy that delay, so the tick's own request is pinned separately, below. |
| `test_close_deferral_runs_in_logic_alone` | With `ctx.run_logic` and no `ui` pass: a control pass that runs **no** tick records no repaint request at all, the next pass with the tick records exactly one, within 100 ms; `CancelClose` is returned for a close while working; and a later `run_logic` returns `Close`. |
| `test_worker_wakes_the_window` | With no pass run at all, the worker's own `request_repaint` reaches the callback with zero delay. |
| `test_status_names_fetching_or_bundling` | A local source shows `Bundling…` and a `file://` source `Fetching…`; editing the field mid-job does not change the label, which follows the source captured at Generate. |
| `test_generate_shows_notices` | A fixture that is not a git work tree gives a status with `Wrote` and `gitignore filtering skipped:`. |
| `test_worker_panic_ends_job_with_error` | A panicking job gives `The job stopped unexpectedly`, not working, and an empty output folder. |
| `test_dropping_app_waits_for_worker` | The job keeps working for 300 ms after its release; the test checks the job is still running when the drop begins, then that the drop returned only after it finished. "The drop waited" and "the job was already over" are now different outcomes. |
| `test_close_while_working_defers_then_closes` | A close during a job produces `CancelClose` and the status `Closing when the current job finishes…`, which persists; `Close` follows the result; the echo of its own `Close` is not cancelled; an idle close produces no `CancelClose`. |
| `test_generate_writes_engine_bytes` | The written file equals `mdeezl::bundle`; see [integration-tests.md](integration-tests.md). |
| `test_generate_applies_the_stem_rule_to_the_file` | A job returning the name `a/b` is written as `bundle.md` **inside** the output folder, with the status naming it. `output_stem` alone is a pure function; this is what proves the written file takes its name from it. |
| `test_failed_job_keeps_existing_output` | A failing job, and a rename failure provoked by making `<name>.md` a directory: the error shows, the earlier file or directory is untouched, and no `.mdeezl-tmp` remains. |
| `test_generate_unwritable_output_writes_nothing` | A write that genuinely fails inside a real output folder: the temporary sibling's own name is occupied by a directory, so `fs::write` cannot create it. The status begins `could not write`, and an existing `gated.md` is untouched. Blocking that exact name also proves the write goes through `temp_name`. |
| `test_generate_missing_folder_writes_nothing`, `test_generate_unreachable_url_writes_nothing` | Each shows an error and writes nothing; the unreachable URL's message begins `could not fetch https://127.0.0.1:1/x.git` and never contains `SECRET`. |
| `test_generate_requires_source_and_output` | `Choose a folder or URL` and `Choose an output folder`, with the job's call counter still 0. |
| `test_generate_rejects_an_output_that_is_not_a_folder` | A file, or a missing folder, named as the output is refused **before** any job starts, with `Not a folder: <path>`; the file is untouched. |
| `test_generate_trims_the_fields` | A path pasted with surrounding spaces works, in both fields. |
| `test_hero_paints_weasel_shapes` | Clipped to `App::hero_rect()`, the painted output holds at least one filled circle per disc and per tube vertex, and at least one line segment per tube segment. |

## Manual checks

A native dialog, a real network fetch and a real window close cannot be driven
by the headless harness. These were run against
`target/release/mdeezl-gui.exe` built from `c92ab73`. Everything the two
critique rounds changed afterwards is test or documentation text: between
that commit and the tested head, `gui/src/lib.rs` is untouched and the shipped
half of `src/lib.rs` -- everything above `#[cfg(test)] mod tests` -- is
byte-identical, which `diff` on the two blobs confirms. The binary these
checks drove is the binary this head builds.

**How they were driven.** Everything was addressed to the app's own window:
UI Automation for focus, buttons, the dialog and reading the status, and
posted key messages for typing. No global input was injected and no window was
brought to the foreground, so the run could not disturb anything else on the
desktop — or be disturbed by it. Each field is read back after typing and
retyped until it matches, so a dropped keystroke fails loudly instead of
quietly testing the wrong thing.

| Check | Observed |
|-------|----------|
| `Browse…` opens a native folder picker and fills its field | The dialog is `Select Folder`, class `#32770`. Setting its `Folder:` field to the lab folder and invoking `Select Folder` left the app's `Source` reading exactly `…\manual-lab\pick`. |
| A cancelled picker leaves the field alone | A second `Browse…` dismissed with the dialog's `Cancel` left `Source` unchanged, and the app alive. |
| A live fetch shows `Fetching…` while the window stays responsive | `https://github.com/git/git` into `…\manual-lab\out`. 0.4 s after `Generate` the status read `Fetching…`, and that was the only working label seen. `Generate` was observed disabled. UI Automation answered **152 of 152** probes during the fetch. |
| …and finishes, with the engine's own output | **38.0 s**, ending `Wrote …\out\git.md (48.1 MB)`. The file is 50,467,737 bytes and its first line is `# https://github.com/git/git`. No `mdeezl-*` folder in `%TEMP%`, no `.mdeezl-tmp` sibling in the output folder. |
| No console window flashes | Visible console-host windows (`ConsoleWindowClass`, `PseudoConsoleWindow`) were counted every 200 ms across that 38.0 s fetch: baseline 2, maximum 2. A flash shorter than the 200 ms sampling interval would not be resolved by this; the mechanism itself is pinned by `test_git_commands_set_create_no_window`, which asserts the flag is applied and that nothing else in the shipped code spawns git. |
| Closing during that fetch | A second run, closed 3 s in. The window stayed up and read `Closing when the current job finishes…`; it answered **137 of 137** probes while the close was deferred; the process exited **39.1 s** after `Generate`; `git.md` was written anyway, all 50,467,737 bytes of it; **0** `mdeezl-*` folders were left in `%TEMP%`. |

## Human verification

![The MDeezl window](window.png)

The screenshot above was captured from the running release window at the
tested head (536×639, `PrintWindow` with `PW_RENDERFULLCONTENT`; 110 distinct
colours in a sampled grid, so it is a real readback of the GL surface rather
than a blank one), and sent to the user in this phase. It is committed beside
this record, because
[INT-0006](../../../intents/INT-0006-desktop-gui.md)'s eighth criterion turns
on it.

**What the user actually said, and when.** The weasel was reviewed across five
rounds during T-006, in the user's words: "two humps to look like an 'M' for
MDeezl"; a tail tip and a cream belly that "match the outline"; a front leg
"bumped over like half of its width to the right" so it reads as a shoulder; a
raised rump; a neck "extended out" and distinct from the back; and that sharp
neck-to-back angle "rounded out", after which they wrote "Nice work." Shown
the last pair of variants, they chose the throat-only one with: "just do this
one and call it a day." That is a verdict on the design, given from rendered
images of it; the window above is that design running, and the user's reading
of this screenshot is theirs to give in this phase.

**Where the design landed.** T-006's task record names commit `5963ace`, which
is the weasel *before* the user's last two rounds. The design they chose is
commit `e015bf1`, and T-006's recorded opaque-coverage figure of 0.177 belongs
to the earlier shape; the test now prints the measured share, **0.223**, on
every run.

## The exploratory run, and what it repaired

Before any of the suites above were run at this head, the user asked for the
app to be driven into a wall rather than tested politely: *"I want you to set
up an environment or worktree where you can drive this thing into a wall while
trying to operate it and repair along the way, and only perform the official
unit and integration tests once we're confident that we've reverse e2e this."*

A worktree on `explore/hammer` held a 47-case CLI sweep and three UI
Automation drivers that typed into the release window.

| What was done | What happened | Repair |
|---------------|---------------|--------|
| a live `https://github.com/git/git` fetch from the window | **the window drew no frames for the whole 27 s fetch**: nothing requested a repaint between pressing Generate and the worker's wake-up | `generate` asks for the first frame; `test_generate_requests_a_repaint` (commit `b4b245f`) |
| `mdeezl ssh://host/repo`, `ftp://…` | the filename-syntax error from the file system, because it had fallen through to a path | the scheme is named, with the three that work |
| `mdeezl git@github.com:o/r.git` | the same filename error | `is an ssh address, which is not supported` |
| `mdeezl https://` | 7.5 s in DNS, then a failure | fails in 77 ms with `has no host` |
| a source path that is not there | the OS error, without the path in it | the path is named |
| `-o` pointed at a directory | `Access is denied. (os error 5)` alone | `could not write <path>: …` |
| `--wrap-none` mistyped | taken as a second path: `unexpected second path` | the unknown option is named, exit 2 |
| a path pasted with surrounding spaces | the filename-syntax error | both fields are trimmed |
| two windows writing one output folder | a shared temp filename, so each could delete the other's file mid-write | the temp name carries the pid |

The first row is the oldest and the most serious: it contradicted INT-0006's
"responsive while working" outright, and it is recorded here as a defect of
this phase rather than left to its own commit message. The eight below it were
repaired together in `de66382`.

**Two of those repairs changed documented behaviour**, and are deviations from
the locked plan rather than pure fixes. Both are recorded in the
[test report](test-report.md):

- `git@host:x/y` is no longer "treated as a local path, as before" (T-002's
  wording) — it is refused with an explanation;
- a mistyped option now exits 2 rather than 1, against T-001's "exit codes 0,
  1 and 2 **SHALL** be unchanged". Rejecting unknown options also made a
  folder whose name begins with `-` unbundlable, which the critique caught; the
  Test Phase added `--` as the escape, documented in `--help` and pinned by
  `test_cli_names_what_it_could_not_use`.

What held up under the same abuse, and is recorded here because it is evidence
for INT-0006's "responsive while working" and "no partial files": a 3000-file
source (the window answered 10 of 10 probes while it worked), resize to
200×150 and to 1600×1000, `Browse…` dismissed with ESC, an output file locked
open by another process, five rapid `Generate` clicks, and the close during a
real clone that is now the last manual check above.

## What is covered only through the shared engine

[INT-0006](../../../intents/INT-0006-desktop-gui.md)'s "no partial files"
criterion names "an unreachable **or unauthorised** URL" reported *in the
window*. The window has `test_generate_unreachable_url_writes_nothing`, which
drives the real job to an unreachable URL and proves the error reaches the
status line with no file written. There is no GUI test for the unauthorised
case. It is proved twice below the window — at the engine by
`test_unauthorised_repository_fails_without_prompting`, and through the real
binary by `test_cli_unauthorised_url_exits_1` — and the window runs the same
`mdeezl::bundle` as the binary. What is unproved is only that the window and
that engine path compose, which every other error path in the table above
shows they do. It is recorded here rather than left silent.

## What is not covered here

[INT-0002](../../../intents/INT-0002-git-and-remote-sources.md)'s second
acceptance criterion — reading a git ref without mutating the working tree —
is **not in this sprint**, as the test plan's traceability map states. It is
backlog T-103, and the chapter stays `active` at this sprint's close.
