---
format: aep.planning-md/2
id: review-result:adversary-retrofit-w2-outcomes-pass-2
kind: review-result
status: active
title: 'Adversary pass 2: precondition prefix, constructor names'
relations:
- reviews: story:outcome-shapes-beyond-ess-14
revision: 1
---
Adversary pass 2 against story:outcome-shapes-beyond-ess-14 (beyond10x/ess#145, #151, #150, #144, #152), aep:adversary, 2026-09-27, after correction round 1.

verdict: NEEDS-CHANGE
cases: executed 922→927, red 3
origin: introduced 3, pre-existing 0, undecided 0

New cases in `outcome_shapes_adversary2.rs` (ess-conformance) and `outcome_shapes_adversary.rs`
(ess-synth). Red: a scenario recreating one precondition's identity drops every precondition, so
the ones it depends on never run; `into: Final` emits `new_r#final`; an `into:` constructor collides
with a transition method of its state. Green: an aggregate whose only creator creates into a later
state; a model without the new constructs keeps its IR keys out.

Coordinator routing: all three to the implementor in the final correction round; `feasibility.rs`
assigned to the unit.

```findings
[{"file": "crates/verify/ess-conformance/src/synthesize.rs", "line": 5535, "category": "acceptance", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "a scenario that recreates one precondition's identity drops the whole precondition list, so the preconditions it depends on never run"},
 {"file": "crates/generate/ess-synth/src/rust/entity.rs", "line": 257, "category": "boundary", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "the per-into constructor name is keyword-escaped twice, emitting new_r#final for a state named Final"},
 {"file": "crates/generate/ess-synth/src/rust/feasibility.rs", "line": 392, "category": "contract-drift", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "target admission does not inventory the new_<state> constructors, so a clash with a transition method is not refused before uncompilable Rust is emitted"}]
```
