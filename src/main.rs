//! MDeezl — feed any repository to text-only agents.
//!
//! Emits one Markdown document for a directory: a box-drawing scaffold tree,
//! then the contents of every included file. Standard library only; see
//! `docs/intents/INT-0001-markdown-repo-context-bundle.md`.

use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process;

/// Pre-populated ignore list. `.*` covers `.git`, `.venv`, and every other
/// dot-entry; `--include` cancels any of these.
const DEFAULT_IGNORES: [&str; 6] = [
    ".*",
    "target",
    "node_modules",
    "dist",
    "build",
    "__pycache__",
];

const HELP: &str = "\
mdeezl — feed any repository to text-only agents

USAGE:
    mdeezl [PATH] [-o FILE] [--wrap MODE] [--exclude PATTERN]... [--include PATTERN]...

PATH defaults to the current directory. Output goes to stdout unless -o is given.

OPTIONS:
    -o, --output FILE     write the document to FILE instead of stdout
        --wrap MODE       fence (default) | inline | none
        --exclude PATTERN add PATTERN to the ignore list (repeatable)
        --include PATTERN keep entries matching PATTERN even if ignored (repeatable)
    -h, --help            print this help

WRAP MODES:
    fence   tree in a fenced block; each file body in a fence long enough to
            survive any backtick run inside it, with a language hint
    inline  each tree line wrapped in single backticks. A multi-line file body
            cannot be inline, so bodies are still fenced in this mode.
    none    no fences and no backticks anywhere

IGNORE LIST:
    Pre-populated with: .* target node_modules dist build __pycache__
    An entry is omitted when it matches an ignore pattern and matches no
    include pattern -- include always wins.

PATTERN FORMS:
    .*            any name beginning with a dot (hidden entries, including .git)
    *.ext         every file of that type, e.g. *.png
    has/a/slash   that exact path, relative to PATH -- one specific file
    anything else that exact name, file or directory, at any depth

    Note: in Bash, quote a pattern containing * or the shell expands it before
    mdeezl sees it: --exclude '*.png'. PowerShell passes it through unchanged.
