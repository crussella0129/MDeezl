Finalized - DO NOT EDIT

# Sprint 0 Build Plan

## Intents
- [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md) — state: planned; acceptance criteria covered: all. The sprint implements the intent end to end: CLI surface and `--help`, ignore list with `--exclude`/`--include`, deterministic symlink-safe traversal, scaffold tree, content dump with elision, wrap modes and fence language hints, `-o`, document skeleton, exit codes, platform-independent `/` paths, two-OS verification, and the empty `[dependencies]` constraint.
- [INT-0002](../../../intents/INT-0002-git-and-remote-sources.md) — state: proposed; not advanced by this sprint. Recorded to bound scope: no task reads a git ref or a remote.
- [INT-0003](../../../intents/INT-0003-llm-scaffold-comments.md) — state: proposed; not advanced by this sprint. Recorded to bound scope: no task emits tree comments or calls a model.

## Schema Tree
- Sprint Goal: a std-only Rust binary that emits one Markdown context bundle
  - Crate and CLI
    - T-001: Cargo skeleton, shared types, and argument parsing
  - Traversal
    - T-002: Filesystem walk with ignore and include resolution
  - Rendering
    - T-003: Scaffold tree renderer
    - T-004: Content section renderer
  - Delivery
    - T-005: Output sink and process exit codes
    - T-006: CI regeneration on a two-OS matrix
    - T-007: User documentation

## Execution Sequence

### T-001: Cargo skeleton, shared types, and argument parsing

- **Intent:** [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md)
- **Touches:** `Cargo.toml`, `src/main.rs`
- **Depends on:** (none)
- **Acceptance criterion:** "`mdeezl <path>` writes a Markdown document to stdout; `mdeezl` with no path operates on the current directory"; "The list is pre-populated with `.*`, `target`, `node_modules`, `dist`, `build`, and `__pycache__`"; "`--exclude <pattern>` appends to that list and `--include <pattern>` cancels an entry from it … Both flags are repeatable"; "`-h`/`--help` prints usage documenting the four pattern forms, the fact that Bash expands an unquoted `*.ext` before MDeezl sees it, and the fact that file bodies cannot be inline"; "`cargo build` succeeds with no third-party crates in `Cargo.toml`, and the test suite adds none."
- **Success criterion (EARS):**
  - **WHEN** `Cargo.toml` is read by Cargo, **THEN** the manifest **SHALL** declare an empty `[dependencies]` table, a binary target named `mdeezl`, and `license = "Apache-2.0"` matching the repository `LICENSE`.
  - **WHEN** `parse_args` receives no arguments, **THEN** it **SHALL** return options whose root is `.`, whose output sink is stdout, whose wrap mode is `Fence`, and whose ignore list is exactly `[".*", "target", "node_modules", "dist", "build", "__pycache__"]`.
  - **WHEN** `parse_args` receives a bare positional argument, **THEN** it **SHALL** use it as the scan root.
  - **WHEN** `parse_args` receives `--exclude` or `--include` more than once, **THEN** every occurrence **SHALL** be retained in declaration order, so both flags are repeatable.
  - **WHEN** `parse_args` receives `--wrap` followed by a value that is not `fence`, `inline`, or `none`, **THEN** it **SHALL** return a usage error naming the offending value.
  - **WHEN** `parse_args` receives any of `-o`, `--wrap`, `--exclude`, or `--include` as the final argument with no value following, **THEN** it **SHALL** return a usage error naming that flag.
  - **WHEN** `parse_args` receives a second positional argument, **THEN** it **SHALL** return a usage error rather than silently ignoring either path.
  - **WHEN** `parse_args` receives `-h` or `--help`, **THEN** it **SHALL** return a help request whose text documents the four pattern forms and states both that Bash expands an unquoted `*.ext` and that file bodies cannot be inline.
  - **WHEN** `cargo tree` is run against the crate, **THEN** it **SHALL** report no dependencies beyond the crate itself, confirming at the toolchain level what the manifest clause asserts textually.
