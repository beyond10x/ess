---
format: aep.planning-md/2
id: story:mutate-drives-an-external-target
kind: story
status: implemented
title: Mutation audits a real implementation through its own runner
relations:
- serves: vision:O2
- decomposes: epic:retrofit-findings-20260927
- supersedes: story:external-mutation-explorer-and-toolchain
revision: 4
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
