# Sprint 1 Research Report

## Intents Reviewed
- [INT-0004](../../../intents/INT-0004-gitignore-aware-exclusion.md) — selected; relevance: this sprint implements it end to end; current state: `proposed`, to be advanced by the Plan Phase. Research resolves its open dependency decision and narrows its acceptance criteria to the mechanism chosen.
- [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md) — reviewed, not advanced; relevance: it owns the single-ignore-list criterion and the include-outranks-ignore rule that this sprint must layer onto without breaking; current state: `realized`. It must not regress.
- [INT-0002](../../../intents/INT-0002-git-and-remote-sources.md) — reviewed, not advanced; relevance: **boundary risk.** This sprint introduces the project's first subprocess call to `git`, which is the same machinery INT-0002 contemplates. It is bounded here to reading ignore rules for a local path; selecting a ref or a remote as the *source* stays out of scope.

## 1. Sprint Goal

Make MDeezl honour a repository's own `.gitignore` so that pointing it at an
unfamiliar project yields a usable document without the user first discovering
which build directory that ecosystem uses. The gitignore result layers onto the
existing ignore list rather than replacing it, `--include` still outranks
everything, and the behaviour can be switched off for a run. INT-0004 requires
the zero-dependency property to be preserved or the decision to break it
recorded first: research concludes it can be preserved outright, so no
dependency is added.

## 2. Existing Code Survey

| File | Relevance | Notes |
|------|-----------|-------|
| src/main.rs | high | Owns `matches`, `is_excluded`, and `walk_children`. `is_excluded(opts, name, rel)` is the single decision point both renderers inherit, so gitignore has exactly one correct place to enter. `Node.rel` is already scan-root-relative and `/`-separated, which is precisely the path form `git check-ignore` wants. |
| docs/intents/INT-0004-gitignore-aware-exclusion.md | high | The intent under implementation. Its Alternatives section poses the four options this report resolves between, and its acceptance criteria demand the supported syntax be documented precisely. |
| docs/intents/INT-0001-markdown-repo-context-bundle.md | high | `realized`, and must stay that way. Its criteria that constrain this sprint: one list governs both halves identically; include always wins; deterministic byte-identical output; `.git` excluded by default. |
| README.md | medium | Documents the ignore list and pattern forms, and states "No gitignore support yet. Add a `--exclude` for any build directory not in the list above." That sentence becomes false this sprint. |
| docs/sprints/s0/sprint-tests/test-report.md | medium | Records the sprint 0 test posture this sprint inherits, including that platform-gated tests must print a SKIP reason rather than pass silently, and that CI runs a two-OS matrix. |
| docs/work/tasks.md | low | T-102 is the queued backlog entry this sprint consumes. |

## 3. External Sources

