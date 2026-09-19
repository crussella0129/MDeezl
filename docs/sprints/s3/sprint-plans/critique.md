# Plan Critique — Sprint 3

Read-only plan critic rounds, from the installed bundle's
`prompts/plan-critic.md`. The primary agent's resolution follows each concern.
Where a fix rests on a measurement, the measurement was taken before the plan
changed.

| Round | Verdict | Concerns |
|-------|---------|----------|
| 1 | `block` | 16 |
| 2 | `proceed-with-caveats` | 10 |
| 3 | `proceed-with-caveats` | 4 |
| 4 | `proceed-with-caveats` | 6 |
| 5 | `proceed-with-caveats` | 5 |

Round 2 verified the round-1 resolutions against the revised plans, the
intents and the crate sources, and raised ten new concerns (R2 C-001 to
C-010). They follow the round-1 concerns. Round 3 confirmed that the round-2
fixes held and that no round-1 fix had regressed, and raised four concerns
(R3 C-001 to C-004). Round 4 confirmed that the CancelClose design works in
eframe 0.36.2, traced through `epi_integration.rs`, `egui-winit` and
`glow_integration.rs`. It found no regressions and complete traceability, and
raised six narrower concerns (R4 C-001 to C-006). Round 5 verified those
resolutions, and confirmed from the egui and eframe sources that the logic-path
design, and `Context::run_logic` in a test, are feasible. It raised five
tightenings (R5 C-001 to C-005), which come last.

**Why no sixth round.** Every round-5 change tightens a test or a sentence, or
removes two more environment variables from a command whose shape is already
pinned. None adds a new behaviour for review, and the concern count and
severity fell in every round. The plans are locked on round 5's
`proceed-with-caveats`, with every concern below resolved.

## Concerns

### C-001: Terminal prompts off is not askpass or ssh prompts off
- **Where:** `build-plan.md` T-002, clone clause; INT-0002 Alternatives
- **Quote:** "with `GIT_TERMINAL_PROMPT=0` and `GCM_INTERACTIVE=never` set"
- **Failure mode:** missing-risk
- **Why it matters:** `GIT_ASKPASS`, `core.askPass` and `SSH_ASKPASS` are all consulted before the terminal. An `insteadOf` rewrite to ssh would also prompt on the TTY.
- **Suggested response:** fix-in-plan
- **Resolution:** fixed. The clone adds `-c core.askPass=`, removes `GIT_ASKPASS` and `SSH_ASKPASS`, and sets `GIT_ALLOW_PROTOCOL=https:http:file`. Measured:
  - an https URL rewritten to ssh fails in 122 ms with "transport 'ssh' not allowed";
  - a hanging `GIT_ASKPASS`, once removed, and a configured `core.askPass`, overridden, both fail fast with "terminal prompts disabled";
  - a hardened `file://` shallow clone still works.

  The unit test asserts the full command and environment. A new test checks that an `ssh://` URL is refused offline. INT-0002 is amended.

### C-002: The X11-only claim is false
- **Where:** T-003 dependency clause; INT-0006 Consequences
- **Quote:** "`glow`, `default_fonts`, `accesskit` and `x11` on"
- **Failure mode:** intent-drift
- **Why it matters:** `accesskit` brings winit's Wayland backend, which picks Wayland when `WAYLAND_DISPLAY` is set. Without eframe's `wayland` feature, glutin cannot open that window.
- **Suggested response:** fix-in-plan
- **Resolution:** fixed. eframe's features are now `glow`, `default_fonts`, `accesskit`, `x11` and `wayland`, as in the probe. INT-0006's Consequences are corrected: both display servers are supported, and no system packages are expected. The self-contradiction is gone.

### C-003: No deterministic seam for the working state
- **Where:** `test-plan.md` GUI tests; T-003 working clause
- **Quote:** "a local source shows `Bundling…`"
- **Failure mode:** plan-test-mismatch
- **Why it matters:** a local bundle finishes before the next frame, and `Harness::run()` panics while the app repaints.
- **Suggested response:** fix-in-plan
- **Resolution:** fixed. The app's job is injectable, and defaults to `mdeezl::bundle`. The working-state tests use a gated job that blocks until the test releases it. Every GUI test waits with a `step()` + 10 ms polling loop and a 10 s deadline, never `run()`. The disabled check uses `accesskit_node().is_disabled()`.

