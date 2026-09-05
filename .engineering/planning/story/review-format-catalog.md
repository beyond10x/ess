---
format: aep.planning-md/1
id: story:review-format-catalog
kind: story
status: draft
title: Catalog format identities and canonical byte contracts
tags:
- P1
- review-2026-09-05
relations:
- decomposes: epic:review-boundary-remediation
- serves: vision:O2
scope:
- confidence: inferred
  path: docs/design/review-format-catalog.md
- confidence: inferred
  path: website/docs
- confidence: inferred
  path: website/docs/reference/formats.md
- confidence: cited
  path: website/sidebars.ts
revision: 6
---
## Finding and source

F13 (P1) from `docs/reviews/2026-09-05-architecture-review.md:449`, reviewed at `fd06a4d61bfb7b4990617810655dc181d6a3ab00`. Evidence locations: `crates/specify/ess-composition/src/lib.rs:277`, `crates/specify/ess-composition/src/lib.rs:548`, `crates/specify/ess-compiler/src/ir.rs:1569`, `crates/generate/ess-deployment/src/identity.rs:128`. Findings are attributed to the review; proposed implementation choices below are coordinator inferences.

## Acceptance

Every currently persisted ESS format has a cited catalog entry separating its discriminator, semantic/release version, validation level and canonical digest profile.

## Implementation boundary

Inventory actual producers and readers, supported versions and strictness, including differing composition input/output shapes and realization type discriminator. Name source_digest accurately in prose as a compiled-model digest. Describe existing canonical bytes and inventory relying parties; do not change bytes merely for uniform naming.

## Validation

Cross-check the inventory against format constants/Serde envelopes and readers; canonical fixture examples identify compact versus pretty-plus-newline hashing. Each proposed successor names migration prerequisites and unknowns instead of presenting a new endpoint as shipped.

Run package-scoped checks while implementing; the integration coordinator runs every step of `task check` and `task site-build` when required by `AGENTS.md`, retaining individual exit statuses and executed-case counts.

## Compatibility and exclusions

Schema resource identity has a separate design; no ess-ir/2 or mass format rename.

## Scope

Derived 2026-09-05 by independent aep-drive:story-scoper reading story, instructions and source; coordinator corrects the publication interpretation below. No source, store or build writes occurred during scoping.

