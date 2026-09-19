# INT-0005 — Reproducible toolchain between local and CI

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0005
- **State:** proposed
- **Work evidence:** none
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
- CI installs or updates exactly the toolchain the file names on every run,
  using `rustup toolchain install` with no argument. Under `stable`, CI
  therefore runs the true current stable, not the runner image's pre-installed
  one, which lags the release. Under a pinned version, CI runs exactly that
  version.
- **Pinning is a one-line change:** replacing `stable` with a version such as
  `1.98.1` makes both local and CI use that version, with no other edit.
- Every CI run records the exact `rustc`, `cargo`, `clippy`, and `git` versions
  it used in its log, so a drift failure diagnoses itself.
- The README states the policy, how to bring a local toolchain level with CI,
  and how to pin.
- Git's version floats with the CI runner image and is not pinned. The
  git-dependent behaviours that matter are covered by tests that run on
  whichever git executes them; the README says so.
- CI keeps both `ubuntu-latest` and `windows-latest` legs and `--nocapture`, so
  INT-0001's two-OS criterion does not regress.
- A test fails if the policy's shape is quietly reverted: the toolchain file
  disappears or loses a component, or the workflow stops installing from it or
  logging versions.

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

Tracking stable does not happen on its own on any side. The lever that makes it
happen is `rustup toolchain install` with no argument. Its own help text says it
installs or updates the active toolchain, updating by default. Run where the
file is, it tracks stable when the file says `stable` and installs the pin when
the file names a version. That one lever is what lets a single mechanism serve
the user's hybrid policy.

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
  one command that fixes it.
- **Updating the development host's `stable` is machine-wide.** It changes the
  Rust used by every other project on that machine that tracks stable.
- **CI runs grow slightly,** by one toolchain install or update per leg.

## Transition history
- 2026-09-18: created as `proposed` at the close of sprint 1, from the toolchain
  drift that failed that sprint's first CI run.
- 2026-09-18: amended by sprint 2 research, still `proposed`. Recorded the
  user's chosen policy — track stable by default, pin when needed. Replaced the
  three policy-neutral criteria with ones specific to that policy. Added the
  measured finding that a toolchain file alone does not update, together with
  the `rustup toolchain install` lever that serves both modes. Added the
  machine-wide and CI-red-on-new-lint consequences.
