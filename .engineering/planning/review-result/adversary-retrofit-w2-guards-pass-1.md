---
format: aep.planning-md/2
id: review-result:adversary-retrofit-w2-guards-pass-1
kind: review-result
status: active
title: 'Adversary pass 1: bare input root, input text length lowering, Unicode fold witnesses, view filters'
relations:
- reviews: story:subject-guard-input-and-case-folding
revision: 1
---
Adversary pass 1 against story:subject-guard-input-and-case-folding (beyond10x/ess#157, #140), aep:adversary, 2026-09-27.

verdict: CONFIRMED (3 warnings, 1 note)
cases: executed 1405→1413, red 4
origin: introduced 3, pre-existing 0, undecided 1

New cases in `adversary_guards_positions.rs` (ess-domain), `adversary_guards_lowering.rs`
(ess-entity-runtime), `adversary_guards.rs` (ess-conformance). Red: a bare `input` root below ess/15
is told to upgrade; `input.note.count` in a stored-field predicate is not refused by name in Entity
Runtime lowering; fold witnesses never separate ASCII from Unicode folding; a fold-filtered view is
asserted only over a non-matching row. Green: every predicate position gated at ess/15; folds under
`when:` refused by name in lowering; E6 witnessed both ways for six operand types; a stored-field
fold witnessed on a case-changed row. Held: Rust/Go/TS fold parity on é, ß, Kelvin sign, İ, ı,
fullwidth letters; suites without folds keep their format number.

Coordinator routing: findings 1-3 back to the implementor; finding 4 decided pre-existing
(an equality filter behaves the same) and filed as `story:view-filters-witnessed-on-matching-rows`,
its test ignored with that reference.

```findings
[{"file": "crates/specify/ess-domain/src/command/subject_fact.rs", "line": 268, "category": "boundary", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "A bare input root in a when_subject predicate below ess/15 is told to declare ess/15, where the same document is still refused as unobservable_fact."},
 {"file": "crates/generate/ess-entity-runtime/src/lib.rs", "line": 535, "category": "contract-drift", "severity": "warning", "verdict": "CONFIRMED", "origin": "introduced", "message": "An input text length read through input. in a stored-field predicate is never refused by name as TextLengthUnsupported, only rejected by entity-core."},
 {"file": "crates/verify/ess-conformance/src/witness.rs", "line": 732, "category": "acceptance", "severity": "warning", "verdict": "CONFIRMED", "origin": "introduced", "message": "Fold witnesses only swap ASCII case, so a target using Unicode folding passes every suite."},
 {"file": "crates/verify/ess-conformance/tests/adversary_guards.rs", "line": 189, "category": "acceptance", "severity": "warning", "verdict": "CONFIRMED", "origin": "undecided", "message": "A view filtered by a fold is asserted only over a non-matching row, so a byte-wise filter passes; an equality-filter control behaves the same."}]
```
