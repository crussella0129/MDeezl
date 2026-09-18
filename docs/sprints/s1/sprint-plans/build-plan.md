Finalized - DO NOT EDIT

# Sprint 1 Build Plan

## Intents
- [INT-0004](../../../intents/INT-0004-gitignore-aware-exclusion.md) — state: planned; acceptance criteria covered: all twenty — layering with include winning; the directory-include exemption, limited to directories git reports ignored; the ancestor rule; the directory-removal guard that keeps tracked content; identical governance of both halves with subtree pruning; the off switch; full gitignore syntax by delegation; git's tracked-file rule; degradation with a stated reason; skipping when git reports the scan root ignored; no filtering inside nested repositories or submodules other than the scan root; `./`-prefixed paths with bracket-class escaping; position-mapped results; isolation from ambient git variables; byte-identical output for the same git state; authority for the second source; zero dependencies; and one subprocess per run.
- [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md) — state: realized; not advanced, but constrained. Its transition history records how INT-0004 reads two of its criteria. Its single-file-set property, include-wins rule, document skeleton, and determinism must not regress: all 72 sprint 0 tests are the regression bar and must pass unedited.
- [INT-0002](../../../intents/INT-0002-git-and-remote-sources.md) — state: proposed; not advanced. Boundary only: one read-only, local git subcommand; no ref, no remote.

## Schema Tree
- Sprint Goal: honour the repository's own `.gitignore`, dependency-free
  - CLI
    - T-001: Off switch and help text
  - Query
    - T-002: Batched gitignore query
  - Pruning
    - T-003: Tree pruning with include exemption
  - Delivery
    - T-004: User documentation

## Execution Sequence

### T-001: Off switch and help text

- **Intent:** [INT-0004](../../../intents/INT-0004-gitignore-aware-exclusion.md)
- **Touches:** `src/main.rs`, plus one check appended to the existing `tests/cli.rs`
- **Depends on:** (none)
- **Acceptance criterion:** "The behaviour can be switched off for a run"; and from Consequences, "`--help` and the README must say where an omission came from."
- **Success criterion (EARS):**
  - **WHEN** `parse_args` receives no arguments, **THEN** it **SHALL** return options with `use_gitignore` set to true.
  - **WHEN** `parse_args` receives `--no-gitignore`, **THEN** it **SHALL** return options with `use_gitignore` set to false, and every other option **SHALL** keep its default.
  - **WHEN** `-h` or `--help` is given, **THEN** the help text **SHALL** name both exclusion sources — the ignore list and the repository's `.gitignore` — and **SHALL** document `--no-gitignore` and that gitignore filtering needs git on `PATH` and a git work tree.
- **Notes:** `use_gitignore` is a new field on the existing `Options`; `Default` sets it true.

### T-002: Batched gitignore query

