---
format: aep.planning-md/3
id: story:identity-guard-decided-on-the-row-sent-on-every-synthesis-path
kind: story
status: draft
title: Synthesis decides an identity guard on the row sent on the stored-field and addressed-row paths
relations:
- serves: vision:O2
revision: 1
---
## Finding

Measured by adversary pass 1 of wave D343 (`review-result:adversary-d343-u3-pass-1`, F3 and F2, out
of scope there): synthesis decides a branch's identity guard on the row it sends on the
subject-addressed paths D343 U3 fixed, but not on the stored-field path `prepare_state_input` nor on
`addressed_row` (row-set accepting branches, complete external refusals), which passes no input
(`crates/verify/ess-conformance/src/synthesize.rs`). F2 (inferred): a related-boundary run for one
conjunct takes the first answering branch, so under an identity guard it can be answered by a
branch that never reads the related row and isolates nothing.

## Acceptance

- On `prepare_state_input` and `addressed_row`, an identity guard is decided on the row sent, or the
  branch is refused ESS-SYNTH-003 naming the guard; a red case per path first.
- F2 is reproduced by a run or closed as not reproducible, with the run named.
