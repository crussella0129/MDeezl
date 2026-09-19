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
- [INT-0002](../../../intents/INT-0002-git-and-remote-sources.md) — state: planned. The **remote half** is covered: its first, third, fourth, fifth and sixth criteria, for URL sources. The second criterion — reading a git ref without mutating the working tree — belongs to the git-ref half, which stays in backlog T-103. INT-0002 therefore cannot be `realized` at this sprint's close.
- [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md) — state: realized. Not advanced, but constrained:
  - local-path output must stay byte-identical;
  - the `mdeezl` crate keeps an empty `[dependencies]` table;
  - both CI legs remain.
- [INT-0005](../../../intents/INT-0005-reproducible-toolchain.md) — state: realized. Not advanced, but constrained: its guard test pins the exact CI gate lines. T-004 changes both the lines and the test, and re-runs the mutation check.

## Schema Tree
- Sprint Goal: a minimal egui window over the CLI's own engine, taking a path or URL
  - Engine
    - T-001: Engine library split
    - T-002: URL sources
  - Desktop app
    - T-003: GUI crate, controls and worker
    - T-005: Weasel hero and window icon
  - Delivery
    - T-004: CI gates cover the workspace
    - T-006: README desktop-app section

## Execution Sequence

### T-001: Engine library split
- **Intent:** [INT-0006](../../../intents/INT-0006-desktop-gui.md)
- **Touches:** `src/lib.rs` (new; the content of `src/main.rs` moves here), `src/main.rs`, `tests/cli.rs`
- **Depends on:** (none)
- **Acceptance criterion:** INT-0006 "The CLI's own output"; the engine becomes callable in-process, so GUI and CLI output match by construction. INT-0006 "The CLI stays dependency-free".
- **Success criterion (EARS):**
  - **WHEN** the package is built, **THEN** it **SHALL** have a library target `mdeezl` holding the engine and a binary target `mdeezl` whose `src/main.rs` does only one thing: it passes the process arguments after the program name to `mdeezl::cli` and exits with the code returned.
  - **WHEN** `mdeezl::bundle(&Options)` is called for a readable local directory, **THEN** it **SHALL**:
    - return `Bundle { name, document, notices }`;
    - make `document` equal to the bytes the CLI writes with `-o` for the same options;
    - make `name` the scanned directory's name;
    - put in `notices` the text of each notice the CLI prints, without the `mdeezl: ` prefix;
    - write nothing to stdout or stderr.
  - **WHEN** the CLI runs, **THEN** its stdout, its stderr (each notice printed as `mdeezl: <notice>`) and its exit codes 0, 1 and 2 **SHALL** be unchanged. All 122 existing tests **SHALL** pass. The `#[cfg(test)] mod tests` block **SHALL** move to `src/lib.rs` byte-identical, and `tests/cli.rs` **SHALL** receive insertions only.
  - **WHEN** `cargo tree -p mdeezl` runs, **THEN** it **SHALL** report only `mdeezl`, and the root `[dependencies]` table **SHALL** stay empty.
- **Notes:**
  - The public surface is `Options`, whose fields become `pub`, plus `Wrap`, `Sink`, `Bundle`, `bundle` and `cli`.
  - `run(&Options)` stays, as `bundle`, then the printed notices, then the sink write. The unit test that calls it keeps working.
  - Everything else stays private.

### T-002: URL sources
- **Intent:** [INT-0002](../../../intents/INT-0002-git-and-remote-sources.md)
- **Touches:** `src/lib.rs`, `tests/cli.rs`, `README.md`
- **Depends on:** T-001
- **Acceptance criterion:** INT-0002's remote half:
  - a URL source named on the command line, recorded in the document;
  - a remote addressed by URL;
  - failures reported on stderr with a non-zero exit, and no partial document;
  - never waiting on a person;
  - the temporary copy removed afterwards.
