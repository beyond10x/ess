---
format: aep.planning-md/3
id: review-result:diff-compatibility-290-20261004-r1
kind: review-result
status: active
title: 'Diff compatibility gate adversary pass 1: fail-open on untracked type uses'
relations:
- reviews: story:feature-request-290
revision: 1
---
unit: W2-1 #290 ess verify diff compatibility gate, pass 1
verdict: NEEDS-CHANGE
cases: executed 23→29, red 3
origin: introduced 2 / pre-existing 0 / undecided 0
wrote-outside-worktree: review scratch and the reviewer's own build directory (cleaned)
needs-coordinator: none

Publication copy of the only adversary pass on the #290 unit (worktree `<worktrees>/ess/ess-w2-290-diff-breaking-20261004`, base `5fc5f623c`, uncommitted diff of 11 files). The reviewer added `crates/verify/ess-diff/tests/adversary_290_pass1.rs` and edited no production file.

| case | asserts | now |
|---|---|---|
| `a_variant_removed_from_a_type_only_a_component_setting_holds_is_not_compatible` (:64) | variant removed from a type used only as a component `settings:` type is not compatible | red |
| `a_variant_removed_from_a_type_only_a_delivery_context_holds_is_not_compatible` (:102) | same for an unmapped delivery-context field type | red |
| `a_variant_removed_from_a_type_only_a_periodic_host_supplies_is_not_compatible` (:141) | same for a periodic host `context_fields` entry | red |
| `a_type_wrapped_in_optional_list_or_map_on_a_command_input_is_a_caller_input` | wrappers give uses={input}, callers breaking | green |
| `a_type_both_sent_and_returned_breaks_callers_one_way_and_readers_the_other` | narrowing breaks callers only, widening readers only | green |
| `a_narrowing_written_as_a_type_swap_on_a_command_input_is_not_compatible` | type swap not compatible | green (unknown) |

Red output: `adversary_290_pass1.rs:71` `left: Compatible right: Compatible … uses = Some({})`; same at :109 and :148. Run: `cargo test -p ess-diff --test compatibility --test adv2_diff11 --test adversary_290_pass1` → 4 + 19 passed, adversary 3 passed / 3 failed, exit 101.

F1 (blocker, introduced) `crates/verify/ess-diff/src/compatibility.rs:513`: `uses_in` sees a type only through dependency-graph edges; the graph (`ess-compiler/src/graph.rs` `walk_components`, `walk_bindings`, unchanged) has no edge for component settings or delivery-context fields, so a narrowing of a type used only there gets `uses = {}` and is compatible in all three dimensions, passing both `--fail-on` levels. Reached by any spec with `settings:` typed by a declared type (repo fixture `ess-service-contract/tests/fixtures/wiring.yaml:18-21`) or an unmapped context field. Named fix: read setting and delivery-context field types as uses, or answer unknown for a type with no recognised use.

F2 (warning, introduced) `compatibility.rs:575`: the `_ => {}` arm drops binding maps edges, so a type used only as a periodic host context field is compatible rather than unknown.

Attacked and not broken: wrappers; both-direction use; type swap (unknown); D1 default output; D5 reader refusals; acknowledgement rules; missing acknowledgements file exits 1; `ess verify impact` has no delta reader; old /2–/13 deltas (adv2_diff11 4/4).

```findings
[
  {"file": "crates/verify/ess-diff/src/compatibility.rs", "line": 513, "category": "acceptance", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "uses_in reads only graph edges, and the graph has none for component settings or delivery-context fields, so narrowing a type used only there is classified compatible in every dimension and passes both --fail-on levels instead of being unknown or breaking"},
  {"file": "crates/verify/ess-diff/src/compatibility.rs", "line": 575, "category": "acceptance", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "the catch-all arm drops binding maps edges, so a type used only as a periodic host context field is classified compatible rather than unknown when narrowed"}
]
```
