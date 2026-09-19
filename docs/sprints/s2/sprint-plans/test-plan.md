Finalized - DO NOT EDIT

# Sprint 2 Test Plan

## Intent Traceability

There is one row per EARS clause, in build-plan order.

| Intent | Acceptance criterion | Build task / EARS clause | Verification |
|--------|----------------------|--------------------------|--------------|
| [INT-0005](../../../intents/INT-0005-reproducible-toolchain.md) | The toolchain file names a channel and both components; stable by default | T-001 / WHEN read THEN `[toolchain]`, a `channel`, `rustfmt` and `clippy`; stable at close | `test_toolchain_file_declares_channel_and_components` (accepts any channel), plus a **close-time inspection** that the channel is `stable` |
| [INT-0005](../../../intents/INT-0005-reproducible-toolchain.md) | CI records the image's own stable | T-001 / WHEN CI runs THEN `rustc +stable --version` is logged before the update | `test_ci_updates_installs_and_logs_in_order` |
| [INT-0005](../../../intents/INT-0005-reproducible-toolchain.md) | CI brings itself to the file's toolchain | T-001 / WHEN CI runs THEN update-stable, then no-argument install, each its own step, in that order, before any gate | `test_ci_updates_installs_and_logs_in_order` |
| [INT-0005](../../../intents/INT-0005-reproducible-toolchain.md) | CI records the toolchain it used; no `rustup check` | T-001 / WHEN CI runs THEN post-install versions are logged under `shell: bash`, before the gates, without `rustup check` | `test_ci_updates_installs_and_logs_in_order` |
| [INT-0005](../../../intents/INT-0005-reproducible-toolchain.md) | Under `stable`, CI runs true current stable | T-001 / WHEN CI runs under stable THEN the post-install `rustc` equals the update step's reported version; `unchanged` means the path was not exercised | **CI log gate**, on both legs (see Integration Tests) |
| [INT-0005](../../../intents/INT-0005-reproducible-toolchain.md) | Pinning is a one-line change | T-001 / WHEN the channel is set to an installed non-stable version THEN `rustc` reports it | **Pin check**, local (see below) |
| [INT-0005](../../../intents/INT-0005-reproducible-toolchain.md) | Under a pin, CI runs exactly that version (pinned halves of criteria 2 and 3) | T-001 / WHEN pinned to a version absent from the runners THEN on both legs it is installed and the post-install `rustc` reports it | **Pinned CI gate** (see Integration Tests), run only with the **user's consent at the Test Phase**; if declined, this clause is unverified and INT-0005 closes `active` |
| [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md) | Two-OS verification does not regress | T-001 / WHEN the workflow is read THEN both OS legs, `fail-fast: false`, and `--nocapture` remain | `test_ci_workflow_runs_tests_on_both_platforms` (sprint 0, unedited); `test_ci_updates_installs_and_logs_in_order` |
| [INT-0005](../../../intents/INT-0005-reproducible-toolchain.md) | A test catches a quiet revert without blocking a pin | T-001 / WHEN a listed element changes THEN a named test fails; `stable` is not required | the two T-001 tests, **mutation-checked** against every listed change, plus one pin mutation that must **pass** |
| [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md) (zero dependencies); INT-0005 non-goal | No crate added | T-001 / WHEN built THEN `cargo tree` shows only this crate | `test_manifest_dependencies_table_is_empty` (sprint 0, unedited), plus the literal `cargo tree` output in the report |
| [INT-0005](../../../intents/INT-0005-reproducible-toolchain.md) | The README states the policy, commands, pinning, verified rustup version, upgrade path, and git-version policy | T-002 / WHEN read THEN the nine literal strings are present | `test_readme_documents_toolchain_policy` |
| [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md), [INT-0004](../../../intents/INT-0004-gitignore-aware-exclusion.md) | All realized criteria — no regression | none; constrained, not advanced | all 119 existing tests, **unedited**, green on 1.98.1 locally and in CI |

**Eleven EARS clauses:**

