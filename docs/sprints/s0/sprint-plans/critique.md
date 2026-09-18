# Plan Critique — Sprint 0

Four bounded read-only review rounds were run against the sprint 0 plans using
the installed bundle's `prompts/plan-critic.md`. Rounds one and two returned
`block`; every concern was addressed in the plans or in INT-0001 before the next
round, and each subsequent round re-verified the previous round's fixes rather
than taking them on trust. The concerns below are the final round's; the
resolution notes record what changed after it.

## Concerns

### C-001: `--wrap none` semantics are asserted by the plans but authorized by no INT-0001 criterion
- **Where:** `test-plan.md` traceability rows for `none`; `build-plan.md` T-003 and T-004 `None` clauses; INT-0001 `## Acceptance criteria`
- **Quote:** "`none`: the tree is bare, with no fence and no backticks" — a criterion column with no counterpart in INT-0001, whose only text on the mode was "`--wrap <fence|inline|none>` controls how output is surrounded and defaults to `fence`."
- **Failure mode:** intent-drift
- **Why it matters:** The plans tightened the plan-side commitment and added `test_tree_none_wrap`, but the chapter spelled out semantics only for `fence` and `inline`; `none` appeared solely as an enum value. Two traceability rows quoted criteria that did not exist, and the mode the plans test most sharply was the one the chapter never authorized.
- **Suggested response:** fix-in-plan
- **Resolution:** INT-0001's wrap criterion now states the `none` semantics explicitly — no fence and no backticks on the tree, no fence on bodies — and Transition history records the fifth amendment.

### C-002: the two composed integration tests have no owning task
- **Where:** `test-plan.md` `## Integration Tests`; `build-plan.md` T-002, T-004, T-005
- **Quote:** "`test_excluded_entry_absent_from_both_halves`: T-002 + T-003 + T-004 composed — a tree containing `target/` is walked once and rendered twice"
- **Failure mode:** hidden-dep
- **Why it matters:** The round-three fix gave `tests/cli.rs` an owner but left the separate integration section out. No task's Touches or EARS clause claimed these two tests, and the traceability row credited them to a T-002 clause although they cannot be written until T-003 and T-004 exist. INT-0001's single-ignore-list criterion was therefore verified by tests belonging to no diff.
- **Suggested response:** fix-in-plan
- **Resolution:** T-004 gained a composed EARS clause covering the both-halves property and now owns both tests; the traceability row points at that clause. T-004 takes `Depends on: T-003` with an ordering note stating the dependency exists for the composed verification, not for the renderer itself.

### C-003: T-006/T-007 checks are filed as unit tests but placed in the integration target
- **Where:** `test-plan.md` `### T-006 / T-007 checks`; `build-plan.md` T-005 note and the T-006/T-007 Touches lines
- **Quote:** "T-006 and T-007 each add their check to that existing file rather than creating one, which is why neither lists a Rust path in Touches." — yet both Touches lines had been rewritten to name `tests/cli.rs`.
- **Failure mode:** plan-test-mismatch
- **Why it matters:** The T-005 note's justification had become false of the very fields the previous fix rewrote, and the test plan still classified both checks as unit tests. A builder following the test plan would place them in an in-crate module, making T-006's and T-007's `Depends on: T-005` spurious.
- **Suggested response:** fix-in-plan
- **Resolution:** the T-005 note was corrected to match the Touches fields, and the test plan now states explicitly that both checks live in `tests/cli.rs` as integration tests, grouped beside their owning tasks for readability only.

## Confidence
proceed-with-caveats
