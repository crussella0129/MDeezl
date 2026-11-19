//! MDeezl — feed any repository to text-only agents.
//!
//! Emits one Markdown document for a directory: a box-drawing scaffold tree,
//! then the contents of every included file. Standard library only; see
//! `docs/intents/INT-0001-markdown-repo-context-bundle.md`.
//!
//! The engine lives here so the CLI (`src/main.rs`) and the desktop app
//! (`gui/`, INT-0006) call the same code: [`bundle`] builds a document,
//! [`cli`] is the whole command line.

use std::collections::HashSet;
use std::env;
use std::fs;
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::{self, Command, Stdio};

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
    mdeezl [PATH | URL] [-o FILE] [--wrap MODE] [--exclude PATTERN]... [--include PATTERN]...

PATH defaults to the current directory. Output goes to stdout unless -o is given.

URL SOURCES:
    A URL beginning https://, http:// or file:// is fetched instead of read
    from disk. This needs git: mdeezl makes a shallow clone into a temporary
    folder, bundles it, and the folder is removed afterwards. The document's
    title is the URL. mdeezl never prompts for credentials and uses no
    credential helper, so a private repository fails instead of asking you to
    log in.

OPTIONS:
    -o, --output FILE     write the document to FILE instead of stdout
        --wrap MODE       fence (default) | inline | none
        --exclude PATTERN add PATTERN to the ignore list (repeatable)
        --include PATTERN keep entries matching PATTERN even if ignored (repeatable)
        --no-gitignore    do not apply the repository's .gitignore rules
        --                every later argument is a path, for a PATH that
                          itself begins with -
    -h, --help            print this help

WRAP MODES:
    fence   tree in a fenced block; each file body in a fence long enough to
            survive any backtick run inside it, with a language hint
    inline  each tree line wrapped in single backticks. A multi-line file body
            cannot be inline, so bodies are still fenced in this mode.
    none    no fences and no backticks anywhere

WHAT IS OMITTED:
    Entries are omitted from two sources, and --include outranks both.

    1. The ignore list, pre-populated with:
           .* target node_modules dist build __pycache__
       An entry is omitted when it matches an ignore pattern and matches no
       include pattern -- include always wins.

    2. The repository's own .gitignore rules, applied by asking git itself, so
       the full gitignore syntax works. Files the repository tracks are kept
       even when a rule matches them. This needs git on your PATH, and the
       scanned directory must be inside a git work tree; otherwise mdeezl says
       so on stderr and uses the ignore list alone. Turn it off with
       --no-gitignore.

PATTERN FORMS:
    .*            any name beginning with a dot (hidden entries, including .git)
    *.ext         every file of that type, e.g. *.png
    has/a/slash   that exact path, relative to PATH -- one specific file
    anything else that exact name, file or directory, at any depth

    Note: in Bash, quote a pattern containing * or the shell expands it before
    mdeezl sees it: --exclude '*.png'. PowerShell passes it through unchanged.
";

/// How the tree and the file bodies are wrapped: `--wrap` in the help text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Wrap {
    Fence,
    Inline,
    None,
}

/// Where [`cli`] writes the document. [`bundle`] ignores it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Sink {
    Stdout,
    File(PathBuf),
}

/// Everything one run needs. [`Options::default`] is the CLI with no flags.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    pub root: PathBuf,
    pub sink: Sink,
    pub wrap: Wrap,
    pub ignore: Vec<String>,
    pub include: Vec<String>,
    /// Apply the repository's own `.gitignore` rules. See INT-0004.
    pub use_gitignore: bool,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            root: PathBuf::from("."),
            sink: Sink::Stdout,
            wrap: Wrap::Fence,
            ignore: DEFAULT_IGNORES.iter().map(|s| s.to_string()).collect(),
            include: Vec::new(),
            use_gitignore: true,
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
    let mut end_of_options = false;
    let mut args = args.into_iter();

    while let Some(arg) = args.next() {
        // After `--`, everything is a path. Without it, a directory whose
        // name begins with `-` could not be bundled at all once unknown
        // options began to be rejected.
        if end_of_options {
            if root_seen {
                return Err(format!("unexpected second path `{arg}`"));
            }
            opts.root = PathBuf::from(arg);
            root_seen = true;
            continue;
        }
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
            "--no-gitignore" => opts.use_gitignore = false,
            "--" => end_of_options = true,
            // Anything else beginning with `-` is a mistyped option, not a
            // path: saying "unexpected second path `--nope`" sent the reader
            // looking for a path they never typed.
            other if other.starts_with('-') && other != "-" => {
                return Err(format!("unknown option `{other}`"));
            }
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
    // Name the path: "The system cannot find the file specified" alone left
    // the reader guessing which path mdeezl had tried.
    let meta = fs::metadata(&opts.root)
        .map_err(|e| io::Error::new(e.kind(), format!("{}: {e}", opts.root.display())))?;
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

/// CommonMark requires a closing fence at least as long as the opening one, so
/// an opening fence longer than any backtick run inside the body cannot be
/// closed from within it.
fn fence_len(body: &str) -> usize {
    let mut longest = 0;
    let mut run = 0;
    for ch in body.chars() {
        if ch == '`' {
            run += 1;
            longest = longest.max(run);
        } else {
            run = 0;
        }
    }
    (longest + 1).max(3)
}

/// A deliberately small table. An unknown extension is not an error.
fn language_hint(name: &str) -> &'static str {
    match name.rsplit_once('.').map(|(_, ext)| ext) {
        Some("rs") => "rust",
        Some("md") => "markdown",
        Some("toml") => "toml",
        Some("json") => "json",
        Some("yml" | "yaml") => "yaml",
        Some("py") => "python",
        Some("go") => "go",
        Some("js") => "javascript",
        Some("ts") => "typescript",
        Some("sh" | "bash") => "bash",
        Some("ps1") => "powershell",
        Some("html") => "html",
        Some("css") => "css",
        _ => "",
    }
}

/// Collect every file node in tree order.
fn collect_files<'a>(node: &'a Node, out: &mut Vec<&'a Node>) {
    for child in &node.children {
        match child.kind {
            Kind::File | Kind::Link => out.push(child),
            Kind::Dir => collect_files(child, out),
        }
    }
}

/// Render one file section: the `---` / `File: <path>` / `---` header the awk
/// one-liner emits, then the body. `body` is `Err` when the file could not be
/// read, which is rendered in place rather than aborting the document.
fn render_section(node: &Node, body: &Result<Vec<u8>, io::Error>, wrap: Wrap) -> String {
    let mut out = format!("\n---\nFile: {}\n---\n\n", node.rel);

    let text = match body {
        Err(err) => {
            out.push_str(&format!("[unreadable: {err}]\n"));
            return out;
        }
        Ok(bytes) => match std::str::from_utf8(bytes) {
            Ok(text) => text,
            Err(_) => {
                out.push_str(&format!("[binary file, {} bytes elided]\n", bytes.len()));
                return out;
            }
        },
    };

    match wrap {
        // A multi-line body cannot be inline, so inline falls back to fencing.
        Wrap::Fence | Wrap::Inline => {
            let fence = "`".repeat(fence_len(text));
            out.push_str(&format!("{fence}{}\n", language_hint(&node.name)));
            out.push_str(text);
            if !text.ends_with('\n') {
                out.push('\n');
            }
            out.push_str(&format!("{fence}\n"));
        }
        Wrap::None => {
            out.push_str(text);
            if !text.ends_with('\n') {
                out.push('\n');
            }
        }
    }
    out
}

fn render_contents(root: &Node, opts: &Options) -> String {
    let mut files = Vec::new();
    collect_files(root, &mut files);

    let mut out = String::new();
    for node in files {
        let body = fs::read(opts.root.join(&node.rel));
        out.push_str(&render_section(node, &body, opts.wrap));
    }
    out
}

fn render_document(root: &Node, opts: &Options) -> String {
    render_titled(&root.name, root, opts)
}

/// The document under an explicit `# title`, which is the root's name except
/// for a URL source (INT-0002).
fn render_titled(title: &str, root: &Node, opts: &Options) -> String {
    format!(
        "# {title}\n\n## Structure\n\n{}\n## Contents\n{}",
        render_tree(root, opts.wrap),
        render_contents(root, opts)
    )
}

// ---- The repository's own .gitignore, by asking git. See INT-0004. ----

/// Every variable `git rev-parse --local-env-vars` lists (git 2.54). Removed
/// from git's environment so an ambient repository -- as when mdeezl runs from
/// inside a git hook -- cannot redirect the query away from the scan root.
/// Hard-coded rather than queried, because querying would cost a second process.
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

