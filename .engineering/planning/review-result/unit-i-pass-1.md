---
format: aep.planning-md/3
id: review-result:unit-i-pass-1
kind: review-result
status: active
title: Unit I adversary pass 1
summary: CONFIRMED; two pre-existing warnings in the ess/30 reference proof deferred to 0.71.0, one infeasible note
relations:
- reviews: story:create-or-update-beside-a-related-row-refusal-synthesizes-every-branch
revision: 1
---
## Adversary pass 1, unit I (ess-0700 wave)

Worktree ess-0700-i, base 0948f833e; the adversary added `crates/verify/ess-conformance/tests/adversary_i_create_or_update.rs`, 11 cases (3 red). Cases executed 10 to 21. The unit's own claims held: all 24 declaration orders validation admits synthesize and pass, the interpreter with Existence and PresentRelated exchanged fails `already-bound`, and a second related predicate, a second related row, an `Optional` agent, a struct identity, ess/27 and determinism stay green. The pin tables only add rows.

Decision 2 of the implementor confirmed: the #282 preflight maps the unknown seat to `bound` whatever the order, so the faulty control on the generated target is the substitute for "`bound` fails" on the interpreter.

No correction round: the red cases are pre-existing in ess-domain (findings 1 and 2 share the cause) or infeasible. They stay as ignored cases naming story:a-create-or-update-writer-is-no-proof-a-stored-reference-is-present (ess-0.71.0) and this review.

```findings
- {file: crates/verify/ess-conformance/src/synthesize/related_guard/stored.rs, line: 139, category: acceptance, severity: warning, verdict: CONFIRMED, origin: undecided, message: "synthesis refuses a later stored-reference exists false branch as unreachable because it counts the unknown_instance creator as refusing a missing related row first, yet that creator stores one under the one rule (deferred to story:a-create-or-update-writer-is-no-proof-a-stored-reference-is-present)"}
- {file: crates/specify/ess-domain/src/command/reference_integrity.rs, line: 184, category: contract-drift, severity: warning, verdict: CONFIRMED, origin: pre-existing, message: "the ess/30 enforced-reference proof drops the missing-row refusal although a create-or-update creator beside the related refusal stores an identity no row carries (deferred to the same story)"}
- {file: crates/specify/ess-domain/src/command/related_guard.rs, line: 2455, category: boundary, severity: note, verdict: INFEASIBLE, origin: pre-existing, message: "with two input-named related refusals validation reports already-bound and bound as selected together although one needs a held seat and the other an unknown one"}
```
