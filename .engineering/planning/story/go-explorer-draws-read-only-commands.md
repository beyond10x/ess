---
format: aep.planning-md/3
id: story:go-explorer-draws-read-only-commands
kind: story
status: draft
title: The Go explorer draws read-only commands on a supplied subject
tags:
- adopter-report
- defect
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Outcome

The generated Go explorer draws a command whose outcomes include a `preserves:` outcome on a
supplied subject (a read-only command), instead of excluding it from random sequences.

## Evidence

An adopter hardening OAuth specifications on ess 0.56.0: two specifications had their read-only
commands excluded. On `main`, `exploreEffectRefusal`
(`crates/verify/ess-conformance/src/go/explore.go:852-865`) admits only `creates` from observed or
`moves`/`updates` from supplied; every other effect, `preserves` included, is an exclusion reason
(called at `explore.go:1012-1015`).

## Acceptance

- A test generates the Go explorer for a specification with a read-only command on a supplied
  subject; the command is in the explored set and a drawn step expects the subject unchanged.
- A target that changes the subject on that command fails the exploration.
