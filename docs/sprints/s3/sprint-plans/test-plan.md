# Sprint 3 Test Plan

## Intent Traceability

| Intent | Acceptance criterion | Build task / EARS clause | Verification |
|--------|----------------------|--------------------------|--------------|
| [INT-0006](../../../intents/INT-0006-desktop-gui.md) | The CLI's own output | T-001 / WHEN built THEN a lib target and a thin `main.rs` calling `mdeezl::cli` | `test_main_rs_only_delegates_to_cli` |
| [INT-0006](../../../intents/INT-0006-desktop-gui.md) | The CLI's own output | T-001 / WHEN `bundle` is called THEN it returns name, document equal to the CLI's `-o` bytes, and notices, and prints nothing | `test_bundle_matches_cli_output`; `test_bundle_returns_notices_as_data` |
| [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md) | All realized criteria, with no regression | T-001 / WHEN the CLI runs THEN its output, stderr and exit codes are unchanged; tests moved verbatim | all 122 existing tests; the **verbatim check** below |
| [INT-0006](../../../intents/INT-0006-desktop-gui.md), [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md) | The CLI stays dependency-free | T-001 / WHEN `cargo tree -p mdeezl` runs THEN only `mdeezl` | `test_manifest_dependencies_table_is_empty` (unedited); literal `cargo tree -p mdeezl` output in the report |
| [INT-0002](../../../intents/INT-0002-git-and-remote-sources.md) | A remote addressed by URL | T-002 / WHEN the argument begins `https://`, `http://` or `file://` THEN URL, else path | `test_is_url` |
| [INT-0002](../../../intents/INT-0002-git-and-remote-sources.md) | Never waits on a person | T-002 / WHEN a URL is bundled THEN the exact clone command, env and stdin | `test_clone_command_disables_prompts_and_helpers` |
| [INT-0002](../../../intents/INT-0002-git-and-remote-sources.md) | A remote addressed by URL | T-002 / WHEN `<repo-name>` is derived THEN last segment minus `.git`, or `repository` | `test_repo_name` |
| [INT-0002](../../../intents/INT-0002-git-and-remote-sources.md) | The document records its source | T-002 / WHEN the clone succeeds THEN heading `# <url>`, tree root `<repo-name>/`, `name` is `<repo-name>` | `test_fetch_file_url_bundles_the_clone`; `test_cli_file_url_source_bundles_the_clone` |
| [INT-0002](../../../intents/INT-0002-git-and-remote-sources.md) | Failures give stderr, non-zero exit and no partial document | T-002 / WHEN the fetch fails THEN exit 1, empty stdout, one `could not fetch` line | `test_cli_unreachable_url_exits_1`; `test_cli_url_without_git_exits_1`; `test_fetch_bad_url_errs` |
| [INT-0002](../../../intents/INT-0002-git-and-remote-sources.md) | The temporary copy is removed | T-002 / WHEN a URL run ends THEN its temp directory is gone | `test_fetch_file_url_bundles_the_clone`; `test_fetch_bad_url_errs`; `test_cli_file_url_source_bundles_the_clone`; `test_cli_unreachable_url_exits_1` |
| [INT-0002](../../../intents/INT-0002-git-and-remote-sources.md) | Documented source forms | T-002 / WHEN `--help` or the README is read THEN URL, git, temporary clone, no private repositories | `test_cli_help_documents_url_sources`; `test_readme_documents_url_sources` |
| [INT-0002](../../../intents/INT-0002-git-and-remote-sources.md) | Remote over the real network | T-002 (all clauses, live) | `test_live_github_clone` (`#[ignore]`), run by hand in the Test Phase |
| [INT-0002](../../../intents/INT-0002-git-and-remote-sources.md) | Reading a git ref does not mutate the working tree | none — the git-ref half, backlog T-103 | **not in this sprint**; INT-0002 stays `active` |
| [INT-0006](../../../intents/INT-0006-desktop-gui.md) | The CLI stays dependency-free | T-003 / WHEN the workspace is built THEN `mdeezl-gui` has exactly the planned dependencies | `test_gui_manifest_dependencies` |
| [INT-0006](../../../intents/INT-0006-desktop-gui.md) | One window, four controls | T-003 / WHEN shown THEN the heading and every control, with their defaults | `test_app_shows_title_and_controls`; `test_app_defaults` |
| [INT-0006](../../../intents/INT-0006-desktop-gui.md) | The CLI's own output | T-003 / WHEN the fields are set THEN `Options` follow them | `test_options_follow_the_fields` |
| [INT-0006](../../../intents/INT-0006-desktop-gui.md) | The CLI's own output; responsive while working | T-003 / WHEN Generate is activated THEN working at once, a worker, `<name>.md` equal to `bundle`, and the status | `test_generate_writes_engine_bytes`; `test_generate_marks_working_before_returning` |
| [INT-0006](../../../intents/INT-0006-desktop-gui.md) | Responsive while working | T-003 / WHEN the worker runs THEN Generate is disabled; status `Fetching…` or `Bundling…` | `test_generate_disabled_while_working`; `test_status_names_fetching_or_bundling` |
| [INT-0006](../../../intents/INT-0006-desktop-gui.md) | No partial files | T-003 / WHEN bundling or writing fails THEN an error and no file | `test_generate_missing_folder_writes_nothing`; `test_generate_unreachable_url_writes_nothing`; `test_generate_unwritable_output_writes_nothing` |
| [INT-0006](../../../intents/INT-0006-desktop-gui.md) | One window, four controls | T-003 / WHEN a field is empty THEN the status names it, and no worker | `test_generate_requires_source_and_output` |
| [INT-0006](../../../intents/INT-0006-desktop-gui.md) | One window, four controls | T-003 / WHEN a folder is dropped THEN it becomes the source | `test_dropped_folder_fills_source` |
| [INT-0006](../../../intents/INT-0006-desktop-gui.md) | Tested in CI on both OSes | T-004 / WHEN CI runs THEN the three workspace gates | `test_ci_updates_installs_and_logs_in_order` (updated), plus the **gate mutation check** below |
| [INT-0006](../../../intents/INT-0006-desktop-gui.md) | Tested in CI on both OSes | T-004 / WHEN `dev` is pushed THEN both legs build and test the GUI | **CI observation**: both legs green, and each log shows `mdeezl-gui` test results |
| [INT-0005](../../../intents/INT-0005-reproducible-toolchain.md), [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md) | Guard and two-OS legs do not regress | T-004 / every sprint 2 mutation still fails | the re-run sprint 2 mutation table |
| [INT-0006](../../../intents/INT-0006-desktop-gui.md) | The weasel is drawn by the app | T-005 / WHEN `shapes()` is called THEN discs and tubes in the unit box | `test_weasel_shapes_fit_the_unit_box` |
| [INT-0006](../../../intents/INT-0006-desktop-gui.md) | The weasel is drawn by the app | T-005 / WHEN shown THEN a hero painted from the shapes, with no image dependency | `test_hero_paints_weasel_shapes`; `test_gui_manifest_dependencies` |
| [INT-0006](../../../intents/INT-0006-desktop-gui.md) | The weasel is drawn by the app | T-005 / WHEN started THEN the window icon is `icon(64)` with the stated coverage | `test_icon_rasterises_the_weasel` |
| [INT-0006](../../../intents/INT-0006-desktop-gui.md) | The weasel is judged by a person | T-005 / a screenshot is shown to the user | **Human verification**: a screenshot of the running window, and the user's verdict recorded in the test report |
| [INT-0006](../../../intents/INT-0006-desktop-gui.md) | Documented | T-006 / WHEN the README is read THEN the desktop-app items | `test_readme_documents_desktop_app` |

