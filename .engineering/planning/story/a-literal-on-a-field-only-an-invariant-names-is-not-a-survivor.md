---
format: aep.planning-md/3
id: story:a-literal-on-a-field-only-an-invariant-names-is-not-a-survivor
kind: story
status: draft
title: A literal on a field only an invariant names is not a survivor
summary: sets-retarget literal on an invariant-only field stays a scored survivor; oracle default exit 3 to 1 since ebed16afd6 (W2 MUT1 finding, D413, backlog)
tags:
- backlog
- defect
- from-finding
- review-finding
- severity-medium
- zero-open-issues
relations:
- informed_by: review-result:adversary-w2-mut1-pass-1
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: inferred
  path: crates/edge/ess-cli/src/main.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/mutate.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/mutation_audit.rs
revision: 5
---
## Need

Since ebed16afd6 (wave W2, story:the-mutation-audit-reports-the-sites-it-could-not-mutate),
`sets-retarget` gives a creating or `Optional` write with no same-typed sibling input a literal
mutant. MUT1 fix round 1 (ee16051c22) lists such a literal as `unobserved_field` where no view or
derivation reads the field. The test for "read" is `field_is_read`
(`crates/verify/ess-conformance/src/mutate.rs:1380`): it renames the field and recompiles, and a
copy that does not compile counts as read. An entity invariant that names the field makes the copy
fail to compile, so a field that only an invariant names counts as read and the literal mutant is
scored. No scenario can tell it from the original when the literal satisfies the invariant, so it
is a survivor that no suite can kill.

Reproduction on 0.92.0 with the oracle fixture (`Order.weight_grams` is read by no view, only by
the entity invariant `weight_grams >= 0`):

```text
$ ess verify conform mutate --path examples/oracle-fixture --target oracle-fixture --class sets-retarget
mutation audit of oracle v1 against oracle-reference: 2 mutant(s), 1 killed, 1 survived, 0 inconclusive, 0 stillborn, 0 unwitnessed, 0 equivalent (baseline: 34 scenario(s), 6 refusal(s)); 2 of 2 site(s) scored
survived sets-retarget/oracle.order.PlaceOrder/accepted/weight_grams: `weight_grams: input.weight_grams` becomes `weight_grams: 7919`
killed sets-retarget/oracle.order.PlaceOrder/accepted/contact: ...
exit=1
```

The full default audit (`--target oracle-fixture`, every class) also exits 1; before ebed16afd6 it
exited 3 (unavailable `emit-swap` sites). The survivor is pinned as equivalent in
`ORACLE_SURVIVORS` (`crates/verify/ess-conformance/tests/mutation_audit.rs:427`). Found in MUT1 fix
round 1 (review-result:adversary-w2-mut1-pass-1, F1); D413 keeps it out of W2, because a narrower
notion of "read" changes that story's acceptance.

## Acceptance

- `ess verify conform mutate --path examples/oracle-fixture --target oracle-fixture --class sets-retarget`
  reports `sets-retarget/oracle.order.PlaceOrder/accepted/weight_grams` as `equivalent` or as an
  unmutated site (`unobserved_field`), not `survived`, and exits 0.
- The same audit with every class exits 3, as it did before ebed16afd6.
- A literal whose value breaks the invariant, or a literal on a field a view, guard, derivation,
  payload or other write reads, keeps its id and verdict; `sets-retarget/oracle.order.PlaceOrder/accepted/contact`
  stays `killed`.
- `ORACLE_SURVIVORS` in `tests/mutation_audit.rs` is empty and the pinned verdicts and unmutated
  list move with it.

## Scope

- cited: crates/verify/ess-conformance/src/mutate.rs
- cited: crates/verify/ess-conformance/tests/mutation_audit.rs
- inferred: crates/edge/ess-cli/src/main.rs
- inferred: CHANGELOG.md

confidence: medium

## Schedule

Backlog: not waved; considered only after every predefined wave is released.
