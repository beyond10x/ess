---
format: aep.planning-md/3
id: story:binding-rewrites-are-followed-for-subject-guards-and-decoy-rows
kind: story
status: draft
title: Synthesis follows binding rewrites for when_subject rows and decoy rows
relations:
- serves: vision:O2
revision: 1
---
## Finding

Found by U2 of wave D350 (out of scope there, from reading): D350 row 6 makes synthesis pass over
a `when_related:` row that a step's binding rewrites. Two neighbours are not covered: the same
rewrite gap for `when_subject:` guards, whose rows are arranged in
`crates/verify/ess-conformance/src/synthesize/subject_fact.rs`, and decoy rows, which are not
checked for binding rewrites (only the selected row, the branch's earlier steps and the rows
beside it are).

## Acceptance

- A `when_subject:` branch whose row a step's binding rewrites (field, state or removal) is sent
  on a row no binding rewrites, or refused as ESS-SYNTH-003 naming the step, the binding and the
  command, as `when_related:` is since 0.81.0.
- A decoy row a binding rewrites is not used as a decoy.
- Red cases first show each gap on a fixture.
