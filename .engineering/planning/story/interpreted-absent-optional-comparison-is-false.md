---
format: aep.planning-md/3
id: story:interpreted-absent-optional-comparison-is-false
kind: story
status: archived
title: An interpreted when_subject comparison with an absent Optional field is false
tags:
- adopter-report
- defect
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 3
transitions:
- {from: "draft", to: "archived", at: "2026-11-02T20:44:00Z", actor: "human:timo", revision: 3}
---
## Outcome

On the interpreted target, a `when_subject` comparison that meets an absent Optional stored field
is false, so the scenario runs and the command takes its other branch instead of being reported
unsupported.

## Evidence

An adopter on ess 0.56.0: entity `Code {method: Optional<String>}`, an outcome guarded
`when_subject: {predicate: method == "plain"}`. A scenario exchanging a code created without a
method, expecting the default success outcome, is reported unsupported by `--target interpreted`.
Workaround in use: `{method: {defined: true}}` beside every comparison. Not yet reproduced on
`main`.

## Acceptance

- A conformance test runs that scenario on `--target interpreted` and it passes with the default
  success outcome.
- The same scenario agrees on the Rust, Go and TypeScript targets.
- The predicates reference states what a comparison with an absent stored field evaluates to.

## Resolution

2026-11-02, store hygiene H0 of wave W1 (reconcile roadmap section 2b, R3; decision D6 option A,
taken as recommended by orchestrator D376: "D6 A archive interpreted-absent-optional-comparison-is-false"):
obsolete. The design keeps Unknown (`docs/design/generic-language-features.md:112`); coalesce serves
the case, and `story:a-guard-says-what-an-absent-value-compares-as` supersedes this story. Triage row
8 case 2 lives in `story:a-parent-tenant-guard-holds-across-every-related-row-and-absent-field` (W5).
Archived.
