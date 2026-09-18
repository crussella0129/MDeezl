# Test Critique — Sprint 1

Three bounded read-only review rounds were run against the sprint 1 test evidence, using the installed bundle's `prompts/test-critic.md`. Each critic was instructed to read the assertions themselves and the CI log, not the result documents.

| Round | Verdict | Concerns |
|-------|---------|----------|
| 1 | `block` | 12 |
| 2 | `proceed-with-caveats` | 3 |
| 3 | `proceed-with-caveats` | 1 |

Every concern was addressed before the next round, and each later round re-verified the earlier fixes against the code.

As in sprint 0, none of the findings was an implementation defect. They were tests that would have gone green through a real regression, or evidence that claimed more than it showed. The most consequential:

- **Git version.** CI runs git 2.55, but every git behaviour the design relies on had been measured on 2.54, and nothing pinned the directory-removal guard's precondition on the version CI uses. A new unit test now pins it; it passed on 2.55.0 on both legs.
- **Nested repositories.** Every nested-repository fixture used a gitlink *file*, so an `.exists()` → `.is_file()` regression passed all 116 tests. A nested clone with a `.git` **directory** now covers it. A deliberate mutation confirmed both a unit and an end-to-end test catch the regression.
- **`GIT_INDEX_FILE`.** Removal of this variable was untested, and it fails *silently*: a missed removal drops tracked files with nothing on stderr. It is now covered and mutation-checked.
- **The built-in list.** It was never checked on a run where the git query succeeded, so dropping the list whenever git answered would have gone unnoticed.
- **Isolation.** The in-process tests were claimed to be un-isolatable without a process-wide `set_var`. That was wrong: a `#[cfg(test)]` block on the child command does it.

The concern below is the final round's; its resolution note records what changed after it.

## Concerns

### C-001: Records cite a tested head that predates the round-two assertions
- **Where:** `e2e-tests.md` and `unit-tests.md` headers and CI section
- **Quote:** "**Tested head:** `1a4fb986939feba1ffd8dadcffaa5b97d174c5d1` (branch `dev`)"
- **Failure mode:** evidence-drift
- **Why it matters:** The round-two fixes landed in `0ff9f2a`, but the records still pointed at `1a4fb98` and its CI run, neither of which contains them. The critic confirmed the fixes themselves hold: CI run 35327617895 on `0ff9f2a` reports the three affected tests `ok` on both legs.
- **Suggested response:** tighten-assertion — restamp the tested head and CI link.
- **Resolution:** all three artifacts now cite `0ff9f2a9785b0acdd5f152c6e5d019e5506e5276` and run 35327617895. The CI section lists each earlier attempt and what it contained. Only the head reference and link changed; no evidence or assertion did, so no further review round was run.

## Confidence
proceed-with-caveats
