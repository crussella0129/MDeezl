# INT-0001 — Markdown repository context bundle

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0001
- **State:** proposed
- **Work evidence:** none
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent

MDeezl is a command-line program that takes one directory path and emits a
single Markdown document that lets a text-only agent understand a repository
without filesystem access. The document has two parts, in this order:

1. a **scaffold tree** of the directory, drawn with the Unicode Box-Drawing
   symbols described in `Scaffolding symbols generator.md`; and
2. a **content dump** of every included file, each introduced by a
   `--- File: <path> ---` header, reproducing the behaviour the `find`/`awk`
   and PowerShell one-liners in `MDeezl.md` produce today.

The program is written in Rust against the standard library. Minimality is part
of the intent, not an implementation detail: the binary must build with an empty
`[dependencies]` table, and the implementation should stay small enough to read
in one sitting.

**Non-goals for this intent:** selecting a git branch, worktree, or remote
provider as the source (see [INT-0002](INT-0002-git-and-remote-sources.md));
LLM-generated per-entry comments in the tree (see
[INT-0003](INT-0003-llm-scaffold-comments.md)); interactive or streaming output;
token budgeting or truncation heuristics.

## Acceptance criteria

- `mdeezl <path>` writes a Markdown document to stdout; `mdeezl` with no path
  operates on the current directory.
- The tree section renders `├── ` for a non-final entry, `└── ` for the final
  entry at a level, and `│   ` for continuation, each branch symbol followed by
  a single space before the name. Directory names carry a trailing `/`.
- Directory traversal is deterministic: entries at each level are sorted by
  name, and the same input tree always produces byte-identical output.
- `.git` is always excluded. Other dot-prefixed entries are excluded by default
  and included with `--hidden`.
- Symbolic links are listed but never traversed, so a link cycle cannot hang or
  duplicate output.
- A file whose bytes are not valid UTF-8 is listed in the tree and in the
  content section, but its body is replaced by an explicit elision marker rather
  than emitted as mojibake.
- `--wrap <none|inline|fence>` controls how a file body is surrounded. In
  `fence` mode the opening fence is longer than the longest backtick run in that
  file's body, so a file that itself contains ``` cannot terminate its own block.
- `-o <file>` writes the document to that file instead of stdout.
- Paths in the output use `/` separators and are relative to the scanned root on
  every platform, including Windows.
- `cargo build` succeeds with no third-party crates in `Cargo.toml`.

## Rationale

The two shell one-liners recorded in `MDeezl.md` already do roughly the right
thing, but they are platform-split (one Bash, one PowerShell), they differ in
what they exclude (`*/.*` versus `\.git\`), the PowerShell variant emits
absolute paths, and neither produces the scaffold tree. A single small binary
removes the platform split, makes exclusion rules identical everywhere, and lets
the tree and the dump come from one traversal.

Rust with only the standard library is chosen because every capability this
intent needs — recursive directory reads, UTF-8 validation, path handling,
buffered writing — is already in `std`. Adding a walker or an argument parser
would trade a readable single-purpose program for a dependency tree that a
context-bundling tool does not need.

## Alternatives

- **Keep the shell one-liners.** Rejected: no tree output, no shared exclusion
  rules, and the PowerShell form re-opens the output file per line.
- **Rust with `walkdir`, `clap`, and `ignore`.** Rejected for this intent: it
  would be less code to write but contradicts the stated minimal-dependency
  goal, and `ignore`'s gitignore semantics are a larger behaviour than the
  intent asks for.
- **Respect `.gitignore` by default.** Deferred, not rejected. It is a real
  improvement for large repos but requires either a dependency or a nontrivial
  pattern matcher; it belongs to its own intent once the base tool exists.
- **Emit JSON and render Markdown separately.** Rejected: the consumer is a
  text-only agent reading Markdown, so a second format is unused surface.

## Consequences

- Without gitignore support, scanning a repository that has a populated
  `target/` or `node_modules/` produces a very large document. Users must scope
  the path or delete such directories first until a later intent addresses it.
- Hand-rolled argument parsing means the flag surface must stay small; every new
  flag is written by hand rather than declared.
- Holding no dependency on a walker means symlink and permission-error handling
  are this project's responsibility and must be covered by tests.
- Binary files are represented but not reproduced, so the output is not a
  lossless archive and must not be used as one.

## Transition history
- 2026-09-17: created as `proposed`.
