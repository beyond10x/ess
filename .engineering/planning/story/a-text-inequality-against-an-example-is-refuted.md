---
format: aep.planning-md/3
id: story:a-text-inequality-against-an-example-is-refuted
kind: story
status: implemented
title: A text inequality against an example value gets its refuting case
summary: Synthesis tries only guard literals on the text ladder, so a != guard against the example value has no refuting candidate
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-10-31T19:59:07Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-10-31T19:59:08Z", actor: "human:timo", revision: 4}
- {from: "active", to: "implemented", at: "2026-11-01T01:28:09Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}}
---
## Problem

Synthesis cannot write the case that refutes a `!=` guard against an example's own text value. The
text ladder tries only the guard's literals (`crates/verify/ess-conformance/src/witness.rs`, the
text ladder near 5510-5525), so where the example already holds the compared value, no candidate
selects the other side. Scalar examples have the same limit. Found by the D325 U1 implementor while
writing `ess-conformance/tests/structured_input_example.rs`, which uses a `==` guard for that
reason.

## Acceptance

- A command whose input carries an `example:` and whose guard is `<path> != <the example's value>`
  synthesizes both branches: one sending the example, one sending a value the guard rejects.
- The same holds for a scalar input with an `example:`.
- Suites of models without such a guard keep their bytes.

## Decisions

**accept** (wave D335, 2026-10-31, the coordinator's call inside `approval-record:ess-d335-next-adopter-gaps`).

- Need: a `!=` guard against the value an `example:` already holds has no refuting candidate, so the
  refusal or the other branch is never sent; found by the D325 implementor, who wrote a `==` guard in
  `structured_input_example.rs` to avoid it.
- Fit: a defect in synthesis's candidate ladder, not a language change; no new format. The refuting
  candidate is the guard's own literal (equal, so `!=` is false), which the ladder must offer even
  when the example already supplies the plain instance.
- Suites of models without such a guard keep their bytes (`synthesis-suite-bytes-table.tsv`).
