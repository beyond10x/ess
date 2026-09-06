---
format: aep.planning-md/1
id: review-result:review-boundaries-6-scenarios-adversary-pass-1
kind: review-result
status: active
title: Wave 6 scenarios adversary pass 1
relations:
- reviews: story:scenarios-directory-compiles-nothing
revision: 1
---
unit: story:scenarios-directory-compiles-nothing at 75db774d84ed5883e670ee4c94a0c2986c87dc12 plus the added adversarial test file
verdict: nothing found
cases: executed 106→115, red 0
origin: introduced 0 / pre-existing 0 / undecided 0
wrote-outside-worktree: 2 shared cache locations; socket and package-lock access disclosed in part 6
needs-coordinator: none
```console
git --no-pager diff --stat
```

The command above returned exit 0 with empty stdout: no tracked file changed. Git's ordinary diff omits the new untracked test; its actual separate stat follows.

```console
git --no-pager diff --no-index --stat /dev/null crates/edge/ess-cli/tests/authored_scenarios_adversary.rs
 .../ess-cli/tests/authored_scenarios_adversary.rs  | 344 +++++++++++++++++++++
 1 file changed, 344 insertions(+)
```

The no-index command returned exit 1, its ordinary difference status. `git status --short` reports only `?? crates/edge/ess-cli/tests/authored_scenarios_adversary.rs`. No production, existing case, planning, Git index/ref, or lifecycle write was made.

## 1. Subject and retained sources

Subject: 75db774d84ed5883e670ee4c94a0c2986c87dc12; base: 009bf3cad2f01eaf1717ea737fa045ffb52d7f12. Worktree: /home/timo/.local/state/worktree/trees/b10x/ess/review-empty-scenarios. Assigned scratch: /home/timo/.local/state/worktree/trees/b10x/ess/review-empty-scenarios/target/review-boundaries-6. The exact 0.7.0 adversary charter was read at /home/timo/.claude/plugins/cache/beyond10x/aep-drive/0.7.0/agents/adversary.md, along with repository AGENTS.md, unit-brief.md, the subject's complete two-file diff, acceptance, original cases, source callers and retained implementor report sections. Oversized historical assertion dumps were inspected with their command/status and output boundaries; every raw byte remains in the implementor's immutable report.

The implementor report SHA-256 is d3d37edef40775be5904f61e155513c2bd477fc5067d0cd4c532ba0000c5aa5f. Its declared final count of 106 is the before count; no preemptive suite was run here. Its baseline binary provenance says it was copied after the unchanged package baseline and before production edits. Its retained SHA-256 was checked before the baseline probes: c10f5ee5a510cecac1adcd5ad2c06f45e7737f5ee35ddb3b05c6e9212f48cd22.

Source read: authored_sources at main.rs:2682; callers in synthesized Run at :2430, synthesize_suite at :2493, conform_web at :2586 and author_suite at :2629. Each propagates the discovery error before its writes. The Run branch selects the reference runner at :2445 after successful discovery. Committed --suite selects the other branch at :2421. Runtime cases assert no runner output and no output mutation; no internal runner-call instrumentation or independent-verifier claim is made.

Final SHA-256 values:

- Production main.rs: 447f242a7c2ef6f8662fe7d89216e9ecd78def8c1edc2396d058f018d761e4d4.
- Existing authored_scenarios.rs: 361b7e983a8f90b148022fb09f5e536f1f039d7f74c2e9977275c6e8591be51e.
- New authored_scenarios_adversary.rs: 4ce6981231e3ed795c6d2063f936488eded76de8d6e054b742a46b8fd30508cb.
- Actual final no-index patch, adversary-final-tests.patch: 15308d05489c6899cf6e96cc65e8a01bf5d5e97059043b07975c3ac46d9ed3e3.

## 2. Added deciding cases, before any suite execution

All nine cases were written before any test execution. Only the new file was formatted, by `rustfmt --edition 2024 crates/edge/ess-cli/tests/authored_scenarios_adversary.rs` (exit 0, empty output retained in adversary-format-apply.log). Each case then ran alone with exact selection: 1 executed, 1 passed, 8 filtered. No added case was red on the subject, and no zero-selected run is counted.

1. `crates/edge/ess-cli/tests/authored_scenarios_adversary.rs:127`, `empty_selection_precedes_conflicting_output_for_every_alias_format_and_runner`: Empty input refusal wins over an output destination that is the selected directory itself; all six operations, both aliases, and text/JSON/YAML give exit 1, an actionable shallow refusal, empty stdout, and no generated entries. **Green now.**

### adversary-case-1

```console
env -u CARGO_TARGET_DIR TMPDIR=/home/timo/.local/state/worktree/trees/b10x/ess/review-empty-scenarios/target CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock GOCACHE=/home/timo/.local/state/worktree/trees/b10x/ess/review-empty-scenarios/target/go-cache GOTMPDIR=/home/timo/.local/state/worktree/trees/b10x/ess/review-empty-scenarios/target/go-tmp cargo test -p ess-cli --test authored_scenarios_adversary --locked --offline empty_selection_precedes_conflicting_output_for_every_alias_format_and_runner -- --exact --nocapture
start: 2026-09-05T23:49:01Z
end: 2026-09-05T23:49:02Z
exit: 0
   Compiling ess-cli v0.19.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-empty-scenarios/crates/edge/ess-cli)
    Finished `test` profile [unoptimized] target(s) in 0.25s
     Running tests/authored_scenarios_adversary.rs (target/debug/deps/authored_scenarios_adversary-5515e5bc921946a6)

running 1 test
test empty_selection_precedes_conflicting_output_for_every_alias_format_and_runner ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.65s

```

2. `crates/edge/ess-cli/tests/authored_scenarios_adversary.rs:150`, `selected_directory_symlink_reports_the_requested_path_and_stays_shallow`: A selected symlink to a corpus containing only a child scenario directory refuses, identifies the user-selected symlink spelling, explains shallow selection, and names the child-directory remedy. **Green now.**

### adversary-case-2

