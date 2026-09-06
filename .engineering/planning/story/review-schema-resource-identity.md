---
format: aep.planning-md/1
id: story:review-schema-resource-identity
kind: story
status: implemented
title: Define the boundary between generated schemas and registry resources
tags:
- P2
- review-2026-09-05
relations:
- decomposes: epic:review-boundary-remediation
- serves: vision:O2
- depends_on: story:review-format-catalog
scope:
- confidence: cited
  path: CHANGELOG.md
- confidence: inferred
  path: crates/edge/ess-cli/tests/fixtures/schema-resource-identity
- confidence: inferred
  path: crates/edge/ess-cli/tests/schema_registry_identity.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/schema_registry_identity_adversary.rs
- confidence: cited
  path: docs/design/review-format-catalog.md
- confidence: inferred
  path: docs/design/review-schema-resource-identity.md
- confidence: cited
  path: website/docs/guides/generate-artifacts.md
- confidence: cited
  path: website/docs/reference/formats.md
revision: 18
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

## Confirmed implementation handoff

At frozen50c1001369c90f662b50d162aac97961199c1e72, the implementor confirmed all five assigned reservations: one new CLI test, five small JSON fixtures and three documentation pages. No production or generated schema changed. Both generated dialects accepted separate adopter copies/envelopes through the actual CLI. The original guide outside its inserted section remains byte-exact.

Full ess-cli package executed158→167passing cases,0failed/ignored,26summaries,30.009861360seconds. Formatting, strict all-target Clippy and diff checks exited0. Corrected target and final package each retained34actual CLI invocations:6successes and28expected refusals. The public shell blocks ran with only the sample output root relocated to assigned scratch: four resources/two accepted envelopes; actual semantic assembly refused ESS-SPEC-001 at system.domains. Neither subprocess calls nor vector iterations inflate Rust test counts.

The first9-case observation retained8passes/1failure: the new external-reference TypeScript test expected the word unsupported, while the existing source emits its only-local-reference diagnostic. Root inspected the unchanged source owner at typescript.rs:34 and approved only that new substring correction, preserving exact exit, referenced ID, sentinel and absent-parent checks. The first33reached calls and original source are retained; this was no production defect or scope expansion.

Immutable implementor report SHA25673b736a6f1480333ed568f6a4b6fe5c262728188f05f5d0d9b2782a5c74cdfe8. Root verified16summary hashes,507evidence files and15final source entries before the bot-authored/committed freeze. The actual inherited158-case baseline names42a44c4; its diff to openinge283a26 contains only coordinator planning/design/review documents, as recorded by the implementor.

First source attack is assigned to count_writer_impl8 under the installed adversary0.8.0 charter, in the same unit with additive-tests-only custody after full implementor relinquishment. Its scratch is target/review-boundaries-10/schema-resource-identity/adversary-pass-1. Full integration gates and publication are not yet claimed.

## Source review and integration handoff

review-result:schema-resource-identity-adversary-pass-1 preserves the complete source attack against50c1001369c90f662b50d162aac97961199c1e72. Six added cases passed alongside all167inheritedcases:173passed,0failed/ignored,27summaries,21.48seconds. Final formatting and strict all-target Clippy passed. Root verified8sealed records,765catalog entries and all15frozen source entries, then froze the single additive396-line test at e7aca0f044b092eaadc630e4e2dc29388e4bc5d8. Report SHA2562c7de0ec18270544b9080bcbeb9b7ffeca7cc155e6892d486dae31dd5d1a046f. No product finding was returned and no finding outcome is invented.

The original new adversary fixture wrongly assumed currency had an enum; the actual generated schema accepts any string. Its5pass/1fail target, exact initial test and all receipts remain retained. Changing only that new input to a boolean restored the intended nested type refusal, passed alone and then in the full package. This is distinct from the implementor's retained diagnostic-expectation correction. Final package receipts include31new and34inherited actual CLI calls, not65Rust cases.

The learned surface adds the independent test crates/edge/ess-cli/tests/schema_registry_identity_adversary.rs and a root-owned Unreleased changelog entry. Unit ownership otherwise remains its9frozen files; production and original generated schemas are unchanged. The source review is complete after its first green pass. Integration gates and public delivery remain pending.

Incoming main6bf76440c38331f6dd5214e5667c210b58a24015 supplies independently published TypeScript normalization. Root retained its canonical1406-line planning journal, replayed9local schema commands through AEP and verified162unrelated incoming artifacts plus the one local story byte-exact. Reconciliation commit ea5b8f055cdf1c53025692f4eac8b01eeaa66053 precedes this schema merge. Both shared public pages merged automatically; the full integration gate will verify the combined source.

## Integrated verification

verification-report:review-boundaries-10-integrated records clean9e82d207b4c7ef6713c19b3bb842c8453018a6bc:8declared task-check lanes plus site-build and planning all returned0;2018workspace cases passed,0failed/ignored,178summaries. Every tracked source byte stayed unchanged. Both public merges match independently composed hashes, preserving the incoming TypeScript normalization work. The gate establishes source implementation completion under the standing grant; publication, CI, immutable public delivery and managed cleanup remain separately observed outcomes.

## Published source and verified public delivery

Published source239996d846460aee342ce42514378c25b2be5152 has successful exact-source CI, source bundle and documentation validation with both bot actor fields. Joint Atlas publication34044223213 succeeded on exact controlb0fb59be02712e221c2a07d3e589120ed0ae3d7a. Its complete1709-file artifact pins this reader/schema source pair, and the independent artifact verifier passed356routes/1328sitefiles. Full Website gate passed99tests,0failed/skipped/cancelled,141.370037115seconds; every artifact byte remained unchanged. Both live provenance endpoints and all3changed documentation pages matched exact artifact bytes. The dated wave page preserves every run/hash and the separate three existing Atlas workspace fence issues.

Implementation evidence is archived and independently reread; the exact managed implementation tree and merged unit branch were retired through the manager/non-forced Git path. Joint Website support archival/cleanup is still finishing. Actual future ESS coverage producer correspondence remains unexecuted and required before that later writer's publication. No release, tag, installed binary or default switch is selected here.
