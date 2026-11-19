# Sprint 3 Integration Test Results

- **Intents:** [INT-0006](../../../intents/INT-0006-desktop-gui.md),
  [INT-0002](../../../intents/INT-0002-git-and-remote-sources.md),
  [INT-0005](../../../intents/INT-0005-reproducible-toolchain.md) (its guard)
- **Tested head:** `77b0c00570c978b4d12ea2b8d40fa88816f285e2` (branch `dev`)
- **Result, by part:**
  - **Engine ↔ CLI:** passed.
  - **GUI ↔ engine:** passed.
  - **CI observation, both legs:** passed. Both compiled `mdeezl-gui` and ran
    its tests, and neither installed a system package.
  - **Gate mutation check:** passed. All 49 changes fail their named test.

The first two are the composition this sprint is about: the GUI and the CLI
must produce the same bytes, and they do because they call one function.

## Engine ↔ CLI

`test_bundle_matches_cli_output` (`tests/cli.rs`, using the library):

- the fixture is a git repository with a `.gitignore`, and the test runs every
  wrap mode with gitignore on and off — six combinations;
- `mdeezl::bundle(&opts).document` equals the bytes the CLI writes with `-o`
  for the same options, and `name` equals the fixture folder's name;
- **both sides run under the same environment**: the CLI child inherits the
  test process's environment unchanged, with no `isolate()` on either side, so
  host git configuration affects both equally and cannot hide a difference.

## GUI ↔ engine

`test_generate_writes_engine_bytes` (`gui/tests/app.rs`, egui_kittest). The
harness is `Harness::new_eframe` holding the real `App`:

1. a fixture path is typed into the source field through AccessKit, the output
   folder is set on the state, and `inline` is picked;
2. `Generate` is clicked;
3. the test polls with `harness.step()` and a 10 ms sleep until the app stops
   working, or 10 s pass;
4. `<out>/<name>.md` equals `mdeezl::bundle` for those options, the status
   contains `Wrote` and the formatted size, and no `.mdeezl-tmp` file remains.

Together with the row above, GUI output equals CLI output through the one
shared function, by construction rather than by comparison of two renderers.

## CI observation

