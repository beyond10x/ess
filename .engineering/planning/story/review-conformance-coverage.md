---
format: aep.planning-md/1
id: story:review-conformance-coverage
kind: story
status: draft
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
- confidence: inferred
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
revision: 17
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

Refreshed 2026-09-06 by story-scoper at ESS be0eefd7ec125d46bb3b664c4b95b8d638a2b1fe, story revision6; coordinator incorporates the separate proposed transport design — cited.

- crates/verify/ess-conformance — cited; admission.rs:51 retains original bytes/view/digest, runner.rs:258 and :347 bind actual execution, counts.rs:125 checks it. Preserve the immutable capabilities and historical unadmitted DTO. Own complete synthesis/authored inventory, source ownership, report/run coverage, Go and browser admission.
- Suite/5 representation — inferred; keep a separate closed document/builder and private admitted coverage rather than changing legacy DTO serialization. Preserve full parent scenario bodies/dependencies, knowledge, every refusal occurrence and source map.
- Authored producers — cited; synthesize.rs:197,923,981 and authored.rs:231,715,1439 own candidates and refusals. Source.origin is display text; stable input identity must precede new duplicate ownership. Preserve old APIs and all known identities.
- Go — cited; go/mod.rs:49 emits legacy suites; runtime.go:504 admits before target creation, :1945 permits only1–4 and :2591 emits unknown coverage. Its :2634 digest handoff must retain the inner original suite when the new carrier is embedded. Keep terminal/no-destination/teardown guards.
- Browser — cited; web.rs:54,75 and player.js:28 currently emit/read an unversioned reduced model before admission. Add explicit paired typed input and actual browser rejection tests. Coverage pairing does not close the separate replay-fidelity finding.
- crates/edge/ess-cli — cited; main.rs:2018 owns impact loading, :2436 execution admission, :2525 synthesis, :2637 web, :2676 authoring and :2737 source acquisition. The v5 path must retain presently discarded refusals and merged candidate dispositions. Add explicit format/carrier/filter controls without changing defaults.
- crates/verify/ess-diff — cited; impact.rs:766 consumes a DTO and EssImpact:700 carries existing version3 provenance/invalidation. Add admitted input and in-memory selection context; preserve the persisted envelope. The proposed binding selects operation refusal for incomplete v5 coverage instead of adding a new WholeAnswer spelling under3.
- docs/design/review-conformance-coverage.md — cited; accepted inventory, multiplicity, identity and matrix authority, with stale current-impact2 citations to correct after review.
- docs/design/review-conformance-coverage-transport.md — inferred; proposed separate input/1 carrier, stable identities, browser pairing, CLI routing and impact clarification; not yet accepted.
- website/docs/guides/verify-conformance.md — cited; current lines64–69 say suite5 unavailable.
- website/docs/reference/formats.md — cited; current lines205–206 describe count-only coverage and the future successor.
- docs/design/review-format-catalog.md — cited; lines288–295 record count-only behavior and reserved suite5.
- website/docs — inferred; retain the exact parent collision token for public instructions.
- Tests — cited; conformance count_reports/count_writer_pass1/count_writer_pass2/synthesis/authored, CLI count_reports/go_conformance/count_writer_pass1/count_writer_pass2/authored_scenarios and diff impact are controls. Add actual lineage/source/browser cases; emitted-source assertions and billing/WASM are not generic-browser execution.
- Cross-repository prerequisite — cited; separately governed AEP story:admit-ess-conformance-coverage and a new current Atlas ADR must precede writer publication. Installed/generated adoption remains distinct; no defaults or releases are selected.
- No extra production reservation established — inferred; shared primitives/compiler, ess-gen, normalization, realization, Cargo.lock and Taskfile remain outside the proposed implementation unless a measured mechanism requires expansion.
- Confidence: medium — inferred; the three production owners and count capabilities are established; transport/source/browser decisions are proposed and untested.
- Would collide with: the three package tokens, two binding paths, catalog, cited guides and website/docs parent token — inferred. AEP scope matching is literal; keep those directory tokens.

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
