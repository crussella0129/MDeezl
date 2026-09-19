# Sprint 3 Test Plan

## Intent Traceability

| Intent | Acceptance criterion | Build task / EARS clause | Verification |
|--------|----------------------|--------------------------|--------------|
| [INT-0006](../../../intents/INT-0006-desktop-gui.md) | The CLI's own output | T-001 / WHEN built THEN a lib target; `main.rs` is one `fn main` calling `mdeezl::cli` | `test_main_rs_only_delegates_to_cli` |
| [INT-0006](../../../intents/INT-0006-desktop-gui.md) | The CLI's own output | T-001 / WHEN `bundle` is called THEN name, document equal to the CLI's `-o` bytes, notices, and nothing printed | `test_bundle_matches_cli_output`; `test_bundle_returns_notices_as_data` |
| [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md) | All realized criteria, with no regression | T-001 / WHEN the CLI runs THEN output, stderr and exit codes unchanged | all 122 existing tests, including the one-line stderr assertions that would catch a `bundle` that also prints |
| [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md) | All realized criteria, with no regression | T-001 / WHEN the move is checked THEN the old test block is a byte prefix of the new one | **verbatim check** (T-001 unit tests) |
| [INT-0006](../../../intents/INT-0006-desktop-gui.md), [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md) | The CLI stays dependency-free | T-001 / WHEN `cargo tree -p mdeezl` runs THEN only `mdeezl` | `test_manifest_dependencies_table_is_empty` (unedited); literal `cargo tree -p mdeezl` output in the report |
| [INT-0002](../../../intents/INT-0002-git-and-remote-sources.md) | A remote addressed by URL | T-002 / WHEN `is_url` is called THEN true exactly for `https://`, `http://` and `file://` | `test_is_url` |
| [INT-0002](../../../intents/INT-0002-git-and-remote-sources.md) | Never waits on a person | T-002 / WHEN fetched THEN the exact clone command and env | `test_clone_command_is_hardened`; `test_fetch_rejects_protocol_outside_allow_list` |
| [INT-0002](../../../intents/INT-0002-git-and-remote-sources.md) | A remote addressed by URL | T-002 / WHEN `<repo-name>` is derived THEN last segment minus `.git`, or `repository` | `test_repo_name` |
| [INT-0002](../../../intents/INT-0002-git-and-remote-sources.md) | The document records its source | T-002 / WHEN the clone succeeds THEN `# <url>`, `<repo-name>/`, and `name`; committed bytes | `test_fetch_file_url_bundles_the_clone`; `test_cli_file_url_source_bundles_the_clone` |
| [INT-0002](../../../intents/INT-0002-git-and-remote-sources.md) | Failures give stderr, non-zero exit and no partial document | T-002 / WHEN the fetch fails THEN exit 1, empty stdout, one `could not fetch` line | `test_cli_unreachable_url_exits_1`; `test_cli_url_without_git_exits_1`; `test_fetch_bad_url_errs` |
| [INT-0002](../../../intents/INT-0002-git-and-remote-sources.md) | The temporary copy is removed | T-002 / WHEN a fetch ends THEN its directory is gone | `test_fetch_file_url_bundles_the_clone`; `test_fetch_bad_url_errs`; `test_cli_file_url_source_bundles_the_clone`; `test_cli_unreachable_url_exits_1` |
| [INT-0006](../../../intents/INT-0006-desktop-gui.md) | Responsive while working (no console flash) | T-002 / WHEN git is spawned on Windows THEN `CREATE_NO_WINDOW` | `test_git_commands_set_create_no_window` (Windows only; self-skips elsewhere); **manual check** with the release GUI |
| [INT-0002](../../../intents/INT-0002-git-and-remote-sources.md) | Documented source forms | T-002 / WHEN `--help` or the README is read THEN URL, git, temporary clone, no private repositories | `test_cli_help_documents_url_sources`; `test_readme_documents_url_sources` |
| [INT-0002](../../../intents/INT-0002-git-and-remote-sources.md) | Remote over the real network | T-002 (all clauses, live) | `test_live_github_clone` (`#[ignore]`), run by hand in the Test Phase |
| [INT-0002](../../../intents/INT-0002-git-and-remote-sources.md) | Reading a git ref does not mutate the working tree | none — the git-ref half, backlog T-103 | **not in this sprint**; INT-0002 stays `active` |
| [INT-0006](../../../intents/INT-0006-desktop-gui.md) | The CLI stays dependency-free | T-003 / WHEN built THEN `mdeezl-gui` has exactly the planned dependencies and features | `test_gui_manifest_dependencies` |
| [INT-0006](../../../intents/INT-0006-desktop-gui.md) | One window, four controls | T-003 / WHEN shown THEN exactly the planned widgets, plus the heading | `test_app_has_exactly_the_controls` |
| [INT-0006](../../../intents/INT-0006-desktop-gui.md) | One window, four controls | T-003 / WHEN started THEN the defaults | `test_app_defaults` |
| [INT-0006](../../../intents/INT-0006-desktop-gui.md) | The CLI's own output | T-003 / WHEN the fields are set THEN `App::options()` follows them | `test_options_follow_the_fields` |
| [INT-0006](../../../intents/INT-0006-desktop-gui.md) | One window, four controls | T-003 / WHEN a folder is dropped THEN it becomes the source | `test_dropped_folder_fills_source` |
| [INT-0006](../../../intents/INT-0006-desktop-gui.md) | Responsive while working | T-004 / WHEN Generate is activated THEN working before return, and the job on a worker | `test_generate_marks_working_before_returning` |
| [INT-0006](../../../intents/INT-0006-desktop-gui.md) | The CLI's own output | T-004 / WHEN the job succeeds THEN temp, rename, `Wrote … (<size>)`, notices, and engine bytes | `test_generate_writes_engine_bytes`; `test_generate_shows_notices`; `test_format_size` |
| [INT-0006](../../../intents/INT-0006-desktop-gui.md) | Responsive while working | T-004 / WHEN working THEN Generate disabled; `Fetching…` or `Bundling…` | `test_generate_disabled_while_working`; `test_status_names_fetching_or_bundling` |
| [INT-0006](../../../intents/INT-0006-desktop-gui.md) | No partial files | T-004 / WHEN the job or write fails THEN error, temp removed, existing file untouched | `test_failed_job_keeps_existing_output`; `test_generate_missing_folder_writes_nothing`; `test_generate_unreachable_url_writes_nothing`; `test_generate_unwritable_output_writes_nothing` |
| [INT-0006](../../../intents/INT-0006-desktop-gui.md) | One window, four controls | T-004 / WHEN a field is empty THEN the status names it, and no job | `test_generate_requires_source_and_output` |
| [INT-0006](../../../intents/INT-0006-desktop-gui.md) | Tested in CI on both OSes | T-005 / WHEN CI runs THEN the three workspace gates, nothing else changed | `test_ci_updates_installs_and_logs_in_order` (updated); the **gate mutation check** |
| [INT-0006](../../../intents/INT-0006-desktop-gui.md) | Tested in CI on both OSes | T-005 / WHEN `dev` is pushed THEN both legs build and test the GUI, with no package install | **CI observation** (Integration Tests) |
| [INT-0005](../../../intents/INT-0005-reproducible-toolchain.md), [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md) | The guard and the two-OS legs do not regress | T-005 / every sprint 2 mutation still fails | the re-run sprint 2 mutation table |
| [INT-0006](../../../intents/INT-0006-desktop-gui.md) | The weasel is drawn by the app | T-006 / WHEN `shapes()` is called THEN discs and tubes in the unit box | `test_weasel_shapes_fit_the_unit_box` |
| [INT-0006](../../../intents/INT-0006-desktop-gui.md) | The weasel is drawn by the app | T-006 / WHEN shown THEN the hero's circles and segments inside `hero_rect`, with no image | `test_hero_paints_weasel_shapes`; `test_gui_manifest_dependencies` |
| [INT-0006](../../../intents/INT-0006-desktop-gui.md) | The weasel is drawn by the app | T-006 / WHEN `native_options()` is built THEN the icon is `icon(64)` with the stated coverage | `test_icon_rasterises_the_weasel`; `test_native_options_carry_the_icon` |
| [INT-0006](../../../intents/INT-0006-desktop-gui.md) | The weasel is judged by a person | T-006 / a screenshot is shown to the user | **Human verification**: a screenshot of the running window, with the user's verdict quoted in the test report |
| [INT-0006](../../../intents/INT-0006-desktop-gui.md) | Documented | T-007 / WHEN the README is read THEN the desktop-app items | `test_readme_documents_desktop_app` |

