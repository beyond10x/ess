---
format: aep.planning-md/1
id: review-result:browser-replay-source-pass2
kind: review-result
status: active
title: Browser replay fidelity source pass 2
relations:
- reviews: story:review-browser-replay-fidelity
revision: 1
---
unit: story:review-browser-replay-fidelity — source pass 2, correction-1 working tree at d7f12b7a027018fc020fadf8f1d66ca6caac1e68
verdict: nothing found
cases: executed 31→34, red 0
origin: introduced 0 / pre-existing 0 / undecided 0
wrote-outside-worktree: 9043 inventoried paths under /home/timo/.cache/ess-w15-browser-adversary-2-tmp
needs-coordinator: none; root retains AEP, Git, integration and delivery gates

`git --no-pager diff --no-index --stat /home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/review-boundaries-15/preparation/browser-source-pass-2/handed-source/crates/edge/ess-cli/tests/replay_fidelity_browser.rs /home/timo/.local/state/worktree/trees/b10x/ess/ess-browser-replay-fidelity/crates/edge/ess-cli/tests/replay_fidelity_browser.rs`

```text
 .../edge/ess-cli/tests/replay_fidelity_browser.rs  | 157 +++++++++++++++++++++
 1 file changed, 157 insertions(+)
```

This is the test-only delta against the exact handed-source snapshot, as the filled brief requires.
The complete existing 46,352-byte test prefix is unchanged; only 157 lines are appended. All nine
other source pins are unchanged, including coverage_browser.rs, both corrected players, binding,
lock and historical player. The unit already carried implementation changes and an untracked test
file when dispatched; inherited-working-tree-stat.txt and the retained initial-status.txt distinguish
those inherited changes from this pass. No production, prior assertion, planning, Git, network,
installation or cleanup mutation was made. test-only.patch SHA256: 092699d467d068f73c4d148a35b1a15aff051c3a1caa272075ab40e7d1dd0b1b.

1. Cases written before execution

All three additions were written before the first producer. cases-written-before-producer.json
retains the timestamp and initial source hash. They use actual CLI-emitted pages, the unchanged
required Firefox/BiDi harness, admitted persisted data where stated, API snapshots and DOM checks.

| New case in crates/edge/ess-cli/tests/replay_fidelity_browser.rs | Contract and reachable route | Result |
|---|---|---|
| :1154 adversary2_exact_metadata_and_literal_lookalikes_keep_distinct_kinds | B04/B11/B13: suite/5 emits the assets first, then the real Rust suite and paired replay readers admit the exact fixture. u32 elapsed and u64 count/halt/position maxima stay exact decimal tokens; explicit at_least:null and nested literal metadata lookalikes keep their kinds; no computed rows or observed events appear. | Green, 1 focused case, direct 0 |
| :1216 adversary2_reused_creation_alias_is_diagnostic_and_preserves_prior_prefix | B01/B10/B12: both emitted routes admit a persisted second creation capture reusing the prior alias. The declaration stays unchanged, a diagnostic and unbound effect appear, and Back/replay reproduce both API and DOM. | Green, 1 focused case covering both routes, direct 0 |
| :1251 adversary2_switching_to_a_distinct_authored_scenario_cancels_pending_play | B12: real CLI emits two authored scenario files on each route. A DOM scenario selection during pending playback, then slower restart, must match the independently stepped alternate prefix and contain only its aliases. | Green, 1 focused case covering both routes, direct 0 |

The first metadata run stopped during fixture admission because elapsed is u32 and the initial new
fixture supplied u64::MAX. It never reached Firefox. This is a fixture-construction failure, not a
product finding or product red. The new fixture was corrected to u32::MAX and the actual Ranking
string encoding before its focused rerun; no inherited assertion was altered. Original source,
command, stdout/stderr and direct 101 are retained below. No product assertion-red output exists.

`cargo test -p ess-cli --locked --offline --test replay_fidelity_browser adversary2_exact_metadata_and_literal_lookalikes_keep_distinct_kinds -- --exact --nocapture --test-threads=1`

Receipt: /home/timo/.local/state/worktree/trees/b10x/ess/ess-browser-replay-fidelity/target/review-boundaries-15/adversary-pass-2/implementation/focused-metadata

