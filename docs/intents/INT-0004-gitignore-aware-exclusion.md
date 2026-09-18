# INT-0004 — Gitignore-aware exclusion

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0004
- **State:** active
- **Work evidence:** [Sprint 1 build plan](../sprints/s1/sprint-plans/build-plan.md), [T-002 batched gitignore query](../sprints/s1/sprint-plans/build-plan.md#t-002-batched-gitignore-query)
- **Completion evidence:** none
- **Code evidence:** [src/main.rs](../../src/main.rs)
- **Test evidence:** [Sprint 1 test report](../sprints/s1/sprint-tests/test-report.md), [unit](../sprints/s1/sprint-tests/unit-tests.md), [integration](../sprints/s1/sprint-tests/integration-tests.md), [end-to-end](../sprints/s1/sprint-tests/e2e-tests.md)
- **Documentation evidence:** [README.md](../../README.md)

## Intent

Let MDeezl honour a repository's own `.gitignore` when deciding what to omit,
so pointing it at an unfamiliar project produces a useful document without the
user first having to discover which build directories that ecosystem uses.

The rules are read by delegating to `git check-ignore`, so the supported syntax
is git's own, entire and by definition. Gitignore filtering is therefore
available only where git is: it is a runtime capability that degrades when
absent, never a requirement.

**Non-goals:** replacing the ignore list from
[INT-0001](INT-0001-markdown-repo-context-bundle.md) — gitignore support layers
onto it and `--include` must still outrank everything; selecting a git ref or a
remote as the *source*, which is
[INT-0002](INT-0002-git-and-remote-sources.md); and any write, fetch, or network
operation — the one git subcommand invoked is read-only and local.

## Acceptance criteria

- Paths the repository ignores are omitted in addition to those the
  pre-populated list omits, and `--include` still outranks both. The
  include-wins rule from INT-0001 is unchanged.
- **Including a directory that git reports ignored exempts that directory and
  its whole subtree from gitignore.** Git reports every descendant of an ignored
  directory as ignored, not just the directory, so without this exemption
  `--include out` would restore an empty `out/`. The exemption applies **only**
  when the included directory is itself gitignored: a directory included merely
  to undo the built-in list — `--include build` where `build/` is not in
  `.gitignore` — keeps the repository's rules for everything inside it.
- **An include pattern cannot rescue an entry whose ancestor directory is
  pruned.** `--include "*.o"` restores a gitignored `top.o`, but not
  `out/a.o` when `out/` itself is gitignored and not included. This is the rule
  the built-in list already follows — the walk never descends into an excluded
  directory — and git's own: a file cannot be re-included if a parent directory
  is excluded. To get `out/a.o`, include `out`.
- Gitignore filtering governs the scaffold tree and the content dump
  identically, preserving INT-0001's single-file-set property. A pruned
  directory takes its whole subtree with it; whether a reported directory is
  pruned is decided by the directory-removal guard below.
- The behaviour can be switched off for a run, because a context bundle
  sometimes wants exactly the files git is told to forget.
- **The supported syntax is all of gitignore**, including `**`, negation,
  directory-only patterns, nested `.gitignore` files, `.git/info/exclude`, and
  `core.excludesFile` — because the matching is performed by git itself rather
  than reimplemented. No subset needs defining, and none may be silently
  assumed.
- **A file the repository tracks is included even when it matches an ignore
  pattern**, which is git's own rule: tracked files are not subject to exclude
  rules. A context bundle should contain what the repository actually keeps.
- **A directory git reports ignored is removed with its subtree only when git
  also reports every queried entry beneath it.** Otherwise the directory is kept
  and its entries are judged individually. For ordinary names this changes
  nothing, because git never reports a directory that holds tracked files. It
  guards the case where escaping hides a directory's tracked contents from git.
  Measured under the common whitelist idiom — `*`, `!*/`, `!*.tsx` — the
  directory `app/[slug]` is reported ignored, because git cannot `lstat` the
  escaped name to apply `!*/`, while its tracked `page.tsx` is not. Without this
  guard, top-down pruning would drop that tracked file.
- When git is unavailable, the scan root is not a git work tree, or the query
  fails for any reason, MDeezl **still produces the document** using the
  pre-populated list alone, and says on stderr that gitignore filtering was
  skipped and why. A plain directory is a first-class input and must not error.
- **When git reports the scan root itself ignored** — `mdeezl out/` where `out/`
  is gitignored — gitignore filtering is skipped with the same stderr notice,
  rather than producing a title over an empty tree. Pointing at a directory is an
  explicit request for its contents. A directory that holds tracked files is
  **not** reported ignored — git's tracked-file rule — so filtering still
  applies inside it: its tracked files are kept and its untracked ignored files
  omitted, which is exactly what the tracked-file criterion requires.
- **Gitignore is not applied inside a nested repository or submodule**, meaning
  any directory **other than the scan root** that contains a `.git` entry. The
  scan root is normally a repository itself, and must not be mistaken for a
  nested one. Git refuses to answer for a path inside a submodule and aborts the
  entire query, and for a nested repository the enclosing repository's rules
  are the wrong rules. Such contents fall back to the built-in list.
- **Every path is sent `./`-prefixed, with each glob character wrapped in a
  bracket class** — `[` as `[[]`, `*` as `[*]`, `?` as `[?]`, and `\` as
  `[\\]` — so that **git's tracked-file lookup treats each name literally**.
  Unprefixed, a leading `:` is parsed as pathspec magic and aborts the query.
  Unescaped, a name such as `p*.log` is read as a glob that can match a tracked
  `plain.log`, and git then suppresses it as tracked, so a file that should be
  omitted is silently kept. Backslash escaping is **not** used: Git for Windows
  converts `\` to `/` in every pathspec, so `a\[1].log` would be queried as
  `a/[1].log` — measured to report a tracked `x[1].log` as ignored, omitting a
  file the repository keeps, and to report an unrelated `out[1].txt` as ignored
  by an `out/` rule. The bracket-class form contains no backslash for the
  characters Windows permits in filenames, and was measured correct on Windows
  for tracked, untracked, and unrelated names.
- **Results are mapped to paths by position, never by parsing the path git
  echoes back.** The query asks for one record per input path, in input order,
  including paths that match nothing. Git's echo of an escaped path is not
  consistent across characters, so any scheme that parsed it would be fragile.
- **The query is always about the scan root.** Every variable listed by
  `git rev-parse --local-env-vars` — `GIT_DIR`, `GIT_WORK_TREE`,
  `GIT_INDEX_FILE`, `GIT_COMMON_DIR`, and eleven others — is removed from the git
  child's environment, so an ambient repository, as when MDeezl is run from
  inside a git hook, cannot redirect it.
- Output stays byte-identical across runs **for the same tree and the same git
  state** — the installed git, the ignore files, `.git/info/exclude`, and
  `core.excludesFile` — so the gitignore result must not leak the subprocess's
  output ordering into the document.
- This chapter is the authority for the repository's rules as a second
  exclusion source. INT-0001's "a single ignore list governs what is omitted"
  continues to describe the built-in mechanism it defined.
- The zero-dependency property of INT-0001 is preserved: `cargo tree` still
  reports only this crate, and the test suite adds nothing.
- One subprocess per run, not one per path.

## Rationale

INT-0001 shipped with a fixed six-entry list — `.*`, `target`, `node_modules`,
`dist`, `build`, `__pycache__` — and recorded the resulting gap as an accepted
consequence: a repository whose build directory is not in that list still
produces an enormous document, and the user has to notice and pass `--exclude`.
That is the single largest usability gap in the tool as shipped, and the
repository almost always already states the answer in its `.gitignore`.

**Sprint 1 research established that this chapter's original framing was
false.** It had presented the choice as a trade between correctness and the
zero-dependency property, on the assumption that exactness required either a
crate or a per-path subprocess. Neither holds: `git check-ignore --stdin`
batches every path through a single process, so delegation is exact *and*
dependency-free at the same time, with no per-path process cost. There is no
subset to define, no divergence from git to discover later, and no crate to add.
It is not free, however: the query's cost grows with the size of the
repository's index, which is recorded under Consequences.

What delegation costs instead is a runtime dependency on `git` being present and
the scan root being a work tree — measured, not assumed: outside a work tree the
command exits 128, which the documentation does not state. That cost is real but
bounded, and it is exactly what the degradation criterion absorbs. A plain
directory behaves as MDeezl does today.

## Alternatives

- **Delegate to `git check-ignore -z --stdin`. Selected.** Exact by
  construction, zero added dependencies, one subprocess per run, and it inherits
  nested ignore files, `core.excludesFile`, `**`, and negation for free. The
  original objection — "a pass per path unless batched" — was answered by
  research: `--stdin` batches, and `-z` makes the protocol NUL-separated so a
  filename containing a newline cannot corrupt it.
- **Hand-roll a gitignore subset** (~200–300 lines: anchored and unanchored
  patterns, `*` and `**`, directory-only trailing `/`, negation with `!`).
  Rejected: strictly more code for strictly less correctness, and it would make
  "document the supported subset precisely" a permanent maintenance burden
  instead of a solved question.
- **Depend on the `ignore` crate.** Rejected: it breaks the zero-dependency
  property for a capability a subprocess already provides exactly. This
  project's dependency constraint is explicit.
- **Grow the built-in list instead.** Rejected outright: it does not read what
  the repository already declares, which is the whole point of this intent.
- **Require a git work tree and error without one.** Rejected: it would
  contradict MDeezl's premise that any directory is a valid input.

## Consequences

- **MDeezl acquires a runtime dependency on `git` for this feature only.** Where
  git is absent the tool is exactly as capable as it was before, which is why
  this is a degradation and not a requirement — but it does mean identical
  inputs can produce different documents on two machines. The stderr notice is
  what makes that visible rather than mysterious.
- **The same repository now bundles differently than it did in sprint 0.** A
  user who runs MDeezl before and after this change gets different output with
  no flag change. The off switch exists so that difference is recoverable.
- **"What is omitted" is no longer a single inspectable list.** INT-0001's
  rationale prized exactly that property. There are now two sources — the list
  and the repository's own rules — so `--help` and the README must say where an
  omission came from.
- **This is the project's first subprocess call.** It brings process spawning,
  and with it the pipe-deadlock hazard that appears only once the path list is
  large enough to fill the pipe buffer. It also puts the codebase one step onto
  INT-0002's territory, so the boundary is stated in Non-goals above rather than
  left to drift.
- Reading ignore files means the output depends on repository state MDeezl did
  not previously read, which is one more input to keep deterministic.
- **A successful run does not announce that gitignore removed anything.** Only
  the skip is reported. This is a deliberate deferral, not an oversight: the
  common case is a run where filtering works, and a line on every such run is
  noise on a tool whose stdout is often piped. The off switch, `--help`, and the
  README carry the explanation instead. Revisit if users are surprised by
  missing files in practice.
- **Contents of submodules and nested repositories are not gitignore-filtered.**
  Filtering them correctly would need one query per repository, breaking the
  one-subprocess criterion. They keep the built-in list.
- **A filename that is not valid UTF-8 is matched only partially.** MDeezl
  builds relative paths with lossy UTF-8 conversion — a pre-existing property of
  INT-0001's path handling — so such a name reaches git carrying a replacement
  character. File globs such as `*.tmp` still match it, since git matches file
  globs against paths that need not exist on disk. But a rule that names the
  original bytes cannot match, and neither can a directory-only rule, because
  git cannot `lstat` the converted name to learn it is a directory. Such an
  entry falls back to the built-in list rather than erroring.
- **A name containing `*`, `?`, `[`, or `\` is matched only partially by the
  ignore rules.** Escaping fixes git's tracked-file lookup, but git hands the
  escaped text — not the real name — to its ignore matcher and to `lstat`. File
  globs such as `*.log` still match. A rule naming such an entry exactly does
  not, and neither does a directory-only rule for such a *directory*, since git
  cannot `lstat` the escaped name to learn it is one. Measured: under `tmp*/`, a
  directory `tmp[1]/` is not reported while its contents are, so it appears in
  the scaffold as an empty directory. The same blindness can make git report
  such a directory ignored while its tracked contents are not; the
  directory-removal guard above exists for exactly that case, so **tracked
  content is never omitted**. Three effects remain. An escaped name may be kept
  where an exact or directory-only rule would have omitted it, and may appear as
  an empty directory; both keep content. The third can lose *untracked* content
  an include asked for: under `*`, `!*/`, `!*.tsx`, a directory `app/[slug]/`
  holding only an untracked `notes.md` is reported ignored with everything
  beneath it, so it is pruned — and the ancestor rule then stops
  `--include "*.md"` from rescuing `notes.md`, where the same file under an
  ordinary `app/slug/` would be kept, because git re-includes that directory
  through `!*/`. This is accepted rather than fixed: making the guard count
  include-matched descendants would keep `out/` under `--include "*.o"` and
  break the ancestor rule. The workaround is to include the directory itself.
  Tracked framework files such as `pages/[id].tsx` and `app/[slug]/page.tsx` are
  handled correctly.
- **The query's cost grows with the size of the repository's index.** Git
  checks each path against the index to apply the tracked-file rule. Measured on
  the sprint 1 development host with git 2.54: 8,000 paths took 0.28 s against
  an empty index and 1.33 s against a 16,000-entry index, and 16,000 tracked
  paths took 2.65 s. That is acceptable for the tool's purpose — the
  alternative is a document containing every one of those files — but it is a
  real cost. **Revisit if a real repository pushes a run past roughly ten
  seconds.** The natural remedy is to query directories first and skip the
  contents of ignored ones, which needs a second subprocess and therefore a
  deliberate relaxation of the one-subprocess criterion.

## Transition history
- 2026-09-17: created as `proposed` at the close of sprint 0, promoting the
  gitignore gap that INT-0001 recorded as a deferred alternative and an accepted
  consequence.
- 2026-09-18: amended by sprint 1 research, still `proposed`. The chapter had
  framed the decision as correctness versus zero dependencies; that trade-off
  does not exist, because `git check-ignore --stdin` is exact, batched, and
  dependency-free. Recorded `git check-ignore -z --stdin` as the selected
  alternative, replaced the "document the supported subset" criterion with "the
  supported syntax is all of gitignore", and added criteria for graceful
  degradation outside a work tree, git's tracked-file rule, determinism, and
  one-subprocess-per-run. Added the runtime-git, differing-output, and
  first-subprocess consequences.
- 2026-09-18: `proposed → planned` for sprint 1. Work evidence attached to the
  sprint 1 build plan; no criterion changed at planning, since research had
  already amended them.
- 2026-09-18: amended while `planned`, from the sprint 1 plan critique, which
  returned `block`. Each change was verified against git 2.54 before being
  written here. Added criteria for: including a directory exempting its subtree
  (git reports every descendant of an ignored directory, so `--include out`
  would otherwise restore an empty `out/`); skipping with a notice when the scan
  root is itself ignored (otherwise a silent empty document); not filtering
  inside nested repositories or submodules (a path inside a submodule aborts the
  whole query with exit 128); and `./`-prefixing paths (a leading `:` is
  otherwise parsed as pathspec magic). Scoped the byte-identical criterion to
  the same git state, and stated that INT-0001's single-list criterion
  describes the built-in mechanism. Recorded the deferred success-path notice,
  the submodule and non-UTF-8 limitations, and the git-environment isolation as
  consequences. No criterion was weakened.
- 2026-09-18: amended while `planned`. Added the ancestor rule — an include
  pattern cannot rescue an entry whose ancestor directory is pruned. Found while
  revising the test plan, which had wrongly claimed `--include "*.o"` would
  restore `.o` files inside a gitignored `out/`. Pruning is top-down, so it
  cannot; and that is the consistent behaviour, matching both the built-in list
  and git's own rule. It is user-visible, so it is stated here rather than left
  implicit.
- 2026-09-18: amended while `planned`, from the second plan-critique round,
  which returned `block`. Each change was verified against git 2.54 first.
  The nested-repository rule now excludes the scan root, which is normally a
  repository — read literally, `mdeezl .` would have sent only `.` and
  gitignore would never have removed anything. The root-skip criterion now says
  "reported by git", with the tracked-content case stated: a directory holding
  tracked files is not reported ignored, so filtering applies inside it. The
  directory-include exemption is narrowed to directories git itself reports
  ignored, so `--include build` no longer switches off the repository's rules
  under `build/`. Added criteria for escaping glob characters (a name like
  `p*.log` was silently kept because its glob matched a tracked file), for
  mapping results by position rather than parsing git's inconsistent echo, and
  for removing every repository-local git variable rather than three. Corrected
  "cheap" to a measured cost with a revisit trigger, and corrected the
  non-UTF-8 consequence, which had wrongly said such names are never matched.
- 2026-09-18: amended while `planned`, from the third plan-critique round.
  Replaced backslash escaping with bracket-class escaping. Measured on Git for
  Windows: backslash escaping is converted to a path separator, so the tracked
  `x[1].log` was reported ignored — an omission the tracked-file criterion
  forbids — and an unrelated `out[1].txt` was reported ignored by `out/`. Both
  are correct under bracket-class escaping, as is the tracked `pages/[id].tsx`.
  Narrowed the criterion's claim from "git reads each name literally" to "the
  tracked-file lookup treats each name literally", and recorded as a consequence
  that the ignore matcher still sees the escaped text, measured with `tmp*/`
  against `tmp[1]/`.
- 2026-09-18: amended while `planned`, from the fourth plan-critique round,
  which returned `proceed-with-caveats`. Its first caveat was a real violation of
  the tracked-file criterion rather than a caveat, so it was fixed instead of
  recorded: under the whitelist idiom `*` / `!*/` / `!*.tsx`, git reports the
  escaped directory `app/[slug]` ignored while its tracked `page.tsx` is not —
  measured — and top-down pruning would have dropped the tracked file. Added the
  directory-removal guard as a criterion, and replaced the consequence's
  absolute "every failure keeps content" with the precise statement it now
  supports: tracked content is never omitted.
- 2026-09-18: clarified while `planned`, from a narrow fifth review of the
  directory-removal guard, which returned `proceed-with-caveats`. Reworded the
  both-halves criterion's "an ignored directory prunes its whole subtree" to "a
  pruned directory takes its whole subtree", since it had contradicted the guard.
  Recorded the one remaining case where escaped names lose content — untracked
  content an include asked for, in an escaped directory holding nothing git
  keeps — and why the guard deliberately does not count includes. No design
  change.
- 2026-09-18: `planned → active`; sprint 1 Build Phase began implementing
  T-001 through T-004. Backlog item T-102, which this intent carried, was
  decomposed into those four tasks. Work evidence unchanged.
- 2026-09-18: Test Phase evidence attached while `active`. 119 tests pass at
  `0ff9f2a`, locally and in CI on `ubuntu-latest` and `windows-latest` with git
  2.55.0, and the four Linux-only cases ran on the Linux leg with no SKIP. All
  eighteen acceptance criteria are mapped in the test report: seventeen to
  executed tests, and the authority criterion to recorded inspection.
