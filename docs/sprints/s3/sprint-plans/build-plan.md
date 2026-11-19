Finalized - DO NOT EDIT

# Sprint 3 Build Plan

## Intents
- [INT-0006](../../../intents/INT-0006-desktop-gui.md) — state: planned. All eight acceptance criteria are covered:
  1. one window, four controls;
  2. the CLI's own output;
  3. responsive while working;
  4. no partial files;
  5. the CLI stays dependency-free;
  6. tested in CI on both OSes;
  7. documented;
  8. the weasel judged by a person.
- [INT-0002](../../../intents/INT-0002-git-and-remote-sources.md) — state: planned. The **remote half** is covered: criteria 1 and 3–6, for URL sources. Criterion 2, reading a git ref without mutating the working tree, belongs to the git-ref half, which stays in backlog T-103. INT-0002 therefore cannot be `realized` at this sprint's close.
- [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md) — state: realized. Not advanced, but constrained:
  - local-path output must stay byte-identical;
  - the `mdeezl` crate keeps an empty `[dependencies]` table;
  - both CI legs remain.

  Its chapter records the URL-heading exception and the narrowed dependency property.
- [INT-0005](../../../intents/INT-0005-reproducible-toolchain.md) — state: realized. Not advanced, but constrained. Its guard test pins the exact CI gate lines, which T-005 changes together with the test, re-running the mutation check. eframe's minimum of Rust 1.95 now bounds a pin; its chapter records this.

## Schema Tree
- Sprint Goal: a minimal egui window over the CLI's own engine, taking a path or URL
  - Engine
    - T-001: Engine library split
    - T-002: URL sources
  - Desktop app
    - T-003: GUI crate, window and fields
    - T-004: Generate: worker, status and safe write
    - T-006: Weasel hero and window icon
  - Delivery
    - T-005: CI gates cover the workspace
    - T-007: README desktop-app section and scope

## Execution Sequence

### T-001: Engine library split
- **Intent:** [INT-0006](../../../intents/INT-0006-desktop-gui.md)
- **Touches:** `src/lib.rs` (new; the content of `src/main.rs` moves here), `src/main.rs`, `tests/cli.rs`
- **Depends on:** (none)
- **Acceptance criterion:** INT-0006 "The CLI's own output": the engine is callable in-process, so GUI and CLI output match by construction. INT-0006 "The CLI stays dependency-free".
- **Success criterion (EARS):**
  - **WHEN** the package is built, **THEN** it **SHALL** have a library target `mdeezl` holding the engine, and a binary target `mdeezl` whose `src/main.rs`, ignoring comments and blank lines, is one `fn main` that evaluates `std::process::exit(mdeezl::cli(std::env::args().skip(1)))`.
  - **WHEN** `mdeezl::bundle(&Options)` is called for a readable local directory, **THEN** it **SHALL**:
    - return `Bundle { name, document, notices }`;
    - make `document` equal to the bytes the CLI writes with `-o` for the same options, under the same environment;
    - make `name` the scanned directory's name;
    - put in `notices` the text of each notice the CLI prints, without the `mdeezl: ` prefix;
    - write nothing to stdout or stderr.
  - **WHEN** the CLI runs, **THEN** its stdout, its stderr (each notice printed as `mdeezl: <notice>`) and its exit codes 0, 1 and 2 **SHALL** be unchanged, and all 122 existing tests **SHALL** pass.
  - **WHEN** the move is checked, **THEN** the new `#[cfg(test)] mod tests` block in `src/lib.rs` **SHALL** begin with the old block from `24442f1:src/main.rs`, without its closing brace, byte for byte. New unit tests are appended after the old ones.
  - **WHEN** `cargo tree -p mdeezl` runs, **THEN** it **SHALL** report only `mdeezl`, and the root `[dependencies]` table **SHALL** stay empty.
- **Notes:**
  - **Public surface.** `Options` (fields become `pub`), `Wrap`, `Sink`, `Bundle`, `bundle` and `cli`. `is_url` joins it in T-002. Everything else stays private.
  - **Signatures stay.** `render_document(&Node, &Options)` keeps its signature because the moved tests call it. A new private `render_titled(title, &Node, &Options)` does the work, and `render_document` delegates to it.
  - **`run` stays**, as `bundle`, then the printed notices, then the sink write.
  - **Imports.** An import that only the moved tests use through `use super::*` is kept, and marked `#[cfg_attr(not(test), allow(unused_imports))]` if the non-test build no longer uses it. It is never removed. The attribute goes once T-002 uses the import.
  - **`tests/cli.rs`** receives insertions only, in T-001 through T-004.

