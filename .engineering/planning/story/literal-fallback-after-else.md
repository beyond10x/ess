---
format: aep.planning-md/3
id: story:literal-fallback-after-else
kind: story
status: active
title: 'A payload or sets: fallback after else: can be a literal'
refs:
- provider: github
  reference: beyond10x/ess#163
relations:
- decomposes: epic:retrofit-findings-round-3
- serves: vision:O2
scope:
- confidence: cited
  path: CHANGELOG.md
- confidence: inferred
  path: crates/edge/ess-xtask/src/consumer_coverage/wire.rs
- confidence: cited
  path: crates/edge/ess-xtask/src/docs.rs
- confidence: inferred
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
  path: crates/specify/ess-domain/src/system.rs
- confidence: inferred
  path: crates/specify/ess-domain/tests/value_expressions.rs
- confidence: inferred
  path: crates/specify/ess-service-contract/src/lib.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/web.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/value_expressions.rs
- confidence: inferred
  path: crates/verify/ess-diff/src/diff.rs
- confidence: cited
  path: docs/design/value-expressions.md
- confidence: cited
  path: models/toolchain/domains/specify.yaml
- confidence: cited
  path: website/docs/reference/formats.md
- confidence: cited
  path: website/docs/reference/spec-versions.md
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-09-27T20:28:31Z", actor: "human:timo", revision: 5, imported: true}
- {from: "proposed", to: "active", at: "2026-09-27T20:29:34Z", actor: "human:timo", revision: 6, imported: true}
---
## Scope

`{input: f, else: <literal>}` in a payload or `sets:` value, under a new source format. Extends E4 of `docs/design/value-expressions.md`, which admits only `{generated: true}` after `else:`.

## Acceptance

A scenario that omits the optional input asserts the literal; a mutant storing a different default is killed; the literal is type-checked against the target field.

## Derived scope

Derived 2026-09-27 by `story-scoper`. Every line is **cited** (read from the story or the tree) or
**inferred** (a reading that could be wrong).

- **Primary surface:** `crates/specify/ess-domain` (value-expression parsing and checking) — cited
- **Files:** `crates/specify/ess-domain/src/command.rs` — cited: the only `else` parse arm refuses anything but `{generated: true}`; `PayloadSource::InputOrGenerated`, `sets_literal`/`literal_representation`
- **Files:** `crates/specify/ess-domain/src/command/value_expression.rs` — cited: ess/14 gate and the `{input, else}` type check
- **Files:** `crates/specify/ess-compiler/src/ir.rs`, `crates/specify/ess-compiler/src/resolve.rs` — cited: `ResolvedPayloadValue::InputOrGenerated`
- **Files:** `crates/verify/ess-conformance/src/synthesize.rs` — cited: the `InputOrGenerated` arm returns `None` when the input is omitted; must assert the literal
- **Also likely:** exhaustive matches on `ResolvedPayloadValue` in `ess-entity-runtime/src/lib.rs`, `ess-diff/src/diff.rs`, `ess-service-contract/src/lib.rs`, `ess-conformance/src/web.rs` — inferred, only with a new IR variant
- **Also likely:** `crates/specify/ess-domain/tests/value_expressions.rs`, `crates/verify/ess-conformance/tests/value_expressions.rs`, `crates/edge/ess-xtask/src/consumer_coverage/wire.rs` — inferred
- **Documents:** `docs/design/value-expressions.md` (E4) — cited
- **Shared format registries (source format bump):** `crates/specify/ess-domain/src/system.rs`, `crates/edge/ess-xtask/src/docs.rs`, `models/toolchain/domains/specify.yaml`, `website/docs/reference/formats.md`, `website/docs/reference/spec-versions.md`, `CHANGELOG.md` — cited
- **Confidence:** medium
- **Would collide with:** any unit bumping the source format; any unit in `command.rs` payload-source parsing or `value_expression.rs`
