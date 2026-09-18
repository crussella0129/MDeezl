# Sprint 0 Test Plan

## Intent Traceability

| Intent | Acceptance criterion | Build task / EARS clause | Verification |
|--------|----------------------|--------------------------|--------------|
| [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md) | `mdeezl <path>` writes to stdout; no path means the current directory | T-001 / WHEN `parse_args` receives no arguments THEN defaults SHALL be `.`, stdout, `Fence`, default ignore list | `test_args_defaults`, `test_cli_writes_document_to_stdout` |
| [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md) | `mdeezl <path>` writes to stdout; no path means the current directory | T-001 / WHEN a bare positional argument is given THEN it SHALL be the scan root | `test_args_positional_root` |
| [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md) | Builds with no third-party crates and the test suite adds none | T-001 / WHEN `Cargo.toml` is read THEN it SHALL declare an empty `[dependencies]` table, a binary named `mdeezl`, and `license = "Apache-2.0"` | `test_manifest_dependencies_table_is_empty`, `test_manifest_declares_bin_name_and_license` |
| [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md) | Builds with no third-party crates and the test suite adds none | T-001 / WHEN `cargo tree` is run THEN it SHALL report no dependencies | manual gate in T-001, recorded in the test report; `test_manifest_dependencies_table_is_empty` is the automated half |
| [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md) | Tree renders `├── `, `└── `, `│   ` with one space before the name; directories carry `/` | T-003 / the two branch-symbol clauses, the prefix-composition clause, the trailing-`/` clause | `test_tree_branch_symbols`, `test_tree_prefix_composition`, `test_tree_directory_suffix`, `test_cli_tree_shape` |
| [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md) | An unreadable directory is marked in place and does not abort the document | T-003 / WHEN a directory is recorded unreadable THEN the line SHALL carry a marker | `test_tree_unreadable_marker` |
| [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md) | Traversal is deterministic and sorted by name | T-002 / WHEN `walk` reads a directory THEN children SHALL be sorted by name | `test_walk_sorts_children`, `test_cli_output_is_byte_identical_across_runs` |
| [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md) | One ignore list governs tree and content dump identically | T-004 / WHEN a single walk is rendered by both renderers THEN an omitted entry SHALL be absent from both and a re-admitted entry SHALL be present in both | `test_excluded_entry_absent_from_both_halves`, `test_included_entry_present_in_both_halves` |
| [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md) | List is pre-populated with `.*`, `target`, `node_modules`, `dist`, `build`, `__pycache__` | T-001 / WHEN no arguments are given THEN the ignore list SHALL be exactly that set | `test_args_defaults`, `test_cli_default_ignores_target_and_dotfiles` |
| [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md) | `--exclude` appends, `--include` cancels, include always wins | T-002 / WHEN an entry matches both an ignore and an include pattern THEN `is_excluded` SHALL be false | `test_include_outranks_ignore`, `test_cli_include_readmits_dot_entry`, `test_cli_exclude_adds_pattern` |
| [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md) | Both flags are repeatable | T-001 / WHEN `--exclude` or `--include` appears more than once THEN every occurrence SHALL be retained in declaration order | `test_args_accumulates_exclude_and_include` |
| [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md) | Pattern form: `.*` matches any dot-prefixed name | T-002 / WHEN pattern is `.*` THEN it SHALL match names beginning with `.` | `test_pattern_dot_star` |
| [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md) | Pattern form: `*.ext` matches a whole file type | T-002 / WHEN pattern begins `*.` THEN it SHALL match names ending in the remainder | `test_pattern_extension` |
| [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md) | Pattern form: a pattern with `/` matches one exact relative path | T-002 / WHEN pattern contains `/` THEN it SHALL compare against the root-relative path, not the name | `test_pattern_relative_path`, `test_cli_exclude_single_file_by_path` |
| [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md) | Pattern form: any other pattern matches that exact name at any depth | T-002 / WHEN pattern is any other form THEN it SHALL match an exactly equal name | `test_pattern_exact_name`, `test_pattern_name_matches_at_depth` |
| [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md) | Consequence: `--include ".*"` re-admits `.git` | T-002 / WHEN an entry matches both an ignore and an include pattern THEN `is_excluded` SHALL be false | `test_include_dot_star_readmits_git` |
| [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md) | Symlinks are listed but never traversed | T-002 / WHEN `file_type` reports a symlink THEN `walk` SHALL NOT descend | `test_walk_does_not_follow_symlink` (Linux leg of the T-006 matrix is the authoritative run) |
| [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md) | The scan root must exist and be a directory | T-002 / WHEN the root does not exist THEN the walk SHALL return an error rather than an empty tree | `test_walk_rejects_missing_root`, `test_cli_missing_root_exits_1_with_empty_stdout` |
| [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md) | Each file is introduced by a `--- File: <path> ---` header | T-004 / WHEN a file section is emitted THEN it SHALL precede the body with `---`, `File: <path>`, `---` | `test_file_header_format`, `test_cli_file_header_format` |
| [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md) | Non-UTF-8 bodies are elided, not emitted as mojibake | T-004 / WHEN bytes are not valid UTF-8 THEN the body SHALL be an elision marker stating the byte count | `test_binary_body_elided`, `test_cli_binary_file_elided` |
| [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md) | An unreadable file is marked in place and does not abort the document | T-004 / WHEN a file cannot be read THEN rendering SHALL continue | `test_unreadable_file_body_marked` |
| [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md) | An unreadable directory is marked in place and does not abort the document | T-002 / WHEN `read_dir` fails THEN `walk` SHALL record it unreadable and SHALL continue traversing the remainder of the tree | `test_unreadable_dir_marked_and_walk_continues` |
| [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md) | Every file section is separated from what precedes it by a blank line | T-004 / WHEN a section follows earlier output THEN a blank line SHALL precede its opening `---` | `test_section_separator_blank_line`, `test_cli_wrap_none_preserves_section_separation` |
| [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md) | `--wrap` defaults to `fence`; an opening fence outlives any backtick run in the body | T-004 / WHEN the longest backtick run is `n` THEN `fence_len` SHALL return `max(3, n + 1)` | `test_fence_len_plain_body`, `test_fence_len_body_with_triple_backticks`, `test_fence_len_body_with_quad_backticks`, `test_fence_len_counts_longest_run_not_total`, `test_cli_markdown_body_does_not_close_its_own_fence` |
| [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md) | `fence` mode: the tree sits in one fenced block | T-003 / WHEN the wrap mode is `Fence` THEN the tree SHALL be emitted inside exactly one fenced block | `test_tree_fence_wrap`, `test_cli_fence_mode_tree_in_one_block` |
| [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md) | Wherever a body is fenced — `fence` and `inline` alike — a known extension yields a language hint; an unknown one is not an error | T-004 / WHEN the wrap mode is `Fence` or `Inline` and the extension is in the fixed table THEN the fence SHALL carry the hint, otherwise none and no error | `test_language_hint_known_and_unknown`, `test_cli_rust_body_carries_language_hint`, `test_cli_inline_mode_body_carries_language_hint` |
| [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md) | `inline` wraps each tree line in single backticks; bodies stay fenced | T-003 / WHEN the wrap mode is `Inline` THEN each tree line SHALL be surrounded by single backticks | `test_tree_inline_wrap`, `test_cli_inline_mode_keeps_bodies_fenced` |
| [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md) | `none`: the tree is bare, with no fence and no backticks | T-003 / WHEN the wrap mode is `None` THEN the tree SHALL be emitted bare | `test_tree_none_wrap`, `test_cli_wrap_none_has_no_fences` |
| [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md) | `none`: bodies are emitted with no surrounding fence | T-004 / WHEN the wrap mode is `None` THEN the body SHALL have no fence | `test_cli_wrap_none_has_no_fences` |
| [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md) | `-o <file>` writes to that file instead of stdout | T-005 / WHEN `-o` is given THEN the document SHALL go to the file and SHALL NOT go to stdout | `test_cli_output_flag_writes_file` |
| [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md) | With no `-o`, the document goes to stdout | T-005 / WHEN no `-o` is given THEN the complete document SHALL appear on stdout and nothing SHALL be written to any other file | `test_cli_writes_document_to_stdout` |
| [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md) | Output paths use `/` and are relative to the scan root on every platform | T-004 / WHEN a body is emitted THEN its `File:` path SHALL use `/` and be root-relative | `test_relative_path_uses_forward_slashes`, `test_cli_nested_paths_use_forward_slashes` (Windows leg of the T-006 matrix is the authoritative run) |
| [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md) | The document is a title, a `## Structure` section, then a `## Contents` section | T-005 / WHEN the document is assembled THEN it SHALL have that order | `test_document_assembles_in_order`, `test_cli_document_section_order` |
| [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md) | Exit 0 on success, 2 on an argument error, 1 on an unreadable root; no partial stdout | T-001 / the three usage-error clauses; T-005 / the exit-code and no-partial-output clauses | `test_args_rejects_unknown_wrap`, `test_args_rejects_missing_flag_value`, `test_args_rejects_second_positional`, `test_cli_usage_error_exits_2`, `test_cli_missing_root_exits_1_with_empty_stdout` |
| [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md) | Exit 1 when the *output* cannot be written — the second, independent exit-1 trigger | T-005 / WHEN the output cannot be written THEN it SHALL print to stderr and exit 1 | `test_cli_unwritable_output_exits_1` |
| [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md) | `--help` documents the pattern forms and the two caveats | T-001 / WHEN `-h`/`--help` is given THEN the text SHALL document the four forms and both caveats | `test_cli_help_documents_patterns_and_caveats` |
| [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md) | Verification runs on both Linux and Windows | T-006 / WHEN `scaffold-ci.sh` is re-run THEN a workflow SHALL exist that runs `cargo test`; WHEN it is read THEN its matrix SHALL include a Linux and a Windows runner; WHEN a test cannot run on a platform THEN it SHALL report the skip | `test_ci_workflow_runs_tests_on_both_platforms`, `test_walk_does_not_follow_symlink` and `test_unreadable_dir_marked_and_walk_continues` (each reports its skip reason) |
| [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md) | `README.md` documents the same surface as `--help`, discoverable before the binary is built | T-007 / WHEN `README.md` is read THEN it SHALL document every flag, all four pattern forms, the ignore list, and the three wrap modes | `test_readme_documents_cli_surface` |

