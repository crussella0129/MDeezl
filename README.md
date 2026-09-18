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
mdeezl [PATH] [-o FILE] [--wrap MODE] [--exclude PATTERN]... [--include PATTERN]...
```

`PATH` defaults to the current directory. Output goes to stdout unless `-o` is
given.

| Flag | Meaning |
|------|---------|
| `-o`, `--output FILE` | write the document to `FILE` instead of stdout |
| `--wrap MODE` | `fence` (default), `inline`, or `none` |
| `--exclude PATTERN` | add `PATTERN` to the ignore list (repeatable) |
| `--include PATTERN` | keep entries matching `PATTERN` even if ignored (repeatable) |
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

## The ignore list

One list decides what is omitted, and it governs the tree and the contents
identically — the two halves can never disagree. It comes pre-populated with:

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
  tree produce byte-identical output.
- **Symlink-safe.** Links are listed but never followed, so a link cycle cannot
  hang the walk.
- **Degrades in place.** A non-UTF-8 file becomes
  `[binary file, N bytes elided]`; an unreadable file or directory is marked
  where it sits. Neither aborts the document.
- **Portable paths.** Output always uses `/`, including on Windows.
- **No gitignore support yet.** Add a `--exclude` for any build directory not
  in the list above.

## Not yet

Bundling a git branch, worktree, or a remote repository on GitHub/GitLab/Gitea/
Codeberg, and LLM-generated comments in the tree, are both intended but out of
scope for now. They are written up as `docs/intents/INT-0002` and `INT-0003`.

## License

Apache-2.0.