- **Ten in T-001.**
  - **Seven have named tests:**
    - the toolchain file, plus a close-time inspection;
    - the pre-update log;
    - the update-then-install order;
    - the post-install log;
    - the two-OS legs;
    - the revert guard, mutation-checked;
    - no dependencies, plus the literal `cargo tree` output.
  - **Three are verified by recorded observation:**
    - CI tracking true stable, by the CI log gate;
    - the one-line pin, by the pin check;
    - CI under a pin, by the pinned CI gate, subject to the user's consent.
- **One in T-002,** verified by a named test.

## Unit Tests

None. This sprint changes configuration and documentation, not `src/main.rs`.

## Integration Tests

- **Intents:** [INT-0005](../../../intents/INT-0005-reproducible-toolchain.md)
- **CI log gate.** On both legs of the real runners, the log must show:
  1. the pre-update `rustc +stable --version`;
  2. the update step's own `updated - rustc X` or `unchanged - rustc X` line;
  3. a post-install `rustc --version` equal to that `X`.

  A leg whose update step says `unchanged` has the update path **not exercised** in that run; the report says so instead of counting it as passed. The gate is deliberately not hard-coded to a version number.
- **Pinned CI gate** (consent-gated). The throwaway pull request's only change pins `rust-toolchain.toml` to a version absent from the runners. On both legs its log must show that version being installed before the gates, and a post-install `rustc --version` that reports it. The pull request is then closed without merging, and its branch deleted.

## End-to-End Tests

- **Status:** possible.
- **`test_toolchain_file_declares_channel_and_components`** reads `rust-toolchain.toml` at runtime through `CARGO_MANIFEST_DIR`. It asserts a `[toolchain]` table, a `channel = "…"` line with a non-empty value, and that `rustfmt` and `clippy` are listed. It deliberately does not assert `stable`.
- **`test_ci_updates_installs_and_logs_in_order`** reads the workflow at runtime. It asserts these steps, in this order by position:
  1. `rustc +stable --version`;
  2. `- run: rustup update --no-self-update stable`, as its own step line;
  3. `- run: rustup toolchain install --no-self-update`, as its own step line, ending there with no toolchain argument;
  4. a post-install `rustc --version`;
  5. `cargo --version`, `cargo clippy --version`, `rustup --version` and `git --version`;
  6. `cargo fmt --check`.

  It also asserts that:
  - the post-install log's step declares `shell: bash`;
  - `rustup check` appears nowhere;
  - `fail-fast: false` and `cargo test --all -- --nocapture` remain.
- **`test_readme_documents_toolchain_policy`** asserts the nine literal strings from T-002.

## Mutation check

Each change below is applied to the real file and then restored byte-identical. Each must fail the named test.

| Change | Test that must fail |
|--------|---------------------|
| `rust-toolchain.toml` deleted | `test_toolchain_file_declares_channel_and_components` (runtime read, so this is not a compile failure) |
| pre-update log step removed | `test_ci_updates_installs_and_logs_in_order` |
| pre-update log moved after the update | `test_ci_updates_installs_and_logs_in_order` |
| the `channel` line removed | `test_toolchain_file_declares_channel_and_components` |
| `clippy` dropped from `components` | `test_toolchain_file_declares_channel_and_components` |
| update step removed | `test_ci_updates_installs_and_logs_in_order` |
| install step removed | `test_ci_updates_installs_and_logs_in_order` |
| update and install swapped | `test_ci_updates_installs_and_logs_in_order` |
| post-install log step removed | `test_ci_updates_installs_and_logs_in_order` |
| post-install log moved ahead of the install | `test_ci_updates_installs_and_logs_in_order` |
| `rustup check` added to the log step | `test_ci_updates_installs_and_logs_in_order` |

**Must pass:** the channel is set to `1.96.0` and the full test suite is run. That toolchain is installed on this machine with `rustfmt` and `clippy`, so no download is triggered. This confirms a pin is a one-file change.

## Pin check

The channel of a scratch copy of the repository's real file is set to `1.96.0`, which is installed and differs from stable, with no other edit. `rustc --version` there must report 1.96.0, while `stable` elsewhere reports 1.98.1. No download is needed. The same check was measured during planning, and it is repeated at the Test Phase.