## Unit Tests

### T-001 unit tests (`src/lib.rs`, appended to `mod tests`)
- **Intent:** [INT-0006](../../../intents/INT-0006-desktop-gui.md)
- `test_bundle_returns_notices_as_data`: `bundle` on a plain `tmp_dir`, which is not a git work tree, returns exactly one notice. It begins `gitignore filtering skipped:` and has no `mdeezl: ` prefix.
- **Verbatim check** (recorded, not a test):
  1. extract the `#[cfg(test)] mod tests { … }` block from `git show 24442f1:src/main.rs`;
  2. strip its final `}`;
  3. confirm it is a byte prefix of the block in the new `src/lib.rs`.

### T-002 unit tests (`src/lib.rs`, appended to `mod tests`, reusing its helpers)
- **Intent:** [INT-0002](../../../intents/INT-0002-git-and-remote-sources.md)
- `test_is_url`: `https://x/y`, `http://x/y` and `file:///x` are URLs. `C:\x`, `./https`, `ftp:/weird`, `git@host:x/y` and the empty string are not.
- `test_repo_name`:

  | URL | Name |
  |-----|------|
  | `https://github.com/octocat/Hello-World` | `Hello-World` |
  | `…/repo.git` | `repo` |
  | `…/repo/` | `repo` |
  | `file:///C:/tmp/fixture` | `fixture` |
  | `https://host/` | `repository` |
