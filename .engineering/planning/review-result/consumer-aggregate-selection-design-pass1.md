---
format: aep.planning-md/3
id: review-result:consumer-aggregate-selection-design-pass1
kind: review-result
status: active
title: Aggregate exactness must include retained source preconditions
relations:
- reviews: story:feature-request-361
- reviews: story:feature-request-362
revision: 1
---
needs-revision

Independent source review of `aggregate-group-selection-design.md`, SHA256 93bac2e2a70b38933cbca97bc09603e0a54c4998bd9974f58961800ef8850e81. The per-query selection design is sound in direction, but Empty provenance alone does not establish the proposed exact baseline after declared preconditions have run.

```findings
[
  {
    "file": "aggregate-group-selection-design.md",
    "line": 34,
    "category": "aggregate-baseline-authority",
    "severity": "blocker",
    "verdict": "NEEDS-CHANGE",
    "origin": "introduced",
    "message": "Exact grouped totals and row counts are computed from aggregate-planner Arranged rows, but fresh Empty provenance describes the state before scenario setup, not after it. synthesize_invocations runs aggregate::aggregates at synthesize.rs:1889 and then preconditions at1892. preconditions() later prepends each retained declared precondition, including commands that create or mutate the aggregate's source rows. A precondition creating a fixed-ID source row therefore contributes to a state-only or other unscoped group without appearing in Arranged. The proposed exact count/sum then rejects a healthy target; an extra precondition group is wrongly classified as spurious. Establish the complete query-time source-row inventory from the actual retained prefix and subsequent scenario effects, or another equally sound baseline proof, before exact assertions. Preserve precondition composition; do not ban it or clear its effects. Account for the existing duplicate-creation prefix truncation and unknown generated/fixture values without inventing totals. Expand the named synthesis scope as needed and add actual precondition-created row/group and prefix-truncation controls."
  }
]
```

## Exact source and execution evidence

Source inspected at carrier HEAD 6727e07363877925aa6a0f2bb1cb47b08d2348e3. A read-only Git diff from the candidate's 5c5aeaf795a46aacfd3709e04f630d83d8a6a837 checkpoint showed no changes in the three reviewed aggregate/synthesis files:

- `crates/verify/ess-conformance/src/synthesize/aggregate.rs`: blob f23dfa7faa4edaf7dcda744bb6755f43ed2dc846.
- `crates/verify/ess-conformance/src/synthesize.rs`: blob 50c47be7be851eaefcb64bf1ea2d012e157de7c1.
- `crates/verify/ess-conformance/src/aggregate.rs`: blob 7535c837d656029d3e9d464d55959672354efd7f.

Reviewer executions: **0 builds, tests, target executions or probes**. Source/file/hash inspection only. No production or AEP changes.

The author-provided current-source `current-probe/run.log` was read: all four sources admitted; copied-group control produced 1 aggregate / 3 total scenarios with no refusals; direct selector produced 0 aggregate / 5 total with ESS-SYNTH-017; copied selector 0 / 2 with ESS-SYNTH-017; state-only 0 / 5 with ESS-SYNTH-016; final named AGGREGATE_GROUP_SELECTION_GAP assertion failed. These are retained author executions, not reviewer reruns. The new precondition counterexample below is source-derived and still needs an actual compiled regression.

## Why the baseline finding is concrete

`aggregate.rs::observe` around 2716 executes owner/related preludes and each arranged row, but expected groups come from `groups(plan, arranged)` and aggregates only from their `Arranged` member indices. The new design retains this arrangement inventory while broadening its authority to exact whole-result counts under Empty.

After this scenario is built, `synthesize.rs::preconditions` at 9019 collects real ExecuteCommand/ExpectOutcome pairs from `ir.preconditions()` and prepends them to every scenario at 9080–9083. It does not remove their rows before aggregate queries. It retains a prefix selected by `recreates`, dropping the first precondition that would duplicate an identity created by the scenario and all later ones. Consequently, neither “no preconditions exist” nor “all preconditions always run” is a sound implicit assumption.