## Unit Tests

### T-001 unit tests
- **Intent:** [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md)
- `test_args_defaults`: no arguments → root `.`, stdout sink, `Wrap::Fence`, ignore list exactly `[".*", "target", "node_modules", "dist", "build", "__pycache__"]`, empty include list.
- `test_args_positional_root`: `["some/dir"]` → root `some/dir`.
- `test_args_accumulates_exclude_and_include`: `["--exclude", "*.png", "--include", ".github", "--exclude", "vendor"]` → ignore list gains `*.png` then `vendor` in that order, include list is `[".github"]`.
- `test_args_rejects_unknown_wrap`: `["--wrap", "banana"]` → usage error naming `banana`.
- `test_args_rejects_missing_flag_value`: each of `["-o"]`, `["--wrap"]`, `["--exclude"]`, `["--include"]` → usage error naming that flag. All four arms are asserted, not just two.
- `test_args_rejects_second_positional`: `["a", "b"]` → usage error.
- `test_manifest_dependencies_table_is_empty`: read `Cargo.toml` via `include_str!` and assert there is no entry line between the `[dependencies]` header and the next table header or end of file. Stated this way, the test does not impose an ordering constraint on where `[dependencies]` sits in the manifest.
- `test_manifest_declares_bin_name_and_license`: the manifest names the binary target `mdeezl` — load-bearing, since `env!("CARGO_BIN_EXE_mdeezl")` resolves by target name — and sets `license = "Apache-2.0"` to match the repository `LICENSE`.
- Stubs: none — `parse_args` takes an iterator of `String`, so tests call it directly without touching the process environment.