- `test_clone_command_is_hardened`: reads `Command::get_args` and `get_envs`.
  - The arguments are exactly the planned `-c …` and `clone …` list, ending `-- <url> <dest>`.
  - `GIT_TERMINAL_PROMPT=0`, `GCM_INTERACTIVE=never` and `GIT_ALLOW_PROTOCOL=https:http:file` are set.
  - `GIT_ASKPASS`, `SSH_ASKPASS` and all 15 local variables are set to be removed.
- `test_fetch_rejects_protocol_outside_allow_list`: `fetch` with `ssh://127.0.0.1/x` returns `Err`, and the reason names the transport as not allowed. No network is reached: the allow-list refuses it first.
- `test_fetch_file_url_bundles_the_clone`: uses a committed fixture repository, with an identity passed through `-c`, and an untracked file beside it. `fetch` runs against a base folder owned by the test.
  - The bundle's heading is `# <url>`, its tree root `<name>/`, and it holds the committed file's text but not the untracked file. The content check ignores line endings.
  - Once the clone is dropped, the base folder is empty.
- `test_fetch_bad_url_errs`: a `file://` URL to a missing path returns `Err` whose text begins `could not fetch`. The test-owned base folder is empty afterwards.
- `test_git_commands_set_create_no_window` (Windows only): on Windows the flag is applied by one shared helper. This test calls that helper on a `Command`, then spawns `git --version` through it, which proves the flagged process still runs and returns output. Elsewhere it prints SKIP.

### T-003 and T-004 unit tests (`gui`)
- **Intent:** [INT-0006](../../../intents/INT-0006-desktop-gui.md)
- `test_gui_manifest_dependencies`: parses `gui/Cargo.toml`.
  - `[dependencies]` names exactly `mdeezl`, `eframe` and `rfd`.
  - `eframe` has `default-features = false` and exactly the five planned features.
  - `rfd` has `default-features = false` and exactly `xdg-portal`.
  - `[dev-dependencies]` names exactly `egui_kittest`.
  - `image`, `egui_extras`, `resvg` and `usvg` appear nowhere.
