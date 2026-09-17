# Sprint 0 Research Report

## Intents Reviewed
- [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md) — created; relevance: this sprint implements it end to end; current state: `proposed`, to be advanced by the Plan Phase.
- [INT-0002](../../../intents/INT-0002-git-and-remote-sources.md) — created; relevance: bounds the sprint by naming git/remote source selection as out of scope; current state: `proposed`, not scheduled.
- [INT-0003](../../../intents/INT-0003-llm-scaffold-comments.md) — created; relevance: bounds the sprint by naming model-generated tree comments as out of scope; current state: `proposed`, not scheduled.

## 1. Sprint Goal

Produce MDeezl as a working Rust binary that takes a directory path and writes
one Markdown document containing a box-drawing scaffold tree of that directory
followed by the text contents of every included file. The program targets the
Rust standard library only, with an empty `[dependencies]` table, and the
smallest implementation that satisfies INT-0001's acceptance criteria. Git ref
selection, remote providers, and model-generated tree comments are deliberately
excluded and recorded as INT-0002 and INT-0003.

## 2. Existing Code Survey

| File | Relevance | Notes |
|------|-----------|-------|
| MDeezl.md | high | Primary source of intent. Specifies the two output halves and records the Bash/awk and PowerShell one-liners that define the `--- File: <path> ---` content format. |
| Scaffolding symbols generator.md | high | Defines the exact tree symbols (U+251C, U+2500, U+2514, U+2502) and the single space before each name, plus the "surround with inline option" requirement for backtick and triple-backtick wrapping. Also the origin of INT-0002 and INT-0003. |
| README.md | medium | One-line project framing: feed any repository to text-only agents. Confirms the consumer is an LLM reading Markdown, not a human archiver. |
| .gitignore | medium | Already Rust-shaped (`target`, `**/*.rs.bk`, `*.pdb`, `mutants.out`). Confirms Rust was the intended language before this sprint, and that no Cargo project has been created yet. |
| LICENSE | low | Apache-2.0. No effect on design; noted so the Build Phase puts matching `license` metadata in `Cargo.toml`. |
| docs/work/remote-profile.md | low | Substrate, not product. Records provider `github`, base `main`, work `dev`, `human-approve` — the checkpoint at the end of this sprint is a PR for review, not an auto-merge. |

Nothing else exists: there is no Rust source, no `Cargo.toml`, and no prior
sprint or failure report. The survey is six files because the repository is six
files; the twenty-file cap is not approached.

## 3. External Sources

- [CommonMark Spec 0.31.2 — Fenced code blocks](https://spec.commonmark.org/0.31.2/) — Verified the two rules that govern the fence wrap mode: a fence is at least three backticks, and the closing fence must be at least as long as the opening one, while backticks are legal inside the block. This makes a variable-length opening fence a correct solution rather than a guess.
- [std::fs::read_dir](https://doc.rust-lang.org/std/fs/fn.read_dir.html) — The traversal primitive. Yields entries in unspecified order, which is why the plan must sort each level explicitly for deterministic output.
- [std::fs::DirEntry](https://doc.rust-lang.org/std/fs/struct.DirEntry.html) — `DirEntry::file_type` does not follow symbolic links, so it is the correct way to classify entries without risking a link cycle; `Path::is_dir` follows links and is therefore the wrong choice here.
- [String::from_utf8](https://doc.rust-lang.org/std/string/struct.String.html#method.from_utf8) — Gives read-bytes-then-validate as the binary-detection mechanism, with no extra crate and no heuristic byte sniffing.
- [Unicode Box Drawing (U+2500 to U+257F)](https://www.unicode.org/charts/PDF/U2500.pdf) — Confirms the code points behind the symbols the project document specifies.

## 4. Risks, Unknowns, Dependencies

- **Risk: unbounded output.** With no gitignore support, pointing MDeezl at a
  repository containing `target/` or `node_modules/` produces an enormous
  document. Mitigation for this sprint: `.git` is always skipped and hidden
  entries are skipped by default; the general case is an accepted consequence
  recorded in INT-0001 and a candidate for a follow-on intent.
- **Risk: a file terminating its own code fence.** A repository full of Markdown
  — including this one — contains triple-backtick runs. A fixed three-backtick
  wrapper would corrupt the document. Mitigation: compute the longest backtick
  run per file and open with one more, per the CommonMark rule verified above.
- **Risk: symlink cycles.** A self-referential link would hang a naive recursive
  walk. Mitigation: classify with `DirEntry::file_type`, which does not follow
  links, and never recurse into a link.
- **Risk: Windows path separators leaking into output.** The build host here and
  a Linux CI host differ. Mitigation: build relative display paths from
  components joined with a forward slash rather than printing a `Path` directly,
  and assert it in a test.
- **Unknown: how a permission-denied directory should read in the output.** A
  `read_dir` failure mid-walk must not abort the whole document. The Plan Phase
  should choose: emit an inline error marker for that subtree and continue.
- **Unknown: whether the wrap mode should default to fenced or unwrapped.**
  Fencing is friendlier to a human reading the document; no wrapping most
  closely reproduces the shell one-liners in `MDeezl.md`. Resolve in the Plan
  Phase.
- **Dependency: Rust toolchain.** `cargo 1.96.0` and `rustc 1.96.0` are
  installed and verified on this machine. No third-party crates are required, so
  there is no registry dependency at build time.
- **Dependency: CI.** Substrate convergence generated no CI workflow because the
  language detector found no source files yet. Once `Cargo.toml` exists, CI
  scaffolding must be re-run, or the sprint checkpoint is green merely because
  nothing ran.

## 5. Recommended Approach

Primary: a single-crate binary with no third-party dependencies, structured as
one traversal that collects an in-memory tree of entries, then two renderers
over that tree — one emitting the box-drawing scaffold, one emitting the
`--- File: <path> ---` content sections. Argument parsing is a hand-written loop
over `std::env::args` covering the positional path, an output flag, a wrap-mode
flag, a hidden-entries flag, and help. Binary detection is `fs::read` followed
by `String::from_utf8`. Output goes through a single buffered writer.

Alternative considered: two independent walks, one per output half. Marginally
simpler per function, but it reads the directory twice and can disagree between
the tree and the dump if the filesystem changes mid-run. Also considered and
rejected for this sprint: `walkdir` plus `clap` plus `ignore`, which would be
less code to write but contradicts the stated zero-dependency goal.

Rationale: collecting once and rendering twice guarantees the tree and the
content sections describe the same set of files, which is the property an agent
consuming the document actually relies on. Every primitive required — recursive
reads, UTF-8 validation, path handling, buffered output — is in the standard
library, so the dependency-free constraint costs very little code.

## Artifacts
- `docs/intents/INT-0001-markdown-repo-context-bundle.md` — the intent this sprint realizes.
- `docs/intents/INT-0002-git-and-remote-sources.md` — deferred scope, recorded to bound the sprint.
- `docs/intents/INT-0003-llm-scaffold-comments.md` — deferred scope, recorded to bound the sprint.

No code snippets or error traces were produced during research; the repository
contained no code to reproduce a failure from.
