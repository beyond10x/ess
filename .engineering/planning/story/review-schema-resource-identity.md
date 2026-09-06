---
format: aep.planning-md/1
id: story:review-schema-resource-identity
kind: story
status: active
title: Define the boundary between generated schemas and registry resources
tags:
- P2
- review-2026-09-05
relations:
- decomposes: epic:review-boundary-remediation
- serves: vision:O2
- depends_on: story:review-format-catalog
scope:
- confidence: inferred
  path: crates/edge/ess-cli/tests/fixtures/schema-resource-identity
- confidence: inferred
  path: crates/edge/ess-cli/tests/schema_registry_identity.rs
- confidence: cited
  path: docs/design/review-format-catalog.md
- confidence: inferred
  path: docs/design/review-schema-resource-identity.md
- confidence: cited
  path: website/docs/guides/generate-artifacts.md
- confidence: cited
  path: website/docs/reference/formats.md
revision: 11
---
## Finding and source

F13 (P2) from `docs/reviews/2026-09-05-architecture-review.md:449`, reviewed at `fd06a4d61bfb7b4990617810655dc181d6a3ab00`. Evidence locations: `crates/edge/ess-xtask/src/main.rs:423`, `crates/generate/ess-gen/src/schema.rs:154`, `crates/generate/schema-contract/src/typescript.rs:69`. Findings are attributed to the review; proposed implementation choices below are coordinator inferences.

## Acceptance

The schema workflow documents an executable supported path from generated syntax/contract schemas to adopter-owned registry entries without implying an existing stable schema endpoint.

## Implementation boundary

Design/document the current boundary first: syntax schema is not whole-system semantic validation, self-contained generated contracts are not automatically registry resources. Decide whether a concrete consumer needs logical immutable IDs or digest identities; record owner, collision rules, offline resolution and publication prerequisites before adding generated $id values. Provide a worked local registry example using an adopter-chosen absolute ID if that is the supported path.

## Validation

Validate the worked example with the existing schema registry CLI, including missing-id refusal and offline resolution. Mark any proposed organization namespace as unshipped.

Run package-scoped checks while implementing; the integration coordinator runs every step of `task check` and `task site-build` when required by `AGENTS.md`, retaining individual exit statuses and executed-case counts.

## Compatibility and exclusions

No new public schema service or silent changes to released schemas; any automated ID projection is follow-on work only after ownership is decided.

## Scope

Derived 2026-09-06 by aep-drive:story-scoper 0.8.0 from the complete draft revision 3 and the assigned reconciled ESS source 4777c1de3a80ea7645a255e4e229ad30a8a5cc8e — cited.

- **Primary executable surface:** `crates/edge/ess-cli/tests/schema_registry_identity.rs` — inferred; new actual-CLI integration cases proving generated-source missing-ID refusal, adopter-owned resource copies, strict selector envelopes, exact root-ID lookup, offline references, syntax versus semantic validation and existing output-refusal behavior.
- **Fixture surface:** `crates/edge/ess-cli/tests/fixtures/schema-resource-identity` — inferred; new small adopter envelope/instance/source fixtures, with payload-resource copies derived at test runtime from the existing frozen generated schemas in assigned scratch.
- **Binding document:** `docs/design/review-schema-resource-identity.md` — inferred; the story already reserves this uncreated page; record adopter ownership, local immutable-ID choice, collision and publication limits, distinct byte identities, and the accepted worked representation before implementation dispatch.
- **Internal catalog correction:** `docs/design/review-format-catalog.md` — cited; line 148 currently lists `$id` as an identity carried by ESS-generated contract schemas, although the unchanged producer and generated source have no such root member.
- **Public worked workflow:** `website/docs/guides/generate-artifacts.md` — cited; lines 22–45 own generated contract output/provenance, while lines 363–412 and 466–564 now document separate normalization/model contracts that the new local registry example must preserve.
- **Public identity reference:** `website/docs/reference/formats.md` — cited; line 180 still gives generated contract schemas a `$id`; lines 184 and 201 distinguish syntax schemas and adopter resource identity, and lines 95–118 define adjacent bundle/model/normalization identities that must remain intact.
- **Implementation character:** documentation plus actual CLI regression/example tests; the inspected existing registry and projection APIs support the proposed representation without a new source API — inferred.
- **Compatibility boundary:** no generated `$id`, automatic namespace, format or flag; no rewriting frozen schemas, generator output, normalization version rules, native codecs, legacy registry vocabulary, resolver dependencies or public navigation — inferred.
- **Confidence:** high — cited; the target story, unchanged registry/generator sources, exact frozen fixtures, CLI argument definitions and current documentation identify the owners directly; the three new path names remain proposed reservations.
- **Would collide with:** writers of the generation guide, either format catalog, the new binding page, the CLI integration test/fixture paths, or broader reservations enclosing those files; serialize these concrete surfaces during later wave selection — inferred.
- **Shared ownership:** the coordinator retains planning records, accepted binding decisions, wave selection, changelog, integration gates and publication; this scope does not authorize editing those shared records — inferred.

## Accepted implementation decision

Coordinator decision for wave10: use the adopter-owned root-ID copy plus separate strict selector envelope described in docs/design/review-schema-resource-identity.md. This interprets the existing story and changes no generated bytes, registry API or production identity. Source scope refresh report SHA256 a623c9ce9fd7047ea7f6dacdf34978030264b5a676c81557277be1dc3eb6277b is retained in target/review-boundaries-9/schema-preparation/refresh-4777. Actual paired CLI success remains an implementation requirement, not an observation made by the scoper.