- [git-check-ignore](https://git-scm.com/docs/git-check-ignore) — The mechanism this sprint adopts. Verified against the documentation: `--stdin` reads "pathnames from the standard input, one per line"; without `-v` it prints only the paths that match, so negated patterns are already resolved for us; exit **0** means one or more paths are ignored, **1** means none are, **128** is a fatal error; and "By default, tracked files are not shown at all since they are not subject to exclude rules."
- [gitignore](https://git-scm.com/docs/gitignore) — The pattern syntax being honoured. Relevant because delegating to git means the supported syntax is *all of it*, including `**`, negation, directory-only patterns, and the precedence of `.git/info/exclude` and `core.excludesFile` — none of which a hand-rolled subset would get right for free.
- [std::process::Command](https://doc.rust-lang.org/std/process/struct.Command.html) — The subprocess primitive, and the `spawn` + `wait_with_output` pattern that avoids the pipe deadlock described under Risks.

## 4. Risks, Unknowns, Dependencies

Empirical findings first — each was measured on this machine with
`git 2.54.0.windows.1`, because two of them are not answered by the
documentation:

- **`git check-ignore` requires a git work tree.** Outside one it exits 128 with
  `fatal: not a git repository`. The documentation does not state this; the test
  does. This matters because MDeezl's whole premise is "a given path for a
  directory/repo" — plain directories are a first-class input, so gitignore
  support must be *optional at runtime*, not assumed.
- **One process handles the whole run.** `--stdin` batches every path, so there
  is no per-path process cost. Measured: six paths, one invocation.
- **`-z --stdin` is NUL-separated in *and* out.** Verified by piping NUL-
  separated input and inspecting the raw output bytes. This removes the only
  correctness hole in the newline protocol: a filename containing a newline.
- **Ignored directories are reported as such.** Querying `build` (a directory
  matched by `build/`) returns it, so an ignored directory can prune its entire
  subtree rather than being rediscovered file by file.
- **A tracked file matching a pattern is *not* reported as ignored** — confirmed
  against a repo where `tracked.log` was committed with `add -f` under a
  `*.log` rule. `--no-index` overrides that. Default behaviour is the right one
  for a context bundle: a file the repository actually tracks belongs in the
  bundle.
- **Paths are resolved relative to the `-C` directory.** Running
  `git -C <scan-root> check-ignore --stdin` with scan-root-relative paths works
  even when the scan root is a subdirectory of the repository, and rules from
  the repository root still apply.

Risks:

- **Risk: pipe deadlock.** Writing a large path list to the child's stdin while
  not draining its stdout can block forever once the pipe buffer fills — and a
  large repository is exactly when the list is large. Mitigation: `spawn` with
  both piped, write stdin from a `std::thread` (or drop stdin before reading),
  then `wait_with_output`. This is the one place a naive implementation breaks
  only under load, which is the worst kind of bug to ship.
- **Risk: silent behaviour change.** Gitignore support turns previously-included
  files into omitted ones. A user who bundled a repo before and after this
  sprint gets different output with no flag change. Mitigation: it must be
  possible to switch off, and the document or stderr should make clear that
  gitignore filtering was applied.
- **Risk: INT-0001 regression.** `is_excluded` currently has one precedence rule.
  Adding a second source risks breaking "include always wins" — the property
  that makes `--include .github` work. Mitigation: gitignore enters as an
  additional *ignore* source only, checked after the include list has had its
  say, and the existing precedence tests must keep passing untouched.
- **Risk: determinism.** Output must stay byte-identical across runs. Git's
  output order need not match input order, so the result must be collected into
  a set and the tree's own sort order preserved.
- **Risk: scope bleed into INT-0002.** Shelling out to git is the first step
  onto INT-0002's territory. Mitigation: this sprint runs exactly one git
  subcommand, read-only, against the local scan root, and touches no ref, no
  remote, and no network.
- **Unknown: whether `.git` should stay force-excluded.** It is in the default
  list today and `--include ".*"` re-admits it. Gitignore does not change that;
  noted so the Plan Phase does not accidentally alter it.
- **Dependency: `git` on PATH, at runtime only.** Not a build dependency, not a
  crate. `cargo tree` stays empty and the CI matrix keeps working.

## 5. Recommended Approach

Primary: **delegate to `git check-ignore -z --stdin`, one subprocess per run.**
After the walk produces its `Node` tree — already pruned by the built-in list —
collect every surviving node's root-relative path, send them NUL-separated to a
single `git -C <root> check-ignore -z --stdin`, read the NUL-separated set of
ignored paths back, and prune the tree against it. When git is missing, the root
is not a work tree, or the call fails, report the degradation on stderr and
continue with the built-in list alone; the document is still produced.

Rationale: it is **exact by construction and dependency-free at the same time**,
which is the combination the other three options cannot offer. INT-0004 framed
the choice as a trade between correctness and the zero-dependency property, and
that framing turns out to be false — `check-ignore` is git's own matcher, so
there is no subset to define, no divergence to discover later, and no crate to
add. It also inherits `.git/info/exclude`, `core.excludesFile`, nested
`.gitignore` files, `**`, and negation for free, all of which a hand-rolled
matcher would have to earn one bug at a time.

Alternatives considered and rejected:

- **Hand-roll a gitignore subset** (~200–300 lines). Rejected: it is strictly
  more code for strictly less correctness, and INT-0004's own criterion that
  "whatever subset is supported is documented precisely" becomes a standing
  maintenance burden. Delegation makes the supported syntax "all of it".
- **Depend on the `ignore` crate.** Rejected: breaks the zero-dependency
  property for a capability that a subprocess already provides exactly. The
  user's constraint for this project is explicit on dependencies.
- **Grow the built-in list.** Rejected outright — it does not read what the
  repository already declares, which is the entire point of the intent.

The one genuine cost of delegation is the runtime `git` requirement, and it is
the cost the degradation path exists to absorb: a plain directory, or a machine
without git, behaves exactly as MDeezl does today.

## Artifacts

No files were saved; the findings are empirical measurements against throwaway
git repositories under the session scratchpad, and each is reproduced in
section 4 with the command shape that produced it. The decisive ones:

- `git check-ignore` outside a work tree → exit 128, `fatal: not a git repository`.
- `printf 'a\0b\0' | git -C <root> check-ignore -z --stdin` → NUL-separated
  ignored paths on stdout.
- a committed `tracked.log` under a `*.log` rule → not reported as ignored.
