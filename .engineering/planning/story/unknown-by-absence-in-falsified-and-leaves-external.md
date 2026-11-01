---
format: aep.planning-md/3
id: story:unknown-by-absence-in-falsified-and-leaves-external
kind: story
status: draft
title: 'Unknown by absence in falsified and leaves_external follows the wrong-state witness, or #234 is restated'
relations:
- serves: vision:O2
revision: 1
---
## Finding

Found by U3 of wave D350 (out of scope there): `falsified` and `leaves_external`
(`crates/verify/ess-conformance/src/synthesize/subject_fact.rs`, near :3017) treat a guard that is
Unknown only because a field is absent as false, per the #234 ruling. D350 U3 made the wrong-state
witness treat such an absence as stopping selection; the two functions still follow #234.

## Decision needed

Reversing #234 for these two functions is a ruling, not a fix. Until a person decides, this stays
a draft.

## Acceptance

- After the decision: either both functions treat Unknown-by-absence as the wrong-state witness
  does, with red cases first, or the #234 ruling is restated in the design text beside the D350
  rule and this story is closed.
