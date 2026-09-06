---
format: aep.planning-md/1
id: story:review-conformance-coverage
kind: story
status: active
title: Carry conformance coverage through persisted suites and evidence
tags:
- P0
- review-2026-09-05
relations:
- decomposes: epic:review-boundary-remediation
- serves: vision:O2
- depends_on: story:review-report-reader-validation
- depends_on: story:a-skipped-scenario-is-not-a-failed-one
- depends_on: story:review-conformance-format-design
scope:
- confidence: cited
  path: crates/edge/ess-cli
- confidence: cited
  path: crates/verify/ess-conformance
- confidence: cited
  path: crates/verify/ess-diff
- confidence: cited
  path: docs/design/review-conformance-coverage-transport.md
- confidence: cited
  path: docs/design/review-conformance-coverage.md
- confidence: cited
  path: docs/design/review-format-catalog.md
- confidence: inferred
  path: website/docs
- confidence: cited
  path: website/docs/guides/verify-conformance.md
- confidence: cited
  path: website/docs/reference/formats.md
revision: 21
---
## Finding and source

F03 (P0) from `docs/reviews/2026-09-05-architecture-review.md:217`, reviewed at `fd06a4d61bfb7b4990617810655dc181d6a3ab00`. Evidence locations: `crates/verify/ess-conformance/src/scenario.rs:1`, `crates/verify/ess-conformance/src/evidence.rs:18`, `crates/edge/ess-cli/src/main.rs:2210`, `crates/edge/ess-cli/src/main.rs:2319`. Findings are attributed to the review; proposed implementation choices below are coordinator inferences.

## Acceptance

After suite serialization and execution, the report reproduces the coverage classification required by the versioned exact-suite contract.

## Implementation boundary

Design the suite/report compatibility contract before coding: account for generated, authored, outside-component and refused obligations and distinguish execution status from source coverage. Incomplete, unknown or empty coverage never succeeds in strict mode. Explicit --allow-incomplete selects truthful diagnostic execution; combining it with --strict refuses before execution. Inventory Rust, generated Go, CLI and downstream readers; publish a migration order and immutable format versions when envelopes/meaning change. The existing skipped-count story owns splitting non-pass categories and must land first under that design.

## Validation

Round-trip complete, partial and zero-generated suites through JSON, YAML, files and Go output; prove strict refusal and explicit diagnostic execution of incomplete inventory, exact-suite digest mismatch rejection and old-version behavior. Use an unsupported but type-correct synthesis case so fixing F09 cannot erase the coverage regression.

Run package-scoped checks while implementing; the integration coordinator runs every step of `task check` and `task site-build` when required by `AGENTS.md`, retaining individual exit statuses and executed-case counts.

## Compatibility and exclusions

The first-wave v1 reader fix is independent; do not treat legacy zero-test evidence as proof of full coverage or retroactively as failed execution. Cross-repository rollout requires an Atlas ADR and separately scoped downstream work before release.

## Scope

Derived 2026-09-06 by aep-drive:story-scoper 0.8.0 against clean ESS 239996d846460aee342ce42514378c25b2be5152 and complete draft revision 17; current accepted coverage/transport bindings and the refreshed source/hash record govern — cited.

- **Primary surface:** crates/verify/ess-conformance — cited; existing suite/scenario admission, synthesis/authored inventories, immutable execution/count pairing, Go emission/runtime and generic browser projection/player own closed suite/5, input/1 and replay/1. Preserve legacy DTO construction, original suite1–4 bytes, count APIs, checked Result returns and supported execution vocabulary.
- **CLI surface:** crates/edge/ess-cli — cited; main.rs owns fresh/loaded conformance, source acquisition, output preflight and impact input. Add accepted explicit suite-format, suite-input and conform select controls, read-once root-relative sources and pairing before effects. Its separately integrated normalize.rs TypeScript route is a collision surface to preserve.
- **Impact surface:** crates/verify/ess-diff — cited; existing semantic invalidation owns admitted exact-input and in-memory selection context. Preserve ess-impact/3 with ess-diff/2 fields/bytes and conservative dependency obligations; refuse incomplete suite/5 or unavailable exact lineage without fallback.
- **Binding authority:** docs/design/review-conformance-coverage.md — cited; accepted outcome, inventory, byte/scalar, 75-row matrix and P1–P10 authority. Source-status clarification remains coordinator-owned; implementation does not rewrite the contract.
- **Transport authority:** docs/design/review-conformance-coverage-transport.md — cited; accepted original input/1 lineage, authored identities, paired replay/1, impact/3 and reader-first shipment rules, including model Binary64 refusal before inventory/output.
- **Internal format inventory:** docs/design/review-format-catalog.md — cited; update conformance/carrier/replay support after implementation while preserving the current checked synthesis, normalization and schema-resource distinctions.
- **Public guide:** website/docs/guides/verify-conformance.md — cited; extend current count-only instructions with explicit suite/5 production, original input/parents, selection and truthful report/2 pairing after implementation. No default or deployed-adopter transition is selected.
- **Public format reference:** website/docs/reference/formats.md — cited; add the implemented suite/5, input/1 and replay/1 support while preserving TypeScript normalization report/1–3, recipe/1–6, known floating-pair equality limits and adopter-owned schema IDs.
- **Public collision token:** website/docs — inferred; retain the existing literal parent reservation alongside the two concrete public leaves.
- **Execution and scalar limits:** exact u64 metadata, original inner/parent bytes, inherited nested defaults and finite Number meaning only at declared Node payload positions remain distinct. Model/shape Binary64 refusal must precede inventory/target effects; no invented authored identity/code or numeric normalization across unknown fields — cited.
- **Consumer prerequisite:** the coordinator reports AEP 658cf76e6371b1628f6de69548e724b52803f5c2 published with all three exact-source CI/bundle/docs runs successful. The repinned helper's frozen compiled inputs match that source, but it has not executed; actual frozen Rust/Go correspondence through both readers/replay remains required before ESS writer publication — cited.
- **Preservation boundary:** replay pairing does not close F15 browser fidelity, authenticate a publisher or prove inventory honesty. No ESS-to-AEP dependency, normalization equality repair, new impact envelope or conformance default movement is selected — cited.
- **No additional production reservation:** no changed owner establishes a coverage-writer edit in schema-contract, ess-gen, primitives/compiler, composition, realization, shared manifests/lock or Taskfile. New private modules and permanent Rust test harnesses fit the three reserved packages; report a concrete need before expansion — inferred.
- **Confidence: high** — cited; all prior production/test owners and both accepted bindings are byte-identical, and the two changed public/internal references preserve the same coverage boundary. Private names and exact fixture placement remain implementation choices.
- **Would collide with:** edits anywhere in the three package tokens, either binding, the catalog, either public leaf or website/docs parent token; notably CLI normalization work already shares the ess-cli package reservation. Store/generator/delivery scheduling remains coordinator-owned — inferred.

