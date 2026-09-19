# Sprint 2 Unit Test Results

- **Tested head:** `60a529a83a4c00032286efe1772ccf7216342800` (branch `dev`)
- **Runner:** `cargo test --all -- --nocapture`
- **Host:** Windows 11, `rustc 1.98.1` / `cargo 1.98.1`, `rustup 1.29.0`, `git 2.54.0.windows.1`. CI ran the same Rust with `rustup 1.29.1` and git 2.55.0; see the test report.
- **Result:** 68 passed, 0 failed, 0 ignored. That is the same 68 unit tests
  that closed sprint 1, **all unedited**. This sprint added none.
- **Gates:** `cargo fmt --check` clean; `cargo clippy --all-targets -- -D warnings` clean; `cargo tree` reports this crate and nothing else. Its literal output is quoted in the [test report](test-report.md).

## Why there are no new unit tests

The test plan states "Unit Tests: None". This sprint changes configuration and
documentation: `rust-toolchain.toml`, the CI workflow and the README.
`src/main.rs` is untouched, and `git diff 7b2cba7..60a529a -- src/` is empty.
The new checks read repository files through the real test binary, so they are
recorded in [e2e-tests.md](e2e-tests.md).

## Regression role

The 68 unit tests are part of the regression bar for INT-0001 and INT-0004. On
this sprint's toolchain they ran unedited and passed in three places:

- locally on 1.98.1;
- on both CI legs on 1.98.1;
- again under the 1.96.0 pin, both in the local pass-mutation and on both legs
  of the pinned CI run.