stdout (verbatim):
```text

running 1 test
test adversary2_exact_metadata_and_literal_lookalikes_keep_distinct_kinds ... FAILED

failures:

failures:
    adversary2_exact_metadata_and_literal_lookalikes_keep_distinct_kinds

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 29 filtered out; finished in 0.03s

```

stderr (verbatim):
```text
   Compiling ess-cli v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-browser-replay-fidelity/crates/edge/ess-cli)
    Finished `test` profile [unoptimized] target(s) in 0.56s
     Running tests/replay_fidelity_browser.rs (target/debug/deps/replay_fidelity_browser-3b05a0461d802984)

thread 'adversary2_exact_metadata_and_literal_lookalikes_keep_distinct_kinds' (4015256) panicked at crates/edge/ess-cli/tests/replay_fidelity_browser.rs:258:88:
called `Result::unwrap()` on an `Err` value: AdmissionError { issues: [AdmissionIssue { reason: "InvalidSuite", path: "$suite", detail: "invalid value: integer `18446744073709551615`, expected u32 at line 1 column 5003" }] }
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
error: test failed, to rerun pass `-p ess-cli --test replay_fidelity_browser`
```

Direct exit 101; duration 1.000878987 seconds; resource stop None.

`cargo test -p ess-cli --locked --offline --test replay_fidelity_browser adversary2_exact_metadata_and_literal_lookalikes_keep_distinct_kinds -- --exact --nocapture --test-threads=1`

Receipt: /home/timo/.local/state/worktree/trees/b10x/ess/ess-browser-replay-fidelity/target/review-boundaries-15/adversary-pass-2/implementation/focused-metadata-admitted

stdout (verbatim):
```text

running 1 test
test adversary2_exact_metadata_and_literal_lookalikes_keep_distinct_kinds ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 29 filtered out; finished in 1.67s

```

stderr (verbatim):
```text
   Compiling ess-cli v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-browser-replay-fidelity/crates/edge/ess-cli)
    Finished `test` profile [unoptimized] target(s) in 0.54s
     Running tests/replay_fidelity_browser.rs (target/debug/deps/replay_fidelity_browser-3b05a0461d802984)
```

Direct exit 0; duration 2.501331111 seconds; resource stop None.

`cargo test -p ess-cli --locked --offline --test replay_fidelity_browser adversary2_reused_creation_alias_is_diagnostic_and_preserves_prior_prefix -- --exact --nocapture --test-threads=1`

Receipt: /home/timo/.local/state/worktree/trees/b10x/ess/ess-browser-replay-fidelity/target/review-boundaries-15/adversary-pass-2/implementation/focused-reused-alias

stdout (verbatim):
```text

running 1 test
test adversary2_reused_creation_alias_is_diagnostic_and_preserves_prior_prefix ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 29 filtered out; finished in 3.36s

```

stderr (verbatim):
```text
    Finished `test` profile [unoptimized] target(s) in 0.07s
     Running tests/replay_fidelity_browser.rs (target/debug/deps/replay_fidelity_browser-3b05a0461d802984)
```

Direct exit 0; duration 3.501355733 seconds; resource stop None.

`cargo test -p ess-cli --locked --offline --test replay_fidelity_browser adversary2_switching_to_a_distinct_authored_scenario_cancels_pending_play -- --exact --nocapture --test-threads=1`

Receipt: /home/timo/.local/state/worktree/trees/b10x/ess/ess-browser-replay-fidelity/target/review-boundaries-15/adversary-pass-2/implementation/focused-scenario-select

stdout (verbatim):
```text

running 1 test
test adversary2_switching_to_a_distinct_authored_scenario_cancels_pending_play ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 29 filtered out; finished in 3.78s

```

stderr (verbatim):
```text
    Finished `test` profile [unoptimized] target(s) in 0.07s
     Running tests/replay_fidelity_browser.rs (target/debug/deps/replay_fidelity_browser-3b05a0461d802984)
```

Direct exit 0; duration 4.001264816 seconds; resource stop None.

Only the appended additions were formatted using frozen rustfmt on an assigned scratch copy. The
formatted file was required to retain the exact handed prefix before copying it back. The focused
cases preceded this formatting and the combined suite; formatting changed no assertion meaning.

2. Combined browser suite and package checks