### T-002: URL sources
- **Intent:** [INT-0002](../../../intents/INT-0002-git-and-remote-sources.md); [INT-0006](../../../intents/INT-0006-desktop-gui.md) for the no-console-window clause
- **Touches:** `src/lib.rs`, `tests/cli.rs`, `README.md`
- **Depends on:** T-001
- **Acceptance criterion:** INT-0002's remote half:
  - a URL source named on the command line, recorded in the document;
  - addressed by URL;
  - failures reported on stderr, with a non-zero exit and no partial document;
  - never waiting on a person;
  - the temporary copy removed.
- **Success criterion (EARS):**
  - **WHEN** `mdeezl::is_url(s)` is called, **THEN** it **SHALL** return true exactly when `s` begins with `https://`, `http://` or `file://`. **WHEN** the source argument is not a URL — including `ftp:/weird`, `git@host:x/y` and `C:\x` — **THEN** mdeezl **SHALL** treat it as a local path, as before. Detection **SHALL** use the argument's text (`to_str`), never path components.
  - **WHEN** a URL source is fetched, **THEN** the clone command **SHALL** be exactly `git -c credential.helper= -c core.askPass= -c core.autocrlf=false -c core.eol=lf -c http.lowSpeedLimit=1 -c http.lowSpeedTime=30 clone --depth 1 --quiet -- <url> <dest>`. It **SHALL** run with:
    - `GIT_TERMINAL_PROMPT=0`, `GCM_INTERACTIVE=never`, `GIT_ALLOW_PROTOCOL=https:http:file` and `GIT_LFS_SKIP_SMUDGE=1` set;
    - `GIT_ASKPASS`, `SSH_ASKPASS`, `GIT_HTTP_LOW_SPEED_LIMIT`, `GIT_HTTP_LOW_SPEED_TIME` and the 15 repository-local git variables removed. git lets the two low-speed variables override the `-c` settings, so they are removed;
    - `<dest>` as `<base>/mdeezl-<pid>-<n>/<repo-name>`, where `<base>` is `std::env::temp_dir()` in production.
  - **WHEN** `<repo-name>` is derived from a URL, **THEN** it **SHALL** be derived as follows:
    1. take the URL's path: everything after the scheme's `//` and the authority up to the next `/`, with any `?…` query or `#…` fragment dropped;
    2. take the last non-empty `/`-separated segment of that path, with one trailing `.git` removed;
    3. use `repository` if no segment remains, or if the segment is `.` or `..`, or if it contains `\`, `:`, `?` or `#`.

    The name is used as a path component, so it can never leave its folder.
  - **WHEN** a URL is shown — in the document heading, on stderr or in the GUI — **THEN** it **SHALL** be the display form, with any `user[:password]@` userinfo removed from its authority. The clone itself uses the URL as given.
  - **WHEN** the clone succeeds, **THEN** the document **SHALL** be the clone's bundle, with heading `# <display-url>` and tree root line `<repo-name>/`, and `Bundle.name` **SHALL** be `<repo-name>`. Line endings **SHALL** be the same on every OS: `core.autocrlf=false` and `core.eol=lf` check text files out with LF, and a file with an explicit `eol` attribute follows it.
  - **WHEN** an HTTP(S) transfer, once connected, moves less than 1 byte per second for 30 seconds, **THEN** the fetch **SHALL** fail rather than wait. Measured: without the setting, a server that accepts and never answers held the clone until an outside `timeout` killed it; with a 5-second window it failed in 5.2 s with "Operation too slow".
  - **WHEN** the fetch fails — a missing, unauthorised, unreachable or stalled repository, a protocol outside the allow-list, or git not found — **THEN** the CLI **SHALL** exit 1, with empty stdout and one stderr line `mdeezl: could not fetch <display-url>: <reason>`. `<reason>` **SHALL** be one of:
    - git's first stderr line beginning `fatal: `, with that prefix removed;
    - if no such line exists, git's last non-empty stderr line;
    - if git is not found, `git was not found on PATH`.

    It is measured: a missing repository's first `fatal:` line is `'<path>' does not appear to be a git repository`, and its last line is boilerplate.
  - **WHEN** a fetch ends, successful or not, **THEN** its `mdeezl-<pid>-<n>` directory **SHALL** no longer exist.
  - **WHEN** mdeezl spawns git on Windows, for `check-ignore` or `clone`, **THEN** it **SHALL** pass `CREATE_NO_WINDOW`, so a windowed parent never flashes a console.
  - **WHEN** `--help` or the README is read, **THEN** it **SHALL** say four things:
    - a URL may be given in place of PATH;
    - it needs git;
    - it is shallow-cloned into a temporary folder that is removed afterwards;
    - mdeezl never prompts for credentials and uses no credential helper.

    The README's exit-code line **SHALL** count a failed fetch under `1`. Its "Not yet" section **SHALL** no longer list remote repositories; it keeps the git-ref half and INT-0003.
