---
format: aep.planning-md/3
id: story:a-literal-on-a-field-only-an-invariant-names-is-not-a-survivor
kind: story
status: draft
title: A literal on a field only an invariant names is not a survivor
summary: sets-retarget literal on an invariant-only field stays a scored survivor; oracle default exit 3 to 1 since ebed16afd6 (W2 MUT1 finding, D413, backlog)
tags:
- backlog
- from-finding
- review-finding
relations:
- informed_by: review-result:adversary-w2-mut1-pass-1
revision: 1
---
## Need

Since ebed16afd6 (wave W2, story:the-mutation-audit-reports-the-sites-it-could-not-mutate),
`sets-retarget` gives a creating or `Optional` write with no same-typed sibling input a literal
mutant. MUT1 fix round 1 (ee16051c22) lists such a literal as `unobserved_field` where no view or
derivation reads the field. A field that only an invariant names still counts as read. So the
oracle fixture's `sets-retarget/oracle.order.PlaceOrder/accepted/weight_grams` (`weight_grams >= 0`)
stays a scored survivor that no scenario can tell from the original. It is pinned as an equivalent
survivor in `ORACLE_SURVIVORS` (`crates/verify/ess-conformance/tests/mutation_audit.rs`, 4b58a5ee95).
Since ebed16afd6 the oracle fixture's default audit exits 1 where it exited 3. Found in MUT1 fix
round 1 (review-result:adversary-w2-mut1-pass-1, F1); D413 keeps it out of W2, because a narrower
notion of "read" changes that story's acceptance.

## Acceptance

- A `sets-retarget` literal on a field that only an invariant names, and that no view, derivation
  or response reads, is scored `equivalent` or listed as an unmutated site, not `survived`.
- The oracle fixture's default audit exit status is the one it had before ebed16afd6.
- A literal on a field a view reads keeps its id and verdict.

## Schedule

Backlog: not waved; considered only after every predefined wave is released.
