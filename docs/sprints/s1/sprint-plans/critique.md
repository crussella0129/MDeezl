# Plan Critique — Sprint 1

Five bounded read-only review rounds were run against the sprint 1 plans using
the installed bundle's `prompts/plan-critic.md`. Rounds one, two, and three
returned `block`, with 13, 7, and 3 concerns respectively. Round four returned
`proceed-with-caveats`, but its first caveat was a real violation of an
acceptance criterion rather than a caveat, so it was fixed instead of recorded,
and a narrow fifth round reviewed that fix alone. Every factual claim a critic
made was reproduced against git 2.54 before any plan or intent was changed.

The rounds caught defects that would otherwise have shipped:

- The root node's empty path made git exit 128, which would have skipped
  gitignore on every run.
- The scan root usually holds `.git`, so a literal nested-repository rule would
  have queried only `.` for `mdeezl .` — gitignore doing nothing in the main use
  case.
- `--include out` could not restore a gitignored directory's contents, because
  git reports every descendant.
- A path inside a submodule aborts the whole query.
- Backslash escaping is read as a path separator by Git for Windows. It was
  measured reporting a tracked `x[1].log` as ignored.
- An escaped directory name defeats git's tracked-content check, which would
  have dropped a tracked `app/[slug]/page.tsx` under the common whitelist idiom.

The concerns below are the final round's; each resolution note records what
changed after it.

## Concerns

### C-001: Older subtree-pruning criterion contradicts the new guard
- **Where:** INT-0004's both-halves criterion; `test-plan.md` traceability row for T-003's first clause
- **Quote:** "An ignored directory prunes its whole subtree."
- **Failure mode:** EARS-vague
- **Why it matters:** With "ignored" meaning "git reports it", this sentence said `app/[slug]` goes with its subtree while the new guard criterion says it stays, so no test could satisfy both.
- **Suggested response:** fix-in-plan
- **Resolution:** the criterion now reads "a pruned directory takes its whole subtree with it; whether a reported directory is pruned is decided by the directory-removal guard", and the row label matches.

### C-002: The guard ignores includes, so an escaped directory can still lose included content
- **Where:** INT-0004 Consequences, the glob-character bullet
- **Quote:** "both of which keep content rather than lose it"
- **Failure mode:** missing-risk
- **Why it matters:** Under `*`, `!*/`, `!*.tsx`, an escaped `app/[slug]/` holding only an untracked `notes.md` is pruned, and the ancestor rule stops `--include "*.md"` from rescuing it — while the same file under an ordinary name is kept. The consequence's absolute claim was false.
- **Suggested response:** fix-in-plan — record it; do not make the guard count includes, which would break the ancestor rule.
- **Resolution:** recorded as the third remaining effect in INT-0004's Consequences, with the reason the guard deliberately does not count includes and the workaround of including the directory itself. Added to the T-004 README list and to `test_readme_documents_gitignore`. The design is unchanged, as the critic recommended.

### C-003: The guard test's `nested` entry is not what git returns for the stated rules
- **Where:** `test-plan.md`, `test_prune_gitignored_guard_keeps_unreported_descendant`
- **Quote:** "It mirrors what git returned in planning for the whitelist idiom"
- **Failure mode:** plan-test-mismatch
- **Why it matters:** `nested` is an ordinary name, so `!*/` re-includes it and git would not report it under the three stated rules. An implementer checking the fixture against git could have "corrected" away its nested-repository half.
- **Suggested response:** fix-in-plan
- **Resolution:** the fixture's stated rules now add `nested/` after `!*/`, with the reason given, so the sets match what git returns for the stated rules. The critic confirmed the expected tree was already correct for those sets.

## Confidence
proceed-with-caveats
