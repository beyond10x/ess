---
format: aep.planning-md/3
id: release-plan:interpreter-runner-parity
kind: release-plan
status: draft
title: 'Unscheduled: the interpreter executes the model and the runners agree'
relations:
- serves: vision:O2
- delivers: story:interpreted-bindings-and-unmet-obligations
- delivers: story:interpreted-eventual-views
- delivers: story:interpreted-scenario-supplied-facts
- delivers: story:interpreted-trust-gate
- delivers: story:the-interpreter-executes-stored-field-guards
- delivers: story:go-and-typescript-read-current-suites
- delivers: story:cross-runtime-verdict-equivalence
- delivers: story:a-report-says-why-a-scenario-was-skipped
- delivers: story:explorer-restart-suite-step
- delivers: story:concurrent-history-records-inputs
revision: 1
---
Carried from `release-plan:ess-056` on 2026-10-07, when that version number went to the release that actually shipped under it. Not scheduled to a version; its original intent follows.

## Intent

Release ESS 0.56.0 (target 2026-10-09): the reference interpreter executes what the model says,
and the Go, TypeScript and Rust runners reach the same verdicts on the same suites. A suite-format
bump is admitted if a story needs one; no source-format bump.

## Scope

| group | stories |
|---|---|
| interpreter (`epic:model-driven-interpretation`) | `interpreted-bindings-and-unmet-obligations`, `interpreted-eventual-views`, `interpreted-scenario-supplied-facts`, `interpreted-trust-gate`, `the-interpreter-executes-stored-field-guards` |
| runtime parity | `go-and-typescript-read-current-suites`, `cross-runtime-verdict-equivalence` |
| reports | `a-report-says-why-a-scenario-was-skipped` |
| explorer | `explorer-restart-suite-step`, `concurrent-history-records-inputs` |

## Order

Draft stories need a scope (`story-scoper`) and the plan-critic panel before a wave. Decompose
`epic:model-driven-interpretation` first; `interpreted-trust-gate` depends on the other three
interpreter stories.

## Required completion

As `release-plan:ess-054`.