## Unit Tests

### T-001 unit tests
- **Intent:** [INT-0006](../../../intents/INT-0006-desktop-gui.md)
- `test_bundle_returns_notices_as_data`: `bundle` on a plain directory that is not a git work tree returns one notice. The notice begins `gitignore filtering skipped:` and has no `mdeezl: ` prefix.
- **Verbatim check** (recorded, not a test): the `#[cfg(test)] mod tests` block, extracted from `git show 24442f1:src/main.rs` and from the new `src/lib.rs`, is byte-identical.

### T-002 unit tests
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
- `test_clone_command_disables_prompts_and_helpers`: reads `Command::get_args` and `get_envs` on the clone command.
  - The arguments are exactly `-c credential.helper= clone --depth 1 --quiet -- <url> <dest>`.
  - `GIT_TERMINAL_PROMPT` is `0`, and `GCM_INTERACTIVE` is `never`.
  - Each of the 15 local variables is set to be removed.
- `test_fetch_file_url_bundles_the_clone`: uses a committed fixture repository, with an identity passed through `-c`, and an untracked file beside it.
  - `bundle` on its `file://` URL returns heading `# <url>`, tree root `<name>/` and the committed file's text, and not the untracked file.
  - Afterwards the `mdeezl-<pid>-<n>` directory is gone.