- **Intent:** [INT-0004](../../../intents/INT-0004-gitignore-aware-exclusion.md)
- **Touches:** `src/main.rs`, `tests/cli.rs`
- **Depends on:** T-001
- **Acceptance criterion:** "The supported syntax is all of gitignore"; "A file the repository tracks is included even when it matches"; the degradation criterion; "When git reports the scan root itself ignored…"; "Gitignore is not applied inside a nested repository or submodule … other than the scan root"; "Every path is sent `./`-prefixed, with each glob character wrapped in a bracket class"; "Results are mapped to paths by position"; "The query is always about the scan root"; "The zero-dependency property … is preserved"; "One subprocess per run."
- **Success criterion (EARS):**
  - **WHEN** `nul_payload` is given a list of paths, **THEN** it **SHALL** return each path prefixed with `./`, with each glob character wrapped in a bracket class — `[` as `[[]`, `*` as `[*]`, `?` as `[?]`, and `\` as the **four bytes** `[`, `\`, `\`, `]` — and followed by a single NUL, in order, with no other separator. It **SHALL NOT** use backslash escaping. The backslash class is specified by its bytes because in an ordinary Rust string literal `"[\\]"` is only three bytes; the implementation and its test use raw strings or byte values so the two cannot share a transcription slip.
  - **WHEN** `parse_records` is given the NUL-separated output of `check-ignore -v -n` and the number of paths sent, **THEN** it **SHALL** return one boolean per path in input order, true exactly when that record's source field is non-empty and its pattern does not begin with `!`; **WHEN** the record count differs from the number of paths sent, **THEN** it **SHALL** return `Err`.
  - **WHEN** `gitignore_candidates` collects paths from the walked tree, **THEN** the result **SHALL** contain `.` standing for the scan root, **SHALL** contain every other node's `rel`, **SHALL NOT** contain an empty entry, and **SHALL NOT** contain any path beneath a directory — other than the scan root — that holds a `.git` entry; that directory itself **SHALL** still be included.
  - **WHEN** `git_ignored` is called inside a git work tree, **THEN** it **SHALL** spawn `git -C <root> check-ignore -z -v -n --stdin` exactly once, with every variable listed by `git rev-parse --local-env-vars` removed from the child's environment, **SHALL** send every candidate through that one process, and **SHALL** return `Ok` holding exactly the candidates `parse_records` marks ignored.
  - **WHEN** git cannot be spawned, **THEN** `git_ignored` **SHALL** return `Err` stating that git was not found; **WHEN** git exits with any status other than 0 or 1, **THEN** it **SHALL** return `Err` carrying the first line of git's stderr.
  - **WHEN** a tracked file matches an ignore pattern, **THEN** it **SHALL NOT** be in the returned set, because the query runs without `--no-index`.
  - **WHEN** the candidate list exceeds the operating system's pipe buffer, **THEN** `git_ignored` **SHALL** complete without deadlock: stdin is `take()`n and moved into a thread that owns and drops it, write errors there are ignored so git's exit status decides the outcome, `wait_with_output` drains stdout and stderr, and the thread is joined without panicking.
  - **WHEN** `use_gitignore` is true, **THEN** `run` **SHALL** call `git_ignored` exactly once; **WHEN** that call returns `Err`, or its result contains `.` because git reports the scan root ignored, **THEN** `run` **SHALL** write exactly one stderr line beginning `mdeezl: gitignore filtering skipped:` followed by the reason, **SHALL** still write the complete document, and the process **SHALL** exit 0.
  - **WHEN** `use_gitignore` is false, **THEN** `run` **SHALL NOT** invoke git and **SHALL** write nothing to stderr.
  - **WHEN** the crate is built after this task, **THEN** `Cargo.toml` **SHALL** still declare no dependencies of any kind and `cargo tree` **SHALL** report only this crate.
- **Notes:** the signature is `fn git_ignored(root: &Path, paths: &[String]) -> Result<HashSet<String>, String>`. At this task's boundary `run` computes the set and reports degradation but does not yet prune; T-003 applies the set. Every function this task adds is therefore live in the non-test binary, so `clippy -D warnings` passes here without `allow(dead_code)`. A `#[cfg(test)]` thread-local counter in `git_ignored` records spawns, so "exactly once" is measured rather than inferred. The fifteen local variables are hard-coded from `git rev-parse --local-env-vars` on git 2.54 rather than queried, because querying would cost a second subprocess. Behaviours verified against git 2.54 during planning, each of which this task depends on:
  - An empty entry aborts the whole query with exit 128, so the root is sent as `.`.
  - `.` is reported exactly when git considers the scan root ignored. A directory holding tracked files is not.
  - With `-v -n`, every input yields exactly one four-field record, in input order. That includes tracked and non-matching paths, which come back with empty fields. This is what makes position mapping sound.
  - Exit codes under `-v -n` are unchanged: 0 means some path is ignored, 1 means none is, 128 is a fatal error.
  - A path inside a registered submodule aborts the whole query with exit 128.
  - A bare `:!foo` is parsed as pathspec magic and aborts. `./:!foo` is accepted.
  - An unescaped `p*.log` is not reported when a tracked `plain.log` exists, because its glob matches the tracked file.
  - **Backslash escaping is wrong on Windows.** Git for Windows converts `\` to `/` in pathspecs: `./x\[1].log` reported a *tracked* `x[1].log` as ignored, and `./out\[1].txt` was reported ignored by the `out/` rule. Bracket-class escaping gets both right, along with an untracked `a[1].log` and a tracked `pages/[id].tsx`.
  - Git's ignore matcher sees the escaped text, not the real name. Under `tmp*/`, the directory `tmp[1]` is not reported while `tmp[1]/f.txt` is. This keeps content rather than omitting it, and is recorded in INT-0004's Consequences.
  - A staged but uncommitted file counts as tracked.

### T-003: Tree pruning with include exemption

- **Intent:** [INT-0004](../../../intents/INT-0004-gitignore-aware-exclusion.md)
- **Touches:** `src/main.rs`, `tests/cli.rs`
- **Depends on:** T-002
- **Acceptance criterion:** "Paths the repository ignores are omitted in addition to those the pre-populated list omits, and `--include` still outranks both"; "Including a directory that git reports ignored exempts that directory and its whole subtree"; "An include pattern cannot rescue an entry whose ancestor directory is pruned"; "A directory git reports ignored is removed with its subtree only when git also reports every queried entry beneath it"; "Gitignore filtering governs the scaffold tree and the content dump identically"; "Output stays byte-identical across runs for the same tree and the same git state."
- **Success criterion (EARS):**
  - **WHEN** `prune_gitignored` meets a node whose `rel` is in the ignored set and that matches no include pattern, **THEN** it **SHALL** remove the node, and **WHEN** the node is a directory its entire subtree **SHALL** go with it — subject to the guard in the next clause.
  - **WHEN** a directory is in the ignored set, matches no include pattern, and some queried entry beneath it is **not** in the ignored set, **THEN** `prune_gitignored` **SHALL** keep the directory and judge each of its children individually, so a tracked file inside it is never dropped.
  - **WHEN** `prune_gitignored` meets a node that is in the ignored set **and** matches an include pattern, in any of the four pattern forms, **THEN** it **SHALL** keep that node and **SHALL NOT** apply the ignored set anywhere in its subtree.
  - **WHEN** a node matches an include pattern but is **not** in the ignored set, **THEN** `prune_gitignored` **SHALL** keep it without exempting its subtree, so each descendant is still judged against the ignored set.
  - **WHEN** an ancestor directory is pruned, **THEN** every entry beneath it **SHALL** be pruned with it, including an entry that matches an include pattern, because pruning is top-down.
  - **WHEN** `git_ignored` returns `Ok` and git does not report the scan root ignored, **THEN** `run` **SHALL** prune the walked tree before rendering, so every pruned entry is absent from the scaffold and from the content dump alike.
  - **WHEN** `run` is executed twice on an unchanged repository with unchanged git state, **THEN** the two documents **SHALL** be byte-identical.
- **Notes:**
  - **Reusing `matches`.** The include check reuses `matches`, so the four pattern forms select the same entries against both exclusion sources. The subtree exemption is **not** identical to the built-in list, and is deliberately narrower than a blanket rule. It applies only to an included directory that git itself reports ignored, because only there does git mark every descendant. `--include build`, used just to undo the built-in list, leaves the repository's rules in force inside `build/`.
  - **The guard.** `prune_gitignored` receives both the ignored set and the queried candidate set, because "some queried entry beneath it is not ignored" needs both. Descendants never sent as candidates — the contents of a nested repository — do not count, so an ignored directory holding a nested repository is still pruned. For ordinary names git never reports a directory that holds tracked files, so the guard only fires where escaping has hidden tracked contents from git. That was measured with `app/[slug]` under `*`, `!*/`, `!*.tsx`.
  - **Unit fixtures.** Unit fixtures use the set git actually returns, with every descendant of an ignored directory listed.
  - **Determinism.** The pruner reads a `HashSet` over an already-sorted tree, so git's output order cannot reach the document.
  - **Which directories fixtures ignore.** Fixture repositories ignore only directories absent from the built-in list — `out/` and `generated/`, never `build/` or `target/`. Ignoring a directory the built-in list already prunes would prove nothing.
  - **Where the absence tests land.** The end-to-end tests that assert a gitignored path is absent land in this task, since before pruning they could not pass.

### T-004: User documentation

- **Intent:** [INT-0004](../../../intents/INT-0004-gitignore-aware-exclusion.md)
- **Touches:** `README.md`, plus one check appended to `tests/cli.rs`
- **Depends on:** T-003
- **Acceptance criterion:** from Consequences, "`--help` and the README must say where an omission came from"; and the degradation criterion, whose runtime requirement a reader needs before running the tool.
- **Success criterion (EARS):**
  - **WHEN** `README.md` is read after this task, **THEN** it **SHALL** document:
    - the repository's `.gitignore` as a second exclusion source;
    - `--no-gitignore`;
    - the requirement for git and a work tree;
    - the degradation behaviour;
    - git's tracked-file rule;
    - the directory-include exemption and its limit;
    - the ancestor rule;
    - that nested repositories and submodules are not gitignore-filtered;
    - that names containing `*`, `?`, `[`, or `\` are matched only partially by the ignore rules, including the one case where an include cannot rescue untracked content in such a directory.
  - **WHEN** `README.md` is read after this task, **THEN** it **SHALL NOT** contain the sprint 0 statement "No gitignore support yet".