### T-002 unit tests
- **Intent:** [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md)
- `test_pattern_dot_star`: `.*` matches `.git` and `.env`, does not match `src`.
- `test_pattern_extension`: `*.png` matches `logo.png`, does not match `png`, `logo.pngx`, or a directory named `assets`.
- `test_pattern_relative_path`: `docs/big.csv` matches the entry at that root-relative path and does not match a file named `big.csv` elsewhere.
- `test_pattern_exact_name`: `target` matches a `target` entry and does not match `targets` or `my_target`.
- `test_pattern_name_matches_at_depth`: `node_modules` matches at `a/b/node_modules`, confirming name patterns are depth-independent.
- `test_include_outranks_ignore`: ignore `[".*"]` + include `[".github"]` → `.github` is kept while `.env` is dropped.
- `test_include_dot_star_readmits_git`: ignore `[".*"]` + include `[".*"]` → `.git` is kept, pinning the consequence INT-0001 records rather than leaving it accidental.
- `test_walk_sorts_children`: a temp directory created in shuffled order yields children in name order.
- `test_walk_does_not_follow_symlink`: a directory symlink pointing at its own parent is listed once and not descended; the walk terminates. Where the platform refuses to create the symlink without privileges — Windows without Developer Mode — the test prints its skip reason and the Linux leg of the T-006 matrix is the authoritative run, so the guarantee is never merely assumed.
- `test_unreadable_dir_marked_and_walk_continues`: a directory whose read fails is flagged and its siblings still appear. The failure is produced with `PermissionsExt::from_mode(0o000)` on Unix. On Windows `std::fs::set_permissions` can only toggle the read-only attribute, which does not block `read_dir`, and std exposes no ACL API — so on Windows the test prints its skip reason and the Linux leg of the T-006 matrix is the authoritative run, exactly as the symlink test does. The directory's permissions are restored before cleanup so the temp tree can be removed.
- `test_walk_rejects_missing_root`: a nonexistent root → error, not an empty tree.
- Stubs: a `tmp_dir()` helper built from `std::env::temp_dir()`, the process id, and an atomic counter; no `tempfile` crate. Symlink creation uses `std::os::unix::fs::symlink` / `std::os::windows::fs::symlink_dir` behind `#[cfg]`, both in std.

