---
format: aep.planning-md/1
id: story:review-conformance-coverage
kind: story
status: implemented
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
revision: 23
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

### Confirmed implementation scope

The implementor's confirmation table below establishes the three production packages and the
three concrete documentation files against its 65-file initial delta. Two source attacks and
root-reviewed corrections add permanent tests and the exact refusal oracle, for 70 integrated
unit files at d3c2c1dd09ad279f5bf6b4c1050b589a4a7ad761. Root owns both bindings, changelog and
engineering records. The nine scheduling tokens remain unchanged; website/docs is retained
as a coordination reservation beside its two actual public leaves.

| Scope hypothesis | Source-derived result and measurement |
|---|---|
| Separate typed inventory/document and admitted original input | Confirmed in coverage.rs (AdmittedInput, SuiteInputDocument, validate_parent) and admission.rs. Original admission-red/input-red and shared Rust/Go/Firefox lineage cases establish the boundaries; no unchecked mutable admitted escape. |
| Checked authored input layer and final inventory ownership | Confirmed in coverage_build.rs: CoverageSource, compile_sources, merge_batches, partition/classify. Actual duplicate and independently compiled batch controls establish ownership only after final merge; failed candidates never borrow an outside survivor's proof. |
| Coverage-bearing immutable execution identity | Existing ExecutedRun capability already binds admitted original bytes; it was reused, not replaced or reconstructed from legacy ConformanceReport. Count writer pass1/pass2 inherited guards pass unchanged. |
| CLI original source acquisition and pre-write admission | coverage.rs sources() and fresh/generate/select/web plus main.rs pair closure execute original input before output/target effects. Actual relocation, invalid path and Binary64 controls passed. |
| Go full lineage admission before reduced adapter | Confirmed by 95 shared vectors and actual callback/refusal controls. Initial exact-u64 parent admission exposed a separate Go int adaptation limit, resolved by root D7: complete exact wire/lineage first; selected fields only are converted with actual int-width checks. |
| Generic browser pairing before state | New replay/1 typed model and input retain exact selected/parent originals; actual Firefox executes both the new player and frozen old player. Original-byte integer, model and inventory guard mutations fail behaviorally. 314 recursive closed-model vectors refuse in both Rust and Firefox. |
| Exact impact context without new persisted format | New impact_input uses the immutable input/selection in memory. Existing ess-impact/3 fields and serialization remain. Raw suite5, unknown coverage and unavailable lineage are refused; existing conservative graph logic is reused. |
| No additional production reservation | Confirmed by source-audit.json: 19 modified and 46 new files all within the three packages or three named docs. No manifests/lock/Taskfile/compiler/primitives/realization/AEP changes. |
| website/docs collision token | Confirmed as coordination reservation only. Only guides/verify-conformance.md and reference/formats.md were edited there. Root retains delivery ownership. |
| Reader-first prerequisite | Supplied published AEP 658cf76 and closing70ec336 satisfy the opening prerequisite. This implementation never executed the AEP helper; actual frozen correspondence, workspace/site gates and publication remain root-owned. |

The table's final row states the implementation handoff boundary. Root subsequently executed
all ten integrated ESS gate lanes and all 43 actual Rust/generated Go correspondence cases
through published AEP readers. See verification-report:coverage-writer-actual-correspondence.
Its successful negative cases preserve empty, incomplete, unknown and mismatched classifications.
Current source is a333949e6581e151f2c3b154d7df30e125d07375; no default or release was changed.

### Opening scope hypotheses retained for traceability

The following is the source of the original reservations, not a claim that implementation or
correspondence remains unexecuted. Confirmations and updated evidence above supersede its
preparation-only statements.

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

## Published implementation and delivery — 2026-09-06 UTC

The full gate at a333949e6581e151f2c3b154d7df30e125d07375 passed all ten lanes and
2,058 Rust cases (zero failed or ignored). Actual frozen Rust/generated Go exports passed
43 published AEP compatibility cases and four separate no-report refusal controls.
verification-report:coverage-writer-actual-correspondence records the exact source, independent
pre-writer expectations, actual clock/callback evidence and all positive/negative classifications.
Both source attacks and the root's precise final test-scope correction remain recorded.

The exact source is published. GitHub CI 34062258911, documentation validation 34062258901 and
source bundle 34062258926 each completed successfully for that commit. Atlas publication
34062674853 completed successfully under b89e5b835b384965818eccf0703c8a548ebdbe47, selecting
ESS a333949 and published AEP reader 658cf76. Its source-set SHA256 is
 a77180d60bb40648c4a99f9b9824b98c3379983fd7c2854a6e053f32db0625d2.
The independent Website verifier checked 356 routes and 1,328 site files; the complete Website
gate passed 99 tests, zero failed, skipped or cancelled, in 139.979430589 seconds. All 1,709
publication artifact files remained unchanged. At 22:07:53–22:07:54 UTC, both live provenance
endpoints, the two changed ESS pages and the AEP CLI page matched the exact artifact bytes.
This is five actual fetches, not a download of every production file.

The full Atlas fence on the exact b89e5b authority passed 149 Rust cases and retained its three
previously reported workspace failures: primary AgentIDE v4 collector compatibility, the primary
Website Docs System pin, and missing Widgets Serves. That full fence exited 1; the successful
Atlas publication and Website gates above are separate observations. No organization-wide green
fence, installed-reader upgrade, publisher authentication or default readiness is claimed.

All unit evidence except the reproducible target/debug cache has been archived with independent
readback: 91,516 entries, 73,807 regular payloads, 17,407 directories and 302 literal links.
The archive SHA256 is 3d73c167b351ffcd33ff8a7c923ce683301ffbdbf722fd220c6dcecbdeda9b52;
1,304,478,062 compressed bytes retain 4,580,555,866 original file bytes. All 3,167 excluded compiler
entries are inventoried. The six assigned external browser roots are separately archived and
verified: 7,357 entries, 4,462 files and 85 literal links, SHA256
5d8161f388116ac78a1a57045a446f98b1d5ca0168e1ff9ce21d05662249497a.
The cache directory is coverage-writer-unit-retirement under the session's 2026-09-06-resume
retention root. Managed retirement remains a separate step after recording this implementation.

Suite4/report1 defaults remain. Browser-fidelity F15 and the execution-recovery implementation
obligation remain open. Source and delivery completion do not select a release, tag or installation.