### C-004: No test that notices reach the GUI
- **Where:** T-003 Generate clause
- **Quote:** "show `Wrote <path> (<size>)` and every notice"
- **Failure mode:** plan-test-mismatch
- **Why it matters:** a GUI that drops notices would pass.
- **Suggested response:** fix-in-plan
- **Resolution:** fixed. `test_generate_shows_notices` asserts `gitignore filtering skipped:` on a non-git fixture. `<size>` is pinned as `format_size`, which `test_format_size` checks.

### C-005: The hero test could pass with no weasel drawn
- **Where:** `test_hero_paints_weasel_shapes`
- **Quote:** "at least as many circle shapes as `weasel::shapes()` has discs"
- **Failure mode:** plan-test-mismatch
- **Why it matters:** the radio buttons alone paint four circles.
- **Suggested response:** fix-in-plan
- **Resolution:** fixed. The test counts only shapes inside `App::hero_rect()`. It requires circles for every disc and tube vertex, and line segments for every tube segment.

### C-006: "Moved verbatim" freezes more than the plan says
- **Where:** T-001 move clause
- **Quote:** "The `#[cfg(test)] mod tests` block **SHALL** move to `src/lib.rs` byte-identical"
- **Failure mode:** hidden-dep
- **Why it matters:** the moved tests constrain signatures and imports. New unit tests would break a strict byte-identity rule, or need duplicated helpers.
- **Suggested response:** fix-in-plan
- **Resolution:** fixed. The check is now a byte prefix: the old block, without its closing brace, begins the new one, and new unit tests are appended inside it, reusing its helpers.
  - `render_document` keeps its signature and delegates to a new `render_titled`.
  - Imports used only by the moved tests are kept, marked `cfg_attr(not(test), allow(unused_imports))` until T-002 uses them.

### C-007: The fetch tests cannot identify their own temp directory
- **Where:** T-002 unit tests
- **Quote:** "No `mdeezl-<pid>-*` directory from this call remains."
- **Failure mode:** plan-test-mismatch
- **Why it matters:** there is one shared temp folder with parallel tests, and a `file://` URL built naively on Windows is wrong.
- **Suggested response:** fix-in-plan
- **Resolution:** fixed.
  - `fetch(url, base)` takes a base folder: unit tests pass their own, and `bundle` passes `temp_dir()`.
  - The `file://` construction is specified once, in T-002's Notes: canonicalise, strip `\\?\`, turn `\` into `/`, then add `file://` or `file:///`.
  - The critic verified that std's Windows `remove_dir_all` ignores the read-only attribute.

### C-008: In-process and isolated git configuration differ
- **Where:** `test_bundle_matches_cli_output`; the clone
- **Quote:** "`mdeezl::bundle(&opts).document` equals the bytes of the CLI's `-o` file"
- **Failure mode:** hidden-dep
- **Why it matters:** `isolate()` on one side only would make host git config differ between the sides. An un-isolated clone could get CRLF from a system config.
- **Suggested response:** fix-in-plan
- **Resolution:** fixed.
  - Both sides run under the same inherited environment, with no `isolate()` on either.
  - The clone gets the same `cfg(test)` isolation as `git_ignored`, and `-c core.autocrlf=false` in production, so it holds the committed bytes.
  - Content assertions ignore line endings.

### C-009: The GUI needs URL detection, but it stays private
- **Where:** T-001 Notes
- **Quote:** "Everything else stays private."
- **Failure mode:** hidden-dep
- **Why it matters:** the GUI would duplicate the rule.
- **Suggested response:** fix-in-plan
- **Resolution:** fixed. `pub fn is_url` joins the public surface in T-002, and the GUI calls it.

