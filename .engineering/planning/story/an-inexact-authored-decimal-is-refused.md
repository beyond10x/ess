---
format: aep.planning-md/3
id: story:an-inexact-authored-decimal-is-refused
kind: story
status: active
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
  path: crates/edge/ess-xtask/src/consumer_coverage/macro-guards.json
- confidence: cited
  path: crates/specify/ess-compiler/src/resolve.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/command/value_expression.rs
- confidence: inferred
  path: crates/specify/ess-domain/tests/inexact_decimal_literal.rs
- confidence: cited
  path: crates/specify/ess-primitives/src/error.rs
- confidence: cited
  path: crates/specify/ess-primitives/src/facts.rs
- confidence: inferred
  path: crates/specify/ess-yaml/src/lib.rs
- confidence: inferred
  path: docs/design/review-typed-diagnostics.md
- confidence: inferred
  path: website/docs/reference/diagnostics.md
revision: 12
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:39:47Z", actor: "human:timo", revision: 11, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:39:49Z", actor: "human:timo", revision: 12, decided_on: {"recorded":{"review_outcome":2}}}
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
  specification fails the generated suite. The collapse happens in `Number::parse_decimal`
  (`crates/specify/ess-primitives/src/facts.rs`), which keeps the authored digits only when they
  equal the canonical decimal of their binary64.

`0.1`, `0.5` and `1e2` validate and compile unchanged.

## Decision

Taken 2026-10-07 on `decision-blocker:decimal-canonical-serialization-crosses-repositories`: a
value is refused by name rather than changed silently. The exact-decimal writer (Decimal as a JSON
string, `ess-conformance/6`, `ess-conformance-report/2`, a reader in AEP first) stays parked on
`obligation:review-contract-rollout-coordination`.

## Decisions

- The refusal is a new validation code, `inexact_decimal_literal`, so a harness matches on the code
  as it does on every other refusal (`ValidationCode`, `crates/specify/ess-primitives/src/error.rs`).
- The former "modelled in the repository's ESS specification first" bullet is dropped: the
  toolchain model (`models/toolchain/domains/specify.yaml`) declares every validation refusal as
  one `toolchain.specify.Refused` carrying a list of diagnostics, so a new code adds no declaration
  there and the bullet named no observable result.

## Not covered

A `Decimal` that reaches ESS at run time (an implementation's reply, a conformance report value)
is not authored and is not refused by this story. It still collapses to the nearest binary64 value
when written. That case stays with the parked blocker.

## Acceptance

Each case below is one test in `cargo test -p ess-domain --locked --test inexact_decimal_literal`,
over a specification with an entity field `amount: Decimal`.

- A creator's `sets: {amount: 0.1000000000000000000001}` fails validation with
  `[inexact_decimal_literal]`, at the `sets` location of that outcome, with a message that names
  `0.1000000000000000000001` and the nearest binary64 value `0.1`.
- A refusal `when: amount < 0.1000000000000000000001` fails validation with
  `[inexact_decimal_literal]` at that `when` location, naming the same two values; the same literal
  under `when_subject:` fails the same way at its own location.
- `sets: {amount: 0.1}`, `0.5` and `1e2`, and `when: amount < 0.1`, each validate with no
  diagnostic.
- `cargo test -p ess-primitives --locked --test primitive_corpus` passes and
  `crates/specify/ess-primitives/tests/vectors/primitive-semantics.json` is byte-identical to its
  state before the change (`git diff --exit-code` on it).
- `task test-xtask` and `cargo test -p ess-compiler --locked --test typed_diagnostics` pass with
  the pins the new code moves (AGENTS.md, "A new `ValidationCode` or diagnostic moves pins outside
  the crate that declares it"): the `validation_codes!` digest in `macro-guards.json` and the
  per-file counts in `docs/design/review-typed-diagnostics.md`.

No GitHub issue reports this case and no fixture carries such a literal (measured in wave 22).

## Scope

- cited: `crates/specify/ess-domain/src/command.rs` (`RawPayloadSource::Decimal`, `decimal_text`)
- cited: `crates/specify/ess-primitives/src/facts.rs` (`Number::parse_decimal`, `Repr::Exact` /
  `Repr::Binary64`)
- cited: `crates/specify/ess-primitives/src/error.rs` (the new `ValidationCode`)
- cited: `crates/specify/ess-compiler/src/resolve.rs` (`class_of` for the new code)
- inferred: `crates/specify/ess-yaml/src/lib.rs` (the literal's source spelling, `number`)
- inferred: `crates/specify/ess-domain/src/command/value_expression.rs`
- inferred: `crates/specify/ess-domain/tests/inexact_decimal_literal.rs`
- inferred: `website/docs/reference/diagnostics.md` (rewritten by `cargo xtask diagnostics`)
- cited: `crates/edge/ess-xtask/src/consumer_coverage/macro-guards.json`
- inferred: `docs/design/review-typed-diagnostics.md`

confidence: medium

Moves no shared surface.
