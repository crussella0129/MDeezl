# Sprint 1 Integration Test Results

- **Tested head:** `b1755d6ad6bf49ed8c09ba9eafa5ba8dc59c1976` (branch `dev`)
- **Runner:** `cargo test --all -- --nocapture`
- **Result:** passed. Both compositions below run on every push, on both CI runners.

## Query composed with `run`, in-crate

`test_run_spawns_git_exactly_once` calls `run` directly against a real repository with nested directories, writing to `Sink::File`. That composes several pieces through the real control flow:

- the walk;
- `gitignore_candidates`;
- `git_ignored`;
- pruning;
- rendering.

A `#[cfg(test)]` thread-local counter then confirms that the whole run spawned git exactly once. This is the direct measurement of INT-0004's one-subprocess criterion. Sprint 1's plan critique rejected the earlier idea of inferring it from a timing bound.

## Pruning composed with both renderers, end to end

`test_cli_gitignore_omits_from_both_halves` drives the real binary against a real repository. The repository ignores `out/` and `*.log` and re-includes `keep.log`.

- Each ignored entry is absent from the scaffold **and** from the content sections.
- Each kept entry is present in both.

That is the single-file-set property from INT-0001, now holding across two exclusion sources. It is recorded in [e2e-tests.md](e2e-tests.md) with the other end-to-end results. A separate in-crate composition test would re-verify the same path with less fidelity, so none was written.

## Sprint 0 compositions

`test_excluded_entry_absent_from_both_halves` and `test_included_entry_present_in_both_halves` ran **unedited** and passed. The built-in list's composition is unaffected by the second source.