### C-010: The release GUI would flash console windows
- **Where:** T-003 Notes
- **Quote:** "A release build sets `windows_subsystem = "windows"`"
- **Failure mode:** missing-risk
- **Why it matters:** a GUI-subsystem parent spawning `git.exe` gets a new console window for each spawn.
- **Suggested response:** fix-in-plan
- **Resolution:** fixed.
  - Both git spawns go through one helper, which sets `CREATE_NO_WINDOW` on Windows. It uses std only.
  - A Windows test checks that a helper-built command still runs.
  - A manual check with the release GUI confirms that nothing flashes. INT-0002 records it.

### C-011: Renderer and logo decisions lived only in sprint prose, with a wrong reason
- **Where:** research §4–5; INT-0006 Alternatives
- **Quote:** "with no image files or image decoders"
- **Failure mode:** intent-drift
- **Why it matters:** eframe always links a PNG decoder, and glow's OpenGL 2+ requirement was unrecorded.
- **Suggested response:** fix-in-plan
- **Resolution:** fixed. INT-0006 now records:
  - the wgpu alternative, at +43 crates, with its software fallback;
  - the PNG alternative, at near-zero cost, not chosen because no raster weasel exists and code shapes feed both hero and icon;
  - the OpenGL 2+ requirement.

  The logo criterion and T-006 now read "no image decoding for the logo".

### C-012: INT-0001 and INT-0005 were constrained without a record
- **Where:** INT-0001 title criterion; INT-0005 Non-goals
- **Quote:** "a `# <root name>` title" / "changing what the gates check"
- **Failure mode:** intent-drift
- **Why it matters:** the URL heading is an exception, the gate lines widen, and eframe's minimum Rust bounds a pin.
- **Suggested response:** fix-in-plan
- **Resolution:** fixed. INT-0001 has a no-state-change note covering the URL title and the dependency property's scope. INT-0005 has a note covering the widened gates, the re-run guard mutation, and pins needing Rust 1.95 or later.

### C-013: T-004's fallback contradicted its own EARS
- **Where:** T-004 (now T-005)
- **Quote:** "No other step or key **SHALL** change." vs "a bash step … is the contingency"
- **Failure mode:** EARS-vague
- **Why it matters:** using the fallback would break the clause.
- **Suggested response:** fix-in-plan
- **Resolution:** fixed. The fallback is gone. The EARS clause now also requires that no package is installed. A Linux build failure means the task is not done, and any added step is a plan amendment. The critic's source reading found every Linux library loaded at run time. The pre-push cross-check was not adopted, because it would need a Linux target installed on this machine's toolchain. The first CI run is the check.

### C-014: Smaller gaps between clauses and tests
- **Where:** T-001, T-002, T-003, T-005 and T-006
- **Quote:** "with stdin connected to null"; "and nothing else"
- **Failure mode:** plan-test-mismatch
- **Why it matters:** several clauses could not be verified as written.
- **Suggested response:** fix-in-plan
- **Resolution:** fixed.
  - **stdin.** Moved to a note, left to code review, because `Command` does not expose it.
  - **`main.rs`.** The clause names the exact expression, and the test allows `std::env::args` while forbidding `use` lines and other functions.
  - **README.** The test lists every control.
  - **GUI manifest.** The test pins `default-features` and the exact features.
  - **Window icon.** `test_native_options_carry_the_icon` covers it.
  - **Exact controls.** `test_app_has_exactly_the_controls` counts widgets by role.
  - **Insertions only.** Scoped to T-001 through T-004.
  - **Dropped files.** The test implements egui's `DroppedFile` trait.

### C-015: A failed write could delete an earlier good output
- **Where:** T-003 failure clause
- **Quote:** "A failed write removes its file."
- **Failure mode:** missing-risk
- **Why it matters:** truncating, then cleaning up, destroys the previous file.
- **Suggested response:** fix-in-plan
- **Resolution:** fixed. Generate writes `.<name>.md.mdeezl-tmp`, then renames it over `<name>.md`. On failure only the temporary file is removed, and an existing file is untouched. `test_failed_job_keeps_existing_output` checks this, and INT-0006 records the overwrite rule.