- **Notes:** this task **defines the shared types the later tasks consume** — `Options`, the three-variant `Wrap` enum, and the `Sink` representation whose behaviour T-005 implements. Declaring them here is what lets `parse_args` and its tests exist before the sink does; T-003, T-004, and T-005 depend on this task for those symbols. Parsing is a hand-written loop over an iterator of `String`, so tests call it directly without touching the process environment; no argument-parsing crate is added.

### T-002: Filesystem walk with ignore and include resolution

- **Intent:** [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md)
- **Touches:** `src/main.rs`
- **Depends on:** T-001
- **Acceptance criterion:** "A single ignore list governs what is omitted, and it governs the tree and the content dump identically"; "An entry is omitted when it matches an ignore pattern and matches no include pattern; include always wins"; "A pattern selects by whole file type, by individual file, or by name"; "Directory traversal is deterministic"; "Symbolic links are listed but never traversed"; "a directory that cannot be listed [is] marked in place; neither aborts the document."
- **Success criterion (EARS):**
  - **WHEN** `matches` is called with the pattern `.*` and an entry name beginning with `.`, **THEN** it **SHALL** return true, and **WHEN** called with a name not beginning with `.`, it **SHALL** return false.
  - **WHEN** `matches` is called with a pattern beginning `*.`, **THEN** it **SHALL** return true only for an entry name ending in the remainder of that pattern.
  - **WHEN** `matches` is called with a pattern containing `/`, **THEN** it **SHALL** compare the pattern against the entry's root-relative path and **SHALL NOT** compare it against the entry name.
  - **WHEN** `matches` is called with any other pattern, **THEN** it **SHALL** return true only for an entry whose name equals the pattern exactly, at any depth.
  - **WHEN** an entry matches at least one ignore pattern and no include pattern, **THEN** `is_excluded` **SHALL** return true.
  - **WHEN** an entry matches both an ignore pattern and an include pattern, **THEN** `is_excluded` **SHALL** return false, so that include outranks ignore.
  - **WHEN** `walk` reads a directory, **THEN** it **SHALL** emit that directory's children sorted by name, so two runs over an unchanged tree produce byte-identical output.
  - **WHEN** `walk` encounters an entry whose `DirEntry::file_type` reports a symbolic link, **THEN** it **SHALL** record the entry and **SHALL NOT** descend into it.
  - **WHEN** `read_dir` fails for a directory during the walk, **THEN** `walk` **SHALL** record that directory as unreadable and **SHALL** continue traversing the remainder of the tree.
  - **WHEN** the scan root does not exist or is not a directory, **THEN** the walk **SHALL** return an error rather than an empty tree.
- **Notes:** one walk builds an in-memory `Node` tree consumed by both renderers, so the scaffold and the dump cannot disagree. Classify with `DirEntry::file_type`, never `Path::is_dir`, which follows links. Root-relative paths are built by joining components with `/`.

### T-003: Scaffold tree renderer

- **Intent:** [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md)
- **Touches:** `src/main.rs`
- **Depends on:** T-001, T-002
- **Acceptance criterion:** "The tree section renders `├── ` for a non-final entry, `└── ` for the final entry at a level, and `│   ` for continuation, each branch symbol followed by a single space before the name. Directory names carry a trailing `/`"; "In `inline` mode each tree line is wrapped in single backticks."
- **Success criterion (EARS):**
  - **WHEN** `render_tree` emits a child that is not the last at its level, **THEN** the line **SHALL** begin with that level's prefix followed by `├── `.
  - **WHEN** `render_tree` emits the last child at its level, **THEN** the line **SHALL** begin with that level's prefix followed by `└── `.
  - **WHEN** `render_tree` descends past a non-last entry, **THEN** the child prefix **SHALL** be extended with `│   `, and **WHEN** it descends past a last entry, the child prefix **SHALL** be extended with four spaces.
  - **WHEN** `render_tree` emits a directory node, **THEN** the rendered name **SHALL** carry a trailing `/`.
  - **WHEN** `render_tree` emits a directory recorded as unreadable, **THEN** the line **SHALL** carry a trailing marker identifying it as unreadable.
  - **WHEN** the wrap mode is `Fence`, **THEN** the tree **SHALL** be emitted inside exactly one fenced block.
  - **WHEN** the wrap mode is `Inline`, **THEN** each emitted tree line **SHALL** be surrounded by single backticks with the branch symbols unchanged.
  - **WHEN** the wrap mode is `None`, **THEN** the tree **SHALL** be emitted bare, with no fence and no backticks.