```console
env -u CARGO_TARGET_DIR TMPDIR=/home/timo/.local/state/worktree/trees/b10x/ess/review-empty-scenarios/target CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock GOCACHE=/home/timo/.local/state/worktree/trees/b10x/ess/review-empty-scenarios/target/go-cache GOTMPDIR=/home/timo/.local/state/worktree/trees/b10x/ess/review-empty-scenarios/target/go-tmp cargo test -p ess-cli --test authored_scenarios_adversary --locked --offline selected_directory_symlink_reports_the_requested_path_and_stays_shallow -- --exact --nocapture
start: 2026-09-05T23:49:20Z
end: 2026-09-05T23:49:20Z
exit: 0
    Finished `test` profile [unoptimized] target(s) in 0.05s
     Running tests/authored_scenarios_adversary.rs (target/debug/deps/authored_scenarios_adversary-5515e5bc921946a6)

running 1 test
test selected_directory_symlink_reports_the_requested_path_and_stays_shallow ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.10s

```

3. `crates/edge/ess-cli/tests/authored_scenarios_adversary.rs:178`, `empty_selection_cannot_follow_or_replace_an_output_symlink`: Empty input refusal preserves both an output symlink and its target's binary sentinel across all six operations. **Green now.**

### adversary-case-3

```console
env -u CARGO_TARGET_DIR TMPDIR=/home/timo/.local/state/worktree/trees/b10x/ess/review-empty-scenarios/target CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock GOCACHE=/home/timo/.local/state/worktree/trees/b10x/ess/review-empty-scenarios/target/go-cache GOTMPDIR=/home/timo/.local/state/worktree/trees/b10x/ess/review-empty-scenarios/target/go-tmp cargo test -p ess-cli --test authored_scenarios_adversary --locked --offline empty_selection_cannot_follow_or_replace_an_output_symlink -- --exact --nocapture
start: 2026-09-05T23:49:20Z
end: 2026-09-05T23:49:20Z
exit: 0
    Finished `test` profile [unoptimized] target(s) in 0.05s
     Running tests/authored_scenarios_adversary.rs (target/debug/deps/authored_scenarios_adversary-5515e5bc921946a6)

running 1 test
test empty_selection_cannot_follow_or_replace_an_output_symlink ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.11s

```

4. `crates/edge/ess-cli/tests/authored_scenarios_adversary.rs:200`, `matching_broken_links_are_read_errors_even_beside_a_valid_source`: A dangling matching .yml entry remains a read error both alone and beside valid YAML; no output path appears for any caller. **Green now.**

### adversary-case-4

```console
env -u CARGO_TARGET_DIR TMPDIR=/home/timo/.local/state/worktree/trees/b10x/ess/review-empty-scenarios/target CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock GOCACHE=/home/timo/.local/state/worktree/trees/b10x/ess/review-empty-scenarios/target/go-cache GOTMPDIR=/home/timo/.local/state/worktree/trees/b10x/ess/review-empty-scenarios/target/go-tmp cargo test -p ess-cli --test authored_scenarios_adversary --locked --offline matching_broken_links_are_read_errors_even_beside_a_valid_source -- --exact --nocapture
start: 2026-09-05T23:49:20Z
end: 2026-09-05T23:49:21Z
exit: 0
    Finished `test` profile [unoptimized] target(s) in 0.05s
     Running tests/authored_scenarios_adversary.rs (target/debug/deps/authored_scenarios_adversary-5515e5bc921946a6)

running 1 test
test matching_broken_links_are_read_errors_even_beside_a_valid_source ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.22s

```

5. `crates/edge/ess-cli/tests/authored_scenarios_adversary.rs:221`, `lexical_selection_order_decides_the_first_read_error`: Two matching dangling entries created in opposite orders always report a-first.yml, testing observable sorted reading rather than downstream map serialization. **Green now.**

### adversary-case-5

```console
env -u CARGO_TARGET_DIR TMPDIR=/home/timo/.local/state/worktree/trees/b10x/ess/review-empty-scenarios/target CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock GOCACHE=/home/timo/.local/state/worktree/trees/b10x/ess/review-empty-scenarios/target/go-cache GOTMPDIR=/home/timo/.local/state/worktree/trees/b10x/ess/review-empty-scenarios/target/go-tmp cargo test -p ess-cli --test authored_scenarios_adversary --locked --offline lexical_selection_order_decides_the_first_read_error -- --exact --nocapture
start: 2026-09-05T23:49:21Z
end: 2026-09-05T23:49:21Z
exit: 0
    Finished `test` profile [unoptimized] target(s) in 0.08s
     Running tests/authored_scenarios_adversary.rs (target/debug/deps/authored_scenarios_adversary-5515e5bc921946a6)

running 1 test
test lexical_selection_order_decides_the_first_read_error ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.48s

```

6. `crates/edge/ess-cli/tests/authored_scenarios_adversary.rs:244`, `non_utf8_matching_content_is_refused_before_any_output_write`: A valid first file followed by invalid UTF-8 .yml content refuses during reading and leaves an owned output sentinel unchanged for every caller. **Green now.**

### adversary-case-6

```console
env -u CARGO_TARGET_DIR TMPDIR=/home/timo/.local/state/worktree/trees/b10x/ess/review-empty-scenarios/target CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock GOCACHE=/home/timo/.local/state/worktree/trees/b10x/ess/review-empty-scenarios/target/go-cache GOTMPDIR=/home/timo/.local/state/worktree/trees/b10x/ess/review-empty-scenarios/target/go-tmp cargo test -p ess-cli --test authored_scenarios_adversary --locked --offline non_utf8_matching_content_is_refused_before_any_output_write -- --exact --nocapture
start: 2026-09-05T23:49:22Z
end: 2026-09-05T23:49:22Z
exit: 0
    Finished `test` profile [unoptimized] target(s) in 0.09s
     Running tests/authored_scenarios_adversary.rs (target/debug/deps/authored_scenarios_adversary-5515e5bc921946a6)

running 1 test
test non_utf8_matching_content_is_refused_before_any_output_write ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.14s

```

7. `crates/edge/ess-cli/tests/authored_scenarios_adversary.rs:262`, `valid_file_links_keep_direct_extension_independence_and_ignore_nonmatching_links`: A direct extensionless file, direct file symlink with another extension, and immediate matching file symlink yield identical status and both streams; nested malformed YAML, nonmatching symlink loops and uppercase invalid content stay unselected. **Green now.**

### adversary-case-7

