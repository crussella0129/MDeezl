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

// ======================= Sprint 1: gitignore (INT-0004) =======================

/// T-001. `--help` must say where an omission can come from.
#[test]
fn test_cli_help_names_both_exclusion_sources() {
    let out = isolate(Command::new(EXE).arg("--help")).output().unwrap();
    assert!(out.status.success());
    let help = String::from_utf8(out.stdout).unwrap();
    // Needles only the sprint 1 text contains: sprint 0's help already said
    // "PATH" (the positional argument) and "the ignore list".
    for needle in [
        "--include outranks both",
        ".gitignore",
        "--no-gitignore",
        "git on your PATH",
        "git work tree",
    ] {
        assert!(help.contains(needle), "help must mention {needle}");
    }
}

// ---- Sprint 1 test harness: real git repositories, isolated from the host ----

/// Every variable `git rev-parse --local-env-vars` lists (git 2.54).
const GIT_LOCAL_ENV_VARS: [&str; 15] = [
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
    "GIT_CONFIG",
    "GIT_CONFIG_PARAMETERS",
    "GIT_CONFIG_COUNT",
    "GIT_OBJECT_DIRECTORY",
    "GIT_DIR",
    "GIT_WORK_TREE",
    "GIT_IMPLICIT_WORK_TREE",
    "GIT_GRAFT_FILE",
    "GIT_INDEX_FILE",
    "GIT_NO_REPLACE_OBJECTS",
    "GIT_REPLACE_REF_BASE",
    "GIT_PREFIX",
    "GIT_SHALLOW_FILE",
    "GIT_COMMON_DIR",
];

/// Isolate a command from the host's git configuration and any ambient
/// repository. `XDG_CONFIG_HOME` matters as much as `GIT_CONFIG_GLOBAL`: git's
/// default excludes file lives at `$XDG_CONFIG_HOME/git/ignore`, which
/// `GIT_CONFIG_GLOBAL` alone does not disable.
fn isolate(cmd: &mut Command) -> &mut Command {
    let iso = std::env::temp_dir().join(format!("mdeezl-e2e-gitiso-{}", std::process::id()));
    fs::create_dir_all(&iso).unwrap();
    let _ = fs::write(iso.join("gitconfig"), "");
    cmd.env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", iso.join("gitconfig"))
        .env("XDG_CONFIG_HOME", &iso)
        .env("GIT_CEILING_DIRECTORIES", std::env::temp_dir());
    for var in GIT_LOCAL_ENV_VARS {
        cmd.env_remove(var);
    }
    cmd
}

fn git_available() -> bool {
    Command::new("git")
        .arg("--version")
        .output()
        .is_ok_and(|o| o.status.success())
}

macro_rules! require_git {
    ($name:literal) => {
        if !git_available() {
            eprintln!(concat!("SKIP ", $name, ": git not on PATH"));
            return;
        }
    };
}

