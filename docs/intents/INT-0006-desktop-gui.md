# INT-0006 — Minimal desktop GUI

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0006
- **State:** planned
- **Work evidence:** [Sprint 3 build plan](../sprints/s3/sprint-plans/build-plan.md), [T-003 GUI crate, controls and worker](../sprints/s3/sprint-plans/build-plan.md#t-003-gui-crate-controls-and-worker)
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
  files or image decoders. Whether it reads as a weasel is a human judgement,
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
- **An image-file logo, such as SVG or PNG.** Rendering it needs a decoder: an
  SVG renderer measured at 31 more crates. Drawing the weasel with egui's own
  shapes costs none.

## Consequences

- **The zero-dependency property narrows.** It applies to the `mdeezl` crate,
  which is the CLI and the engine. The GUI crate depends on `eframe` and a
  native folder dialog. Measured on Windows, the GUI crate builds roughly 130
  crates.
- **CI grows.** Both legs compile the GUI, which takes about 35 s cold on the
  development host. The Linux leg may need system libraries for windowing,
  which is to be confirmed in CI.
- **On Linux the window is X11 only.** eframe's and rfd's Wayland features are
  left out. X11 and GL are loaded at run time, so no system packages are needed
  to build, and Wayland desktops run the app through XWayland. Native Wayland
  is one feature flag away if it is ever wanted.
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
