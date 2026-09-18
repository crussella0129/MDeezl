//! MDeezl — feed any repository to text-only agents.
//!
//! Emits one Markdown document for a directory: a box-drawing scaffold tree,
//! then the contents of every included file. Standard library only; see
//! `docs/intents/INT-0001-markdown-repo-context-bundle.md`.

use std::env;
use std::io;
use std::path::PathBuf;
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

fn run(_opts: &Options) -> io::Result<()> {
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

    #[test]
    fn test_manifest_declares_bin_name_and_license() {
        let manifest = include_str!("../Cargo.toml");
        // The bin name is load-bearing: CARGO_BIN_EXE_mdeezl resolves by it.
        assert!(manifest.contains("name = \"mdeezl\""));
        assert!(manifest.contains("license = \"Apache-2.0\""));
    }
}
