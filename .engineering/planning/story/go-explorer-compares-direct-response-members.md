---
format: aep.planning-md/3
id: story:go-explorer-compares-direct-response-members
kind: story
status: draft
title: The Go explorer compares direct response members
tags:
- adopter-report
- defect
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Outcome

The generated Go explorer compares a command's direct response members with the model's, so a
target that omits a REQUIRED member, or answers a different value the model determines, fails
exploration.

## Evidence

An adopter on ess 0.56.0: a server that omits a REQUIRED response member passes 12,000 explorer
steps (the synthesized suite still catches it). On `main`,
`crates/verify/ess-conformance/src/go/explore.go:2013-2053` compares only the outcome, the error,
the event names and the event payload; nothing in `explore.go` reads the command response.

## Acceptance

- A test runs the Go explorer against a target that drops a REQUIRED direct response member; the
  exploration fails naming the member.
- A member whose value the model does not determine (a `{generated: true}` field) is checked for
  presence only.
