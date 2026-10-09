---
format: aep.planning-md/3
id: story:component-explorer-walks-only-its-component
kind: story
status: draft
title: A component's Go explorer walks only that component's commands
tags:
- adopter-report
- defect
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Outcome

The Go package `ess generate synthesize --component <c>` writes explores only that component's
commands, so a single-component target passes without `AllowExcluded`, and `AllowExcluded` keeps
meaning that a real exclusion was accepted.

## Evidence

An adopter on ess 0.56.0: the component package's explorer walks the other component's commands.
On `main`, `crates/edge/ess-cli/src/main.rs:4600` passes the whole IR to `emit_with_model`, which
`crates/verify/ess-conformance/src/go/mod.rs:136` writes as `ir.json`; `explore.go` has no
component filter. https://github.com/beyond10x/ess/pull/518 fixed authored scenarios outside
`--component` only.

## Acceptance

- A test synthesizes the Go package for one component of a two-component specification; the
  explorer's command set holds only that component's commands and its run passes with
  `AllowExcluded` unset.
