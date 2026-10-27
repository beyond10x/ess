---
format: aep.planning-md/3
id: review-result:ess-0710-wave-1-adversary-pass-1
kind: review-result
status: active
title: Wave 1 of 0.71.0 adversary pass 1
summary: CONFIRMED; three U12 findings fixed in cfd392c507, the table re-pin already landed, one pre-existing note deferred to 0.72.0
relations:
- reviews: story:a-set-update-arranges-rows-whose-filtered-value-the-creator-copies-from-a-related-row
- reviews: story:a-set-update-witness-varies-the-creators-input-against-the-written-value
- reviews: story:an-affects-row-is-arranged-past-a-creators-emitting-not-found-branch
- reviews: story:a-converted-binding-input-arranges-the-related-row
revision: 1
---
## Adversary pass 1, wave 1 of 0.71.0 (one pass over the integrated branch)

Worktree ess-0710-adv at e73f9dfa2 (U3 876216edf, U4 e68be451f, U12 04cb2765f merged). The
adversary added `crates/verify/ess-conformance/tests/adversary_0710_pass1.rs`, 11 cases (4 red),
and fixtures under `tests/fixtures/adversary-0710/`. It also reported both suite-bytes tables red,
which 40e89c656 had already re-pinned on the integration branch.

Passing and kept as pins: an `exists: false` branch that itself creates the row is not taken as the
arranging creator (U3); a conversion at the trigger payload and a chain of two unconstrained
conversions are followed, and a prefixed target is named in the refusal (U4); enum and
`Optional<Boolean>` fields written from both inputs are varied and pass, and an update that refuses
the related row it names stays refused (U12).

Outcome: F1, F2 and F3 fixed in cfd392c507 (merged 8d1f7e2d57), the 11 cases green. F5 predates the
wave (0.70.0 gives the same result): pinned at today's behaviour naming
story:a-scenario-beside-a-prefixed-conversion-target-runs-on-the-interpreter (0.72.0).

```findings
- {file: crates/verify/ess-conformance/src/synthesize/set_effects.rs, line: 1443, category: property, severity: blocker, verdict: INFEASIBLE, origin: introduced, message: "copies of the first creator were applied to rows a second creator made, so the scenario expected a value the interpreter does not hold (constructed model); fixed in cfd392c507"}
- {file: crates/verify/ess-conformance/src/synthesize/set_effects.rs, line: 808, category: acceptance, severity: warning, verdict: CONFIRMED, origin: introduced, message: "the changing steer moved the suite bytes of a moving set update the base already witnessed; fixed in cfd392c507"}
- {file: crates/verify/ess-conformance/src/synthesize/set_effects.rs, line: 1687, category: boundary, severity: note, verdict: CONFIRMED, origin: introduced, message: "UNCHANGED was reported where the rows would change; fixed in cfd392c507"}
- {file: crates/verify/ess-conformance/tests/fixtures/synthesis-suite-bytes-table.tsv, category: contract-drift, severity: warning, verdict: NEEDS-CHANGE, origin: introduced, message: "U3 and U4 fixtures unpinned at the reviewed head; re-pinned in 40e89c656"}
- {file: crates/verify/ess-conformance/tests/adversary_0710_pass1.rs, category: boundary, severity: note, verdict: CONFIRMED, origin: pre-existing, message: "with a prefixed conversion target the trigger and mapping scenarios are Unsupported on the interpreter, as on 0.70.0 (deferred to story:a-scenario-beside-a-prefixed-conversion-target-runs-on-the-interpreter)"}
```