### C-016: T-003 mixed several concerns
- **Where:** T-003
- **Quote:** eight EARS clauses in one task
- **Failure mode:** granularity
- **Why it matters:** a worker failure could not be reviewed apart from the manifest diff.
- **Suggested response:** fix-in-plan
- **Resolution:** fixed. T-003 is now the crate, window and fields. The new T-004 is Generate: the worker, status and safe write. The old T-004 to T-006 are renumbered T-005 to T-007.

### R2 C-001: The failure-reason rule gives a useless message for `file://`
- **Where:** T-002 fetch-failure clause; `test_fetch_bad_url_errs`
- **Quote:** "`<reason>` **SHALL** be git's last non-empty stderr line without a leading `fatal: `"
- **Failure mode:** plan-test-mismatch
- **Why it matters:** a missing repository's last stderr line is "and the repository exists.", and the planned test checked only the prefix.
- **Suggested response:** fix-in-plan
- **Resolution:** fixed. The reason is git's first `fatal: ` line with the prefix removed, falling back to the last non-empty line. Measured: the first line is `'C:/definitely-missing-mdeezl-xyz' does not appear to be a git repository`. `test_fetch_bad_url_errs` now asserts that text.

### R2 C-002: `core.autocrlf=false` does not keep line endings the same across OSes
- **Where:** T-002 success clause; INT-0002 Alternatives
- **Quote:** "The committed files' bytes are as committed, because `core.autocrlf=false`."
- **Failure mode:** intent-drift
- **Why it matters:** `core.eol` defaults to `native`, so `* text=auto` repositories check out CRLF on Windows.
- **Suggested response:** fix-in-plan
- **Resolution:** fixed.
  - **Measured:** a `* text=auto` fixture cloned with `autocrlf=false` checked out `one\r\ntwo\r\n` on Windows; adding `-c core.eol=lf` gave `one\ntwo\n`.
  - **Plan and intent:** the command gains `core.eol=lf`, and the clause and INT-0002 now say text files check out with LF on every OS, while an explicit `eol` attribute still applies.
  - **Test:** `test_fetch_file_url_bundles_the_clone` now compares exact bytes on a `* text=auto` fixture.

### R2 C-003: Closing the GUI during a fetch leaves the clone behind
- **Where:** T-002 cleanup clause; T-004
- **Quote:** "**WHEN** a fetch ends, successful or not, **THEN** its `mdeezl-<pid>-<n>` directory **SHALL** no longer exist."
- **Failure mode:** missing-risk
- **Why it matters:** returning from `main` kills the worker before its guard runs.
- **Suggested response:** fix-in-plan or defer-with-rationale
- **Resolution:** fixed for the GUI. The app's `Drop` joins its worker, which `test_dropping_app_waits_for_worker` checks, and a manual check closes the window mid-fetch. CLI Ctrl+C is deferred with rationale: INT-0002's Consequences record that an interrupted CLI run can leave its temporary clone behind.

### R2 C-004: Git LFS smudging is inherited by the clone
- **Where:** T-002 clone clause
- **Quote:** the exact clone command
- **Failure mode:** missing-risk
- **Why it matters:** a host's git-lfs setup would download LFS objects, only for them to be elided, or fail the clone on an exhausted quota.
- **Suggested response:** defer-with-rationale or fix-in-plan
- **Resolution:** fixed. The clone sets `GIT_LFS_SKIP_SMUDGE=1`, so LFS files are bundled as pointer text; the command test asserts it. This is git-lfs's documented behaviour. It was not measured against a live LFS repository, and the test report will say so. INT-0002 records the consequence.

### R2 C-005: The `<repo-name>` rule contradicted its test and allowed unsafe names
- **Where:** T-002 repo-name clause; `test_repo_name`
- **Quote:** "the last non-empty `/`-separated segment"
- **Failure mode:** EARS-vague
- **Why it matters:** applied to the whole URL, `https://host/` yielded `host`, and `a\..\..\x` could escape the output folder on Windows.
- **Suggested response:** fix-in-plan
- **Resolution:** fixed. The rule now works on the URL's path, after the authority, with the query and fragment dropped. `.`, `..`, and any segment containing `\`, `:`, `?` or `#`, become `repository`. `test_repo_name` gains those rows, and INT-0002 records the rule.

