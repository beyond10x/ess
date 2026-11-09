---
format: aep.planning-md/3
id: story:an-inexact-authored-decimal-is-refused
kind: story
status: draft
title: An authored Decimal that binary64 cannot hold exactly is refused at validation
tags:
- defect
- severity-high
- zero-open-issues
relations:
- serves: vision:O2
- informed_by: decision-blocker:decimal-canonical-serialization-crosses-repositories
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: cited
  path: crates/specify/ess-domain/src/command.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/command/value_expression.rs
- confidence: inferred
  path: crates/specify/ess-domain/tests/
- confidence: cited
  path: crates/specify/ess-primitives/src/facts.rs
- confidence: cited
  path: crates/specify/ess-primitives/tests/vectors/primitive-semantics.json
- confidence: inferred
  path: crates/specify/ess-yaml
- confidence: inferred
  path: the
revision: 5
---
## Outcome

`ess specify validate` refuses, by name and at the literal's location, an authored `Decimal`
literal whose value binary64 cannot represent exactly (for example `0.1000000000000000000001`,
which today is written and displayed as `0.1`). Serialization is unchanged: no persisted byte
moves, no format version changes.

## Problem

Reproduced on 0.92.0 (`ess/35`), two authored positions:

- **A `sets:` literal.** An entity field `amount: Decimal` set by its creator as
  `amount: 0.1000000000000000000001` validates (exit 0) and compiles to
  `"value": {"kind": "literal", "value": "0.1"}`. The YAML number reaches `ess-domain` as an `f64`
  (`RawPayloadSource::Decimal(f64)`, `crates/specify/ess-domain/src/command.rs`, written back by
  `decimal_text`), so the authored digits are gone before any check could see them.
- **A guard literal.** A refusal `too-small` with `when: amount < 0.1000000000000000000001` over a
  `Decimal` input validates and compiles to `"predicate": "amount < 0.1"`. Synthesis then sends
  `amount = 0.1` in `gap.demo.CreateParent/outcome/created`; under the authored guard 0.1 is less
  than the literal, so that input selects `too-small`. An implementation faithful to the authored
  specification fails the generated suite.

`0.1`, `0.5` and `1e2` validate and compile unchanged.

## Decision

Taken 2026-10-07 on `decision-blocker:decimal-canonical-serialization-crosses-repositories`: a
value is refused by name rather than changed silently. The exact-decimal writer (Decimal as a JSON
string, `ess-conformance/6`, `ess-conformance-report/2`, a reader in AEP first) stays parked on
`obligation:review-contract-rollout-coordination`.

## Not covered

A `Decimal` that reaches ESS at run time (an implementation's reply, a conformance report value)
is not authored and is not refused by this story. It still collapses to the nearest binary64 value
when written. That case stays with the parked blocker.

## Acceptance

- A specification carrying `0.1000000000000000000001` as a `Decimal` literal fails
  `ess specify validate` with a refusal naming the literal, its location and the nearest binary64
  value; `0.1`, `0.5` and `1e2` still validate. Both positions above are covered: a `sets:` /
  payload literal and a guard literal (`when:`, `when_subject:`), each with its own case.
- The refusal is modelled in the repository's ESS specification before the validator changes
  (spec first), and the change is one conformance scenario or test per accepted and refused case.
- No vector in `crates/specify/ess-primitives/tests/vectors/primitive-semantics.json` changes its
  `json` bytes.

No GitHub issue reports this case and no fixture carries such a literal (measured in wave 22).

## Scope

- cited: crates/specify/ess-domain/src/command.rs (`RawPayloadSource::Decimal`, `decimal_text`)
- cited: crates/specify/ess-primitives/src/facts.rs (`Number`, `Repr::Exact` / `Repr::Binary64`)
- cited: crates/specify/ess-primitives/tests/vectors/primitive-semantics.json
- inferred: crates/specify/ess-yaml (the literal's source text and location)
- inferred: crates/specify/ess-domain/src/command/value_expression.rs
- inferred: crates/specify/ess-domain/tests/
- inferred: the repository's ESS specification of the validator (spec first)

confidence: medium
