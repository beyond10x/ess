---
format: aep.planning-md/2
id: review-result:b1b-adversary-pass-1
kind: review-result
status: active
title: B1b adversary pass 1 (string operators)
relations:
- reviews: story:string-prefix-suffix-substring-operators
revision: 1
---
Adversary pass 1 on wave unit B1b (#95 string operators, implementation). Tree
`ess-wave-20260925-b1b` on `a8a13e08c` plus the implementor's uncommitted diff.

Header as returned: verdict red; cases executed 1019→1027, red 6; origin introduced 4,
pre-existing 0, undecided 1. Added two adversary test files.

Coordinator routing: F1, F4, F5 back to the implementor; F2 (INFEASIBLE, hand-written suites only)
fixed anyway so Go conjoins as Rust does; F3 (undecided) decided as a fix — invariant-literal
candidates for any input path whose base witness violates its type's invariant. The toolchain
model's missing `NullComparison` (from unit A1) added in the same round.

```findings
[{"file":"crates/verify/ess-conformance/src/go/runtime.go","line":855,"category":"contract-drift","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"Go coverage-lineage meaning reads a string-operator operand through meaningScalar, so a child swapping +44 for 44 passes Go's parent check while Rust refuses it."},{"file":"crates/verify/ess-conformance/src/go/predicate.go","line":305,"category":"contract-drift","severity":"note","verdict":"INFEASIBLE","origin":"introduced","message":"Go admits a mapping with several string operators but evaluates only the first, where Rust conjoins them."},{"file":"crates/verify/ess-conformance/src/witness.rs","line":274,"category":"boundary","severity":"warning","verdict":"CONFIRMED","origin":"undecided","message":"A newtype input whose string invariant refuses its base witness is unwitnessable when no guard reads it."},{"file":"crates/verify/ess-conformance/src/witness.rs","line":765,"category":"boundary","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"Composition takes the first positive prefix, so a satisfiable guard with a later prefix extending it plus a suffix gets no witness."},{"file":"crates/specify/ess-domain/src/expression.rs","line":922,"category":"judgement","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"The quote-it hint builds its suggested literal from the parsed number, telling an author who wrote +44 to write 44."}]
```
