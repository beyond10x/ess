---
format: aep.planning-md/2
id: review-result:adversary-retrofit-w2-aggregates-pass-1
kind: review-result
status: active
title: 'Adversary pass 1: one absent row lacks every skipping input'
relations:
- reviews: story:aggregates-over-optional-fields
revision: 1
---
Adversary pass 1 against story:aggregates-over-optional-fields (beyond10x/ess#148), aep:adversary, 2026-09-27.

verdict: CONFIRMED (1 warning)
cases: executed 1709→1715, red 1
origin: introduced 2, pre-existing 1, undecided 0

New cases in `aggregate_optional_fields_adversary.rs` (ess-domain and ess-conformance tests). Red:
with two skipping inputs, every arranged row holds both or neither, so a target dropping rows that
lack any skipping input passes. Green: `count_distinct` over the optional key is 0 in the null group;
a parameter-scoped view asserts its null group exactly; the format gate holds with a required
declared key; `skip_absent` before the function parses. Held: SQL results per function, null group
vs real keys, mutants of `evaluate_skipping_absent`, IR bytes without `skip_absent`.

Coordinator routing: finding 1 back to the implementor; finding 2 (ungrouped DurationTotal, ESS-SYNTH-016,
pre-existing) filed as `story:ungrouped-aggregate-views-are-witnessed`; finding 3 accepted (the
coordinator applied those patches).

```findings
[{"file": "crates/verify/ess-conformance/src/synthesize/aggregate.rs", "line": 879, "category": "mutant", "severity": "warning", "verdict": "CONFIRMED", "origin": "introduced", "message": "The single absent row lacks every skipping input at once, so a target that drops rows lacking any skipping input passes every assertion for views whose aggregates are all skipping over two or more optional fields."},
 {"file": "crates/verify/ess-conformance/src/synthesize/aggregate.rs", "line": 651, "category": "acceptance", "severity": "note", "verdict": "INFEASIBLE", "origin": "pre-existing", "message": "The issue's ungrouped DurationTotal repro still synthesizes no scenario (ESS-SYNTH-016); the unit substitutes a parameter-scoped copy."},
 {"file": "crates/specify/ess-domain/src/primitive_admission.rs", "line": 395, "category": "judgement", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "primitive_admission.rs and docs/design/review-typed-diagnostics.md are edited outside the brief's file assignment and need the coordinator's acceptance."}]
```