Minimal regression to compile: a source entity has an ordinary creator, Active lifecycle state, and a state-grouped count view. A declared precondition calls a separate permitted creator with a fixed noncolliding identity, creating one Active row. The aggregate scenario creates its usual rows under distinct identities. On an Empty-start healthy target the expected Active count must include the precondition's row. A precondition creating a reachable other-state row exposes the exact result-row count as well. This is not leakage from a previous scenario: it is setup explicitly required inside this scenario.

The revision should state where complete setup/effect authority is established and how it reaches the aggregate observer. Merely moving select_fresh_format_for earlier is insufficient. Known prefix rows may be incorporated using existing source-selected arrangement authority and exact identity bindings; a generated unknown aggregate input or unresolved fixture is a real missing-value issue requiring its existing typed treatment, not a made-up baseline. Existing legacy/scoped behavior remains independently valid when its scope proves prefix rows cannot contribute. No blanket precondition exclusion is warranted.

Required controls include: precondition row in an existing group; precondition-only extra group; a known nonmatching precondition row excluded by the full query filter; precondition updates changing the final group/aggregate contribution; and the existing duplicate-creation prefix truncation. Retain an independently faulty target that invents an additional row beyond this complete inventory so exactness still detects spurious groups.

## Other reviewed decisions

**Per-query predicate evaluation.** Separating arrangement goals from query admission addresses the actual coupling in Row.admitted, scope-field overwrites in rows(), and observe's single params map. Query results must use the full original filter on every actual reached source row for that query. A row excluded for one selector may belong to another. Residual lifecycle conditions and non-group scope conditions cannot be discarded merely because a selector is bound. The proposed Unknown-as-failed-observation rule is correct.

**Actual related identity.** Existing held/key_value distinguish internal opaque identity tokens from ScenarioValue::Instance. Reusing the captured binding for both query parameters and expected keys is necessary; emitting `instance:...` as a literal would be wrong. Current related_key explicitly refuses a related row of the view's own source entity at around 305 because the view would count it. The candidate retains unrelated existing limits rather than silently lifting that check. If that limit is later lifted, those rows also belong in the complete source-row inventory. Owner/related preludes must still precede all dependent creations to retain first/last-row mutant sensitivity.

**Optional keys.** The design correctly declines to treat absent equality as a selected absent group. In implementation, current tuples/rows add an admitted N-row for each absent-capable key; this pattern must be reconciled with selector queries explicitly. A null/missing operand does not become False just because a present parameter was drawn: shows_row ultimately uses the existing three-valued predicate evaluator. Do not preserve the old N-row blindly and then hide its Unknown as an excluded row or refuse every otherwise valid present selection. Cover selectable present cases and genuinely admitted absent-group views as the candidate requires. This is a required implementation edge of the stated design, not an additional contrary finding.

**Finite domains and matching vectors.** Actual reached tuples and typed candidate values are the right authority. Boolean, enum and one-state domains may collapse nominal tuple labels, so count distinct *actual full tuples*, not A/B/C labels. Unequal parameter tuples must be proved valid and nonmatching; a singleton cannot produce an ignored-selector witness. Multiple selectors that constrain the same field must be resolved consistently, with known contradictory parameter vectors yielding the appropriate empty result only when the actual predicate is False.

**Exact row counts.** Contains for expected groups plus exact Counts is needed to detect extra rows, duplicate aggregate groups and a target returning a previous selection. Under the corrected complete inventory, grouped zero-result queries need exact zero; ungrouped empty input retains exactly one aggregate row. Existing evaluate/evaluate_skipping_absent should remain the only aggregate arithmetic authority, including exact values above 2^53 and per-column optional skipping. Legacy scope/delta cases must not inherit Empty authority merely from a suite version.

**Provenance and delivery scope.** Reading typed Empty provenance at the planner boundary is appropriate; it must describe the actual suite being synthesized. Historical suites remain unchanged. Four-runtime independent targets and cold reset controls are required implementation evidence. A WASM runner execution does not constitute product-browser verification. The proposed two-file production scope needs revision to name the complete-prefix/observation seam before edits; the necessary behavior must not be replaced by a feature fence.

## Rereview boundary

Revise the candidate's exactness authority, complete-row inventory and precondition scope/tests, preserving the selector and unscoped-group support. A delta design review can then resolve this finding. This report does not authorize implementation or claim that aggregate support is complete.