The baseline 31 is the correction-1 report's executed count, not a suite run before the additions.
The combined runner executed 4 coverage cases and 30 replay cases: 34 total, 34 passed, 0 failed,
0 ignored, 0 filtered. The full B01–B15 and historical fixture families remain selected. The
287-case conformance baseline is inherited from correction 1 and was not rerun in this review;
this pass does not claim the combined 321 package total as its own execution.

`rustfmt --edition 2021 --config skip_children=true target/review-boundaries-15/adversary-pass-2/replay_fidelity_format.rs`

Receipt: /home/timo/.local/state/worktree/trees/b10x/ess/ess-browser-replay-fidelity/target/review-boundaries-15/adversary-pass-2/implementation/format-additions

stdout (verbatim):
```text
```

stderr (verbatim):
```text
```

Direct exit 0; duration 0.500629787 seconds; resource stop None.

`cargo test -p ess-cli --locked --offline --test replay_fidelity_browser --test coverage_browser -- --nocapture --test-threads=1`

Receipt: /home/timo/.local/state/worktree/trees/b10x/ess/ess-browser-replay-fidelity/target/review-boundaries-15/adversary-pass-2/implementation/combined-browser

stdout (verbatim):
```text

running 4 tests
test actual_browser_admits_the_pair_before_creating_replay_state ... ok
test actual_browser_and_rust_refuse_every_closed_model_field_boundary ... actual closed model boundary vectors: 314
ok
test actual_browser_checks_full_lineage_and_integer_metadata ... ok
test retained_legacy_player_bytes_still_replay_in_actual_firefox ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 10.80s


running 30 tests
test adversary2_exact_metadata_and_literal_lookalikes_keep_distinct_kinds ... ok
test adversary2_reused_creation_alias_is_diagnostic_and_preserves_prior_prefix ... ok
test adversary2_switching_to_a_distinct_authored_scenario_cancels_pending_play ... ok
test adversary_explicit_null_count_mounts_suite4 ... ok
test adversary_explicit_null_count_mounts_suite5 ... ok
test adversary_lower_only_count_mounts_suite4 ... ok
test adversary_lower_only_count_mounts_suite5 ... ok
test adversary_query_only_prefix_remains_visible_and_reconstructible ... ok
test adversary_upper_only_count_mounts_suite4 ... ok
test adversary_upper_only_count_mounts_suite5 ... ok
test b01_capture_establishes_only_local_alias_and_initial_state ... ok
test b01_capture_event_field_need_not_equal_entity_identity_field ... ok
test b01_conflicting_captures_outcomes_and_missing_assignment_input_are_diagnostic ... ok
test b02_move_processes_every_set_without_guessing_subject ... ok
test b02_unknown_writes_invalidate_existing_values_and_preserve_unrelated_facts ... ok
test b03_missing_literals_never_copy_same_named_decoys ... ok
test b04_typed_literals_are_retained_and_literal_mapping_is_no_reference ... ok
test b05_different_conversion_models_have_same_unknown_reduced_assignment ... ok
test b06_swapped_ordered_alias_vectors_never_select_a_subject ... ok
test b06_update_with_single_reference_keeps_state_but_cannot_choose_subject ... ok
test b07_both_model_rank_directions_have_no_computed_rows ... ok
test b08_missing_required_authored_parameter_still_refuses ... ok
test b08_supplied_and_unresolved_query_parameters_remain_unknown ... ok
test b09_filters_are_visible_without_partial_evaluation ... ok
test b10_unbound_effects_and_zero_candidates_are_unknown ... ok
test b11_binding_and_external_controls_never_create_observations_or_instances ... ok
test b11_original_controls_and_expectations_are_unexecuted ... ok
test b11_refusal_preserves_established_declarations_and_unknown_facts ... ok
test b12_controls_reconstruct_prefixes_and_cancel_stale_playback ... ok
test b12_reset_select_pause_and_restart_cancel_callbacks_from_previous_play ... ok

test result: ok. 30 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 141.66s

```

stderr (verbatim):
```text
   Compiling ess-cli v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-browser-replay-fidelity/crates/edge/ess-cli)
    Finished `test` profile [unoptimized] target(s) in 0.53s
     Running tests/coverage_browser.rs (target/debug/deps/coverage_browser-2a9fe7f1faee6112)
     Running tests/replay_fidelity_browser.rs (target/debug/deps/replay_fidelity_browser-3b05a0461d802984)
```

