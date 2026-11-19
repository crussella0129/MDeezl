//! MDeezl's desktop app (INT-0006): one small window over the CLI's own
//! engine. Name a folder or a URL, choose the format, choose where the `.md`
//! goes, and press Generate.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, mpsc};
use std::thread::JoinHandle;
use std::time::Duration;

use eframe::egui;
use mdeezl::{Bundle, Options, Wrap};

pub mod weasel;

/// The window's size: room for the controls and the status line, no more.
const WINDOW_SIZE: [f32; 2] = [520.0, 600.0];

/// How often a working app looks for its result, even with no input.
const POLL: Duration = Duration::from_millis(100);

const CLOSING: &str = "Closing when the current job finishes…";

/// The hero weasel's width in points; its height follows the unit box.
const HERO_WIDTH: f32 = 280.0;

/// How eframe should open the window, with the weasel as its icon.
pub fn native_options() -> eframe::NativeOptions {
    eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("MDeezl")
            .with_inner_size(WINDOW_SIZE)
            .with_min_inner_size([420.0, 480.0])
            .with_icon(Arc::new(weasel::icon(64))),
        ..Default::default()
    }
}

/// Where output goes until the user picks somewhere else.
pub fn home_folder() -> String {
    let var = if cfg!(windows) { "USERPROFILE" } else { "HOME" };
    std::env::var(var).unwrap_or_else(|_| ".".to_string())
}

/// `N bytes`, `N.N KB` or `N.N MB`.
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    match bytes {
        b if b < KB => format!("{b} bytes"),
        b if b < KB * KB => format!("{:.1} KB", b as f64 / KB as f64),
        b => format!("{:.1} MB", b as f64 / (KB * KB) as f64),
    }
}

/// The output file's stem: the bundle's name, unless that could leave the
/// output folder or is not a plain name -- a bundled filesystem root falls
/// back to its whole path, for one. Stated by characters, the same on every OS.
pub fn output_stem(name: &str) -> &str {
    if name.is_empty() || name == "." || name == ".." || name.contains(['/', '\\', ':']) {
        "bundle"
    } else {
        name
    }
}

/// The temporary sibling `<stem>.md` is written through. The process id is in
/// it because two copies of the app, generating the same source into the same
/// folder, otherwise share one temporary file and can delete each other's
/// work mid-write -- which driving two instances at once showed them doing.
pub fn temp_name(stem: &str) -> String {
    format!(".{stem}.md.{}.mdeezl-tmp", std::process::id())
}

/// What Generate runs: `mdeezl::bundle` in the app, a stand-in in tests.
pub type Job = Arc<dyn Fn(&Options) -> io::Result<Bundle> + Send + Sync>;

/// Write `<dir>/<stem>.md` through a temporary sibling and one rename, so a
/// failure can never damage an earlier good file or leave a partial one.
fn write_output(dir: &Path, bundle: &Bundle) -> Result<PathBuf, String> {
    let stem = output_stem(&bundle.name);
    let target = dir.join(format!("{stem}.md"));
    let temp = dir.join(temp_name(stem));
    let written = fs::write(&temp, &bundle.document).and_then(|()| fs::rename(&temp, &target));
    match written {
        Ok(()) => Ok(target),
        Err(e) => {
            let _ = fs::remove_file(&temp);
            Err(format!("could not write {}: {e}", target.display()))
        }
    }
}

/// A job in flight: its result arrives once, on `rx`.
struct Worker {
    rx: mpsc::Receiver<Result<String, String>>,
    handle: Option<JoinHandle<()>>,
    /// Chosen when Generate was pressed, so editing the field mid-job cannot
    /// change it.
    label: &'static str,
}

