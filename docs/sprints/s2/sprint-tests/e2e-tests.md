# Sprint 2 End-to-End Test Results

- **Status:** possible. All end-to-end coverage the test plan promised is implemented.
- **Tested head:** `56bd7577b677311d3f3f936bb2e7cd07649514db` (branch `dev`)
- **Runner:** `cargo test --all -- --nocapture`
- **Local result:** 54 passed, 0 failed, 0 ignored, on Windows 11 with
  `rustc 1.98.1`. That is the 51 end-to-end tests that closed sprint 1, run
  **unedited**, plus 3 new. `git diff --numstat 7b2cba7 56bd757 -- tests/cli.rs`
  reads `238 0`, insertions only.
- **CI result:** see the test report for the run on the tested head. The Linux
  leg printed no SKIP lines. The Windows leg printed the same four platform
  SKIPs as sprint 1, each with its reason.

The three new tests read files at run time through `CARGO_MANIFEST_DIR`, via
`repo_file`, which normalises CRLF. The Windows runner checks files out with
CRLF. For `rust-toolchain.toml`, which only these tests read, a deleted file
therefore fails one named test (mutation 1). `README.md` and the workflow are
also read with `include_str!` by unedited sprint 0 and sprint 1 tests. Deleting
either of those still breaks compilation of the test binary, which fails the
suite just as surely, but not as one named test.

## Revisions after the test critique

The first versions of the two T-001 tests matched plain substrings. The critic
showed that commenting out the update, install and version-log steps left them
green. That is the usual quiet way to revert a YAML step, and exactly what
INT-0005's last criterion says must fail. It was confirmed and fixed in
`60a529a`:

- **The tests now parse structure.** The workflow test reads the job's steps
  with comment lines dropped. The toolchain test reads only the `[toolchain]`
  table. The README test reads only the `## Toolchain` section.
- **The mutation check was widened** from the plan's eleven changes to 32, and
  re-run on the new tests. It now includes commenting out, disabling with
  `if:` or `continue-on-error:`, merging, and moving each gate.

Round 2 returned `proceed-with-caveats`. It verified every round-1 fix against
the code and CI, and found that keys **above** step level still passed:

- a job-level `continue-on-error`, or a matrix `exclude:`, could drop or excuse
  the Windows leg;
- `RUSTUP_TOOLCHAIN` or `rustup override` could outrank the toolchain file;
- `set +e` could defeat the bash log step.

`fb943c2` closes these, and six more mutations (rows 30–35) confirm it.

Round 3 returned `proceed-with-caveats` and confirmed the round-2 fixes. It
found one more way to override the toolchain file. rustup prefers a legacy,
extension-less `rust-toolchain` file in the same directory, so adding one would
silently void a pin. `56bd757` asserts that no such file exists, and row 39
confirms it. The whole table was re-run against `56bd757`. Rows 1–38 gave the
same messages as the run against `fb943c2`.

## T-001 — toolchain file and CI steps

| Test | EARS clause verified |
|------|----------------------|
| `test_toolchain_file_declares_channel_and_components` | No legacy `rust-toolchain` file, which rustup would prefer, exists beside it. Within the `[toolchain]` table only, with comments stripped: `channel` is a quoted, non-empty TOML string, in either `"…"` or `'…'` form, and `components` is a closed one-line array whose quoted items include `rustfmt` and `clippy`. It deliberately accepts any channel, so a pin stays a one-file change. That the channel is `stable` at close is a close-time inspection; see the test report. |
| `test_ci_updates_installs_and_logs_in_order` | The workflow is parsed into its steps, with comment lines dropped. **Each is its own step:** `run: rustc +stable --version`, `run: rustup update --no-self-update stable` and `run: rustup toolchain install --no-self-update` must each be a step made of exactly that one line. That rules out merging, adding an argument, and any `if:` or `continue-on-error:`. **Order:** they come in that order. **One log step:** a single step then holds `shell: bash` and all five of `rustc`, `cargo`, `cargo clippy`, `rustup` and `git --version`, with no `if:` or `continue-on-error:`. It comes after the install. **Gates:** each of the three gates — `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test --all -- --nocapture` — is a one-line step after that log. The log step also has no `set +` line, so bash's exit-on-error holds. **Elsewhere:** `rustup check`, `RUSTUP_TOOLCHAIN` and `rustup override` appear nowhere. `fail-fast: false`, `os: [ubuntu-latest, windows-latest]` and `runs-on: ${{ matrix.os }}` are present on non-comment lines. No non-comment line at any level — workflow, job, matrix or step — starts with `if:`, `continue-on-error:` or `exclude:`, so nothing can skip or excuse a leg or a step. |