Direct exit 0; duration 153.187460535 seconds; resource stop None.

`cargo fmt --package ess-conformance --package ess-cli --check`

Receipt: /home/timo/.local/state/worktree/trees/b10x/ess/ess-browser-replay-fidelity/target/review-boundaries-15/adversary-pass-2/implementation/package-fmt

stdout (verbatim):
```text
```

stderr (verbatim):
```text
```

Direct exit 0; duration 0.500603235 seconds; resource stop None.

`cargo clippy -p ess-conformance -p ess-cli --all-targets --locked --offline -- -D warnings`

Receipt: /home/timo/.local/state/worktree/trees/b10x/ess/ess-browser-replay-fidelity/target/review-boundaries-15/adversary-pass-2/implementation/package-clippy

stdout (verbatim):
```text
```

stderr (verbatim):
```text
    Checking ess-cli v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-browser-replay-fidelity/crates/edge/ess-cli)
    Finished `dev` profile [unoptimized] target(s) in 0.34s
```

Direct exit 0; duration 0.500636399 seconds; resource stop None.

3. Findings

Nothing found. No judgement finding, approval or verifier-independence claim is returned. The first
source pass's interrupted report and its admitted null-count defect remain prior history; this pass
reviewed the corrected source and the combined runner executed both retained null regressions.

4. Boundaries exercised without a product failure

- Exact coverage metadata, opposite optional-null bound, and literal metadata lookalikes remain distinct.
- Reused aliases diagnose an unbound creation without overwriting the preceding established declaration.
- Actual selection of a different authored scenario cancels the prior playback generation.
- Inherited B01–B12 cases cover typed values, sets with moves, conversions, unknown subjects/views,
  refusal policy, renamed capture fields, unexecuted controls, and prefix reconstruction on both players.
- Inherited B13–B15 cases retain closed-model refusal before state/banner, pairing and UTF-8 failures,
  full lineage/metadata controls, 314 closed-model vectors, and unchanged historical player bytes.

5. Exact source, tool, command and output references

| Source | Final SHA256 |
|---|---|
| crates/verify/ess-conformance/src/web.rs | 04e1785fe9f3b80863bc16d1d7ba08c2b61756580b6af52684e547f695a28067 |
| crates/verify/ess-conformance/assets/player.js | dfb42c7fbc04e579c14753dc8a463d0cc781446380442102be2ab85dcb7385d8 |
| crates/verify/ess-conformance/assets/coverage-player.js | 5254eedc36c3bb5075da0ce5dedde98e52274bdc7ec1d241acd50a50f249b29c |
| crates/verify/ess-conformance/assets/index.html | 570a6eeb4918e0d64b9e6ea037e41bff068b811adac3a76100d928b92c3b77ed |
| crates/edge/ess-cli/tests/coverage_browser.rs | 9664a12313f748411b8fbc5fdd3123d2199b9eaeee272287030b7a1e2171b9d6 |
| crates/edge/ess-cli/src/main.rs | 442b74f09b8f962a06fd9ea988cc0a4b23fda8c964efdf9fc03e1d12da5a1f49 |
| docs/design/review-replay-subset.md | 59add7f328f5a2c41bcd996b0280d626e1ea15d93748f3ed5210f03b1326b5a3 |
| crates/verify/ess-conformance/tests/fixtures/coverage/legacy-player.js | 990575f6db31d13fecd7d06df51f3d34190cc8df33fa1a320efe682900d2e23f |
| Cargo.lock | 3f5b5583e5c4b164ba782062ebade3d0f6d6fa9da8ced2f46d54d7922e6438c9 |
| crates/edge/ess-cli/tests/replay_fidelity_browser.rs | 0d306af84f12de6edd9132c4d365f08d242715260849610ec85d6ce0fc9d4d88 |