### R2 C-006: Credentials embedded in a URL are copied into the output
- **Where:** T-002 success and failure clauses
- **Quote:** "with heading `# <url>`"
- **Failure mode:** missing-risk
- **Why it matters:** a token in the URL would become the first line of a document meant for pasting.
- **Suggested response:** fix-in-plan
- **Resolution:** fixed. Every shown URL — heading, stderr and GUI status — is the display form, with the userinfo removed. `test_display_url_strips_credentials` checks the conversion. `test_cli_unreachable_url_exits_1` now passes `user:SECRET@` and asserts that `SECRET` is absent from stderr. Measured separately: git itself redacted the credentials from its own error line.

### R2 C-007: Nothing required a repaint when a job finishes
- **Where:** T-004
- **Quote:** "`run()` panics while the app keeps requesting repaints"
- **Failure mode:** plan-test-mismatch
- **Why it matters:** eframe's reactive loop would leave `Fetching…` on screen until the mouse moved.
- **Suggested response:** fix-in-plan
- **Resolution:** fixed. A T-004 clause requires a repaint within 100 ms on every working frame. `test_generate_disabled_while_working` asserts the root viewport's `repaint_delay` is at most 100 ms, and INT-0006 records it.

### R2 C-008: A history entry was rewritten, and an anchor was stale
- **Where:** INT-0002 Transition history; INT-0006 Work evidence
- **Quote:** "amended while `planned`, at sprint 3 planning"
- **Failure mode:** intent-drift
- **Why it matters:** the schema forbids erasing entries, and INT-0006's anchor no longer resolved.
- **Suggested response:** fix-in-plan
- **Resolution:** fixed.
  - **INT-0002:** the entry is restored byte for byte, in its committed position. A `diff` confirmed that the `6eae258` history is a prefix of the current one. An appended correction note records that the amendment actually came after the move to `planned`.
  - **INT-0006:** the anchor now points at `#t-003-gui-crate-window-and-fields`. The unnoted Consequences additions got their own history note.

### R2 C-009: The README would contradict the new behaviour
- **Where:** T-002 and T-007; README "Not yet", exit codes and introduction
- **Quote:** "Bundling a git branch, worktree, or a remote repository … out of scope for now."
- **Failure mode:** plan-test-mismatch
- **Why it matters:** the README would both document URL sources and call them unsupported. It would also claim the whole repository is std-only.
- **Suggested response:** fix-in-plan
- **Resolution:** fixed.
  - **T-002 clause:** "Not yet" drops remote repositories, and exit code `1` covers a failed fetch.
  - **T-007 clause:** the introduction scopes std-only to the `mdeezl` crate.
  - **Tests:** `test_readme_documents_url_sources` gains a negative needle. `test_readme_scopes_std_only_to_the_engine` is new.

### R2 C-010: Smaller gaps between the plan and its tests
- **Where:** T-001 to T-004 and their tests
- **Quote:** "hinted `Folder path or https:// URL`"; "`bytes_async`"; "a byte prefix of the block in the new `src/lib.rs`"
- **Failure mode:** plan-test-mismatch
- **Why it matters:** several clauses could pass or fail for reasons their tests would not see.
- **Suggested response:** fix-in-plan
- **Resolution:** fixed.
  - **Placeholder:** asserted through AccessKit.
  - **Rename failure:** provoked by making `<name>.md` a directory, as the second case of `test_failed_job_keeps_existing_output`.
  - **`bytes_async`:** dropped, because it is wasm-only.
  - **Verbatim check:** runs on committed blobs.
  - **T-003 Touches:** lists `Cargo.lock`.
  - **T-002 Intent:** names INT-0006 for the no-console-window clause.

