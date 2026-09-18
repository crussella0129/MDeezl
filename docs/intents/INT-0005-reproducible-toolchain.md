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

**Non-goals:** adding any crate; changing what the gates check.

## Acceptance criteria

- The Rust toolchain CI uses is the one a contributor gets locally, or the
  difference is deliberate and documented.
- A lint introduced by a newer toolchain surfaces through a deliberate upgrade,
  not as a surprise CI failure on unrelated work.
- Whatever choice is made about git versions — CI's git floats with the runner
  image — is recorded, with the tests that pin git-dependent behaviour named.

## Rationale

Sprint 1 found the drift by breaking on it. CI uses the runner's floating stable
Rust (1.98.0 at the time) while the development host had 1.96.0. The first push
of sprint 1 therefore failed CI at clippy: 1.98 extends `byte_char_slices` to an
array literal that 1.96 accepts. Local `clippy -D warnings` had passed, so it was
not a reliable predictor of CI.

Git drifts the same way: CI ran git 2.55.0, while every git behaviour INT-0004
relies on was measured on 2.54.0. Sprint 1 answered that by pinning the two
riskiest behaviours in tests that run on whatever git executes them. That covers
correctness, but not predictability.

## Alternatives

- **Pin with `rust-toolchain.toml`.** It is a file, not a dependency, and makes
  local and CI agree exactly. The cost is that CI stops picking up new lints
  until someone bumps the pin deliberately.
- **Track stable on both sides**, with a contributor note to run `rustup update`.
  Nothing is pinned, and drift returns whenever someone forgets.
- **Leave it.** Accept an occasional red CI run on a lint the local toolchain
  did not know about.

## Consequences

- Pinning makes toolchain upgrades a deliberate, reviewable change.
- It does not address git's version, which is governed by the runner image. That
  stays covered by the git-behaviour tests sprint 1 added.

## Transition history
- 2026-09-18: created as `proposed` at the close of sprint 1, from the toolchain
  drift that failed that sprint's first CI run.
