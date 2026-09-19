# Test Critique — Sprint 2

Read-only test critic rounds from the installed bundle's
`prompts/test-critic.md`. The primary agent's resolution follows each concern.

| Round | Verdict | Concerns |
|-------|---------|----------|
| 1 | `block` | 12 |
| 2 | `proceed-with-caveats` | 3 |

Round 2 re-verified all twelve round-1 resolutions against `60a529a`, git, and
the four cited CI runs, and found them to hold as recorded. The round-2
concerns follow the round-1 ones.

## Concerns

### C-001: Workflow-order test passes when the new steps are commented out
- **Where:** `tests/cli.rs` `test_ci_updates_installs_and_logs_in_order`; `INT-0005` last criterion
- **Quote:** `let update = pos("- run: rustup update --no-self-update stable\n", 0);`
- **Failure mode:** weak-assertion
- **Why it matters:** Plain substring search matches `      # - run: rustup update …`, so commenting out all four new steps (the usual quiet revert) leaves the test green.
- **Suggested response:** tighten-assertion
- **Resolution:** confirmed, and fixed in `60a529a`. The test now parses the job's steps with comment lines dropped. The pre-log, update and install must each be a step consisting of exactly that one line. Mutations 16–20 (comment out each step; merge update and install with the originals commented out) now fail.

### C-002: "Before any gate" is checked against `cargo fmt --check` only
- **Where:** build-plan T-001 EARS 3 and 4
- **Quote:** `let fmt = pos("cargo fmt --check", 0); assert!(log_end < fmt, ...)`
- **Failure mode:** weak-assertion
- **Why it matters:** Clippy or test could move above the update and the test would pass. The other four `--version` lines could leave the bash step.
- **Suggested response:** tighten-assertion
- **Resolution:** fixed. All three gates must be one-line steps after the log step. All five `--version` lines and `shell: bash` must be in one step. Mutations 25–27 now fail.

### C-003: A step can be disabled with `if:` or softened with `continue-on-error:`
- **Where:** `INT-0005` criterion 2
- **Quote:** "Each of the two commands is its own CI step, so a failure in either fails the run on both shells."
- **Failure mode:** weak-assertion
- **Why it matters:** Either key under the install step keeps every needle in place.
- **Suggested response:** tighten-assertion
- **Resolution:** fixed. A one-line step cannot carry either key, and the log step is checked for both. Mutations 21–24 and 27 now fail.

### C-004: The failure path of the update and install steps was never executed
- **Where:** `INT-0005` criterion 2; `integration-tests.md`
- **Quote:** "Conclusion: success on both legs."
- **Failure mode:** negative-path
- **Why it matters:** Only success paths were observed; the claim that a failing command fails the step on `pwsh` rests on the runner's wrapper.
- **Suggested response:** add-test (consent-gated) or defer-with-rationale
- **Resolution:** deferred with evidence; see `integration-tests.md`, "Failure path of a one-command step". Sprint 1's run 35325492936 failed on both legs at the one-command `- run: cargo clippy …` step. It logged `Process completed with exit code 1.` under pwsh and `exit code 101` under `bash -e`, and both jobs concluded `failure`. A second throwaway pull request would need fresh consent, and was not opened.

### C-005: The update path ran `unchanged` in both CI runs
- **Where:** `integration-tests.md`; `INT-0005` criterion 2
- **Quote:** "**Result:** passed. The CI log gate held on both legs, with its update path **not exercised**"
- **Failure mode:** intent-coverage
- **Why it matters:** A top-line "passed" could be aggregated as lag correction verified in CI; the follow-up has no owner.
- **Suggested response:** defer-with-rationale
- **Resolution:** the record's result is now split by part, with the lag-correction half marked not exercised. INT-0005's Consequences gained an "Open observation" asking for the first CI log that reads `updated`. The test report gives that half its own row.

### C-006: The toolchain test accepts malformed files and rejects equivalent pins
- **Where:** `test_toolchain_file_declares_channel_and_components`
- **Quote:** `.find_map(|l| l.strip_prefix("channel = \""))`
- **Failure mode:** weak-assertion
- **Why it matters:** An unclosed array, `clippy` only in a comment, or keys under another table all passed, while `channel = '1.98.1'` failed.
- **Suggested response:** tighten-assertion
- **Resolution:** fixed. The test reads only the `[toolchain]` table with comments stripped, requires a closed array of quoted items, and accepts both TOML string forms. Mutations 12–15 fail; the two quote-style pass-mutations pass.

### C-007: The README test checks presence, not what the README says
- **Where:** `test_readme_documents_toolchain_policy`
- **Quote:** `assert!(readme.contains(needle), ...)`
- **Failure mode:** weak-assertion
- **Why it matters:** Swapping the two commands, which breaks the procedure, would still pass.
- **Suggested response:** tighten-assertion
- **Resolution:** fixed. The search is scoped to the `## Toolchain` section, and the update must come before the install. Mutations 36–38 fail (numbered 30–32 in round 1). A close-time semantic reading is recorded in `e2e-tests.md`.