- **Notes:**
  - The fetch is a private `fetch(url, base)`, so unit tests pass their own base folder. `bundle` passes `temp_dir()`.
  - The low-speed window is a constant, 30. Under `cfg(test)` it is read through a thread-local that defaults to that constant, the same pattern as `GIT_SPAWNS`. Only `test_stalled_server_fails_the_fetch` lowers it, on its own thread, so the parallel `test_clone_command_is_hardened` always sees 30.
  - The stall bound was measured for a server that accepts the connection and then stays silent. A stalled TCP connect or TLS handshake is governed by curl's own connect handling, which was not measured.
  - Under `cfg(test)`, the clone gets the same config isolation as `git_ignored`.
  - Stdin is null. `Command` does not expose stdin, so that is left to code review.
  - With `GIT_LFS_SKIP_SMUDGE=1`, an LFS-tracked file is bundled as its small pointer text, and no LFS object is downloaded. This is git-lfs's documented behaviour, and INT-0002 records it.
  - An interrupted CLI run, such as Ctrl+C, can leave its temporary clone behind. INT-0002 records this limitation.
  - `create_dir` retries with the next `<n>` if a stale directory exists.
  - Every test builds its `file://` URL the same way: canonicalise, strip a `\\?\` prefix, turn `\` into `/`, then prefix `file://` to a path beginning `/` or `file:///` to a drive path.

