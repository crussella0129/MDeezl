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

**Non-goals:** replacing the ignore list from
[INT-0001](INT-0001-markdown-repo-context-bundle.md) — gitignore support layers
onto it and `--include` must still outrank everything; and implementing the full
gitignore specification, if a defensible subset turns out to cover the real
cases.

## Acceptance criteria

- Patterns from the scanned repository's `.gitignore` files are applied in
  addition to the pre-populated list, and `--include` still outranks them.
- The behaviour can be turned off for a run, because a context bundle sometimes
  wants exactly the files git is told to forget.
- Whatever subset of gitignore syntax is supported is documented precisely, and
  a pattern outside that subset is reported rather than silently misapplied.
- The zero-dependency property of INT-0001 is either preserved or the decision
  to break it is recorded here first.

## Rationale

INT-0001 shipped with a fixed six-entry list — `.*`, `target`, `node_modules`,
`dist`, `build`, `__pycache__` — and recorded the resulting gap as an accepted
consequence: a repository whose build directory is not in that list still
produces an enormous document, and the user has to notice and pass `--exclude`.
That is the single largest usability gap in the tool as shipped, and the
repository almost always already states the answer in its `.gitignore`.

It is a separate intent because it is a genuinely different desired outcome —
"respect what this repository already declares" rather than "omit what the user
names" — and because the honest implementation options have materially different
costs, which deserve their own decision record.

## Alternatives

- **Depend on the `ignore` crate.** Correct and complete immediately, at the
  cost of the zero-dependency property INT-0001 chose deliberately.
- **Hand-roll a subset** — anchored and unanchored patterns, `*` and `**`,
  directory-only trailing `/`, negation with `!`. Keeps the dependency count at
  zero and is a few hundred lines, with real risk of subtle divergence from git.
- **Shell out to `git check-ignore`.** Exact by construction and dependency-free,
  but requires `git` on PATH and a pass per path unless batched.
- **Grow the built-in list instead.** Cheapest, and strictly worse than reading
  what the repository already says.

## Consequences

- Whichever option is chosen, MDeezl acquires a second source of exclusions, so
  the "what is omitted" story stops being the single inspectable list INT-0001's
  rationale prized. The `--help` and README explanation will need to say where
  each omission came from.
- Reading `.gitignore` files means the tool's output depends on repository state
  it did not previously read, which is one more thing to keep deterministic.

## Transition history
- 2026-09-17: created as `proposed` at the close of sprint 0, promoting the
  gitignore gap that INT-0001 recorded as a deferred alternative and an accepted
  consequence.
