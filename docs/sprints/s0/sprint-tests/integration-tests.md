# Sprint 0 Integration Test Results

- **Tested head:** `2e48a1cb5e7380631092e2e74371f28792afb583` (branch `dev`)
- **Runner:** `cargo test --all -- --nocapture`
- **Result:** 2 passed, 0 failed.

These two compose T-002's walk with both renderers and verify the property the
single-walk design exists to guarantee: the scaffold and the content dump can
never describe different file sets. They are owned by T-004, whose composed
EARS clause they verify, and live in `src/main.rs` alongside the unit tests
because they exercise internal functions rather than the binary.

| Test | Composition | Property |
|------|-------------|----------|
| `test_excluded_entry_absent_from_both_halves` | T-002 walk + T-003 tree + T-004 contents | a real temp tree containing `target/` is walked once and rendered twice; `target` and its file's contents appear in neither half, while a sibling survives in both |
| `test_included_entry_present_in_both_halves` | same composition | with ignore `.*` and include `.github`, `.github/` appears in the scaffold **and** `File: .github/ci.yml` with its body appears in the contents |

Both build their fixture under a unique temp directory and remove it
afterwards. No third-party crate is used for temp directories: the helper is
`std::env::temp_dir()` plus the process id, a per-test tag, and an atomic
counter. Cleanup runs through a `Drop` guard in both test modules, so a
failing assertion cannot leave a tree behind.