```console
env -u CARGO_TARGET_DIR TMPDIR=/home/timo/.local/state/worktree/trees/b10x/ess/review-empty-scenarios/target CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock GOCACHE=/home/timo/.local/state/worktree/trees/b10x/ess/review-empty-scenarios/target/go-cache GOTMPDIR=/home/timo/.local/state/worktree/trees/b10x/ess/review-empty-scenarios/target/go-tmp cargo test -p ess-cli --test authored_scenarios_adversary --locked --offline valid_file_links_keep_direct_extension_independence_and_ignore_nonmatching_links -- --exact --nocapture
start: 2026-09-05T23:49:22Z
end: 2026-09-05T23:49:23Z
exit: 0
    Finished `test` profile [unoptimized] target(s) in 0.07s
     Running tests/authored_scenarios_adversary.rs (target/debug/deps/authored_scenarios_adversary-5515e5bc921946a6)

running 1 test
test valid_file_links_keep_direct_extension_independence_and_ignore_nonmatching_links ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.43s

```

8. `crates/edge/ess-cli/tests/authored_scenarios_adversary.rs:296`, `omitted_scenarios_ignore_a_poisoned_working_directory_default`: Moving the command into a working directory with malformed scenarios/bad.yaml does not change omitted-flag status or either stream. **Green now.**

### adversary-case-8

```console
env -u CARGO_TARGET_DIR TMPDIR=/home/timo/.local/state/worktree/trees/b10x/ess/review-empty-scenarios/target CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock GOCACHE=/home/timo/.local/state/worktree/trees/b10x/ess/review-empty-scenarios/target/go-cache GOTMPDIR=/home/timo/.local/state/worktree/trees/b10x/ess/review-empty-scenarios/target/go-tmp cargo test -p ess-cli --test authored_scenarios_adversary --locked --offline omitted_scenarios_ignore_a_poisoned_working_directory_default -- --exact --nocapture
start: 2026-09-05T23:49:23Z
end: 2026-09-05T23:49:23Z
exit: 0
    Finished `test` profile [unoptimized] target(s) in 0.06s
     Running tests/authored_scenarios_adversary.rs (target/debug/deps/authored_scenarios_adversary-5515e5bc921946a6)

running 1 test
test omitted_scenarios_ignore_a_poisoned_working_directory_default ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.28s

```

9. `crates/edge/ess-cli/tests/authored_scenarios_adversary.rs:309`, `committed_suite_bypasses_poisoned_scenarios_and_missing_model_for_both_runners`: A committed suite bypasses malformed UTF-8 input, a symlink loop, and a directory containing child.yml as a directory even with a missing model path; both built-in reference targets retain identical status and streams. **Green now.**

### adversary-case-9

```console
env -u CARGO_TARGET_DIR TMPDIR=/home/timo/.local/state/worktree/trees/b10x/ess/review-empty-scenarios/target CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock GOCACHE=/home/timo/.local/state/worktree/trees/b10x/ess/review-empty-scenarios/target/go-cache GOTMPDIR=/home/timo/.local/state/worktree/trees/b10x/ess/review-empty-scenarios/target/go-tmp cargo test -p ess-cli --test authored_scenarios_adversary --locked --offline committed_suite_bypasses_poisoned_scenarios_and_missing_model_for_both_runners -- --exact --nocapture
start: 2026-09-05T23:49:23Z
end: 2026-09-05T23:49:24Z
exit: 0
    Finished `test` profile [unoptimized] target(s) in 0.06s
     Running tests/authored_scenarios_adversary.rs (target/debug/deps/authored_scenarios_adversary-5515e5bc921946a6)

running 1 test
test committed_suite_bypasses_poisoned_scenarios_and_missing_model_for_both_runners ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.14s

```

## 3. Package execution and subsequent controls, verbatim

After the nine individual executions, the same standard-library-only test file was compiled into the assigned scratch directory with CARGO_BIN_EXE_ess naming the retained base binary. That compiles a separate test harness and changes no production file or checkout. The first full current package run then executed 115 cases, all passing. Subsequent baseline executions are diagnostic regression controls: the three empty-selection cases fail against the historical binary, while the six preservation cases pass. Those historical failures are not findings on the subject and are not counted as current red cases.

The initial strict Clippy run found manual_assert_eq at my new test's line 172. I changed only that added assertion from `assert!(count == 1)` to `assert_eq!(count, 1)`, preserving its condition. No production or existing case changed. Final package formatting and strict Clippy returned 0; a final full package run on the final test bytes again executed 115 cases, with 0 failed, 0 ignored and 0 filtered. All original 106 cases remain selected.

### adversary-baseline-compile

```console
env -u CARGO_TARGET_DIR TMPDIR=/home/timo/.local/state/worktree/trees/b10x/ess/review-empty-scenarios/target CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock GOCACHE=/home/timo/.local/state/worktree/trees/b10x/ess/review-empty-scenarios/target/go-cache GOTMPDIR=/home/timo/.local/state/worktree/trees/b10x/ess/review-empty-scenarios/target/go-tmp CARGO_MANIFEST_DIR=/home/timo/.local/state/worktree/trees/b10x/ess/review-empty-scenarios/crates/edge/ess-cli CARGO_BIN_EXE_ess=/home/timo/.local/state/worktree/trees/b10x/ess/review-empty-scenarios/target/review-boundaries-6/baseline-ess rustc --test --edition=2024 crates/edge/ess-cli/tests/authored_scenarios_adversary.rs -o target/review-boundaries-6/adversary-baseline-tests
start: 2026-09-05T23:49:51Z
end: 2026-09-05T23:49:51Z
exit: 0
```

### adversary-suite