- `test_fetch_bad_url_errs`: `file:///` pointing at a missing path returns `Err` whose text begins `could not fetch`. No `mdeezl-<pid>-*` directory from this call remains.

### T-003 unit tests (gui)
- **Intent:** [INT-0006](../../../intents/INT-0006-desktop-gui.md)
- `test_options_follow_the_fields`: for each of the three wrap modes, with the checkbox on and off, the app's `Options` equal the expected literal.
- `test_gui_manifest_dependencies`: reads `gui/Cargo.toml`. Normal dependencies are exactly `mdeezl`, `eframe` and `rfd`. Dev-dependencies are exactly `egui_kittest`. No `image`, `egui_extras`, `resvg` or `usvg` appears anywhere.

### T-005 unit tests (gui)
- **Intent:** [INT-0006](../../../intents/INT-0006-desktop-gui.md)
- `test_weasel_shapes_fit_the_unit_box`: every disc and tube point, extended by its radius, lies within `[0,1] × [0,0.6]`. The list is non-empty and holds at least one tube.
- `test_icon_rasterises_the_weasel`: `icon(64)` is 64×64 with 16,384 RGBA bytes. The four corner pixels have alpha 0. Between 15 % and 60 % of pixels have alpha ≥ 128.

## Integration Tests

### Engine ↔ CLI, and GUI ↔ engine
- **Intents:** [INT-0006](../../../intents/INT-0006-desktop-gui.md), [INT-0002](../../../intents/INT-0002-git-and-remote-sources.md)
- `test_bundle_matches_cli_output` (`tests/cli.rs`, using the library): on a git fixture, for each wrap mode with gitignore on and off, `mdeezl::bundle(&opts).document` equals the bytes of the CLI's `-o` file for the same options. `name` equals the fixture folder's name.
- `test_generate_writes_engine_bytes` (`gui/tests/app.rs`, egui_kittest):
  1. type a fixture path into the source field and a temporary folder into the output field;
  2. pick `inline`;
  3. click `Generate`, and wait at most 10 s for the worker;
  4. check `<out>/<name>.md` equals `mdeezl::bundle` for those options;
  5. check the status shows `Wrote`.
- Together these make GUI output equal to CLI output, through the one shared function.
- **CI observation:** both legs green on `dev` after T-004. Each log lists `mdeezl-gui` test results, and the Linux leg installs no system package unless the T-004 contingency was needed.

### Gate mutation check (T-004)
Each change is applied to the real workflow, `test_ci_updates_installs_and_logs_in_order` is run, and the file is restored byte-identical.

