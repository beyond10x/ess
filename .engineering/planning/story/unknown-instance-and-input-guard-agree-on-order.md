---
format: aep.planning-md/3
id: story:unknown-instance-and-input-guard-agree-on-order
kind: story
status: draft
title: Validation, interpreter and explorer agree on an unknown-instance branch below an input guard
tags:
- adopter-report
- defect
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Outcome

The validator, the interpreter and the Go explorer agree on which branch answers when an
unknown-instance (`when_related exists: false` or `unknown_instance`) refusal and an input guard
declared above it both hold; a specification whose declaration order disagrees with that
precedence is refused at validation, as ESS-COMMAND-004 already does for held-state branches.

## Evidence

An adopter on ess 0.56.0: the explorer fails a target that checks in declared order, and
`specify validate` accepts either expectation. On `main`, the precedence answers `exists: false`
before input refusals (`docs/design/cross-record-and-stored-field-guards.md:746-748`,
`crates/verify/ess-conformance/src/interpret/execute.rs:516-520`); the declaration-order check
(`conflicting_declaration`) covers held-state and external steps only (design page `:762`).

## Acceptance

- A validation test: an input guard declared above an unknown-instance branch whose guards can
  hold together is refused, naming both branches and the order to declare them in.
- A test over a valid order: the interpreter and the Go explorer expect the same branch.

## Decisions

- Design first: whether the fix is a refusal (as for held state) or an authored precedence; the
  design page is updated before code.