/// Dropping a worker waits for its thread. A closing window must not strand
/// a job, because a job's temporary clone is removed only when it ends. The
/// close deferral in [`App::tick`] normally makes this instant; this is the
/// backstop, and it is also how a finished job's thread is reaped.
impl Drop for Worker {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

/// The app's state: the four controls' values, and the job behind Generate.
pub struct App {
    /// A folder path or a URL, exactly as typed.
    pub source: String,
    pub wrap: Wrap,
    pub use_gitignore: bool,
    /// The folder `<name>.md` is written into.
    pub output: String,
    job: Job,
    worker: Option<Worker>,
    status: String,
    status_is_error: bool,
    /// The window asked to close while working; close once the job ends.
    close_pending: bool,
    /// Where the hero weasel was last painted.
    hero_rect: Option<egui::Rect>,
}

impl Default for App {
    fn default() -> Self {
        App {
            source: String::new(),
            wrap: Wrap::Fence,
            use_gitignore: true,
            output: home_folder(),
            job: Arc::new(mdeezl::bundle),
            worker: None,
            status: String::new(),
            status_is_error: false,
            close_pending: false,
            hero_rect: None,
        }
    }
}

impl App {
    /// Replace what Generate runs.
    pub fn with_job(mut self, job: Job) -> Self {
        self.job = job;
        self
    }

    /// The options the engine runs with: the CLI's defaults plus the fields.
    pub fn options(&self) -> Options {
        Options {
            // Trimmed: a path pasted from a file manager or a chat message
            // often carries a space, and the raw string failed with "The
            // filename, directory name, or volume label syntax is incorrect".
            root: PathBuf::from(self.source.trim()),
            wrap: self.wrap,
            use_gitignore: self.use_gitignore,
            ..Options::default()
        }
    }

    pub fn is_working(&self) -> bool {
        self.worker.is_some()
    }

    /// The status line's text.
    pub fn status(&self) -> &str {
        &self.status
    }

    pub fn status_is_error(&self) -> bool {
        self.status_is_error
    }

    /// Where the hero weasel was painted in the last frame.
    pub fn hero_rect(&self) -> Option<egui::Rect> {
        self.hero_rect
    }

    fn fail(&mut self, message: impl Into<String>) {
        self.status = message.into();
        self.status_is_error = true;
    }

    /// Start a job on a worker thread, and return at once, already working.
    /// The worker bundles, writes the file, and wakes the window.
    pub fn generate(&mut self, ctx: &egui::Context) {
        if self.is_working() {
            return;
        }
        if self.source.trim().is_empty() {
            return self.fail("Choose a folder or URL");
        }
        if self.output.trim().is_empty() {
            return self.fail("Choose an output folder");
        }
        let options = self.options();
        let dir = PathBuf::from(self.output.trim());
        // Check the destination before doing the work: a mistyped folder was
        // otherwise only reported after a long bundle, as a write failure.
        if !dir.is_dir() {
            return self.fail(format!("Not a folder: {}", dir.display()));
        }
        let job = Arc::clone(&self.job);
        let worker_ctx = ctx.clone();
        let (tx, rx) = mpsc::channel();
        let handle = std::thread::spawn(move || {
            let result = job(&options).map_err(|e| e.to_string()).and_then(|bundle| {
                let path = write_output(&dir, &bundle)?;
                let mut message = format!(
                    "Wrote {} ({})",
                    path.display(),
                    format_size(bundle.document.len() as u64)
                );
                for notice in &bundle.notices {
                    message.push('\n');
                    message.push_str(notice);
                }
                Ok(message)
            });
            let _ = tx.send(result);
            worker_ctx.request_repaint();
        });
        let label = if mdeezl::is_url(&self.source) {
            "Fetching…"
        } else {
            "Bundling…"
        };
        self.worker = Some(Worker {
            rx,
            handle: Some(handle),
            label,
        });
        self.status = label.to_string();
        self.status_is_error = false;
        // Ask for the frame that starts the polling. Without it, the window
        // would wait for input, or for the worker's own wake-up at the end:
        // a live run showed 27 s without a single frame when Generate was
        // triggered from outside the drawing pass.
        ctx.request_repaint();
    }

