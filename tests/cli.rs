//! End-to-end tests: build a fixture tree, run the real binary, assert on the
//! document it emits. The binary is located with `CARGO_BIN_EXE_mdeezl`, a
//! Cargo built-in for integration targets, so no test dependency is needed.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

const EXE: &str = env!("CARGO_BIN_EXE_mdeezl");

/// A temp directory that removes itself even when an assertion panics, so a
/// failing test cannot leave a tree behind in the system temp directory.
struct Tmp(PathBuf);

impl Drop for Tmp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

impl std::ops::Deref for Tmp {
    type Target = Path;
    fn deref(&self) -> &Path {
        &self.0
    }
}

impl AsRef<Path> for Tmp {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

fn tmp_dir(tag: &str) -> Tmp {
    static N: AtomicUsize = AtomicUsize::new(0);
    let n = N.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("mdeezl-e2e-{}-{tag}-{n}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    Tmp(dir)
}

/// Walk the document as a fence state machine: a fence opens on a line of >= 3
/// backticks (an info string may follow) and closes only on a line of nothing
/// but backticks, at least as long as the opener. Counting "fence-only lines"
/// and checking parity is not equivalent — it misses every opening fence that
/// carries a language hint while still counting its closer.
fn fences_balanced(doc: &str) -> bool {
    let mut open: Option<usize> = None;
    for line in doc.lines() {
        let ticks = line.chars().take_while(|c| *c == '`').count();
        match open {
            None => {
                if ticks >= 3 {
                    open = Some(ticks);
                }
            }
            Some(need) => {
                if ticks >= need && line.trim_end_matches('`').is_empty() {
                    open = None;
                }
            }
        }
    }
    open.is_none()
}

/// The opening fence length of `path`'s section, and the longest backtick run
/// inside that section's body. Balance alone cannot catch the regression this
/// project actually fears — a fence that is too short closes early, re-opens on
/// the body's next fence line, and still balances. The invariant that matters
/// is opener > longest run inside.
fn body_fence_vs_content(doc: &str, path: &str) -> (usize, usize) {
    let marker = format!("---\nFile: {path}\n---\n\n");
    let section = doc.split(&marker).nth(1).expect("section not found");
    let section = section.split("\n---\nFile: ").next().unwrap();

    let mut lines = section.lines();
    let opener = lines.next().unwrap();
    let open_len = opener.chars().take_while(|c| *c == '`').count();

    // Everything up to the matching closing fence is body.
    let mut longest = 0;
    for line in lines {
        if line.chars().take_while(|c| *c == '`').count() >= open_len
            && line.trim_end_matches('`').is_empty()
        {
            break;
        }
        let mut run = 0;
        for ch in line.chars() {
            if ch == '`' {
                run += 1;
                longest = longest.max(run);
            } else {
                run = 0;
            }
        }
    }
    (open_len, longest)
}

/// The `## Structure` block's lines, with the fence lines and the root line
/// removed, so a fixture's randomised root name does not leak into assertions.
fn tree_lines(doc: &str) -> Vec<String> {
    let structure = doc.split("## Structure").nth(1).unwrap();
    let block = structure.split("## Contents").next().unwrap();
    block
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.starts_with("```"))
        .skip(1) // the root line carries the temp directory's name
        .map(|l| l.to_string())
        .collect()
}

/// A fixture exercising every interesting case at once:
/// nested dirs, an ignored build dir, a dot-entry, a `.github` dir, a binary
/// file, a Markdown file containing a fence, and two plain text files.
fn fixture(tag: &str) -> Tmp {
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
}

#[test]
fn test_cli_document_section_order() {
    let dir = fixture("order");
    let doc = ok_stdout(&dir, &[]);
    let title = doc.lines().position(|l| l.starts_with("# ")).unwrap();
    let structure = doc.lines().position(|l| l == "## Structure").unwrap();
    let contents = doc.lines().position(|l| l == "## Contents").unwrap();
    assert!(title < structure && structure < contents);
}

#[test]
fn test_cli_tree_shape() {
    let dir = fixture("shape");
    let doc = ok_stdout(&dir, &[]);
    // The whole scaffold, compared literally. Substring checks would not catch
    // wrong ordering, a duplicated subtree, or a dropped entry.
    let expected = [
        "├── a.txt",
        "├── blob.bin",
        "├── debug.log",
        "├── notes.md",
        "├── one.txt",
        "├── src/",
        "│   └── main.rs",
        "└── sub/",
        "    └── one.txt",
    ];
    assert_eq!(tree_lines(&doc), expected);
}

#[test]
fn test_cli_file_header_format() {
    let dir = fixture("header");
    let doc = ok_stdout(&dir, &[]);
    assert!(doc.contains("\n---\nFile: src/main.rs\n---\n"));
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
    // Positive controls. Without these, a regression that emitted an empty
    // document would satisfy every assertion above.
    assert!(doc.contains("├── a.txt"), "non-ignored siblings survive");
    assert!(doc.contains("File: src/main.rs"));
    assert!(doc.contains("fn main() {}"), "bodies are still emitted");
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
}

#[test]
fn test_cli_exclude_adds_pattern() {
    let dir = fixture("exclude");
    let doc = ok_stdout(&dir, &["--exclude", "*.log"]);
    assert!(!doc.contains("debug.log"));
    assert!(!doc.contains("noisy"));
    assert!(doc.contains("a.txt"), "siblings survive");
}

#[test]
fn test_cli_exclude_single_file_by_path() {
    let dir = fixture("bypath");
    let doc = ok_stdout(&dir, &["--exclude", "sub/one.txt"]);
    assert!(!doc.contains("nested one"));
    // The same-named file at the root is untouched.
    assert!(doc.contains("File: one.txt"));
    assert!(doc.contains("root one"));
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
}

#[test]
fn test_cli_markdown_body_does_not_close_its_own_fence() {
    let dir = fixture("fences");
    let doc = ok_stdout(&dir, &[]);
    // The notes.md body holds a ``` run, so its fence must be longer.
    assert!(doc.contains("````markdown\n"));
    assert!(
        fences_balanced(&doc),
        "every fence opened must be closed by a long-enough fence"
    );
    // The body's own ``` lines survive inside the block rather than ending it.
    assert!(doc.contains("before\n\n```\nfenced\n```\n\nafter"));
    // The invariant balance alone cannot see: a too-short opener would close
    // early on the body's first fence line, re-open on the second, and still
    // balance. The opener must strictly outgrow the longest run inside.
    let (opener, longest) = body_fence_vs_content(&doc, "notes.md");
    assert_eq!(longest, 3, "fixture body holds a triple-backtick run");
    assert!(
        opener > longest,
        "opening fence {opener} must outgrow the body's longest run {longest}"
    );
}

/// The balance checker must be able to fail, or the test above proves nothing.
#[test]
fn test_fences_balanced_detects_imbalance() {
    assert!(fences_balanced("```rust\nx\n```\n"));
    assert!(fences_balanced("````md\n```\ninner\n```\n````\n"));
    assert!(!fences_balanced("```rust\nx\n"), "unclosed fence");
    assert!(
        !fences_balanced("````md\nx\n```\n"),
        "closer shorter than opener does not close"
    );
    // Known limitation, recorded rather than papered over: a too-short opener
    // closes early and re-opens on the body's next fence line, so the document
    // still balances. That regression is caught by the opener-vs-content
    // invariant in test_cli_markdown_body_does_not_close_its_own_fence, not
    // here.
    assert!(fences_balanced("```md\n```\ninner\n```\n```\n"));
}

#[test]
fn test_cli_fence_mode_tree_in_one_block() {
    let dir = fixture("treefence");
    let doc = ok_stdout(&dir, &[]);
    let structure = doc.split("## Structure").nth(1).unwrap();
    let tree = structure.split("## Contents").next().unwrap();
    assert_eq!(tree.matches("```").count(), 2, "exactly one fenced block");
}

#[test]
fn test_cli_rust_body_carries_language_hint() {
    let dir = fixture("hint");
    let doc = ok_stdout(&dir, &[]);
    assert!(doc.contains("```rust\nfn main() {}"));
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
}

#[test]
fn test_cli_inline_mode_body_carries_language_hint() {
    let dir = fixture("inlinehint");
    let doc = ok_stdout(&dir, &["--wrap", "inline"]);
    assert!(doc.contains("```rust\nfn main() {}"));
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
    // And bodies are unfenced too: the header is followed directly by the
    // file's text. Asserted separately, because the tree slice above cannot
    // see the content sections at all.
    assert!(
        doc.contains("---\nFile: src/main.rs\n---\n\nfn main() {}\n"),
        "a body in none mode carries no fence"
    );
}

#[test]
fn test_cli_wrap_none_preserves_section_separation() {
    let dir = fixture("sep");
    let doc = ok_stdout(&dir, &["--wrap", "none"]);
    // a.txt's body ends in a text line; without the blank line the next
    // section's `---` would underline it as a setext heading and swallow the
    // following header.
    assert!(doc.contains("last line of a\n\n---\nFile: "));
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
}

#[test]
fn test_cli_nested_paths_use_forward_slashes() {
    let dir = fixture("slashes");
    let doc = ok_stdout(&dir, &[]);
    for line in doc.lines().filter(|l| l.starts_with("File: ")) {
        assert!(!line.contains('\\'), "backslash in path: {line}");
    }
    assert!(doc.contains("File: sub/one.txt"));
}

#[test]
fn test_cli_output_is_byte_identical_across_runs() {
    let dir = fixture("determinism");
    // Both runs must succeed and produce something: two identically-empty
    // outputs from a binary that started failing would otherwise "prove"
    // determinism.
    let first = ok_stdout(&dir, &[]);
    let second = ok_stdout(&dir, &[]);
    assert!(!first.is_empty());
    assert_eq!(first, second);
}

/// With no positional argument MDeezl scans the current directory. Every other
/// end-to-end test passes an explicit path, so without this one that half of
/// the criterion is only ever checked at the argument-parsing level.
#[test]
fn test_cli_no_path_scans_current_directory() {
    let dir = fixture("cwd");
    let out = Command::new(EXE)
        .current_dir(&dir)
        .output()
        .expect("failed to run mdeezl");
    assert!(out.status.success());
    let doc = stdout_of(&out);
    let root_name = dir.file_name().unwrap().to_string_lossy().into_owned();
    assert!(
        doc.starts_with(&format!("# {root_name}\n")),
        "the title must name the scanned directory, not `.`"
    );
    assert!(doc.contains("File: src/main.rs"));
}

#[test]
fn test_cli_usage_error_exits_2() {
    let dir = fixture("usage");
    let out = run(&dir, &["--wrap", "banana"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
    assert!(String::from_utf8_lossy(&out.stderr).contains("banana"));
}

#[test]
fn test_cli_missing_root_exits_1_with_empty_stdout() {
    let missing = std::env::temp_dir().join("mdeezl-absent-xyzzy");
    // The test means nothing if something else ever creates this path.
    assert!(
        !missing.exists(),
        "precondition: {missing:?} must not exist"
    );
    let out = Command::new(EXE).arg(&missing).output().unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(out.stdout.is_empty(), "no partial document on stdout");
    assert!(!out.stderr.is_empty());
}

#[test]
fn test_cli_file_as_root_exits_1() {
    let dir = fixture("fileroot");
    let out = Command::new(EXE)
        .arg(dir.join("a.txt"))
        .output()
        .expect("failed to run mdeezl");
    assert_eq!(out.status.code(), Some(1));
    assert!(
        out.stdout.is_empty(),
        "no document for a non-directory root"
    );
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("not a directory"),
        "the message must say why"
    );
}

#[test]
fn test_cli_unwritable_output_exits_1() {
    let dir = fixture("unwritable");
    let bad = dir.join("no-such-dir").join("out.md");
    let out = run(&dir, &["-o", bad.to_str().unwrap()]);
    assert_eq!(out.status.code(), Some(1), "not a panic's 101");
    assert!(out.stdout.is_empty());
    assert!(!out.stderr.is_empty());
}

/// T-006. The one check whose absence would let the sprint checkpoint be green
/// because nothing ran.
#[test]
fn test_ci_workflow_runs_tests_on_both_platforms() {
    let workflow = include_str!("../.github/workflows/sprint-loops-ci.yml");
    assert!(workflow.contains("cargo test"), "CI must run the tests");
    assert!(
        workflow.contains("ubuntu-latest"),
        "the symlink guarantee needs a Linux runner"
    );
    assert!(
        workflow.contains("windows-latest"),
        "the path-separator guarantee needs a Windows runner"
    );
}

/// T-007. The README carries the same surface as `--help`, for a reader who
/// has not built the binary yet.
#[test]
fn test_readme_documents_cli_surface() {
    let readme = include_str!("../README.md");
    for flag in ["-o", "--wrap", "--exclude", "--include", "--help"] {
        assert!(readme.contains(flag), "README must document {flag}");
    }
    for form in [".*", "*.ext", "anything else"] {
        assert!(
            readme.contains(form),
            "README must document the {form} form"
        );
    }
    assert!(readme.contains("contains `/`"), "per-file pattern form");
    for ignored in ["target", "node_modules", "dist", "build", "__pycache__"] {
        assert!(
            readme.contains(ignored),
            "README must list the pre-populated entry {ignored}"
        );
    }
    for mode in ["fence", "inline", "none"] {
        assert!(
            readme.contains(mode),
            "README must document wrap mode {mode}"
        );
    }
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