- **Notes:** the symbols are U+251C U+2500, U+2514 U+2500, and U+2502, each followed by one space before the name, per `Scaffolding symbols generator.md`. The renderer consumes the `Wrap` type defined in T-001 and the `Node` tree built in T-002.

### T-004: Content section renderer

- **Intent:** [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md)
- **Touches:** `src/main.rs`
- **Depends on:** T-001, T-002, T-003
- **Acceptance criterion:** "a content dump of every included file, each introduced by a `--- File: <path> ---` header"; "A file whose bytes are not valid UTF-8 is listed … but its body is replaced by an explicit elision marker"; "In `fence` mode the opening fence of a file body is longer than the longest backtick run in that body"; "a file whose extension is in a small fixed table carries the corresponding language hint"; "Paths in the output use `/` separators and are relative to the scanned root on every platform."
- **Success criterion (EARS):**
  - **WHEN** `render_contents` emits a file section, **THEN** it **SHALL** precede the body with a `---` line, a `File: <root-relative path>` line, and a closing `---` line, matching the header the `awk` one-liner in `MDeezl.md` produces.
  - **WHEN** `render_contents` emits a file section that follows any earlier output, **THEN** it **SHALL** emit a blank line before the header's opening `---`, reproducing the leading newline in the `awk` one-liner's `"\n---\nFile: …"` so that the opening `---` is a thematic break and never an underline for the preceding line.
  - **WHEN** a file's bytes are not valid UTF-8, **THEN** the body **SHALL** be replaced by an elision marker stating the byte count, and the raw bytes **SHALL NOT** be written to the document.
  - **WHEN** a file cannot be read, **THEN** the body **SHALL** be replaced by a marker carrying the underlying error, and rendering **SHALL** continue with the next file.
  - **WHEN** `fence_len` is called with a body whose longest run of consecutive backticks is `n`, **THEN** it **SHALL** return `max(3, n + 1)`, so the opening fence is always longer than any run inside the body and the body cannot close its own block.
  - **WHEN** the wrap mode is `Fence` or `Inline` and a file has an extension present in the fixed table, **THEN** the opening fence **SHALL** carry the corresponding language hint; **WHEN** the extension is absent from the table, the fence **SHALL** carry no info string and **SHALL NOT** be an error.
  - **WHEN** the wrap mode is `None`, **THEN** the body **SHALL** be emitted with no surrounding fence.
  - **WHEN** a body is emitted on any platform, **THEN** its `File:` path **SHALL** use `/` separators and **SHALL** be relative to the scan root.
  - **WHEN** a single walk is rendered by both renderers, **THEN** an entry omitted by the ignore list **SHALL** be absent from the scaffold *and* from the content sections, and an entry re-admitted by `--include` **SHALL** be present in both — the composed property the single-walk design exists to guarantee.
- **Ordering note:** this task takes `Depends on: T-003` only because the composed clause above cannot be verified until both renderers exist; the content renderer itself does not consume the tree renderer.
- **Notes:** CommonMark 0.31.2 requires a closing fence at least as long as the opening one, which is what makes the computed length correct rather than a guess. Under the same spec the inherited header's second `---` is a **setext heading underline**, so `File: <path>` renders as an `<h2>` rather than being surrounded by thematic breaks. INT-0001 records this as an accepted consequence; the header is kept verbatim for byte-compatibility, and structural assertions therefore match exact lines instead of reasoning about heading levels.

