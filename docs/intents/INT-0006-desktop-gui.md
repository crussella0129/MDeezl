# INT-0006 — Minimal desktop GUI

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0006
- **State:** active
- **Work evidence:** [Sprint 3 build plan](../sprints/s3/sprint-plans/build-plan.md), [T-003 GUI crate, window and fields](../sprints/s3/sprint-plans/build-plan.md#t-003-gui-crate-window-and-fields)
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent

A small egui desktop app, so the user can see what MDeezl does and learn to use
it without reading the CLI help. It is deliberately minimal to start. The
whole workflow is:

1. name a local folder or a repository URL;
2. tick a few boxes for the output format;
3. choose the folder the output goes into;
4. get the repository's `.md`.

It carries a **weasel** as its hero graphic and logo, because *weasel* rhymes
with *MDeezl*.

**Non-goals for now:**

- settings that persist between runs;
- the `--exclude` and `--include` pattern lists;
- private-repository authentication;
- installers, packaging or code signing;
- any second rendering engine. The app drives the same engine as the CLI, and
  never a copy of it.

## Acceptance criteria

- **One window, four controls.** The window shows the MDeezl name and the
  weasel logo, and holds exactly the controls of the workflow:
  - a source field that takes a local folder path or an `https://` repository
    URL, with a folder picker;
  - the format options: a wrap mode (fence, inline or none), and whether to
    respect the repository's `.gitignore`;
  - an output-folder field with a folder picker;
  - a Generate button.
- **The CLI's own output.** Generate writes `<name>.md` into the output folder.
  `<name>` is the folder's or repository's name. The file is byte-identical to
  what the CLI writes for the same source and options.
- **Responsive while working.** The window stays responsive while a bundle is
  built or a repository is fetched. A status line reports progress, then either
  the written path or the error.
- **No partial files.** A missing folder, an unreachable or unauthorised URL,
  or an unwritable output folder is reported in the window, and no file is left
  behind.
- **The CLI stays dependency-free.** The GUI's dependencies live in its own
  crate, and `cargo tree -p mdeezl` still reports only `mdeezl`.
- **Tested in CI on both OSes.** CI builds the GUI and drives it headlessly on
  `ubuntu-latest` and `windows-latest`.
- **Documented.** The README says how to launch the app.
- **The weasel is judged by a person.** The app draws it itself, with no image
  file and no image decoding for the logo. Whether it reads as a weasel is a human judgement,
  so the user verifies it from a screenshot.

## Rationale

MDeezl has been a CLI only. The user wants to *see* what it does, and to learn
how to use it by using it. A window with four controls teaches the tool's
whole surface at a glance: a source, a format, a destination and a button. A
tool whose output is meant for agents should still be easy for a person to
drive.

The user asked for egui by name. It keeps the app in Rust and lets it call the
engine in-process, so the GUI and CLI cannot drift apart. It also adds no web
stack to a project that has so far had no runtime dependencies at all.

## Alternatives

- **Electron.** The user's general default for a first desktop build. Not
  chosen: the user asked for egui explicitly for this project, and Electron
  would put a second language and runtime between the UI and a Rust engine.
- **Tauri.** Would still need a web frontend. Its advantage over Electron is a
  smaller bundle, which is not this project's constraint.
- **The GUI spawns the `mdeezl` binary.** It keeps the core untouched, but two
  executables must then ship and be found together. The core's stderr notices
  would also have to be screen-scraped.
- **An SVG logo.** Its renderer measured at 31 more crates.
- **A PNG logo.** It would cost almost nothing, because eframe already links an
  image decoder with PNG support. It was not chosen because no raster weasel
  exists to ship. Code-drawn shapes stay sharp at any display scale, and one
  shape list feeds both the hero and the window icon.
- **The wgpu renderer**, eframe's default. It measured at 43 more crates than
  glow. Its advantage is a software fallback on machines without OpenGL 2+.
  Not chosen for a minimal first build; switching is a one-feature change.

## Consequences

- **The zero-dependency property narrows.** It applies to the `mdeezl` crate,
  which is the CLI and the engine. The GUI crate depends on `eframe` and a
  native folder dialog. Measured on Windows, the GUI crate builds roughly 130
  crates.
- **CI grows.** Both legs compile the GUI, which takes about 35 s cold on the
  development host.
- **The Linux build needs no system packages.** That is expected from the
  sources, and the first CI run confirms it: the windowing, GL, Wayland and
  D-Bus libraries are all loaded at run time.
- **The Linux window supports both X11 and Wayland.** eframe's `wayland`
  feature must stay on. `accesskit` already brings in winit's Wayland backend,
  which picks Wayland whenever `WAYLAND_DISPLAY` is set, and without eframe's
  `wayland` feature glutin cannot open that window.
- **The window needs OpenGL 2+.** That comes with the glow renderer. A machine
  without GPU drivers may fail to open it; the wgpu renderer is the remedy.
- **Some sources are written as `bundle.md`.** A name that is empty, `.` or
  `..`, or that contains `/`, `\` or `:`, is written as `bundle.md`. That
  covers a bundled filesystem root, whose name falls back to the whole path,
  and a Linux folder with `:` in its name. The file therefore always stays
  inside the output folder.
- **Generate replaces an existing `<name>.md`.** The document is written to a
  temporary sibling file and renamed over the target in one step, so a failed
  run never damages an earlier good file.
- **Closing the window during a fetch defers the close.** While a job runs,
  a close request is cancelled. The status then says the window will close
  when the job ends, the window stays responsive, and it closes itself when
  the worker finishes, so the engine's temporary clone is removed first. The
  app's drop also joins the worker, as a backstop.
- **How long a close can wait.** A fetch whose transfer stalls once connected
  is cut off by the engine's 30-second low-speed limit. A stalled connect or
  TLS handshake falls under curl's own connect handling. A fetch that keeps moving slowly, or a
  very large local bundle, can keep the window open for as long as it takes.
  Ending the process from outside, for example with Task Manager, then leaves
  the temporary clone behind, as Ctrl+C does in the CLI.
- **The polling lives in `eframe::App::logic`.** Result polling, the close
  deferral and the repaint request run in the app's own `tick`, which
  eframe's `logic` hook calls. eframe calls that hook even while the window is
  minimised. A minimised window still closes, and still
  collects its result.
- **Every CLI build now resolves the GUI's dependency graph.** Cargo keeps one
  lockfile per workspace, so building or installing the `mdeezl` crate on a
  fresh machine needs registry index data for the GUI's crates too, but
  downloads none of them. This is reasoned from Cargo's workspace resolution
  and not measured. `cargo tree -p mdeezl` still reports only `mdeezl`.
- **The window repaints on its own while working**, so the result appears
  without the user moving the mouse.
- **The CI gate lines change.** `fmt` and `clippy` must cover the workspace,
  and [INT-0005](INT-0005-reproducible-toolchain.md)'s guard test pins those
  lines, so the two change together.
- **The engine becomes a library.** The CLI becomes a thin wrapper over it.
  Notices the engine used to print to stderr must reach the GUI as data.
- **Remote URLs are fetched with git.** A URL source is fetched through
  [INT-0002](INT-0002-git-and-remote-sources.md)'s remote half, so git is
  needed on PATH for URLs, as it already is for gitignore support.

## Transition history
- 2026-09-19: created as `proposed` at the sprint 3 boundary. The user asked
  for a minimal egui frontend with a weasel logo in place of the backlog items
  that were offered.
- 2026-09-19: `proposed → planned` for sprint 3. The plan was approved, and
  work evidence was attached: tasks T-001 and T-003 to T-006.
- 2026-09-19: amended while `planned`, from the sprint 3 plan critique, which
  returned `block`. The critic verified each change against the crate sources.
  - **Wayland back on.** The X11-only choice would have failed to open a window
    on Wayland desktops.
  - **Alternatives corrected.** The wgpu and PNG alternatives now state their
    real costs, and the OpenGL 2+ runtime requirement is recorded.
  - **Logo criterion reworded.** It now says "no image decoding for the logo",
    because eframe links a PNG decoder regardless.
  - **Overwrite rule added.** Generate replaces an existing file through a
    temporary sibling and a rename.

  The plan's tasks are now T-001 and T-003 to T-007.
- 2026-09-19: correction. The Consequences entry "The CI gate lines change",
  and a since-replaced X11-only entry, were added at sprint 3 planning without
  a history note; this is that note. The Work-evidence anchor was repointed to
  T-003's new heading.
- 2026-09-19: amended while `planned`, from the second sprint 3 plan critique.
  Two Consequences were added: closing the window waits for a running fetch,
  and the window repaints on its own while working.
- 2026-09-19: amended while `planned`, from the third sprint 3 plan critique.
  - The close consequence was corrected. eframe drops the app inside its event
    loop, so joining there would freeze a still-visible window. A close during
    a job is now cancelled and completed when the job ends.
  - The workspace-resolution consequence for CLI builds was added.
- 2026-09-19: amended while `planned`, from the fourth sprint 3 plan critique.
  - The close consequence now states its real bound: the engine's stall
    limit, and otherwise the job's own length.
  - It records that the app's polling lives in `logic`, so a minimised window
    behaves the same.
- 2026-09-19: amended while `planned`, from the fifth sprint 3 plan critique.
  - The `bundle.md` fallback name and its reason are recorded.
  - The `logic`/`tick` wording is made exact.
  - The stall bound is scoped to transfers that stall once connected.
- 2026-09-19: `planned → active`; the sprint 3 Build Phase began, with T-001.
