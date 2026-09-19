# Sprint 2 Test Report

- **Verdict:** pass. All eight INT-0005 acceptance criteria are proved by
  executed tests, CI logs, or recorded inspection. One half of criterion 2 —
  lag correction observed in CI — was **not exercised**, because the runner
  images were already on the true current stable. It is reported separately
  below and is not counted as passed.
- **Tested head:** `56bd7577b677311d3f3f936bb2e7cd07649514db` (branch `dev`).
- **Runner:** `cargo test --all -- --nocapture`, the project's canonical suite.
- **CI:** [run 35426964502](https://github.com/crussella0129/MDeezl/actions/runs/35426964502)
  passed on `ubuntu-latest` and `windows-latest`. Both legs ran rustc 1.98.1,
  rustup 1.29.1 and git 2.55.0 (Windows: 2.55.0.windows.5).
- **Pinned CI:** [run 35425529632](https://github.com/crussella0129/MDeezl/actions/runs/35425529632)
  on throwaway PR #4 installed and ran 1.96.0 on both legs. It ran with the
  user's consent, and the PR was closed unmerged.
- **Local host:** Windows 11, rustc 1.98.1, rustup 1.29.0, git 2.54.0. Stable
  was moved here from 1.96.0 with the user's plan-time approval.
- **Critique:** [critique.md](critique.md). Four rounds: `block`, then
  `proceed-with-caveats` twice, then `clean`.

## Results

| Suite | Local (Windows) | `ubuntu-latest` | `windows-latest` | Detail |
|-------|-----------------|-----------------|------------------|--------|
| Unit | 68 passed | 68 passed | 68 passed | [unit-tests.md](unit-tests.md) |
| Integration | passed, pin check | passed, with the lag-correction half not exercised | passed, with the lag-correction half not exercised | [integration-tests.md](integration-tests.md) |
| End-to-end | 54 passed | 54 passed | 54 passed | [e2e-tests.md](e2e-tests.md) |

That is 122 tests. **All 119 tests that closed sprint 1 ran unedited and
passed.** `git diff --numstat 7b2cba7 56bd757 -- tests/cli.rs` reads `238 0`,
and `src/` is untouched. That was the plan's regression bar for INT-0001 and
INT-0004. The same 122 also passed under the 1.96.0 pin: locally, at the tested
head, and on both CI legs of the pinned run, whose tree held the first-round
tests.

The Linux leg printed no SKIP lines. The Windows leg printed the four platform
SKIPs sprint 1 recorded, each with its reason.

## Gates

| Gate | Result |
|------|--------|
| `cargo fmt --check` | clean: locally on 1.98.1 and 1.96.0; in CI on 1.98.1, and on 1.96.0 in the pinned run |
| `cargo clippy --all-targets -- -D warnings` | clean on the same four |
| `cargo tree` | exactly the line below, and nothing else |

```
mdeezl v0.1.0 (C:\Users\charl\MDeezl)
```

`test_manifest_dependencies_table_is_empty`, a sprint 0 test that ran
unedited, is the automated counterpart.

## Intent verification — INT-0005

| # | INT-0005 acceptance criterion | Verified by |
|---|-------------------------------|-------------|
| 1 | `rust-toolchain.toml` at the root names the channel, `stable` by default, and `rustfmt` and `clippy` | `test_toolchain_file_declares_channel_and_components` (mutations 1–3, 12–15, 39); **close-time inspection:** `git show 56bd757:rust-toolchain.toml` line 4 reads `channel = "stable"` |
| 2a | CI runs `rustup update --no-self-update stable`, then the no-argument `rustup toolchain install --no-self-update`, before any gate, each as its own step | `test_ci_updates_installs_and_logs_in_order` (mutations 4–10, 16–27, 30–35); both steps visible, in order, on both legs of all five sprint 2 runs |
| 2b | Under `stable`, CI runs the true current stable | **CI log gate, equality half:** in four stable runs on both legs, the post-install `rustc` equals the update step's reported `rustc 1.98.1`. That is the true current stable. |
| 2c | …not the runner image's pre-installed one, which lags the release | **Not exercised in CI.** Every update step reported `unchanged`, because the images had caught up to 1.98.1. The only evidence is one host measurement: 1.96.0 moved to 1.98.1, and the step reported `updated` ([research report, Correction](../sprint-research/research-report.md)). **Owned by backlog T-106.** |
| 2d | Under a pinned version, CI runs exactly that version | **Pinned CI gate:** on both legs, the no-argument install downloaded 1.96.0, and the post-install `rustc` reported `rustc 1.96.0 (ac68faa20 2026-05-25)` |
| 2e | Each command is its own step, so a failure in either fails the run on both shells | The structure is guarded by the workflow test. A one-line step with no `if:` or `continue-on-error:` at any level is covered by mutations 21–23 and 30–32. **Failure path deferred with evidence:** sprint 1's run 35325492936 failed on both legs at a one-command step, under pwsh and under `bash -e` ([integration-tests.md](integration-tests.md)). |
| 3 | Pinning is a one-line change for local and CI | **Local pin check** (scratch copy reports 1.96.0); **pass-mutation** (full suite under `channel = "1.96.0"`, 68 + 54 passed); **pinned CI gate** (a one-line diff, `git diff 6f07237 b720f05`) |
| 4 | Every run logs the image's stable before the update, the update's own report, and after the install `rustc`, `cargo`, `clippy`, `rustup` and `git`; no `rustup check` | `test_ci_updates_installs_and_logs_in_order` (mutations 4–5, 9–11, 19, 24–25, 35); all three log parts are present on both legs of every sprint 2 run |
| 5 | The README states the policy, the commands, how to pin, the verified rustup version, and the upgrade path | `test_readme_documents_toolchain_policy` (mutations 36–38); **close-time reading** of the whole section ([e2e-tests.md](e2e-tests.md)) |
| 6 | Git floats with the runner image and is not pinned; the README says so | `test_readme_documents_toolchain_policy` (`git's version`); the workflow installs no git; CI logs git 2.55.0 and 2.55.0.windows.5 |
| 7 | Both OS legs and `--nocapture` remain | `test_ci_updates_installs_and_logs_in_order` (mutations 28–31); `test_ci_workflow_runs_tests_on_both_platforms` (sprint 0, unedited); both legs ran in every sprint 2 run |
| 8 | A test catches a quiet revert without blocking a pin | 39 mutations, each failing its named test with the recorded assertion message; three pass-mutations pass: `'1.96.0'`, `channel="1.98.1"`, and the full suite under `"1.96.0"`. No test requires `stable`. |

**Non-goals held:**

- no crate was added (`cargo tree` above);
- no third-party CI action was added: the workflow still uses only `actions/checkout`;
- the gates run the same three commands as before;
- git is not pinned.

### Can INT-0005 be `realized` with 2c unexercised?

Yes. The recommendation is `realized`, with 2c carried as an open observation.

- **The outcome held in every CI run.** Criterion 2's observable outcome is that
  CI runs the true current stable. It held in every run. What went unobserved
  is the path taken when the image lags, and that depends on an external
  condition this project cannot produce.
- **The mechanism is rustup's own documented command,** measured on this host
  with the same rustup minor version, 1.29.0 against CI's 1.29.1.
- **The locked test plan anticipated this:** "`unchanged` means the path was not
  exercised". Its only stated condition for closing below `realized` was the
  user declining the pinned CI gate. The user accepted, and that gate passed.
- **The gap is owned.** INT-0005 records it as an open observation, and
  backlog T-106 owns attaching the first CI log that reads `updated`.

## EARS clause coverage

| Task / clause | Evidence |
|---------------|----------|
| T-001: the file declares `[toolchain]`, a channel, `rustfmt` and `clippy`; `stable` at close | toolchain test; close-time inspection |
| T-001: `rustc +stable --version` is logged before the update | workflow test; CI logs |
| T-001: update, then the no-argument install, each its own step, before any gate | workflow test; CI logs |
| T-001: one bash step logs the five versions before `cargo fmt --check`; no `rustup check` | workflow test; CI logs |
| T-001: under stable, the post-install `rustc` equals the update step's version; `unchanged` recorded as not exercised | CI log gate, four runs × two legs; recorded as not exercised |
| T-001: a pin to an installed version makes `rustc` report it | local pin check |
| T-001: a pin to a version absent from the runners is installed on both legs | pinned CI gate (consented) |
| T-001: both OS legs, `fail-fast: false` and `--nocapture` remain | workflow test; sprint 0 test |
| T-001: listed changes fail a named test; `stable` not required | mutation table rows 1–11 as planned, plus 28 more; pass-mutations |
| T-001: `cargo tree` shows only this crate | literal output above; sprint 0 test |
| T-002: the nine literal strings | README test, scoped to the section and ordered; close-time reading |

## Findings

- **The runner images caught up between sprints.** Sprint 1 saw 1.98.0; every
  sprint 2 run found 1.98.1 already installed, so the update path never ran.
  That is why 2c is unexercised, not a defect. The pre-update log is what made
  this visible.
- **The host still has 1.96.0 installed.** It is used as the local pin target
  and does not affect `stable`.
- **The pinned run proves the no-argument install is the installer.** The pin
  arrived through `rustup toolchain install --no-self-update` itself
  (`downloading 5 components`), not through auto-install on first use. The
  `+stable` pre-log stayed on 1.98.1 under the pin, as designed.
- **Earlier test rounds used different heads.** Runs 35414508093 (`6f07237`),
  35426031839 (`60a529a`) and 35426602219 (`fb943c2`) all passed on both legs.
  They carried earlier versions of the tests, and are listed for provenance.

## Notable defects caught before release

All were caught by the test critique, and none was in shipped behaviour:
`src/` is untouched. Each was a way to quietly undo the policy while the
guard stayed green:

- commenting out the update, install or version-log step;
- `if:`, `continue-on-error:` or `exclude:` at step, job or matrix level;
- moving the clippy or test gate above the update;
- `RUSTUP_TOOLCHAIN`, `rustup override`, or a legacy `rust-toolchain` file
  outranking the pin;
- `set +e` in the version-log step;
- a malformed `components` array, or `clippy` present only in a comment;
- the README giving the two commands in the order that does not work.