### T-003: GUI crate, window and fields
- **Intent:** [INT-0006](../../../intents/INT-0006-desktop-gui.md)
- **Touches:** `Cargo.toml` (`[workspace] members`), `Cargo.lock` (gains the GUI's dependency graph), `gui/Cargo.toml`, `gui/src/lib.rs`, `gui/src/main.rs`, `gui/tests/app.rs`
- **Depends on:** T-001, T-002
- **Acceptance criterion:** INT-0006 "One window, four controls" and "The CLI stays dependency-free".
- **Success criterion (EARS):**
  - **WHEN** the workspace is built, **THEN** `gui/` **SHALL** be the member package `mdeezl-gui`. Its dependencies **SHALL** be exactly:
    - `mdeezl`, by path;
    - `eframe` 0.36, with `default-features = false` and features exactly `glow`, `default_fonts`, `accesskit`, `x11` and `wayland`;
    - `rfd` 0.17, with `default-features = false` and features exactly `xdg-portal`.

    Its dev-dependencies **SHALL** be exactly `egui_kittest`, with the single feature `eframe`, which measured at one more crate. The root `[dependencies]` table stays empty.
  - **WHEN** the app is shown, **THEN** it **SHALL** present exactly these interactive widgets:
    - two text fields: one labelled `Source` and hinted `Folder path or https:// URL`, and one labelled `Output folder`;
    - three buttons: `Browse…` for each field, and `Generate`;
    - three radio buttons: `fence`, `inline` and `none`;
    - one checkbox: `Respect .gitignore`.

    Beside them sits the heading `MDeezl`.
  - **WHEN** the app starts, **THEN** `fence` **SHALL** be selected and the checkbox checked. The output field **SHALL** hold the home folder: `USERPROFILE` on Windows, `HOME` elsewhere, `.` if unset.
  - **WHEN** the fields hold a source, a wrap choice and a checkbox state, **THEN** `App::options()` **SHALL** equal `Options { root: source, wrap: choice, use_gitignore: checkbox, ..Options::default() }`.
  - **WHEN** a folder is dropped onto the window, **THEN** the source field **SHALL** become that folder's path.
  - **WHEN** a `Browse…` button is clicked, **THEN** a native folder picker (`rfd::FileDialog::pick_folder`) **SHALL** open, and a chosen folder **SHALL** replace its field's text. A cancelled picker leaves the field unchanged. A native dialog cannot be driven headlessly, so this clause is verified by hand.
- **Notes:**
  - `gui/src/main.rs` calls `mdeezl_gui::native_options()` and `eframe::run_native`. A release build sets `windows_subsystem = "windows"`.
  - `wayland` is on because `accesskit` already brings winit's Wayland backend, which picks Wayland whenever `WAYLAND_DISPLAY` is set. Without eframe's `wayland`, glutin could not open that window. The critic verified this.

### T-004: Generate: worker, status and safe write
- **Intent:** [INT-0006](../../../intents/INT-0006-desktop-gui.md)
- **Touches:** `gui/src/lib.rs`, `gui/tests/app.rs`
- **Depends on:** T-003
- **Acceptance criterion:** INT-0006:
  - "The CLI's own output";
  - "Responsive while working";
  - "No partial files".
- **Success criterion (EARS):**
  - **WHEN** `Generate` is activated with a non-empty source and output folder, **THEN** `App::generate()` **SHALL** mark the app working before it returns, and run the app's job on a worker thread. The job defaults to `mdeezl::bundle`, and tests inject one.
  - **WHEN** the job succeeds, **THEN** the app **SHALL**:
    1. write the document to a temporary sibling, `<output>/.<name>.md.mdeezl-tmp`;
    2. rename it over `<output>/<name>.md`;
    3. show `Wrote <path> (<size>)` and every notice.

    `<size>` is `format_size(bytes)`: `N bytes` below 1024, `N.N KB` below 1 MiB, `N.N MB` otherwise. The file's bytes **SHALL** equal `bundle(&options).document`.
  - **WHEN** the app is working, **THEN**:
    - `Generate` **SHALL** be disabled;
    - unless a close is pending, the status **SHALL** read `Fetching…` if `mdeezl::is_url` held for the source *captured when Generate was activated*, and `Bundling…` otherwise;
    - each tick **SHALL** request a repaint within 100 ms;
    - the worker **SHALL** call `request_repaint` on a clone of the `egui::Context` once it has sent its result.

    The window therefore shows the result without waiting for input, minimised or not.
  - **WHEN** eframe drives the app, **THEN** three things **SHALL** live in one inherent `App::tick(&egui::Context)`, which `eframe::App::logic` calls: polling for the worker's result, the close deferral, and the repaint request. eframe runs `logic` even while the window is minimised or hidden, and then runs no `ui` pass. Drawing stays in `eframe::App::ui`. The GUI tests drive the app through `Harness::new_eframe`, which calls the real `logic` and then the real `ui`, as eframe does.
  - **WHEN** the worker's result channel disconnects without a result, as when the job panics, **THEN** the tick **SHALL** end the job, with the status `The job stopped unexpectedly`. The app **SHALL** stop working, and no output file **SHALL** be written.
  - **WHEN** the window's close is requested while the app is working, **THEN** the app **SHALL**:
    - send `ViewportCommand::CancelClose`;
    - set the status to `Closing when the current job finishes…`, which persists until the job ends;
    - keep repainting;
    - send `ViewportCommand::Close` in the tick after the worker's result, or its disconnection, arrives.

    It **SHALL NOT** cancel the close request that its own `Close` produces.

    **WHEN** a close is requested while idle, **THEN** it **SHALL** proceed normally.
  - **WHEN** the app is dropped while its worker runs, **THEN** the drop **SHALL** wait for the worker to finish, as a backstop, so the engine's temporary-clone guard runs before the process exits.
  - **WHEN** the job fails, or the write or rename fails, **THEN** the status **SHALL** show the error, and the temporary sibling **SHALL** be removed. An existing `<name>.md` **SHALL** be left exactly as it was; if none existed, none **SHALL** exist. A rename failure is provoked by making `<name>.md` a directory.
  - **WHEN** `Generate` is activated with an empty source, **THEN** the status **SHALL** read `Choose a folder or URL`. With an empty output folder it **SHALL** read `Choose an output folder`. In both cases no job **SHALL** start.
  - **WHEN** the output file is named, **THEN** its stem **SHALL** be `Bundle.name`, unless the name is empty, `.` or `..`, or contains `/`, `\` or `:`. In those cases it **SHALL** be `bundle`. The rule is stated by characters, so it is the same on every OS. A bad name arises when a filesystem root is bundled, because its name falls back to the whole path. With this rule the file always lands inside the output folder.
- **Notes:** a successful Generate replaces an existing `<name>.md` in one rename, which INT-0006 records.

### T-005: CI gates cover the workspace
- **Intent:** [INT-0006](../../../intents/INT-0006-desktop-gui.md)
- **Touches:** `.github/workflows/sprint-loops-ci.yml`, `tests/cli.rs`
- **Depends on:** T-003, T-004
- **Acceptance criterion:** INT-0006 "Tested in CI on both OSes". INT-0005's guard and INT-0001's two-OS legs must not regress.
- **Success criterion (EARS):**
  - **WHEN** CI runs, **THEN** its gates **SHALL** be `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --all -- --nocapture`, each a one-line step after the toolchain log. No other step, and no key, **SHALL** be added or changed.
  - **WHEN** `test_ci_updates_installs_and_logs_in_order` reads the workflow, **THEN** it **SHALL** require those three gate lines. Every mutation in its sprint 2 table **SHALL** still fail it. Restoring either pre-workspace gate line **SHALL** fail it too.
  - **WHEN** `dev` is pushed after this task, **THEN** both legs **SHALL** compile `mdeezl-gui` and run its tests, both **SHALL** pass, and neither **SHALL** install a system package.
- **Notes:** from the sources, the critic found that every Linux windowing, GL and D-Bus library these features reach is loaded at run time. If the Linux leg fails to build anyway, this task is not done: any fix that adds a step is a plan amendment, recorded in the test report.

### T-006: Weasel hero and window icon
- **Intent:** [INT-0006](../../../intents/INT-0006-desktop-gui.md)
- **Touches:** `gui/src/weasel.rs` (new), `gui/src/lib.rs`, `gui/tests/app.rs`
- **Depends on:** T-003
- **Acceptance criterion:** INT-0006 "One window, four controls" (the logo) and "The weasel is judged by a person".
- **Success criterion (EARS):**
  - **WHEN** `weasel::shapes()` is called, **THEN** it **SHALL** return a non-empty list of coloured discs and tubes, lying within the unit box `[0,1] × [0,0.6]` once each is extended by its radius. A tube is a polyline whose points carry a radius. The list **SHALL** hold at least one tube.
  - **WHEN** the app is shown, **THEN** it **SHALL** paint a weasel hero above the heading into a rect recorded as `App::hero_rect()`.
    - Each disc **SHALL** be one filled circle.
    - Each tube vertex **SHALL** be one filled circle.
    - Each tube segment **SHALL** be one line segment.
    - No image file, and no image decoding, **SHALL** be used for the logo.
  - **WHEN** `native_options()` is built, **THEN** its viewport icon **SHALL** be `weasel::icon(64)`: a 64×64 RGBA raster of the same shapes, with transparent corners and opaque (alpha ≥ 128) coverage between 15 % and 60 % of pixels.
  - **WHEN** the Test Phase ends, **THEN** a screenshot of the running window **SHALL** have been shown to the user. The user's judgement decides whether it reads as a weasel.

### T-007: README desktop-app section and scope
- **Intent:** [INT-0006](../../../intents/INT-0006-desktop-gui.md)
- **Touches:** `README.md`, `tests/cli.rs`
- **Depends on:** T-004, T-006
- **Acceptance criterion:** INT-0006 "Documented".
- **Success criterion (EARS):**
  - **WHEN** `README.md` is read, **THEN** its `## Desktop app` section **SHALL** give:
    - how to run the app: `cargo run -p mdeezl-gui --release`;
    - how to install it: `cargo install --path gui`;
    - the controls: `Source`, `Browse…`, `fence`, `inline`, `none`, `Respect .gitignore`, `Output folder` and `Generate`;
    - that output is written as `<name>.md`, replacing a file of that name;
    - that a URL source needs git.
  - **WHEN** the README's introduction is read, **THEN** it **SHALL** scope "standard library only, no dependencies" to the `mdeezl` crate, and say that the desktop app is a separate crate with its own dependencies.