```console
env -u CARGO_TARGET_DIR TMPDIR=/home/timo/.local/state/worktree/trees/b10x/ess/review-empty-scenarios/target CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock GOCACHE=/home/timo/.local/state/worktree/trees/b10x/ess/review-empty-scenarios/target/go-cache GOTMPDIR=/home/timo/.local/state/worktree/trees/b10x/ess/review-empty-scenarios/target/go-tmp cargo test -p ess-cli --locked --offline
start: 2026-09-05T23:49:51Z
end: 2026-09-05T23:49:55Z
exit: 0
    Finished `test` profile [unoptimized] target(s) in 0.06s
     Running unittests src/main.rs (target/debug/deps/ess-c6179ec2412cb9c0)

running 11 tests
test tests::a_symlink_above_the_requested_root_is_refused ... ok
test tests::normalizing_a_requested_root_does_not_hide_a_symlink_traversal ... ok
test tests::caller_selected_parent_roots_resolve_without_creating_discarded_directories ... ok
test tests::projection_files_and_existing_aliases_are_checked_as_one_set ... ok
test tests::generated_file_conflicts_are_refused_before_new_directories_are_created ... ok
test tests::every_artifact_destination_is_checked_before_the_first_write ... ok
test tests::the_first_level_is_exactly_the_four_areas ... ok
test tests::the_generate_area_answers_to_the_flat_spelling_and_to_its_own ... ok
test tests::every_leaf_is_reachable_by_its_area_path_and_by_its_flat_spelling ... ok
test tests::every_command_and_argument_name_is_unambiguous ... ok
test tests::no_manifest_or_lockfile_depends_on_aep ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/authored_scenarios.rs (target/debug/deps/authored_scenarios-b0b979ddfeafab3b)

running 25 tests
test author_nested_only ... ok
test author_nonmatching_only ... ok
test synthesize_help_describes_explicit_shallow_selection_without_a_default ... ok
test author_empty ... ok
test empty_selection_refusal_is_identical_through_flat_and_area_spellings ... ok
test go_empty ... ok
test ir_nonmatching_only ... ok
test ir_nested_only ... ok
test web_nested_only ... ok
test run_empty ... ok
test run_nonmatching_only ... ok
test go_nested_only ... ok
test web_empty ... ok
test web_nonmatching_only ... ok
test ir_empty ... ok
test run_nested_only ... ok
test go_nonmatching_only ... ok
test a_yaml_named_subdirectory_alone_retains_its_read_refusal ... ok
test supplied_suite_bypasses_empty_and_nonexistent_scenario_paths ... ok
test a_matching_directory_symlink_beside_a_valid_scenario_retains_its_read_refusal ... ok
test a_matching_directory_beside_a_valid_scenario_retains_its_read_refusal ... ok
test omitted_scenarios_preserve_intentional_generated_and_authored_selections ... ok
test shallow_yaml_and_yml_selection_is_independent_of_creation_order ... ok
test missing_paths_and_malformed_matching_sources_remain_failures ... ok
test explicit_files_and_shallow_directories_select_the_same_scenario ... ok

test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.58s

     Running tests/authored_scenarios_adversary.rs (target/debug/deps/authored_scenarios_adversary-5515e5bc921946a6)

running 9 tests
test empty_selection_cannot_follow_or_replace_an_output_symlink ... ok
test selected_directory_symlink_reports_the_requested_path_and_stays_shallow ... ok
test non_utf8_matching_content_is_refused_before_any_output_write ... ok
test committed_suite_bypasses_poisoned_scenarios_and_missing_model_for_both_runners ... ok
test lexical_selection_order_decides_the_first_read_error ... ok
test matching_broken_links_are_read_errors_even_beside_a_valid_source ... ok
test omitted_scenarios_ignore_a_poisoned_working_directory_default ... ok
test valid_file_links_keep_direct_extension_independence_and_ignore_nonmatching_links ... ok
test empty_selection_precedes_conflicting_output_for_every_alias_format_and_runner ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.69s

     Running tests/authored_site.rs (target/debug/deps/authored_site-e4ea45f2ef9aa761)

running 9 tests
test binary_downloads_are_not_silently_decoded ... ok
test an_explicit_missing_front_page_is_an_error ... ok
test asset_symlink_output_is_refused_before_any_page_changes ... ok
test a_page_identity_can_itself_end_in_html ... ok
test publication_without_strict_mode_preserves_unpublished_links ... ok
test strict_links_check_local_and_cross_page_fragments ... ok
test assets_cannot_collide_with_generated_output_or_escape_it ... ok
test selected_sources_resolve_after_relocation_with_queries_fragments_and_downloads ... ok
test undeclared_existing_and_escaping_targets_are_refused_with_source_lines ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

     Running tests/command_surface.rs (target/debug/deps/command_surface-600321eec2d09fc2)

running 5 tests
test the_generate_area_help_offers_the_verbs_options_and_the_areas_subcommands ... ok
test the_help_offers_exactly_the_four_areas ... ok
test a_clap_refusal_differs_only_in_its_usage_line ... ok
test the_generate_area_refuses_its_arguments_beside_a_sibling_verb ... ok
test a_flat_spelling_prints_what_its_area_path_prints ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.28s

     Running tests/command_surface_adversary.rs (target/debug/deps/command_surface_adversary-6c0f0b6221974cca)

running 4 tests
test the_generate_area_honours_the_arguments_it_accepts_or_refuses_them ... ok
test the_generate_area_does_not_run_a_sibling_verb_against_a_path_it_was_not_given ... ok
test a_flat_spelling_prints_what_its_area_path_prints_when_clap_refuses ... ok
test the_generate_usage_line_admits_the_arguments_the_command_takes ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

     Running tests/feasibility_adversary.rs (target/debug/deps/feasibility_adversary-2310f44e76a94dfc)

running 4 tests
test web_dependency_module_collision_is_refused_before_cli_output ... ok
test http_codec_local_collision_is_refused_before_cli_output ... ok
test a_binding_function_capture_is_refused_before_web_cli_writes ... ok
test a_binding_function_capture_is_refused_before_rust_cli_writes ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/go_conformance.rs (target/debug/deps/go_conformance-8db715bb634290f2)

running 7 tests
test a_view_that_drops_rows_fails_the_scenarios_that_say_how_many_it_holds ... ok
test a_view_returned_in_the_wrong_order_fails_exactly_the_scenarios_that_assert_its_order ... ok
test one_deliberate_defect_fails_the_scenarios_responsible_for_it_and_no_others ... ok
test the_emitted_package_holds_a_correct_go_implementation_to_the_whole_suite ... ok
test the_emitted_runner_reads_a_positional_assertion_and_refuses_one_in_an_unordered_view ... ok
test the_emitted_runner_holds_a_window_and_fails_a_target_whose_clock_never_moves ... ok
test the_emitted_runner_stops_a_scan_and_fails_a_target_that_builds_the_whole_listing ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.81s

     Running tests/model_types.rs (target/debug/deps/model_types-bce7d20b0294c36c)

running 2 tests
test root_selection_and_output_refusals_preserve_existing_files ... ok
test each_target_retains_the_same_model_selection_and_distinct_input_provenance ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

     Running tests/normalization.rs (target/debug/deps/normalization-caa8f5163b24b34a)

running 5 tests
test output_symlinks_and_hardlinks_are_refused_without_mutation ... ok
test stale_bundle_missing_source_and_unknown_dispatch_refuse ... ok
test checked_recipe_and_run_are_deterministic_with_json_only_stdout ... ok
test output_cannot_replace_any_declared_input ... ok
test failed_check_or_execution_preserves_output_and_does_not_emit_a_result ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s

     Running tests/output_containment.rs (target/debug/deps/output_containment-2fd0b54e38132c26)

running 19 tests
test an_escaping_include_is_refused_before_any_output_changes ... ok
test composition_keeps_native_non_utf8_and_backslash_filenames_distinct ... ok
test composition_refuses_cancelled_parent_links_before_disjoint_companions_change ... ok
test a_hardlinked_destination_is_refused_before_other_files_change ... ok
test composition_keeps_disjoint_caller_selected_filenames_and_parent_roots ... ok
test composition_preserves_disjoint_files_inside_generated_directories ... ok
test a_valid_nested_include_keeps_the_existing_site_layout_and_bytes ... ok
test composition_companion_outputs_cannot_collide_with_the_generated_client_tree ... ok
test late_site_asset_aliases_refuse_before_even_creating_output_directories ... ok
test composition_refuses_companion_links_before_any_other_output_changes ... ok
test composition_does_not_reinterpret_directory_spelling_as_a_named_output_file ... ok
test requested_root_normalization_preserves_parent_roots_and_rejects_hidden_files ... ok
test noncanonical_and_platform_paths_are_refused_before_writing ... ok
test symlink_roots_parents_and_destinations_are_refused_before_writing ... ok
test composition_preflight_includes_companion_generated_aliases_and_both_companions ... ok
test composition_companions_form_one_output_set_even_without_a_generated_tree ... ok
test include_aliases_and_duplicate_generated_pages_are_refused_before_writing ... ok
test local_projection_sinks_refuse_late_conflicts_before_any_generated_file_changes ... ok
test local_generation_sinks_refuse_late_conflicts_before_any_generated_file_changes ... ok

test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.45s

     Running tests/persisted_delivery.rs (target/debug/deps/persisted_delivery-4fd7562842c2b3a8)

running 6 tests
test valid_plan_reaches_both_local_fake_executors_in_rollout_order ... ok
test adversary_noncanonical_topological_order_is_refused_before_execution ... ok
test invalid_current_removal_is_refused_before_analysis_and_execution ... ok
test adversary_duplicate_desired_keys_are_refused_before_any_executor ... ok
test adversary_duplicate_current_keys_block_removal_and_diff ... ok
test entire_desired_plan_is_refused_before_oras_or_helm ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s

     Running tests/schema_bundle.rs (target/debug/deps/schema_bundle-1ce25b829db41234)

running 8 tests
test missing_dialect_and_incomplete_import_leave_existing_output_alone ... ok
test document_import_refusals_preserve_source_and_existing_output ... ok
test corrupted_import_cannot_be_projected_and_output_cannot_replace_source ... ok
test type_planning_and_output_refusals_leave_no_partial_library ... ok
test types_bundle_is_deterministic_and_never_replaces_its_input ... ok
test import_reload_projection_and_instance_validation_keep_original_data ... ok
test document_root_survives_reload_projection_and_each_type_target ... ok
test native_bundle_targets_require_identity_and_emit_build_metadata ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s

     Running tests/target_failure.rs (target/debug/deps/target_failure-143daa876f36b724)

running 1 test
test fatal_synthesis_preserves_destinations_and_has_a_typed_envelope ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s

```