#[cfg(test)]
thread_local! {
    /// Counts git spawns, so "one subprocess per run" is measured, not inferred.
    static GIT_SPAWNS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// Every git process starts here: the repository-local variables removed, and
/// on Windows no console window, which a windowed caller would otherwise flash
/// for each spawn. Under test, git is also isolated from the host's
/// configuration and from any repository above the temp directory, as the
/// end-to-end harness does -- set on the child, so no process-wide `set_var`.
fn git_command() -> Command {
    let mut cmd = Command::new("git");
    for var in GIT_LOCAL_ENV_VARS {
        cmd.env_remove(var);
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    #[cfg(test)]
    {
        let iso = env::temp_dir().join(format!("mdeezl-gitiso-{}", process::id()));
        let _ = fs::create_dir_all(&iso);
        let _ = fs::write(iso.join("gitconfig"), "");
        cmd.env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", iso.join("gitconfig"))
            .env("XDG_CONFIG_HOME", &iso)
            .env("GIT_CEILING_DIRECTORIES", env::temp_dir());
    }
    cmd
}

// ---- URL sources: a shallow clone into a temporary folder. See INT-0002. ----

/// True when `s` names a remote to fetch rather than a local path. Checked on
/// the text, never on path components: git itself reads `ftp:/weird` as ssh.
pub fn is_url(s: &str) -> bool {
    ["https://", "http://", "file://"]
        .iter()
        .any(|p| s.starts_with(p))
}

/// Why `source` cannot be used, when it looks like a URL but is not one
/// mdeezl fetches. Without this, `ssh://host/repo` fell through to the file
/// system and failed with "The filename, directory name, or volume label
/// syntax is incorrect", and `https://` spent seven seconds in DNS.
fn unsupported_source(source: &str) -> Option<String> {
    let supported = "mdeezl fetches https://, http:// and file:// URLs";
    if let Some((scheme, rest)) = source.split_once("://") {
        if !is_url(source) {
            return Some(format!(
                "`{scheme}://` sources are not supported: {supported}"
            ));
        }
        // A missing host is a typo, not a name to look up.
        let authority = &rest[..rest.find('/').unwrap_or(rest.len())];
        if authority.is_empty() && !source.starts_with("file://") {
            return Some(format!("{source} has no host: {supported}"));
        }
        return None;
    }
    // git's own scp-like form, `user@host:path`, which is ssh under another
    // spelling. A Windows path such as `C:\repo` has no `@` before its colon.
    let head = &source[..source.find('/').unwrap_or(source.len())];
    match (head.find('@'), head.find(':')) {
        (Some(at), Some(colon)) if at < colon => Some(format!(
            "`{source}` is an ssh address, which is not supported: {supported}"
        )),
        _ => None,
    }
}

/// `url` as shown to a person -- heading, stderr, the GUI -- with any
/// `user[:password]@` removed, so a token never lands in a pasted document.
fn display_url(url: &str) -> String {
    let Some((scheme, rest)) = url.split_once("://") else {
        return url.to_string();
    };
    let (authority, path) = rest.split_at(rest.find('/').unwrap_or(rest.len()));
    match authority.rfind('@') {
        Some(at) => format!("{scheme}://{}{path}", &authority[at + 1..]),
        None => url.to_string(),
    }
}

/// The folder name for a clone: the URL path's last segment without `.git`.
/// It becomes a path component, so anything that could leave its folder or
/// is not a plain name falls back to `repository`.
fn repo_name(url: &str) -> String {
    let rest = url.split_once("://").map_or(url, |(_, r)| r);
    let path = rest.find('/').map_or("", |i| &rest[i..]);
    let path = path.split(['?', '#']).next().unwrap_or("");
    let segment = path.split('/').rfind(|s| !s.is_empty()).unwrap_or("");
    let name = segment.strip_suffix(".git").unwrap_or(segment);
    if name.is_empty() || name == "." || name == ".." || name.contains(['\\', ':', '?', '#']) {
        "repository".to_string()
    } else {
        name.to_string()
    }
}

/// Seconds a transfer may move under 1 byte/s before git gives up. git has no
/// stall timeout of its own: a silent server would otherwise hang the fetch.
const LOW_SPEED_SECS: u32 = 30;

#[cfg(test)]
thread_local! {
    /// Lowered by the stalled-server test, on its own thread only.
    static LOW_SPEED_OVERRIDE: std::cell::Cell<Option<u32>> = const { std::cell::Cell::new(None) };
}

fn low_speed_secs() -> u32 {
    #[cfg(test)]
    if let Some(secs) = LOW_SPEED_OVERRIDE.with(|c| c.get()) {
        return secs;
    }
    LOW_SPEED_SECS
}

/// The clone, hardened so it can never wait on a person: every prompt route
/// off (terminal, askpass, credential helpers), ssh refused even when an
/// `insteadOf` rewrites to it, stalls bounded, LF text on every OS, and LFS
/// objects left as pointers. `--` keeps a URL from being read as an option.
fn clone_command(url: &str, dest: &Path) -> Command {
    let mut cmd = git_command();
    cmd.args(["-c", "credential.helper=", "-c", "core.askPass="])
        .args(["-c", "core.autocrlf=false", "-c", "core.eol=lf"])
        .args(["-c", "http.lowSpeedLimit=1", "-c"])
        .arg(format!("http.lowSpeedTime={}", low_speed_secs()))
        .args(["clone", "--depth", "1", "--quiet", "--", url])
        .arg(dest)
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GCM_INTERACTIVE", "never")
        .env("GIT_ALLOW_PROTOCOL", "https:http:file")
        .env("GIT_LFS_SKIP_SMUDGE", "1")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    // git lets these override the `-c` settings above.
    for var in [
        "GIT_ASKPASS",
        "SSH_ASKPASS",
        "GIT_HTTP_LOW_SPEED_LIMIT",
        "GIT_HTTP_LOW_SPEED_TIME",
    ] {
        cmd.env_remove(var);
    }
    cmd
}

/// A fetched copy of a remote. Dropping it removes its temporary folder,
/// whether the bundle that used it succeeded or not.
struct Fetched {
    base: PathBuf,
    root: PathBuf,
}

impl Drop for Fetched {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.base);
    }
}

/// Clone `url` into `<temp>/mdeezl-<pid>-<n>/<repo-name>`. On failure the
/// reason is git's first `fatal:` line -- its last is generic advice.
fn fetch(url: &str, temp: &Path) -> io::Result<Fetched> {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static N: AtomicUsize = AtomicUsize::new(0);
    let shown = display_url(url);
    let fail = |reason: &str| {
        io::Error::other(format!(
            "could not fetch {shown}: {}",
            reason.replace(url, &shown)
        ))
    };

    let base = loop {
        let n = N.fetch_add(1, Ordering::Relaxed);
        let dir = temp.join(format!("mdeezl-{}-{n}", process::id()));
        match fs::create_dir(&dir) {
            Ok(()) => break dir,
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(fail(&format!("cannot create a temporary folder: {e}"))),
        }
    };
    let fetched = Fetched {
        root: base.join(repo_name(url)),
        base,
    };
    let output = clone_command(url, &fetched.root)
        .output()
        .map_err(|e| match e.kind() {
            io::ErrorKind::NotFound => fail("git was not found on PATH"),
            _ => fail(&format!("could not run git: {e}")),
        })?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let lines = || stderr.lines().map(str::trim);
        let reason = lines()
            .find_map(|l| l.strip_prefix("fatal: "))
            .or_else(|| lines().rfind(|l| !l.is_empty()))
            .map(str::to_string)
            .unwrap_or_else(|| format!("git clone exited with {}", output.status));
        return Err(fail(&reason));
    }
    Ok(fetched)
}

/// Each path `./`-prefixed, so a leading `:` is not read as pathspec magic, and
/// NUL-terminated. Glob characters are wrapped in bracket classes so git's
/// tracked-file lookup treats the name literally. Never backslash escaping: Git
/// for Windows turns `\` into `/`, which reported tracked files as ignored.
fn nul_payload(paths: &[String]) -> Vec<u8> {
    let mut out = Vec::new();
    for path in paths {
        out.extend_from_slice(b"./");
        for &b in path.as_bytes() {
            match b {
                b'[' => out.extend_from_slice(b"[[]"),
                b'*' => out.extend_from_slice(b"[*]"),
                b'?' => out.extend_from_slice(b"[?]"),
                b'\\' => out.extend_from_slice(br"[\\]"),
                _ => out.push(b),
            }
        }
        out.push(0);
    }
    out
}

/// `check-ignore -z -v -n` emits exactly one four-field record per input, in
/// input order: source, line, pattern, path. Results are mapped by position,
/// never by parsing git's echo of the path, which is inconsistent for escaped
/// names. A path is ignored when some rule matched and it was not a negation.
fn parse_records(out: &[u8], n: usize) -> Result<Vec<bool>, String> {
    let body = out.strip_suffix(b"\0").unwrap_or(out);
    let fields: Vec<&[u8]> = if body.is_empty() {
        Vec::new()
    } else {
        body.split(|&b| b == 0).collect()
    };
    if fields.len() != n * 4 {
        return Err(format!(
            "git check-ignore returned {} fields for {n} paths",
            fields.len()
        ));
    }
    Ok(fields
        .chunks(4)
        .map(|r| !r[0].is_empty() && !r[2].starts_with(b"!"))
        .collect())
}

/// `.` for the scan root -- an empty entry aborts the whole query, and git
/// reports `.` exactly when it considers the root itself ignored -- then every
/// entry's path, skipping what lies inside a nested repository or submodule. A
/// path inside a submodule aborts the query, and a nested repository's rules
/// are not the enclosing one's. The scan root itself is not "nested".
fn gitignore_candidates(root: &Path, tree: &Node) -> Vec<String> {
    fn visit(root: &Path, node: &Node, out: &mut Vec<String>) {
        for child in &node.children {
            out.push(child.rel.clone());
            if child.kind == Kind::Dir && !root.join(&child.rel).join(".git").exists() {
                visit(root, child, out);
            }
        }
    }
    let mut out = vec![".".to_string()];
    visit(root, tree, &mut out);
    out
}