## T-002 — README toolchain section

| Test | EARS clause verified |
|------|----------------------|
| `test_readme_documents_toolchain_policy` | Within the `## Toolchain` section only: each of the nine literal strings, one per required item, is present. They are `Track stable by default`, `rust-toolchain.toml`, `rustup update stable`, `rustup toolchain install`, `To pin`, `rustup 1.29`, `rustup self update`, `a new stable` and `git's version`. The update command must also come before the install, the order that actually brings a lagging stable level. |

The README section was also read in full at close; see the test report.

## Mutation check

- **Method.** Each change was applied to the real file, and only the named test
  was run: `cargo +stable test --test cli <name> -- --exact`. `+stable` keeps a
  mutated toolchain file from choosing the compiler. The file was then
  restored, and every restore was confirmed byte-identical.
- **What was mutated.** The working tree of `56bd757`: the tested head's
  `rust-toolchain.toml`, workflow and README, run against its tests.
- **What each row shows.** Every change failed at its intended assertion. No
  row failed to compile.

Rows 1–11 are the locked plan's mutations. Rows 12–29 and 36–38 were added
after critique round 1, rows 30–35 after round 2, and row 39 after round 3.

| # | Change | Test | Assertion message |
|---|--------|------|-------------------|
| 1 | `rust-toolchain.toml` deleted | toolchain | `rust-toolchain.toml must exist: The system cannot find the file specified. (os error 2)` |
| 2 | `channel` line removed | toolchain | `a quoted channel in [toolchain]` |
| 3 | `clippy` dropped from `components` | toolchain | `components must list clippy: ["rustfmt"]` |
| 4 | pre-update log step removed | workflow | `workflow must have the one-line step "run: rustc +stable --version"` |
| 5 | pre-update log moved after the update | workflow | `image version is logged before the update` |
| 6 | update step removed | workflow | `workflow must have the one-line step "run: rustup update --no-self-update stable"` |
| 7 | install step removed | workflow | `workflow must have the one-line step "run: rustup toolchain install --no-self-update"` |
| 8 | update and install swapped | workflow | `update comes before the install` |
| 9 | post-install log step removed | workflow | `a post-install rustc --version` |
| 10 | post-install log moved ahead of the install | workflow | `the toolchain is logged after the install` |
| 11 | `rustup check` added to the log step | workflow | `assertion failed: !wf.contains("rustup check")` |
| 12 | `rustfmt` dropped from `components` | toolchain | `components must list rustfmt: ["clippy"]` |
| 13 | `clippy` kept only in a trailing comment | toolchain | `components must list clippy: ["rustfmt"]` |
| 14 | `components` array left unclosed | toolchain | `components is a closed one-line array` |
| 15 | `channel` and `components` moved under another table | toolchain | `a non-empty [toolchain] table` |
| 16 | pre-update log step commented out | workflow | `workflow must have the one-line step "run: rustc +stable --version"` |
| 17 | update step commented out | workflow | `workflow must have the one-line step "run: rustup update --no-self-update stable"` |
| 18 | install step commented out | workflow | `workflow must have the one-line step "run: rustup toolchain install --no-self-update"` |
| 19 | post-install log step commented out | workflow | `a post-install rustc --version` |
| 20 | update and install merged into one `run: \|` step, originals commented out | workflow | `workflow must have the one-line step "run: rustup update --no-self-update stable"` |
| 21 | install step given `continue-on-error: true` | workflow | `workflow must have the one-line step "run: rustup toolchain install --no-self-update"` |
| 22 | install step given `if: runner.os == 'Linux'` | workflow | `workflow must have the one-line step "run: rustup toolchain install --no-self-update"` |
| 23 | update step given `if: false` | workflow | `workflow must have the one-line step "run: rustup update --no-self-update stable"` |
| 24 | log step given `continue-on-error: true` | workflow | `the version-log step must be neither conditional nor allowed to fail` |
| 25 | `git --version` moved to its own non-bash step | workflow | `the version-log step must hold "git --version": [...]` |
| 26 | clippy gate moved above the update | workflow | `"cargo clippy --all-targets -- -D warnings" runs after the toolchain is logged` |
| 27 | clippy gate given `continue-on-error: true` | workflow | `workflow must have the one-line step "run: cargo clippy --all-targets -- -D warnings"` |
| 28 | `runs-on` hard-coded to `ubuntu-latest` | workflow | `workflow must keep "runs-on: ${{ matrix.os }}"` |
| 29 | `windows-latest` dropped from the matrix | workflow | `workflow must keep "os: [ubuntu-latest, windows-latest]"` |
| 30 | job-level `continue-on-error: ${{ matrix.os == 'windows-latest' }}` | workflow | `no continue-on-error: may skip or excuse a leg or step: "continue-on-error: ${{ matrix.os == 'windows-latest' }}"` |
| 31 | matrix `exclude:` of `windows-latest` | workflow | `no exclude: may skip or excuse a leg or step: "exclude:"` |
| 32 | job-level `if: github.event_name == 'push'` | workflow | `no if: may skip or excuse a leg or step: "if: github.event_name == 'push'"` |
| 33 | workflow-level `env: RUSTUP_TOOLCHAIN: stable` | workflow | `workflow must not use "RUSTUP_TOOLCHAIN"` |
| 34 | `- run: rustup override set stable` step before the install | workflow | `workflow must not use "rustup override"` |
| 35 | `set +e` added to the version-log step | workflow | `the version-log step must keep bash's exit-on-error` |
| 36 | README: install before update in the command block | README | `the update comes before the install; the install alone updates nothing` |
| 37 | README: `To pin` present only outside the Toolchain section | README | `Toolchain section must say "To pin"` |
| 38 | README: `## Toolchain` heading removed | README | `a ## Toolchain section` |
| 39 | legacy `rust-toolchain` file (`1.96.0`) added beside `rust-toolchain.toml` | toolchain | `no legacy rust-toolchain file may outrank rust-toolchain.toml` |