### adversary-baseline-red-1

```console
env -u CARGO_TARGET_DIR TMPDIR=/home/timo/.local/state/worktree/trees/b10x/ess/review-empty-scenarios/target CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock GOCACHE=/home/timo/.local/state/worktree/trees/b10x/ess/review-empty-scenarios/target/go-cache GOTMPDIR=/home/timo/.local/state/worktree/trees/b10x/ess/review-empty-scenarios/target/go-tmp target/review-boundaries-6/adversary-baseline-tests empty_selection_precedes_conflicting_output_for_every_alias_format_and_runner --exact --nocapture
start: 2026-09-05T23:50:11Z
end: 2026-09-05T23:50:11Z
exit: 101

running 1 test

thread 'empty_selection_precedes_conflicting_output_for_every_alias_format_and_runner' (1263947) panicked at crates/edge/ess-cli/tests/authored_scenarios_adversary.rs:112:9:
expected "refused --scenarios" in "error: writing /home/timo/.local/state/worktree/trees/b10x/ess/review-empty-scenarios/target/review-boundaries-6/adversary-fixture-1263946-0/empty selection: Is a directory (os error 21)\n"
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test empty_selection_precedes_conflicting_output_for_every_alias_format_and_runner ... FAILED

failures:

failures:
    empty_selection_precedes_conflicting_output_for_every_alias_format_and_runner

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.05s

```

### adversary-baseline-red-2

```console
env -u CARGO_TARGET_DIR TMPDIR=/home/timo/.local/state/worktree/trees/b10x/ess/review-empty-scenarios/target CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock GOCACHE=/home/timo/.local/state/worktree/trees/b10x/ess/review-empty-scenarios/target/go-cache GOTMPDIR=/home/timo/.local/state/worktree/trees/b10x/ess/review-empty-scenarios/target/go-tmp target/review-boundaries-6/adversary-baseline-tests selected_directory_symlink_reports_the_requested_path_and_stays_shallow --exact --nocapture
start: 2026-09-05T23:50:11Z
end: 2026-09-05T23:50:11Z
exit: 101

running 1 test

thread 'selected_directory_symlink_reports_the_requested_path_and_stays_shallow' (1264014) panicked at crates/edge/ess-cli/tests/authored_scenarios_adversary.rs:110:5:
assertion `left == right` failed: stderr: 
  left: Some(0)
 right: Some(1)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test selected_directory_symlink_reports_the_requested_path_and_stays_shallow ... FAILED

failures:

failures:
    selected_directory_symlink_reports_the_requested_path_and_stays_shallow

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.03s

```

### adversary-baseline-red-3

