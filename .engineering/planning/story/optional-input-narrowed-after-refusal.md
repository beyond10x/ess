---
format: aep.planning-md/3
id: story:optional-input-narrowed-after-refusal
kind: story
status: active
title: An Optional input is narrowed after an outcome refuses its absence
refs:
- provider: github
  reference: beyond10x/ess#169
relations:
- decomposes: epic:retrofit-findings-round-3
- serves: vision:O2
scope:
- confidence: cited
  path: crates/edge/ess-xtask/src/docs.rs
- confidence: inferred
  path: crates/generate/ess-entity-runtime/src/lib.rs
- confidence: inferred
  path: crates/specify/ess-compiler/src/ir.rs
- confidence: cited
  path: crates/specify/ess-compiler/src/resolve.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/command/value_expression.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/primitive_admission.rs
- confidence: cited
  path: crates/specify/ess-domain/src/system.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/witness.rs
- confidence: cited
  path: models/toolchain/domains/specify.yaml
- confidence: cited
  path: website/docs/reference/formats.md
- confidence: cited
  path: website/docs/reference/spec-versions.md
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-09-27T20:56:53Z", actor: "human:timo", revision: 5, imported: true}
- {from: "proposed", to: "active", at: "2026-09-27T20:58:31Z", actor: "human:timo", revision: 6, imported: true}
---
## Scope

The type checker reads an earlier `when: not defined(x)` refusing branch and treats `x` as `T` in later branches, under a new source format.

## Acceptance

The #169 repro validates without a conversion; a scenario sends the absent input and requires the refusal.

## Derived scope

Derived 2026-09-27 by `story-scoper`. Every line is **cited** or **inferred**.

- **Primary surface:** `crates/specify/ess-domain` command type checker — cited
- **Refusal sites:** `command.rs` `check_payload_entry` (2712, refusal 2769), `validate_sets` (2917, refusal 3020); narrowing reads sibling outcome conditions (`is_unconditional`/`default_outcome` 1743, 1908) — cited
- **Compiler re-check:** `ess-compiler/src/resolve.rs` `crossing` (2329, ESS-COMMAND-002 at 2349), `payload_field` (2044), `sets` (1860) — cited: narrowing only in ess-domain still ends in ESS-COMMAND-002
- **Also likely:** `command/value_expression.rs` (599), `ess-compiler/src/ir.rs` `InputField.type_ref` (897), `synthesize.rs`/`witness.rs` (check witnesses), `ess-entity-runtime/src/lib.rs` (unwrap in narrowed branch) — inferred
- **Format gate:** source-format bump (`system.rs`, `primitive_admission.rs`, `ess-xtask/src/docs.rs`, `models/toolchain/domains/specify.yaml`, `ess.schema.json`, reference pages, `reviewed-schema-metadata.json`) — cited pattern, number inferred
- **Documents:** a new design note (none exists) — inferred
- **Confidence:** medium
- **Would collide with:** `command.rs` payload/sets typing; `resolve.rs` payload resolution; source-format bump
