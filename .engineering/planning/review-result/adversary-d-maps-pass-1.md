---
format: aep.planning-md/3
id: review-result:adversary-d-maps-pass-1
kind: review-result
status: active
title: Adversary pass 1, 0.42 unit maps
relations:
- reviews: story:stored-maps-are-arranged-with-entries
revision: 1
---
unit: maps, story:stored-maps-are-arranged-with-entries (beyond10x/ess#240), working tree ess-e-maps on 300bfd3f5 plus the uncommitted diff
verdict: NEEDS-CHANGE
cases: executed 1727→1735, red 2
origin: introduced 1 / pre-existing 0 / undecided 0
wrote-outside-worktree: 5 paths (part 6)
needs-coordinator: whether single-entry arranged maps (the finding below) hold the unit or ship as a follow-up story

## 1. Diff stat

`git --no-pager diff --stat` (tracked files; all five are the implementor's, none mine):

```
 crates/verify/ess-conformance/src/input.rs         |  95 ++++++++++++++---
 .../ess-conformance/src/synthesize/subject_fact.rs | 115 ++++++++++++++++-----
 crates/verify/ess-conformance/src/witness.rs       |   4 +-
 crates/verify/ess-conformance/tests/synthesis.rs   |  57 ++++------
 website/docs/reference/predicates.md               |  17 ++-
```

Untracked: `tests/stored_map_entries.rs` (implementor) and `tests/adv1_stored_map_entries.rs` (mine, the only file I wrote in the tree). No implementation file touched.

## 2. Cases added (`crates/verify/ess-conformance/tests/adv1_stored_map_entries.rs`)

| case | asserts | now |
|---|---|---|
| `adv1_a_target_reading_only_the_first_map_value_fails_a_scenario` (:480) | for `exists`/`not exists r == input.application` over a stored `Map<String,String>`, a target evaluating only the first (or only the last) value fails ≥1 scenario of Redirect and of Launch | RED |
| `adv1_a_target_that_reads_forall_where_the_model_says_exists_fails_a_scenario` (:533) | model `exists r == input.application`; a target answering `forall` fails ≥1 scenario | RED |
| `adv1_forall_equal_is_witnessed_on_a_non_empty_map` (:557) | `forall r == input.application`: both sides synthesized, an emptiness-only target fails | green |
| `adv1_a_target_reading_the_sibling_map_fails_a_scenario` (:598) | guard over `other_uris`; a target reading same-typed `redirect_uris` fails (class 1) | green |
| `adv1_an_integer_map_value_is_grounded_on_both_sides` (:693) | `Map<String,Integer>` vs `Integer` input | green |
| `adv1_a_timestamp_map_value_is_grounded_on_both_sides` (:700) | `Map<String,Timestamp>` vs `Timestamp` input | green |
| `adv1_a_struct_map_value_is_grounded_on_both_sides` (:710) | `r.href == input.application` over `Map<String, Uri>` | green |
| `adv1_a_map_of_lists_is_grounded_on_both_sides` (:727) | nested `exists l in m: exists r in l` over `Map<String, List<String>>` | green |

Red output, run of that file alone, before the suite (`cargo test -p ess-conformance --test adv1_stored_map_entries --no-fail-fast`, log `red.log`; panic lines are pre-rustfmt, now :526 and :547):

```
thread 'adv1_a_target_that_reads_forall_where_the_model_says_exists_fails_a_scenario' (2937230) panicked at crates/verify/ess-conformance/tests/adv1_stored_map_entries.rs:534:5:
forall-for-exists passes: [
    "demo.apps.Redirect/: {\"demo.apps.Redirect/outcome/no-account\": \"passed\", \"demo.apps.Redirect/outcome/redirected\": \"passed\", \"demo.apps.Redirect/outcome/refused\": \"passed\"}",
    "demo.apps.Launch/: {\"demo.apps.Launch/outcome/launched\": \"passed\", \"demo.apps.Launch/outcome/refused\": \"passed\"}",
thread 'adv1_a_target_reading_only_the_first_map_value_fails_a_scenario' (2937228) panicked at crates/verify/ess-conformance/tests/adv1_stored_map_entries.rs:514:5:
    "not exists == / demo.apps.Redirect/: a target reading only the first value passes every scenario {...all \"passed\"}",
    "not exists == / demo.apps.Launch/: a target reading only the first value passes every scenario {\"demo.apps.Launch/outcome/launched\": \"passed\", \"demo.apps.Launch/outcome/refused\": \"passed\"}",
    "exists == / demo.apps.Launch/: a target reading only the last value passes every scenario {\"demo.apps.Launch/outcome/launched\": \"passed\", \"demo.apps.Launch/outcome/refused\": \"passed\"}",
    arranged: "demo.apps.Redirect/outcome/redirected account-138600: {\"other_uris\": Map({\"other_uris-138599\": Text(\"other_uris.0-138599\")}), \"redirect_uris\": Map({\"redirect_uris-138599\": Text(\"redirect_uris.0-138599\")})}"
test result: FAILED. 6 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
```

Every arranged map in every scenario holds exactly one entry (full list in `red.log`).

## 3. Suite run (after the cases existed)

`cargo test -p ess-conformance --no-fail-fast` (log `suite.log`): passed 1733, failed 2, ignored 3; the 2 failures are the two cases above. Final lines:

```
error: 1 target failed:
    `-p ess-conformance --test adv1_stored_map_entries`
EXIT=101
```

`<before>` 1727 = the implementor's `final-conformance.log` (1726 passed + 1 failed). `cargo xtask generate --check`: `projections are up to date`, EXIT=0. `cargo clippy -p ess-conformance --test adv1_stored_map_entries -- -D warnings`: clean; `rustfmt --check` on my file: clean.

## 4. Findings

| file:line | severity | verdict | origin | finding | fix |
|---|---|---|---|---|---|
| crates/verify/ess-conformance/src/synthesize/subject_fact.rs:840 | warning | NEEDS-CHANGE | introduced | Every arranged stored map holds one entry, so a target that reads only the first (or last) value, or answers `forall` for `exists`, passes every scenario of the #240 shape: brief class 3, "only the first witnessed". | Arrange the holding side of an `exists` (and the failing side of a `forall`) as a map of ≥2 entries whose deciding value is not at the first key, plus one where it is not at the last; the `forall` holding side with ≥2 entries all satisfying. |

What reaches it: every model with a quantifier over a stored map compared with an `input.` field (the issue's own shape, `redirect_uris`); the red case uses the issue's guard verbatim. A real target bug of that shape: `for _, v := range m { return v == app }` in Go. The story's acceptance ("a target that ignores the map entries fails") is met; this is below it. The same single-entry arrangement likely applies to stored `List` guards the unit also newly witnesses. That is not tested here.

## 5. Attacked and not broken

- Key/ordinal collision: `input.rs` publishes no key fact, so a key `"0"` or `"count"` cannot collide in the flattener. The runner's `row_facts` (`runner.rs:3207`) binds view-row maps **by key**, a different convention. No synthesized scenario reads a quantifier through `row_facts`, and the new scenarios decide on the branch taken. That is pre-existing and not reached here.
- Byte stability: `generate --check` clean. Committed map models (billing, gatepass, recorded-log-adapter) have no quantifier over a map.
- Key order: Entity Runtime `serde_json::Map` has no `preserve_order` in `entity-runtime/Cargo.lock`, so it is a BTreeMap, the same order as `Node::Map`. Quantifiers are order-free anyway.
- Integer, Timestamp, struct and map-of-list values: all grounded on both sides, and the model's target passes.
- Class 1: sibling same-typed map values differ (`other_uris.0-N` vs `redirect_uris.0-N`); the wrong-map mutant fails.
- `forall ==`: witnessed on a non-empty map; the emptiness mutant fails.
- Class 2: no silent drop was observed in the eight shapes; every shape synthesized with 0 refusals.
- Noted, not a finding: the rewritten `synthesis.rs` test removes the only synthesis-level ESS-SYNTH-002 control. This was a coordinator decision (decisions.md:101).

## 6. Paths written outside the worktree

- ~/.cache/ess-wave-n2/maps/adv1/review.md
- ~/.cache/ess-wave-n2/maps/adv1/red.log
- ~/.cache/ess-wave-n2/maps/adv1/suite.log
- ~/.cache/ess-wave-n2/maps/adv1/generate.log
- ~/.cache/b10x-target/ess-e-maps (the assigned build dir, reused)

## 7. Findings block

```findings
- file: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
  line: 840
  category: boundary
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: every arranged stored map holds exactly one entry, so first-value-only and forall-for-exists targets pass every scenario of the #240 shape
```