```console
env -u CARGO_TARGET_DIR TMPDIR=/home/timo/.local/state/worktree/trees/b10x/ess/review-empty-scenarios/target CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock GOCACHE=/home/timo/.local/state/worktree/trees/b10x/ess/review-empty-scenarios/target/go-cache GOTMPDIR=/home/timo/.local/state/worktree/trees/b10x/ess/review-empty-scenarios/target/go-tmp target/review-boundaries-6/adversary-baseline-tests empty_selection_cannot_follow_or_replace_an_output_symlink --exact --nocapture
start: 2026-09-05T23:50:11Z
end: 2026-09-05T23:50:11Z
exit: 101

running 1 test

thread 'empty_selection_cannot_follow_or_replace_an_output_symlink' (1264109) panicked at crates/edge/ess-cli/tests/authored_scenarios_adversary.rs:110:5:
assertion `left == right` failed: stderr: 
  left: Some(0)
 right: Some(1)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test empty_selection_cannot_follow_or_replace_an_output_symlink ... FAILED

failures:

failures:
    empty_selection_cannot_follow_or_replace_an_output_symlink

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.03s

```

### adversary-baseline-preserved

```console
env -u CARGO_TARGET_DIR TMPDIR=/home/timo/.local/state/worktree/trees/b10x/ess/review-empty-scenarios/target CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock GOCACHE=/home/timo/.local/state/worktree/trees/b10x/ess/review-empty-scenarios/target/go-cache GOTMPDIR=/home/timo/.local/state/worktree/trees/b10x/ess/review-empty-scenarios/target/go-tmp target/review-boundaries-6/adversary-baseline-tests --skip empty_selection --skip selected_directory_symlink --nocapture
start: 2026-09-05T23:50:11Z
end: 2026-09-05T23:50:12Z
exit: 0

running 6 tests
test non_utf8_matching_content_is_refused_before_any_output_write ... ok
test committed_suite_bypasses_poisoned_scenarios_and_missing_model_for_both_runners ... ok
test lexical_selection_order_decides_the_first_read_error ... ok
test matching_broken_links_are_read_errors_even_beside_a_valid_source ... ok
test omitted_scenarios_ignore_a_poisoned_working_directory_default ... ok
test valid_file_links_keep_direct_extension_independence_and_ignore_nonmatching_links ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.42s

```

### adversary-fmt

```console
env -u CARGO_TARGET_DIR TMPDIR=/home/timo/.local/state/worktree/trees/b10x/ess/review-empty-scenarios/target CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock GOCACHE=/home/timo/.local/state/worktree/trees/b10x/ess/review-empty-scenarios/target/go-cache GOTMPDIR=/home/timo/.local/state/worktree/trees/b10x/ess/review-empty-scenarios/target/go-tmp cargo fmt -p ess-cli --check
start: 2026-09-05T23:50:12Z
end: 2026-09-05T23:50:12Z
exit: 0
```

### adversary-clippy

```console
env -u CARGO_TARGET_DIR TMPDIR=/home/timo/.local/state/worktree/trees/b10x/ess/review-empty-scenarios/target CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock GOCACHE=/home/timo/.local/state/worktree/trees/b10x/ess/review-empty-scenarios/target/go-cache GOTMPDIR=/home/timo/.local/state/worktree/trees/b10x/ess/review-empty-scenarios/target/go-tmp cargo clippy -p ess-cli --all-targets --locked --offline -- -D warnings
start: 2026-09-05T23:50:12Z
end: 2026-09-05T23:50:12Z
exit: 101
    Checking ess-cli v0.19.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-empty-scenarios/crates/edge/ess-cli)
error: used `assert!` with an equality comparison
   --> crates/edge/ess-cli/tests/authored_scenarios_adversary.rs:172:9
    |
172 |         assert!(fs::read_dir(&child).unwrap().count() == 1);
    |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#manual_assert_eq
    = note: `-D clippy::manual-assert-eq` implied by `-D warnings`
    = help: to override `-D warnings` add `#[allow(clippy::manual_assert_eq)]`
help: replace it with `assert_eq!(..)`
    |
172 -         assert!(fs::read_dir(&child).unwrap().count() == 1);
172 +         assert_eq!(fs::read_dir(&child).unwrap().count(), 1);
    |

error: could not compile `ess-cli` (test "authored_scenarios_adversary") due to 1 previous error
```

### adversary-fmt-final

```console
env -u CARGO_TARGET_DIR TMPDIR=/home/timo/.local/state/worktree/trees/b10x/ess/review-empty-scenarios/target CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock GOCACHE=/home/timo/.local/state/worktree/trees/b10x/ess/review-empty-scenarios/target/go-cache GOTMPDIR=/home/timo/.local/state/worktree/trees/b10x/ess/review-empty-scenarios/target/go-tmp cargo fmt -p ess-cli --check
start: 2026-09-05T23:50:40Z
end: 2026-09-05T23:50:40Z
exit: 0
```

### adversary-clippy-final

```console
env -u CARGO_TARGET_DIR TMPDIR=/home/timo/.local/state/worktree/trees/b10x/ess/review-empty-scenarios/target CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock GOCACHE=/home/timo/.local/state/worktree/trees/b10x/ess/review-empty-scenarios/target/go-cache GOTMPDIR=/home/timo/.local/state/worktree/trees/b10x/ess/review-empty-scenarios/target/go-tmp cargo clippy -p ess-cli --all-targets --locked --offline -- -D warnings
start: 2026-09-05T23:50:40Z
end: 2026-09-05T23:50:40Z
exit: 0
    Checking ess-cli v0.19.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-empty-scenarios/crates/edge/ess-cli)
    Finished `dev` profile [unoptimized] target(s) in 0.11s
```

### adversary-suite-final

```console
env -u CARGO_TARGET_DIR TMPDIR=/home/timo/.local/state/worktree/trees/b10x/ess/review-empty-scenarios/target CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock GOCACHE=/home/timo/.local/state/worktree/trees/b10x/ess/review-empty-scenarios/target/go-cache GOTMPDIR=/home/timo/.local/state/worktree/trees/b10x/ess/review-empty-scenarios/target/go-tmp cargo test -p ess-cli --locked --offline
start: 2026-09-05T23:50:41Z
end: 2026-09-05T23:50:44Z
exit: 0
   Compiling ess-cli v0.19.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-empty-scenarios/crates/edge/ess-cli)
    Finished `test` profile [unoptimized] target(s) in 0.22s
     Running unittests src/main.rs (target/debug/deps/ess-c6179ec2412cb9c0)

