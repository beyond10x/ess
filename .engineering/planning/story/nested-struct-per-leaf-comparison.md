---
format: aep.planning-md/3
id: story:nested-struct-per-leaf-comparison
kind: story
status: implemented
title: A nested struct target is compared leaf by leaf
refs:
- provider: github
  reference: beyond10x/ess#179
relations:
- decomposes: epic:retrofit-findings-round-3
- serves: vision:O2
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: inferred
  path: crates/edge/ess-xtask/src/docs.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/admission.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/coverage.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/coverage_build.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/lib.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/runner.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/scenario.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/value_expressions.rs
- confidence: cited
  path: docs/design/value-expressions.md
- confidence: inferred
  path: website/docs/reference/formats.md
- confidence: inferred
  path: website/docs/reference/spec-versions.md
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-09-27T20:39:19Z", actor: "human:timo", revision: 5, imported: true}
- {from: "proposed", to: "active", at: "2026-09-27T20:40:23Z", actor: "human:timo", revision: 6, imported: true}
- {from: "active", to: "implemented", at: "2026-09-28T04:35:20Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"test_result":1}}}
---
## Scope

A nested `sets:`/payload mapping with one `{generated: true}` leaf still asserts every determined leaf, and presence and type for the generated one. The suite-format step that `value-expressions.md` E5 defers.

## Acceptance

The #179 shape (four input leaves, one generated) yields a scenario that fails when an implementation drops `lead.number`.

## Derived scope

Derived 2026-09-27 by `story-scoper`. Every line is **cited** or **inferred**.

- **Primary surface:** `crates/verify/ess-conformance` — cited (E5 in `docs/design/value-expressions.md`)
- **Files:** `crates/verify/ess-conformance/src/synthesize.rs` `expression_value` (~3990), its `ResolvedPayloadValue::Struct` arm (~4017) returns `None` when one leaf is undetermined; callers `determined_payload` (~3636), `settled` (~4100) — cited
- **Files:** `crates/verify/ess-conformance/src/runner.rs` `expect_payload` (~1463) must accept a partial struct or dotted per-leaf keys — inferred
- **New suite format (next free major, ordinary + coverage):** `scenario.rs` `SUPPORTED_SUITE_FORMATS` (391), `select_fresh_format` (156); a construct module like `presence.rs`; `admission.rs` `construct_formats` (501); `coverage_build.rs` `coverage_version` (477); `coverage.rs` `is_coverage_version` (17); `ess-xtask/src/docs.rs` `FORMAT_RELEASES` — inferred
- **Go/TS runtimes:** untouched if they refuse the new major by version, as for /22–/25 — inferred
- **Tests:** `crates/verify/ess-conformance/tests/value_expressions.rs` — inferred
- **Documents:** `docs/design/value-expressions.md` E5 — cited; `website/docs/reference/formats.md`, `spec-versions.md`, `CHANGELOG.md` — inferred
- **Source format:** unchanged — inferred
- **Confidence:** medium (partial map vs dotted keys not chosen)
- **Would collide with:** any unit adding a suite major; any unit editing `synthesize.rs` `expression_value`/`determined_payload`/`settled`; `runner.rs` payload comparison
