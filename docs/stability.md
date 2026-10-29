# Stability

What will not change under you once Sprig is 1.0, and the test that holds each
promise. A promise without a test is a hope, so every row names one; CI fails if
a named test disappears (`crates/sprig-cli/tests/docs.rs`).

## Files

| Promise | Held by |
| --- | --- |
| A file valid under 1.0 means the same to every 1.x tool, forever: the same tree, statuses, dates and diagnostics. | `crates/sprig-core/tests/conformance.rs` `every_example_matches_its_expected_tree`, over every example in `tests/conformance/`. An example's `expected.json` changes only with a 2.0. |
| Changes to the spec after 1.0 only add. Something new reaches an older 1.x tool as text, never as an error: an unknown key is an ordinary field, and a line that is not one of the nine marks is a note. | `crates/sprig-core/tests/conformance.rs` `every_example_matches_its_expected_tree`, through `tokens/107-only-due-and-start-are-dates` and `lines/066-a-capital-x-is-not-a-mark`. |
| Removing anything, or changing what a valid file means, needs a spec 2.0, and a 2.0 file says so with the title's reserved `sprig:` key (§4.6). | `crates/sprig-core/tests/conformance.rs` `every_example_matches_its_expected_tree`, through `tokens/064-a-title-names-the-version-it-is-written-for` and `tokens/065-a-version-the-tool-does-not-read-is-warned-about`. |
| Every numbered rule of the spec has a conformance example. | `crates/sprig-core/tests/conformance.rs` `every_rule_in_the_spec_has_an_example`. |
| A diagnostic code is never reused: once published it keeps its meaning, and a retired code keeps its row. | `crates/sprig-core/tests/catalogue.rs` `a_published_code_stays_in_the_catalogue_with_its_meaning` and `every_code_the_core_emits_is_in_the_catalogue`. |

Changes to the spec go through an RFC (`rfcs/`).

## The tree format

| Promise | Held by |
| --- | --- |
| `"sprig_tree": 1` documents keep every key they have, with its meaning. A new key is added only where Appendix D lists it, and a reader must ignore keys it does not know. A change to an existing key is `"sprig_tree": 2`. | `crates/sprig-core/tests/schema.rs` `the_schema_accepts_every_tree_the_core_writes_for_the_bakery` and `appendix_d_documents_every_key_the_schema_allows`. |

## The command line

The `sprig` command follows semantic versioning, and its interface includes:

| Promise | Held by |
| --- | --- |
| Exit codes: 0 fine; 1 findings (`check` errors, or warnings with `--strict`; `next` with nothing ready; `find` matching nothing; `fmt --check` with files to change; `status --oneline` with something late; `why` on a blocked item); 2 a usage error, an unknown ref or filter, or a path that could not be read. | `crates/sprig-cli/tests/cli.rs` `help_lists_each_command_with_one_line_and_the_exit_codes`, `next_with_nothing_ready_exits_1`, `an_unreadable_path_exits_2_and_names_the_path`; `crates/sprig-cli/tests/tools.rs` `find_with_an_unknown_filter_exits_2_and_lists_the_filters`; `crates/sprig-cli/tests/views.rs` `why_tiling_crosses_into_the_lease_and_exits_1`. |
| The columns of every `--plain` output, in order. A new column is added only at the end. | `crates/sprig-cli/tests/plain.rs`, a snapshot for each command below. |
| `--json` output is the tree format, as above. | `crates/sprig-cli/tests/tools.rs` `next_json_gives_the_items_next_shows_in_the_same_order_and_valid`. |
| The MCP server's tool names and arguments. New tools and optional arguments may be added. | `crates/sprig-cli/tests/mcp.rs` `every_tool_has_a_valid_schema_and_honest_annotations`. |

The `--plain` columns:

| Command | Columns | Snapshot test |
| --- | --- | --- |
| `next` | ref, mark, text, path through the tree, due date, priority | `crates/sprig-cli/tests/plain.rs` `the_plain_columns_of_next_stay_as_they_are` |
| `check` | file, line, column, severity, code, message | `crates/sprig-cli/tests/plain.rs` `the_plain_columns_of_check_stay_as_they_are` |
| `tree` | ref, depth, mark, text, status, done, total | `crates/sprig-cli/tests/plain.rs` `the_plain_columns_of_tree_stay_as_they_are` |
| `status` | file, done, total, ready, waiting, late, errors | `crates/sprig-cli/tests/plain.rs` `the_plain_columns_of_status_stay_as_they_are` |
| `find` | ref, mark, text, path through the tree, due date, status | `crates/sprig-cli/tests/plain.rs` `the_plain_columns_of_find_stay_as_they_are` |
| `agenda` | ref, due date, mark, text, `every:` value | `crates/sprig-cli/tests/plain.rs` `the_plain_columns_of_agenda_stay_as_they_are` |

The wording of messages, help and human-readable output is not part of the
interface, and improves without notice.

## The crates

`sprig-core` follows semantic versioning. Its public API is documented, and
anything not documented as public is not covered. Types that are expected to
grow are `#[non_exhaustive]`, so adding a variant or a field is not a breaking
change.

| Promise | Held by |
| --- | --- |
| Every public item is documented. | `#![deny(missing_docs)]` in `crates/sprig-core/src/lib.rs`, so the crate does not build otherwise, and `cargo doc` with warnings denied in `scripts/task lint`. |
| Every public function is shown working in an example that runs as a test. | `crates/sprig-core/tests/api.rs` `every_public_function_has_an_example_or_a_reason`, and `cargo test --doc` in `scripts/task test`. |
| The core never reads the filesystem, the environment, the network or the clock, so the same input gives the same output on every machine and every day. | `scripts/task lint`, the purity guard. |
