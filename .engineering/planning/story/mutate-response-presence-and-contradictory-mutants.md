---
format: aep.planning-md/3
id: story:mutate-response-presence-and-contradictory-mutants
kind: story
status: draft
title: mutate gains a response-presence class and scores contradictory mutants equivalent
tags:
- adopter-report
- defect
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Outcome

`ess verify mutate` gains a mutant class that drops or adds a response field's presence, and
scores a mutant whose guard contradicts its own outcome as equivalent rather than survived or
unwitnessed.

## Evidence

An adopter on ess 0.56.0. On `main`, `MutantClass`
(`crates/verify/ess-conformance/src/mutate.rs:75-130`) has no response-presence class.
Equivalent mutants: 9a551e2115 (identical-answer precedence swap) and 224f5a97be (dead input
guard beside `when_subject`) score two shapes as equivalent; contradictory mutants are not
checked. Adjacent: `story:mutate-covers-subject-related-guards-sets-and-authored`
(https://github.com/beyond10x/ess/issues/515).

## Acceptance

- A mutate test: the response-presence class finds a site on a command with a REQUIRED response
  member, and a target that omits the member kills it.
- A mutate test: a mutant whose guard can never hold is reported equivalent, with the reason.