### T-003 unit tests
- **Intent:** [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md)
- `test_tree_branch_symbols`: a two-child node renders `├── ` then `└── `.
- `test_tree_prefix_composition`: descending past a non-last entry yields a `│   ` prefix; past a last entry, four spaces.
- `test_tree_directory_suffix`: directory nodes render with a trailing `/`, files without.
- `test_tree_unreadable_marker`: a directory flagged unreadable renders with the trailing marker.
- `test_tree_fence_wrap`: `Wrap::Fence` puts the whole tree in exactly one fenced block — the default mode, which must not be the only untested one.
- `test_tree_inline_wrap`: `Wrap::Inline` surrounds each line with single backticks and leaves the symbols untouched.
- `test_tree_none_wrap`: `Wrap::None` emits the tree with no fence *and no backticks on any line* — the half of the `none` clause that a fence-only assertion cannot catch.
- Stubs: renderers take an already-built `Node` tree, so these tests construct nodes in memory and touch no filesystem.

### T-004 unit tests
- **Intent:** [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md)
- `test_file_header_format`: a rendered section's header lines are exactly `---`, `File: <root-relative path>`, `---` — the one output format inherited verbatim from `MDeezl.md`, asserted as literal lines rather than by heading level, because the second `---` is a setext underline.
- `test_section_separator_blank_line`: two consecutive sections rendered in sequence are separated by a blank line before the second header's opening `---`, matching the leading newline in the `awk` one-liner's `"\n---\nFile: …"`.
- `test_fence_len_plain_body`: a body with no backticks → 3.
- `test_fence_len_body_with_triple_backticks`: a body containing ``` → 4.
- `test_fence_len_body_with_quad_backticks`: a body containing a four-backtick run → 5.
- `test_fence_len_counts_longest_run_not_total`: a body with many separate single backticks → 3.
- `test_binary_body_elided`: invalid UTF-8 bytes → marker containing the byte count, and none of the raw bytes.
- `test_unreadable_file_body_marked`: a read error → marker carrying the error text. The error is injected at the render seam — the renderer is handed an `Err(io::Error)` body directly rather than a real unreadable file — so this test is platform-independent and, unlike the directory case, needs no skip on Windows.
- `test_language_hint_known_and_unknown`: `.rs` → `rust`, `.md` → `markdown`, `.xyz` → no info string and no error.
- `test_relative_path_uses_forward_slashes`: a nested path renders with `/` on every platform.
- Stubs: the body a section renders is passed in as a `Result<String, io::Error>`, which lets the read-error and binary cases be exercised without a filesystem.

### T-005 unit tests
- **Intent:** [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md)
- `test_document_assembles_in_order`: rendering into a `Vec<u8>` produces the `# <root>` title line, then the `## Structure` line, then the `## Contents` line, matched as exact lines so that the setext headings produced by file sections cannot confuse the assertion.
- Stubs: the document renderer writes to any `io::Write`, so tests pass a `Vec<u8>` instead of a real sink.

### T-006 / T-007 checks
- **Intent:** [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md)
- **Placement:** both checks live in `tests/cli.rs`, the integration target T-005 creates, not in an in-crate `#[cfg(test)]` module. That is what makes their `Depends on: T-005` real rather than nominal. They are listed here beside the tasks that own them for readability; they are integration tests, not unit tests.
- `test_ci_workflow_runs_tests_on_both_platforms`: read the generated workflow under `.github/workflows/` and assert it invokes `cargo test` and names both a Linux and a Windows runner. This is the verification the previous draft of this plan omitted entirely, on the one task whose whole purpose is to stop the checkpoint being green because nothing ran.
- `test_readme_documents_cli_surface`: read `README.md` via `include_str!` and assert it mentions each flag, each of the four pattern forms, **every entry of the pre-populated ignore list**, and each of the three wrap modes — all four elements the criterion names, not three.
- `cargo tree` reporting no dependencies is a manual gate belonging to T-001, which owns the zero-dependency criterion and the manifest; it is recorded in the test report because it is a toolchain query rather than a property of the code.

