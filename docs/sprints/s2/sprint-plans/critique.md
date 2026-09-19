# Plan Critique — Sprint 2

Four bounded read-only review rounds were run against the sprint 2 plans, using the installed bundle's `prompts/plan-critic.md`:

| Round | Scope | Verdict | Concerns |
|-------|-------|---------|----------|
| 1 | full | `block` | 10 |
| 2 | full | `block` | 7 |
| 3 | full | `proceed-with-caveats` | 3 |
| 4 | narrow check of round 3's fixes | `clean` | 0 |

This sprint's critique mattered more than the size of the change suggested. **Round one overturned the research's central claim.** Research had read `rustup toolchain install`'s help — "Install or update … by default the active toolchain" — as meaning it updates by default. That was never measured. The critic argued that "by default" chooses *which* toolchain, not *whether to update*. Measurement on the development host confirmed it: the no-argument command answered "using existing install" and left `stable` at 1.96.0. The mechanism the research recommended would have been a silent no-op on CI.

The research report now carries an explicit Correction with each superseded step marked, and INT-0005 records the change in its transition history.

Every round's technical claims were measured on the host before the plans changed:

- **The no-argument install does not update.** It left `stable` where it was.
- **`rustup update stable` does update.** It moved 1.96.0 to 1.98.1, which is the machine-wide change the user approved in Plan Mode.
- **`rustup check` exits 100** whenever any update is available, so under `bash -eo pipefail` it would fail CI for no code reason. It was dropped.
- **A pinned directory changes the bare `rustc`.** There, `rustc` reported the pin (1.96.0), while `rustc +stable` reported 1.98.1, so the image-version log uses `+stable`.
- **The installed 1.96.0 carries `rustfmt` and `clippy`,** so a pin mutation to it needs no download.

Two things could not be established and are disclosed rather than guessed:

- **The minimum rustup version for the no-argument install.** Neither rustup's documentation nor its changelog states it. The criterion now names the verified version and the upgrade path, and INT-0005's transition history records the change.
- **Pinned mode in CI.** It needs a throwaway pull request, which will be opened only with the user's consent at the Test Phase. If the user declines, INT-0005 closes `active`.

## Concerns
(none — plans are clean per the failure-mode screen.)

## Confidence
clean
