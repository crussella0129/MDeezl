# INT-0004 — Gitignore-aware exclusion

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0004
- **State:** proposed
- **Work evidence:** none
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

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
- Gitignore filtering governs the scaffold tree and the content dump
  identically, preserving INT-0001's single-file-set property. An ignored
  directory prunes its whole subtree.
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
- When git is unavailable, the scan root is not a git work tree, or the query
  fails for any reason, MDeezl **still produces the document** using the
  pre-populated list alone, and says on stderr that gitignore filtering was
  skipped and why. A plain directory is a first-class input and must not error.
- Output stays byte-identical across runs on an unchanged tree, so the
  gitignore result must not leak the subprocess's output ordering into the
  document.
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
dependency-free *and* cheap at the same time. There is no subset to define, no
divergence from git to discover later, and no crate to add.

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