The tested head's run is
[44558109833](https://github.com/crussella0129/MDeezl/actions/runs/44558109833),
a push to `dev`, which concluded **success on both legs**: job 135679613205 on
`ubuntu-latest` and job 135679612560 on `windows-latest`.

| Log line | `ubuntu-latest` | `windows-latest` |
|----------|-----------------|------------------|
| pre-update `rustc +stable --version` | `rustc 1.100.0 (0ef1bc386 2026-11-09)` | `rustc 1.100.0 (0ef1bc386 2026-11-09)` |
| the update step's own report | `stable-x86_64-unknown-linux-gnu unchanged - rustc 1.100.0 (0ef1bc386 2026-11-09)` | `stable-x86_64-pc-windows-msvc unchanged - rustc 1.100.0 (0ef1bc386 2026-11-09)` |
| post-install toolchain | `rustc 1.100.0`, `cargo 1.100.0`, `clippy 0.1.100`; rustup is the image's own 1.29.3, because the update step passes `--no-self-update` | the same |
| git | `git version 2.56.1` | `git version 2.56.1.windows.1` |
| the GUI crate is compiled | `Running unittests src/lib.rs (…/mdeezl_gui-…)` | `Running unittests src\lib.rs (…\mdeezl_gui-….exe)` |
| its tests run | `31 passed` | `31 passed` |
| engine unit tests | `79 passed` | `79 passed` |
| CLI end-to-end tests | `67 passed; 1 ignored` | `67 passed; 1 ignored` |
| SKIP lines | 1: `SKIP test_git_commands_set_create_no_window: Windows only` | 4, the platform SKIPs sprint 1 recorded, each with its reason |
| package installs | none: no `apt-get`, `choco`, `brew` or `vcpkg` line anywhere in the log | none |

**This phase closes sprint 2's open observation — but not with this run.**
INT-0005's lag-correction half, that `rustup update stable` actually moves a
lagging runner image forward, was recorded in sprint 2 as **not exercised**,
because every update step had reported `unchanged`; the chapter asked for the
first CI log reading `updated` to be attached when it appeared. Two runs
earlier in this same phase are that log, on both legs:

| Run | Head | Both legs' update step |
|-----|------|------------------------|
| [44550746892](https://github.com/crussella0129/MDeezl/actions/runs/44550746892) | `c92ab73` | `updated - rustc 1.100.0 (0ef1bc386 2026-11-09) (from rustc 1.99.1 (58cf8f9c3 2026-10-13))` |
| [44554527551](https://github.com/crussella0129/MDeezl/actions/runs/44554527551) | `cb11527` | the same, on both legs |

The images carried 1.99.1, the update step moved each to 1.100.0, and the
gates then ran on 1.100.0 — the behaviour sprint 2 could only reason about.
By the tested head's run the images had caught up, so its own step reads
`unchanged`, which is the other half of the same clause. Both readings
therefore occur inside this one phase, hours apart, and the gates were green
under each. The suite also passes locally on 1.99.1, so this head is green on
two different stables.

The absent package install is the measurable part of T-005's claim that the
Linux windowing, GL and D-Bus libraries these features reach are loaded at run
time. The Linux leg's single SKIP is this sprint's own Windows-only test
announcing itself, which is the behaviour T-002 specified.

**The matrix earned its keep in this phase.** The first run after the critique
fixes, [44549818381](https://github.com/crussella0129/MDeezl/actions/runs/44549818381)
at `930a03e`, was green on Linux and **red on Windows**: the new source check
in `test_git_commands_set_create_no_window` split on `"\n#[cfg(test)]\nmod tests {"`,
and the Windows runner checks the file out with CRLF, which `include_str!`
embeds as it lies on disk. The fix normalises first, and was proved locally
against the file rewritten both ways. That is exactly the single-platform blind
spot [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md)
keeps two legs for.

## Gate mutation check

- **Method.** Each change is applied to the real file, and only the named test
  is run: `cargo +stable test --test cli <name> -- --exact`. `+stable` keeps a
  mutated toolchain file from choosing the compiler. The file is then restored
  from bytes held in memory, and every restore is verified by SHA-256 against
  the original. `git status` is clean afterwards.
- **What was mutated.** The tested head's `rust-toolchain.toml`, workflow and
  README, run against its tests.
- **What each row shows.** Every change failed at its intended assertion, and
  no row failed to compile. **49 rows, 0 not caught.**

Rows 1–39 are sprint 2's table, re-run here against the new workspace gate
lines. Rows 40 and 41 restore the pre-workspace `fmt` and `clippy` lines, which
the test plan requires to fail. **Rows 42–49 are new, from the four critique
rounds**, and every one of them is an *addition* rather than a change — the
shape nothing caught before, because each earlier assertion says some step
must be *present*:

- **42** adds a step. The guard compares the job's step list exactly, in order.
- **43** adds a `with:` key to an existing step. The comparison covers every
  line of every step, not just the first.
- **44** adds a whole second job. `workflow_steps` reads one job's steps, so
  nothing above it could see that.
- **45** adds a job-level `container:`. This is the one that matters most: a
  container image with the libraries baked in would keep the "no package
  install" promise true in the log and false in fact.
- **46** drops `dev` from the branches CI runs on, which would quietly stop
  the gate running at all.
- **47** adds a workflow-level `permissions:` key.

- **48** writes that same `container:` *after* the steps. YAML does not care
  about key order, and rows 42–47's checks all stopped at `steps:`, so this
  one slipped through them.
- **49** adds a second workflow file with its own job, which the guard could
  not see at all because it reads one file.

Rows 45–49 are the "and no key" half of T-005's clause. Until round 3 it was
asserted only inside the job's `steps:` block, and until round 4 only before
`steps:` — each round moved the boundary out to where the next hiding place
was.

The step-list comparison is made last, where a removed or disabled step still
fails at its own assertion first and the message names which one.

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
| 20 | update and install merged into one `run: |` step, originals commented out | workflow | `workflow must have the one-line step "run: rustup update --no-self-update stable"` |
| 21 | install step given `continue-on-error: true` | workflow | `workflow must have the one-line step "run: rustup toolchain install --no-self-update"` |
| 22 | install step given `if: runner.os == 'Linux'` | workflow | `workflow must have the one-line step "run: rustup toolchain install --no-self-update"` |
| 23 | update step given `if: false` | workflow | `workflow must have the one-line step "run: rustup update --no-self-update stable"` |
| 24 | log step given `continue-on-error: true` | workflow | `the version-log step must be neither conditional nor allowed to fail` |
| 25 | `git --version` moved to its own non-bash step | workflow | `the version-log step must hold "git --version": ["shell: bash", "run: \|", "rustc --version", "cargo --version", "cargo clippy --version", "rustup --version"]` |
| 26 | clippy gate moved above the update | workflow | `"cargo clippy --workspace --all-targets -- -D warnings" runs after the toolchain is logged` |
| 27 | clippy gate given `continue-on-error: true` | workflow | `workflow must have the one-line step "run: cargo clippy --workspace --all-targets -- -D warnings"` |
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
| 40 | `- run: cargo fmt --check`, the pre-workspace gate line | workflow | `workflow must have the one-line step "run: cargo fmt --all --check"` |
| 41 | `- run: cargo clippy --all-targets -- -D warnings`, the pre-workspace gate line | workflow | `workflow must have the one-line step "run: cargo clippy --workspace --all-targets -- -D warnings"` |
| 42 | a `sudo apt-get install -y libgtk-3-dev` step added before the gates | workflow | `the job's steps, in order, with no other step and no other key` |
| 43 | a `with: fetch-depth: 0` key added to the checkout step | workflow | `the job's steps, in order, with no other step and no other key` |
| 44 | a second job that installs a package | workflow | `exactly one job, which is the one above` |
| 45 | a job-level `container:` with the packages baked in | workflow | `the job's own keys, wherever they sit, and no others` |
| 46 | `dev` dropped from the branches CI runs on | workflow | `when CI runs, and on which branches` |
| 47 | a workflow-level `permissions:` key added | workflow | `the workflow's own keys, and no others` |
| 48 | a job-level `container:` written *after* the steps, where key order hides it | workflow | `the job's own keys, wherever they sit, and no others` |
| 49 | a second workflow file holding its own job | workflow | `one workflow file, which is the one asserted above` |

"toolchain" is `test_toolchain_file_declares_channel_and_components`,
"workflow" is `test_ci_updates_installs_and_logs_in_order`, and "README" is
`test_readme_documents_toolchain_policy`.

Rows 36 and 38 are the two that read as "NOT CAUGHT" during T-005's build,
when stale CRLF working copies left the script's LF patterns matching nothing.
This run wrote every mutation as bytes with explicit LF and verified each
restore by hash, and both rows failed as intended.

### The guard test's own changes

`git diff --numstat 24442f1 <head> -- tests/cli.rs` is insertions plus exactly
two deletions, the two gate lines T-005 replaced:

```
-        "cargo fmt --check",
-        "cargo clippy --all-targets -- -D warnings",
```

The critique then added the exact-step-list assertion described above. No
earlier assertion was weakened to make room for either change.
