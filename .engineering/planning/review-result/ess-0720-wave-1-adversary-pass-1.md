---
format: aep.planning-md/3
id: review-result:ess-0720-wave-1-adversary-pass-1
kind: review-result
status: active
title: Wave 1 of 0.72.0 adversary pass 1
summary: 'NEEDS-CHANGE then fixed: F1 to F4 fixed in e2f35c133, one F1-kind gap filed for a later wave'
relations:
- reviews: story:a-set-effect-bound-to-a-fresh-input-composes-after-a-disclosure
- reviews: story:a-binding-into-an-updating-command-follows-its-related-guards
- reviews: story:a-binding-arranges-the-row-a-set-guard-selects
- reviews: story:a-binding-whose-trigger-shares-the-row-guard-flows
revision: 1
---
## Adversary pass 1, wave 1 of 0.72.0 (one pass over the integrated branch)

Worktree ess-0720-adv at 2e8716a59 (U4 a51d2e119, U3 f38e97ac3, U2 ec43bd310, U1 552ebe610
merged). The adversary added `crates/verify/ess-conformance/tests/adversary_0720_pass1.rs`, five
cases, all red, with fixtures under `tests/fixtures/adversary-0720/`.

Attacked and held: a same-row set guard together with an updating command is refused naming the
branch (U2 with U3); rows written by another binding stay opaque to a set guard (U3); an `or`
filter, an absent optional input and an origin literal nested in an object stay refused (U1).

Outcome: F1 to F4 fixed in e2f35c133 (merged cbd83c332), and the five cases are green. The
targeted gate passes: 33 binaries, 463 tests, plus the tables. The repros keep their counts. The
suite-bytes row of the set-effect-fresh-input fixture moves, as F2 intends. One gap of the F1
kind remains open, in the destination arrangement of a binding, and is filed as
story:a-binding-destination-arrangement-keeps-the-trigger-on-its-branch.

```findings
- {file: crates/verify/ess-conformance/src/synthesize/binding_effects.rs, line: 1551, category: acceptance, severity: blocker, verdict: NEEDS-CHANGE, origin: introduced, message: "the related row was put into the trigger's input without re-deciding the trigger, so the trigger's outcome scenario and its binding scenarios failed; fixed in e2f35c133"}
- {file: crates/verify/ess-conformance/src/synthesize/caller.rs, line: 999, category: boundary, severity: blocker, verdict: NEEDS-CHANGE, origin: introduced, message: "a follow-up whose set effect selected no row was composed, and its per-row event expectation failed; fixed in e2f35c133"}
- {file: crates/verify/ess-conformance/src/synthesize/binding_effects/set_guard.rs, line: 202, category: acceptance, severity: blocker, verdict: NEEDS-CHANGE, origin: introduced, message: "a captured identity the trigger needs was overwritten by the arranged literal; fixed in e2f35c133"}
- {file: crates/verify/ess-conformance/src/synthesize/binding_effects/set_guard.rs, line: 102, category: boundary, severity: note, verdict: CONFIRMED, origin: undecided, message: "a binding the trigger alone decides was refused; fixed in e2f35c133"}
```