### R3 C-001: Closing the window during a fetch would freeze it on screen
- **Where:** INT-0006 Consequences; T-004 drop clause
- **Quote:** "The process may therefore linger, windowless, until the clone ends."
- **Failure mode:** intent-drift
- **Why it matters:** eframe 0.36 drops the app inside the event loop, before the window, so a join in `Drop` would leave a visible, unresponsive window, marked "Not Responding", until the fetch ends.
- **Suggested response:** fix-in-plan
- **Resolution:** fixed with the critic's option (a).
  - **The close is deferred.** A close requested while working sends `ViewportCommand::CancelClose`, sets the status to `Closing when the current job finishes…` and keeps repainting. `Close` is sent in the frame after the result arrives. An idle close proceeds normally.
  - **The `Drop` join stays** as a backstop.
  - **Checks.** `test_close_while_working_defers_then_closes` pushes `ViewportEvent::Close` and asserts both commands. The manual check now observes the window as well as the temp folder.
  - **INT-0006** records the corrected consequence.

  Both the command and the event were confirmed in egui-0.36.2: `viewport.rs` has `Close` and `CancelClose`, and `viewport_info.rs` has `ViewportEvent::Close`.

### R3 C-002: INT-0002's Alternatives still carried the stale command and a disproved claim
- **Where:** INT-0002 Alternatives
- **Quote:** "`… -c core.autocrlf=false clone --depth 1 --quiet -- <url> <dest>`" / "the same on every OS"
- **Failure mode:** intent-drift
- **Why it matters:** the intent contradicted itself and T-002.
- **Suggested response:** fix-in-plan
- **Resolution:** fixed.
  - The command gains `core.eol=lf`, and the environment gains `GIT_LFS_SKIP_SMUDGE=1`.
  - The disproved "committed bytes are kept" bullet is removed from Alternatives, which are not append-only.
  - The heading reads `# <display-url>`.
  - A history note says the round-1 byte claim is superseded, and leaves that entry as written.

### R3 C-003: The workspace makes every CLI build resolve the GUI's graph
- **Where:** T-003 workspace clause; INT-0006 Consequences
- **Quote:** "`gui/` **SHALL** be the member package `mdeezl-gui`"
- **Failure mode:** intent-drift
- **Why it matters:** a fresh or offline CLI build now needs registry index data for the GUI's crates.
- **Suggested response:** fix-in-plan or defer-with-rationale
- **Resolution:** recorded. INT-0006's Consequences state it, labelled as reasoned from Cargo's workspace resolution and not measured. The workspace design is kept: a separate GUI workspace would need its own CI gates and lockfile, and the user approved the workspace member. Offline CLI builds are not an INT-0001 criterion.

### R3 C-004: Smaller gaps
- **Where:** T-003 Notes; T-002 help clause; T-004; the manual checks; `test_readme_documents_url_sources`
- **Quote:** "`Browse…` opens `rfd::FileDialog::pick_folder`"; "private repositories are not supported"
- **Failure mode:** plan-test-mismatch
- **Why it matters:** there was an untraced manual check, a weak negative needle, inaccurate help text, a GUI display-form gap, and a local name that could escape the output folder.
- **Suggested response:** fix-in-plan
- **Resolution:** fixed.
  - **Browse…** has an EARS clause verified by hand. Every manual check has a traceability row.
  - **The README needle** normalises whitespace and matches `remote repositor`.
  - **The help and README** now say mdeezl "never prompts for credentials and uses no credential helper". INT-0002 records that a URL carrying credentials can clone a private repository.
  - **`test_generate_unreachable_url_writes_nothing`** uses `user:SECRET@` and asserts `SECRET` never appears.
  - **A new T-004 clause** makes the output stem `Bundle.name` only when that is a single normal component, and otherwise `bundle`. `test_output_stem_is_one_component` covers it.

### R4 C-001: The close deferral fails when the window is minimised, unless it runs in `App::logic`
- **Where:** T-004 close and working clauses
- **Quote:** "send `ViewportCommand::Close` in the frame after the worker's result arrives"
- **Failure mode:** missing-risk
- **Why it matters:** eframe runs only `logic` while the window is minimised. A deferral in `ui` would be skipped, and the window would either freeze or never close.
- **Suggested response:** fix-in-plan
- **Resolution:** fixed.
  - **Placement.** A new clause puts result polling, the close deferral and the repaint request in `App::tick(&egui::Context)`, called from `eframe::App::logic`. The eframe 0.36.2 docs confirm that `logic` runs when the window is hidden, through `Context::run_logic`.
  - **Wake-up.** The worker calls `request_repaint` on a clone of the context once it has sent its result, which also wakes a hidden window.
  - **Tests.** The kittest closure calls `tick`, then `ui`. `test_close_deferral_runs_in_logic_alone` drives `run_logic` alone and reads `LogicOutput.viewport_commands`; that field was confirmed in `output.rs`.

