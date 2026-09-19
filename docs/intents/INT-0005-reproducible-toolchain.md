# INT-0005 — Reproducible toolchain between local and CI

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0005
- **State:** active
- **Work evidence:** [Sprint 2 build plan](../sprints/s2/sprint-plans/build-plan.md), [T-001 toolchain file and CI steps](../sprints/s2/sprint-plans/build-plan.md#t-001-toolchain-file-and-ci-steps)
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent

A developer who runs `cargo fmt`, `cargo clippy -D warnings`, and `cargo test`
locally and sees them pass should be able to expect CI to pass too.

**Policy, chosen by the user at the sprint 2 boundary: track stable by default,
and pin when stability demands it.** Both modes use one mechanism, and moving
between them is a one-line change.

**Non-goals:** adding any crate or third-party CI action; changing what the gates
check; pinning git.

## Acceptance criteria

- A `rust-toolchain.toml` at the repository root names the channel, `stable`
  by default, and the `rustfmt` and `clippy` components.
- CI brings itself to exactly the toolchain the file names on every run. It
  runs `rustup update --no-self-update stable`, then
  `rustup toolchain install --no-self-update`, before any gate. Under `stable`,
  CI therefore runs the true current stable, not the runner image's
  pre-installed one, which lags the release. Under a pinned version, CI runs
  exactly that version: it is installed by the second command, or by rustup's
  auto-install on first use. Each of the two commands is its own CI step, so a
  failure in either fails the run on both shells.
- **Pinning is a one-line change:** replacing `stable` with a version such as
  `1.98.1` makes both local and CI use that version, with no other edit.
- Every CI run records its toolchain in the log, so a drift failure diagnoses
  itself:
  - before the update, the runner image's own stable, via
    `rustc +stable --version` so a pinned file cannot auto-install over it;
  - the update step's own report of whether stable was `updated` or
    `unchanged`, and to what;
  - after the install, `rustc`, `cargo`, `clippy`, `rustup`, and `git`.

  `rustup check` is **not** used: it exits 100 whenever any update is available,
  including rustup's own, which would fail the step for no code reason.
- The README states the policy, the commands that bring a local toolchain level
  with CI, how to pin, the rustup version the commands were verified with, and
  how to upgrade an older rustup.
- Git's version floats with the CI runner image and is not pinned. The
  git-dependent behaviours that matter are covered by tests that run on
  whichever git executes them; the README says so.
- CI keeps both `ubuntu-latest` and `windows-latest` legs and `--nocapture`, so
  INT-0001's two-OS criterion does not regress.
- A test fails if the policy's shape is quietly reverted, without blocking a
  legitimate pin:
  - the toolchain file disappears, loses its channel, or loses a component;
  - the workflow stops updating, stops installing from the file, or stops
    logging versions;
  - the workflow reorders those steps.

  The test does **not** require the channel to be `stable`. A test that did
  would make pinning a two-file change.

## Rationale

Sprint 1 found the drift by breaking on it. CI used the runner's pre-installed
Rust, 1.98.0, while the development host had 1.96.0. The first push of sprint 1
therefore failed at clippy on a lint only the newer version flags. The local
`clippy -D warnings` run had passed, so it was not a reliable predictor.

Sprint 2 research found the problem runs deeper. Measured on the development
host, a toolchain file saying `channel = "stable"` **updates nothing** — it
resolved to the locally installed 1.96.0. There were in fact three different
"stables":

- the true current release, 1.98.1;
- the runner image's, 1.98.0;
- the host's, 1.96.0.

Tracking stable does not happen on its own on any side. Research first read
`rustup toolchain install`'s help — "install or update … by default the active
toolchain" — as updating by default. **Measurement showed otherwise.** With no
argument it answered "using existing install" and left `stable` at 1.96.0. It
installs only a toolchain that is missing.

The mechanism is therefore two commands:

1. `rustup update stable` moves tracking-mode CI to the true current stable.
2. `rustup toolchain install` then installs whatever version the file pins.

In tracking mode the second is a no-op; in pinned mode the first is harmless.
Pinning stays a one-line edit to the file. It was measured to take effect with
no download: pinning to the already-installed 1.96.0 gave that version while
stable was 1.98.1.

## Alternatives

- **Pin by default.** Not chosen: the user chose to track stable by default. It
  remains available through the same file.
- **Toolchain file alone.** Rejected: measured to change nothing about drift.
- **A third-party toolchain action such as `dtolnay/rust-toolchain`.** Rejected:
  it adds an external dependency to do what rustup, already on every runner,
  does itself.
- **A local check that fails when behind stable.** Rejected: it would need the
  network during `cargo test`, and would turn an informational condition into a
  failure.

## Consequences

- **Tracking stable means CI can go red when a new stable ships a lint,** on
  work that did not touch the flagged code. This is the accepted trade-off of the
  chosen default. The pin is its remedy.
- **Local can still fall behind.** Nothing forces a contributor to update. CI's
  version log makes the cause obvious when it happens, and the README gives the
  two commands that fix it.
- **Updating the development host's `stable` is machine-wide.** It changes the
  Rust used by every other project on that machine that tracks stable.
- **CI runs grow slightly,** by one toolchain install or update per leg.
- **Open observation: lag correction has not yet been seen in CI.** Through
  sprint 2, every CI update step reported `unchanged`, because the runner
  images already carried the true current stable. The evidence that
  `rustup update stable` moves a lagging stable forward is one measurement on
  the development host, where 1.96.0 moved to 1.98.1. The first CI log whose
  update step reads `updated` should be attached here as evidence when it
  appears. That happens when a new stable ships before the runner images pick
  it up.

## Transition history
- 2026-09-18: created as `proposed` at the close of sprint 1, from the toolchain
  drift that failed that sprint's first CI run.
- 2026-09-18: amended by sprint 2 research, still `proposed`. Recorded the
  user's chosen policy — track stable by default, pin when needed. Replaced the
  three policy-neutral criteria with ones specific to that policy. Added the
  measured finding that a toolchain file alone does not update, together with
  the `rustup toolchain install` lever that serves both modes. Added the
  machine-wide and CI-red-on-new-lint consequences.
- 2026-09-18: `proposed → planned` for sprint 2. Work evidence attached. The
  plan was approved together with the machine-wide update of the development
  host's `stable` toolchain, which the plan put to the user explicitly.
- 2026-09-18: amended while `planned`, from the sprint 2 plan critique, which
  returned `block`. Research had misread `rustup toolchain install`'s help as
  "updates by default". Measurement showed the no-argument form leaves an
  installed `stable` untouched. The CI criterion now names both commands:
  `rustup update stable`, then the no-argument install for a pin. Also sharpened:
  - the version-log criterion, which now records before and after the update,
    plus `rustup check`;
  - the README criterion, which now names the rustup version the commands need;
  - the regression-test criterion, which must not require `stable`, since that
    would make a pin a two-file change.
- 2026-09-18: amended while `planned`, from the second sprint 2 plan critique,
  which returned `block`. Each change was measured on the development host
  first.
  - **Dropped `rustup check` from the version log.** It exited 100 because an
    update was available, and under `bash -eo pipefail` that would fail CI for no
    code reason. The update step's own `updated` or `unchanged` report replaces
    it.
  - **The pre-update log uses `rustc +stable --version`.** In a pinned directory
    a bare `rustc` reported the pin (1.96.0) while `+stable` reported 1.98.1, and
    under a pin rustup's auto-install could otherwise record the pin as the
    image's toolchain.
  - **Clarified the rustup-version criterion.** Neither the rustup
    documentation nor its changelog states the minimum version for installing
    the active toolchain with no argument, so the criterion now names the
    verified version and the upgrade path. It no longer asks for a minimum no
    source establishes.
  - **Pinning's install path and step structure.** The pinned version may be
    installed by the no-argument install or by auto-install on first use, and
    each command is its own CI step.
- 2026-09-18: `planned → active`; sprint 2 Build Phase began T-001 and T-002.
  Backlog item T-105 was consumed by those two tasks.
