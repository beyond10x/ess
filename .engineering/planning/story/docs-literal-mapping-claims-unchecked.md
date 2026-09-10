---
format: aep.planning-md/1
id: story:docs-literal-mapping-claims-unchecked
kind: story
status: active
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
revision: 4
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