### R4 C-002: Nothing tested that the window closes after its own `Close`
- **Where:** `test_close_while_working_defers_then_closes`
- **Quote:** "a later frame's commands contain `Close`"
- **Failure mode:** plan-test-mismatch
- **Why it matters:** kittest does not echo `Close`, so an app that cancels every pending close would pass the test and never close.
- **Suggested response:** fix-in-plan
- **Resolution:** fixed. The test polls each frame until `Close` appears, within 10 s. It then pushes `ViewportEvent::Close`, as eframe would echo it, and asserts that no `CancelClose` follows. The clause adds "**SHALL NOT** cancel the close request that its own `Close` produces".

### R4 C-003: The two status clauses conflicted while a close was pending
- **Where:** T-004 working and close clauses
- **Quote:** "`Fetching…` if `mdeezl::is_url(source)`" vs "`Closing when the current job finishes…`"
- **Failure mode:** EARS-vague
- **Why it matters:** both clauses applied at once, and the source field stays editable mid-job.
- **Suggested response:** fix-in-plan
- **Resolution:** fixed.
  - **Precedence.** The working clause now reads "unless a close is pending", and the closing status persists until the job ends.
  - **Stable label.** `Fetching…` or `Bundling…` is chosen from the source captured at Generate.
  - **Test.** The close test asserts that the closing status holds across two more gated steps.

### R4 C-004: The stem rule depended on the OS
- **Where:** T-004 stem clause; `test_output_stem_is_one_component`
- **Quote:** "a single normal path component"
- **Failure mode:** plan-test-mismatch
- **Why it matters:** on Linux, `a\b` and `C:\` are single normal components, so the test would fail on ubuntu only.
- **Suggested response:** fix-in-plan
- **Resolution:** fixed. The rule is stated by characters, on every OS. An empty name, `.`, `..`, or one containing `/`, `\` or `:` becomes `bundle`. A `c:x` row was added to the test.

### R4 C-005: The deferred close could wait forever
- **Where:** INT-0006 Consequences; T-004
- **Quote:** "A clone that stalls indefinitely keeps the window open until git gives up."
- **Failure mode:** intent-drift
- **Why it matters:** git has no default stall timeout, and a panicked worker would leave `working` set forever.
- **Suggested response:** fix-in-plan
- **Resolution:** fixed.
  - **Measured first, with a local server that accepts and never answers:**
    - without low-speed settings, the clone waited until an outside `timeout` killed it at 12 s;
    - with `http.lowSpeedLimit=1` and `http.lowSpeedTime=5`, it failed in 5.2 s with "Operation too slow".
  - **The stall bound.** The clone now carries `http.lowSpeedLimit=1` and `http.lowSpeedTime=30`. They are pinned by the command test, and exercised by the new `test_stalled_server_fails_the_fetch`, which shortens the window under `cfg(test)`.
  - **A lost worker.** A disconnected result channel ends the job with `The job stopped unexpectedly`, which `test_worker_panic_ends_job_with_error` checks.
  - **The intents.** INT-0006 states the real bound: a stall is cut off at 30 s, and otherwise the job's own length. It also states that ending the process from outside leaves the clone. INT-0002 records the stall setting.

### R4 C-006: Smaller gaps
- **Where:** the help test, INT-0002 wording, traceability labels and the manual checks
- **Quote:** "`never prompts for credentials`"; "The GUI waits for its worker"; "Documented source forms"; "a live GitHub URL"
- **Failure mode:** plan-test-mismatch
- **Why it matters:** there was an untested help item, a stale consequence, two labels that are not criteria, and an unobservably short manual fetch.
- **Suggested response:** fix-in-plan
- **Resolution:** fixed.
  - **Help and README tests** assert `credential helper`.
  - **INT-0002's consequence** now describes the deferred close, and its cross-reference names the display-form bullet.
  - **The two traceability rows** now cite INT-0002's real criteria and INT-0006 "Documented".
  - **The manual checks** use `https://github.com/git/git`, whose shallow clone takes several seconds. The measured duration will be recorded.