"toolchain" is `test_toolchain_file_declares_channel_and_components`,
"workflow" is `test_ci_updates_installs_and_logs_in_order`, and "README" is
`test_readme_documents_toolchain_policy`.

### Must pass

| Change | Result |
|--------|--------|
| channel pinned as a TOML literal string, `channel = '1.96.0'` | toolchain test passed |
| channel written without spaces, `channel="1.98.1"` | toolchain test passed |
| channel set to `"1.96.0"`, full suite run with plain `cargo` so the file chooses the toolchain | `rustc --version` reported `rustc 1.96.0 (ac68faa20 2026-05-25)`; 68 unit and 54 end-to-end tests passed, 0 failed; no download |

`cargo +1.96.0 fmt --check` and
`cargo +1.96.0 clippy --all-targets -- -D warnings` were also clean. The
first-round run of the plan's eleven mutations, against the substring
versions of the tests at `f9913c8`, is superseded by this table.

## Close-time inspection

The test plan requires these to be observed on the tested head, not asserted
by a test.

- **The channel is `stable` at close.**
  `git show 56bd757:rust-toolchain.toml` has, on line 4, `channel = "stable"`. No `rust-toolchain` file exists at the root.
  The working tree matches.
- **No crate was added.** Literal `cargo tree` output at the tested head:

  ```
  mdeezl v0.1.0 (C:\Users\charl\MDeezl)
  ```

  Its automated counterpart, `test_manifest_dependencies_table_is_empty`, ran
  unedited and passed.
- **The README's Toolchain section says what the intent requires.** The whole
  section was read against INT-0005, not only searched for its nine strings.
  It:
  - states the policy, and says pinning is for when stability matters more;
  - says a `stable` file does not update anything;
  - gives `rustup update stable` then `rustup toolchain install`, in that
    order, says what each does, and says the first is machine-wide;
  - says pinning is a one-word change and how to revert it;
  - says a new stable can turn CI red, and that this is deliberate, with
    pinning as the remedy;
  - says git's version floats with the runner image and is logged;
  - names rustup 1.29 as the verified version, with `rustup self update` as the
    upgrade path.

  Each of the nine strings occurs exactly once in the file, inside that
  section. None of them occurred in the README at `7b2cba7`.

## Sprint 0 and sprint 1 tests

All 51 ran **unedited** and passed, both locally and on both CI legs. That
includes `test_ci_workflow_runs_tests_on_both_platforms`, the sprint 0 guard
on the two OS legs, which still passes against the edited workflow.
