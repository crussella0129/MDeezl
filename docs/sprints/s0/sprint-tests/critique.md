# Test Critique — Sprint 0

Three bounded read-only review rounds were run against the sprint 0 test
evidence using the installed bundle's `prompts/test-critic.md`. Rounds one and
two returned `block` — 12 and 6 concerns respectively — and every concern was
addressed in the tests or the result artifacts before the next round, which
re-verified the previous round's fixes against the code rather than against the
result documents. The concern below is the final round's; the resolution note
records what changed after it.

Round one's most consequential finding was that the fence-balance assertion was
invalid: it counted "fence-only lines" and tested parity, a filter that excludes
every opening fence carrying a language hint while still counting its closer,
and that counts a Markdown body's own fence lines as delimiters. It was replaced
by a fence state machine, and round two then showed that balance alone still
cannot catch the regression this project most fears — a too-short opener closes
early, re-opens on the body's next fence line, and balances anyway — so the
opener-outgrows-content invariant is now asserted directly.

## Concerns

### C-001: the unreadable-file clause's "continue with the next file" half is unexecuted and undisclosed
- **Where:** `build-plan.md` T-004 EARS; the only test was `test_unreadable_file_body_marked` in `src/main.rs`
- **Quote:** "**WHEN** a file cannot be read, **THEN** the body **SHALL** be replaced by a marker carrying the underlying error, and rendering **SHALL** continue with the next file."
- **Failure mode:** EARS-coverage
- **Why it matters:** The only test handed `render_section` a synthetic `io::Error` and inspected one section in isolation; no test put an unreadable file inside a real document, so the "continue with the next file" half never ran. That is the same gap shape round two raised for the unreadable *directory*, except the directory case is now disclosed in the result artifacts and this one was not.
- **Suggested response:** add-test
- **Resolution:** added `test_unreadable_file_does_not_abort_document`. It walks a fixture, removes one file after the walk has recorded it, then renders — so the read fails with no permissions manipulation at all. It asserts the elision marker for the removed file **and** that the following file's section and body are still present. Being host-independent, it needs no platform skip, unlike the directory case. The suite is now 72 tests.

## Confidence
proceed-with-caveats