/// Ask git, once, which of `paths` the repository ignores. `Err` carries the
/// reason, for the degradation notice.
fn git_ignored(root: &Path, paths: &[String]) -> Result<HashSet<String>, String> {
    #[cfg(test)]
    GIT_SPAWNS.with(|n| n.set(n.get() + 1));

    let mut cmd = git_command();
    cmd.arg("-C")
        .arg(root)
        .args(["check-ignore", "-z", "-v", "-n", "--stdin"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = cmd.spawn().map_err(|e| match e.kind() {
        io::ErrorKind::NotFound => "git was not found on PATH".to_string(),
        _ => format!("could not run git: {e}"),
    })?;

    // Write from another thread. A large list fills the pipe, and git stops
    // reading its stdin while its own stdout is full and unread; writing and
    // reading on one thread would deadlock. Write errors are ignored -- git exits
    // before reading stdin outside a work tree -- so its exit status decides.
    let mut stdin = child.stdin.take().expect("stdin is piped");
    let payload = nul_payload(paths);
    let writer = std::thread::spawn(move || {
        let _ = stdin.write_all(&payload);
    });
    let output = child
        .wait_with_output()
        .map_err(|e| format!("git check-ignore failed: {e}"))?;
    let _ = writer.join();

    match output.status.code() {
        Some(0) | Some(1) => {
            let marks = parse_records(&output.stdout, paths.len())?;
            Ok(paths
                .iter()
                .zip(marks)
                .filter(|(_, ignored)| *ignored)
                .map(|(p, _)| p.clone())
                .collect())
        }
        _ => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            match stderr.lines().next().map(str::trim) {
                Some(line) if !line.is_empty() => Err(line.to_string()),
                _ => Err(format!("git check-ignore exited with {}", output.status)),
            }
        }
    }
}

/// Does any entry git was asked about, beneath `node`, come back not ignored?
/// Entries never queried -- inside a nested repository -- do not count.
fn has_unreported_descendant(
    node: &Node,
    ignored: &HashSet<String>,
    queried: &HashSet<String>,
) -> bool {
    node.children.iter().any(|c| {
        (queried.contains(&c.rel) && !ignored.contains(&c.rel))
            || (c.kind == Kind::Dir && has_unreported_descendant(c, ignored, queried))
    })
}

/// Remove what git reports ignored, top-down, so a pruned directory takes its
/// subtree -- and any include-matching entry inside it -- with it.
/// - An entry git ignores that matches `--include` is kept, and so is its whole
///   subtree: git reports every descendant of an ignored directory, so judging
///   them one by one would restore an empty directory.
/// - An included entry git does not ignore gets no such exemption.
/// - A reported directory holding something git did not report is kept and its
///   children judged individually. For ordinary names git never reports a
///   directory holding tracked files; this catches the escaped names (such as
///   `app/[slug]`) where it can, so a tracked file is never dropped.
fn prune_gitignored(
    node: &mut Node,
    ignored: &HashSet<String>,
    queried: &HashSet<String>,
    include: &[String],
) {
    node.children.retain_mut(|child| {
        if ignored.contains(&child.rel) {
            if include.iter().any(|p| matches(p, &child.name, &child.rel)) {
                return true;
            }
            if child.kind != Kind::Dir || !has_unreported_descendant(child, ignored, queried) {
                return false;
            }
        }
        if child.kind == Kind::Dir {
            prune_gitignored(child, ignored, queried, include);
        }
        true
    });
}

/// One finished document, with what the CLI would have said about it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bundle {
    /// The scanned directory's name: what the GUI names its `<name>.md`.
    pub name: String,
    pub document: String,
    /// Each notice the CLI prints to stderr, without its `mdeezl: ` prefix.
    pub notices: Vec<String>,
}

/// Build the whole document in memory, printing nothing, so every caller
/// decides what reaches the user -- and a failure can never leave a partial
/// document anywhere. INT-0001 records the peak-memory cost this accepts.
/// A URL source (see [`is_url`]) is cloned first and removed afterwards.
pub fn bundle(opts: &Options) -> io::Result<Bundle> {
    bundle_in(opts, &env::temp_dir())
}

/// [`bundle`], with the folder a URL is cloned under made explicit for tests.
fn bundle_in(opts: &Options, temp: &Path) -> io::Result<Bundle> {
    if let Some(why) = opts.root.to_str().and_then(unsupported_source) {
        return Err(io::Error::other(why));
    }
    match opts.root.to_str().filter(|s| is_url(s)) {
        Some(url) => {
            let fetched = fetch(url, temp)?;
            let local = Options {
                root: fetched.root.clone(),
                ..opts.clone()
            };
            bundle_local(&local, Some(&display_url(url)))
        }
        None => bundle_local(opts, None),
    }
}

/// Bundle a directory on disk, titled by `title` or, by default, its name.
fn bundle_local(opts: &Options, title: Option<&str>) -> io::Result<Bundle> {
    let mut root = walk(opts)?;
    let mut notices = Vec::new();
    if opts.use_gitignore {
        let candidates = gitignore_candidates(&opts.root, &root);
        match git_ignored(&opts.root, &candidates) {
            Ok(ignored) if ignored.contains(".") => notices.push(
                "gitignore filtering skipped: the scanned directory is itself ignored by the repository"
                    .to_string(),
            ),
            Ok(ignored) => {
                let queried: HashSet<String> = candidates.into_iter().collect();
                prune_gitignored(&mut root, &ignored, &queried, &opts.include);
            }
            Err(reason) => notices.push(format!("gitignore filtering skipped: {reason}")),
        }
    }
    let document = match title {
        Some(title) => render_titled(title, &root, opts),
        None => render_document(&root, opts),
    };
    Ok(Bundle {
        name: root.name,
        document,
        notices,
    })
}

/// The CLI's run: bundle, report the notices, then write the sink.
fn run(opts: &Options) -> io::Result<()> {
    let bundle = bundle(opts)?;
    for notice in &bundle.notices {
        eprintln!("mdeezl: {notice}");
    }
    // Name the file in any write failure: "Access is denied. (os error 5)"
    // on its own does not say what mdeezl was trying to write.
    let named = |e: io::Error| match &opts.sink {
        Sink::Stdout => e,
        Sink::File(path) => {
            io::Error::new(e.kind(), format!("could not write {}: {e}", path.display()))
        }
    };
    let mut sink: Box<dyn Write> = match &opts.sink {
        Sink::Stdout => Box::new(BufWriter::new(io::stdout().lock())),
        Sink::File(path) => Box::new(BufWriter::new(fs::File::create(path).map_err(named)?)),
    };
    sink.write_all(bundle.document.as_bytes()).map_err(named)?;
    sink.flush().map_err(named)
}

