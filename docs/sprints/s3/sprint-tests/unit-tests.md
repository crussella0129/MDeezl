# Sprint 3 Unit Test Results

- **Tested head:** `77b0c00570c978b4d12ea2b8d40fa88816f285e2` (branch `dev`)
- **Runner:** `cargo test --all -- --nocapture`, the project's canonical suite.
- **Host:** Windows 11, `rustc 1.99.1` / `cargo 1.99.1`, `rustup 1.29.2`,
  `git 2.56.0.windows.3`. CI ran the same head on `rustc 1.100.0`: by this
  head the runner images already carried it, so the run's update step reads
  `unchanged` — the two runs earlier in this phase are the ones that
  installed it, and [integration-tests.md](integration-tests.md) has those
  logs. Either way this head is green on two different stables, which is
  [INT-0005](../../../intents/INT-0005-reproducible-toolchain.md)'s policy
  working rather than a separate claim.
- **Result:** **79 passed, 0 failed, 0 ignored** in `src/lib.rs`, plus the
  GUI's pure-logic tests, which live in the 31 of `gui/tests/app.rs` and are
  listed below. Sprint 2 closed with 68; this sprint adds 11.
- **Gates:** `cargo fmt --all --check` clean;
  `cargo clippy --workspace --all-targets -- -D warnings` clean.
- **Dependency check:** `cargo tree -p mdeezl` at the tested head prints

  ```
  mdeezl v0.1.0 (C:\Users\charl\MDeezl)
  ```

  and nothing else. Its automated counterpart,
  `test_manifest_dependencies_table_is_empty`, ran **unedited** and passed.

## T-001 — the engine becomes a library

- **Intent:** [INT-0006](../../../intents/INT-0006-desktop-gui.md)

| Test | EARS clause verified |
|------|----------------------|
| `test_bundle_returns_notices_as_data` | `bundle` on a plain `tmp_dir`, which is not a git work tree, returns exactly one notice, beginning `gitignore filtering skipped:` and carrying no `mdeezl: ` prefix. The notices come back as data, and nothing is printed. |

**The verbatim check** is recorded, not asserted by a test, and was re-run at
the tested head. It reads committed blobs, because the working tree is CRLF
while the index is LF:

1. the `#[cfg(test)] mod tests { … }` block of `git show 24442f1:src/main.rs`
   is **47,456 bytes** once its closing brace is dropped;
2. the same block in `git show efca63f:src/lib.rs` is 48,284 bytes, and the
   47,456 bytes are its **byte prefix**;
3. at the tested head the block has grown, and those 47,456 bytes are **still
   its byte prefix**.

So every one of the 68 unit tests that closed sprint 2 moved unedited, and
this sprint's eleven were appended after them.

## T-002 — URL sources

- **Intent:** [INT-0002](../../../intents/INT-0002-git-and-remote-sources.md),
  with [INT-0006](../../../intents/INT-0006-desktop-gui.md) for the last row

| Test | EARS clause verified |
|------|----------------------|
| `test_is_url` | True for `https://x/y`, `http://x/y` and `file:///x`; false for `C:\x`, `./https`, `ftp:/weird`, `git@host:x/y` and the empty string. Detection reads the argument's text. |
| `test_repo_name` | The ten-row table: `.git` and trailing slashes removed, query and fragment dropped, and `repository` for `.`, `..`, an empty segment, or a segment holding `\`, `:`, `?` or `#`. |
| `test_display_url_strips_credentials` | `https://user:TOKEN@github.com/o/r` shows as `https://github.com/o/r`; a URL without userinfo and a `file:///C:/x` URL are unchanged. |
| `test_clone_command_is_hardened` | `Command::get_args` is exactly the planned list, `core.eol=lf`, `http.lowSpeedLimit=1` and `http.lowSpeedTime=30` included, ending `-- <url> <dest>`. `get_envs` sets `GIT_TERMINAL_PROMPT=0`, `GCM_INTERACTIVE=never`, `GIT_ALLOW_PROTOCOL=https:http:file` and `GIT_LFS_SKIP_SMUDGE=1`, and removes `GIT_ASKPASS`, `SSH_ASKPASS`, the two `GIT_HTTP_LOW_SPEED_*` overrides and the 15 repository-local variables. |
| `test_fetch_rejects_protocol_outside_allow_list` | `fetch` with `ssh://127.0.0.1/x` returns `Err` naming the transport as not allowed — git's own enforcement of the allow-list variable. No network is reached. See the note on its reach below. |
| `test_unauthorised_repository_fails_without_prompting` | **New after the critique.** A local socket answers `401` with a Basic challenge, which is what a private repository sends an anonymous client. `fetch` returns `Err` beginning `could not fetch`, naming the refusal — measured here as `could not read Username for 'http://127.0.0.1:PORT': terminal prompts disabled` — in **0.25 s**, and leaves the test-owned base folder empty. This is the authentication half of INT-0002's failure criterion; it had no executed test before. |
| `test_fetch_file_url_bundles_the_clone` | A committed fixture repository with `* text=auto` is cloned through a `file://` URL: heading `# <url>`, tree root `<name>/`, the committed file byte for byte **with LF on Windows too**, the untracked file absent, and the base folder empty once the clone is dropped. |
| `test_fetch_bad_url_errs` | A `file://` URL to a missing path returns `Err` beginning `could not fetch` and containing git's first `fatal:` line, `does not appear to be a git repository`. |
| `test_stalled_server_fails_the_fetch` | A `TcpListener` that accepts and never answers: with the low-speed window lowered to 3 s on that test's own thread, the fetch returns `Err` naming "Operation too slow" within 15 s, and leaves nothing behind. The production window of 30 s is pinned by `test_clone_command_is_hardened`. |
| `test_git_commands_set_create_no_window` | Windows only. A process's creation flags cannot be read back from outside it, so this asserts the two halves that make the promise true: `git_command` applies `CREATE_NO_WINDOW`, and `git_command` is the **only** place the shipped half of the file spawns git — the test module's own fixture spawns are excluded, and the needles are built at run time so they cannot match the test itself. A `git --version` through the helper still runs and answers. On Linux it prints its SKIP line, which the CI log shows. |