- **Success criterion (EARS):**
  - **WHEN** the source argument begins with `https://`, `http://` or `file://`, **THEN** mdeezl **SHALL** treat it as a URL. **WHEN** it begins with anything else, including `ftp:/weird`, `git@host:x/y` and `C:\x`, **THEN** mdeezl **SHALL** treat it as a local path, exactly as before.
  - **WHEN** a URL source is bundled, **THEN** mdeezl **SHALL** run `git -c credential.helper= clone --depth 1 --quiet -- <url> <dest>`:
    - with `GIT_TERMINAL_PROMPT=0` and `GCM_INTERACTIVE=never` set;
    - with the 15 repository-local git variables removed;
    - with stdin connected to null;
    - where `<dest>` is `<temp>/mdeezl-<pid>-<n>/<repo-name>` and `<temp>` is `std::env::temp_dir()`.
  - **WHEN** `<repo-name>` is derived from a URL, **THEN** it **SHALL** be the last non-empty `/`-separated segment with one trailing `.git` removed, or `repository` if none remains.
  - **WHEN** the clone succeeds, **THEN** the document **SHALL** be the clone's bundle, with heading `# <url>` and tree root line `<repo-name>/`, and `Bundle.name` **SHALL** be `<repo-name>`.
  - **WHEN** the fetch fails — a missing, unauthorised or unreachable repository, or git not found — **THEN** the CLI **SHALL** exit 1, with empty stdout and one stderr line `mdeezl: could not fetch <url>: <reason>`. `<reason>` **SHALL** be git's last non-empty stderr line without a leading `fatal: `, or `git was not found on PATH`.
  - **WHEN** a URL run ends, successful or not, **THEN** its `mdeezl-<pid>-<n>` directory **SHALL** no longer exist.
  - **WHEN** `--help` or the README is read, **THEN** it **SHALL** say four things:
    - a URL may be given in place of PATH;
    - it needs git;
    - it is shallow-cloned into a temporary folder that is removed afterwards;
    - private repositories are not supported.
- **Notes:**
  - `--` guards against URL-as-option injection.
  - `create_dir` is retried with the next `<n>` if a stale directory exists.
  - The clone is a git work tree, so INT-0004's gitignore handling applies to it unchanged.

### T-003: GUI crate, controls and worker
- **Intent:** [INT-0006](../../../intents/INT-0006-desktop-gui.md)
- **Touches:** `Cargo.toml` (`[workspace] members`), `gui/Cargo.toml`, `gui/src/lib.rs`, `gui/src/main.rs`, `gui/tests/app.rs`
- **Depends on:** T-001, T-002
- **Acceptance criterion:** INT-0006:
  - "One window, four controls";
  - "The CLI's own output";
  - "Responsive while working";
  - "No partial files";
  - "The CLI stays dependency-free".
- **Success criterion (EARS):**
  - **WHEN** the workspace is built, **THEN** `gui/` **SHALL** be the member package `mdeezl-gui`. Its dependencies **SHALL** be:
    - `mdeezl`, by path;
    - `eframe` 0.36, with default features off and `glow`, `default_fonts`, `accesskit` and `x11` on;
    - `rfd` 0.17, with default features off and `xdg-portal` on.

    Its only dev-dependency **SHALL** be `egui_kittest`. The root `[dependencies]` table stays empty.
  - **WHEN** the app is shown, **THEN** it **SHALL** present:
    - the heading `MDeezl`;
    - a source text field, hinted "Folder path or https:// URL", with a `Browse…` button;
    - radio buttons `fence`, `inline` and `none`, with `fence` selected;
    - a `Respect .gitignore` checkbox, checked;
    - an output-folder text field defaulting to the home folder — `USERPROFILE` on Windows, `HOME` elsewhere, `.` if unset — with a `Browse…` button;
    - a `Generate` button.
  - **WHEN** the fields hold a source, a wrap choice and a checkbox state, **THEN** the options the app passes to `mdeezl::bundle` **SHALL** be `Options { root: source, wrap: choice, use_gitignore: checkbox, ..Options::default() }`.
  - **WHEN** `Generate` is activated with a non-empty source and output folder, **THEN** the app **SHALL**:
    1. mark itself working before returning;
    2. run `bundle` on a worker thread;
    3. on success, write `<output>/<name>.md` with bytes equal to `bundle(&options).document`;
    4. show `Wrote <path> (<size>)` and every notice.
  - **WHEN** the worker is running, **THEN** `Generate` **SHALL** be disabled, and the status **SHALL** read `Fetching…` for a URL source or `Bundling…` otherwise.
  - **WHEN** bundling fails, or writing fails, **THEN** the status **SHALL** show the error, and no `<name>.md` from that attempt **SHALL** exist in the output folder. A failed write removes its file.
  - **WHEN** `Generate` is activated with an empty source or an empty output folder, **THEN** the status **SHALL** name the missing field, and no worker **SHALL** start.
  - **WHEN** a folder is dropped onto the window, **THEN** the source field **SHALL** become that folder's path.