## Integration Tests

### Traversal + rendering integration
- **Intents:** [INT-0001](../../../intents/INT-0001-markdown-repo-context-bundle.md)
- **Owned by T-004**, whose composed EARS clause these two verify; they are written against internal functions in an in-crate `#[cfg(test)]` module, which is why T-004's Touches is `src/main.rs` alone and why it takes `Depends on: T-003`.
- `test_excluded_entry_absent_from_both_halves`: T-002 + T-003 + T-004 composed — a tree containing `target/` is walked once and rendered twice; `target` appears in neither the scaffold nor the content sections. This is the property that the single-walk design exists to guarantee.
- `test_included_entry_present_in_both_halves`: the mirror case — `--include .github` puts `.github` in the scaffold *and* in the content sections.

## End-to-End Tests

- **Status:** possible
- `test_cli_writes_document_to_stdout`: run the binary against a fixture tree; stdout is a Markdown document, exit 0, and the fixture directory gains no new file — the "nothing written to any other file" half of the clause.
- `test_cli_document_section_order`: stdout contains the `# <root>`, `## Structure`, and `## Contents` lines in that order, matched as exact lines.
- `test_cli_tree_shape`: the scaffold for a known fixture matches an expected literal block, symbols and spacing included.
- `test_cli_file_header_format`: a section's header lines are exactly `---`, `File: <path>`, `---`.
- `test_cli_default_ignores_target_and_dotfiles`: fixture contains `target/` and `.secret`; neither appears.
- `test_cli_include_readmits_dot_entry`: `--include .github` → `.github` appears.
- `test_cli_exclude_adds_pattern`: `--exclude "*.log"` → a `.log` file disappears while its siblings remain.
- `test_cli_exclude_single_file_by_path`: `--exclude sub/one.txt` drops exactly that file and leaves a same-named file elsewhere.
- `test_cli_binary_file_elided`: a file of invalid UTF-8 bytes → elision marker, no raw bytes in stdout.
- `test_cli_markdown_body_does_not_close_its_own_fence`: the fixture holds a `.md` file containing a triple-backtick block; the emitted document's fences balance, proving the adaptive length works against the exact case this repository would hit.
- `test_cli_fence_mode_tree_in_one_block`: default mode → the scaffold is inside exactly one fenced block.
- `test_cli_rust_body_carries_language_hint`: a `.rs` file's fence carries the `rust` info string; a `.xyz` file's fence carries none.
- `test_cli_inline_mode_keeps_bodies_fenced`: `--wrap inline` → tree lines are backtick-wrapped and bodies are still fenced.
- `test_cli_inline_mode_body_carries_language_hint`: `--wrap inline` → a `.rs` body's fence still carries the `rust` info string, pinning the criterion's "wherever a body is fenced" scope in the mode that is easy to overlook.
- `test_cli_wrap_none_has_no_fences`: `--wrap none` → no fence appears in the output.
- `test_cli_wrap_none_preserves_section_separation`: the fixture holds two text files, the first ending in an ordinary text line. With `--wrap none` the next section's opening `---` is preceded by a blank line, so the previous file's last line is not swallowed as a setext underline and the second header survives intact. This is the one mode where the missing separator would corrupt the document silently.
- `test_cli_output_flag_writes_file`: `-o out.md` → the file holds the document and stdout is empty.
- `test_cli_nested_paths_use_forward_slashes`: no `\` appears in any `File:` header; the Windows leg of the T-006 matrix is the authoritative run.
- `test_cli_output_is_byte_identical_across_runs`: two runs over an unchanged fixture produce identical bytes.
- `test_cli_usage_error_exits_2`: `--wrap banana` → exit 2, message on stderr, empty stdout.
- `test_cli_missing_root_exits_1_with_empty_stdout`: a nonexistent path → exit 1, message on stderr, no partial document on stdout.
- `test_cli_unwritable_output_exits_1`: `-o <nonexistent-dir>/out.md` → exit 1, message on stderr, empty stdout. This drives the second, independent exit-1 trigger; it fails identically on Linux and Windows with std only, so it needs no platform skip, and it is what distinguishes exit 1 from a panic's 101.
- `test_cli_help_documents_patterns_and_caveats`: `--help` output names all four pattern forms and both caveats, exit 0.
- Each end-to-end test builds its own fixture tree under a unique temp directory and removes it afterwards; the binary is located with `env!("CARGO_BIN_EXE_mdeezl")`, which Cargo sets for integration targets under `tests/` and which resolves by binary target name, so no test dependency is added.