## Required classification checks

The matrix must detect missing generated/authored/outside/refused accounting after round-trip, language emission that drops that accounting, and evidence bound to a different suite digest. It must distinguish execution status from coverage, require strict non-success for incomplete/unknown/empty coverage, preserve explicit --allow-incomplete diagnostic execution, and refuse --strict combined with --allow-incomplete before execution, and retain the designed legacy-reader behavior. All component checks are required to satisfy the one versioned-contract classification result.

## Current preparation

Wave8 is closed atbe0eefd. Root drafted the separate transport proposal; AEP has a governed reader prerequisite, still draft. These are preparation records, not an accepted binding or a selected implementation wave. Complete the binding review and reader-first Atlas coordination before activating the ESS writer.

## Accepted binding decision — 2026-09-06

Under the standing ESS remediation implementation/publication authorization, root accepts the reviewed coverage/transport/AEP contracts and Atlas ADR0040 for implementation. This section supersedes earlier preparation wording that calls them proposals. Both immutable binding reviews are recorded; pass1 exact fact vocabulary and strict/diagnostic wording are corrected, and pass2 Text truthiness guidance was checked against the unchanged AEP facts/predicate owner. No implementation test or compatibility result is inferred from those document checks. No third binding review was opened.

- docs/design/review-conformance-coverage.md — SHA256 ef8028fce04685a4b5936c87bab37f4b05bf674ccbbb0d2fcb5d0dd55da00742.
- docs/design/review-conformance-coverage-transport.md — SHA256 2b06be326986a9f1dd76029d6802d9c63bc601bcaecce138828c10159f5e31c2.
- docs/design/ess-conformance-coverage-evidence.md — SHA256 d5ee4f43d8527134997eaf7fc2ca30e9f4922dba1df2bebd38779d98029db873.
- architecture/adr/0040-ess-complete-selection-evidence.md — SHA256 54931ffb2e52beb09e7ba7b952ec16f996a681161692227e5cdc25b1f3d337ac.

Original review reports are035d8b8fd665457e387a73edfd47fec9b40fbdc85bc6c93a774b48f3f4f8ada3 andf13642c04236967cd71ff28096abb1907aa8e1139687243546f3192c345b058a. Actual AEP source baseline package tests separately passed1320cases0failed/ignored61summaries; that is existing-source resource/control evidence, not new reader implementation. Reader publication remains required before the ESS writer, followed by frozen actual Rust/Go correspondence before ESS writer publication.

## Published Binary64 source refresh

The transport binding records published main6c6620b model/suite refusal before output. Suite/5 adds inventory without introducing a Binary64 codec. Preserve admission::model/suite and checked runner/serializer/projection Result returns. A model-level UnsupportedBinary64 authoring cause cannot be assigned an invented authored source identity or admitted as a new coverage refusal code. The reader prerequisite retains its accepted supported vocabulary; fresh writer source assumptions must be refreshed after this integration. This source-preservation clarification is accepted under the standing remediation instruction, with no new wire vocabulary or default change.

## Wave 11 prerequisite readback

The AEP coverage reader is published at 658cf76e6371b1628f6de69548e724b52803f5c2, with its closing records at 70ec336c00fa67b3d83f5c05bfbe71189462c01e. Joint public delivery and task-owned retirement are recorded in the closed wave 10 page. This supersedes the historical Current preparation paragraph. The refreshed scope report at 239996d is still source-applicable: later ESS commits through 1c7b38b change internal planning records only. Actual frozen Rust/Go correspondence through the published AEP readers remains required before the writer is published.
