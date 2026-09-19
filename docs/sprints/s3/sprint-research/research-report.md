# Sprint 3 Research Report

## Intents Reviewed
- [INT-0006](../../../intents/INT-0006-desktop-gui.md) — created. Relevance: this sprint implements it. The user asked at the sprint 3 boundary for a minimal egui frontend with a weasel logo. Current state: `proposed`.
- [INT-0002](../../../intents/INT-0002-git-and-remote-sources.md) — selected and revised. Relevance: the GUI's source field takes a path *or a URL*, so this sprint delivers INT-0002's remote half. The git-ref half stays in backlog T-103. Two criteria were added, and the measured mechanism was recorded. Current state: `proposed`.
- [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md) — reviewed, constrained. Relevance: its zero-dependency property must survive a GUI that cannot be dependency-free. The two-OS CI criterion also now covers a GUI build. Current state: `realized`.
- [INT-0005](../../../intents/INT-0005-reproducible-toolchain.md) — reviewed, constrained. Relevance: its test pins the exact CI gate lines. Extending the gates to a workspace changes those lines, so the change must be made deliberately. Current state: `realized`.

## 1. Sprint Goal

Give MDeezl a minimal desktop window, so the user can see what it does and
learn it by using it. The whole workflow:

1. name a folder or an `https://` repository URL;
2. choose the wrap mode, and whether to respect `.gitignore`;
3. choose an output folder;
4. press Generate and get `<name>.md`.

The window has a weasel as its logo. It drives the CLI's own engine, so the two
produce byte-identical output. The CLI and engine crate stays dependency-free.
URL sources come from the remote half of INT-0002, which the CLI gains at the
same time.

## 2. Existing Code Survey

| File | Relevance | Notes |
|------|-----------|-------|
| src/main.rs | high | 2,044 lines: engine, CLI and 68 unit tests in one binary crate. It has no library target, so a GUI cannot call the engine in-process until one exists. `run(&Options)` renders the whole document before opening the sink, so no partial output can appear. That property is exactly what the GUI's "no partial file" criterion needs. Notices such as "gitignore filtering skipped" go straight to `eprintln!`, where a GUI cannot see them. |
| src/main.rs `parse_args` | high | Already takes a path, `-o FILE`, `--wrap fence\|inline\|none` and `--no-gitignore`. Those are exactly the GUI's controls, so the GUI can build the same option set the CLI builds. |
| src/main.rs `render_document` | high | The document opens with `# {root.name}`. A clone of `https://…/Hello-World` into `<tmp>/<unique>/Hello-World` gets that heading for free. Recording the *URL* itself is a format decision for the plan. |
| Cargo.toml | high | It has a bare `[workspace]` table, added in sprint 0 to stop Cargo from picking up a stray parent manifest. The GUI crate can join as a workspace member. `test_manifest_dependencies_table_is_empty` reads only this file, so a member crate's dependencies do not trip it. |
| .github/workflows/sprint-loops-ci.yml | high | The gates are `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test --all -- --nocapture`. With a root package, `fmt` and `clippy` cover only the root package unless given `--all` or `--workspace`, while `test --all` already covers the whole workspace. |
| tests/cli.rs (sprint 2 block) | high | `test_ci_updates_installs_and_logs_in_order` requires each gate to be a step made of exactly that one line. Widening a gate to the workspace breaks it, so the test must change with the workflow, not around it. |
| README.md | medium | Documents the CLI only. The GUI needs a launch section. |
| docs/work/tasks.md | low | T-103 names both halves of INT-0002. This sprint consumes the remote half. |

## 3. External Sources

