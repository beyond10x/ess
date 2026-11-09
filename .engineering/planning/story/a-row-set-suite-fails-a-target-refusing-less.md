---
format: aep.planning-md/3
id: story:a-row-set-suite-fails-a-target-refusing-less
kind: story
status: archived
title: A row-set suite beside named rows fails a target that refuses less
tags:
- ess-d368-close
- not-a-defect
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:zero-open-issues-2026-11
revision: 4
transitions:
- {from: "draft", to: "archived", at: "2026-11-09T23:06:50Z", actor: "human:timo", revision: 4}
---
## Finding

Found by wave D368 U4 (first-round implementor note), out of that story's Acceptance: the row-set suite beside named rows is checked against targets that refuse more than the rule; a target that refuses less (accepts a send the row set refuses) is not among the mutants its tests try.

## Acceptance

- A test runs the suite of a row-set-beside-named-rows model against a target that accepts a send the row set refuses, for each refusal order, and the suite fails each.
- Where a case passes, the gap is fixed or withdrawn with an `ESS-SYNTH-001` reason naming it.


## Verification on 0.92.0 (2026-11-09)

Result: **not-a-defect**. Checked against main at 4ab2ea1449 and the 0.92.0 release binary by the
zero-open-issues run (epic `epic:zero-open-issues-2026-11`); this draft leaves the plan as archived.

Proof: repro/run.txt; crates/verify/ess-conformance/tests/row_sets.rs::identity_and_row_set_guards_synthesize_without_refusal

with `source-taken` declared at each of the 5 positions, the original suite run on the interpreter of
two refuse-less mutants (row-set refusal dropped; row set narrowed to the same sink) fails
`Connect/outcome/source-taken` in all 10 cases. Each order's suite passes its own interpreter.
The draft says its tests never try such a mutant; `row_sets.rs::identity_and_row_set_guards_synthesize_without_refusal`
has tried the dropped-row-set mutant since c41646fe5c (2026-10-26), in the declared order only.
What is left is a test-only extension (loop that mutant over the orders in
`adversary_ess0700a_identity_beside_row_set_2.rs`), with no behaviour change. Close it or keep it as a
low hardening task. Body unchanged: its Acceptance is checkable but has no `## Scope`.
