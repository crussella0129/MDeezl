# Sprint 2 Integration Test Results

- **Intents:** [INT-0005](../../../intents/INT-0005-reproducible-toolchain.md)
- **Result, by part:**
  - **CI log gate, equality half:** passed on both legs, in two runs.
  - **CI log gate, lag-correction half:** **not exercised** in CI. Both update
    steps reported `unchanged`. This half is not counted as passed; see below.
  - **Pinned CI gate:** passed on both legs. It was consent-gated.
  - **Local pin check:** passed.
  - **Failure path of a one-command step:** not executed for the rustup steps.
    It is evidenced by an earlier real failure of the same step shape; see the
    last section.

The integration tests here are the real composition the intent is about: the
toolchain file, rustup, and the GitHub runner images. They are observed in
real CI logs rather than asserted by a test binary, because the runner is the
component under test. Cells in code font are copied from the job log. Cells
marked "abridged" are shortened with `…` or summarise several lines; the full
logs remain on the linked runs.

## CI log gate — tracking stable

- **Runs:**
  - [35414508093](https://github.com/crussella0129/MDeezl/actions/runs/35414508093),
    a push to `dev` at `6f0723712e24d461e37cbff56126294ea6e36ef4`, with the
    first-round tests;
  - [35426031839](https://github.com/crussella0129/MDeezl/actions/runs/35426031839),
    a push to `dev` at `60a529a83a4c00032286efe1772ccf7216342800`, with the
    round-1 structural tests;
  - [35426602219](https://github.com/crussella0129/MDeezl/actions/runs/35426602219),
    a push to `dev` at `fb943c2074bf4a6315b6b8adca4b103445c557c3`, **the tested
    head**, with the round-2 tests.
- **Workflow and toolchain file are the same in all three.** The heads differ
  only in `tests/cli.rs` and the Book's records.
- **Conclusion:** success on both legs of all three runs. The table quotes the
  first run; the tested-head run's lines are listed after it.

| Log line | `ubuntu-latest` (job 105820355337) | `windows-latest` (job 105820355363) |
|----------|-----------------------------------|-------------------------------------|
| 1. pre-update `rustc +stable --version` | `rustc 1.98.1 (48a229cea 2026-09-01)` | `rustc 1.98.1 (48a229cea 2026-09-01)` |
| 2. update step's own report | `stable-x86_64-unknown-linux-gnu unchanged - rustc 1.98.1 (48a229cea 2026-09-01)` | `stable-x86_64-pc-windows-msvc unchanged - rustc 1.98.1 (48a229cea 2026-09-01)` |
| 3. post-install `rustc --version` | `rustc 1.98.1 (48a229cea 2026-09-01)` | `rustc 1.98.1 (48a229cea 2026-09-01)` |
| cargo / clippy / rustup (abridged) | `cargo 1.98.1`, `clippy 0.1.98`, `rustup 1.29.1` | `cargo 1.98.1`, `clippy 0.1.98`, `rustup 1.29.1` |
| git | `git version 2.55.0` | `git version 2.55.0.windows.5` |

On both legs, line 3 equals the version `X` in line 2. That satisfies the
equality half of the EARS clause.

**The tested-head run** (35426602219: jobs 105853547277 on `ubuntu-latest` and
105853547372 on `windows-latest`) logged the same sequence on both legs, as did
run 35426031839 (jobs 105852014420 and 105852014191):

- pre-update: `rustc 1.98.1 (48a229cea 2026-09-01)`;
- update step: `stable-x86_64-unknown-linux-gnu unchanged - rustc 1.98.1 (48a229cea 2026-09-01)`
  on Linux, and `stable-x86_64-pc-windows-msvc unchanged - rustc 1.98.1 (48a229cea 2026-09-01)`
  on Windows;
- post-install: `rustc 1.98.1 (48a229cea 2026-09-01)`, `cargo 1.98.1 (797e8a9bc 2026-08-05)`,
  `clippy 0.1.98 (48a229ceae 2026-09-01)` and `rustup 1.29.1 (d95a37b6a 2026-08-13)`;
- git: `git version 2.55.0` on Linux and `git version 2.55.0.windows.5` on
  Windows.

Tests: `68 passed` and `54 passed` on both legs. There are no SKIP lines on
Linux, and the four platform SKIPs on Windows, each printing its reason.

**The update path was not exercised in these runs.** Every update step reports
`unchanged`: the runner images had already moved from 1.98.0, which sprint 1
saw, to 1.98.1, the true current stable. The test plan says an `unchanged`
leg must be recorded as not exercised, not counted as passed. What these runs
do show:

- the update step runs, exits 0, and reports its version in the required form;
- the post-install toolchain is the true current stable.

What it does not show is `rustup update stable` actually moving a lagging
image forward in CI. That behaviour was measured once, on the development host
during planning: 1.96.0 moved to 1.98.1, and the step reported
`updated - rustc 1.98.1`. See the research report's
[Correction section](../sprint-research/research-report.md). It
will be exercised in CI the first time a new stable ships before the runner
images pick it up, and the log line above will then read `updated`.
[INT-0005](../../../intents/INT-0005-reproducible-toolchain.md) records this as
an open observation, so that the first CI log reading `updated` is attached as
evidence when it appears.

## Pinned CI gate — consent-gated

- **Consent:** the user chose "Run it" at the Test Phase, on 2026-09-19.
- **Pull request:** [#4](https://github.com/crussella0129/MDeezl/pull/4), a
  draft against `dev` titled "[throwaway, do not merge]". Its only change is
  `channel = "stable"` → `channel = "1.96.0"`, in head
  `b720f05d39984795c59aa4bb0a9430f2308b6caf`. That version is absent from both
  runner images.
  - **Parent:** `b720f05`'s parent is `6f0723712e24d461e37cbff56126294ea6e36ef4`,
    which was then the tip of `dev`.
  - **The diff:** `git diff 6f07237 b720f05` is exactly the one `channel` line.
  - **What CI built:** both legs' checkout logs read `HEAD is now at c4f8045
    Merge b720f05… into 6f07237…`, and `git log -1` gave
    `c4f8045cd4c9735152db465c9b6319b27a93b205`. The tree built was therefore
    `6f07237` plus the pin.
- **Run:** [35425529632](https://github.com/crussella0129/MDeezl/actions/runs/35425529632), `pull_request` event.
- **Conclusion:** success on both legs.
- **Clean-up:** PR #4 was closed **unmerged** (`mergedAt` is null), and
  `throwaway/s2-pinned-ci-gate` was deleted on the remote. `git ls-remote`
  confirms it is gone.

| Log line | `ubuntu-latest` (job 105850676768) | `windows-latest` (job 105850676685) |
|----------|-----------------------------------|-------------------------------------|
| pre-update `rustc +stable --version` | `rustc 1.98.1 (48a229cea 2026-09-01)` | `rustc 1.98.1 (48a229cea 2026-09-01)` |
| update step (abridged) | `stable-x86_64-unknown-linux-gnu unchanged - rustc 1.98.1 …` | `stable-x86_64-pc-windows-msvc unchanged - rustc 1.98.1 …` |
| no-argument install (three lines) | `info: downloading 5 components` / `info: the active toolchain `1.96.0-x86_64-unknown-linux-gnu` has been installed` / `info: it's active because: overridden by '/home/runner/work/MDeezl/MDeezl/rust-toolchain.toml'` | `info: downloading 5 components` / `info: the active toolchain `1.96.0-x86_64-pc-windows-msvc` has been installed` / `info: it's active because: overridden by 'D:\a\MDeezl\MDeezl\rust-toolchain.toml'` |
| post-install `rustc --version` | `rustc 1.96.0 (ac68faa20 2026-05-25)` | `rustc 1.96.0 (ac68faa20 2026-05-25)` |
| cargo / clippy (abridged) | `cargo 1.96.0`, `clippy 0.1.96` | `cargo 1.96.0`, `clippy 0.1.96` |
| gates (summary) | fmt, clippy `-D warnings` and tests green; 68 + 54 passed | fmt, clippy `-D warnings` and tests green; 68 + 54 passed |

This verifies the pinned half of the INT-0005 criteria. A one-word change to
the file made both legs install exactly the pinned version before the gates,
and then run on it. The install came from the no-argument
`rustup toolchain install --no-self-update` step itself, not from auto-install
on first use. The `+stable` pre-update reading also stayed at 1.98.1 under the
pin, which shows that `+stable` keeps a pinned file from installing itself into
that reading.

This run used the first-round tests at `6f07237`. The structural tests of the
tested head `fb943c2` read files only, and do not depend on the compiler
version. They were run under the same 1.96.0 pin locally, in the full-suite
pass-mutation: 68 + 54 passed. They were not re-run under a pin in CI, because the user's consent
covered one throwaway pull request.

## Pin check — local

- **Arrangement:** a scratch directory holding a copy of the repository's real
  `rust-toolchain.toml`, with the channel changed to `1.96.0` and nothing else.
  `diff` against the original shows exactly the one `channel` line.
- **Observed there:** `rustc 1.96.0 (ac68faa20 2026-05-25)`, and
  `rustup show active-toolchain` reported
  `1.96.0-x86_64-pc-windows-msvc (overridden by '…\pincheck\rust-toolchain.toml')`.
- **Observed in the repository**, which is on `stable`:
  `rustc 1.98.1 (48a229cea 2026-09-01)`.
- No download was needed, because 1.96.0 is installed on this host.

## Failure path of a one-command step

INT-0005 says each rustup command is its own step "so a failure in either
fails the run on both shells". This sprint's runs only saw both commands
succeed. The failing half was not executed for the rustup steps: a second
throwaway pull request, pinned to a version that does not exist, would need
fresh consent. It is deferred on the following evidence.

- **The same step shape has failed for real on both shells.** Sprint 1's run
  [35325492936](https://github.com/crussella0129/MDeezl/actions/runs/35325492936)
  failed on both legs at the one-command step
  `- run: cargo clippy --all-targets -- -D warnings`. On `windows-latest`,
  under `shell: C:\Program Files\PowerShell\7\pwsh.EXE -command ". '{0}'"`,
  the step logged `##[error]Process completed with exit code 1.` On
  `ubuntu-latest`, under `shell: /usr/bin/bash -e {0}`, it logged
  `##[error]Process completed with exit code 101.` Both jobs concluded
  `failure`.
- **The runner fails the step on a non-zero exit, whatever the command.** A
  one-line `- run: rustup …` step takes the same path on the same default
  shells.
- **Rustup does exit non-zero.** Planning measured `rustup check` exiting 100
  on this host. That is why it is absent from the workflow.

The structural test now guarantees each rustup step stays a one-line step, with
no `continue-on-error:` or `if:` that could excuse or skip a failure (mutations
21–23 in [e2e-tests.md](e2e-tests.md)). It also guarantees that no
`continue-on-error:`, `if:` or `exclude:` exists at job or matrix level
(mutations 30–32), so a leg cannot be excused or dropped either.
