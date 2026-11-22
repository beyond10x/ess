---
format: aep.planning-md/3
id: story:work-bound-tests-live-in-one-binary-and-behind-one-counter-module
kind: story
status: draft
title: Work-bound tests live in one binary of ess-conformance and read their counters from one module that release builds do not contain
relations:
- serves: vision:O2
- informed_by: story:the-test-step-meets-the-test-budget
- informed_by: story:one-integration-test-binary-per-large-crate
scope:
- confidence: inferred
  path: .config/nextest.toml
- confidence: inferred
  path: crates/edge/ess-xtask/tests/ci_lanes.rs
- confidence: cited
  path: crates/verify/ess-conformance/Cargo.toml
- confidence: inferred
  path: crates/verify/ess-conformance/src/admission.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/authored.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/fixtures.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/lib.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/scenario.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/created_memo.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/first_kept.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/model_memo.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/run_memo.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/witness_memo.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/caller_readings_made_once_work.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/companion_rows_searched_once_work.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/created_rows_kept_work.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/nested_paths_extended_in_place_work.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/once_per_model_work.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/ranked_and_shorthand_work.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/related_guard_ranked_candidates_work.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/row_set_many_selectors_work.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/rows_moved_by_many_commands_work.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/step_clones_work.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/stored_row_bound_once_work.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/stored_rows_decided_once_work.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/witness_unread_choices_work.rs
revision: 28
---
## Outcome

`ess-conformance` has one integration-test binary for its work-bound tests, one module that owns
the counters and switches those tests read, and a release build that contains none of them. Adding
the next work-bound test adds a module to that binary and a counter to that module, not a link of
the crate's largest test build.

## Need

Release 0.97.3 (tag `0.97.3`, commit `cd9698fd65`) added 11 integration-test binaries named
`*_work.rs` under `crates/verify/ess-conformance/tests/`, 13 in all with the two that already
existed (`related_guard_ranked_candidates_work`, `row_set_many_selectors_work`):

`caller_readings_made_once_work`, `companion_rows_searched_once_work`, `created_rows_kept_work`
(with the module `created_rows_kept_work/wrapped.rs`), `nested_paths_extended_in_place_work`,
`once_per_model_work`, `ranked_and_shorthand_work`, `related_guard_ranked_candidates_work`,
`row_set_many_selectors_work`, `rows_moved_by_many_commands_work`, `step_clones_work`,
`stored_row_bound_once_work`, `stored_rows_decided_once_work`, `witness_unread_choices_work`.

Each file directly under `tests/` is its own test target and links the crate's dependencies
again; the crate had 768 such files at `0.97.1` and has 779 at `0.97.3`
(`git ls-tree --name-only <tag> crates/verify/ess-conformance/tests/ | grep -c '\.rs$'`). The
organization rule is one integration-test binary per crate (`tests/main.rs` with modules), and
`story:one-integration-test-binary-per-large-crate` is the draft that moves the whole crate; this
story moves only the work-bound tests and does not wait for it.

The counters and switches the work-bound tests read sit in release code. Counted on the diff
`0.97.1..0.97.3` of `crates/verify/ess-conformance/src`:

- 42 added `#[doc(hidden)]` items, 28 of them in `synthesize.rs` (re-exports such as
  `first_rows_made`, `conditions_decided`, `literals_collected`, `created_rows_unkept`,
  `wrapped_runs_unkept`, `kept_edges_equal_fresh`, `shorthand_reach_kept_equals_fresh`,
  `ranked_steering_kept_equals_fresh`), in 12 modules;
- `thread_local!` counters and switches added in 24 modules: `witness_memo.rs`, `model_memo.rs`,
  `created_memo.rs`, `run_memo.rs`, `first_kept.rs`, `made.rs`, `subject_fact.rs`, `caller.rs`
  and others, one per fix, each with its own spelling of "count" and of "unkept" (the switch that
  makes the code before the fix run again, so a test can compare bytes of the two).

None is behind a `cfg` or a feature (`git diff 0.97.1 0.97.3 -- crates/verify/ess-conformance/src
| grep -c '^+.*#\[cfg'` prints 0): every counter increment is in the release binary, and every
hook is public API of the crate. No moved file sets an environment variable or the working
directory (`grep -n 'set_var\|set_current_dir\|remove_var'` over the 13 files at `0.97.3` finds
none), so one process running them as threads keeps their meaning.

## Design

