# Plan Critique — Sprint 3

Round 1 of the read-only plan critic, from the installed bundle's
`prompts/plan-critic.md`. The primary agent's resolution follows each concern.
Where a fix rests on a measurement, the measurement was taken before the plan
changed.

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

## Confidence
block
