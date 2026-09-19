Finalized - DO NOT EDIT

# Sprint 2 Build Plan

## Intents
- [INT-0005](../../../intents/INT-0005-reproducible-toolchain.md) — state: planned. All eight acceptance criteria are covered:
  1. the toolchain file;
  2. CI bringing itself to the file's toolchain on every run;
  3. pinning as a one-line change;
  4. CI logging its toolchain;
  5. the README;
  6. the git-version policy;
  7. INT-0001's two-OS legs, which must not regress;
  8. a regression test that does not block a pin.
- [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md) — state: realized. Not advanced, but constrained: the CI workflow this sprint edits delivers its two-OS criterion, and both OS legs and `--nocapture` must survive.
- [INT-0004](../../../intents/INT-0004-gitignore-aware-exclusion.md) — state: realized. Not advanced. Its git-dependent behaviours are why git's version is recorded rather than pinned.

## Schema Tree
- Sprint Goal: local gates predict CI; track stable by default, pin when needed
  - Mechanism
    - T-001: Toolchain file and CI steps
  - Delivery
    - T-002: README toolchain section

## Execution Sequence

### T-001: Toolchain file and CI steps

- **Intent:** [INT-0005](../../../intents/INT-0005-reproducible-toolchain.md)
- **Touches:** `rust-toolchain.toml` (new), `.github/workflows/sprint-loops-ci.yml`, `tests/cli.rs`
- **Depends on:** (none)
- **Acceptance criterion:** the INT-0005 criteria for the toolchain file, CI bringing itself to the file's toolchain, pinning as a one-line change, CI logging its toolchain, the two-OS legs, and the regression test that does not block a pin.
- **Success criterion (EARS):**
  - **WHEN** `rust-toolchain.toml` is read, **THEN** it **SHALL** have a `[toolchain]` table naming a `channel` and listing the `rustfmt` and `clippy` components, and at this sprint's close the channel **SHALL** be `stable`.
  - **WHEN** CI runs, **THEN** before the update it **SHALL** log `rustc +stable --version`, recording the runner image's own stable. `+stable` keeps a pinned file from triggering an auto-install of the pin into that reading.
  - **WHEN** CI runs, **THEN** before any gate, and in this order, it **SHALL** run `rustup update --no-self-update stable` and then `rustup toolchain install --no-self-update`, the latter with no toolchain argument. Each **SHALL** be its own step, so each step's result is that one command's result on both shells.
  - **WHEN** CI runs, **THEN** after the install and before `cargo fmt --check`, one step with `shell: bash` on both legs **SHALL** log `rustc`, `cargo`, `cargo clippy`, `rustup`, and `git` versions, so any failing line fails the step. The workflow **SHALL NOT** run `rustup check`, which exits 100 whenever any update is available.
  - **WHEN** CI runs under `channel = "stable"`, **THEN** the post-install `rustc` **SHALL** equal the version the update step reports, as either `updated - rustc X` or `unchanged - rustc X`. **WHEN** the update step reports `unchanged`, **THEN** the update path **SHALL** be recorded as not exercised in that run, rather than as passed.
  - **WHEN** the file's `channel` is changed to an already-installed version that differs from stable, with no other edit, **THEN** `rustc --version` there **SHALL** report the pinned version.
  - **WHEN** the channel is pinned to a version absent from the runners, with no other edit, **THEN** on both legs that version **SHALL** be installed before the gates, and the post-install `rustc --version` **SHALL** report it. This clause is verified only by the consent-gated throwaway pull request described in the Notes. It covers the pinned halves of INT-0005 criteria 2 and 3.
  - **WHEN** the workflow is read after this task, **THEN** it **SHALL** still list both `ubuntu-latest` and `windows-latest`, `fail-fast: false`, and `cargo test --all -- --nocapture`.
  - **WHEN** any of the following is changed, **THEN** a named test **SHALL** fail:
    - the toolchain file is removed, or stripped of its channel or a component;
    - the pre-update log, update, install, or post-install log step is removed;
    - the pre-update log is moved after the update;
    - the update and install steps are swapped;
    - the post-install log is moved ahead of the install;
    - `rustup check` is reintroduced.

    The test **SHALL NOT** require the channel to be `stable`.
  - **WHEN** the crate is built after this task, **THEN** `cargo tree` **SHALL** report only this crate.
- **Notes:**
  - **This machine's toolchain is already updated.** With the user's approval, `stable` moved from 1.96.0 to 1.98.1, and `rustfmt` and `clippy` are installed.
  - **Every mechanism choice above was measured on this machine:**
    - the no-argument install printed "using existing install" and did not update;
    - `rustup update stable` reports `updated` or `unchanged` and exits 0;
    - `rustup check` exited 100;
    - in a pinned directory, a bare `rustc` reported the pin (1.96.0), while `rustc +stable` reported 1.98.1.
  - **The tests read files at runtime.** They go through `CARGO_MANIFEST_DIR`, so a deleted file fails one named test instead of breaking compilation.
  - **The pinned halves of criteria 2 and 3 need a CI run.** The workflow runs only on pushes to `main` or `dev` and on pull requests, so the Test Phase will **ask the user** before opening a throwaway pull request pinned to a version absent from the runners, then close it. If the user declines, INT-0005 closes `active`, not `realized`, with that half recorded as unverified in CI.

### T-002: README toolchain section

- **Intent:** [INT-0005](../../../intents/INT-0005-reproducible-toolchain.md)
- **Touches:** `README.md`, plus one check appended to `tests/cli.rs`
- **Depends on:** T-001
- **Acceptance criterion:** "The README states the policy, the commands that bring a local toolchain level with CI, how to pin, the rustup version the commands were verified with, and how to upgrade an older rustup"; "Git's version floats with the CI runner image … the README says so."
- **Success criterion (EARS):**
  - **WHEN** `README.md` is read after this task, **THEN** it **SHALL** contain each of these literal strings, one for each required item:

    | Literal string | Required item |
    |----------------|---------------|
    | `Track stable by default` | the policy |
    | `rust-toolchain.toml` | the toolchain file |
    | `rustup update stable` | first command that brings a local toolchain level with CI |
    | `rustup toolchain install` | second command that brings a local toolchain level with CI |
    | `To pin` | how to pin |
    | `rustup 1.29` | the rustup version the commands were verified with |
    | `rustup self update` | how to upgrade an older rustup |
    | `a new stable` | the trade-off that a new stable can turn CI red |
    | `git's version` | the git-version note |