- One test target, `crates/verify/ess-conformance/tests/work/main.rs`, with one module per former
  file, moved with `git mv`. The target is registered by `[[test]]` in `Cargo.toml` with its
  `path`, the rest of the crate's test files untouched. Counters are `thread_local!`, so the tests
  of one process keep their counts apart; no work-bound test sets an environment variable or the
  working directory (check with `grep -n 'set_var\|set_current_dir'` over the moved files before
  the move).
- One module of the crate, `src/test_hooks.rs`, owns every counter and switch, declared with one
  macro that defines the `thread_local!` cell, its reader and its reset, and a no-op form of the
  increment. The memo modules call `test_hooks::count!(NAME)` instead of keeping a cell of their
  own. The re-exports in `synthesize.rs` go.
- Compiled out of release: the cells, readers and resets of `test_hooks` are behind a cargo
  feature `test-hooks`, off by default, enabled for the work target only (`required-features` on
  its `[[test]]`, switched on by the crate's own dev-dependency on itself). Without the feature
  the increment expands to nothing, and a switch that product code reads (the `unkept` switches of
  the memos) is a `const fn` returning `false`.
- A counter that costs nothing measurable is the alternative to the feature: a plain `Cell`
  increment is a handful of instructions per call. It is rejected where a counter sits on the
  hottest paths of synthesis (`made.rs`, `subject_fact.rs`), and decision A below settles it.

## Acceptance

- The number of test executables of `ess-conformance` drops by the number folded: 13 files become
  1 target, so 779 become 767 (count before and after, with the command, in the pull request).
- Every work-bound test of the 13 files still exists in the new target under the same name
  (`cargo nextest list -p ess-conformance --test work` names every test the 13 old targets named;
  both lists in the pull request).
- Two checks hold the contract. First, a release build compiles `ess-conformance` without the
  feature: `cargo clean -p ess-conformance --release`, then
  `cargo build --release -p ess-cli --locked -v 2>&1 | grep -c 'feature="test-hooks"'` prints 0
  (the rustc line of the crate names every feature it was built with). Second, an `ess-xtask`
  check reads the source and fails when a `#[doc(hidden)] pub` item of the crate is outside
  `src/test_hooks.rs`, or when a cell, reader or reset in that module is not under
  `#[cfg(feature = "test-hooks")]`.
- The release binary's synthesis time and suite bytes do not move: the byte pins
  `synthesis_suite_bytes_table`, `synthesis_suite_bytes_table_cli_read`,
  `adversary_related_values_all_bytes` and `external_beside_held_guard` pass, and one synthesis of
  `examples/billing` and of a generic model of 48 commands takes the same instructions:u before
  and after (the time is in the pull request).
- Each work-bound test still fails on the unfixed code path. Shown for one test in the pull
  request, and for the others by the same command: check out the parent of the fix commit, copy
  the new `work` target and `test_hooks.rs` onto it, and run that one test. For
  `rows_moved_by_many_commands_are_checked_within_a_bound` the unfixed code checks 27,429 rows
  and inputs against the bound of 13,000 (its own header), so the test must fail on it with the
  message "rows and inputs were checked ... more than 13000". The command and its output for each
  test are listed in the pull request.
- The new target and every test in it run within 60 s, the test step within 120 s
  (`cargo nextest run -p ess-conformance --test work`, time in the pull request).

## Decisions

A. The counters that sit on the hottest paths of synthesis are behind the `test-hooks` feature
   (recommended: nothing in the release binary, a second `ess-conformance` build in the gate),
   or stay as plain counters with a measured cost under 1 % of instructions:u on the generic model
   of 48 commands (nothing to maintain, a hook in the release).

## Known and outside this story

Existing test targets measured over 60 s on this host (numbers from one loaded machine, not a
budget measurement). They belong to `story:the-test-step-meets-the-test-budget`, which holds the
60 s bound and the 2-minute step; none is made faster or slower by this story.

| target | seconds | note |
|---|---|---|
| `output_ownership` | 1,937 | 72 tests |
| `adversary_map_witness_pass1` | 714 | |
| `tutorial_page` | 363 | |
| `authored_scenarios` | 290 | |
| `related_guard_ranked_candidates_work` | 139 | one of the 13 files this story folds |
| `pre_execution_fixtures` | 110 | |
| `served_codec_names` | 98 | |
| `field_presence` | 98 | |
| `witness_directed_search` | 92 | |
| `external_beside_held_guard` | 79 | |
| `coverage_writer_adversary_pass1` | 71 | |

`related_guard_ranked_candidates_work` at 139 s is the one work-bound test over the 60 s bound; it
is moved as it is and keeps its place in the budget story's list.
