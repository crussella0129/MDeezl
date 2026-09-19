//! MDeezl's desktop app (INT-0006): one small window over the CLI's own
//! engine. Name a folder or a URL, choose the format, choose where the `.md`
//! goes, and press Generate.

use std::path::PathBuf;

use eframe::egui;
use mdeezl::{Options, Wrap};

/// The window's size: room for the controls and the status line, no more.
const WINDOW_SIZE: [f32; 2] = [520.0, 600.0];

/// How eframe should open the window.
pub fn native_options() -> eframe::NativeOptions {
    eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("MDeezl")
            .with_inner_size(WINDOW_SIZE)
            .with_min_inner_size([420.0, 480.0]),
        ..Default::default()
    }
}

/// Where output goes until the user picks somewhere else.
pub fn home_folder() -> String {
    let var = if cfg!(windows) { "USERPROFILE" } else { "HOME" };
    std::env::var(var).unwrap_or_else(|_| ".".to_string())
}

/// The app's state: the four controls' values.
pub struct App {
    /// A folder path or a URL, exactly as typed.
    pub source: String,
    pub wrap: Wrap,
    pub use_gitignore: bool,
    /// The folder `<name>.md` is written into.
    pub output: String,
}

impl Default for App {
    fn default() -> Self {
        App {
            source: String::new(),
            wrap: Wrap::Fence,
            use_gitignore: true,
            output: home_folder(),
        }
    }
}

impl App {
    /// The options the engine runs with: the CLI's defaults plus the fields.
    pub fn options(&self) -> Options {
        Options {
            root: PathBuf::from(&self.source),
            wrap: self.wrap,
            use_gitignore: self.use_gitignore,
            ..Options::default()
        }
    }

    /// Draw the window and handle its input.
    pub fn draw(&mut self, ui: &mut egui::Ui) {
        self.take_dropped_folder(ui.ctx());
        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("MDeezl");
            ui.label("Feed any repository to text-only agents.");
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

            // Present from the start; T-004 gives it its worker.
            let _ = ui.button("Generate");
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
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.draw(ui);
    }
}