";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Wrap {
    Fence,
    Inline,
    None,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Sink {
    Stdout,
    File(PathBuf),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Options {
    root: PathBuf,
    sink: Sink,
    wrap: Wrap,
    ignore: Vec<String>,
    include: Vec<String>,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            root: PathBuf::from("."),
            sink: Sink::Stdout,
            wrap: Wrap::Fence,
            ignore: DEFAULT_IGNORES.iter().map(|s| s.to_string()).collect(),
            include: Vec::new(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Parsed {
    Help,
    Run(Options),
}

/// Parse command-line arguments. Takes an iterator rather than reading the
/// process environment, so tests drive it directly.
fn parse_args<I: IntoIterator<Item = String>>(args: I) -> Result<Parsed, String> {
    let mut opts = Options::default();
    let mut root_seen = false;
    let mut args = args.into_iter();

    while let Some(arg) = args.next() {
        let mut value = |flag: &str| -> Result<String, String> {
            args.next()
                .ok_or_else(|| format!("{flag} requires a value"))
        };
        match arg.as_str() {
            "-h" | "--help" => return Ok(Parsed::Help),
            "-o" | "--output" => opts.sink = Sink::File(PathBuf::from(value(&arg)?)),
            "--wrap" => {
                let mode = value("--wrap")?;
                opts.wrap = match mode.as_str() {
                    "fence" => Wrap::Fence,
                    "inline" => Wrap::Inline,
                    "none" => Wrap::None,
                    other => {
                        return Err(format!(
                            "unknown wrap mode `{other}` (expected fence, inline, or none)"
                        ));
                    }
                };
            }
            "--exclude" => opts.ignore.push(value("--exclude")?),
            "--include" => opts.include.push(value("--include")?),
            other if root_seen => {
                return Err(format!("unexpected second path `{other}`"));
            }
            other => {
                opts.root = PathBuf::from(other);
                root_seen = true;
            }
        }
    }

    Ok(Parsed::Run(opts))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Dir,
    File,
    Link,
}

/// One entry in the scanned tree. Both renderers read this same structure, so
/// the scaffold and the content dump can never describe different file sets.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Node {
    name: String,
    /// Path relative to the scan root, always `/`-separated.
    rel: String,
    kind: Kind,
    /// Set when this directory could not be listed. The walk continues.
    unreadable: bool,
    children: Vec<Node>,
}

/// Does `pattern` select this entry? Four forms, no glob engine:
/// `.*` (any dot-entry), `*.ext` (a whole file type), a pattern containing `/`
/// (one exact relative path), anything else (that exact name, at any depth).
fn matches(pattern: &str, name: &str, rel: &str) -> bool {
    if pattern == ".*" {
        name.starts_with('.')
    } else if pattern.starts_with("*.") {
        name.ends_with(&pattern[1..])
    } else if pattern.contains('/') {
        rel == pattern
    } else {
        name == pattern
    }
}

/// An entry is omitted when it matches an ignore pattern and matches no
/// include pattern. Include always wins.
fn is_excluded(opts: &Options, name: &str, rel: &str) -> bool {
    opts.ignore.iter().any(|p| matches(p, name, rel))
        && !opts.include.iter().any(|p| matches(p, name, rel))
}

fn join_rel(parent: &str, name: &str) -> String {
    if parent.is_empty() {
        name.to_string()
    } else {
        format!("{parent}/{name}")
    }
}

/// Recursively collect `dir`'s children. A directory that cannot be listed is
/// marked and traversal continues; symlinks are recorded but never followed.
fn walk_children(dir: &Path, rel: &str, opts: &Options) -> (Vec<Node>, bool) {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return (Vec::new(), true),
    };

    let mut nodes: Vec<Node> = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let rel_path = join_rel(rel, &name);
        if is_excluded(opts, &name, &rel_path) {
            continue;
        }
        // file_type() does not follow symlinks; Path::is_dir() would.
        let kind = match entry.file_type() {
            Ok(t) if t.is_symlink() => Kind::Link,
            Ok(t) if t.is_dir() => Kind::Dir,
            Ok(_) => Kind::File,
            Err(_) => Kind::File,
        };
        let (children, unreadable) = if kind == Kind::Dir {
            walk_children(&entry.path(), &rel_path, opts)
        } else {
            (Vec::new(), false)
        };
        nodes.push(Node {
            name,
            rel: rel_path,
            kind,
            unreadable,
            children,
        });
    }

    // read_dir yields entries in unspecified order; sort for byte-identical
    // output across runs and platforms.
    nodes.sort_by(|a, b| a.name.cmp(&b.name));
    (nodes, false)
}

fn walk(opts: &Options) -> io::Result<Node> {
    let meta = fs::metadata(&opts.root)?;
    if !meta.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{} is not a directory", opts.root.display()),
        ));
    }
    let name = opts
        .root
        .canonicalize()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        .unwrap_or_else(|| opts.root.to_string_lossy().into_owned());
    let (children, unreadable) = walk_children(&opts.root, "", opts);
    Ok(Node {
        name,
        rel: String::new(),
        kind: Kind::Dir,
        unreadable,
        children,
    })
}

/// Box-drawing symbols, per `Scaffolding symbols generator.md`: U+251C/U+2500,
/// U+2514/U+2500, U+2502, each followed by a single space before the name.
const BRANCH: &str = "├── ";
const LAST_BRANCH: &str = "└── ";
const CONTINUE: &str = "│   ";
const BLANK: &str = "    ";

fn display_name(node: &Node) -> String {
    let mut s = node.name.clone();
    if node.kind == Kind::Dir {
        s.push('/');
    }
    if node.unreadable {
        s.push_str("  [unreadable]");
    }
    s
}

fn push_tree_lines(node: &Node, prefix: &str, out: &mut Vec<String>) {
    let last = node.children.len().saturating_sub(1);
    for (i, child) in node.children.iter().enumerate() {
        let is_last = i == last;
        out.push(format!(
            "{prefix}{}{}",
            if is_last { LAST_BRANCH } else { BRANCH },
            display_name(child)
        ));
        if !child.children.is_empty() {
            let child_prefix = format!("{prefix}{}", if is_last { BLANK } else { CONTINUE });
            push_tree_lines(child, &child_prefix, out);
        }
    }
}

