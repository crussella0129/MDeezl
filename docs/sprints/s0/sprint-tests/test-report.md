# Sprint 0 Test Report

- **Verdict:** pass, with one criterion explicitly unverified (see Not verified).
- **Tested head:** `2e48a1cb5e7380631092e2e74371f28792afb583` (branch `dev`)
- **Runner:** `cargo test --all -- --nocapture` (the project's canonical suite)
- **Host:** Windows 11, `cargo 1.96.0` / `rustc 1.96.0`
- **Critique:** [critique.md](critique.md) — final verdict `proceed-with-caveats`
  after three rounds; rounds one and two returned `block`.

## Results

| Suite | Result | Detail |
|-------|--------|--------|
| Unit | 44 passed, 0 failed, 1 self-skipped with a printed reason | [unit-tests.md](unit-tests.md) |
| Integration | 2 passed, 0 failed | [integration-tests.md](integration-tests.md) |
| End-to-end | 28 passed, 0 failed | [e2e-tests.md](e2e-tests.md) |

The unit and integration counts overlap: both suites live in the same binary
target, and the 44 includes the 2 composed integration tests. Total distinct
tests: 72.

## Gates

| Gate | Result |
|------|--------|
| `cargo fmt --check` | clean |
| `cargo clippy --all-targets -- -D warnings` | clean |
| `cargo tree` | `mdeezl v0.1.0 (C:\Users\charl\MDeezl)` and nothing else — the zero-dependency criterion, confirmed at the toolchain level |

`cargo tree` is the manual gate T-001 owns; its literal output is recorded above
because it is a toolchain query rather than a property of the code. The
automated half is `test_manifest_dependencies_table_is_empty`, which now rejects
entries under any header containing `dependencies`, including
`[dev-dependencies]` and `[dependencies.serde]`.

## Intent verification

Every INT-0001 acceptance criterion traces through the locked test plan's
traceability table to at least one named, executed test, with the single
exception recorded below. The strongest single piece of evidence is the
self-check: running the binary against its own repository produces a
164,836-byte document in which 28 fences open, every one closes with a
long-enough closer, and the longest opening fence is 5 backticks — produced
where the sprint plans contain four-backtick runs. A fixed three-backtick
wrapper would have corrupted the document at exactly that point.

## Not verified

**"Verification runs on both Linux and Windows" is not verified.** Branch `dev`
has never been pushed, so `.github/workflows/sprint-loops-ci.yml` has never run
on any runner. `test_ci_workflow_runs_tests_on_both_platforms` is a static read
of the YAML committed beside it: it proves the file says so, not that a run has
happened.

Two items depend on that first run:

- the "continues traversing the remainder of the tree" half of the
  unreadable-directory clause, whose only full test skips on Windows because
  `std::fs::set_permissions` there cannot block `read_dir` and std exposes no
  ACL API. The marking half **is** covered host-independently by
  `test_walk_children_marks_unreadable_path`.
- every result above on a non-Windows host; all local runs were on Windows 11.

This is recorded as a known gap, not a pass. It closes when the sprint's
`dev -> main` checkpoint opens and the matrix runs green.

## Notable defects caught before release

- **The fence-balance assertion was invalid.** Counting fence-only lines and
  testing parity excludes every opening fence with a language hint while
  counting its closer, and counts a Markdown body's own fences as delimiters.
  Replaced with a state machine, then supplemented with the opener-outgrows-
  content invariant, because balance alone cannot detect an early close.
- **The Linux-authoritative skip guard would have failed rather than skipped
  under root**, the default in many containers: it keyed on whether `chmod`
  returned `Ok` instead of whether `read_dir` actually fails.
- **Three assertions could not fail.** The default-ignore test was four negative
  assertions with no control (an empty document would have passed); the
  forward-slash test handed the renderer an already-slashed literal; the
  determinism test compared two runs without checking either succeeded.
- **Two EARS branches had no test at all**: a root that exists but is a regular
  file, and rendering continuing past an unreadable file.

None of these were implementation bugs — the implementation was correct in each
case. They were tests that would have gone green through a real regression.
