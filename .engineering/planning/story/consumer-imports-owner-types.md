---
format: aep.planning-md/3
id: story:consumer-imports-owner-types
kind: story
status: implemented
title: A consumer specification can reference an imported component's types
refs:
- provider: github
  reference: beyond10x/ess#162
relations:
- decomposes: epic:retrofit-findings-round-3
- serves: vision:O2
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: inferred
  path: crates/edge/ess-xtask/src/docs.rs
- confidence: cited
  path: crates/specify/ess-composition/src/lib.rs
- confidence: inferred
  path: crates/specify/ess-composition/tests/composition.rs
- confidence: inferred
  path: docs/design/review-format-catalog.md
- confidence: inferred
  path: website/docs/reference/formats.md
- confidence: inferred
  path: website/docs/reference/spec-versions.md
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-09-27T19:39:04Z", actor: "human:timo", revision: 5, imported: true}
- {from: "proposed", to: "active", at: "2026-09-27T19:42:14Z", actor: "human:timo", revision: 6, imported: true}
- {from: "active", to: "implemented", at: "2026-09-28T04:35:25Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"test_result":1}}}
---
## Scope

Name another component's declared type through `composition.yaml` or a `requires:` entry, or assert that a local type conforms to an imported one.

## Acceptance

The #162 case validates with no local copy of the owner's type, or with a conformance assertion that refuses a field drift.

## Derived scope

Derived 2026-09-27 by `story-scoper`. Every line is **cited** or **inferred**.

- **Primary surface:** `crates/specify/ess-composition` — cited (refusal `reference_outside_component`, `src/lib.rs:1438`)
- **Chosen shape (smallest):** option 3, a composition-level assertion that a local type conforms to an imported one; `compile` (`lib.rs:1123`) already holds every service IR, field walk exists (`lib.rs:1379-1392`) — cited
- **Why #162 is refused:** `exports` → `EssSemanticRef::Type` (`lib.rs:545`) only knows types reached through commands, events, errors, views (`lib.rs:1336-1350`); `ResolvedDomain.types` (`ess-compiler/src/ir.rs:1806`) lists all — cited
- **Also likely:** new `CompositionCode` for field drift with required-to-optional tolerance; `tests/composition.rs`; format `ess-composition/2` (`deny_unknown_fields` on `CompositionSpec`, `lib.rs:283`) with `FORMAT_RELEASES`, `formats.md`, `spec-versions.md`, `docs/design/review-format-catalog.md` — inferred
- **Not touched under option 3:** `ess-compiler/src/resolve.rs`, `ess-cli/src/input_discovery.rs` — cited
- **Open:** whether `ClientServicePlan::types` (`lib.rs:695`) bytes change if the type set widens
- **Confidence:** medium
- **Would collide with:** units in `ess-composition/src/lib.rs`, `FORMAT_RELEASES`, reference pages
