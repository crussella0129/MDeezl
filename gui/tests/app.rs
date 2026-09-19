//! The desktop app, driven headlessly through egui_kittest: the real
//! `eframe::App` runs frame by frame with no window and no GPU.

use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::time::{Duration, Instant};

use eframe::egui::{
    self, ViewportCommand, ViewportEvent, ViewportId,
    accesskit::{Role, Toggled},
};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use mdeezl::{Bundle, Options, Wrap};
use mdeezl_gui::{App, Job, format_size, home_folder, native_options, output_stem, weasel};

/// A temp directory that removes itself, even when an assertion panics.
struct Tmp(PathBuf);

impl Drop for Tmp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

impl std::ops::Deref for Tmp {
    type Target = Path;
    fn deref(&self) -> &Path {
        &self.0
    }
}

fn tmp_dir(tag: &str) -> Tmp {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static N: AtomicUsize = AtomicUsize::new(0);
    let n = N.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("mdeezl-gui-{}-{tag}-{n}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    Tmp(dir)
}

/// The real app, through eframe's own `logic` then `ui`.
fn harness(app: App) -> Harness<'static, App> {
    let mut app = Some(app);
    let mut h = Harness::new_eframe(move |_cc| app.take().expect("built once"));
    // A few frames: layout, then AccessKit catching up with it.
    for _ in 0..3 {
        h.step();
    }
    h
}

fn toggled(h: &Harness<'_, App>, role: Role, label: &str) -> Option<Toggled> {
    h.get_by_role_and_label(role, label)
        .accesskit_node()
        .toggled()
}