- docs/design/review-format-catalog.md — inferred; new engineering catalog reserved by story.
- website/docs/reference/formats.md — inferred; no current format catalog, narrow Reference landing page.
- website/sidebars.ts — cited; explicit Reference category needs the entry.
- website/docs — inferred conservative scheduling token retained while other public stories use this shared broad token; exact-token collision detection does not understand parent/child paths. Revisit after their independent scopes are grounded.
- Documentation-only: describe current identities, readers and migration prerequisites; change no producer, reader, discriminator, schema or canonical byte — cited.
- Distinguish format discriminator/support, numeric semantic system version versus vN text, release SemVer, serialization profile, hash input and validation stage; separate authored/compiled and unversioned/versioned shapes — cited.
- ess-domain system.rs:60–152 has format ess/1 with syntax versus support checks. ess-compiler ir.rs:1253 persists unversioned EssIr: do not invent ess-ir/1. name.rs:233–305 Version serializes as number, while browser/projection strings can spell vN — cited.
- ess-composition lib.rs:277–338,548–581 uses ess-composition/1 for authored service-array and compiled resolved-service-map shapes. Authored closed decoding precedes compile semantic checks (:1122); compiled output is Serialize-only. EssClientPlan (:686) separately uses ess-client-plan/1 — cited.
- ess-realization lib.rs:397–463 uses type, not format, for ess-realization/1 and ess-realization-ir/1. Closed authored reads precede compile discriminator/exact-ESS identity checks (:689–714); IR Serialize-only — cited.
- Delivery families: ess-build/1, ess-build-ir/1, ess-runtime/1, ess-runtime-ir/1, ess-component/1, ess-component-ir/1, ess-release/1, ess-release-bundle/1, ess-release-catalog/1, ess-stack/1, ess-stack-lock/1, ess-environment/1, ess-deployment/1. CLI main.rs:1589 separately emits ess-deployment-diff/1 — cited constants in deployment build/runtime/component/release/stack/environment.
- Reviewed delivery f1baa9051be7d6cfc48ec1dcd302d0c87ac21a15 adds checked generic Serde routes for BuildIr, RuntimeIr, ComponentIr, ReleaseManifest, ReleaseBundle, ReleaseCatalog, StackLock, DeploymentIr (validation.rs:37–54), nested map uniqueness, recoverable graph/order and available digest relations. CLI mutable desired/current checks precede analysis/executors. Keep authored DTO parsing separate and do not claim absent ESS/realization authority, remote OCI verification or trust. Publication was not established by the scoper — cited.
- Authored conformance uses type ess-scenario/1 (authored.rs:222–256). Suite discriminator is provenance.suite_version, current4/support1–4; syntax read is not execution admission. Standalone uses ess-conformance-report/1; detailed CLI JSON/YAML is currently unversioned. Proposed suite5/report2/run2 remain future contracts — cited.
- Delta ess-diff/1 has closed raw decode then validated format/derived identity/order conversion (delta.rs:13–35, raw.rs:43–135). Current impact producer labels ess-impact/2 (impact.rs:96), without a corresponding report reader — cited.
- ess-docs/1 has derived read plus separate page-ID validation, not uniform strict format admission (ess-gen document.rs:55–88); browser catalog writes ess-browser-catalog/1 (web/catalog.rs:38–77) — cited.
- ess-service-interface/1 has closed decoding plus separate validate; source_openapi and service.version are distinct. Generated JSON Schema/OpenAPI/AsyncAPI retain external dialect identities; schema-resource identity is another story (ess-openapi lib.rs:12–78) — cited.
- Infrastructure families: infra-observation/1, infra-ir/1, infra-spec/1, infra-drift/1, infra-simulation/1, infra-graph/1, infra-projection/1. Raw observations tolerate evolving collected data; compiler read.rs:55–123 instead checks IR format, closed mirrors, model digest and references — cited.
- plan.json and target.json are unversioned (ess-synth plan.rs:74,446; lib.rs:65–75,129,186). Generated Rust/Go HTTP startup records also label ess/1 for a different shape (rust/http.rs:47, go/http.rs:49); classify by producing surface, not discriminator text alone — cited.
- Compiled-model source_digest hashes compact typed JSON without LF, bare64hex (ir.rs:1575). Contract slice hashes another compact document (provenance.rs:272–356). Delivery digest hashes pretty JSON+LF with sha256 prefix (identity.rs:135–139). Realization hashes compact specification/synthesis/implementation tuple, not rendered IR (lib.rs:1047–1054). Infra model hashes sorted compact model JSON excluding observation provenance (ir.rs:562–570). Proposed sha256-json-bytes/1 binds actual suite bytes and is not current — cited.
- Known consumers: CLI generic reads, Rust/Go/browser conformance, generated clients, provenance checks, impact and delivery executors. Available AEP00c742e4179593738a2e8aa69e2ecc07d3c89402 evidence names both aep-ess-evidence::adapt_json and aep-cli::recorded_from_report and closed result/predicate boundary; these are inspected source pins, not fresh deployed-version proof. Realization suite/report digests are references, not established verifying readers — cited.
- No complete external/deployed inventory or executed canonical/compatibility fixture was established. Document-only unit needs source/matrix/whitespace checks, with full gate/site build owned by coordinator — inferred.
- Publication correction: the chosen public file is inside the existing allowlist, so no allowlist edit is needed. Workspace AGENTS nevertheless requires ESS source publication first, Website deterministic source-lock refresh, Atlas snapshot rendering and both delivery gates. Those cross-repository actions are coordinator-owned managed work, not waived by this narrow ESS edit scope — cited instruction, correcting scoper interpretation.
- Confidence high for documentation edit boundary and inspected format owners; collision includes catalog, reference page, sidebar and conservative shared website token. No implementation scheduled by this scope — cited/inferred.