- [eframe on crates.io](https://crates.io/crates/eframe) — Current version **0.36.2**, which requires Rust ≥ 1.95; this project is on 1.98.1. **The API changed from what older examples show.** `run_simple_native` is gone. The entry point is `run_ui_native(title, options, |ui, frame| …)`, and `App::ui(&mut self, ui, frame)` replaces `update(ctx, …)`. That was confirmed against the installed source after a probe failed to compile. The default features pull in wgpu. `glow` is the lighter renderer.
- [egui_kittest on crates.io](https://crates.io/crates/egui_kittest) — egui's own test harness, 0.36.2. It drives a UI headlessly through AccessKit: find a widget by label, click, type, then run frames. A probe test passed in 0.01 s with no GPU and added 11 dev crates. The `snapshot` and `wgpu` features, which render images, were not needed.
- [rfd on crates.io](https://crates.io/crates/rfd) — native file and folder dialogs, 0.17.2. The default features are `xdg-portal` and `wayland`, so it needs no GTK. On top of eframe it added **2** crates, on both Windows and Linux.
- [epaint `PathShape`](https://docs.rs/epaint/0.36.2/epaint/struct.PathShape.html) — "Fill is only supported for convex polygons" (quoted from the 0.36.2 source). A weasel silhouette is not convex, so a code-drawn logo must be built from convex pieces: circles, ellipses, convex polygons, and thick stroked cubic Béziers, which egui strokes at any shape.
- [git-clone documentation](https://git-scm.com/docs/git-clone) — `--depth 1` clones a single commit. Together with `GIT_TERMINAL_PROMPT=0` and an empty `credential.helper`, the failure modes below were measured.

## 4. Risks, Unknowns, Dependencies

Measured on this host (Windows 11, rustc 1.98.1, git 2.54.0), with a scratch
probe. See Artifacts.

- **Dependency weight:**

  | eframe features | Windows host | Linux target |
  |-----------------|--------------|--------------|
  | `glow`, `default_fonts`, `accesskit`, `x11`, `wayland` | 130 crates | 256 crates |
  | same, with `wgpu` instead of `glow` | 173 crates | — |

  - `rfd` adds 2. `egui_kittest` adds 11 dev crates.
  - An SVG logo through `egui_extras` with `svg` adds 31.
  - A cold debug build took 34 s, a rebuild 2 s. The debug exe is 32 MB.
- **The glow renderer runs here.** The probe window stayed up until `timeout 5`
  killed it, exit 124. An earlier PowerShell check that reported "exited" was
  a false alarm from that check itself.
- **Risk: the Linux CI build may need system libraries.** With eframe's default
  features off, winit's `wayland-dlopen` default is off too, so wayland may
  link at build time and need `libwayland-dev`. X11 and GL are loaded at run
  time. **This cannot be measured on Windows.** The first `dev` push will show
  it. The fallbacks are an `apt-get` step, or dropping the `wayland` feature,
  since X11 also runs under XWayland.
- **Risk: glow needs OpenGL 2+ at run time.** A machine without GPU drivers,
  such as a bare VM, may fail to open a window where wgpu could fall back to a
  software adapter. That trades 43 crates for robustness. glow works on this
  host.
- **Remote fetch, with git 2.54:**
  - `git clone --depth 1 --quiet https://github.com/octocat/Hello-World` took
    0.9 s. It produced `.git` and `README`, and `.git` is already omitted by
    the built-in `.*`.
  - A missing or private repository, with prompts and helpers disabled, gave
    `fatal: could not read Username … terminal prompts disabled`, exit 128, in
    0.4 s, with **no directory left behind**.
  - An unresolvable host gave exit 128 with "Could not resolve host".
  - **Risk:** without those two settings, Git Credential Manager on Windows can
    open a login window and hang a GUI.
  - `ftp:/weird` was read by git as an **ssh** target. MDeezl must decide what
    counts as a URL: `https://` or `http://` only.
- **Unknown: how to record a URL source in the document.** INT-0002 wants the
  document to record its source, but forbids changing the output format. The
  existing `# <name>` heading can carry it; the plan must choose.
- **Unknown: whether the weasel will read as a weasel.** It is a visual
  judgement. The Test Phase must put a screenshot in front of the user rather
  than claim it.
- **Dependency: the CI gates must cover the GUI crate.** `fmt` and `clippy`
  need `--all` or `--workspace`, which changes the lines INT-0005's test pins.
  That must be a deliberate, reviewed edit of both files.
- **Dependency: engine notices must become data.** `eprintln!` is invisible
  in a GUI.

## 5. Recommended Approach

Primary:

1. **Split the engine into a library.** Move `src/main.rs` to `src/lib.rs`,
   tests included, and leave a thin `src/main.rs` that parses arguments, calls
   the library and prints. Expose one small API: build `Options`, run the
   bundle, and return the document or an error, plus any notices as strings.
   The CLI prints the notices to stderr exactly as before; the GUI shows them.
2. **Add remote sources to the engine**, so the CLI and GUI share them:
   - an argument starting `https://` or `http://` is shallow-cloned into
     `<temp>/<unique>/<repo-name>`;
   - the clone runs with `GIT_TERMINAL_PROMPT=0`, `GCM_INTERACTIVE=never` and
     `-c credential.helper=`;
   - it is bundled, then removed by a drop guard, success or not;
   - failures go to stderr with a non-zero exit, and never produce a partial
     document.
3. **Add a `gui/` workspace member, `mdeezl-gui`,** depending on
   `mdeezl = { path = ".." }`, `eframe` (glow, `default_fonts`, `accesskit`,
   `x11`, `wayland`, with default features off) and `rfd`. It is one
   `eframe::App` with:
   - a header: the weasel, drawn with egui shapes, and the title;
   - a source field with a Browse… folder picker, which also accepts a folder
     dropped onto the window;
   - a wrap radio group (fence, inline, none) and a "Respect .gitignore"
     checkbox;
   - an output-folder field with Browse…;
   - Generate, which runs on a worker thread and writes
     `<output>/<name>.md`;
   - a status line.
4. **Test it headlessly with egui_kittest.** Fill the fields, click Generate,
   wait for the worker, then assert that the file's bytes equal the CLI's for
   the same source. Also assert that the error path shows a message and writes
   no file.
5. **Widen the CI gates to the workspace** (`cargo fmt --all --check`,
   `cargo clippy --workspace --all-targets …`), updating INT-0005's test in
   the same change. Confirm the Linux build on the first push.
6. **Take a screenshot of the real window** for the user to judge the weasel.

Alternatives considered:

- **GUI spawns the CLI binary.** Not chosen: two executables to ship and find,
  and notices scraped from stderr.
- **wgpu renderer.** More robust on machines without OpenGL, at +43 crates.
  Not chosen for a minimal first build; it can be revisited with a one-word
  feature change.
- **SVG or PNG logo.** Not chosen: +31 crates for an SVG renderer, and a
  second asset to keep in sync.
- **Remote fetch over HTTP with a TLS crate.** Would break the engine's
  dependency-free property. git is already required for gitignore support.

Rationale: the library split and the in-process GUI make "byte-identical to
the CLI" true by construction. Keeping every GUI dependency in its own crate
leaves the CLI exactly as dependency-free as before.

## Artifacts
- [`probe-Cargo.toml`](probe-Cargo.toml) — the probe's manifest: eframe with glow and no default features, rfd, and egui_kittest.
- [`probe-main.rs`](probe-main.rs) — the minimal eframe 0.36 program that built and ran here.
- [`probe-kittest.rs`](probe-kittest.rs) — the headless kittest probe that passed in 0.01 s.
