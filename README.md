# MDeezl

Feed any repository to text-only agents.

MDeezl points at a directory and emits **one Markdown document** containing
everything an agent needs to understand it without filesystem access:

1. a **scaffold tree** of the directory, drawn with Unicode box-drawing symbols;
2. the **full text of every included file**, each introduced by a
   `--- File: <path> ---` header.

It is written in Rust against the standard library alone. `cargo tree` reports
no dependencies, and the test suite adds none.

## Install

```bash
cargo install --path .
```

Or run it straight from the checkout with `cargo run -- <args>`.

## Usage

```
mdeezl [PATH] [-o FILE] [--wrap MODE] [--exclude PATTERN]... [--include PATTERN]... [--no-gitignore]
```

`PATH` defaults to the current directory. Output goes to stdout unless `-o` is
given.

| Flag | Meaning |
|------|---------|
| `-o`, `--output FILE` | write the document to `FILE` instead of stdout |
| `--wrap MODE` | `fence` (default), `inline`, or `none` |
| `--exclude PATTERN` | add `PATTERN` to the ignore list (repeatable) |
| `--include PATTERN` | keep entries matching `PATTERN` even if ignored (repeatable) |
| `--no-gitignore` | do not apply the repository's `.gitignore` rules |
| `-h`, `--help` | print usage |

Exit codes: `0` success, `2` bad arguments, `1` the path could not be read or
the output could not be written. A non-zero exit never leaves a partial
document on stdout.

## Example

```bash
mdeezl . -o context.md
```

Which produces (this block is fenced with four backticks so the inner fences
show — the same trick MDeezl itself uses):

````
# MDeezl

## Structure

```
MDeezl/
├── Cargo.toml
├── README.md
└── src/
    └── main.rs
```

## Contents

---
File: Cargo.toml
---

```toml
[package]
name = "mdeezl"
```
````

## What is omitted

Two sources decide what is omitted, and `--include` outranks both:

1. **the ignore list** — a short built-in list you can extend and override;
2. **the repository's own `.gitignore` rules**, applied by asking git.

Both govern the tree and the contents identically, so the two halves of the
document can never disagree.

### The ignore list

It comes pre-populated with:

```
.*   target   node_modules   dist   build   __pycache__
```

`.*` covers `.git`, `.venv`, and every other dot-entry.

`--exclude` adds a pattern; `--include` cancels one. **An entry is omitted when
it matches an ignore pattern and matches no include pattern — include always
wins.** That is what makes this work:

```bash
mdeezl . --include .github        # dotfiles stay out, but .github comes back
```

### Pattern forms

There is no glob engine. A pattern is one of four things:

| Form | Selects | Example |
|------|---------|---------|
| `.*` | any name beginning with a dot | hidden entries |
| `*.ext` | **a whole file type** | `--exclude "*.png"` |
| contains `/` | **one specific file**, by path relative to `PATH` | `--exclude docs/big.csv` |
| anything else | that exact name, file or directory, at any depth | `--exclude target` |

> In Bash, quote a pattern containing `*` or the shell expands it before MDeezl
> sees it: `--exclude '*.png'`. PowerShell passes it through unchanged.

Note that `--include ".*"` re-admits `.git` along with everything else hidden.
Git's object store is mostly binary and so gets elided rather than dumped, but
the tree gets noisy — name the entries you want instead.

### The repository's `.gitignore`

Point MDeezl at an unfamiliar repository and it omits what that repository
already tells git to ignore, so you don't have to find its build directory
first. It does this by asking git itself — one `git check-ignore` process per
run — so **the full gitignore syntax works**: `**`, negation, directory-only
rules, nested `.gitignore` files, `.git/info/exclude`, and `core.excludesFile`.
No crate is involved; `cargo tree` still shows nothing.

- **It needs git.** Git must be on your `PATH`, and the scanned directory must
  be inside a git work tree. When either is missing — or the query fails — MDeezl
  still produces the whole document with the ignore list alone, and prints one
  line on stderr saying gitignore filtering was skipped and why. A plain
  directory is always a valid input.
- **Turn it off** with `--no-gitignore`, when you want exactly the files git is
  told to forget.
- **Tracked files are kept**, even when a rule matches them. That is git's own
  rule, and a file the repository actually keeps belongs in the bundle.
- **Including a gitignored directory brings back all of it.** Git reports every
  file inside an ignored directory as ignored, so `--include out` restores
  `out/` *and its contents*. This applies only when the directory is itself
  gitignored: `--include build`, used just to undo the ignore list, leaves the
  repository's rules in force inside `build/`.
- **An include cannot reach inside a pruned directory.** `--include "*.o"`
  restores a gitignored `top.o`, but not `out/a.o` when `out/` is gitignored —
  the ignore list behaves the same way, and so does git. Include `out` instead.
- **Scanning an ignored directory directly** — `mdeezl out/` — skips gitignore
  filtering with a notice, rather than printing an empty tree.
- **Nested repositories and submodules** are not gitignore-filtered; their
  contents keep the ignore list only.
- **Names containing `*`, `?`, `[` or `\`** are matched only partially by the
  rules. File globs such as `*.log` still match them, and tracked files like
  `pages/[id].tsx` are always kept. But a rule that names such an entry
  exactly, or a directory-only rule for such a directory, may not match. One
  consequence: an untracked file in a directory like `app/[slug]/` can be lost
  even under an `--include` that names it. Include the directory instead.

## Wrap modes

| Mode | Tree | File bodies |
|------|------|-------------|
| `fence` (default) | one fenced block | fenced, with a language hint |
| `inline` | each line wrapped in single backticks | fenced — a multi-line body cannot be inline |
| `none` | bare: no fences, no backticks | bare |

In `fence` mode the opening fence of each body is made **longer than the longest
backtick run inside that file**, so a Markdown file containing ``` cannot
terminate its own block. Running MDeezl on its own repository is the proof: the
sprint plans contain four-backtick runs, so those bodies open with five.

## Behaviour worth knowing

- **Deterministic.** Entries are sorted per level; two runs over an unchanged
  tree, with the same git installation and ignore rules, produce byte-identical
  output.
- **Symlink-safe.** Links are listed but never followed, so a link cycle cannot
  hang the walk.
- **Degrades in place.** A non-UTF-8 file becomes
  `[binary file, N bytes elided]`; an unreadable file or directory is marked
  where it sits. Neither aborts the document.
- **Portable paths.** Output always uses `/`, including on Windows.

## Not yet

Bundling a git branch, worktree, or a remote repository on GitHub/GitLab/Gitea/
Codeberg, and LLM-generated comments in the tree, are both intended but out of
scope for now. They are written up as `docs/intents/INT-0002` and `INT-0003`.

## License

Apache-2.0.
