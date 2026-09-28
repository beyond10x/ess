---
format: aep.planning-md/3
id: review-result:adversary-a-preconditions-pass-1
kind: review-result
status: active
title: Adversary pass 1, 0.41 unit preconditions
relations:
- reviews: story:precondition-inputs-take-structured-literals
revision: 1
---
unit: preconditions (#205)
verdict: red
cases: executed 2391→2405, red 9
origin: introduced 5, pre-existing 1, undecided 0
wrote-outside-worktree: ~/.cache/ess-wave-n2/preconditions/adv1/
needs-coordinator: no

Adversary pass 1 (`aep:adversary`), 2026-09-28, head bfcded1a5 + `tests/adversary_precondition_literals.rs` (ess-domain), `tests/adversary_explore_preconditions.rs` and fixture (ess-conformance). Explorers held (sendable never leaks; Go/TS byte-equal; exact 2^53+1; Json/Binary64 nested stay refused).

```findings
[{"file":"crates/specify/ess-domain/src/command.rs","line":4011,"category":"boundary","severity":"blocker","verdict":"NEEDS-CHANGE","origin":"introduced","message":"a precondition literal whose type is a newtype over a list, map or struct is never held to that newtype invariants"},{"file":"crates/specify/ess-domain/src/command.rs","line":4123,"category":"contract-drift","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"struct invariants over a nested field or a list count come out unknown and are admitted, while the setup reader requires them true"},{"file":"crates/specify/ess-domain/src/command.rs","line":4106,"category":"boundary","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"a struct field typed by a newtype over Optional must be written, although null is admitted for it"},{"file":"crates/specify/ess-domain/src/command/outcome_shapes.rs","line":847,"category":"boundary","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"a one-entry map written {fixture: text} is read as a fixture reference and refused"},{"file":"crates/specify/ess-domain/src/command/outcome_shapes.rs","line":669,"category":"acceptance","severity":"warning","verdict":"CONFIRMED","origin":"pre-existing","message":"a precondition literal is never checked against the invariants of the entity its branch creates"}]
```

Coordinator routing: all five into correction 1 (F5 pre-existing but reached more widely by this unit).