| Producer | stdout SHA256 | stderr SHA256 |
|---|---|---|
| focused-metadata | 85bbbd24ba5b2d61d1e26ab11d6175b60c407537a2c9406c45ada19c5ee0ae94 | 94b0a262ddf3673ff1cc3684eca594f96f89a45bf34275fbf7e85a810af55988 |
| focused-metadata-admitted | 94a2a020526c8cc23d1cbd04e5a97e683905a588d6d6f652fa1a05d0edc36267 | 92f10747a0733ea97b31e61aea3932c28b86a56bbaef372131e7e1339ab06662 |
| focused-reused-alias | 3afef51739192cc1cacbb49e1fb95c70bd170162c8420b54283826586bc3111c | 26fcc52cc9cc7e72a38054f4b355b60a91f6cfe0e2df786f2ee45b18f1a27829 |
| focused-scenario-select | 6616571df659b33210cc9ff5fc85b1beb4c0d07cdf36d6e99d55c574ebfea7a2 | 26fcc52cc9cc7e72a38054f4b355b60a91f6cfe0e2df786f2ee45b18f1a27829 |
| format-additions | e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855 | e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855 |
| combined-browser | ef40af932232cb03f68cb6ac22d55857bcfe06cbecab715dab2c73ecde95b777 | 53aaaf634046e59d80e2f766fda5fb978260565fa268a93c79a67b0adb7b976d |
| package-fmt | e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855 | e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855 |
| package-clippy | e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855 | 7a37a71cd159e9b9b3afacf1aaed653dc2f7f6e58751a9a7dd84486e9bd59d14 |

Every producer used the exact launch-environment.json set/unset values: the frozen Rust snapshot,
private CARGO_HOME, unit default target, assigned TMP/XDG roots, four empty wrappers, offline mode,
two jobs and disabled incremental/debug output. No target-directory override was supplied. The
full Rust 378-entry payload was checked before and after every producer; the full Firefox 50-entry
payload and wrapper were checked before and after each browser-designated producer, including the
fixture-refused first attempt. Manifest hashes are 387c58849c86996b2aa20d63de19d6e4d925c91552682ed5eb49a1b0a1ce731d
(Rust) and eaac01c1bafaf70d1392d46914f0fd69ff247f7bd9ad0895334add758543bd9f (Firefox).

All eight direct receipts, exact commands, original logs, durations, counts, environment and before/
after resource/tool checks are bound by producer-receipts.json SHA256 e90ff154939bc5102340c3c9e994b8d073e5a056ce6daef0e3700e0b27b2501b.
Resource monitoring counted the unit target and all four assigned external TMP roots, preserving
the 16 GiB combined ceiling and 8 GiB free floor. No resource refusal or termination occurred.
Producer PIDs, Firefox main PIDs and browser-harness PIDs are recorded; all 91 are absent. All eight
producer tool sessions completed, including the retained fixture-only direct 101. Remaining owned
processes/sessions: none. The handoff is quiescent.

6. Written paths and retained outputs

Every external write in this pass is under /home/timo/.cache/ess-w15-browser-adversary-2-tmp.
The full 9043-path absolute list is outside-worktree-paths.txt, SHA256 e77a833fa6f13e2df5b5e9d312b24e59efa027213513d2b0fef8704a726e48c3.
It includes fixture inputs, emitted assets, CLI receipts, Firefox profiles and BiDi records.
No other external writable root was used; inherited HOME was preserved. New scratch/XDG/log writes
are confined to /home/timo/.local/state/worktree/trees/b10x/ess/ess-browser-replay-fidelity/target/review-boundaries-15/adversary-pass-2. Compilation reused target/debug and the private unit Cargo home. Exact executed
binary hashes are retained separately; historical build trees and old TMP evidence were not recounted
or cleaned. Root retains the full mutable build roots and owns subsequent lifecycle decisions.

The finite new scratch/TMP inventory contains 9147 native entries: 2860 directories,
6209 regular files and 78 literal symlinks, 1991223639 regular payload bytes.
It preserves native paths, modes, ownership, inode/link metadata, timestamps, xattrs, payload hashes,
empty/hidden entries and literal link targets without traversing links. Unexpected special entries:
none. Only the self-referential handoff control subtree is excluded and is separately sealed.
new-output-inventory.jsonl SHA256: e6dc07865af56081ee4e49728af15ee791d25d2eea4c7e54918509bd14eada7c.
all-source-hashes.json SHA256: 10fbf3b267045abb159ec167858a00320a4c461ebc0387bf482c97ec8cd4d664.
quiescence.json SHA256: 2f0f75e8680d1410553fb38767b16e9dee05a5d54297d72f8f046d6ea04c5609.
The report hash and complete control/reference seal are returned outside this report to avoid a
self-reference. All raw prior and new records remain retained; nothing was deleted.

```findings
[]
```