running 11 tests
test tests::a_symlink_above_the_requested_root_is_refused ... ok
test tests::normalizing_a_requested_root_does_not_hide_a_symlink_traversal ... ok
test tests::generated_file_conflicts_are_refused_before_new_directories_are_created ... ok
test tests::caller_selected_parent_roots_resolve_without_creating_discarded_directories ... ok
test tests::projection_files_and_existing_aliases_are_checked_as_one_set ... ok
test tests::every_artifact_destination_is_checked_before_the_first_write ... ok
test tests::the_first_level_is_exactly_the_four_areas ... ok
test tests::the_generate_area_answers_to_the_flat_spelling_and_to_its_own ... ok
test tests::every_leaf_is_reachable_by_its_area_path_and_by_its_flat_spelling ... ok
test tests::no_manifest_or_lockfile_depends_on_aep ... ok
test tests::every_command_and_argument_name_is_unambiguous ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/authored_scenarios.rs (target/debug/deps/authored_scenarios-b0b979ddfeafab3b)

running 25 tests
test author_nested_only ... ok
test author_nonmatching_only ... ok
test synthesize_help_describes_explicit_shallow_selection_without_a_default ... ok
test author_empty ... ok
test empty_selection_refusal_is_identical_through_flat_and_area_spellings ... ok
test ir_nonmatching_only ... ok
test run_nested_only ... ok
test go_nested_only ... ok
test web_nonmatching_only ... ok
test ir_nested_only ... ok
test ir_empty ... ok
test run_nonmatching_only ... ok
test web_nested_only ... ok
test go_empty ... ok
test go_nonmatching_only ... ok
test run_empty ... ok
test web_empty ... ok
test a_yaml_named_subdirectory_alone_retains_its_read_refusal ... ok
test supplied_suite_bypasses_empty_and_nonexistent_scenario_paths ... ok
test a_matching_directory_symlink_beside_a_valid_scenario_retains_its_read_refusal ... ok
test a_matching_directory_beside_a_valid_scenario_retains_its_read_refusal ... ok
test omitted_scenarios_preserve_intentional_generated_and_authored_selections ... ok
test missing_paths_and_malformed_matching_sources_remain_failures ... ok
test shallow_yaml_and_yml_selection_is_independent_of_creation_order ... ok
test explicit_files_and_shallow_directories_select_the_same_scenario ... ok

test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.58s

     Running tests/authored_scenarios_adversary.rs (target/debug/deps/authored_scenarios_adversary-5515e5bc921946a6)

running 9 tests
test selected_directory_symlink_reports_the_requested_path_and_stays_shallow ... ok
test empty_selection_cannot_follow_or_replace_an_output_symlink ... ok
test non_utf8_matching_content_is_refused_before_any_output_write ... ok
test committed_suite_bypasses_poisoned_scenarios_and_missing_model_for_both_runners ... ok
test matching_broken_links_are_read_errors_even_beside_a_valid_source ... ok
test lexical_selection_order_decides_the_first_read_error ... ok
test omitted_scenarios_ignore_a_poisoned_working_directory_default ... ok
test valid_file_links_keep_direct_extension_independence_and_ignore_nonmatching_links ... ok
test empty_selection_precedes_conflicting_output_for_every_alias_format_and_runner ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.75s

     Running tests/authored_site.rs (target/debug/deps/authored_site-e4ea45f2ef9aa761)

running 9 tests
test an_explicit_missing_front_page_is_an_error ... ok
test binary_downloads_are_not_silently_decoded ... ok
test asset_symlink_output_is_refused_before_any_page_changes ... ok
test a_page_identity_can_itself_end_in_html ... ok
test publication_without_strict_mode_preserves_unpublished_links ... ok
test strict_links_check_local_and_cross_page_fragments ... ok
test selected_sources_resolve_after_relocation_with_queries_fragments_and_downloads ... ok
test assets_cannot_collide_with_generated_output_or_escape_it ... ok
test undeclared_existing_and_escaping_targets_are_refused_with_source_lines ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/command_surface.rs (target/debug/deps/command_surface-600321eec2d09fc2)

running 5 tests
test the_help_offers_exactly_the_four_areas ... ok
test the_generate_area_help_offers_the_verbs_options_and_the_areas_subcommands ... ok
test a_clap_refusal_differs_only_in_its_usage_line ... ok
test the_generate_area_refuses_its_arguments_beside_a_sibling_verb ... ok
test a_flat_spelling_prints_what_its_area_path_prints ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.30s

     Running tests/command_surface_adversary.rs (target/debug/deps/command_surface_adversary-6c0f0b6221974cca)

running 4 tests
test the_generate_area_does_not_run_a_sibling_verb_against_a_path_it_was_not_given ... ok
test the_generate_area_honours_the_arguments_it_accepts_or_refuses_them ... ok
test a_flat_spelling_prints_what_its_area_path_prints_when_clap_refuses ... ok
test the_generate_usage_line_admits_the_arguments_the_command_takes ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

     Running tests/feasibility_adversary.rs (target/debug/deps/feasibility_adversary-2310f44e76a94dfc)

running 4 tests
test web_dependency_module_collision_is_refused_before_cli_output ... ok
test a_binding_function_capture_is_refused_before_rust_cli_writes ... ok
test http_codec_local_collision_is_refused_before_cli_output ... ok
test a_binding_function_capture_is_refused_before_web_cli_writes ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/go_conformance.rs (target/debug/deps/go_conformance-8db715bb634290f2)

running 7 tests
test the_emitted_runner_reads_a_positional_assertion_and_refuses_one_in_an_unordered_view ... ok
test a_view_returned_in_the_wrong_order_fails_exactly_the_scenarios_that_assert_its_order ... ok
test a_view_that_drops_rows_fails_the_scenarios_that_say_how_many_it_holds ... ok
test one_deliberate_defect_fails_the_scenarios_responsible_for_it_and_no_others ... ok
test the_emitted_package_holds_a_correct_go_implementation_to_the_whole_suite ... ok
test the_emitted_runner_stops_a_scan_and_fails_a_target_that_builds_the_whole_listing ... ok
test the_emitted_runner_holds_a_window_and_fails_a_target_whose_clock_never_moves ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.81s

     Running tests/model_types.rs (target/debug/deps/model_types-bce7d20b0294c36c)

running 2 tests
test root_selection_and_output_refusals_preserve_existing_files ... ok
test each_target_retains_the_same_model_selection_and_distinct_input_provenance ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

     Running tests/normalization.rs (target/debug/deps/normalization-caa8f5163b24b34a)

