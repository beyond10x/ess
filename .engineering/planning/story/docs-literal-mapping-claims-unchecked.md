---
format: aep.planning-md/1
id: story:docs-literal-mapping-claims-unchecked
kind: story
status: implemented
title: Generated interaction docs say a binding literal was taken on trust while the compiler checks it
summary: ess generate --kind docs renders "the compiler took it on trust rather than checking it" for a literal mapping value that ESS-BINDING-002 validates against the enum
owner: timo
tags:
- consumer-downstream-adopter
relations:
- decomposes: epic:review-boundary-remediation
- serves: vision:O2
scope:
- confidence: cited
  path: crates/generate/ess-gen/src/docs.rs
- confidence: cited
  path: crates/generate/ess-gen/tests/corpus/billing/docs/interactions.md
- confidence: cited
  path: crates/generate/ess-gen/tests/corpus/oracle-fixture/docs/interactions.md
- confidence: cited
  path: crates/generate/ess-gen/tests/docs.rs
- confidence: cited
  path: crates/specify/ess-compiler/src/ir.rs
- confidence: cited
  path: crates/specify/ess-domain/src/binding.rs
revision: 9
---
## Context
`specs/services/pusher` (downstream-adopter, 2026-09-10) maps a literal enum value in a binding: `reject-on-refused` supplies `reason: backend_refused` for `pusher.subscription.RejectAuthorization.reason` (`v1/model/components.yaml`). The generated `docs/interactions.md` for that binding says: "Nothing in the model says how to read that as a `pusher.subscription.DenialReason`, so the compiler took it on trust rather than checking it." An adversarial review substituted `reason: not_a_variant_at_all` and ran `ess specify validate`: exit 1, `error[ESS-BINDING-002] … is not a variant of pusher.subscription.DenialReason … variants: rule, backend_refused, backend_unreachable`. The compiler checks the literal; the docs generator says it does not. A committed generated artifact states something false about the model it was generated from.

## Acceptance

Generated binding-literal documentation reports the compiler's actual guarantee: exact declared enum membership, including Optional/newtype wrappers, or text representation admission for String-backed targets. It must not imply numeric/Boolean literal support, String/newtype invariant validation or existence of external resources. Correct the misleading related IR commentary without changing validation or serialized IR shape. Pin enum, wrapped enum and String/newtype distinctions with fixture assertions; preserve existing corpus except the corrected sentences. Invalid variants remain refused by existing validation.

The operator excludes full and ownership gates. Run the affected docs/corpus tests and relevant literal-validation cases, strict affected-package Clippy and formatting; record their actual counts and exits. Required remote integration/release checks remain separate, and no bypass or weakening is authorized.

## Scope

Derived 2026-09-11 by aep-drive:story-scoper at main 6b666e58. Primary surface: generated interaction-documentation literal claims — cited.

- crates/generate/ess-gen/src/docs.rs: mapping_bullet literal arm — cited.
- crates/generate/ess-gen/tests/docs.rs: existing assertion requires the false claim — cited.
- crates/generate/ess-gen/tests/corpus/billing/docs/interactions.md and crates/generate/ess-gen/tests/corpus/oracle-fixture/docs/interactions.md: pinned generated sentences — cited.
- crates/specify/ess-compiler/src/ir.rs: ResolvedMappingValue::Literal and related ResolvedPayloadValue::Literal commentary — cited.
- Validation and serialized IR shape remain unchanged; crates/specify/ess-domain/src/binding.rs is read-only authority — cited.
- Confidence: high; defect sites, assertions and corpus are established — cited.
- Collides with bounded accessors in docs.rs, docs tests and ir.rs — inferred.

## Boundaries
No change to what the compiler checks. No new mapping syntax. Documentation wording only, pinned by a fixture.

## Verification
Fixture with `mapping: {reason: <literal>}` on an enum-typed input renders the corrected sentence; the existing fixtures are unchanged except for that sentence; `task check`.

## Wave source provenance

Imported through the AEP CLI from the operator-selected primary-checkout draft, revision 1. Original bytes retained as local-evidence:ess-evolution-20260910/priority-source-docs-literal-mapping-claims-unchecked.md (SHA256 697bc5654361a6b086979e8ab1d6558cbe80296f7fdd16c7230ff509547e7248). This branch starts a truthful import record; it does not invent the source draft's earlier command history. The primary draft and journal remain untouched.

## Public import correction

The first unpublished import was refused by the coordinated private-identifier check. Its exact rejected patch is retained privately in the local wave evidence. This replacement was created through AEP from the same source snapshot with the private organization identifier generalized before any journal event was written. Source acceptance and source snapshot hashes are preserved; no scanner policy or exception changed.

## First adversary pass

Candidate75845afb passed216 generator tests and eight existing literal checks, but independent review added five docs cases and found two real failures:32 Optional wrappers and33 named newtypes can reach an enum after domain validation stopped, so the renderer's unbounded walk overclaims compiler verification. The affected docs lane executed39 cases:37 passed,2 failed. Full public-safe review recorded as review-result:priority-literal-pass1-20260911 before routing to the same implementor. Keep all new cases; match actual bounded validation and correct related IR prose without changing admission. The underlying unchanged admission gap is tracked separately as story:literal-representation-walk-exhaustion.

## Scope confirmation

Implementation confirmed the original five source/test/corpus paths. First adversary correction additionally changes crates/specify/ess-domain/src/binding.rs only to expose the existing WRAPPER_LIMIT=32 read-only and correct its documentation; the walk and admission behavior are unchanged. The renderer consumes that actual validation bound instead of duplicating a constant or conflating it with the distinct public parser-depth bound. All five adversary cases remain and an additional16-case boundary matrix covers enum/String leaves across Optional/newtype/mixed chains. This scope correction is recorded before the second review.


## Integrated source and final review

Source candidate eb71c85d634c8e252a7ac380843a87c26aefcba3 merged into the wave in 55c2b107. Both source commits and the merge use the required bot author and committer. Corrected generator package evidence is 222 tests (including 40 documentation tests and three corpus cases); eight domain literal cases, strict Clippy and formatting also passed. Final review review-result:priority-literal-pass2-20260911 found nothing after eight deciding cases passed once in 0.24 seconds. No full local gate or ownership suite was run. The corrected admission-bound guarantee and original assertions remain intact.

The two-review finding ledger reports carried [], new [], and one resolved introduced documentation overclaim at docs.rs:1730. The independent underlying source-admission exhaustion gap remains in story:literal-representation-walk-exhaustion; this documentation change preserves source admission. Source implementation is complete under the operator-authorized focused verification contract. Main integration, signed final publication and release remain wave obligations.
