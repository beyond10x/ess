---
format: aep.planning-md/3
id: review-result:adversary-retrofit-w2-outcomes-pass-1
kind: review-result
status: active
title: 'Adversary pass 1: into: aggregates, precondition identity and refusal, deletion events'
relations:
- reviews: story:outcome-shapes-beyond-ess-14
revision: 1
---
Adversary pass 1 against story:outcome-shapes-beyond-ess-14 (beyond10x/ess#145, #151, #150, #144, #152), aep:adversary, 2026-09-27.

verdict: NEEDS-CHANGE
cases: executed 747→753, red 4
origin: introduced 7, pre-existing 0, undecided 0

New cases in `outcome_shapes_adversary.rs` (ess-conformance and ess-entity-runtime tests). Red: an
aggregate arranged through a creation `into:` a state fails against a correct target; a
precondition with a fixture input makes its own command create the identity twice; a refused
precondition lets every scenario pass; a deleted id re-emitting its event passes. Green: Entity
Runtime refuses `unknown_instance:` and `accepts: nothing` by name. Held: every construct refused
below ess/15; resolution order; `deletes:` then a second command; `into:` with outgoing moves;
`accepts: nothing` catches view changes and undeclared events; suite format below 22 unless used.

Coordinator routing: all seven back to the implementor; `synthesize/aggregate.rs` and
`ess-synth/src/rust/entity.rs` assigned to the unit for this round.

```findings
[{"file": "crates/verify/ess-conformance/src/synthesize/aggregate.rs", "line": 945, "category": "acceptance", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "arrange_row plans moves from the lifecycle initial while its row was created into the creator's into: state, so a synthesized aggregate scenario fails against a correct implementation"},
 {"file": "crates/verify/ess-conformance/src/synthesize.rs", "line": 5479, "category": "acceptance", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "the precondition prelude is also prepended to the precondition command's own scenario, which with a fixture input asserts a second creation of the identity the precondition already created"},
 {"file": "crates/verify/ess-conformance/src/synthesize.rs", "line": 5552, "category": "contract-drift", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "with several success candidates and a literal input that selects none, no outcome is asserted after the precondition, so a refused precondition lets every scenario pass"},
 {"file": "crates/verify/ess-conformance/src/synthesize.rs", "line": 5412, "category": "mutant", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "the deletion witness's second send asserts no absence of events, so a target that re-emits the deletion event passes"},
 {"file": "crates/verify/ess-conformance/src/go/explore.go", "line": 1454, "category": "judgement", "severity": "warning", "verdict": "CONFIRMED", "origin": "introduced", "message": "the Go and TS explorers treat a precondition decided as a refusal branch as setup success, disagreeing with synthesis"},
 {"file": "crates/generate/ess-synth/src/rust/entity.rs", "line": 203, "category": "contract-drift", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "the generated typed constructor and its doc still say an instance can only start in the initial state"},
 {"file": "docs/design/outcome-shapes.md", "line": 3, "category": "contract-drift", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "the status line misnames the refusal codes for a subjectless accepts: nothing"}]
```
