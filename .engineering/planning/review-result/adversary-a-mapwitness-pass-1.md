---
format: aep.planning-md/3
id: review-result:adversary-a-mapwitness-pass-1
kind: review-result
status: active
title: Adversary pass 1, 0.41 unit mapwitness
relations:
- reviews: story:map-inputs-get-a-non-empty-witness
revision: 1
---
unit: mapwitness (#196)
verdict: red
cases: executed 1440→1454, red 3
origin: introduced 2, pre-existing 1, undecided 0
wrote-outside-worktree: ~/.cache/ess-wave-n2/mapwitness/adv1/
needs-coordinator: no

Adversary pass 1 (`aep:adversary`), 2026-09-28, head 10c5d91d6 + `tests/adversary_map_witness_pass1.rs` (14 cases, 3 red). Held: dropped/mangled/re-keyed maps fail on Rust, Go and TS; all key primitives; Optional<Map>; count guards == 0, > 1, >= 3 both sides; examples byte-identical to committed suites.

```findings
[{"file":"crates/verify/ess-conformance/src/witness.rs","line":2333,"category":"boundary","severity":"blocker","verdict":"NEEDS-CHANGE","origin":"introduced","message":"a type recursive through a map now gets a depth-limited nested witness that candidates() drops (Ok([])), so the command scenario is refused where the base witnessed it as {}"},{"file":"crates/verify/ess-conformance/src/witness.rs","line":1610,"category":"boundary","severity":"warning","verdict":"CONFIRMED","origin":"introduced","message":"the map count ladder leaves the false side of >= 3 at the one-entry base, so a target implementing >= 2 passes, while the list twin is caught"},{"file":"crates/verify/ess-conformance/src/witness.rs","line":2276,"category":"acceptance","severity":"note","verdict":"CONFIRMED","origin":"pre-existing","message":"a copied list is witnessed as [], so a map inside a list element is never asserted"},{"file":"crates/verify/ess-conformance/tests/fixtures/transcript-target-go.go","line":66,"category":"judgement","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"integralTokens makes every Go parity replay treat 1.0 and 1 as equal, hiding number-spelling divergences between the runtimes"},{"file":"crates/verify/ess-conformance/tests/map_witness.rs","category":"mutant","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"the unit only map count guard is == 0, so dropping the ladder length loop leaves its tests green"}]
```

Coordinator routing: F1, F2 fix; F3 fix in-unit (a list copied by sets: gets one element, as #196 decides for maps; this is the same defect in list form); F4 no-op (ESS compares numbers by value — 1 and 1.0 are one number — so the fixture normalisation matches the contract); F5 closed by the adversary cases.
