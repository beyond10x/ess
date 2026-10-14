---
format: aep.planning-md/3
id: epic:generic-guard-precedence
kind: epic
status: draft
title: One precedence rule for which guard answers first
tags:
- adopter-report
refs:
- provider: github
  reference: beyond10x/ess#554
relations:
- serves: vision:O2
- informed_by: epic:one-selection-plan
revision: 1
---
## Outcome

One generic capability: **one precedence rule for which guard answers first**. Every way a branch
is selected (`unknown_instance`, `existing_instance`, held state, `when_subject`, `when_related`
over an identity, a stored reference or a row set, input guards, `external:`) is a predicate in one
ordered rule, so any combination of guards on one command validates, and the rule, not a refusal
per pair, says which answers first. Validation, the interpreter, synthesis, Entity Runtime
lowering, the generated targets and mutation read that one rule.

Binding design: `docs/design/generic-language-features.md` (being drafted), feature 4. Informed by
`epic:one-selection-plan` (the derived plan every consumer reads); this epic decides the rule the
plan derives, including whether declaration order is the precedence.

## Children

| story | covers |
|---|---|
| `story:one-command-combines-identity-and-row-set-related-guards` | https://github.com/beyond10x/ess/issues/554, https://github.com/beyond10x/ess/issues/473 (and its comment of 2026-10-13) |

Visibility issue: https://github.com/beyond10x/ess/issues/554.

## Earlier requests in the same family (cited, not moved here)

- https://github.com/beyond10x/ess/issues/534 (`story:refusal-above-held-state-branch-validates`),
  https://github.com/beyond10x/ess/issues/530 and https://github.com/beyond10x/ess/issues/535: open,
  in a release pass of their own; their stories are not edited here.
- `story:unknown-instance-and-input-guard-agree-on-order` (an input guard above an unknown-instance
  branch) and `story:external-branch-narrowed-by-subject-guard`.
- https://github.com/beyond10x/ess/issues/282, https://github.com/beyond10x/ess/issues/283,
  https://github.com/beyond10x/ess/issues/454, https://github.com/beyond10x/ess/issues/486: each
  added one ordered pair to `docs/design/cross-record-and-stored-field-guards.md` § "The precedence
  order".

## Acceptance

Every pairwise `ESS-COMMAND-004`/`ESS-COMMAND-009` "which of the two answers first is not stated"
refusal is gone at the new format; one table in the binding design states the rule, and a corpus
test runs the interpreter, synthesis and every generated target over each pair.
