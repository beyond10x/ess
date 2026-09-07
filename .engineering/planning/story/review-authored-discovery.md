---
format: aep.planning-md/1
id: story:review-authored-discovery
kind: story
status: active
title: Define predictable discovery for co-located ESS documents
tags:
- P1
- review-2026-09-05
relations:
- decomposes: epic:review-boundary-remediation
- serves: vision:O2
- depends_on: story:scenarios-directory-compiles-nothing
scope:
- confidence: cited
  path: crates/edge/ess-cli
- confidence: inferred
  path: docs/design/models/authored-discovery/domains/discovery.yaml
- confidence: inferred
  path: docs/design/models/authored-discovery/system.yaml
- confidence: inferred
  path: docs/design/review-authored-discovery.md
- confidence: inferred
  path: docs/design/review-format-catalog.md
- confidence: cited
  path: website/docs/guides/verify-conformance.md
- confidence: cited
  path: website/docs/guides/write-a-specification.md
- confidence: inferred
  path: website/docs/reference/cli.md
- confidence: inferred
  path: website/docs/reference/formats.md
revision: 11
---
## Finding and source

F10 (P1) from `docs/reviews/2026-09-05-architecture-review.md:365`, reviewed at `fd06a4d61bfb7b4990617810655dc181d6a3ab00`. Evidence locations: `crates/edge/ess-cli/src/load.rs:25`, `crates/edge/ess-cli/src/main.rs:2458`. Findings are attributed to the review; proposed implementation choices below are coordinator inferences.

## Acceptance

A documented mixed source/scenario/generated-output layout resolves the same intended authored inputs deterministically without ingesting generated YAML as source.

## Implementation boundary

Specify an explicit input manifest or uniform typed discovery contract before implementation; preserve supported existing layouts or supply actionable migration refusals. Resolve recursion, exclusions, duplicate documents and unknown kinds consistently. Retain the existing zero-authored explicit-path refusal when no scenario document is selected.

## Validation

Fixture the old layouts and a mixed tree with nested authored scenarios and generated YAML; compare resolved input identities over filesystem ordering changes and assert no silent empty success.

Run package-scoped checks while implementing; the integration coordinator runs every step of `task check` and `task site-build` when required by `AGENTS.md`, retaining individual exit statuses and executed-case counts.

## Compatibility and exclusions

The existing scenarios-directory-compiles-nothing story owns the immediate zero-result refusal and remains a prerequisite.

## Scope

Derived 2026-09-07 by aep-drive:story-scoper 0.8.0 at ESS `603dd90f855e994ce41aeab4788ea72f4fa01dbe`, refreshing the retained optional-manifest candidate against source `57e242e8a0eaa721968c3970099b4bc561cb91aa` — cited.

- **Primary acquisition:** `crates/edge/ess-cli` — cited. Own the shared package-local manifest/discovery reader, proposed as `src/input_discovery.rs`; specification, legacy-authored and coverage-authored adapters; deterministic legacy traversal; CLI help; complete acquisition-caller and A1–A24 tests/fixtures. Preserve legacy direct-file, omitted-scenario, shallow discovery, link-policy, source-identity and committed-suite distinctions. Include the current observed-bindings and release-qualification model callers.
- **Binding:** `docs/design/review-authored-discovery.md` — inferred. Record the root-accepted optional `ess-inputs/1` contract, original A1–A24 matrix, explicit model-expression limits and the source-current caller/refusal clarifications below.
- **Public model layout:** `website/docs/guides/write-a-specification.md` — cited. Document directory opt-in and the mixed layout while retaining existing specification layouts and semantic qualifications.
- **Public authored discovery:** `website/docs/guides/verify-conformance.md` — cited. Distinguish manifest selection from unchanged shallow legacy discovery, explicit-file selection and omitted scenarios. Preserve suite/5 identity/digest and source-versus-release qualifications.
- **CLI reference:** `website/docs/reference/cli.md` — inferred. Document directory configuration, active roles and bypass behavior without adding a flag. Preserve the current observed-bindings, release qualification and output-protection descriptions.
- **Public format reference:** `website/docs/reference/formats.md` — inferred. Add the closed acquisition-configuration contract, reader/version admission, role-specific filesystem checks, compatibility refusal and absence of a persisted manifest digest.
- **Engineering format catalog:** `docs/design/review-format-catalog.md` — inferred. Add only a source-cited acquisition-configuration entry corresponding to the new public format entry; preserve historical baselines and existing entries.
- **Model header:** `docs/design/models/authored-discovery/system.yaml` — inferred. Proposed tracked home for the retained 74-byte model declaration, conditional on root acceptance.
- **Model values:** `docs/design/models/authored-discovery/domains/discovery.yaml` — inferred. Proposed tracked home for the retained 538-byte ManifestFormat, RelativeInputPath and InputManifest declarations. Preserve explicit reader obligations and the absence of entity identity, ownership or lifecycle semantics.
- **Library boundary:** domain/compiler parsers and conformance source/identity types remain semantic authorities consumed by the package-local reader; this scope introduces no library writer, universal registry, generated-schema owner or dependency change — inferred.
- **Prerequisite:** preserve the implemented scenarios-directory-compiles-nothing refusal, omitted-scenario behavior and committed-input branches — cited.
- **Would collide with:** any writer inside ess-cli, either named public guide, either reference page, the engineering format catalog or the exact binding/model files — cited.
- **Confidence:** high for these conditional manifest-choice reservations: actual acquisition owners and new callers are established, and the additional catalog path maintains an existing inventory promise. Selecting another discovery contract requires a new concrete scope decision — cited.

## Accepted implementation decision

On 2026-09-07 the coordinator selects the optional immediate-directory ess-inputs.yaml contract
in docs/design/review-authored-discovery.md under the recorded standing implementation approval.
The complete A1–A24 matrix remains required. This updates the former four-reservation Scope to
nine, including the public CLI/formats reference, the engineering format catalog and two named
model files; the former source citations and inference history remain in Git. The current
read-only scoper refresh at 603dd90 confirms the same package-local acquisition owners and adds
observed-bindings and delivery qualification to A19/A21. No new library or dependency owner is
selected. These decisions are coordinator choices informed by actual source, not runtime results.

The retained candidate was already reviewed by review-result:authored-discovery-binding-pass1
with no findings. The unchanged named model now lives at docs/design/models/authored-discovery/
(system.yaml and domains/discovery.yaml). Current-source ESS 0.20.0 validation and compilation
each exited zero; validation printed `discovery v1 — 2 file(s), valid`. Compilation produced
2,192 bytes. The retained validator SHA256 is 94983b7227d8b5c3c69cdb448cf4dfe7a4d8c65b086ae7ce54d1db75c607ebf8.
These checks establish named declaration validity only. Reader behavior, path/filesystem policy,
whole configuration admission and the complete executed acceptance matrix remain owed.

The new wave is docs/plan/2026-09-07-review-boundaries-17.md. Root verified the scoper's 10 current
source pins against exact 603dd90 Git blobs in target/review-boundaries-17/preparation/scoper-root-readback.json.
The previously implemented scenarios-directory-compiles-nothing prerequisite stays intact.
The operator explicitly stopped Atlas follow-up during this continuation; source implementation,
its required source/site checks and incremental publication remain authorized. No release tag,
version bump or live ESS deployment is authorized.
