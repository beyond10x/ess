---
format: aep.planning-md/3
id: story:cli-type-invariants
kind: story
status: draft
title: A CLI type carries an invariant, or the refusal names where the rule belongs
tags:
- feature-request
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-generated-code
revision: 5
---
## Outcome

A CLI type can carry an invariant, or the specification names one other place where a
refusal-stage rule over a CLI failure is stated and checked.

## Evidence

`ess specify cli` (0.56.0) refuses any invariant on a CLI type with "CLI type `<type>` has
unsupported invariants or union semantics" (`crates/generate/ess-cli-project`,
`tests/consumer_cli.rs:272`). An adopter wants to state on its `Failure` type that a timeout
after a dispatch step is reported as stage `dispatch` or `outcome_unknown`, never `admission`, and
carries the rule as a comment beside the type until ESS can express it.

Amendment A2 (thirteenth gap report, 2026-11-24): the refusal (`crates/specify/ess-cli-contract/src/resolve.rs:111-121`) applies to every constrained struct, not only to a `Failure` type: a payload struct with an invariant is refused the same way, with one message for two causes.

## Acceptance

- Either: an invariant over the fields of a CLI type validates and compiles, and the CLI
  projection either enforces it or names it as an obligation in its coverage report; union
  semantics stay refused with their own message, separated from the invariant refusal.
- Or: a design page names the construct where a refusal-stage rule belongs, and the refusal message
  points at it.
- A conformance scenario fails a target that reports the forbidden stage.

- Amendment A2 (thirteenth gap report, 2026-11-24): a struct used as a command payload with an invariant validates and projects, and the refusal message of a union differs from that of a constraint.

## Spec first

Decide the construct in `docs/design/` before code; the refusal message is split first so an
adopter can tell the two causes apart.

## Reconcile note (2026-11-02)

Store hygiene H0 of wave W1, section 2i: Split the refusal message out into its own story first; it ships ahead of the type invariants.

## Decisions

FM-D5 of the thirteenth report, a CLI struct with invariants. A: admit an invariant on a CLI payload struct; the CLI parser enforces what it can and the coverage report names each invariant it does not check as an obligation; union semantics keep their own refusal message, split from the constraint message. B: keep the refusal, split the message in two, and name the idiom (a second struct). C: a design page that names another place for a refusal-stage rule. Decided: A, 2026-11-24 16:36 CET (the coordinator, under the owner's delegation), taken on the default. This is the "Either" of the Acceptance.
