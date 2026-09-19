# Sprint 2 Research Report

## Intents Reviewed
- [INT-0005](../../../intents/INT-0005-reproducible-toolchain.md) — selected and revised. Relevance: this sprint implements it. Current state: `proposed`. Its acceptance criteria were written before the pin-versus-track decision; this research rewrites them around the policy the user chose.
- [INT-0004](../../../intents/INT-0004-gitignore-aware-exclusion.md) — reviewed, not advanced. Relevance: its git-dependent behaviours are what made git-version drift matter. Sprint 1 already pinned the riskiest ones in tests that run on whatever git executes them. Current state: `realized`.
- [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md) — reviewed, not advanced. Relevance: its two-OS verification criterion is delivered by the CI workflow this sprint edits, and must not regress. Current state: `realized`.

## 1. Sprint Goal

Make a local `cargo fmt`, `cargo clippy -D warnings`, and `cargo test` a reliable predictor of CI, under the policy the user chose at the sprint boundary: **track stable by default, and pin when stability demands it.** Both modes should be served by one mechanism, and switching between them should be a one-line change. Git's version, which floats with the CI runner image, gets a recorded policy rather than a pin.

## 2. Existing Code Survey

| File | Relevance | Notes |
|------|-----------|-------|
| docs/intents/INT-0005-reproducible-toolchain.md | high | Written at sprint 1 close, when the policy was still undecided. Its three criteria are satisfiable by either choice and must be made specific to the chosen one. |
| .github/workflows/sprint-loops-ci.yml | high | Runs `ubuntu-latest` and `windows-latest`, using each runner image's pre-installed Rust with no toolchain step. It prints no tool versions, so sprint 1 had to infer Rust 1.98.0 from a clippy help URL and git 2.55.0 from the checkout action's log. |
| Cargo.toml | medium | Edition 2024, no `rust-version`. This sprint does not need an MSRV, but its absence is noted. |
| README.md | medium | Has no toolchain guidance. A contributor currently has no way to know which Rust CI uses. |
| docs/sprints/s1/sprint-tests/test-report.md | medium | The findings section records the drift this intent exists to fix: local 1.96.0 against CI 1.98.0, and a clippy failure on a lint only the newer version flags. |
| docs/work/tasks.md | low | T-105 is the backlog item this sprint consumes. |

## 3. External Sources

- [rustup — Overrides and the toolchain file](https://rust-lang.github.io/rustup/overrides.html) — Consulted for runtime behaviour: whether `channel = "stable"` updates on use, what `rustup toolchain install` with no argument does, and whether listed components auto-install. **The page did not answer any of these.** It documents the file's syntax, not rustup's runtime behaviour. The answers came from the installed rustup's own `--help` and from direct measurement; see section 4.

## 4. Risks, Unknowns, Dependencies

Measured on this host, using rustup 1.29.0:

- **There are three different "stables".** `rustup check` reports true current stable as **1.98.1** (2026-09-01). The CI runner image shipped **1.98.0** during sprint 1, so the image lags the release. This host has **1.96.0**. "Tracking stable" therefore does not happen by itself on any side; each side sits at whatever it last installed.
- **A `channel = "stable"` toolchain file updates nothing.** In a scratch directory containing only that file, `rustup show active-toolchain` resolved to the installed stable, and `rustc --version` printed 1.96.0. The file alone would declare the policy while leaving the drift untouched. This is the central finding.
- **`rustup toolchain install` with no argument is the lever that serves both modes.** Its `--help` reads "Install or update the given toolchains, or by default the active toolchain". Updating is the default, and `--no-update` exists to suppress it. Run in the repository:
  - with `channel = "stable"`, it updates to the true current stable;
  - with a pinned `channel = "1.98.1"`, it installs exactly that version.

  One CI step therefore serves both modes, and pinning becomes a one-line edit to the toolchain file.

Risks and unknowns:

- **Unknown: whether components listed in the file install with that command.** This was not measurable without installing a toolchain, which is left to the Build Phase with approval. Build should verify it with `rustup component list --installed`.
- **Risk: tracking stable means CI can go red when a new stable ships a lint.** This happens on work that did not touch the code involved. It is the trade-off the user chose; the pin is the remedy, and the README must say so.
- **Risk: local can still fall behind.** Nothing forces a contributor to update. The mitigations are a documented `rustup toolchain install`, and CI logging exact versions so that a drift failure diagnoses itself instead of looking mysterious. Sprint 1 spent a round trip inferring CI's version from a URL.
- **Risk: updating this host's `stable` is machine-wide.** It changes the Rust used by every other project on this machine that tracks stable, not only MDeezl. The Plan Phase must state this explicitly for approval rather than bury it in a build step.
- **Dependency: git's version floats with the runner image,** and pinning it is out of reach without installing git in CI. It stays covered by the sprint 1 tests that run on whatever git executes them: the escaped-directory precondition and the local-environment list. CI should log git's version too.
- **Risk: regression of INT-0001's two-OS criterion.** The workflow edit must keep both matrix legs and `--nocapture`.

## 5. Recommended Approach

Primary:

1. **Add a `rust-toolchain.toml`** declaring `channel = "stable"` and the `rustfmt` and `clippy` components.
2. **Add one CI step before the gates:** `rustup toolchain install --no-self-update`. It installs or updates whatever the file names, so CI runs the true current stable, not the image's lagging one, and runs the exact version once a pin is set.
3. **Add a CI step that prints `rustc`, `cargo`, `clippy`, and `git` versions,** so every run records its toolchain.
4. **Document the policy in the README:** how to update locally with the same command, and how to pin.
5. **Bring this host to current stable** with that command, flagged in the plan as a machine-wide change needing approval.
6. **Add a test** that pins the policy's shape: the toolchain file exists, names a channel, and lists both components, and the workflow installs from it and logs versions. That makes a quiet revert fail.

Alternatives considered:

- **Pin by default.** Rejected: the user chose to track stable by default. Pinning remains available through the same file.
- **Toolchain file alone.** Rejected: measured to change nothing about drift.
- **A third-party CI action such as `dtolnay/rust-toolchain`.** Rejected: it adds a dependency on an external action to do what rustup, already on every runner, does itself.
- **A drift check that fails local builds when behind stable.** Rejected: it would need the network during `cargo test`, and it would turn an informational condition into a failure.

Rationale: one file with one install command gives both modes of the user's hybrid policy, with no new dependency. Switching modes is a one-line diff that both sides follow automatically. What the approach cannot do is force a contributor to update, so it makes drift *visible* instead: CI logs its versions, and the README tells contributors how to match them.

## Artifacts

None were saved. The measurements come from `rustup check`, `rustup show active-toolchain` in a scratch directory holding only a toolchain file, and `rustup toolchain install --help`. Each is reproduced in section 4, with the output that decided it.