- `test_options_follow_the_fields`: for each of the three wrap modes, with the checkbox on and off, `App::options()` equals the expected literal.
- `test_format_size`: `0` → `0 bytes`, `1023` → `1023 bytes`, `1024` → `1.0 KB`, `1536` → `1.5 KB`, `1048576` → `1.0 MB`.

### T-006 unit tests (`gui`)
- **Intent:** [INT-0006](../../../intents/INT-0006-desktop-gui.md)
- `test_weasel_shapes_fit_the_unit_box`: every disc, and every tube point extended by its radius, lies within `[0,1] × [0,0.6]`. The list is non-empty and holds at least one tube.
- `test_icon_rasterises_the_weasel`: `icon(64)` is 64×64, with 16,384 RGBA bytes. The four corner pixels have alpha 0. Between 15 % and 60 % of pixels have alpha ≥ 128.
- `test_native_options_carry_the_icon`: `native_options().viewport.icon` is `Some`, 64×64.

## Integration Tests

### Engine ↔ CLI, and GUI ↔ engine
- **Intents:** [INT-0006](../../../intents/INT-0006-desktop-gui.md), [INT-0002](../../../intents/INT-0002-git-and-remote-sources.md)
- `test_bundle_matches_cli_output` (`tests/cli.rs`, using the library):
  - The fixture is a git repository with a `.gitignore`, and the test runs through each wrap mode with gitignore on and off.
  - `mdeezl::bundle(&opts).document` equals the bytes of the CLI's `-o` file for the same options, and `name` equals the fixture folder's name.
  - **Both sides run under the same environment.** The CLI child inherits the test process's environment unchanged, with no `isolate()` on either side, so host git configuration affects both equally.
- `test_generate_writes_engine_bytes` (`gui/tests/app.rs`, egui_kittest). The harness is `Harness::new_ui_state` with the real `App`.
  1. Type a fixture path and a temporary output folder into the fields, and pick `inline`.
  2. Click `Generate`.
  3. Poll with `harness.step()` plus a 10 ms sleep, until the app stops working or 10 s pass.
  4. `<out>/<name>.md` equals `mdeezl::bundle` for those options. The status contains `Wrote` and the formatted size. No `.mdeezl-tmp` file remains.
- Together these make GUI output equal to CLI output, through the one shared function.
- **CI observation**, after T-005: both legs green on `dev`. Each log lists `mdeezl-gui` test results, and neither log shows a package-install step.

### Gate mutation check (T-005)
Each change is applied to the real workflow, `test_ci_updates_installs_and_logs_in_order` is run with `-- --exact`, and the file is restored byte-identical. The assertion message is recorded.

- **Must fail:** `- run: cargo fmt --check`, in place of `cargo fmt --all --check`.
- **Must fail:** `- run: cargo clippy --all-targets -- -D warnings`, in place of the workspace line.
- **Must fail:** every row of the sprint 2 table, re-run against the new lines.

## End-to-End Tests
- **Status:** possible.
- **The `file://` URL** in every test is built as T-002's Notes specify.
- **The binary tests** point the child's `TMP`, `TEMP` and `TMPDIR` at a test-owned folder.
- `test_main_rs_only_delegates_to_cli`: `src/main.rs`, with `//` comments and blank lines removed, has at most 3 lines. It contains `mdeezl::cli(` and `std::process::exit(`, and no `use` or other `fn`.
- `test_cli_file_url_source_bundles_the_clone`:
  - exit 0, and the heading is the URL;
  - the committed file's text is present and the untracked file absent, compared ignoring line endings;
  - the test-owned temp folder is empty afterwards.
- `test_cli_unreachable_url_exits_1`: `https://127.0.0.1:1/x.git`.
  - Exit 1 and empty stdout.
  - Stderr is exactly one line beginning `mdeezl: could not fetch https://127.0.0.1:1/x.git: `.
  - The temp folder is empty afterwards.
  - Localhost only, so no network is used.
