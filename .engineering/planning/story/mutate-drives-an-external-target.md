---
format: aep.planning-md/3
id: story:mutate-drives-an-external-target
kind: story
status: implemented
title: Mutation audits a real implementation through its own runner
relations:
- serves: vision:O2
- decomposes: epic:retrofit-findings-20260927
- supersedes: story:external-mutation-explorer-and-toolchain
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-09-27T07:56:18Z", actor: "human:timo", revision: 2, imported: true}
- {from: "proposed", to: "active", at: "2026-09-27T07:57:00Z", actor: "human:timo", revision: 3, imported: true}
- {from: "active", to: "implemented", at: "2026-09-27T14:56:24Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":1}}, imported: true}
---
## Scope

- #153: `ess verify conform mutate` audits a real implementation: it emits each mutant's
  specification and suite for the project's own runner and scores the reports it collects
  (`--emit` / `--collect`), or runs a project command per mutant.

Split from `story:external-mutation-explorer-and-toolchain` (archived). Cited sites:
`crates/edge/ess-cli/src/main.rs:588,693,3025`, `crates/verify/ess-conformance/src/mutate.rs`.

## Acceptance

A mutation audit of a specification that no built-in target passes runs against an external
runner, and its report counts killed, survived and invalid mutants per class.
