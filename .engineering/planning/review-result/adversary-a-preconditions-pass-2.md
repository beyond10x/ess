---
format: aep.planning-md/3
id: review-result:adversary-a-preconditions-pass-2
kind: review-result
status: active
title: Adversary pass 2, 0.41 unit preconditions
relations:
- reviews: story:precondition-inputs-take-structured-literals
revision: 1
---
unit: preconditions (#205)
verdict: red
cases: executed 2408→2419, red 10
origin: introduced 4, pre-existing 1, undecided 0
wrote-outside-worktree: ~/.cache/ess-wave-n2/preconditions/adv2/
needs-coordinator: yes

Adversary pass 2 (`aep:adversary`), 2026-09-28, head 3b268da5c + `ess-domain/tests/adversary_precondition_literals_pass2.rs` (7), `ess-conformance/tests/adversary_explore_preconditions_pass2.rs` (4). Pass-1 F1–F4 fixed; F5 partly. `ess specify validate` exit 0 on every real spec.

```findings
[{"file":"crates/specify/ess-domain/src/command/outcome_shapes.rs","line":885,"category":"acceptance","severity":"blocker","verdict":"NEEDS-CHANGE","origin":"introduced","message":"precondition_branch binds no fact for a list, map or struct literal, so a guard over one is falsely refused and defined() over one selects a branch the interpreter does not take"},{"file":"crates/specify/ess-domain/src/command/outcome_shapes.rs","line":823,"category":"contract-drift","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"an omitted input typed by a newtype over Optional is admitted while the setup reader bind refuses it as nothing supplied"},{"file":"crates/specify/ess-domain/src/command.rs","line":4138,"category":"contract-drift","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"a struct literal leaving out a member typed by a newtype over Optional is admitted while the setup reader struct walk refuses it"},{"file":"crates/specify/ess-domain/src/command/outcome_shapes.rs","line":720,"category":"acceptance","severity":"warning","verdict":"NEEDS-CHANGE","origin":"pre-existing","message":"precondition_row checks only input.<field> sources, so a literal, fallback or struct-sourced field that breaks an entity invariant is admitted and fails at run time"},{"file":"docs/design/outcome-shapes.md","category":"contract-drift","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"the conflicting_declaration row promises refusal of any created row a false invariant forbids, which only the input.<field> source kind receives"}]
```

Trend: pass 1 → 5, pass 2 → 5, carried 0 by signature (F4 is pass-1 F5 remainder). Coordinator routing: correction 2 (last). F2/F3 decided on the reader side: a newtype over Optional admits absence everywhere, so the setup reader (ess-conformance input.rs is_optional and the struct walk) sees through newtype layers, matching the domain.
