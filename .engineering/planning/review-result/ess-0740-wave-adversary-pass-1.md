---
format: aep.planning-md/3
id: review-result:ess-0740-wave-adversary-pass-1
kind: review-result
status: active
title: The 0.74.0 adopter wave adversary pass 1
summary: 'NEEDS-CHANGE then fixed: F1 to F4 fixed in 02194e732'
relations:
- reviews: story:a-binding-tries-every-publisher-of-its-trigger
- reviews: story:a-synthesis-seed-is-accepted-for-a-caller-model
- reviews: story:a-caller-attribute-keeps-the-binding-refusal-scenario
- reviews: story:a-state-only-a-set-move-reaches-is-named-in-the-refusal
revision: 1
---
## Adversary pass 1, the 0.74.0 adopter wave (one pass over the integrated branch)

The pass ran on integration tree ess-0740-int at 1461d30e3, which has these merged, with the tables
re-pinned:
- U1 7a3bb80f7 (J);
- U2 668cb1b57 (P and O);
- U3 fd81b7346 (L);
- the 0.73.0 coverage-list fix 04ad7b10b.

The adversary added `crates/verify/ess-conformance/tests/adversary_0740_pass1.rs` with nine cases,
four of them red. Its fixtures are under `tests/fixtures/adversary-0740/`.

Five cases were attacked, held, and kept as passing:
- a per-row trigger whose row rests where a single-row move takes it (U1);
- the creator of the arranged row also publishes the trigger (U1);
- a seed whose second actor also sends the field-writing command (U2);
- a seed whose implied caller contradicts the seeded branch's guard (U2);
- a per-row trigger keyed by a caller attribute (U1 with U2).

Outcome:
- F1 to F4 are fixed in 02194e732, and all nine cases are green.
- The targeted gate passes, and no pinned table row moves.
- Every reduced reproduction keeps its count.

Not tried:
- the U1 tie rule with two publishers each refusing a different aspect;
- a publisher that is the binding's own invoked command;
- an actor with two caller attributes;
- a chain of two set moves.

```findings
- {file: crates/verify/ess-conformance/src/synthesize/caller.rs, line: 392, category: acceptance, severity: warning, verdict: NEEDS-CHANGE, origin: introduced, message: "a field any outcome sets from a caller attribute implied the caller even where a creator writes it from input, so rows any caller can create were refused as caller-contradiction; fixed in 02194e732"}
- {file: crates/verify/ess-conformance/src/synthesize/caller.rs, line: 404, category: acceptance, severity: warning, verdict: NEEDS-CHANGE, origin: introduced, message: "the implied caller value reached only actors allowed to send the field-writing command, so an admitted seed left another actor's branch refused; fixed in 02194e732"}
- {file: crates/verify/ess-conformance/src/synthesize/set_effects.rs, line: 2044, category: contract-drift, severity: note, verdict: NEEDS-CHANGE, origin: introduced, message: "the set-move-only text replaced refusals whose filter is also unmet, dropping the filter's own cause; fixed in 02194e732"}
- {file: crates/verify/ess-conformance/src/synthesize.rs, line: 2458, category: judgement, severity: note, verdict: NEEDS-CHANGE, origin: introduced, message: "TriggerRowUnarranged gave an input-key remedy where the trigger row rests only in a state a set move reaches; fixed in 02194e732"}
```