### T-005: Output sink and process exit codes

- **Intent:** [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md)
- **Touches:** `src/main.rs`, `tests/cli.rs`
- **Depends on:** T-001, T-003, T-004
- **Acceptance criterion:** "`-o <file>` writes the document to that file instead of stdout"; "The emitted document consists of a `# <root name>` title, a `## Structure` section holding the scaffold, and a `## Contents` section holding the file sections, in that order"; "the process exits 0 on success, 2 on an argument error, and 1 when the root cannot be read or the output cannot be written. A non-zero exit never leaves a partial document on stdout."
- **Success criterion (EARS):**
  - **WHEN** no `-o` is given, **THEN** the complete document **SHALL** appear on stdout and nothing **SHALL** be written to any other file.
  - **WHEN** `-o <file>` is given, **THEN** the document **SHALL** be written to that file and **SHALL NOT** be written to stdout.
  - **WHEN** the document is assembled, **THEN** it **SHALL** consist of a `# <root name>` title, a `## Structure` section holding the tree, and a `## Contents` section holding the file sections, in that order.
  - **WHEN** the run succeeds, **THEN** the process **SHALL** exit 0; **WHEN** arguments are invalid, it **SHALL** print the message to stderr and exit 2; **WHEN** the root is unreadable or the output cannot be written, it **SHALL** print the message to stderr and exit 1.
  - **WHEN** the process exits non-zero, **THEN** it **SHALL NOT** have written a partial document to stdout.
- **Notes:** one `BufWriter` over either `File` or `io::stdout().lock()`. The document is rendered into memory before the sink is opened, so a failure cannot leave half a document on stdout; INT-0001 records the resulting peak-memory consequence and the rejected streaming alternative. **This task creates `tests/cli.rs`**, the single Cargo integration target that carries every end-to-end test. T-006 and T-007 each append one check to that file rather than creating it, which is why both name it in Touches as an appended check and take `Depends on: T-005`.

### T-006: CI regeneration on a two-OS matrix

- **Intent:** [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md)
- **Touches:** `.github/workflows/`, plus one check appended to the `tests/cli.rs` created by T-005
- **Depends on:** T-005
- **Acceptance criterion:** "Verification runs on both Linux and Windows, because two of the properties above — symlinks are never traversed, and paths use `/` on every platform — cannot both be exercised on a single operating system."
- **Success criterion (EARS):**
  - **WHEN** `Cargo.toml` exists and the bundle's `scaffold-ci.sh` is re-run, **THEN** a workflow file **SHALL** exist under `.github/workflows/` that runs `cargo test`, so the sprint checkpoint is not green merely because nothing ran.
  - **WHEN** that workflow is read, **THEN** its job matrix **SHALL** include both a Linux runner and a Windows runner, so the symlink guarantee and the path-separator guarantee are each exercised somewhere.
  - **WHEN** a test cannot run on the current platform, **THEN** it **SHALL** report the skip and its reason rather than passing silently.
- **Notes:** substrate convergence generated no workflow because it ran before any Rust existed; this task closes that gap. The bundle generates the file, and this task adds the second OS to the matrix if the generated form is single-OS.

### T-007: User documentation

- **Intent:** [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md)
- **Touches:** `README.md`, plus one check appended to the `tests/cli.rs` created by T-005
- **Depends on:** T-005
- **Acceptance criterion:** "`README.md` documents the same surface as `--help` — the flags, the four pattern forms, the pre-populated ignore list, and the three wrap modes — so that surface is discoverable before the binary is built."
- **Success criterion (EARS):**
  - **WHEN** `README.md` is read after this task, **THEN** it **SHALL** document every flag, all four pattern forms, the pre-populated ignore list, and the three wrap modes.
- **Notes:** split from CI regeneration because the two share no diff, no reviewer, and no failure mode. The `cargo tree` gate that briefly lived here moved to T-001, which owns the zero-dependency criterion and touches `Cargo.toml`; this task touches only `README.md`, which currently holds a single line of framing and no usage at all.
