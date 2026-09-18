//! End-to-end tests: build a fixture tree, run the real binary, assert on the
//! document it emits. The binary is located with `CARGO_BIN_EXE_mdeezl`, a
//! Cargo built-in for integration targets, so no test dependency is needed.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

const EXE: &str = env!("CARGO_BIN_EXE_mdeezl");

fn tmp_dir(tag: &str) -> PathBuf {
    static N: AtomicUsize = AtomicUsize::new(0);
    let n = N.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("mdeezl-e2e-{}-{tag}-{n}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

/// A fixture exercising every interesting case at once:
/// nested dirs, an ignored build dir, a dot-entry, a `.github` dir, a binary
/// file, a Markdown file containing a fence, and two plain text files.
fn fixture(tag: &str) -> PathBuf {
    let dir = tmp_dir(tag);
    fs::create_dir(dir.join("src")).unwrap();
    fs::write(dir.join("src/main.rs"), "fn main() {}\n").unwrap();
    fs::create_dir(dir.join("sub")).unwrap();
    fs::write(dir.join("sub/one.txt"), "nested one\n").unwrap();
    fs::write(dir.join("one.txt"), "root one\n").unwrap();
    fs::write(dir.join("a.txt"), "last line of a\n").unwrap();
    fs::write(
        dir.join("notes.md"),
        "before\n\n```\nfenced\n```\n\nafter\n",
    )
    .unwrap();
    fs::write(dir.join("blob.bin"), [0xffu8, 0xfe, 0x00, 0x01]).unwrap();
    fs::write(dir.join("debug.log"), "noisy\n").unwrap();
    fs::create_dir(dir.join("target")).unwrap();
    fs::write(dir.join("target/artifact.txt"), "built\n").unwrap();
    fs::write(dir.join(".secret"), "hidden\n").unwrap();
    fs::create_dir(dir.join(".github")).unwrap();
    fs::write(dir.join(".github/ci.yml"), "on: push\n").unwrap();
    dir
}

fn run(dir: &Path, args: &[&str]) -> Output {
    let mut cmd = Command::new(EXE);
    cmd.arg(dir);
    cmd.args(args);
    cmd.output().expect("failed to run mdeezl")
}

fn stdout_of(out: &Output) -> String {
    String::from_utf8(out.stdout.clone()).expect("stdout must be valid UTF-8")
}

fn ok_stdout(dir: &Path, args: &[&str]) -> String {
    let out = run(dir, args);
    assert!(
        out.status.success(),
        "expected success, got {:?}",
        out.status
    );
    stdout_of(&out)
}

#[test]
fn test_cli_writes_document_to_stdout() {
    let dir = fixture("stdout");
    let before = fs::read_dir(&dir).unwrap().count();
    let doc = ok_stdout(&dir, &[]);
    assert!(doc.starts_with("# "));
    assert!(doc.contains("## Structure"));
    // Nothing was written to any other file.
    let after = fs::read_dir(&dir).unwrap().count();
    assert_eq!(before, after);
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn test_cli_document_section_order() {
    let dir = fixture("order");
    let doc = ok_stdout(&dir, &[]);
    let title = doc.lines().position(|l| l.starts_with("# ")).unwrap();
    let structure = doc.lines().position(|l| l == "## Structure").unwrap();
    let contents = doc.lines().position(|l| l == "## Contents").unwrap();
    assert!(title < structure && structure < contents);
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn test_cli_tree_shape() {
    let dir = fixture("shape");
    let doc = ok_stdout(&dir, &[]);
    // Sorted, with the exact box-drawing symbols and one space before the name.
    assert!(doc.contains("├── a.txt"));
    assert!(doc.contains("├── src/"));
    assert!(doc.contains("│   └── main.rs"));
    assert!(doc.contains("└── sub/"));
    assert!(doc.contains("    └── one.txt"));
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn test_cli_file_header_format() {
    let dir = fixture("header");
    let doc = ok_stdout(&dir, &[]);
    assert!(doc.contains("\n---\nFile: src/main.rs\n---\n"));
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn test_cli_default_ignores_target_and_dotfiles() {
    let dir = fixture("ignores");
    let doc = ok_stdout(&dir, &[]);
    assert!(!doc.contains("target"), "target/ is ignored by default");
    assert!(!doc.contains("built"));
    assert!(
        !doc.contains(".secret"),
        "dot-entries are ignored by default"
    );
    assert!(!doc.contains("hidden"));
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn test_cli_include_readmits_dot_entry() {
    let dir = fixture("include");
    let doc = ok_stdout(&dir, &["--include", ".github"]);
    assert!(doc.contains(".github/"));
    assert!(doc.contains("File: .github/ci.yml"));
    assert!(doc.contains("on: push"));
    // Other dot-entries stay out.
    assert!(!doc.contains(".secret"));
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn test_cli_exclude_adds_pattern() {
    let dir = fixture("exclude");
    let doc = ok_stdout(&dir, &["--exclude", "*.log"]);
    assert!(!doc.contains("debug.log"));
    assert!(!doc.contains("noisy"));
    assert!(doc.contains("a.txt"), "siblings survive");
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn test_cli_exclude_single_file_by_path() {
    let dir = fixture("bypath");
    let doc = ok_stdout(&dir, &["--exclude", "sub/one.txt"]);
    assert!(!doc.contains("nested one"));
    // The same-named file at the root is untouched.
    assert!(doc.contains("File: one.txt"));
    assert!(doc.contains("root one"));
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn test_cli_binary_file_elided() {
    let dir = fixture("binary");
    let out = run(&dir, &[]);
    let doc = stdout_of(&out);
    assert!(doc.contains("[binary file, 4 bytes elided]"));
    assert!(
        !out.stdout.contains(&0xffu8),
        "raw bytes must not be emitted"
    );
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn test_cli_markdown_body_does_not_close_its_own_fence() {
    let dir = fixture("fences");
    let doc = ok_stdout(&dir, &[]);
    // The notes.md body holds a ``` run, so its fence must be longer.
    assert!(doc.contains("````markdown\n"));
    // Every fence opened is closed: an even count of fence-only lines.
    let fence_lines = doc
        .lines()
        .filter(|l| l.starts_with("```") && l.trim_end_matches('`').is_empty())
        .count();
    assert_eq!(fence_lines % 2, 0, "fences must balance");
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn test_cli_fence_mode_tree_in_one_block() {
    let dir = fixture("treefence");
    let doc = ok_stdout(&dir, &[]);
    let structure = doc.split("## Structure").nth(1).unwrap();
    let tree = structure.split("## Contents").next().unwrap();
    assert_eq!(tree.matches("```").count(), 2, "exactly one fenced block");
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn test_cli_rust_body_carries_language_hint() {
    let dir = fixture("hint");
    let doc = ok_stdout(&dir, &[]);
    assert!(doc.contains("```rust\nfn main() {}"));
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn test_cli_inline_mode_keeps_bodies_fenced() {
    let dir = fixture("inline");
    let doc = ok_stdout(&dir, &["--wrap", "inline"]);
    assert!(
        doc.contains("`└── sub/`"),
        "tree lines are backtick-wrapped"
    );
    assert!(doc.contains("```rust\n"), "bodies are still fenced");
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn test_cli_inline_mode_body_carries_language_hint() {
    let dir = fixture("inlinehint");
    let doc = ok_stdout(&dir, &["--wrap", "inline"]);
    assert!(doc.contains("```rust\nfn main() {}"));
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn test_cli_wrap_none_has_no_fences() {
    let dir = fixture("none");
    let doc = ok_stdout(&dir, &["--wrap", "none"]);
    let structure = doc.split("## Structure").nth(1).unwrap();
    let tree = structure.split("## Contents").next().unwrap();
    assert!(
        !tree.contains('`'),
        "no fences and no backticks in the tree"
    );
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn test_cli_wrap_none_preserves_section_separation() {
    let dir = fixture("sep");
    let doc = ok_stdout(&dir, &["--wrap", "none"]);
    // a.txt's body ends in a text line; without the blank line the next
    // section's `---` would underline it as a setext heading and swallow the
    // following header.
    assert!(doc.contains("last line of a\n\n---\nFile: "));
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn test_cli_output_flag_writes_file() {
    let dir = fixture("outflag");
    let out_path = dir.join("bundle.md.out");
    let out = run(&dir, &["-o", out_path.to_str().unwrap()]);
    assert!(out.status.success());
    assert!(out.stdout.is_empty(), "stdout stays empty when -o is given");
    let written = fs::read_to_string(&out_path).unwrap();
    assert!(written.contains("## Structure"));
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn test_cli_nested_paths_use_forward_slashes() {
    let dir = fixture("slashes");
    let doc = ok_stdout(&dir, &[]);
    for line in doc.lines().filter(|l| l.starts_with("File: ")) {
        assert!(!line.contains('\\'), "backslash in path: {line}");
    }
    assert!(doc.contains("File: sub/one.txt"));
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn test_cli_output_is_byte_identical_across_runs() {
    let dir = fixture("determinism");
    let first = run(&dir, &[]).stdout;
    let second = run(&dir, &[]).stdout;
    assert_eq!(first, second);
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn test_cli_usage_error_exits_2() {
    let dir = fixture("usage");
    let out = run(&dir, &["--wrap", "banana"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
    assert!(String::from_utf8_lossy(&out.stderr).contains("banana"));
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn test_cli_missing_root_exits_1_with_empty_stdout() {
    let missing = std::env::temp_dir().join("mdeezl-absent-xyzzy");
    let out = Command::new(EXE).arg(&missing).output().unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(out.stdout.is_empty(), "no partial document on stdout");
    assert!(!out.stderr.is_empty());
}

#[test]
fn test_cli_unwritable_output_exits_1() {
    let dir = fixture("unwritable");
    let bad = dir.join("no-such-dir").join("out.md");
    let out = run(&dir, &["-o", bad.to_str().unwrap()]);
    assert_eq!(out.status.code(), Some(1), "not a panic's 101");
    assert!(out.stdout.is_empty());
    assert!(!out.stderr.is_empty());
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn test_cli_help_documents_patterns_and_caveats() {
    let out = Command::new(EXE).arg("--help").output().unwrap();
    assert!(out.status.success());
    let help = String::from_utf8(out.stdout).unwrap();
    for form in [".*", "*.ext", "has/a/slash", "anything else"] {
        assert!(help.contains(form), "help must document the {form} form");
    }
    assert!(
        help.contains("Bash"),
        "help must warn about shell expansion"
    );
    assert!(
        help.contains("cannot be inline"),
        "help must state that bodies cannot be inline"
    );
}