- `test_cli_url_without_git_exits_1`: an empty `PATH` and a `file://` URL give exit 1, empty stdout, and stderr ending `git was not found on PATH`.
- `test_cli_help_documents_url_sources`: `--help` contains `URL`, `https://`, `git`, `temporary`, and `private repositories are not supported`.
- `test_readme_documents_url_sources`: the README contains the same.
- `test_live_github_clone` (`#[ignore]`, run by hand in the Test Phase):
  - `https://github.com/octocat/Hello-World` bundles, with heading `# https://github.com/octocat/Hello-World` and a `README` section;
  - `https://github.com/octocat/definitely-not-a-repo-mdeezl` fails within 10 s, with exit 1 and no prompt;
  - the temp folder is empty after both.
- **GUI** (`gui/tests/app.rs`, egui_kittest, headless on both legs). Every test holds the app through `new_ui_state`, and waits with the `step()` polling loop, never `run()`: `run()` panics while the app keeps requesting repaints. Tests that need the working state inject a **gated job**, which blocks until the test releases a channel.
  - `test_app_has_exactly_the_controls`:
    - `query_all_by_role` counts exactly 2 `TextInput`, 3 `Button`, 3 `RadioButton` and 1 `CheckBox`;
    - the labels `MDeezl`, `Source`, `Output folder`, `fence`, `inline`, `none`, `Respect .gitignore` and `Generate` are found;
    - two `Browse…` buttons are found.
  - `test_app_defaults`: `fence` is selected, the checkbox is checked, and the output field equals the home folder computed by the rule in T-003.
  - `test_dropped_folder_fills_source`: a test type implementing egui's `DroppedFile` trait (`path`, `bytes`, `bytes_async`) is pushed as an `Arc` into `harness.input_mut().dropped_files`. After one `step()`, the source field holds its path.
  - `test_generate_marks_working_before_returning`: with a gated job, `App::generate()` returns with `is_working()` true. After release, the polling loop sees the result.
  - `test_generate_disabled_while_working`: with a gated job, after one `step()`, the `Generate` node's `accesskit_node().is_disabled()` is true. After release and completion, it is false.
  - `test_status_names_fetching_or_bundling`: with a gated job, a local source shows `Bundling…` and a `file://` source shows `Fetching…`.
  - `test_generate_shows_notices`: the real job on a fixture that is not a git work tree gives a status containing `Wrote` and `gitignore filtering skipped:`.
  - `test_failed_job_keeps_existing_output`: `<out>/<name>.md` is pre-seeded with a sentinel, then a job returns `Err`. The status shows the error, the sentinel file is unchanged, and no `.mdeezl-tmp` file remains.
  - `test_generate_missing_folder_writes_nothing`: the real job on a missing path shows an error, and the output folder stays empty.
  - `test_generate_unreachable_url_writes_nothing`: the real job on `https://127.0.0.1:1/x.git` shows an error beginning `could not fetch`, and the output folder stays empty.
  - `test_generate_unwritable_output_writes_nothing`: the output path is an existing *file*. An error is shown, and the file's contents are unchanged.
  - `test_generate_requires_source_and_output`: an empty source gives `Choose a folder or URL`, and an empty output gives `Choose an output folder`. The job's call counter stays 0.
  - `test_hero_paints_weasel_shapes`: after one `step()`, the painted output, clipped to `App::hero_rect()`, contains:
    - at least as many filled circles as `shapes()` has discs plus tube vertices;
    - at least as many line segments as the tubes have segments.

    The radio buttons' circles lie outside the hero rect and are not counted.
- `test_readme_documents_desktop_app`: the `## Desktop app` section contains:
  - `cargo run -p mdeezl-gui --release` and `cargo install --path gui`;
  - `Source`, `Browse…`, `fence`, `inline`, `none`, `Respect .gitignore`, `Output folder` and `Generate`;
  - `.md`, `replac`, and `git`.
- **Human verification:** a screenshot of the running app, captured from the real window, is shown to the user. Their verdict on the weasel is quoted in the test report.
- **Manual checks**, recorded in the test report:
  - `Browse…` opens a native folder picker and fills its field;
  - a live GitHub URL generated from the release window shows `Fetching…` while the window stays responsive;
  - no console window flashes while it runs.
