---
format: aep.planning-md/3
id: story:explorer-and-interpreter-agree-on-external-cause-for-stored-record
kind: story
status: draft
title: The explorer and the interpreter agree on an external cause arranged for a stored record
tags:
- adopter-report
- defect
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Outcome

The Go explorer and the interpreted target expect the same branch when an `external:` cause is
arranged for a stored record whose held state another branch also answers.

## Evidence

An adopter on ess 0.56.0: once an `external:` unauthorized cause is arranged for a stored record,
the explorer and the interpreted target disagree (ESS-CF-OUTCOME). On `main`,
`crates/verify/ess-conformance/src/go/explore.go:1522-1537` offers an eligible external branch
beside `wrong_state`, sorted ahead of it, while the interpreter applies held state before
externals (precedence steps 4 and 6). Related:
`story:refusal-above-held-state-branch-validates`, which may change that precedence.

## Acceptance

- A test runs the Go explorer and the interpreted target over one specification with an external
  branch and a held-state branch on the same command; both expect the same branch for every
  arranged cause.
