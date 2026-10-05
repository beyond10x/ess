---
format: aep.planning-md/3
id: review-result:related-values-285-20261004-r1
kind: review-result
status: active
title: 'Related values adversary pass 1: shared first reference, absent event value, middle updates'
relations:
- reviews: story:feature-request-285
revision: 1
---
unit: W3-12 #285 related values through an Optional reference or across two references, pass 1
verdict: NEEDS-CHANGE
cases: executed 33→44, red 8
origin: introduced 7 / pre-existing 0 / undecided 0
wrote-outside-worktree: review scratch (probe, base digests, logs); two build dirs cleaned
needs-coordinator: none

Publication copy of the only adversary pass on the #285 unit (worktree `<worktrees>/ess/ess-w3-285-related-values-20261004`, base `26ad39057`, uncommitted diff of 15 files). The reviewer added `crates/verify/ess-conformance/tests/adversary_related_values_chained.rs`, `adversary_related_values_all_bytes.rs` and `fixtures/adversary-285-base-digests.tsv`; no production file edited. IR, suite and refusal digests of 88 models plus billing, gatepass and oracle-fixture are identical to base.

F1 (blocker) `crates/verify/ess-conformance/src/synthesize/related.rs:908`: two reads sharing a first reference along different chains leave one chain unarranged on a row whose next reference names no row, so the native interpreter cannot pass the synthesized suite (`aggregate.rs` refuses the same situation by name).

F2 (warning) `synthesize.rs:5755`: the absent run never asserts the event's copied value, so a wrong event value passes the reduction and an event-only copy's absent witness asserts nothing (contradicting `synthesize.rs:2282`).

F3 (warning) `related.rs:916`: updates are rerun on the last row only, so a target keeping the middle reference as first set passes, despite `website/docs/guides/specify/values-and-views.md:122`.

F4 (warning) `related.rs:286`: with both references Optional only the first absence is witnessed; without an aggregate view a target ignoring the second passes every scenario.

F5 (warning) `synthesize.rs:6728`: an updates branch reading through a stored Optional subject reference is refused ESS-SYNTH-001 "has no scenario" while its scenario exists, with an inapplicable hint and no absent witness.

F6 (note) `schemas/generated/ess.schema.json:2802`: `RawRelatedVia` admits an array of any length while the reader admits exactly two.

Attacked and not broken: targets reading the latest or first middle row or caching the first read fail; self-referencing chain, rewritten middle row with the reference on its creating event, and a copied identity grouped by the aggregate all pass the interpreter.

```findings
[
 {"file":"crates/verify/ess-conformance/src/synthesize/related.rs","line":908,"category":"acceptance","severity":"blocker","verdict":"NEEDS-CHANGE","origin":"introduced","message":"two reads sharing a first reference along different chains leave one chain unarranged on a row whose next reference names no row, so the native interpreter cannot pass the synthesized suite"},
 {"file":"crates/verify/ess-conformance/src/synthesize.rs","line":5755,"category":"acceptance","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"the absent run never asserts the event's copied value, so a wrong event value passes the reduction and an event-only copy's absent witness asserts nothing"},
 {"file":"crates/verify/ess-conformance/src/synthesize/related.rs","line":916,"category":"contract-drift","severity":"warning","verdict":"CONFIRMED","origin":"introduced","message":"updates are rerun on the last row only, so a target keeping the middle reference as first set passes despite the docs' promise"},
 {"file":"crates/verify/ess-conformance/src/synthesize/related.rs","line":286,"category":"boundary","severity":"warning","verdict":"CONFIRMED","origin":"introduced","message":"with both references Optional only the first absence is witnessed, and without an aggregate view a target ignoring the second passes every scenario"},
 {"file":"crates/verify/ess-conformance/src/synthesize.rs","line":6728,"category":"acceptance","severity":"warning","verdict":"CONFIRMED","origin":"introduced","message":"an updates branch reading through a stored Optional subject reference is refused as having no scenario while its scenario exists, with an inapplicable hint and no absent witness"},
 {"file":"schemas/generated/ess.schema.json","line":2802,"category":"judgement","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"RawRelatedVia lets the array have any length while the reader admits exactly two"}
]
```