### C-008: Mutation results lack a commit and the failure output
- **Where:** `e2e-tests.md` Mutation check
- **Quote:** "| `update step removed` | … | failed, as required |"
- **Failure mode:** evidence-drift
- **Why it matters:** A compile error would look the same as the intended assertion firing.
- **Suggested response:** tighten-assertion (record)
- **Resolution:** fixed. All 32 mutations were re-run against `60a529a`. Each row quotes its assertion message, none failed to compile, and the must-pass run gives 68 + 54.

### C-009: The two-OS guard checks strings, not the legs
- **Where:** build-plan EARS 8
- **Quote:** `assert!(wf.contains("fail-fast: false"));`
- **Failure mode:** weak-assertion
- **Why it matters:** Hard-coding `runs-on: ubuntu-latest` passed.
- **Suggested response:** tighten-assertion
- **Resolution:** fixed. `fail-fast: false`, `os: [ubuntu-latest, windows-latest]` and `runs-on: ${{ matrix.os }}` must all be on non-comment lines. Mutations 28–29 fail.

### C-010: The pinned run is not tied to the tested head
- **Where:** `integration-tests.md` pinned CI gate
- **Quote:** "Its only change is `channel = "stable"` → `channel = "1.96.0"`, in head `b720f05…`."
- **Failure mode:** evidence-drift
- **Why it matters:** A `pull_request` run builds a merge ref, and the branch is gone.
- **Suggested response:** tighten-assertion (record)
- **Resolution:** fixed. The record gives the parent `6f07237`, the one-line diff, and the checked-out merge `c4f8045cd4c9735152db465c9b6319b27a93b205` quoted from both legs' checkout logs. It also states that this run used the first-round tests.

### C-011: Close-time items and intent evidence links are missing
- **Where:** test-plan rows for EARS 1 and EARS 10; `INT-0005` header
- **Quote:** "`cargo tree` reports `mdeezl v0.1.0` and nothing else."
- **Failure mode:** evidence-drift
- **Why it matters:** The plan promised a literal `cargo tree` output and a close-time inspection of the channel.
- **Suggested response:** add-test (record)
- **Resolution:** `e2e-tests.md` now has a "Close-time inspection" section, which quotes the literal `cargo tree` output and `channel = "stable"` at `60a529a`. The Test evidence links are attached to INT-0005 when the test report is written, as the phase requires.

### C-012: Two claims overstate the evidence
- **Where:** `e2e-tests.md` and `integration-tests.md` intros
- **Quote:** "A deleted file therefore fails one named test instead of breaking compilation" and "Every quoted line is copied from the job log."
- **Failure mode:** evidence-drift
- **Why it matters:** `README.md` and the workflow are still read with `include_str!` by unedited tests; several cells were abridged.
- **Suggested response:** tighten-assertion (record)
- **Resolution:** fixed. The first claim is now limited to `rust-toolchain.toml`. Abridged cells are marked, and the intro says so.

### R2 C-001: Job-level keys can drop or excuse the Windows leg with every checked line intact
- **Where:** `test_ci_updates_installs_and_logs_in_order`; `INT-0005` criteria 2 and 7
- **Quote:** "The structural test now guarantees each rustup step stays a one-line step, with no `continue-on-error:` or `if:` that could excuse or skip a failure"
- **Failure mode:** weak-assertion
- **Why it matters:** A job-level `continue-on-error: ${{ matrix.os == 'windows-latest' }}` or a matrix `exclude:` passed. `workflow_steps` reads only step bodies.
- **Suggested response:** tighten-assertion
- **Resolution:** fixed in `fb943c2`. No non-comment line at any level may start with `if:`, `continue-on-error:` or `exclude:`. Mutations 30–32 fail.

### R2 C-002: Toolchain overrides and `set +e` leave every checked line in place
- **Where:** `test_ci_updates_installs_and_logs_in_order`; build-plan T-001 EARS 4
- **Quote:** "the workflow stops updating, stops installing from the file, or stops logging versions"
- **Failure mode:** weak-assertion
- **Why it matters:** `RUSTUP_TOOLCHAIN` or `rustup override` would outrank a pin, and `set +e` would defeat the bash log step.
- **Suggested response:** tighten-assertion
- **Resolution:** fixed in `fb943c2`. `RUSTUP_TOOLCHAIN` and `rustup override` may not appear anywhere, and the log step may hold no `set +` line. Mutations 33–35 fail. The full table of 38 was re-run against `fb943c2`. CI run 35426602219 passed on both legs at `fb943c2`.

### R2 C-003: The lag-correction open observation has no trigger or owner
- **Where:** `INT-0005` Consequences, "Open observation"; `docs/work/tasks.md`
- **Quote:** "The first CI log whose update step reads `updated` should be attached here as evidence when it appears."
- **Failure mode:** intent-coverage
- **Why it matters:** If INT-0005 closes `realized`, nobody is prompted to revisit the observation.
- **Suggested response:** defer-with-rationale
- **Resolution:** backlog item T-106 now owns it, and INT-0005's observation names it. The test report states explicitly whether INT-0005 may be `realized` while lag correction rests on one host measurement.

## Confidence
proceed-with-caveats