/// The whole command line: `args` are the arguments after the program name.
/// Returns the exit code -- 0, 1 for a run that failed, 2 for bad arguments.
pub fn cli<I: IntoIterator<Item = String>>(args: I) -> i32 {
    match parse_args(args) {
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
    }
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

    // ---- Sprint 1 / T-001: gitignore off switch ----

    #[test]
    fn test_args_gitignore_on_by_default() {
        assert!(run_opts(&[]).use_gitignore);
    }

    #[test]
    fn test_args_no_gitignore() {
        let o = run_opts(&["--no-gitignore"]);
        assert!(!o.use_gitignore);
        // Every other option keeps its default.
        let d = Options::default();
        assert_eq!(o.root, d.root);
        assert_eq!(o.sink, d.sink);
        assert_eq!(o.wrap, d.wrap);
        assert_eq!(o.ignore, d.ignore);
        assert_eq!(o.include, d.include);
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
        // dev- and build-dependencies count too: INT-0001 says the test suite
        // adds none, and a [dev-dependencies] table is the only way it could.
        // Matched by substring rather than an exact list, because Cargo also
        // accepts [dependencies.serde], [dev-dependencies.tempfile] and
        // [target.'cfg(unix)'.dev-dependencies], which exact equality skips.
        let mut in_deps = false;
        for line in manifest.lines() {
            let line = line.trim();
            if line.starts_with('[') {
                in_deps = line.contains("dependencies");
                continue;
            }
            if in_deps && !line.is_empty() && !line.starts_with('#') {
                panic!("dependency tables must stay empty, found: {line}");
            }
        }
    }

    // ---- T-002: walk, pattern matching, ignore/include resolution ----

    /// A temp directory that removes itself even when an assertion panics, so
    /// a failing test cannot strand a tree in the system temp directory.
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
        use std::sync::atomic::{AtomicUsize, Ordering};
        static N: AtomicUsize = AtomicUsize::new(0);
        let n = N.fetch_add(1, Ordering::Relaxed);
        let dir = env::temp_dir().join(format!("mdeezl-{}-{tag}-{n}", process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        Tmp(dir)
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
            let _ = fs::set_permissions(dir.join("locked"), fs::Permissions::from_mode(0o000));
            // Probe the actual condition, not whether the chmod call returned
            // Ok: running as root (the default in many containers) the chmod
            // succeeds and read_dir still works, which would fail this test
            // rather than skip it.
            fs::read_dir(dir.join("locked")).is_err()
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

        // Restore before asserting, so a failing assertion cannot leave a
        // 0o000 directory behind that nothing can subsequently remove.
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(dir.join("locked"), fs::Permissions::from_mode(0o755));
        }

        let locked = root.children.iter().find(|c| c.name == "locked").unwrap();
        assert!(locked.unreadable, "unlistable directory must be marked");
        assert!(
            root.children.iter().any(|c| c.name == "sibling.txt"),
            "the walk must continue past an unlistable directory"
        );
    }

    /// A host-independent proof of the same failure branch: `read_dir` on a
    /// regular file fails on every platform, so this covers "records the
    /// directory as unreadable and does not abort" even where the permission
    /// test above must skip.
    #[test]
    fn test_walk_children_marks_unreadable_path() {
        let dir = tmp_dir("notadir");
        let file = dir.join("regular.txt");
        fs::write(&file, "x").unwrap();

        let (children, unreadable) = walk_children(&file, "regular.txt", &opts_for(&dir, &[], &[]));
        assert!(unreadable, "a path that cannot be listed must be marked");
        assert!(children.is_empty());
    }

    #[test]
    fn test_walk_rejects_missing_root() {
        let missing = env::temp_dir().join("mdeezl-does-not-exist-xyzzy");
        // The test means nothing if something else ever creates this path.
        assert!(
            !missing.exists(),
            "precondition: {missing:?} must not exist"
        );
        assert!(walk(&opts_for(&missing, &[], &[])).is_err());
    }

    /// The other half of the same clause: a root that exists but is a regular
    /// file. It leaves `walk` through a different branch than the missing-root
    /// case, and without this test a regression there would emit a document
    /// with an empty tree and exit 0.
    #[test]
    fn test_walk_rejects_file_as_root() {
        let dir = tmp_dir("fileroot");
        let file = dir.join("regular.txt");
        fs::write(&file, "x").unwrap();

        let err = walk(&opts_for(&file, &[], &[])).unwrap_err();
        assert!(
            err.to_string().contains("not a directory"),
            "unexpected error: {err}"
        );
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

    // ---- T-004: content section renderer ----

    fn ok_body(s: &str) -> Result<Vec<u8>, io::Error> {
        Ok(s.as_bytes().to_vec())
    }

    fn node_at(rel: &str) -> Node {
        Node {
            name: rel.rsplit('/').next().unwrap().to_string(),
            rel: rel.to_string(),
            kind: Kind::File,
            unreadable: false,
            children: Vec::new(),
        }
    }

    #[test]
    fn test_file_header_format() {
        let out = render_section(&node_at("src/main.rs"), &ok_body("x\n"), Wrap::None);
        let lines: Vec<&str> = out.lines().collect();
        // Leading blank line, then the inherited three-line header.
        assert_eq!(lines[0], "");
        assert_eq!(lines[1], "---");
        assert_eq!(lines[2], "File: src/main.rs");
        assert_eq!(lines[3], "---");
    }

    #[test]
    fn test_section_separator_blank_line() {
        let a = render_section(&node_at("a.txt"), &ok_body("last line\n"), Wrap::None);
        let b = render_section(&node_at("b.txt"), &ok_body("x\n"), Wrap::None);
        let joined = format!("{a}{b}");
        // Without the blank line, `---` would underline "last line" as a
        // setext heading and b.txt's header would vanish.
        assert!(joined.contains("last line\n\n---\nFile: b.txt"));
    }

    #[test]
    fn test_fence_len_plain_body() {
        assert_eq!(fence_len("no backticks here"), 3);
    }

    #[test]
    fn test_fence_len_body_with_triple_backticks() {
        assert_eq!(fence_len("text\n```\ncode\n```\n"), 4);
    }

    #[test]
    fn test_fence_len_body_with_quad_backticks() {
        assert_eq!(fence_len("````\nx\n````"), 5);
    }

    #[test]
    fn test_fence_len_counts_longest_run_not_total() {
        assert_eq!(fence_len("`a` `b` `c` `d` `e`"), 3);
    }

    #[test]
    fn test_binary_body_elided() {
        let bytes = vec![0xffu8, 0xfe, 0x00, 0x01];
        let out = render_section(&node_at("blob.bin"), &Ok(bytes), Wrap::Fence);
        assert!(out.contains("[binary file, 4 bytes elided]"));
        assert!(!out.contains('\u{fffd}'), "raw bytes must not be emitted");
    }

    #[test]
    fn test_unreadable_file_body_marked() {
        let err = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");
        let out = render_section(&node_at("secret.txt"), &Err(err), Wrap::Fence);
        assert!(out.contains("[unreadable: "));
        assert!(out.contains("access denied"));
    }

    /// The second half of the same clause: rendering continues past a file it
    /// cannot read. Deterministic and host-independent — the file is removed
    /// after the walk records it, so the read fails without needing
    /// permissions, which is what lets this run on Windows too.
    #[test]
    fn test_unreadable_file_does_not_abort_document() {
        let dir = tmp_dir("gonefile");
        fs::write(dir.join("a-gone.txt"), "will vanish\n").unwrap();
        fs::write(dir.join("b-stays.txt"), "still here\n").unwrap();

        let opts = opts_for(&dir, &[], &[]);
        let root = walk(&opts).unwrap();
        fs::remove_file(dir.join("a-gone.txt")).unwrap();
        let doc = render_document(&root, &opts);

        assert!(doc.contains("File: a-gone.txt"));
        assert!(
            doc.contains("[unreadable: "),
            "the failure is marked in place"
        );
        assert!(
            doc.contains("File: b-stays.txt") && doc.contains("still here"),
            "rendering must continue with the next file"
        );
    }

    #[test]
    fn test_language_hint_known_and_unknown() {
        assert_eq!(language_hint("main.rs"), "rust");
        assert_eq!(language_hint("README.md"), "markdown");
        assert_eq!(language_hint("thing.xyz"), "");
        assert_eq!(language_hint("LICENSE"), "");
    }

    #[test]
    fn test_body_fence_survives_inner_fence() {
        let out = render_section(
            &node_at("doc.md"),
            &ok_body("```\ninner\n```\n"),
            Wrap::Fence,
        );
        assert!(
            out.contains("````markdown\n"),
            "opening fence must outgrow the body"
        );
    }

    /// Exercises `join_rel` through a real nested walk. Handing
    /// `render_section` a node whose `rel` is already forward-slashed would be
    /// a tautology: the renderer only interpolates the string it is given, so
    /// the platform behaviour lives in the walk, not here.
    #[test]
    fn test_relative_path_uses_forward_slashes() {
        let dir = tmp_dir("slashes");
        fs::create_dir_all(dir.join("a/b")).unwrap();
        fs::write(dir.join("a/b/c.txt"), "x\n").unwrap();

        let opts = opts_for(&dir, &[], &[]);
        let root = walk(&opts).unwrap();
        let doc = render_document(&root, &opts);

        assert!(doc.contains("File: a/b/c.txt"));
        for line in doc.lines().filter(|l| l.starts_with("File: ")) {
            assert!(!line.contains('\\'), "backslash in path: {line}");
        }
    }

    #[test]
    fn test_document_assembles_in_order() {
        let root = dir_node("root", vec![]);
        let doc = render_document(&root, &Options::default());
        let title = doc.find("# root").unwrap();
        let structure = doc.find("## Structure").unwrap();
        let contents = doc.find("## Contents").unwrap();
        assert!(title < structure && structure < contents);
    }

    // ---- T-004: traversal + rendering integration ----

    #[test]
    fn test_excluded_entry_absent_from_both_halves() {
        let dir = tmp_dir("excluded");
        fs::create_dir(dir.join("target")).unwrap();
        fs::write(dir.join("target/artifact.txt"), "built").unwrap();
        fs::write(dir.join("keep.txt"), "kept").unwrap();

        let opts = opts_for(&dir, &["target"], &[]);
        let root = walk(&opts).unwrap();
        let doc = render_document(&root, &opts);

        assert!(!doc.contains("target"), "absent from scaffold and contents");
        assert!(!doc.contains("built"));
        assert!(doc.contains("keep.txt") && doc.contains("kept"));
    }

    #[test]
    fn test_included_entry_present_in_both_halves() {
        let dir = tmp_dir("included");
        fs::create_dir(dir.join(".github")).unwrap();
        fs::write(dir.join(".github/ci.yml"), "on: push").unwrap();

        let opts = opts_for(&dir, &[".*"], &[".github"]);
        let root = walk(&opts).unwrap();
        let doc = render_document(&root, &opts);

        assert!(doc.contains("├── .github/") || doc.contains("└── .github/"));
        assert!(doc.contains("File: .github/ci.yml"));
        assert!(doc.contains("on: push"));
    }

    // ---- Sprint 1 / T-002: the batched gitignore query ----

    fn git_available() -> bool {
        Command::new("git")
            .arg("--version")
            .output()
            .is_ok_and(|o| o.status.success())
    }

    /// Run git for fixture setup, isolated from the host's git configuration
    /// and from any ambient repository.
    fn git(dir: &Path, args: &[&str]) {
        let iso = env::temp_dir().join(format!("mdeezl-gitiso-{}", process::id()));
        fs::create_dir_all(&iso).unwrap();
        let _ = fs::write(iso.join("gitconfig"), "");
        let mut cmd = Command::new("git");
        cmd.current_dir(dir)
            .args(args)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", iso.join("gitconfig"))
            .env("XDG_CONFIG_HOME", &iso)
            .env("GIT_CEILING_DIRECTORIES", env::temp_dir());
        for var in GIT_LOCAL_ENV_VARS {
            cmd.env_remove(var);
        }
        let out = cmd.output().unwrap();
        assert!(
            out.status.success(),
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }

    /// Stage without committing: a staged file already counts as tracked, and
    /// committing would need an identity CI runners do not have. Literal
    /// pathspecs, so `x[1].log` is not read as a glob.
    fn git_add(dir: &Path, paths: &[&str]) {
        let mut args = vec!["--literal-pathspecs", "add", "-f", "--"];
        args.extend_from_slice(paths);
        git(dir, &args);
    }

    fn init_repo(tag: &str) -> Tmp {
        let dir = tmp_dir(tag);
        git(&dir, &["init", "-q", "."]);
        dir
    }

    fn touch(dir: &Path, rel: &str) {
        let p = dir.join(rel);
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(p, "x\n").unwrap();
    }

    fn strings(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    fn set(items: &[&str]) -> HashSet<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    macro_rules! require_git {
        ($name:literal) => {
            if !git_available() {
                eprintln!(concat!("SKIP ", $name, ": git not on PATH"));
                return;
            }
        };
    }

    #[test]
    fn test_nul_payload() {
        let got = nul_payload(&strings(&["a", "b/c", ":x", r"p*[1]?\z"]));
        // Built from explicit byte values, so the backslash class cannot share a
        // transcription slip with the implementation's literal.
        let mut want = Vec::new();
        for p in [&b"./a"[..], b"./b/c", b"./:x"] {
            want.extend_from_slice(p);
            want.push(0);
        }
        want.extend_from_slice(b"./p[*][[]1][?]");
        // Non-raw notation, unlike the implementation's raw literal, and its
        // length pinned: `[`, `\`, `\`, `]`.
        let backslash_class: &[u8] = b"[\\\\]";
        assert_eq!(backslash_class.len(), 4);
        want.extend_from_slice(backslash_class);
        want.extend_from_slice(b"z\0");
        assert_eq!(got, want);

        // No backslash outside the four-byte class: that is what keeps the
        // payload safe on Git for Windows, which turns a bare `\` into `/`.
        let stripped: Vec<u8> = got
            .split(|&b| b == 0)
            .flat_map(|f| {
                let mut v = f.to_vec();
                while let Some(i) = v.windows(4).position(|w| w == backslash_class) {
                    v.drain(i..i + 4);
                }
                v
            })
            .collect();
        assert!(!stripped.contains(&b'\\'));
    }

    #[test]
    fn test_parse_records_maps_by_position() {
        fn rec(src: &str, line: &str, pat: &str, path: &str) -> Vec<u8> {
            [src, line, pat, path]
                .iter()
                .flat_map(|f| f.bytes().chain([0]))
                .collect()
        }
        let out: Vec<u8> = [
            rec("", "", "", "./tracked.log"),
            rec(".gitignore", "1", "*.log", "./a.log"),
            rec(".gitignore", "2", "!keep.log", "./keep.log"),
            rec("", "", "", "./x.txt"),
        ]
        .concat();
        assert_eq!(parse_records(&out, 4), Ok(vec![false, true, false, false]));
        // One record short must be an error, never a shifted mapping.
        let short: Vec<u8> = out[..out.len() - rec("", "", "", "./x.txt").len()].to_vec();
        assert!(parse_records(&short, 4).is_err());
    }

    #[test]
    fn test_gitignore_candidates_root_repo_and_nested_repo() {
        let dir = tmp_dir("candidates");
        fs::create_dir(dir.join(".git")).unwrap(); // the scan root is a repository
        touch(&dir, "a.txt");
        touch(&dir, "sub/b.txt");
        touch(&dir, "nested/.git"); // a nested repository's gitlink file
        touch(&dir, "nested/c.txt");
        // An ordinary nested clone: its `.git` is a directory, not a gitlink
        // file. Both forms must stop the walk, or `.exists()` could regress to
        // `.is_file()` unnoticed.
        fs::create_dir_all(dir.join("clone/.git")).unwrap();
        touch(&dir, "clone/d.txt");

        let tree = walk(&opts_for(&dir, &[".*"], &[])).unwrap();
        let got = gitignore_candidates(&dir, &tree);

        for want in [".", "a.txt", "sub", "sub/b.txt", "nested", "clone"] {
            assert!(got.contains(&want.to_string()), "missing {want}: {got:?}");
        }
        assert!(!got.contains(&"nested/c.txt".to_string()));
        assert!(!got.contains(&"clone/d.txt".to_string()));
        assert!(!got.iter().any(String::is_empty));
    }

    /// The directory-removal guard's precondition, pinned against whatever git
    /// the tests run on. Planning measured it on git 2.54; CI runs a newer git.
    /// If a git ever stopped reporting the escaped directory, the end-to-end
    /// guard test would still pass without exercising the guard -- this fails.
    #[test]
    fn test_git_ignored_whitelist_reports_escaped_directory() {
        require_git!("test_git_ignored_whitelist_reports_escaped_directory");
        let dir = init_repo("gi-whitelist");
        fs::write(dir.join(".gitignore"), "*\n!*/\n!*.tsx\n!.gitignore\n").unwrap();
        touch(&dir, "app/[slug]/page.tsx");
        touch(&dir, "app/[slug]/notes.md");
        git_add(&dir, &["app/[slug]/page.tsx"]);
        let got = git_ignored(
            &dir,
            &strings(&[
                ".",
                "app",
                "app/[slug]",
                "app/[slug]/page.tsx",
                "app/[slug]/notes.md",
            ]),
        )
        .unwrap();
        assert!(
            got.contains("app/[slug]"),
            "precondition for the guard: {got:?}"
        );
        assert!(!got.contains("app/[slug]/page.tsx"));
        assert!(got.contains("app/[slug]/notes.md"));
    }

    /// Every variable git lists as repository-local must be removed from the
    /// query's environment. A git that adds one should fail this, not silently
    /// let it redirect the query.
    #[test]
    fn test_git_local_env_vars_cover_gits_list() {
        require_git!("test_git_local_env_vars_cover_gits_list");
        let out = Command::new("git")
            .args(["rev-parse", "--local-env-vars"])
            .output()
            .unwrap();
        assert!(out.status.success());
        let listed = String::from_utf8_lossy(&out.stdout);
        // Guard against a vacuous pass on empty output.
        for known in ["GIT_DIR", "GIT_INDEX_FILE"] {
            assert!(
                listed.lines().any(|l| l.trim() == known),
                "git's list should include {known}: {listed:?}"
            );
        }
        for var in listed.lines() {
            assert!(
                GIT_LOCAL_ENV_VARS.contains(&var.trim()),
                "git lists {var} as repository-local, but mdeezl does not remove it"
            );
        }
    }

    #[test]
    fn test_git_ignored_returns_ignored_set() {
        require_git!("test_git_ignored_returns_ignored_set");
        let dir = init_repo("gi-set");
        fs::write(dir.join(".gitignore"), "out/\n*.log\n").unwrap();
        touch(&dir, "out/a.o");
        touch(&dir, "app.log");
        touch(&dir, "keep.txt");
        let got = git_ignored(
            &dir,
            &strings(&[".", "out", "out/a.o", "app.log", "keep.txt"]),
        );
        assert_eq!(got, Ok(set(&["out", "out/a.o", "app.log"])));
    }

    #[test]
    fn test_git_ignored_nothing_ignored_is_empty() {
        require_git!("test_git_ignored_nothing_ignored_is_empty");
        let dir = init_repo("gi-none");
        touch(&dir, "a.txt");
        assert_eq!(git_ignored(&dir, &strings(&[".", "a.txt"])), Ok(set(&[])));
    }

    #[test]
    fn test_git_ignored_full_syntax() {
        require_git!("test_git_ignored_full_syntax");
        let dir = init_repo("gi-syntax");
        fs::write(dir.join(".gitignore"), "*.log\n!keep.log\n**/generated/\n").unwrap();
        fs::create_dir_all(dir.join("sub")).unwrap();
        fs::write(dir.join("sub/.gitignore"), "local.tmp\n").unwrap();
        let exclude = dir.join(".git/info/exclude");
        let mut ex = fs::read_to_string(&exclude).unwrap_or_default();
        ex.push_str("excluded.txt\n");
        fs::write(&exclude, ex).unwrap();
        fs::write(dir.join("globalish"), "viaconfig.txt\n").unwrap();
        // Forward slashes: a Windows backslash path in .git/config is read as
        // escape sequences.
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
            touch(&dir, f);
        }
        let got = git_ignored(
            &dir,
            &strings(&[
                ".",
                "app.log",
                "keep.log",
                "deep",
                "deep/x",
                "deep/x/generated",
                "deep/x/generated/g.txt",
                "sub",
                "sub/local.tmp",
                "sub/other.tmp",
                "excluded.txt",
                "viaconfig.txt",
                "plain.txt",
            ]),
        );
        assert_eq!(
            got,
            Ok(set(&[
                "app.log",
                "deep/x/generated",
                "deep/x/generated/g.txt",
                "sub/local.tmp",
                "excluded.txt",
                "viaconfig.txt",
            ]))
        );
    }

    #[test]
    fn test_git_ignored_subdir_root() {
        require_git!("test_git_ignored_subdir_root");
        let dir = init_repo("gi-subdir");
        fs::write(dir.join(".gitignore"), "*.log\n").unwrap();
        touch(&dir, "sub/x.log");
        touch(&dir, "sub/keep.txt");
        let got = git_ignored(&dir.join("sub"), &strings(&[".", "x.log", "keep.txt"]));
        assert_eq!(got, Ok(set(&["x.log"])));
    }

    #[test]
    fn test_git_ignored_reports_root_only_when_ignored() {
        require_git!("test_git_ignored_reports_root_only_when_ignored");
        let dir = init_repo("gi-rootdot");
        fs::write(dir.join(".gitignore"), "out/\n").unwrap();
        touch(&dir, "out/a.o");
        let out = dir.join("out");

        let untracked_only = git_ignored(&out, &strings(&[".", "a.o"])).unwrap();
        assert!(
            untracked_only.contains("."),
            "root holding only untracked files is ignored"
        );
        assert!(untracked_only.contains("a.o"));

        touch(&dir, "out/keep.txt");
        git_add(&dir, &["out/keep.txt"]);
        let with_tracked = git_ignored(&out, &strings(&[".", "a.o", "keep.txt"])).unwrap();
        assert!(
            !with_tracked.contains("."),
            "a root holding tracked files is not reported"
        );
        assert!(with_tracked.contains("a.o"));
        assert!(!with_tracked.contains("keep.txt"));
    }

    #[test]
    fn test_git_ignored_glob_chars_are_literal() {
        require_git!("test_git_ignored_glob_chars_are_literal");
        let dir = init_repo("gi-glob");
        fs::write(dir.join(".gitignore"), "*.log\nout/\n").unwrap();
        for f in [
            "a1.log",
            "a[1].log",
            "x[1].log",
            "out[1].txt",
            "out/z.o",
            "pages/[id].tsx",
        ] {
            touch(&dir, f);
        }
        git_add(&dir, &["a1.log", "x[1].log", "pages/[id].tsx"]);
        let got = git_ignored(
            &dir,
            &strings(&[
                ".",
                "a1.log",
                "a[1].log",
                "x[1].log",
                "out",
                "out[1].txt",
                "pages",
                "pages/[id].tsx",
            ]),
        )
        .unwrap();
        // Unescaped, a[1].log's glob would hit the tracked a1.log and be suppressed.
        assert!(got.contains("a[1].log"));
        assert!(!got.contains("a1.log"));
        // Windows-sensitive: backslash escaping reads these as x/[1].log and out/[1].txt.
        assert!(
            !got.contains("x[1].log"),
            "a tracked file must not be reported"
        );
        assert!(!got.contains("out[1].txt"), "no rule names out[1].txt");
        assert!(!got.contains("pages/[id].tsx"));

        #[cfg(unix)]
        {
            // The one git-level check of the four-byte `\` class. A wrong class
            // makes git miss the index entry and report the tracked file.
            touch(&dir, r"x\y.log");
            touch(&dir, r"u\v.log");
            git_add(&dir, &[r"x\y.log"]);
            let got = git_ignored(&dir, &strings(&[".", r"x\y.log", r"u\v.log"])).unwrap();
            assert!(
                !got.contains(r"x\y.log"),
                "tracked x\\y.log must not be reported"
            );
            assert!(got.contains(r"u\v.log"));
        }
        #[cfg(not(unix))]
        eprintln!(
            "SKIP test_git_ignored_glob_chars_are_literal (backslash case): \
             this platform forbids `\\` in filenames. The Linux CI leg runs it."
        );
    }

    #[test]
    fn test_git_ignored_submodule_candidates_do_not_abort() {
        require_git!("test_git_ignored_submodule_candidates_do_not_abort");
        let src = init_repo("gi-subsrc");
        touch(&src, "inner.txt");
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

        let dir = init_repo("gi-submod");
        fs::write(dir.join(".gitignore"), "*.log\n").unwrap();
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
        touch(&dir, "mod/x.log");
        touch(&dir, "top.log");

        let tree = walk(&opts_for(&dir, &[".*"], &[])).unwrap();
        let candidates = gitignore_candidates(&dir, &tree);
        assert!(!candidates.contains(&"mod/x.log".to_string()));
        let got = git_ignored(&dir, &candidates);
        assert!(got.is_ok(), "a submodule must not abort the query: {got:?}");
        assert!(got.unwrap().contains("top.log"));
    }

    #[test]
    fn test_git_ignored_outside_work_tree_errs() {
        require_git!("test_git_ignored_outside_work_tree_errs");
        let dir = tmp_dir("gi-plain");
        let err = git_ignored(&dir, &strings(&[".", "a"])).unwrap_err();
        assert!(err.contains("not a git repository"), "unexpected: {err}");
    }

    #[test]
    fn test_git_ignored_tracked_file_not_reported() {
        require_git!("test_git_ignored_tracked_file_not_reported");
        let dir = init_repo("gi-tracked");
        fs::write(dir.join(".gitignore"), "*.log\n").unwrap();
        touch(&dir, "tracked.log");
        touch(&dir, "free.log");
        git_add(&dir, &["tracked.log"]);
        let got = git_ignored(&dir, &strings(&[".", "tracked.log", "free.log"])).unwrap();
        assert!(!got.contains("tracked.log"));
        assert!(got.contains("free.log"));
    }

    /// 20,000 synthetic paths of 60 bytes: over 1.2 MB each way, far past
    /// any pipe buffer, which is what deadlocks a write-then-read implementation.
    /// File globs match paths that do not exist, so no files are created.
    fn synthetic_paths() -> Vec<String> {
        (0..20_000)
            .map(|i| {
                format!(
                    "generated_output/directory_{:03}/compiled_object_file_{i:06}.o",
                    i / 100
                )
            })
            .collect()
    }

    fn call_with_deadline(dir: &Path, paths: Vec<String>) -> Result<HashSet<String>, String> {
        let (tx, rx) = std::sync::mpsc::channel();
        let d = dir.to_path_buf();
        std::thread::spawn(move || {
            let _ = tx.send(git_ignored(&d, &paths));
        });
        rx.recv_timeout(std::time::Duration::from_secs(60))
            .expect("git_ignored deadlocked or panicked: no result within 60 s")
    }

    #[test]
    fn test_git_ignored_large_list_no_deadlock() {
        require_git!("test_git_ignored_large_list_no_deadlock");
        let dir = init_repo("gi-large");
        fs::write(dir.join(".gitignore"), "*.o\n").unwrap();
        let paths = synthetic_paths();
        assert_eq!(paths[0].len(), 60, "paths are about 60 bytes, as planned");
        // Checked, not claimed: over 1.2 MB inbound; each outbound record
        // repeats the path beside the matching rule, so outbound is larger.
        assert!(nul_payload(&paths).len() > 1_200_000);
        let got = call_with_deadline(&dir, paths).unwrap();
        assert_eq!(got.len(), 20_000);
    }

    #[test]
    fn test_git_ignored_large_list_outside_work_tree_errs_without_panic() {
        require_git!("test_git_ignored_large_list_outside_work_tree_errs_without_panic");
        // Git exits during setup, before reading stdin, so the writer hits a
        // broken pipe. That must surface as Err, not a panic.
        let dir = tmp_dir("gi-large-plain");
        assert!(call_with_deadline(&dir, synthetic_paths()).is_err());
    }

    #[test]
    fn test_run_spawns_git_exactly_once() {
        require_git!("test_run_spawns_git_exactly_once");
        let dir = init_repo("gi-once");
        fs::write(dir.join(".gitignore"), "*.log\n").unwrap();
        for f in ["a.txt", "b.log", "src/c.rs", "src/d/e.rs", "docs/f.md"] {
            touch(&dir, f);
        }
        let sink = tmp_dir("gi-once-out");
        let opts = Options {
            root: dir.to_path_buf(),
            sink: Sink::File(sink.join("doc.md")),
            ..Options::default()
        };
        GIT_SPAWNS.with(|n| n.set(0));
        run(&opts).unwrap();
        assert_eq!(GIT_SPAWNS.with(|n| n.get()), 1);
        // And the one call did its job end to end: pruned, then rendered. `run`
        // returns Ok even when the query fails, so the count alone would not
        // show that.
        let doc = fs::read_to_string(sink.join("doc.md")).unwrap();
        assert!(
            !doc.contains("File: b.log"),
            "gitignored file must be pruned"
        );
        assert!(
            doc.contains("File: src/d/e.rs"),
            "nested kept file must render"
        );
    }

    // ---- Sprint 1 / T-003: pruning ----

    fn f(rel: &str) -> Node {
        Node {
            name: rel.rsplit('/').next().unwrap().to_string(),
            rel: rel.to_string(),
            kind: Kind::File,
            unreadable: false,
            children: Vec::new(),
        }
    }

    fn d(rel: &str, children: Vec<Node>) -> Node {
        Node {
            kind: Kind::Dir,
            children,
            ..f(rel)
        }
    }

    /// Every surviving path, sorted.
    fn surviving(node: &Node) -> Vec<String> {
        fn visit(n: &Node, out: &mut Vec<String>) {
            for c in &n.children {
                out.push(c.rel.clone());
                visit(c, out);
            }
        }
        let mut out = Vec::new();
        visit(node, &mut out);
        out.sort();
        out
    }

    /// root/{out/{a.o, deep/b.o}, src/{gen.o, main.rs}, top.o, .cache/{c.bin}, keep.txt}
    fn prune_fixture() -> Node {
        d(
            "",
            vec![
                d(
                    "out",
                    vec![f("out/a.o"), d("out/deep", vec![f("out/deep/b.o")])],
                ),
                d("src", vec![f("src/gen.o"), f("src/main.rs")]),
                f("top.o"),
                d(".cache", vec![f(".cache/c.bin")]),
                f("keep.txt"),
            ],
        )
    }

    /// What git returns for `out/`, `*.o`, `.cache/`: every descendant of an
    /// ignored directory is listed, as git really does.
    fn prune_fixture_ignored() -> HashSet<String> {
        set(&[
            "out",
            "out/a.o",
            "out/deep",
            "out/deep/b.o",
            "src/gen.o",
            "top.o",
            ".cache",
            ".cache/c.bin",
        ])
    }

    fn pruned_with(include: &[&str]) -> Vec<String> {
        let mut tree = prune_fixture();
        let mut queried: HashSet<String> = surviving(&tree).into_iter().collect();
        queried.insert(".".to_string());
        prune_gitignored(
            &mut tree,
            &prune_fixture_ignored(),
            &queried,
            &strings(include),
        );
        surviving(&tree)
    }

    fn sorted(items: &[&str]) -> Vec<String> {
        let mut v = strings(items);
        v.sort();
        v
    }

    #[test]
    fn test_prune_gitignored_removes_subtree() {
        assert_eq!(
            pruned_with(&[]),
            sorted(&["src", "src/main.rs", "keep.txt"])
        );
    }

    #[test]
    fn test_prune_gitignored_include_exempts_subtree() {
        // Exact name, in the set: the whole out/ subtree survives.
        assert_eq!(
            pruned_with(&["out"]),
            sorted(&[
                "out",
                "out/a.o",
                "out/deep",
                "out/deep/b.o",
                "src",
                "src/main.rs",
                "keep.txt"
            ])
        );
        // File type: top-level and src/ .o files survive; out/ is still pruned
        // with the .o files inside it, because pruning is top-down.
        assert_eq!(
            pruned_with(&["*.o"]),
            sorted(&["src", "src/gen.o", "src/main.rs", "top.o", "keep.txt"])
        );
        // A path.
        assert_eq!(
            pruned_with(&["src/gen.o"]),
            sorted(&["src", "src/gen.o", "src/main.rs", "keep.txt"])
        );
        // Dot-entries, via .cache in the set.
        assert_eq!(
            pruned_with(&[".*"]),
            sorted(&[".cache", ".cache/c.bin", "src", "src/main.rs", "keep.txt"])
        );
    }

    #[test]
    fn test_prune_gitignored_include_of_unignored_dir_gives_no_exemption() {
        // src is not in the set, so including it exempts nothing inside it.
        assert_eq!(
            pruned_with(&["src"]),
            sorted(&["src", "src/main.rs", "keep.txt"])
        );
    }

    #[test]
    fn test_prune_gitignored_ancestor_blocks_include() {
        let got = pruned_with(&["out/deep/b.o"]);
        assert!(!got.contains(&"out/deep/b.o".to_string()));
        assert_eq!(got, sorted(&["src", "src/main.rs", "keep.txt"]));
    }

    #[test]
    fn test_prune_gitignored_guard_keeps_unreported_descendant() {
        // Mirrors git for `*`, `!*/`, `!*.tsx`, then `nested/`: the escaped
        // app/[slug] is reported while its tracked page.tsx is not.
        let mut tree = d(
            "",
            vec![
                d(
                    "app",
                    vec![d(
                        "app/[slug]",
                        vec![f("app/[slug]/page.tsx"), f("app/[slug]/notes.md")],
                    )],
                ),
                d("nested", vec![f("nested/inner.txt")]),
            ],
        );
        // nested/inner.txt was never queried: nested holds a nested repository.
        let queried = set(&[
            ".",
            "app",
            "app/[slug]",
            "app/[slug]/page.tsx",
            "app/[slug]/notes.md",
            "nested",
        ]);
        let ignored = set(&["app/[slug]", "app/[slug]/notes.md", "nested"]);
        prune_gitignored(&mut tree, &ignored, &queried, &[]);
        assert_eq!(
            surviving(&tree),
            sorted(&["app", "app/[slug]", "app/[slug]/page.tsx"])
        );
    }

    #[test]
    fn test_manifest_declares_bin_name_and_license() {
        let manifest = include_str!("../Cargo.toml");
        // The bin name is load-bearing: CARGO_BIN_EXE_mdeezl resolves by it.
        assert!(manifest.contains("name = \"mdeezl\""));
        assert!(manifest.contains("license = \"Apache-2.0\""));
    }

    // ============ Sprint 3: library split, URL sources (INT-0006, INT-0002) ============

    /// T-001. A notice comes back as data, without the CLI's `mdeezl: ` prefix.
    /// That `bundle` prints nothing itself is pinned end to end: the CLI's
    /// one-line stderr assertions would see a second copy.
    #[test]
    fn test_bundle_returns_notices_as_data() {
        let dir = tmp_dir("notices");
        fs::write(dir.join("a.txt"), "a\n").unwrap();
        let b = bundle(&opts_for(&dir, &[], &[])).unwrap();
        assert_eq!(b.notices.len(), 1, "{:?}", b.notices);
        assert!(
            b.notices[0].starts_with("gitignore filtering skipped:"),
            "{:?}",
            b.notices
        );
        assert!(!b.notices[0].contains("mdeezl: "));
        assert!(b.document.contains("a.txt"));
    }

    /// A `file://` URL for a local folder, the one way every test builds it:
    /// canonicalise, drop Windows' `\\?\`, forward slashes, then `file://`
    /// before an absolute Unix path or `file:///` before a drive path.
    fn file_url(path: &Path) -> String {
        let p = path
            .canonicalize()
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        let p = p.trim_start_matches("//?/");
        if p.starts_with('/') {
            format!("file://{p}")
        } else {
            format!("file:///{p}")
        }
    }

    /// A committed repository: a `* text=auto` attribute, an LF text file
    /// committed with `core.autocrlf=false`, and an untracked file beside it.
    fn committed_repo(tag: &str) -> Tmp {
        let dir = init_repo(tag);
        fs::write(dir.join(".gitattributes"), "* text=auto\n").unwrap();
        fs::write(dir.join("t.txt"), "one\ntwo\n").unwrap();
        git(
            &dir,
            &[
                "-c",
                "core.autocrlf=false",
                "add",
                "--",
                ".gitattributes",
                "t.txt",
            ],
        );
        git(
            &dir,
            &[
                "-c",
                "user.name=t",
                "-c",
                "user.email=t@t",
                "-c",
                "core.autocrlf=false",
                "commit",
                "-q",
                "-m",
                "init",
            ],
        );
        fs::write(dir.join("untracked.txt"), "not committed\n").unwrap();
        dir
    }

    fn is_empty_dir(dir: &Path) -> bool {
        fs::read_dir(dir).unwrap().next().is_none()
    }

    /// T-002.
    #[test]
    fn test_is_url() {
        for yes in ["https://x/y", "http://x/y", "file:///x"] {
            assert!(is_url(yes), "{yes}");
        }
        for no in [r"C:\x", "./https", "ftp:/weird", "git@host:x/y", ""] {
            assert!(!is_url(no), "{no}");
        }
    }

    /// T-002. The name becomes a path component, so it must never leave its
    /// folder.
    #[test]
    fn test_repo_name() {
        for (url, name) in [
            ("https://github.com/octocat/Hello-World", "Hello-World"),
            ("https://github.com/o/repo.git", "repo"),
            ("https://github.com/o/repo/", "repo"),
            ("file:///C:/tmp/fixture", "fixture"),
            ("https://host/", "repository"),
            ("https://host", "repository"),
            ("https://h/o/r?x=1#top", "r"),
            ("https://h/o/..", "repository"),
            (r"https://h/o/a\..\..\x", "repository"),
            ("https://h/o/c:x", "repository"),
        ] {
            assert_eq!(repo_name(url), name, "{url}");
        }
    }

    /// T-002.
    #[test]
    fn test_display_url_strips_credentials() {
        assert_eq!(
            display_url("https://user:TOKEN@github.com/o/r"),
            "https://github.com/o/r"
        );
        assert_eq!(display_url("https://tok@h/r"), "https://h/r");
        for same in ["https://github.com/o/r", "file:///C:/x"] {
            assert_eq!(display_url(same), same);
        }
    }

    /// T-002. The exact command and environment that keep a fetch from ever
    /// waiting on a person.
    #[test]
    fn test_clone_command_is_hardened() {
        let cmd = clone_command("https://h/o/r", Path::new("dest"));
        let args: Vec<String> = cmd
            .get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        let want = [
            "-c",
            "credential.helper=",
            "-c",
            "core.askPass=",
            "-c",
            "core.autocrlf=false",
            "-c",
            "core.eol=lf",
            "-c",
            "http.lowSpeedLimit=1",
            "-c",
            "http.lowSpeedTime=30",
            "clone",
            "--depth",
            "1",
            "--quiet",
            "--",
            "https://h/o/r",
            "dest",
        ];
        assert_eq!(args, want);
        let envs: Vec<(String, Option<String>)> = cmd
            .get_envs()
            .map(|(k, v)| {
                (
                    k.to_string_lossy().into_owned(),
                    v.map(|v| v.to_string_lossy().into_owned()),
                )
            })
            .collect();
        let set_to = |key: &str| {
            envs.iter()
                .find(|(k, _)| k == key)
                .and_then(|(_, v)| v.clone())
        };
        let removed = |key: &str| envs.iter().any(|(k, v)| k == key && v.is_none());
        assert_eq!(set_to("GIT_TERMINAL_PROMPT").as_deref(), Some("0"));
        assert_eq!(set_to("GCM_INTERACTIVE").as_deref(), Some("never"));
        assert_eq!(
            set_to("GIT_ALLOW_PROTOCOL").as_deref(),
            Some("https:http:file")
        );
        assert_eq!(set_to("GIT_LFS_SKIP_SMUDGE").as_deref(), Some("1"));
        for var in [
            "GIT_ASKPASS",
            "SSH_ASKPASS",
            "GIT_HTTP_LOW_SPEED_LIMIT",
            "GIT_HTTP_LOW_SPEED_TIME",
        ]
        .into_iter()
        .chain(GIT_LOCAL_ENV_VARS)
        {
            assert!(removed(var), "{var} must be removed");
        }
    }

    /// T-002. The allow-list refuses ssh before any network is reached.
    #[test]
    fn test_fetch_rejects_protocol_outside_allow_list() {
        require_git!("test_fetch_rejects_protocol_outside_allow_list");
        let base = tmp_dir("fetch-ssh");
        let err = fetch("ssh://127.0.0.1/x", &base).err().expect("must fail");
        assert!(err.to_string().contains("not allowed"), "{err}");
        assert!(is_empty_dir(&base));
    }

    /// T-002. A clone is bundled under its URL, holds only committed files with
    /// LF text on every OS, and leaves nothing behind.
    #[test]
    fn test_fetch_file_url_bundles_the_clone() {
        require_git!("test_fetch_file_url_bundles_the_clone");
        let repo = committed_repo("fetch-src");
        let url = file_url(&repo);
        let base = tmp_dir("fetch-base");
        let opts = Options {
            root: PathBuf::from(&url),
            ..Options::default()
        };
        let b = bundle_in(&opts, &base).unwrap();
        let name = repo.file_name().unwrap().to_str().unwrap();
        assert!(
            b.document.starts_with(&format!("# {url}\n")),
            "{}",
            b.document
        );
        assert!(
            b.document.contains(&format!("\n{name}/\n")),
            "{}",
            b.document
        );
        assert_eq!(b.name, name);
        assert!(b.document.contains("\none\ntwo\n"), "exact LF bytes");
        assert!(!b.document.contains("\r"));
        assert!(!b.document.contains("untracked.txt"));
        assert!(is_empty_dir(&base), "the clone must be removed");
    }

    /// T-002. The reason is git's first `fatal:` line, not its boilerplate.
    #[test]
    fn test_fetch_bad_url_errs() {
        require_git!("test_fetch_bad_url_errs");
        let base = tmp_dir("fetch-bad");
        let missing = base.join("no-such-repo");
        let url = format!("{}/no-such-repo", file_url(&base));
        assert!(!missing.exists());
        let err = fetch(&url, &base).err().expect("must fail").to_string();
        assert!(err.starts_with("could not fetch "), "{err}");
        assert!(
            err.contains("does not appear to be a git repository"),
            "{err}"
        );
        assert!(is_empty_dir(&base));
    }

    /// T-002. A server that accepts and never answers must not hang the fetch.
    #[test]
    fn test_stalled_server_fails_the_fetch() {
        require_git!("test_stalled_server_fails_the_fetch");
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        std::thread::spawn(move || {
            let mut held = Vec::new();
            for conn in listener.incoming() {
                held.push(conn); // hold it open, send nothing
            }
        });
        let base = tmp_dir("fetch-stall");
        LOW_SPEED_OVERRIDE.with(|c| c.set(Some(3)));
        let started = std::time::Instant::now();
        let result = fetch(&format!("http://127.0.0.1:{port}/x.git"), &base);
        LOW_SPEED_OVERRIDE.with(|c| c.set(None));
        let err = result.err().expect("a stalled fetch must fail").to_string();
        assert!(err.contains("Operation too slow"), "{err}");
        assert!(started.elapsed() < std::time::Duration::from_secs(15));
        assert!(is_empty_dir(&base));
    }

    /// T-002. A repository that demands credentials: the half of INT-0002's
    /// failure criterion that needs a server to refuse, rather than to be
    /// absent. A local socket answers 401 with a Basic challenge, which is
    /// what a private repository sends an anonymous client. mdeezl must fail
    /// with that reason instead of waiting for someone to type a password.
    #[test]
    fn test_unauthorised_repository_fails_without_prompting() {
        require_git!("test_unauthorised_repository_fails_without_prompting");
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        std::thread::spawn(move || {
            for conn in listener.incoming().flatten() {
                let mut conn = conn;
                let _ = conn.set_read_timeout(Some(std::time::Duration::from_secs(5)));
                // Read just enough of the request to answer it.
                let mut buf = [0u8; 1024];
                let _ = std::io::Read::read(&mut conn, &mut buf);
                let _ = std::io::Write::write_all(
                    &mut conn,
                    b"HTTP/1.1 401 Unauthorized\r\n\
                      WWW-Authenticate: Basic realm=\"mdeezl-test\"\r\n\
                      Content-Length: 0\r\n\r\n",
                );
            }
        });
        let base = tmp_dir("fetch-401");
        let started = std::time::Instant::now();
        let err = fetch(&format!("http://127.0.0.1:{port}/private.git"), &base)
            .err()
            .expect("an unauthorised fetch must fail")
            .to_string();
        // git's own words for the refusal differ by build: it either reports
        // that it cannot read a username with prompts disabled, or that
        // authentication failed. Either proves it gave up rather than asked.
        // Measured here: `could not read Username for 'http://127.0.0.1:PORT':
        // terminal prompts disabled`, in 0.25 s.
        let refused = ["terminal prompts disabled", "Authentication failed", "401"]
            .iter()
            .any(|needle| err.contains(needle));
        assert!(refused, "the reason must name the refusal: {err}");
        assert!(err.starts_with("could not fetch"), "{err}");
        assert!(
            started.elapsed() < std::time::Duration::from_secs(15),
            "it must fail promptly, not wait on a person"
        );
        assert!(is_empty_dir(&base));
    }

    /// T-002. Every git spawn goes through `git_command`, which sets
    /// `CREATE_NO_WINDOW` on Windows; a process made that way still runs and
    /// answers. `Command` does not expose creation flags, so a windowed
    /// caller's lack of a console flash is checked by hand.
    #[test]
    fn test_git_commands_set_create_no_window() {
        if !cfg!(windows) {
            eprintln!("SKIP test_git_commands_set_create_no_window: Windows only");
            return;
        }
        require_git!("test_git_commands_set_create_no_window");
        let out = git_command().arg("--version").output().unwrap();
        assert!(out.status.success());
        assert!(String::from_utf8_lossy(&out.stdout).starts_with("git version"));

        // A spawned process's creation flags cannot be read back from
        // outside it, so the two halves that make the promise true are
        // asserted on this file instead: the flag is set in `git_command`,
        // and `git_command` is the only place that spawns git. The needles
        // are built rather than written, so they cannot match this test.
        // Only the shipped half of the file: the tests below spawn git
        // themselves, for fixtures, and never reach a user's window.
        let source = include_str!("lib.rs");
        let shipped = source
            .split_once("\n#[cfg(test)]\nmod tests {")
            .expect("a tests module")
            .0;
        let spawn = format!("Command::new({:?})", "git");
        assert_eq!(
            shipped.matches(&spawn).count(),
            1,
            "git must be spawned in exactly one place, which carries the flag"
        );
        let body = source
            .split_once("\nfn git_command()")
            .expect("a git_command function")
            .1;
        let body = body.split_once("\nfn ").map_or(body, |(b, _)| b);
        let flag = format!("cmd.creation_flags({})", "CREATE_NO_WINDOW");
        assert!(
            body.contains(&flag),
            "git_command must apply CREATE_NO_WINDOW, so a windowed parent never flashes a console"
        );
    }
}