- **Must fail:** `cargo fmt --check` restored in place of `cargo fmt --all --check`.
- **Must fail:** `cargo clippy --all-targets -- -D warnings` restored in place of the workspace line.
- **Must fail:** every row of the sprint 2 table, re-run against the new lines.

## End-to-End Tests
- **Status:** possible.
- `test_main_rs_only_delegates_to_cli`: `src/main.rs`, with comments and blank lines removed, has at most 5 lines. It calls `mdeezl::cli` and `std::process::exit`, and nothing else.
- `test_cli_file_url_source_bundles_the_clone`: the binary is run on the fixture's `file://` URL, with `TMP`, `TEMP` and `TMPDIR` pointing at a test-owned folder.
  - It exits 0, the heading is the URL, the committed file is present and the untracked one absent.
  - The test-owned temp folder is empty afterwards.
- `test_cli_unreachable_url_exits_1`: `https://127.0.0.1:1/x.git`, with the same temp redirection. It exits 1 with empty stdout, stderr is exactly one line beginning `mdeezl: could not fetch https://127.0.0.1:1/x.git: `, and the temp folder is empty. Localhost only, so no network is used.
- `test_cli_url_without_git_exits_1`: an empty `PATH`, and a `file://` URL. It exits 1, stdout is empty, and stderr ends `git was not found on PATH`.
- `test_cli_help_documents_url_sources`: `--help` contains `URL`, `https://`, `git`, `temporary`, and `private repositories are not supported`.
- `test_readme_documents_url_sources`: the README contains the same four items.
- `test_live_github_clone` (`#[ignore]`, run by hand in the Test Phase):
  - `https://github.com/octocat/Hello-World` bundles with heading `# https://github.com/octocat/Hello-World` and a `README` section;
  - `https://github.com/octocat/definitely-not-a-repo-mdeezl` fails within 10 s with exit 1 and no prompt;
  - the temp folder is empty after both.
- GUI (`gui/tests/app.rs`, egui_kittest, headless on both legs):
  - `test_app_shows_title_and_controls`: the harness finds by label `MDeezl`, two `Browse…` buttons, `fence`, `inline`, `none`, `Respect .gitignore` and `Generate`, and a text field hinted `Folder path or https:// URL`.
  - `test_app_defaults`: `fence` is selected, the checkbox is checked, and the output field equals the home folder.
  - `test_generate_marks_working_before_returning`: `App::generate()` returns with the app working, and the result then arrives from the worker.
  - `test_generate_disabled_while_working`: while working, the `Generate` node is disabled.
  - `test_status_names_fetching_or_bundling`: a local source shows `Bundling…`; a `file://` source shows `Fetching…`.
  - `test_generate_missing_folder_writes_nothing`: the status shows an error, and the output folder stays empty.
  - `test_generate_unreachable_url_writes_nothing`: `https://127.0.0.1:1/x.git` shows an error beginning `could not fetch`, and the output folder stays empty.
  - `test_generate_unwritable_output_writes_nothing`: the output path is an existing *file*. The status shows an error, and no `<name>.md` exists.
  - `test_generate_requires_source_and_output`: an empty source gives `Choose a folder or URL`; an empty output gives `Choose an output folder`. No worker is started.
  - `test_dropped_folder_fills_source`: a `DroppedFile` with a folder path, pushed into the harness input, becomes the source field's text.
  - `test_hero_paints_weasel_shapes`: after one frame, the painted output holds at least as many circle shapes as `weasel::shapes()` has discs.
- `test_readme_documents_desktop_app`: the README's `## Desktop app` section contains `cargo run -p mdeezl-gui --release`, `cargo install --path gui`, `Generate`, `.md`, and `git`.
- **Human verification:** a screenshot of the running app, captured from the real window, is shown to the user. Their verdict on the weasel is quoted in the test report.
- **Manual checks**, recorded in the test report:
  - `Browse…` opens a native folder picker and fills the field;
  - a live GitHub URL generated from the window shows `Fetching…`;
  - the window keeps repainting while the clone runs.