running 5 tests
test output_symlinks_and_hardlinks_are_refused_without_mutation ... ok
test stale_bundle_missing_source_and_unknown_dispatch_refuse ... ok
test checked_recipe_and_run_are_deterministic_with_json_only_stdout ... ok
test output_cannot_replace_any_declared_input ... ok
test failed_check_or_execution_preserves_output_and_does_not_emit_a_result ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s

     Running tests/output_containment.rs (target/debug/deps/output_containment-2fd0b54e38132c26)

running 19 tests
test an_escaping_include_is_refused_before_any_output_changes ... ok
test composition_keeps_native_non_utf8_and_backslash_filenames_distinct ... ok
test a_hardlinked_destination_is_refused_before_other_files_change ... ok
test composition_keeps_disjoint_caller_selected_filenames_and_parent_roots ... ok
test composition_refuses_cancelled_parent_links_before_disjoint_companions_change ... ok
test composition_preserves_disjoint_files_inside_generated_directories ... ok
test composition_refuses_companion_links_before_any_other_output_changes ... ok
test a_valid_nested_include_keeps_the_existing_site_layout_and_bytes ... ok
test composition_companion_outputs_cannot_collide_with_the_generated_client_tree ... ok
test late_site_asset_aliases_refuse_before_even_creating_output_directories ... ok
test requested_root_normalization_preserves_parent_roots_and_rejects_hidden_files ... ok
test include_aliases_and_duplicate_generated_pages_are_refused_before_writing ... ok
test composition_does_not_reinterpret_directory_spelling_as_a_named_output_file ... ok
test symlink_roots_parents_and_destinations_are_refused_before_writing ... ok
test noncanonical_and_platform_paths_are_refused_before_writing ... ok
test composition_preflight_includes_companion_generated_aliases_and_both_companions ... ok
test composition_companions_form_one_output_set_even_without_a_generated_tree ... ok
test local_projection_sinks_refuse_late_conflicts_before_any_generated_file_changes ... ok
test local_generation_sinks_refuse_late_conflicts_before_any_generated_file_changes ... ok

test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.47s

     Running tests/persisted_delivery.rs (target/debug/deps/persisted_delivery-4fd7562842c2b3a8)

running 6 tests
test valid_plan_reaches_both_local_fake_executors_in_rollout_order ... ok
test adversary_noncanonical_topological_order_is_refused_before_execution ... ok
test adversary_duplicate_desired_keys_are_refused_before_any_executor ... ok
test invalid_current_removal_is_refused_before_analysis_and_execution ... ok
test adversary_duplicate_current_keys_block_removal_and_diff ... ok
test entire_desired_plan_is_refused_before_oras_or_helm ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s

     Running tests/schema_bundle.rs (target/debug/deps/schema_bundle-1ce25b829db41234)

running 8 tests
test missing_dialect_and_incomplete_import_leave_existing_output_alone ... ok
test document_import_refusals_preserve_source_and_existing_output ... ok
test corrupted_import_cannot_be_projected_and_output_cannot_replace_source ... ok
test type_planning_and_output_refusals_leave_no_partial_library ... ok
test types_bundle_is_deterministic_and_never_replaces_its_input ... ok
test import_reload_projection_and_instance_validation_keep_original_data ... ok
test document_root_survives_reload_projection_and_each_type_target ... ok
test native_bundle_targets_require_identity_and_emit_build_metadata ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s

     Running tests/target_failure.rs (target/debug/deps/target_failure-143daa876f36b724)

running 1 test
test fatal_synthesis_preserves_destinations_and_has_a_typed_envelope ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s

```

Actual final per-runner counts: main 11; authored_scenarios 25; authored_scenarios_adversary 9; authored_site 9; command_surface 5; command_surface_adversary 4; feasibility_adversary 4; go_conformance 7; model_types 2; normalization 5; output_containment 19; persisted_delivery 6; schema_bundle 8; target_failure 1. Total: 115. The seven Go integration tests ran; no unavailable-toolchain skip was printed.

The `--skip` flags appear only in the historical baseline preservation control and deselect exactly the three new-empty-refusal expectations already run red individually. No subject package test was skipped or ignored. The final full gate owned by the coordinator is not claimed here.

## 4. Judgement findings

Nothing found on subject 75db774d84ed5883e670ee4c94a0c2986c87dc12; there are no finding rows.

## 5. Attacks that did not break the subject

- Empty selection beats conflicting directory destinations and preserves linked output sentinels across the real command aliases, formats and reference targets.
- Selected directory symlinks preserve the requested diagnostic path and do not make discovery recursive.
- Matching broken links and non-UTF-8 files remain read failures before output; existing matching-directory and matching-directory-symlink regressions also remain green.
- Reversed filesystem creation order still produces the lexical first read failure.
- Direct extensionless files and regular-file symlinks preserve status and bytes, while unselected nested malformed documents, loops and uppercase extensions stay ignored.
- Omission does not discover a poisoned working-directory default; committed suites bypass poisoned scenario paths and missing models.

General output ownership, semantic compilation partial writes after nonempty discovery, and typed/recursive mixed-document discovery remain outside this assigned unit. No new theory in those areas is presented as a finding.

## 6. Paths outside the worktree and handoff

All explicit test, command-record, fixture, build, standalone-harness, log, patch and report writes are under /home/timo/.local/state/worktree/trees/b10x/ess/review-empty-scenarios. No cleanup ran, and every record/build directory remains. New subprocess records are retained under target/review-boundaries-6/adversary-fixture-*/command-* as command.txt, stdout, stderr and exit.

Shared infrastructure accessed by the instructed Cargo environment:

- /home/timo/.cache/sccache — explicitly authorized shared compiler cache; compiler cache writes may occur here.
- /home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock — explicitly authorized cache-server socket; server was neither stopped nor cleaned.
- /home/timo/.cargo/.global-cache — Cargo global metadata; its timestamp advanced during this concurrently active workspace session, so exact per-agent write attribution is unavailable.
- /home/timo/.cargo/.package-cache — Cargo package-cache lock; observed timestamp predates this task.
- /home/timo/.cargo/.package-cache-mutate — Cargo package-cache mutation lock; observed timestamp predates this task.

No task-owned path was created outside the worktree. TMPDIR, GOCACHE and GOTMPDIR name the assigned tree's target directory, and CARGO_TARGET_DIR was explicitly unset. The two shared cache locations named in the header are cache data and Cargo metadata, not task-owned cleanup targets.

This is the immutable pass-1 handoff. The coordinator owns recording, commits and lifecycle. Writes are relinquished after this report and its final digest are saved.

```findings
[]
```