**What `test_fetch_rejects_protocol_outside_allow_list` now reaches.** Since
the Test Phase repairs, `bundle_in` refuses an `ssh://` source before `fetch`
is called, so that literal case is no longer reachable from the CLI or the
window — the two now refuse it in that order, as defence in depth. The test
still exercises git's enforcement of `GIT_ALLOW_PROTOCOL` at the fetch
boundary, which is the mechanism INT-0002 relies on for an `insteadOf` rule
that rewrites an `https://` URL to ssh. That rewrite itself is **not executed
in this sprint**: the clone runs with an isolated, empty global config by
design, so a rewrite cannot be staged without weakening that isolation. Its
evidence remains the planning-time measurement recorded in INT-0002.

## T-003 and T-004 — the window's pure logic

- **Intent:** [INT-0006](../../../intents/INT-0006-desktop-gui.md)
- These live in `gui/tests/app.rs`, because the crate's logic is reached
  through its library target.

| Test | EARS clause verified |
|------|----------------------|
| `test_gui_manifest_dependencies` | `[dependencies]` is exactly `mdeezl`, `eframe` and `rfd`; `eframe` has `default-features = false` and exactly `glow`, `default_fonts`, `accesskit`, `x11`, `wayland`; `rfd` exactly `xdg-portal`; `[dev-dependencies]` exactly `egui_kittest` with the single feature `eframe`. `image`, `egui_extras`, `resvg` and `usvg` appear nowhere. |
| `test_options_follow_the_fields` | For each of the three wrap modes, with the checkbox on and off, `App::options()` equals the expected literal `Options`. |
| `test_format_size` | `0 bytes`, `1023 bytes`, `1.0 KB`, `1.5 KB`, `1.0 MB`. |
| `test_output_stem_is_one_component` | `Hello-World` and `repo.v2` are kept; `""`, `.`, `..`, `a/b`, `a\b`, `C:\`, `c:x` and `/` each become `bundle`, by the same character rule on every OS. That the written file takes its name from this rule is a separate test, in [e2e-tests.md](e2e-tests.md). |
| `test_temp_name_carries_the_pid` | **New after the critique.** `temp_name("repo")` starts `.repo.md.`, ends `.mdeezl-tmp`, and contains this process's id. The shared name it replaces is the defect two instances hit when writing one output folder. |

## T-006 — the weasel's geometry

- **Intent:** [INT-0006](../../../intents/INT-0006-desktop-gui.md)

| Test | EARS clause verified |
|------|----------------------|
| `test_weasel_shapes_fit_the_unit_box` | Every disc, and every tube point extended by its radius, lies inside `[0,1] × [0,0.6]`. The list is non-empty and holds at least one tube. |
| `test_icon_rasterises_the_weasel` | `icon(64)` is 64×64 with 16,384 RGBA bytes, the four corners have alpha 0, and opaque coverage is between 15 % and 60 %. The run **prints the measured share: 0.223** at the tested head. T-006's task record quotes 0.177, which was the weasel before the user's last two rounds of changes. |
| `test_native_options_carry_the_icon` | `native_options().viewport.icon` is `Some`, 64×64, and equals `icon(64)` byte for byte. |

## What the critique changed here

The round-1 critique returned `block`. Three of its concerns were about this
file's claims, and each was confirmed by deleting the mechanism and watching
nothing turn red:

- **`CREATE_NO_WINDOW` (C-002).** The old test asserted only that git ran; it
  passed with the flag deleted. The row above now says what is actually
  asserted, and the mutation turns it red.
- **The temporary name's pid (C-003).** Nothing failed against the old shared
  name; `test_temp_name_carries_the_pid` now does.
- **Authentication (C-009).** There was no executed test for a repository that
  refuses an anonymous client. There is now, and it runs offline.

## Regression role

The 79 engine tests are the regression bar for
[INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md) and
[INT-0004](../../../intents/INT-0004-gitignore-aware-exclusion.md). They
passed locally on 1.99.1 at the tested head, and on both CI legs on 1.100.0.
The verbatim check above is what proves the 68 inherited ones are unedited,
rather than merely still green.
