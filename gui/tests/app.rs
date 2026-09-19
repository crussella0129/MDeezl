//! The desktop app, driven headlessly through egui_kittest: the real
//! `eframe::App` runs frame by frame with no window and no GPU.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use eframe::egui::{
    self,
    accesskit::{Role, Toggled},
};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use mdeezl::{Options, Wrap};
use mdeezl_gui::{App, home_folder};

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
            let app = App {
                source: "some/folder".to_string(),
                wrap,
                use_gitignore,
                ..App::default()
            };
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