/// Render the scaffold. `fence` puts the whole tree in one fenced block,
/// `inline` wraps each line in single backticks, `none` emits it bare.
fn render_tree(root: &Node, wrap: Wrap) -> String {
    let mut lines = vec![display_name(root)];
    push_tree_lines(root, "", &mut lines);

    match wrap {
        Wrap::Fence => format!("```\n{}\n```\n", lines.join("\n")),
        Wrap::Inline => {
            let wrapped: Vec<String> = lines.iter().map(|l| format!("`{l}`")).collect();
            format!("{}\n", wrapped.join("\n"))
        }
        Wrap::None => format!("{}\n", lines.join("\n")),
    }
}

fn run(opts: &Options) -> io::Result<()> {
    let root = walk(opts)?;
    let _tree = render_tree(&root, opts.wrap);
    Ok(())
}

fn main() {
    let code = match parse_args(env::args().skip(1)) {
        Ok(Parsed::Help) => {
            print!("{HELP}");
            0
        }
        Ok(Parsed::Run(opts)) => match run(&opts) {
            Ok(()) => 0,
            Err(err) => {
                eprintln!("mdeezl: {err}");
                1
            }
        },
        Err(err) => {
            eprintln!("mdeezl: {err}\n\n{HELP}");
            2
        }
    };
    process::exit(code);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    fn run_opts(items: &[&str]) -> Options {
        match parse_args(args(items)) {
            Ok(Parsed::Run(o)) => o,
            other => panic!("expected Run, got {other:?}"),
        }
    }

    fn err(items: &[&str]) -> String {
        match parse_args(args(items)) {
            Err(e) => e,
            other => panic!("expected usage error, got {other:?}"),
        }
    }

    #[test]
    fn test_args_defaults() {
        let o = run_opts(&[]);
        assert_eq!(o.root, PathBuf::from("."));
        assert_eq!(o.sink, Sink::Stdout);
        assert_eq!(o.wrap, Wrap::Fence);
        assert_eq!(
            o.ignore,
            vec![
                ".*",
                "target",
                "node_modules",
                "dist",
                "build",
                "__pycache__"
            ]
        );
        assert!(o.include.is_empty());
    }

    #[test]
    fn test_args_positional_root() {
        assert_eq!(run_opts(&["some/dir"]).root, PathBuf::from("some/dir"));
    }

    #[test]
    fn test_args_accumulates_exclude_and_include() {
        let o = run_opts(&[
            "--exclude",
            "*.png",
            "--include",
            ".github",
            "--exclude",
            "vendor",
        ]);
        // Appended in declaration order, after the pre-populated entries.
        assert_eq!(&o.ignore[6..], &["*.png".to_string(), "vendor".to_string()]);
        assert_eq!(o.include, vec![".github".to_string()]);
    }

    #[test]
    fn test_args_rejects_unknown_wrap() {
        assert!(err(&["--wrap", "banana"]).contains("banana"));
    }

    #[test]
    fn test_args_rejects_missing_flag_value() {
        for flag in ["-o", "--wrap", "--exclude", "--include"] {
            assert!(
                err(&[flag]).contains(flag),
                "error for {flag} should name the flag"
            );
        }
    }

    #[test]
    fn test_args_rejects_second_positional() {
        assert!(err(&["a", "b"]).contains('b'));
    }

    #[test]
    fn test_args_help() {
        assert_eq!(parse_args(args(&["--help"])), Ok(Parsed::Help));
        assert_eq!(parse_args(args(&["-h"])), Ok(Parsed::Help));
    }

    /// INT-0001 requires an empty `[dependencies]` table. Asserted as "no
    /// entry lines before the next table header" so the manifest stays free to
    /// order its tables however it likes.
    #[test]
    fn test_manifest_dependencies_table_is_empty() {
        let manifest = include_str!("../Cargo.toml");
        let mut in_deps = false;
        for line in manifest.lines() {
            let line = line.trim();
            if line.starts_with('[') {
                in_deps = line == "[dependencies]";
                continue;
            }
            if in_deps && !line.is_empty() && !line.starts_with('#') {
                panic!("[dependencies] must stay empty, found: {line}");
            }
        }
    }

    // ---- T-002: walk, pattern matching, ignore/include resolution ----

    fn tmp_dir(tag: &str) -> PathBuf {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static N: AtomicUsize = AtomicUsize::new(0);
        let n = N.fetch_add(1, Ordering::Relaxed);
        let dir = env::temp_dir().join(format!("mdeezl-{}-{tag}-{n}", process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn opts_for(root: &Path, ignore: &[&str], include: &[&str]) -> Options {
        Options {
            root: root.to_path_buf(),
            ignore: ignore.iter().map(|s| s.to_string()).collect(),
            include: include.iter().map(|s| s.to_string()).collect(),
            ..Options::default()
        }
    }

    fn names(node: &Node) -> Vec<&str> {
        node.children.iter().map(|c| c.name.as_str()).collect()
    }

    #[test]
    fn test_pattern_dot_star() {
        assert!(matches(".*", ".git", ".git"));
        assert!(matches(".*", ".env", "a/.env"));
        assert!(!matches(".*", "src", "src"));
    }

    #[test]
    fn test_pattern_extension() {
        assert!(matches("*.png", "logo.png", "img/logo.png"));
        assert!(!matches("*.png", "png", "png"));
        assert!(!matches("*.png", "logo.pngx", "logo.pngx"));
        assert!(!matches("*.png", "assets", "assets"));
    }

    #[test]
    fn test_pattern_relative_path() {
        assert!(matches("docs/big.csv", "big.csv", "docs/big.csv"));
        assert!(!matches("docs/big.csv", "big.csv", "other/big.csv"));
    }

    #[test]
    fn test_pattern_exact_name() {
        assert!(matches("target", "target", "target"));
        assert!(!matches("target", "targets", "targets"));
        assert!(!matches("target", "my_target", "my_target"));
    }

    #[test]
    fn test_pattern_name_matches_at_depth() {
        assert!(matches("node_modules", "node_modules", "a/b/node_modules"));
    }

    #[test]
    fn test_include_outranks_ignore() {
        let o = opts_for(Path::new("."), &[".*"], &[".github"]);
        assert!(!is_excluded(&o, ".github", ".github"));
        assert!(is_excluded(&o, ".env", ".env"));
    }

    #[test]
    fn test_include_dot_star_readmits_git() {
        // INT-0001 records this as a consequence: include outranks ignore, so
        // asking for dot-entries brings .git back too.
        let o = opts_for(Path::new("."), &[".*"], &[".*"]);
        assert!(!is_excluded(&o, ".git", ".git"));
    }

    #[test]
    fn test_walk_sorts_children() {
        let dir = tmp_dir("sort");
        for name in ["zebra.txt", "alpha.txt", "middle.txt"] {
            fs::write(dir.join(name), "x").unwrap();
        }
        let root = walk(&opts_for(&dir, &[], &[])).unwrap();
        assert_eq!(names(&root), ["alpha.txt", "middle.txt", "zebra.txt"]);
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_walk_does_not_follow_symlink() {
        let dir = tmp_dir("symlink");
        fs::create_dir(dir.join("sub")).unwrap();
        fs::write(dir.join("sub/file.txt"), "x").unwrap();

        #[cfg(unix)]
        let made = std::os::unix::fs::symlink(&dir, dir.join("loop")).is_ok();
        #[cfg(windows)]
        let made = std::os::windows::fs::symlink_dir(&dir, dir.join("loop")).is_ok();

        if !made {
            eprintln!(
                "SKIP test_walk_does_not_follow_symlink: this platform refuses to \
                 create a directory symlink without elevated privileges. The Linux \
                 leg of the CI matrix is the authoritative run."
            );
            let _ = fs::remove_dir_all(&dir);
            return;
        }

        // Terminates at all => the cycle was not followed.
        let root = walk(&opts_for(&dir, &[], &[])).unwrap();
        let link = root.children.iter().find(|c| c.name == "loop").unwrap();
        assert_eq!(link.kind, Kind::Link);
        assert!(link.children.is_empty(), "a symlink must not be descended");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_unreadable_dir_marked_and_walk_continues() {
        let dir = tmp_dir("unreadable");
        fs::create_dir(dir.join("locked")).unwrap();
        fs::write(dir.join("locked/hidden.txt"), "x").unwrap();
        fs::write(dir.join("sibling.txt"), "x").unwrap();

        #[cfg(unix)]
        let blocked = {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(dir.join("locked"), fs::Permissions::from_mode(0o000)).is_ok()
        };
        // Windows set_permissions only toggles the read-only attribute, which
        // does not block read_dir, and std exposes no ACL API.
        #[cfg(not(unix))]
        let blocked = false;

        if !blocked {
            eprintln!(
                "SKIP test_unreadable_dir_marked_and_walk_continues: this platform \
                 cannot make a directory unlistable through std alone. The Linux leg \
                 of the CI matrix is the authoritative run."
            );
            let _ = fs::remove_dir_all(&dir);
            return;
        }

        let root = walk(&opts_for(&dir, &[], &[])).unwrap();
        let locked = root.children.iter().find(|c| c.name == "locked").unwrap();
        assert!(locked.unreadable, "unlistable directory must be marked");
        assert!(
            root.children.iter().any(|c| c.name == "sibling.txt"),
            "the walk must continue past an unlistable directory"
        );

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(dir.join("locked"), fs::Permissions::from_mode(0o755));
        }
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_walk_rejects_missing_root() {
        let missing = env::temp_dir().join("mdeezl-does-not-exist-xyzzy");
        assert!(walk(&opts_for(&missing, &[], &[])).is_err());
    }

    // ---- T-003: scaffold tree renderer ----

    fn dir_node(name: &str, children: Vec<Node>) -> Node {
        Node {
            name: name.to_string(),
            rel: name.to_string(),
            kind: Kind::Dir,
            unreadable: false,
            children,
        }
    }

    fn file_node(name: &str) -> Node {
        Node {
            name: name.to_string(),
            rel: name.to_string(),
            kind: Kind::File,
            unreadable: false,
            children: Vec::new(),
        }
    }

    #[test]
    fn test_tree_branch_symbols() {
        let root = dir_node("root", vec![file_node("a.txt"), file_node("b.txt")]);
        let out = render_tree(&root, Wrap::None);
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines[1], "├── a.txt");
        assert_eq!(lines[2], "└── b.txt");
    }

    #[test]
    fn test_tree_prefix_composition() {
        let root = dir_node(
            "root",
            vec![
                dir_node("first", vec![file_node("deep.txt")]),
                dir_node("last", vec![file_node("deep.txt")]),
            ],
        );
        let out = render_tree(&root, Wrap::None);
        let lines: Vec<&str> = out.lines().collect();
        // Descending past a non-last entry continues the vertical line...
        assert_eq!(lines[2], "│   └── deep.txt");
        // ...and past a last entry it becomes four spaces.
        assert_eq!(lines[4], "    └── deep.txt");
    }

    #[test]
    fn test_tree_directory_suffix() {
        let root = dir_node("root", vec![dir_node("sub", vec![]), file_node("f.txt")]);
        let out = render_tree(&root, Wrap::None);
        assert!(out.contains("├── sub/"));
        assert!(out.contains("└── f.txt"));
        assert!(!out.contains("f.txt/"));
    }

    #[test]
    fn test_tree_unreadable_marker() {
        let mut locked = dir_node("locked", vec![]);
        locked.unreadable = true;
        let root = dir_node("root", vec![locked]);
        assert!(render_tree(&root, Wrap::None).contains("└── locked/  [unreadable]"));
    }

    #[test]
    fn test_tree_fence_wrap() {
        let root = dir_node("root", vec![file_node("a.txt")]);
        let out = render_tree(&root, Wrap::Fence);
        assert_eq!(out.matches("```").count(), 2, "exactly one fenced block");
        assert!(out.starts_with("```\n"));
        assert!(out.ends_with("```\n"));
    }

    #[test]
    fn test_tree_inline_wrap() {
        let root = dir_node("root", vec![file_node("a.txt")]);
        let out = render_tree(&root, Wrap::Inline);
        for line in out.lines() {
            assert!(line.starts_with('`') && line.ends_with('`'), "line: {line}");
        }
        // Symbols survive the wrapping untouched.
        assert!(out.contains("`└── a.txt`"));
    }

    #[test]
    fn test_tree_none_wrap() {
        let root = dir_node("root", vec![file_node("a.txt")]);
        let out = render_tree(&root, Wrap::None);
        assert!(!out.contains('`'), "none mode emits no backticks at all");
    }

    #[test]
    fn test_manifest_declares_bin_name_and_license() {
        let manifest = include_str!("../Cargo.toml");
        // The bin name is load-bearing: CARGO_BIN_EXE_mdeezl resolves by it.
        assert!(manifest.contains("name = \"mdeezl\""));
        assert!(manifest.contains("license = \"Apache-2.0\""));
    }
}