fn git(dir: &Path, args: &[&str]) {
    let out = isolate(Command::new("git").current_dir(dir).args(args))
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

fn git_repo(tag: &str) -> Tmp {
    let dir = tmp_dir(tag);
    git(&dir, &["init", "-q", "."]);
    dir
}

fn put(dir: &Path, rel: &str, body: &str) {
    let p = dir.join(rel);
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(p, body).unwrap();
}

/// Run the binary against `root`, isolated.
fn run_iso(root: &Path, args: &[&str]) -> Output {
    isolate(Command::new(EXE).arg(root).args(args))
        .output()
        .expect("failed to run mdeezl")
}

fn stderr_of(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

/// The sprint 0 fixture's scaffold, which every degraded run must still produce.
const FIXTURE_TREE: [&str; 9] = [
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

const SKIP_NOTICE: &str = "mdeezl: gitignore filtering skipped:";

// ---- T-002: verifiable before pruning exists ----

#[test]
fn test_cli_no_gitignore_does_not_invoke_git() {
    let dir = fixture("nogi-plain");
    let default_run = run_iso(&dir, &[]);
    let off = run_iso(&dir, &["--no-gitignore"]);
    assert!(default_run.status.success() && off.status.success());
    // In a plain directory the default run degrades and says so...
    assert!(stderr_of(&default_run).contains(SKIP_NOTICE));
    // ...so an empty stderr here proves git was never consulted,
    assert!(off.stderr.is_empty(), "stderr: {}", stderr_of(&off));
    // and the document equals what the ignore list alone produces.
    assert_eq!(default_run.stdout, off.stdout);
}

#[test]
fn test_cli_plain_directory_degrades_with_reason() {
    require_git!("test_cli_plain_directory_degrades_with_reason");
    let dir = fixture("plain-degrade");
    let out = run_iso(&dir, &[]);
    assert!(out.status.success());
    let err = stderr_of(&out);
    assert_eq!(err.lines().count(), 1, "exactly one stderr line: {err}");
    assert!(err.starts_with(SKIP_NOTICE), "{err}");
    assert!(err.contains("not a git repository"), "{err}");
    assert_eq!(tree_lines(&stdout_of(&out)), FIXTURE_TREE);
}

#[test]
fn test_cli_missing_git_degrades_with_reason() {
    let dir = fixture("nogit");
    // `.env("PATH", "")`, not `env_remove`: glibc answers a missing PATH by
    // searching /bin:/usr/bin and would find git. Not `env_clear`: that drops
    // SystemRoot on Windows. On Windows std searches the application
    // directory, System32 and the Windows directory before PATH; none hold git
    // on a standard runner.
    let out = isolate(
        Command::new(EXE)
            .arg(&*dir)
            .current_dir(&*dir)
            .env("PATH", ""),
    )
    .output()
    .expect("failed to run mdeezl");
    assert!(out.status.success());
    let err = stderr_of(&out);
    assert_eq!(err.lines().count(), 1, "exactly one stderr line: {err}");
    assert!(err.starts_with(SKIP_NOTICE), "{err}");
    assert!(err.contains("git was not found"), "{err}");
    assert_eq!(tree_lines(&stdout_of(&out)), FIXTURE_TREE);
}

#[test]
fn test_cli_ignored_scan_root_degrades_with_reason() {
    require_git!("test_cli_ignored_scan_root_degrades_with_reason");
    let dir = git_repo("ignored-root");
    put(&dir, ".gitignore", "out/\n");
    put(&dir, "out/a.o", "object\n");
    put(&dir, "out/b.txt", "text\n");
    let out = run_iso(&dir.join("out"), &[]);
    assert!(out.status.success());
    let err = stderr_of(&out);
    assert_eq!(err.lines().count(), 1, "exactly one stderr line: {err}");
    assert!(err.starts_with(SKIP_NOTICE), "{err}");
    assert!(err.contains("itself ignored"), "{err}");
    // Every file present, not a title over an empty tree.
    let doc = stdout_of(&out);
    assert!(doc.contains("File: a.o") && doc.contains("File: b.txt"));
}

#[test]
fn test_cli_ignores_ambient_git_environment() {
    require_git!("test_cli_ignores_ambient_git_environment");
    let dir = git_repo("ambient-env");
    put(&dir, ".gitignore", "*.log\n");
    put(&dir, "app.log", "log\n");
    put(&dir, "keep.txt", "keep\n");
    put(&dir, "tracked.log", "tracked\n");
    git_add(&dir, &["tracked.log"]);
    let bogus = std::env::temp_dir().join("mdeezl-no-such-git-dir");
    assert!(!bogus.exists(), "precondition: {bogus:?} must not exist");
    // Deliberately NOT through `isolate`'s removal: these variables are the
    // point. Left in place, GIT_DIR and GIT_WORK_TREE would override -C and
    // make git fail loudly. GIT_INDEX_FILE would fail *silently*: pointed at a
    // missing index, git would treat tracked.log as untracked and omit it.
    let mut cmd = Command::new(EXE);
    isolate(cmd.arg(&*dir));
    let out = cmd
        .env("GIT_DIR", &bogus)
        .env("GIT_WORK_TREE", &bogus)
        .env("GIT_INDEX_FILE", bogus.join("index"))
        .output()
        .expect("failed to run mdeezl");
    assert!(out.status.success());
    assert!(out.stderr.is_empty(), "stderr: {}", stderr_of(&out));
    let doc = stdout_of(&out);
    assert!(
        !in_contents(&doc, "app.log"),
        "the scan root's rules applied"
    );
    assert!(
        in_contents(&doc, "tracked.log"),
        "the scan root's index applied"
    );
}

// ---- T-003: pruning, end to end ----

/// Stage without committing; literal pathspecs so bracketed names are not globs.
fn git_add(dir: &Path, paths: &[&str]) {
    let mut args = vec!["--literal-pathspecs", "add", "-f", "--"];
    args.extend_from_slice(paths);
    git(dir, &args);
}

fn ok_iso(root: &Path, args: &[&str]) -> String {
    let out = run_iso(root, args);
    assert!(out.status.success(), "mdeezl failed: {}", stderr_of(&out));
    stdout_of(&out)
}

/// Is `rel` present in the content sections?
fn in_contents(doc: &str, rel: &str) -> bool {
    doc.contains(&format!("\nFile: {rel}\n"))
}

/// Is an entry named `name` present in the scaffold?
fn in_tree(doc: &str, name: &str) -> bool {
    tree_lines(doc)
        .iter()
        .any(|l| l.ends_with(&format!("── {name}")) || l.ends_with(&format!("── {name}/")))
}

/// `out/`, `*.log`, `!keep.log` -- `out/` is deliberately not on the built-in list.
fn gitignore_fixture(tag: &str) -> Tmp {
    let dir = git_repo(tag);
    put(&dir, ".gitignore", "out/\n*.log\n!keep.log\n");
    put(&dir, "out/a.o", "object body\n");
    put(&dir, "app.log", "app log body\n");
    put(&dir, "keep.log", "keep log body\n");
    put(&dir, "keep.txt", "keep text body\n");
    put(&dir, "src/main.rs", "fn main() {}\n");
    // On the built-in list, not in .gitignore: must still be omitted when the
    // query succeeds, since gitignore layers onto the list, never replaces it.
    put(&dir, "target/artifact.txt", "built\n");
    dir
}

#[test]
fn test_cli_gitignore_omits_from_both_halves() {
    require_git!("test_cli_gitignore_omits_from_both_halves");
    let dir = gitignore_fixture("gi-both");
    let out = run_iso(&dir, &[]);
    assert!(out.status.success());
    assert!(
        out.stderr.is_empty(),
        "the query must run: {}",
        stderr_of(&out)
    );
    let doc = stdout_of(&out);
    // The built-in list still applies on a run where git answered.
    assert!(!in_tree(&doc, "target"), "built-in list: target/");
    assert!(!in_tree(&doc, ".git"), "built-in list: .git");
    assert!(!in_tree(&doc, ".gitignore"), "built-in list: .gitignore");
    assert!(!doc.contains("built\n"));
    for gone in ["out", "a.o", "app.log"] {
        assert!(
            !in_tree(&doc, gone),
            "{gone} must be absent from the scaffold"
        );
    }
    for gone in ["out/a.o", "app.log"] {
        assert!(
            !in_contents(&doc, gone),
            "{gone} must be absent from the contents"
        );
    }
    assert!(!doc.contains("object body") && !doc.contains("app log body"));
    for kept in ["keep.log", "keep.txt", "src/main.rs"] {
        assert!(in_contents(&doc, kept), "{kept} must be present");
    }
    assert!(in_tree(&doc, "keep.log") && in_tree(&doc, "keep.txt"));
}

#[test]
fn test_cli_gitignore_honours_full_syntax() {
    require_git!("test_cli_gitignore_honours_full_syntax");
    let dir = git_repo("gi-syntax");
    put(&dir, ".gitignore", "*.log\n!keep.log\n**/generated/\n");
    put(&dir, "sub/.gitignore", "local.tmp\n");
    let exclude = dir.join(".git/info/exclude");
    let mut ex = fs::read_to_string(&exclude).unwrap_or_default();
    ex.push_str("excluded.txt\n");
    fs::write(&exclude, ex).unwrap();
    put(&dir, "globalish", "viaconfig.txt\n");
    // Forward slashes: a Windows backslash path in .git/config is read as escapes.
    let excludes_file = dir.join("globalish").to_string_lossy().replace('\\', "/");
    git(&dir, &["config", "core.excludesFile", &excludes_file]);
    for f in [
        "app.log",
        "keep.log",
        "deep/x/generated/g.txt",
        "sub/local.tmp",
        "sub/other.tmp",
        "excluded.txt",
        "viaconfig.txt",
        "plain.txt",
    ] {
        put(&dir, f, "x\n");
    }
    let doc = ok_iso(&dir, &[]);
    for gone in [
        "app.log",
        "deep/x/generated/g.txt",
        "sub/local.tmp",
        "excluded.txt",
        "viaconfig.txt",
    ] {
        assert!(!in_contents(&doc, gone), "{gone} must be absent");
    }
    assert!(!in_tree(&doc, "generated"));
    for kept in ["keep.log", "sub/other.tmp", "plain.txt"] {
        assert!(in_contents(&doc, kept), "{kept} must be present");
    }
}

#[test]
fn test_cli_scan_subdir_applies_root_rules() {
    require_git!("test_cli_scan_subdir_applies_root_rules");
    let dir = git_repo("gi-subdir");
    put(&dir, ".gitignore", "*.log\n");
    put(&dir, "sub/x.log", "x\n");
    put(&dir, "sub/keep.txt", "k\n");
    let doc = ok_iso(&dir.join("sub"), &[]);
    assert!(!in_contents(&doc, "x.log"));
    assert!(in_contents(&doc, "keep.txt"));
}

#[test]
fn test_cli_tracked_file_matching_pattern_is_bundled() {
    require_git!("test_cli_tracked_file_matching_pattern_is_bundled");
    let dir = git_repo("gi-tracked");
    put(&dir, ".gitignore", "*.log\n");
    put(&dir, "tracked.log", "tracked body\n");
    put(&dir, "free.log", "untracked body\n");
    git_add(&dir, &["tracked.log"]);
    let out = run_iso(&dir, &[]);
    assert!(out.status.success());
    assert!(
        out.stderr.is_empty(),
        "the query must run: {}",
        stderr_of(&out)
    );
    let doc = stdout_of(&out);
    assert!(in_contents(&doc, "tracked.log"));
    assert!(doc.contains("tracked body"));
    assert!(
        !in_contents(&doc, "free.log"),
        "control: the rule does apply"
    );
}

#[test]
fn test_cli_tracked_file_inside_ignored_directory() {
    require_git!("test_cli_tracked_file_inside_ignored_directory");
    let dir = git_repo("gi-tracked-in-ignored");
    put(&dir, ".gitignore", "out/\n");
    put(&dir, "out/keep.txt", "kept body\n");
    put(&dir, "out/a.o", "object\n");
    git_add(&dir, &["out/keep.txt"]);

    let doc = ok_iso(&dir, &[]);
    assert!(in_contents(&doc, "out/keep.txt"));
    assert!(!in_contents(&doc, "out/a.o"));

    // Scanning out/ itself: it holds tracked content, so git does not report it
    // ignored -- no skip notice, and filtering still applies inside it.
    let out = run_iso(&dir.join("out"), &[]);
    assert!(out.status.success());
    assert!(out.stderr.is_empty(), "stderr: {}", stderr_of(&out));
    let doc = stdout_of(&out);
    assert!(in_contents(&doc, "keep.txt"));
    assert!(!in_contents(&doc, "a.o"));
}

#[test]
fn test_cli_no_gitignore_restores_paths() {
    require_git!("test_cli_no_gitignore_restores_paths");
    let dir = gitignore_fixture("gi-off");
    let doc = ok_iso(&dir, &["--no-gitignore"]);
    for back in ["out/a.o", "app.log", "keep.log", "keep.txt"] {
        assert!(in_contents(&doc, back), "{back} must return");
    }
}

#[test]
fn test_cli_include_beats_gitignore() {
    require_git!("test_cli_include_beats_gitignore");
    let dir = gitignore_fixture("gi-include");
    let doc = ok_iso(&dir, &["--include", "out"]);
    assert!(in_tree(&doc, "out"));
    assert!(in_contents(&doc, "out/a.o"));
    assert!(doc.contains("object body"));
    assert!(!in_contents(&doc, "app.log"), "other rules still apply");
}

#[test]
fn test_cli_include_dot_star_with_gitignore() {
    require_git!("test_cli_include_dot_star_with_gitignore");
    let dir = git_repo("gi-dotstar");
    put(&dir, ".gitignore", ".cache/\nout/\n");
    put(&dir, ".cache/c.bin", "cached\n");
    put(&dir, "out/a.o", "object\n");
    let out = run_iso(&dir, &["--include", ".*"]);
    assert!(out.status.success());
    assert!(out.stderr.is_empty(), "stderr: {}", stderr_of(&out));
    let doc = stdout_of(&out);
    assert!(in_tree(&doc, ".cache"));
    assert!(in_contents(&doc, ".cache/c.bin"));
    assert!(in_tree(&doc, ".git"));
    // Control: pruning did run, so .cache survived because of the include.
    assert!(
        !in_tree(&doc, "out"),
        "a non-dot gitignored entry is still pruned"
    );
}

#[test]
fn test_cli_submodule_contents_not_filtered() {
    require_git!("test_cli_submodule_contents_not_filtered");
    let src = git_repo("gi-subsrc");
    put(&src, "inner.txt", "inner\n");
    git_add(&src, &["inner.txt"]);
    git(
        &src,
        &[
            "-c",
            "user.name=t",
            "-c",
            "user.email=t@t",
            "commit",
            "-qm",
            "i",
        ],
    );

    let dir = git_repo("gi-submod");
    put(&dir, ".gitignore", "*.log\n");
    let src_url = src.to_string_lossy().replace('\\', "/");
    git(
        &dir,
        &[
            "-c",
            "protocol.file.allow=always",
            "submodule",
            "add",
            "-q",
            &src_url,
            "mod",
        ],
    );
    put(&dir, "mod/x.log", "inside submodule\n");
    put(&dir, "top.log", "top\n");

    let out = run_iso(&dir, &[]);
    assert!(out.status.success());
    assert!(
        out.stderr.is_empty(),
        "the query must survive: {}",
        stderr_of(&out)
    );
    let doc = stdout_of(&out);
    assert!(!in_contents(&doc, "top.log"));
    assert!(
        in_contents(&doc, "mod/x.log"),
        "no gitignore inside a submodule"
    );
}

#[test]
fn test_cli_tracked_file_in_escaped_directory_is_kept() {
    require_git!("test_cli_tracked_file_in_escaped_directory_is_kept");
    let dir = git_repo("gi-escaped-dir");
    // The whitelist idiom: git reports app/[slug] ignored (it cannot lstat the
    // escaped name to apply `!*/`) while the tracked page.tsx is not.
    put(&dir, ".gitignore", "*\n!*/\n!*.tsx\n!.gitignore\n");
    put(
        &dir,
        "app/[slug]/page.tsx",
        "export default function Page() {}\n",
    );
    put(&dir, "app/[slug]/notes.md", "untracked notes\n");
    git_add(&dir, &["app/[slug]/page.tsx"]);
    let doc = ok_iso(&dir, &[]);
    assert!(in_contents(&doc, "app/[slug]/page.tsx"));
    assert!(doc.contains("export default function Page"));
    assert!(!in_contents(&doc, "app/[slug]/notes.md"));
}

#[test]
fn test_cli_glob_char_filename_is_literal() {
    require_git!("test_cli_glob_char_filename_is_literal");
    let dir = git_repo("gi-globname");
    put(&dir, ".gitignore", "*.log\n");
    put(&dir, "a1.log", "tracked\n");
    put(&dir, "a[1].log", "untracked\n");
    git_add(&dir, &["a1.log"]);
    let doc = ok_iso(&dir, &[]);
    assert!(in_contents(&doc, "a1.log"));
    assert!(!in_contents(&doc, "a[1].log"));
}

#[test]
fn test_cli_colon_prefixed_filename_is_a_path() {
    if cfg!(not(unix)) {
        eprintln!(
            "SKIP test_cli_colon_prefixed_filename_is_a_path: this platform \
             forbids `:` in filenames. The Linux CI leg runs it."
        );
        return;
    }
    require_git!("test_cli_colon_prefixed_filename_is_a_path");
    let dir = git_repo("gi-colon");
    put(&dir, ".gitignore", "secret\n*.log\n");
    put(&dir, ":secret", "not a pathspec\n");
    put(&dir, ":x.log", "matched\n");
    let doc = ok_iso(&dir, &[]);
    assert!(
        in_contents(&doc, ":secret"),
        "as a path, :secret does not match `secret`"
    );
    assert!(!in_contents(&doc, ":x.log"));
}

#[test]
fn test_cli_newline_in_filename_is_matched() {
    if cfg!(not(unix)) {
        eprintln!(
            "SKIP test_cli_newline_in_filename_is_matched: this platform \
             forbids newlines in filenames. The Linux CI leg runs it."
        );
        return;
    }
    require_git!("test_cli_newline_in_filename_is_matched");
    let dir = git_repo("gi-newline");
    put(&dir, ".gitignore", "*.log\n");
    put(&dir, "line1\nline2.log", "multi-line name\n");
    put(&dir, "keep.txt", "k\n");
    let doc = ok_iso(&dir, &[]);
    assert!(!doc.contains("multi-line name"));
    assert!(in_contents(&doc, "keep.txt"));
}

#[test]
fn test_cli_large_repo_completes() {
    require_git!("test_cli_large_repo_completes");
    let dir = git_repo("gi-large");
    put(&dir, ".gitignore", "*.o\n");
    for i in 0..5_000 {
        let ext = if i % 2 == 0 { "o" } else { "txt" };
        put(
            &dir,
            &format!("data/d{:02}/file_{i:05}.{ext}", i / 100),
            "x\n",
        );
    }
    let out_dir = tmp_dir("gi-large-out");
    let doc_path = out_dir.join("doc.md");
    // Written to -o so the harness never has to drain a large stdout; polling
    // try_wait while leaving stdout unread would otherwise block the child.
    let mut cmd = Command::new(EXE);
    isolate(cmd.arg(&*dir).arg("-o").arg(&doc_path));
    let mut child = cmd.spawn().expect("failed to run mdeezl");
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    let status = loop {
        if let Some(s) = child.try_wait().unwrap() {
            break s;
        }
        if std::time::Instant::now() > deadline {
            let _ = child.kill();
            panic!("mdeezl did not finish within 60 s");
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    };
    assert!(status.success());
    let doc = fs::read_to_string(&doc_path).unwrap();
    assert!(!doc.contains(".o\n"), "no ignored .o file survives");
    assert_eq!(doc.matches("\nFile: data/").count(), 2_500, "the kept half");
}

#[test]
fn test_cli_gitignore_output_is_byte_identical_across_runs() {
    require_git!("test_cli_gitignore_output_is_byte_identical_across_runs");
    let dir = gitignore_fixture("gi-determinism");
    let first = run_iso(&dir, &[]);
    let second = run_iso(&dir, &[]);
    assert!(first.status.success() && second.status.success());
    assert!(
        first.stderr.is_empty(),
        "the query must run: {}",
        stderr_of(&first)
    );
    assert!(!first.stdout.is_empty());
    assert!(
        !in_contents(&stdout_of(&first), "app.log"),
        "gitignore applied"
    );
    assert_eq!(first.stdout, second.stdout);
}

// ---- T-004: the README ----

#[test]
fn test_readme_documents_gitignore() {
    let readme = include_str!("../README.md");
    for (what, needle) in [
        ("the second exclusion source", "`.gitignore`"),
        ("the off switch", "--no-gitignore"),
        ("the git requirement", "Git must be on your `PATH`"),
        ("the work-tree requirement", "git work tree"),
        (
            "the degradation behaviour",
            "gitignore filtering was skipped",
        ),
        ("the tracked-file rule", "Tracked files are kept"),
        (
            "the directory-include exemption",
            "Including a gitignored directory",
        ),
        ("the exemption's limit", "`--include build`"),
        (
            "the ancestor rule",
            "An include cannot reach inside a pruned directory",
        ),
        (
            "the submodule limitation",
            "Nested repositories and submodules",
        ),
        (
            "the glob-character limitation",
            "are matched only partially",
        ),
        (
            "the case an include cannot rescue",
            "Include the directory instead",
        ),
    ] {
        assert!(readme.contains(needle), "README must document {what}");
    }
    assert!(
        !readme.contains("No gitignore support yet"),
        "the sprint 0 statement must be gone"
    );
}

/// An ordinary nested clone -- its `.git` is a directory, unlike a submodule's
/// gitlink file. The enclosing repository's rules are the wrong rules for it,
/// so its contents keep only the ignore list.
#[test]
fn test_cli_nested_repository_not_filtered() {
    require_git!("test_cli_nested_repository_not_filtered");
    let dir = git_repo("gi-nested-clone");
    put(&dir, ".gitignore", "*.log\n");
    put(&dir, "top.log", "top\n");
    put(&dir, "inner/x.log", "inside nested clone\n");
    git(&dir.join("inner"), &["init", "-q", "."]);
    assert!(
        dir.join("inner/.git").is_dir(),
        "precondition: .git is a directory"
    );

    let out = run_iso(&dir, &[]);
    assert!(out.status.success());
    assert!(out.stderr.is_empty(), "stderr: {}", stderr_of(&out));
    let doc = stdout_of(&out);
    assert!(
        !in_contents(&doc, "top.log"),
        "the enclosing rules apply outside"
    );
    assert!(
        in_contents(&doc, "inner/x.log"),
        "not inside the nested repository"
    );
}

// ======================= Sprint 2: toolchain policy (INT-0005) =======================

/// Read a repository file at runtime, not with `include_str!`, so a deleted
/// file fails one named test instead of breaking compilation of this file.
/// CRLF is normalised: the Windows runner checks files out with CRLF.
fn repo_file(rel: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("{rel} must exist: {e}"))
        .replace("\r\n", "\n")
}

/// T-001. The file names a channel and both components. It deliberately does
/// NOT require `stable`: pinning must stay a one-line change to the file, and
/// a test demanding `stable` would make it a two-file change.
#[test]
fn test_toolchain_file_declares_channel_and_components() {
    let toml = repo_file("rust-toolchain.toml");
    let lines: Vec<&str> = toml.lines().map(str::trim).collect();
    assert!(lines.contains(&"[toolchain]"), "a [toolchain] table");
    let channel = lines
        .iter()
        .find_map(|l| l.strip_prefix("channel = \""))
        .and_then(|rest| rest.strip_suffix('"'));
    assert!(
        matches!(channel, Some(c) if !c.is_empty()),
        "a non-empty channel, stable or a pinned version"
    );
    let components = lines
        .iter()
        .find(|l| l.starts_with("components"))
        .expect("a components line");
    for c in ["\"rustfmt\"", "\"clippy\""] {
        assert!(components.contains(c), "components must list {c}");
    }
}

/// T-001. CI records the image's stable, updates, installs, records what it
/// now uses, then runs the gates -- in that order, one command per step.
#[test]
fn test_ci_updates_installs_and_logs_in_order() {
    let wf = repo_file(".github/workflows/sprint-loops-ci.yml");
    let pos = |needle: &str, from: usize| -> usize {
        wf[from..]
            .find(needle)
            .map(|i| i + from)
            .unwrap_or_else(|| panic!("workflow must contain {needle:?} after byte {from}"))
    };

    let pre = pos("rustc +stable --version", 0);
    let update = pos("- run: rustup update --no-self-update stable\n", 0);
    // Ends at the newline: no toolchain argument, so the file decides.
    let install = pos("- run: rustup toolchain install --no-self-update\n", 0);
    assert!(pre < update, "image version is logged before the update");
    assert!(update < install, "update comes before the install");

    let post = pos("rustc --version", install);
    let mut log_end = post;
    for needle in [
        "cargo --version",
        "cargo clippy --version",
        "rustup --version",
        "git --version",
    ] {
        log_end = log_end.max(pos(needle, install));
    }
    let fmt = pos("cargo fmt --check", 0);
    assert!(
        log_end < fmt,
        "the toolchain is logged before the gates run"
    );

    // The post-install log's own step runs under bash on both legs.
    let step_start = wf[..post].rfind("\n      - ").expect("step start");
    let step_end = wf[post..]
        .find("\n      - ")
        .map(|i| i + post)
        .unwrap_or(wf.len());
    assert!(
        wf[step_start..step_end].contains("shell: bash"),
        "the version log must run under bash so any failing line fails it"
    );

    // Exits 100 whenever any update exists; would fail CI for no code reason.
    assert!(!wf.contains("rustup check"));
    // INT-0001's two-OS criterion must not regress.
    assert!(wf.contains("fail-fast: false"));
    assert!(wf.contains("cargo test --all -- --nocapture"));
}

/// T-002. The README's Toolchain section states each required item; one
/// literal string per item, as the build plan lists them.
#[test]
fn test_readme_documents_toolchain_policy() {
    let readme = repo_file("README.md");
    for needle in [
        "Track stable by default",
        "rust-toolchain.toml",
        "rustup update stable",
        "rustup toolchain install",
        "To pin",
        "rustup 1.29",
        "rustup self update",
        "a new stable",
        "git's version",
    ] {
        assert!(readme.contains(needle), "README must say {needle:?}");
    }
}
