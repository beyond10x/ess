---
format: aep.planning-md/2
id: story:related-record-value-source
kind: story
status: active
title: 'A payload or sets: value can read a field of a record the subject references'
refs:
- provider: github
  reference: beyond10x/ess#166
relations:
- serves: vision:O2
- decomposes: epic:retrofit-findings-round-3
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: inferred
  path: crates/edge/ess-xtask/src/consumer_coverage/entry-classifications.json
- confidence: inferred
  path: crates/edge/ess-xtask/src/docs.rs
- confidence: cited
  path: crates/generate/ess-entity-runtime/src/lib.rs
- confidence: cited
  path: crates/specify/ess-compiler/src/ir.rs
- confidence: cited
  path: crates/specify/ess-compiler/src/resolve.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/value_expression.rs
- confidence: cited
  path: crates/specify/ess-domain/src/entity.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/system.rs
- confidence: inferred
  path: crates/specify/ess-domain/tests/value_expressions.rs
- confidence: cited
  path: crates/specify/ess-service-contract/src/lib.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/web.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/value_expressions.rs
- confidence: cited
  path: crates/verify/ess-diff/src/diff.rs
- confidence: cited
  path: docs/design/value-expressions.md
- confidence: inferred
  path: website/docs/reference/formats.md
- confidence: inferred
  path: website/docs/reference/spec-versions.md
revision: 6
---
## Scope

A value source through a declared `relations:` reference of the subject (e.g. the shipment's customer's region). Listed under *Not in this design* in `value-expressions.md`.

## Acceptance

The #166 repro validates; the scenario arranges two customers and fails an implementation that emits the wrong customer's region.

## Derived scope

Derived 2026-09-27 by `story-scoper`. Every line is **cited** or **inferred**.

- **Primary surface:** `crates/specify/ess-domain` + `crates/specify/ess-compiler` — cited
- **Source parse:** `ess-domain/src/command.rs` `PayloadSource` (852), `SOURCE_KEYWORDS` (1053), `ExplicitPayloadSource` (1142), `{subject:}` refusal (1337) — cited
- **Source typing:** `ess-domain/src/command/value_expression.rs` `check` (113), `check_subject` (201), `existing_subject_field` (226) — cited
- **Relations:** `ess-domain/src/entity.rs` `RelationSpec` (691), `RelationKind::References` (616) — cited
- **IR/resolve:** `ess-compiler/src/ir.rs` `ResolvedPayloadValue` (919), `resolve.rs` (2170, 3963) — cited
- **Synthesis:** `ess-conformance/src/synthesize.rs` `arrange_owner` (2792), `arrange_first` (2531), `expression_value` (3990) — cited
- **Entity Runtime refusal:** `ess-entity-runtime/src/lib.rs` `ValueExpressionUnsupported` (2340), `is_value_expression` (3166) — cited
- **Exhaustive matches:** `ess-service-contract/src/lib.rs`, `ess-conformance/src/web.rs`, `ess-diff/src/diff.rs` — cited
- **Shared registries:** source-format bump (`system.rs`, `ess-xtask/src/docs.rs`, `consumer_coverage/entry-classifications.json`, `schemas/generated/ess.schema.json`, reference pages, `CHANGELOG.md`) — inferred
- **Open:** spelling (`relations:` reference vs `{via, entity, field}`) decides whether `entity.rs` changes
- **Confidence:** high (domain, compiler, synthesis), medium (registries)
- **Would collide with:** any unit adding a payload source variant or keyword; `synthesize.rs` arrangement; source-format bump
