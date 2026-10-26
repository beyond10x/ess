---
format: aep.planning-md/3
id: story:a-literal-set-field-steers-the-creator-of-a-subject-row
kind: story
status: implemented
title: A subject guard over a field a creator sets to a literal finds that creator
summary: hinted folds literal-set fields against the creator's literal, so when_subject over (literal, input) pairs is arranged instead of refused ESS-SYNTH-003 (row 41).
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-10-22T23:53:09Z", actor: "human:timo", revision: 2, correlation: "ess-0690"}
- {from: "proposed", to: "active", at: "2026-10-22T23:53:09Z", actor: "human:timo", revision: 3, correlation: "ess-0690"}
- {from: "active", to: "implemented", at: "2026-10-26T09:27:37Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":1,"verification":1}}}
---
## Need

Reported by adopter B (tracker row 41, 2026-10-23, `ess/29`, re-tested on 0.68.0). A `Dispatch`
command's branches select the subject row by `when_subject: {all: [mode == Key, route == D]}`. A
creating outcome writes `mode: Key` as a literal and `route` from its input. With three creating
outcomes and at least three earlier refusals, the branch is refused as ESS-SYNTH-003 "stored …
selecting this branch, over the rows N bounded arrangements left", although the creator writes that
pair. A second shape (route `Builtin` vs `Relay`, the three creating outcomes guarded on Booleans)
is refused the same way.

## Where it lives

- `crates/verify/ess-conformance/src/synthesize/subject_fact.rs:3637-3685` (`hinted`): the hint is
  translated through the creator's input mapping only, so the literal-set atom `mode == Key` stays an
  atom no input sets; `Directed` gives it no truth value and steers no creator input.
- Refusal at `:5125-5145`; creators at `:4313-4417`.
- Removing the literal atom from the guard gives 0 refusals (9 scenarios); the 64-candidate walk is
  not the cause.

## Acceptance

- In `tests/arrangement_creators.rs`, the adopter's shape (route enum from input, `mode` set to a
  literal, three creating outcomes, three earlier refusals) gets a scenario for every `Dispatch`
  branch, and so does the Boolean-guarded variant.
- A literal-set atom the creator's literal contradicts keeps the branch refused, naming it.
- No format change.