### R5 C-001: Environment variables could switch the stall limit off
- **Where:** T-002 clone and stall clauses
- **Quote:** "`GIT_ASKPASS`, `SSH_ASKPASS` and the 15 repository-local git variables removed"
- **Failure mode:** missing-risk
- **Why it matters:** git documents that `GIT_HTTP_LOW_SPEED_LIMIT` and `GIT_HTTP_LOW_SPEED_TIME` override `http.lowSpeed*`.
- **Suggested response:** fix-in-plan
- **Resolution:** fixed. Both are removed from the clone's environment, `test_clone_command_is_hardened` asserts their removal, and INT-0002 records it.

### R5 C-002: The `cfg(test)` low-speed override had to be per-thread
- **Where:** `test_stalled_server_fails_the_fetch` and `test_clone_command_is_hardened`
- **Quote:** "through a `cfg(test)`-only override"
- **Failure mode:** hidden-dep
- **Why it matters:** a process-wide override would race with the parallel pinning test.
- **Suggested response:** fix-in-plan
- **Resolution:** fixed.
  - **Mechanism.** The window is a production constant of 30. Under `cfg(test)` it is read through a thread-local that defaults to that constant, the `GIT_SPAWNS` pattern.
  - **Scope.** Only the stalled test lowers it, on its own thread.

### R5 C-003: The logic path was tested through a hand-built mirror, and two repaint claims were untested
- **Where:** T-004 `tick`/`logic` clause; the GUI harness
- **Quote:** "whose closure calls `app.tick(ui.ctx())` and then `app.ui(ui)`. That mirrors eframe"
- **Failure mode:** plan-test-mismatch
- **Why it matters:** an `eframe::App::logic` that forgot to call `tick` would pass, and neither the worker's wake-up nor the repaint under `run_logic` was checked.
- **Suggested response:** fix-in-plan
- **Resolution:** fixed.
  - **Real `logic`.** The GUI tests use `Harness::new_eframe`, which calls the real `logic` and then `ui`. That needs `egui_kittest`'s `eframe` feature, measured at +1 crate (143 → 144), and `test_gui_manifest_dependencies` pins it.
  - **Repaint claims.** `test_close_deferral_runs_in_logic_alone` records `set_request_repaint_callback` calls. It asserts the tick's request of at most 100 ms, and the worker's zero-delay wake-up with no tick run.

### R5 C-004: The `bundle` fallback name existed only in sprint prose
- **Where:** T-004 output-name clause; INT-0006
- **Quote:** "In those cases it **SHALL** be `bundle`."
- **Failure mode:** intent-drift
- **Why it matters:** it departs from INT-0006's "`<name>` is the folder's or repository's name" without a record.
- **Suggested response:** fix-in-plan
- **Resolution:** fixed. INT-0006's Consequences record the `bundle.md` fallback, which cases trigger it, and why: the file always stays inside the output folder.

### R5 C-005: Smaller gaps
- **Where:** the status test; the help and README needles; the gated-job tests; the Harness bullet; INT-0006 wording
- **Quote:** "*captured when Generate was activated*"; "`git`"; "Every test holds the app through `new_ui_state`"
- **Failure mode:** plan-test-mismatch
- **Why it matters:** there was an untested capture, a vacuous needle, gated jobs writing into the runner's home folder, an inaccurate "every test", and a stall claim wider than its evidence.
- **Suggested response:** fix-in-plan
- **Resolution:** fixed.
  - **The status test** edits the source mid-job and asserts that the label holds.
  - **The help and README needles** are URL-specific: `shallow`, `temporary`, `removed`, and the credential phrases.
  - **Every gated-job test** sets a test-owned output folder.
  - **The Harness bullet** names its one exception.
  - **INT-0006** now says `eframe::App::logic` calls `tick`.
  - **The stall bound is scoped** in both intents and T-002 to transfers that stall once connected. A stalled connect or TLS handshake is left to curl's connect handling, which was not measured.

## Confidence
proceed-with-caveats