/// T-003. Exactly the planned manifest: the engine, a window, a folder
/// picker, and the one headless test harness -- no image decoding for the logo.
#[test]
fn test_gui_manifest_dependencies() {
    let manifest =
        std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
            .unwrap()
            .replace("\r\n", "\n");
    let table = |name: &str| -> Vec<String> {
        manifest
            .split(&format!("\n[{name}]\n"))
            .nth(1)
            .unwrap_or_else(|| panic!("a [{name}] table"))
            .split("\n[")
            .next()
            .unwrap()
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .map(str::to_string)
            .collect()
    };
    let deps = table("dependencies");
    let names: Vec<&str> = deps
        .iter()
        .map(|l| l.split('=').next().unwrap().trim())
        .collect();
    assert_eq!(names, ["mdeezl", "eframe", "rfd"]);
    let line = |name: &str| deps.iter().find(|l| l.starts_with(name)).unwrap().clone();
    assert!(line("mdeezl").contains(r#"path = "..""#));
    let eframe = line("eframe");
    assert!(eframe.contains("default-features = false"), "{eframe}");
    assert!(
        eframe.contains(r#"features = ["glow", "default_fonts", "accesskit", "x11", "wayland"]"#),
        "{eframe}"
    );
    let rfd = line("rfd");
    assert!(rfd.contains("default-features = false"), "{rfd}");
    assert!(rfd.contains(r#"features = ["xdg-portal"]"#), "{rfd}");

    let dev = table("dev-dependencies");
    assert_eq!(dev.len(), 1, "{dev:?}");
    assert!(dev[0].starts_with("egui_kittest"), "{dev:?}");
    assert!(dev[0].contains(r#"features = ["eframe"]"#), "{dev:?}");

    for banned in ["image", "egui_extras", "resvg", "usvg"] {
        assert!(
            !deps.iter().chain(&dev).any(|l| l.starts_with(banned)),
            "{banned} must not be a dependency"
        );
    }
}

/// T-003. Exactly the workflow's widgets: two fields, three buttons, three
/// radios and one checkbox, beside the heading.
#[test]
fn test_app_has_exactly_the_controls() {
    let h = harness(App::default());
    assert_eq!(h.query_all_by_role(Role::TextInput).count(), 2);
    assert_eq!(h.query_all_by_role(Role::Button).count(), 3);
    assert_eq!(h.query_all_by_role(Role::RadioButton).count(), 3);
    assert_eq!(h.query_all_by_role(Role::CheckBox).count(), 1);
    for label in [
        "MDeezl",
        "Source",
        "Output folder",
        "fence",
        "inline",
        "none",
        "Respect .gitignore",
        "Generate",
    ] {
        h.get_by_label(label);
    }
    assert_eq!(h.get_all_by_label("Browse…").count(), 2);
    let source = h.get_by_role_and_label(Role::TextInput, "Source");
    assert_eq!(
        source.accesskit_node().placeholder(),
        Some("Folder path or https:// URL")
    );
}

/// T-003. Fence, gitignore respected, output in the home folder.
#[test]
fn test_app_defaults() {
    let h = harness(App::default());
    assert_eq!(toggled(&h, Role::RadioButton, "fence"), Some(Toggled::True));
    assert_eq!(
        toggled(&h, Role::RadioButton, "inline"),
        Some(Toggled::False)
    );
    assert_eq!(
        toggled(&h, Role::CheckBox, "Respect .gitignore"),
        Some(Toggled::True)
    );
    let var = if cfg!(windows) { "USERPROFILE" } else { "HOME" };
    let want = std::env::var(var).unwrap_or_else(|_| ".".to_string());
    assert_eq!(home_folder(), want);
    assert_eq!(h.state().output, want);
}

/// T-003. The engine runs with the CLI's defaults plus exactly the fields.
#[test]
fn test_options_follow_the_fields() {
    for wrap in [Wrap::Fence, Wrap::Inline, Wrap::None] {
        for use_gitignore in [true, false] {
            let mut app = App::default();
            app.source = "some/folder".to_string();
            app.wrap = wrap;
            app.use_gitignore = use_gitignore;
            assert_eq!(
                app.options(),
                Options {
                    root: PathBuf::from("some/folder"),
                    wrap,
                    use_gitignore,
                    ..Options::default()
                }
            );
        }
    }
}

#[derive(Debug)]
struct Dropped(PathBuf);

impl egui::DroppedFile for Dropped {
    fn path(&self) -> &Path {
        &self.0
    }
    fn bytes(&self) -> Result<Vec<u8>, String> {
        Err("a folder has no bytes".to_string())
    }
}

/// T-003. A folder dropped on the window becomes the source.
#[test]
fn test_dropped_folder_fills_source() {
    let dir = tmp_dir("drop");
    let mut h = harness(App::default());
    h.input_mut()
        .dropped_files
        .push(Arc::new(Dropped(dir.to_path_buf())));
    h.step();
    assert_eq!(h.state().source, dir.display().to_string());
}

// ======================= T-004: Generate =======================

/// Hands a gated job its release, and reports whether and how often it ran.
struct Gate {
    release: mpsc::Sender<()>,
    done: Arc<AtomicBool>,
    calls: Arc<AtomicUsize>,
}

impl Gate {
    fn release(&self) {
        let _ = self.release.send(());
    }
}

/// A job that waits for the test to release it, then returns `result()`.
fn gated_job(result: impl Fn() -> io::Result<Bundle> + Send + Sync + 'static) -> (Job, Gate) {
    let (tx, rx) = mpsc::channel::<()>();
    let rx = Mutex::new(rx);
    let done = Arc::new(AtomicBool::new(false));
    let calls = Arc::new(AtomicUsize::new(0));
    let (job_done, job_calls) = (Arc::clone(&done), Arc::clone(&calls));
    let job: Job = Arc::new(move |_: &Options| {
        job_calls.fetch_add(1, Ordering::SeqCst);
        let _ = rx.lock().unwrap().recv();
        let r = result();
        job_done.store(true, Ordering::SeqCst);
        r
    });
    (
        job,
        Gate {
            release: tx,
            done,
            calls,
        },
    )
}

fn gated_bundle() -> io::Result<Bundle> {
    Ok(Bundle {
        name: "gated".to_string(),
        document: "gated document\n".to_string(),
        notices: Vec::new(),
    })
}

/// An app with its source, output and job set.
fn app_with(source: &str, output: &Path, job: Option<Job>) -> App {
    let mut app = App::default();
    if let Some(job) = job {
        app = app.with_job(job);
    }
    app.source = source.to_string();
    app.output = output.display().to_string();
    app
}

/// Step until the job ends, never `run()`: `run()` panics while the app keeps
/// requesting repaints, which a working app does on purpose.
fn wait_idle(h: &mut Harness<'_, App>) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while h.state().is_working() {
        assert!(Instant::now() < deadline, "the job did not end within 10 s");
        std::thread::sleep(Duration::from_millis(10));
        h.step();
    }
    h.step();
}

fn root_commands(h: &Harness<'_, App>) -> Vec<ViewportCommand> {
    h.output()
        .viewport_output
        .get(&ViewportId::ROOT)
        .map(|v| v.commands.clone())
        .unwrap_or_default()
}

fn push_close(h: &mut Harness<'_, App>) {
    h.input_mut()
        .viewports
        .entry(ViewportId::ROOT)
        .or_default()
        .events
        .push(ViewportEvent::Close);
}

/// A plain folder, not a git work tree, with two files.
fn plain_fixture(tag: &str) -> Tmp {
    let dir = tmp_dir(tag);
    std::fs::create_dir(dir.join("src")).unwrap();
    std::fs::write(dir.join("src/a.rs"), "fn a() {}\n").unwrap();
    std::fs::write(dir.join("notes.txt"), "some notes\n").unwrap();
    dir
}

fn entries(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

fn no_temp_files(dir: &Path) -> bool {
    !entries(dir).iter().any(|n| n.ends_with(".mdeezl-tmp"))
}

/// T-004.
#[test]
fn test_format_size() {
    for (bytes, text) in [
        (0, "0 bytes"),
        (1023, "1023 bytes"),
        (1024, "1.0 KB"),
        (1536, "1.5 KB"),
        (1_048_576, "1.0 MB"),
    ] {
        assert_eq!(format_size(bytes), text);
    }
}

/// T-004. The file always lands inside the output folder, on every OS.
#[test]
fn test_output_stem_is_one_component() {
    assert_eq!(output_stem("Hello-World"), "Hello-World");
    assert_eq!(output_stem("repo.v2"), "repo.v2");
    for bad in ["", ".", "..", "a/b", r"a\b", r"C:\", "c:x", "/"] {
        assert_eq!(output_stem(bad), "bundle", "{bad:?}");
    }
}

/// T-004. Generate returns at once, already working; the result then
/// arrives from the worker.
#[test]
fn test_generate_marks_working_before_returning() {
    let out = tmp_dir("marks-out");
    let (job, gate) = gated_job(gated_bundle);
    let mut h = harness(app_with("some/folder", &out, Some(job)));
    let ctx = h.ctx.clone();
    h.state_mut().generate(&ctx);
    assert!(h.state().is_working());
    gate.release();
    wait_idle(&mut h);
    assert!(
        h.state().status().starts_with("Wrote "),
        "{}",
        h.state().status()
    );
    assert!(gate.done.load(Ordering::SeqCst));
}

/// T-004. While working, Generate is disabled and the window keeps waking.
#[test]
fn test_generate_disabled_while_working() {
    let out = tmp_dir("disabled-out");
    let (job, gate) = gated_job(gated_bundle);
    let mut h = harness(app_with("some/folder", &out, Some(job)));
    h.get_by_label("Generate").click();
    h.step();
    h.step();
    assert!(h.state().is_working());
    assert!(h.get_by_label("Generate").accesskit_node().is_disabled());
    let delay = h.output().viewport_output[&ViewportId::ROOT].repaint_delay;
    assert!(delay <= Duration::from_millis(100), "{delay:?}");
    gate.release();
    wait_idle(&mut h);
    assert!(!h.get_by_label("Generate").accesskit_node().is_disabled());
}

/// T-004. The label follows the source captured at Generate, not the field.
#[test]
fn test_status_names_fetching_or_bundling() {
    let out = tmp_dir("label-out");
    for (source, label, edit) in [
        ("some/folder", "Bundling…", "file:///elsewhere"),
        ("file:///some/repo", "Fetching…", "some/folder"),
    ] {
        let (job, gate) = gated_job(gated_bundle);
        let mut h = harness(app_with(source, &out, Some(job)));
        let ctx = h.ctx.clone();
        h.state_mut().generate(&ctx);
        h.step();
        assert_eq!(h.state().status(), label);
        h.state_mut().source = edit.to_string();
        h.step();
        assert_eq!(h.state().status(), label, "editing the field mid-job");
        gate.release();
        wait_idle(&mut h);
    }
}

/// T-004. The engine's own bytes, typed in and clicked through the window.
#[test]
fn test_generate_writes_engine_bytes() {
    let src = plain_fixture("engine-src");
    let out = tmp_dir("engine-out");
    let mut h = harness(App::default());
    h.state_mut().output = out.display().to_string();
    let path = src.display().to_string();
    h.get_by_role_and_label(Role::TextInput, "Source").focus();
    h.step();
    h.get_by_role_and_label(Role::TextInput, "Source")
        .type_text(&path);
    h.step();
    assert_eq!(h.state().source, path);
    h.get_by_label("inline").click();
    h.step();
    h.get_by_label("Generate").click();
    h.step();
    wait_idle(&mut h);

    let want = mdeezl::bundle(&Options {
        root: PathBuf::from(&path),
        wrap: Wrap::Inline,
        ..Options::default()
    })
    .unwrap();
    let name = src.file_name().unwrap().to_str().unwrap();
    let written = std::fs::read(out.join(format!("{name}.md"))).unwrap();
    assert!(
        written == want.document.as_bytes(),
        "the engine's own bytes"
    );
    let status = h.state().status();
    assert!(status.starts_with("Wrote "), "{status}");
    assert!(
        status.contains(&format_size(want.document.len() as u64)),
        "{status}"
    );
    assert!(no_temp_files(&out));
}

/// T-004. Notices reach the status line.
#[test]
fn test_generate_shows_notices() {
    let src = plain_fixture("notice-src");
    let out = tmp_dir("notice-out");
    let mut h = harness(app_with(&src.display().to_string(), &out, None));
    let ctx = h.ctx.clone();
    h.state_mut().generate(&ctx);
    wait_idle(&mut h);
    let status = h.state().status();
    assert!(status.starts_with("Wrote "), "{status}");
    assert!(status.contains("gitignore filtering skipped:"), "{status}");
}

/// T-004. A failure leaves an earlier file exactly as it was.
#[test]
fn test_failed_job_keeps_existing_output() {
    // A failed job.
    let out = tmp_dir("keep-out");
    std::fs::write(out.join("gated.md"), "earlier good file\n").unwrap();
    let job: Job = Arc::new(|_: &Options| Err(io::Error::other("boom")));
    let mut h = harness(app_with("some/folder", &out, Some(job)));
    let ctx = h.ctx.clone();
    h.state_mut().generate(&ctx);
    wait_idle(&mut h);
    assert_eq!(h.state().status(), "boom");
    assert!(h.state().status_is_error());
    assert_eq!(
        std::fs::read_to_string(out.join("gated.md")).unwrap(),
        "earlier good file\n"
    );
    assert!(no_temp_files(&out));

    // A failed rename: the target is a directory.
    let out = tmp_dir("rename-out");
    std::fs::create_dir(out.join("gated.md")).unwrap();
    let job: Job = Arc::new(|_: &Options| gated_bundle());
    let mut h = harness(app_with("some/folder", &out, Some(job)));
    let ctx = h.ctx.clone();
    h.state_mut().generate(&ctx);
    wait_idle(&mut h);
    assert!(h.state().status_is_error());
    assert!(
        h.state().status().starts_with("could not write "),
        "{}",
        h.state().status()
    );
    assert!(out.join("gated.md").is_dir());
    assert!(no_temp_files(&out));
}

/// T-004. A missing source folder: an error, and nothing written.
#[test]
fn test_generate_missing_folder_writes_nothing() {
    let out = tmp_dir("missing-out");
    let missing = out.join("no-such-folder");
    let out_files = tmp_dir("missing-files");
    let mut h = harness(app_with(&missing.display().to_string(), &out_files, None));
    let ctx = h.ctx.clone();
    h.state_mut().generate(&ctx);
    wait_idle(&mut h);
    assert!(h.state().status_is_error(), "{}", h.state().status());
    assert!(entries(&out_files).is_empty());
}

/// T-004. An unreachable URL: an error without its credential, nothing
/// written. Localhost only.
#[test]
fn test_generate_unreachable_url_writes_nothing() {
    let out = tmp_dir("url-out");
    let mut h = harness(app_with(
        "https://user:SECRET@127.0.0.1:1/x.git",
        &out,
        None,
    ));
    let ctx = h.ctx.clone();
    h.state_mut().generate(&ctx);
    wait_idle(&mut h);
    let status = h.state().status();
    assert!(
        status.starts_with("could not fetch https://127.0.0.1:1/x.git"),
        "{status}"
    );
    assert!(!status.contains("SECRET"), "{status}");
    assert!(entries(&out).is_empty());
}

/// T-004. An output "folder" that is a file: an error, the file untouched.
#[test]
fn test_generate_unwritable_output_writes_nothing() {
    let src = plain_fixture("unwritable-src");
    let holder = tmp_dir("unwritable-holder");
    let not_a_folder = holder.join("a-file");
    std::fs::write(&not_a_folder, "keep me\n").unwrap();
    let mut h = harness(app_with(&src.display().to_string(), &not_a_folder, None));
    let ctx = h.ctx.clone();
    h.state_mut().generate(&ctx);
    wait_idle(&mut h);
    assert!(h.state().status_is_error(), "{}", h.state().status());
    assert_eq!(std::fs::read_to_string(&not_a_folder).unwrap(), "keep me\n");
    assert_eq!(entries(&holder), ["a-file"]);
}

/// T-004. An empty field is named, and no job starts.
#[test]
fn test_generate_requires_source_and_output() {
    let (job, gate) = gated_job(gated_bundle);
    let mut h = harness(App::default().with_job(job));
    let ctx = h.ctx.clone();
    h.state_mut().source = "  ".to_string();
    h.state_mut().generate(&ctx);
    assert_eq!(h.state().status(), "Choose a folder or URL");
    assert!(!h.state().is_working());
    h.state_mut().source = "some/folder".to_string();
    h.state_mut().output = String::new();
    h.state_mut().generate(&ctx);
    assert_eq!(h.state().status(), "Choose an output folder");
    assert!(!h.state().is_working());
    assert_eq!(gate.calls.load(Ordering::SeqCst), 0);
}

/// T-004. A job that panics ends with an error, and writes nothing.
#[test]
fn test_worker_panic_ends_job_with_error() {
    let out = tmp_dir("panic-out");
    let job: Job = Arc::new(|_: &Options| panic!("a deliberate test panic"));
    let mut h = harness(app_with("some/folder", &out, Some(job)));
    let ctx = h.ctx.clone();
    h.state_mut().generate(&ctx);
    wait_idle(&mut h);
    assert_eq!(h.state().status(), "The job stopped unexpectedly");
    assert!(h.state().status_is_error());
    assert!(entries(&out).is_empty());
}

/// T-004. Dropping the app waits for its worker, so a job's temporary clone
/// is always cleaned up before the process can exit.
#[test]
fn test_dropping_app_waits_for_worker() {
    let out = tmp_dir("drop-out");
    let (job, gate) = gated_job(gated_bundle);
    let mut h = harness(app_with("some/folder", &out, Some(job)));
    let ctx = h.ctx.clone();
    h.state_mut().generate(&ctx);
    let done = Arc::clone(&gate.done);
    let releaser = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(200));
        gate.release();
    });
    drop(h);
    assert!(
        done.load(Ordering::SeqCst),
        "the drop returned before the job ended"
    );
    releaser.join().unwrap();
}

/// T-004. Closing while working is deferred, then the app closes itself, and
/// does not cancel the close its own `Close` produces.
#[test]
fn test_close_while_working_defers_then_closes() {
    let out = tmp_dir("close-out");
    let (job, gate) = gated_job(gated_bundle);
    let mut h = harness(app_with("some/folder", &out, Some(job)));
    let ctx = h.ctx.clone();
    h.state_mut().generate(&ctx);
    push_close(&mut h);
    h.step();
    assert!(root_commands(&h).contains(&ViewportCommand::CancelClose));
    assert_eq!(h.state().status(), "Closing when the current job finishes…");
    h.step();
    h.step();
    assert_eq!(h.state().status(), "Closing when the current job finishes…");

    gate.release();
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        h.step();
        if root_commands(&h).contains(&ViewportCommand::Close) {
            break;
        }
        assert!(Instant::now() < deadline, "no Close within 10 s");
        std::thread::sleep(Duration::from_millis(10));
    }
    // eframe echoes the app's own Close as a close request: honour it.
    push_close(&mut h);
    h.step();
    assert!(!root_commands(&h).contains(&ViewportCommand::CancelClose));

    // Idle: a close simply proceeds.
    let mut idle = harness(App::default());
    push_close(&mut idle);
    idle.step();
    assert!(!root_commands(&idle).contains(&ViewportCommand::CancelClose));
}

/// A context whose repaint requests are recorded, delay by delay.
fn recording_context() -> (egui::Context, Arc<Mutex<Vec<Duration>>>) {
    let ctx = egui::Context::default();
    let delays = Arc::new(Mutex::new(Vec::new()));
    let log = Arc::clone(&delays);
    ctx.set_request_repaint_callback(move |info| log.lock().unwrap().push(info.delay));
    (ctx, delays)
}

fn root_logic_commands(out: &egui::LogicOutput) -> Vec<ViewportCommand> {
    out.viewport_commands
        .get(&ViewportId::ROOT)
        .cloned()
        .unwrap_or_default()
}

/// T-004. The deferral, the repaint and the result collection all live in
/// `tick`, which eframe runs through `logic` even when no `ui` pass does, as
/// for a minimised window. egui calls the repaint callback only for a request
/// sooner than one already pending, so each request is observed where
/// nothing sooner is pending.
#[test]
fn test_close_deferral_runs_in_logic_alone() {
    let out = tmp_dir("logic-out");
    let (job, gate) = gated_job(gated_bundle);
    let mut app = app_with("some/folder", &out, Some(job));
    let (ctx, delays) = recording_context();
    app.generate(&ctx);

    // A logic pass with no input events: the only repaint request is the
    // tick's own, and it is within 100 ms.
    let quiet = ctx.run_logic(&egui::RawInput::default(), |ctx| app.tick(ctx));
    assert!(!root_logic_commands(&quiet).contains(&ViewportCommand::CancelClose));
    let logged = delays.lock().unwrap().clone();
    assert!(
        !logged.is_empty() && logged.iter().all(|d| *d <= Duration::from_millis(100)),
        "the tick keeps the window waking: {logged:?}"
    );

    // A close request, in logic alone, is cancelled while the job runs.
    let mut input = egui::RawInput::default();
    input
        .viewports
        .entry(ViewportId::ROOT)
        .or_default()
        .events
        .push(ViewportEvent::Close);
    let cancelled = ctx.run_logic(&input, |ctx| app.tick(ctx));
    assert!(
        root_logic_commands(&cancelled).contains(&ViewportCommand::CancelClose),
        "{:?}",
        cancelled.viewport_commands
    );

    // Released, the result is collected in logic alone and the close follows.
    gate.release();
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let later = ctx.run_logic(&egui::RawInput::default(), |ctx| app.tick(ctx));
        if root_logic_commands(&later).contains(&ViewportCommand::Close) {
            break;
        }
        assert!(Instant::now() < deadline, "no Close within 10 s");
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(!app.is_working());
}

/// T-004. The worker itself wakes the window when its result is ready, with
/// no tick run: what lets a hidden window collect a result promptly.
#[test]
fn test_worker_wakes_the_window() {
    let out = tmp_dir("wake-out");
    let (job, gate) = gated_job(gated_bundle);
    let mut app = app_with("some/folder", &out, Some(job));
    let (ctx, delays) = recording_context();
    app.generate(&ctx);
    assert!(delays.lock().unwrap().is_empty(), "nothing requested yet");
    gate.release();
    let deadline = Instant::now() + Duration::from_secs(10);
    while !delays.lock().unwrap().contains(&Duration::ZERO) {
        assert!(
            Instant::now() < deadline,
            "the worker never woke the window"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}

// ======================= T-006: the weasel =======================

/// T-006. The weasel lies in its unit box, and is more than a blob.
#[test]
fn test_weasel_shapes_fit_the_unit_box() {
    let shapes = weasel::shapes();
    assert!(!shapes.is_empty());
    let mut tubes = 0;
    let inside = |x: f32, y: f32, r: f32| {
        x - r >= 0.0 && x + r <= 1.0 && y - r >= 0.0 && y + r <= weasel::HEIGHT
    };
    for shape in &shapes {
        match shape {
            weasel::Shape::Disc { center, radius, .. } => {
                assert!(inside(center[0], center[1], *radius), "{shape:?}");
            }
            weasel::Shape::Tube { points, .. } => {
                tubes += 1;
                assert!(points.len() >= 2, "{shape:?}");
                for p in points {
                    assert!(inside(p[0], p[1], p[2]), "{p:?} in {shape:?}");
                }
            }
        }
    }
    assert!(tubes >= 1);
}

/// T-006. The icon is the same weasel, rasterised: transparent corners and a
/// plausible share of opaque pixels.
#[test]
fn test_icon_rasterises_the_weasel() {
    let icon = weasel::icon(64);
    assert_eq!((icon.width, icon.height), (64, 64));
    assert_eq!(icon.rgba.len(), 64 * 64 * 4);
    let alpha = |x: usize, y: usize| icon.rgba[(y * 64 + x) * 4 + 3];
    for (x, y) in [(0, 0), (63, 0), (0, 63), (63, 63)] {
        assert_eq!(alpha(x, y), 0, "corner ({x}, {y})");
    }
    let opaque = icon.rgba.chunks(4).filter(|p| p[3] >= 128).count();
    let share = opaque as f64 / (64.0 * 64.0);
    assert!((0.15..=0.60).contains(&share), "opaque share {share:.3}");
}

/// T-006. The window opens with the weasel as its icon.
#[test]
fn test_native_options_carry_the_icon() {
    let options = native_options();
    let icon = options.viewport.icon.expect("an icon");
    assert_eq!((icon.width, icon.height), (64, 64));
    assert_eq!(icon.rgba, weasel::icon(64).rgba);
}

/// T-006. The hero is really painted: every disc and tube point as a filled
/// circle and every tube segment as a line, all inside the hero's rect --
/// which the radio buttons' circles are not.
#[test]
fn test_hero_paints_weasel_shapes() {
    let h = harness(App::default());
    let hero = h
        .state()
        .hero_rect()
        .expect("the hero was painted")
        .expand(1.0);
    let (mut want_circles, mut want_segments) = (0, 0);
    for shape in weasel::shapes() {
        match shape {
            weasel::Shape::Disc { .. } => want_circles += 1,
            weasel::Shape::Tube { points, .. } => {
                want_circles += points.len();
                want_segments += points.len() - 1;
            }
        }
    }
    let (mut circles, mut segments) = (0, 0);
    for clipped in &h.output().shapes {
        match &clipped.shape {
            egui::Shape::Circle(c) if hero.contains(c.center) => circles += 1,
            egui::Shape::LineSegment { points, .. }
                if hero.contains(points[0]) && hero.contains(points[1]) =>
            {
                segments += 1;
            }
            _ => {}
        }
    }
    assert!(circles >= want_circles, "{circles} < {want_circles}");
    assert!(segments >= want_segments, "{segments} < {want_segments}");
}