- **Notes:**
  - `Browse…` opens `rfd::FileDialog::pick_folder`. A native dialog cannot be driven headlessly, so it is checked by hand in the Test Phase.
  - A release build sets `windows_subsystem = "windows"`, so no console window opens beside the app.

### T-004: CI gates cover the workspace
- **Intent:** [INT-0006](../../../intents/INT-0006-desktop-gui.md)
- **Touches:** `.github/workflows/sprint-loops-ci.yml`, `tests/cli.rs`
- **Depends on:** T-003
- **Acceptance criterion:** INT-0006 "Tested in CI on both OSes"; INT-0005's guard, and INT-0001's two-OS legs, must not regress.
- **Success criterion (EARS):**
  - **WHEN** CI runs, **THEN** its gates **SHALL** be `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --all -- --nocapture`. Each **SHALL** be a one-line step after the toolchain log. No other step or key **SHALL** change.
  - **WHEN** `test_ci_updates_installs_and_logs_in_order` reads the workflow, **THEN** it **SHALL** require those three gate lines. Every mutation from its sprint 2 table **SHALL** still fail it. Restoring either pre-workspace gate line **SHALL** also fail it.
  - **WHEN** `dev` is pushed after this task, **THEN** both legs **SHALL** compile the GUI crate and run its tests, and both **SHALL** pass.
- **Notes:** if the Linux leg cannot link, a bash step that installs the missing package is the contingency. Its script would test `$RUNNER_OS` inside the script, never with a YAML `if:` key, which the guard forbids. It is used only if the first push shows the need, and it is recorded in the test report.

### T-005: Weasel hero and window icon
- **Intent:** [INT-0006](../../../intents/INT-0006-desktop-gui.md)
- **Touches:** `gui/src/weasel.rs` (new), `gui/src/lib.rs`, `gui/src/main.rs`, `gui/tests/app.rs`
- **Depends on:** T-003
- **Acceptance criterion:** INT-0006 "One window, four controls" (the logo half), and "The weasel is judged by a person".
- **Success criterion (EARS):**
  - **WHEN** `weasel::shapes()` is called, **THEN** it **SHALL** return one list of discs and radius-carrying polylines ("tubes"), each with a colour, lying within the unit box `[0,1] × [0,0.6]`.
  - **WHEN** the app is shown, **THEN** it **SHALL** paint a weasel hero above the heading from `weasel::shapes()`, using only egui circles and line segments. No image file, image decoder or new dependency **SHALL** be involved.
  - **WHEN** the app starts, **THEN** its window icon **SHALL** be `weasel::icon(64)`: a 64×64 RGBA raster of the same shapes, with transparent corners and opaque coverage between 15 % and 60 % of pixels.
  - **WHEN** the Test Phase ends, **THEN** a screenshot of the running window **SHALL** have been shown to the user, whose judgement decides whether it reads as a weasel.

### T-006: README desktop-app section
- **Intent:** [INT-0006](../../../intents/INT-0006-desktop-gui.md)
- **Touches:** `README.md`, `tests/cli.rs`
- **Depends on:** T-003, T-005
- **Acceptance criterion:** INT-0006 "Documented".
- **Success criterion (EARS):**
  - **WHEN** `README.md` is read, **THEN** a `## Desktop app` section **SHALL** give:
    - how to run the app: `cargo run -p mdeezl-gui --release`;
    - how to install it: `cargo install --path gui`;
    - the four controls;
    - that output is written as `<name>.md` in the chosen folder;
    - that a URL source needs git.