    /// Everything that must happen even while the window is minimised:
    /// eframe's `logic` calls this before every `ui`, and on its own when no
    /// `ui` pass runs. Collects the result, defers a close while a job runs,
    /// and keeps waking the window until the job ends.
    pub fn tick(&mut self, ctx: &egui::Context) {
        if self.is_working() && ctx.input(|i| i.viewport().close_requested()) {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.close_pending = true;
        }
        if let Some(worker) = &self.worker {
            let arrived = match worker.rx.try_recv() {
                Ok(result) => Some(result),
                Err(mpsc::TryRecvError::Disconnected) => {
                    Some(Err("The job stopped unexpectedly".to_string()))
                }
                Err(mpsc::TryRecvError::Empty) => None,
            };
            if let Some(result) = arrived {
                self.finish(result);
            }
        }
        match &self.worker {
            Some(worker) => {
                self.status = if self.close_pending {
                    CLOSING
                } else {
                    worker.label
                }
                .to_string();
                ctx.request_repaint_after(POLL);
            }
            None if self.close_pending => {
                self.close_pending = false;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            None => {}
        }
    }

    fn finish(&mut self, result: Result<String, String>) {
        self.worker = None;
        match result {
            Ok(message) => {
                self.status = message;
                self.status_is_error = false;
            }
            Err(message) => self.fail(message),
        }
    }

    /// Draw the window and handle its input.
    pub fn draw(&mut self, ui: &mut egui::Ui) {
        self.take_dropped_folder(ui.ctx());
        egui::CentralPanel::default().show(ui, |ui| {
            ui.vertical_centered(|ui| {
                let width = HERO_WIDTH.min(ui.available_width());
                let size = egui::vec2(width, width * weasel::HEIGHT);
                let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
                weasel::paint(ui.painter(), rect);
                self.hero_rect = Some(rect);
                ui.heading("MDeezl");
                ui.label("Feed any repository to text-only agents.");
            });
            ui.add_space(16.0);

            let label = ui.label("Source");
            ui.horizontal(|ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut self.source)
                        .hint_text("Folder path or https:// URL")
                        .desired_width(360.0),
                )
                .labelled_by(label.id);
                if ui.button("Browse…").clicked() {
                    pick_folder_into(&mut self.source);
                }
            });
            ui.add_space(12.0);

            ui.label("Format");
            ui.horizontal(|ui| {
                ui.radio_value(&mut self.wrap, Wrap::Fence, "fence");
                ui.radio_value(&mut self.wrap, Wrap::Inline, "inline");
                ui.radio_value(&mut self.wrap, Wrap::None, "none");
            });
            ui.checkbox(&mut self.use_gitignore, "Respect .gitignore");
            ui.add_space(12.0);

            let label = ui.label("Output folder");
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut self.output).desired_width(360.0))
                    .labelled_by(label.id);
                if ui.button("Browse…").clicked() {
                    pick_folder_into(&mut self.output);
                }
            });
            ui.add_space(16.0);

            let generate = ui.add_enabled(!self.is_working(), egui::Button::new("Generate"));
            if generate.clicked() {
                self.generate(ui.ctx());
            }
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if self.is_working() {
                    ui.spinner();
                }
                let text = egui::RichText::new(&self.status);
                if self.status_is_error {
                    ui.label(text.color(ui.visuals().error_fg_color));
                } else {
                    ui.label(text);
                }
            });
        });
    }

    /// A folder dropped anywhere on the window becomes the source.
    fn take_dropped_folder(&mut self, ctx: &egui::Context) {
        let dropped = ctx.input(|i| i.raw.dropped_files.clone());
        if let Some(dir) = dropped.iter().map(|f| f.path()).find(|p| p.is_dir()) {
            self.source = dir.display().to_string();
        }
    }
}

/// Open the native folder picker; a chosen folder replaces `field`, and a
/// cancelled picker leaves it alone.
fn pick_folder_into(field: &mut String) {
    if let Some(dir) = rfd::FileDialog::new().pick_folder() {
        *field = dir.display().to_string();
    }
}

impl eframe::App for App {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.tick(ctx);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.draw(ui);
    }
}
