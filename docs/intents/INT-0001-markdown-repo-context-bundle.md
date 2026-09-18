# INT-0001 — Markdown repository context bundle

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0001
- **State:** active
- **Work evidence:** [Sprint 0 build plan](../sprints/s0/sprint-plans/build-plan.md), [T-002 ignore and include resolution](../sprints/s0/sprint-plans/build-plan.md#t-002-filesystem-walk-with-ignore-and-include-resolution)
- **Completion evidence:** none
- **Code evidence:** [src/main.rs](../../src/main.rs), [Cargo.toml](../../Cargo.toml)
- **Test evidence:** [Sprint 0 test report](../sprints/s0/sprint-tests/test-report.md), [unit](../sprints/s0/sprint-tests/unit-tests.md), [integration](../sprints/s0/sprint-tests/integration-tests.md), [end-to-end](../sprints/s0/sprint-tests/e2e-tests.md)
- **Documentation evidence:** [README.md](../../README.md)

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
token budgeting or truncation heuristics; gitignore parsing.

## Acceptance criteria

- `mdeezl <path>` writes a Markdown document to stdout; `mdeezl` with no path
  operates on the current directory.
- The tree section renders `├── ` for a non-final entry, `└── ` for the final
  entry at a level, and `│   ` for continuation, each branch symbol followed by
  a single space before the name. Directory names carry a trailing `/`.
- Directory traversal is deterministic: entries at each level are sorted by
  name, and the same input tree always produces byte-identical output.
- **A single ignore list governs what is omitted, and it governs the tree and
  the content dump identically**, so the two halves can never describe different
  file sets. The list is pre-populated with `.*`, `target`, `node_modules`,
  `dist`, `build`, and `__pycache__`.
- **`--exclude <pattern>` appends to that list and `--include <pattern>` cancels
  an entry from it.** An entry is omitted when it matches an ignore pattern and
  matches no include pattern; include always wins. Both flags are repeatable.
- **A pattern selects by whole file type, by individual file, or by name:**
  `.*` matches any name beginning with a dot; `*.ext` matches every file of that
  extension; a pattern containing `/` matches that exact path relative to the
  scan root; any other pattern matches that exact name, file or directory, at
  any depth.
- Symbolic links are listed but never traversed, so a link cycle cannot hang or
  duplicate output.
- A file whose bytes are not valid UTF-8 is listed in the tree and in the
  content section, but its body is replaced by an explicit elision marker rather
  than emitted as mojibake. A file that cannot be read, and a directory that
  cannot be listed, are likewise marked in place; neither aborts the document.
- `--wrap <fence|inline|none>` controls how output is surrounded and defaults to
  `fence`. In `fence` mode the opening fence of a file body is longer than the
  longest backtick run in that body, so a file that itself contains a fence
  cannot terminate its own block. In `inline` mode each tree line is wrapped in
  single backticks, as `Scaffolding symbols generator.md` asks; file bodies
  cannot be inline and remain fenced. In `none` mode nothing is surrounded at
  all: the tree carries no fence and no backticks, and bodies carry no fence,
  reproducing the shell one-liners' raw output.
- Wherever a body is fenced — `fence` mode, and `inline` mode, whose bodies are
  also fenced — a file whose extension is in a small fixed table carries the
  corresponding language hint as the fence info string; an unrecognized
  extension is not an error and simply yields no info string.
- `-o <file>` writes the document to that file instead of stdout.
- Paths in the output use `/` separators and are relative to the scanned root on
  every platform, including Windows.
- The emitted document consists of a `# <root name>` title, a `## Structure`
  section holding the scaffold, and a `## Contents` section holding the file
  sections, in that order.
- Invalid usage is rejected rather than silently accepted: the process exits 0
  on success, 2 on an argument error, and 1 when the root cannot be read or the
  output cannot be written. A non-zero exit never leaves a partial document on
  stdout.
- `-h`/`--help` prints usage documenting the four pattern forms, the fact that
  Bash expands an unquoted `*.ext` before MDeezl sees it, and the fact that file
  bodies cannot be inline.
- `README.md` documents the same surface as `--help` — the flags, the four
  pattern forms, the pre-populated ignore list, and the three wrap modes — so
  that surface is discoverable before the binary is built.
- Every file section is separated from what precedes it by a blank line, exactly
  as the `awk` one-liner's `"\n---\nFile: …"` does. Without it, in `none` mode
  the header's opening `---` would underline the previous file's last line
  instead of opening a new section.
- `cargo build` succeeds with no third-party crates in `Cargo.toml`, and the
  test suite adds none.
- Verification runs on both Linux and Windows, because two of the properties
  above — symlinks are never traversed, and paths use `/` on every platform —
  cannot both be exercised on a single operating system.

## Rationale

The two shell one-liners recorded in `MDeezl.md` already do roughly the right
thing, but they are platform-split (one Bash, one PowerShell), they differ in
what they exclude (`*/.*` versus `\.git\`), the PowerShell variant emits
absolute paths and reopens the output file once per line, and neither produces
the scaffold tree. A single small binary removes the platform split, makes
exclusion rules identical everywhere, and lets the tree and the dump come from
one traversal.

Rust with only the standard library is chosen because every capability this
intent needs — recursive directory reads, UTF-8 validation, path handling,
buffered writing — is already in `std`. Adding a walker or an argument parser
would trade a readable single-purpose program for a dependency tree that a
context-bundling tool does not need.

Exclusion is expressed as one editable list rather than a set of fixed flags
because the three things a user actually wants to drop — a whole file type, one
particular file, and a build directory — are the same operation at different
granularities. One list with one precedence rule expresses all three and stays
inspectable: what is omitted is always the pre-populated list plus `--exclude`
minus `--include`, with no second hidden mechanism layered on top.

## Alternatives

- **Keep the shell one-liners.** Rejected: no tree output, no shared exclusion
  rules, and the PowerShell form re-opens the output file per line.
- **Rust with `walkdir`, `clap`, and `ignore`.** Rejected for this intent: it
  would be less code to write but contradicts the stated minimal-dependency
  goal, and `ignore`'s gitignore semantics are a larger behaviour than the
  intent asks for.
- **A separate `--hidden` flag alongside the ignore list.** Rejected: it is a
  second mechanism for something the list already expresses as the `.*` pattern,
  and two mechanisms means two precedence rules to reason about.
- **Treating `--include` as list subtraction by exact pattern text** rather than
  as a match that outranks an ignore. Rejected: subtraction cannot express
  "hidden entries stay out, but `.github` comes back", which is the common case.
- **Defaulting `--wrap` to `none`.** Rejected, though it is the closer
  reproduction of the shell one-liners in `MDeezl.md`. Unwrapped output puts the
  raw text of arbitrary files — most of which, in a documentation-heavy
  repository, is itself Markdown — directly into a Markdown document, where its
  headings and lists merge into the surrounding structure. Fenced output keeps
  each file's boundaries unambiguous to the agent consuming it, which is the
  point of the tool; `none` remains available for anyone who wants the
  one-liner's exact bytes.
- **Streaming each file to the sink as it is read**, rather than assembling the
  document first. Rejected: it cannot satisfy the criterion that a non-zero exit
  leaves no partial document on stdout. The cost is recorded in Consequences.
- **Respect `.gitignore` by default.** Deferred, not rejected. It is a real
  improvement for large repos but requires either a dependency or a nontrivial
  pattern matcher; it belongs to its own intent once the base tool exists.
- **Emit JSON and render Markdown separately.** Rejected: the consumer is a
  text-only agent reading Markdown, so a second format is unused surface.

## Consequences

- Because include outranks ignore, `--include ".*"` re-admits `.git` along with
  every other dot-entry. Git's object store is mostly binary and is therefore
  elided rather than dumped, so the cost is a noisy tree rather than an enormous
  document — but the tree is noisy, and a user who wants dotfiles without git
  history should name the entries they want instead.
- Without gitignore support, a repository whose build directory is not in the
  pre-populated list still produces a very large document. Users must add a
  `--exclude` for it until a later intent addresses gitignore properly.
- Hand-rolled argument parsing means the flag surface must stay small; every new
  flag is written by hand rather than declared.
- Pattern matching without a glob engine means only the four documented forms
  work. `src/**/*.rs` is not a pattern, and users will occasionally expect it to
  be.
- On Bash, an `*.ext` pattern must be quoted or the shell expands it before
  MDeezl sees it. This is a standard shell footgun, but it is one users will hit.
- Holding no dependency on a walker means symlink and permission-error handling
  are this project's responsibility and must be covered by tests.
- Binary files are represented but not reproduced, so the output is not a
  lossless archive and must not be used as one.
- **The inherited `---` / `File: <path>` / `---` header is a setext heading, not
  two thematic breaks.** Under CommonMark, a `---` line directly beneath a
  paragraph line makes that line an `<h2>`. The header is kept verbatim anyway,
  because byte-compatibility with the `awk` one-liner's output is an explicit
  criterion above and because rendering each file path as a heading is useful
  rather than harmful. It is only well-formed because of the blank-line
  separator criterion above: the header's *first* `---` follows a blank line and
  is therefore a thematic break, while the second follows a paragraph line and
  is an underline. The consequence is that file sections are siblings of
  `## Structure` and `## Contents` in the heading hierarchy rather than children
  of the latter, and that assertions about document structure must match exact
  lines rather than reason about heading levels.
- **Peak memory is proportional to the total included text.** The document is
  assembled in memory before the sink is opened, which is what makes "a non-zero
  exit never leaves a partial document on stdout" achievable. Combined with the
  absence of gitignore support, a large repository is held in memory in full.
  This is accepted at the intended scale — repositories a person means to hand
  to an agent — and is the first thing to revisit if that stops holding.
- Because two acceptance criteria are platform-specific in opposite directions,
  CI must run a two-OS matrix. A single-OS pipeline would silently skip the
  symlink guarantee on Windows or the path-separator guarantee on Linux.
- The fixed extension-to-language table is a small maintenance surface: it will
  never be complete, and adding an entry is a deliberate edit rather than a
  configuration option.

## Transition history
- 2026-09-17: created as `proposed`.
- 2026-09-17: `proposed → planned` for sprint 0. Acceptance criteria, rationale,
  alternatives, and consequences amended before planning to replace the drafted
  `--hidden` flag and the "`.git` is always excluded" rule with the single
  ignore list plus `--exclude`/`--include`, and to add the `inline` wrap mode
  the source document asks for. Work evidence attached to the sprint 0 build
  plan.
- 2026-09-17: amended again in response to the plan critique, while still
  `planned`. Added the acceptance criteria the plans had assumed but the chapter
  never authorized — document skeleton, exit codes, `--help` contents, fence
  language hints, and two-OS verification — and recorded four consequences and
  two rejected alternatives that had existed only in sprint prose: the setext
  reading of the inherited `File:` header, whole-document in-memory assembly,
  the two-OS matrix requirement, and the extension-table maintenance surface.
  No criterion was weakened; the chapter was widened to cover behaviour the
  sprint was already committing to.
- 2026-09-17: third amendment, still `planned`, from the second critique round.
  Added the `README.md` documentation criterion, which a whole build task
  depended on while the chapter authorized nothing, and the blank-line section
  separator criterion. The separator had been dropped from the inherited `awk`
  format; without it, `--wrap none` makes the next section's opening `---`
  underline the previous file's last line, silently destroying the header. The
  setext consequence was corrected to depend on that separator explicitly.
- 2026-09-17: fourth amendment, still `planned`, from the third critique round.
  The language-hint criterion was scoped to `fence` mode alone and now reads
  "wherever a body is fenced", because `inline` mode fences bodies too and the
  narrower wording would have made byte-identical output conformant in one mode
  and non-conformant in the other. This widens the criterion to match behaviour
  the plans already described; it weakens nothing.
- 2026-09-17: fifth amendment, still `planned`, from the fourth critique round.
  Spelled out `none`-mode semantics — no fence and no backticks on the tree, no
  fence on bodies — which had appeared only as an enum value in the wrap
  criterion while the plans asserted and tested the behaviour in detail. Same
  correction as the previous entry, applied to the remaining mode.
- 2026-09-17: `planned → active`; sprint 0 Build Phase began implementing
  T-001 through T-007 against this chapter. Work evidence unchanged.
- 2026-09-17: Test Phase evidence attached while `active`. 72 tests pass at
  `2e48a1c` with `cargo fmt`, `cargo clippy -D warnings`, and a zero-dependency
  `cargo tree` clean. The chapter stays `active` rather than moving to
  `realized`: the "verification runs on both Linux and Windows" criterion is
  unverified because CI has never run on this branch, and the test report
  records that as a known gap.
- 2026-09-17: sprint 0 closed with the chapter still `active`, not `realized`.
  Every acceptance criterion but one is proved by the sprint 0 test report; the
  exception, "verification runs on both Linux and Windows", cannot be verified
  before the checkpoint exists, because the checkpoint is what first pushes the
  branch and triggers CI. Carried forward as T-101. The gitignore gap recorded
  in Alternatives and Consequences was promoted to
  [INT-0004](INT-0004-gitignore-aware-exclusion.md).
