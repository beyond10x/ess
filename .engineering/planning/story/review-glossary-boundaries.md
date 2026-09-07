---
format: aep.planning-md/1
id: story:review-glossary-boundaries
kind: story
status: active
title: Disambiguate ESS logical, interface and delivery concepts
tags:
- P2
- review-2026-09-05
relations:
- decomposes: epic:review-boundary-remediation
- serves: vision:O2
scope:
- confidence: cited
  path: docs/design/review-concept-boundaries.md
- confidence: cited
  path: website/docs/concepts/ess.md
- confidence: cited
  path: website/docs/guides/record-realization.md
- confidence: cited
  path: website/docs/guides/write-a-specification.md
revision: 15
---
## Finding and source

F12 (P2) from `docs/reviews/2026-09-05-architecture-review.md:424`, reviewed at `fd06a4d61bfb7b4990617810655dc181d6a3ab00`. Evidence locations: `crates/specify/ess-domain/src/component.rs:1`, `crates/specify/ess-domain/src/component.rs:103`, `crates/generate/ess-deployment/src/component.rs:27`, `crates/specify/ess-realization/src/lib.rs:1`. Findings are attributed to the review; proposed implementation choices below are coordinator inferences.

## Acceptance

For each F12 concept or entrypoint example, the reference names one logical, interface or delivery owner or an explicit supported limitation.

## Implementation boundary

Use the review's glossary as a proposal, comparing each term with current Rust owners. Separate semantic component/deliverable, composition selection/stack service, requirements/runtime mapping and realization. Decide whether reach/CLI layout is contract-level or implementation-level; for multiple entrypoints, document the current supported limitation and criteria for an additive typed design.

## Validation

Trace the glossary's terms to code and walk CLI, HTTP, composition and delivery examples; no example implies a capability the model lacks. Validate links and run site-build for public doc edits.

Run package-scoped checks while implementing; the integration coordinator runs every step of `task check` and `task site-build` when required by `AGENTS.md`, retaining individual exit statuses and executed-case counts.

## Compatibility and exclusions

No crate rename, universal component registry or semantic/infra merger. Persisted API changes need their own design and migration, not a terminology edit.

## Previous scope (retained)

Derived 2026-09-07 by `story-scoper` against ESS `bab9206e7f02cbf9519c92f8c7712606e086caee` — cited.

- **Primary public reference:** `website/docs/concepts/ess.md` — cited; the existing model reference owns the conceptual vocabulary and already describes logical components and topology. Its topology description presently blurs requirements with deployable grouping.
- **Public interface explanation:** `website/docs/guides/write-a-specification.md` — cited; “Three layers above the domains” owns the component/binding/topology distinction and reach explanation. It currently describes a two-value reach set, while the implementation also admits command_line.
- **Binding:** `docs/design/review-concept-boundaries.md` — inferred; this is the story’s reserved engineering design path, absent at the frozen subject, for recording the terminology and interface-ownership decision before documentation implementation.
- **Source owners:** the specification component/topology model, compiled component IR, composition selector, realization compiler, deployment descriptor/runtime/stack compilers, and CLI/HTTP projectors — cited; these are read-only evidence, not write reservations.
- **Examples:** existing CLI synthesis fixture, gatepass network component/server, Todo/Usage composition, billing realization and Oracle delivery chain — cited; each supports a bounded statement, and none establishes simultaneous CLI-and-HTTP semantic reach.
- **Implementation boundary:** documentation only; describe present contracts, retain technical type and format names, and distinguish supported metadata from generated or running behavior — cited; the story expressly excludes persisted API changes and registry or model unification.
- **Confidence:** high — cited; acceptance is a terminology/ownership reference, and the inspected public explanations and concrete source owners establish the exact documentation surfaces.
- **Would collide with:** any unit changing the two named public pages or the reserved concept-boundaries design — cited; reserve these exact paths instead of the previous whole-website token.

## Boundary-map validation rows

Check semantic component versus deliverable, composition alias versus stack service, requirements versus runtime choices, and realization versus interface layout. Include CLI-only, HTTP-only and simultaneous-entrypoint examples; each must name the owner/disposition or a current unsupported limitation, with a source citation. The interface-ownership decision remains required, not removed by the single-result acceptance.
## Current independent scope evidence

The source scoper inspected frozen `bab9206e7f02cbf9519c92f8c7712606e086caee` and returned
`target/review-boundaries-12/preparation/glossary-scope/report.md`, SHA256
`4e672add7a68de89fd7edb43ddfaa7f50eb679b0aa41efcf1a90f826a4d07d1d`. Root retained the final
message unchanged and verified all 34 inputs, including 32 exact Git blobs. It performed no
build, test or implementation. The original whole-website reservation is replaced by the two
confirmed public pages; the existing inferred internal design reservation is retained.

The report's owner table and example dispositions distinguish semantic reach/CLI layout, which
already participate in model identity, from realization invocation and attachment metadata.
Multiple physical entrypoint descriptions do not establish simultaneous CLI-and-HTTP semantic
reach or synthesis. Terminology and explicit interface-ownership wording still require the
binding document; no model migration, new type, implemented glossary or next-wave selection
is established by this scope refresh.


## Scope

Derived 2026-09-07 by `aep-drive:story-scoper` 0.8.0 against ESS dbe78c5b15df478ec2cd4883c67d0012cdf90e17 — cited.

- **Primary public reference:** `website/docs/concepts/ess.md` — cited; name each logical, interface and delivery owner, correct topology's deployable-grouping description, and state bounded example dispositions.
- **Authoring explanation:** `website/docs/guides/write-a-specification.md` — cited; correct the two-value reach explanation, describe the existing model-owned CLI contract, and link the ownership reference.
- **Realization explanation:** `website/docs/guides/record-realization.md` — cited; qualify lines 8–11 so physical invocation and attachment choices are distinguished from reach and CLI layout already owned by the authored/compiled model.
- **Binding document:** `docs/design/review-concept-boundaries.md` — inferred; reserved by the story but absent at the frozen subject. Record the terminology, interface-ownership statement and supported limitations before implementation.
- **Read-only evidence:** existing model, compiler, composition, realization, deployment and interface-emission owners, together with the unchanged CLI, HTTP, composition and delivery examples — cited.
- **Implementation boundary:** documentation of existing contracts; preserve Rust names, persisted formats, model ownership and executor behavior. Multiple realization entrypoints must not imply simultaneous CLI-and-HTTP semantic reach or synthesis — cited.
- **Confidence:** high — cited; exact public statements and their owning code establish the four documentation surfaces. Preferred labels and binding acceptance remain candidate decisions.
- **Would collide with:** edits to the three named public pages or the reserved concept-boundaries design — cited.

## Cache-integrated scope refresh

The independent report at
`target/review-boundaries-12/preparation/glossary-scope-refresh/report.md` is SHA256
`45974a2d7c601baf1d01324c17f0aed8a528b0a6494534756d16f1bd51cb16fa`.
Root verified all 32 frozen `dbe78c5b15df478ec2cd4883c67d0012cdf90e17` repository blobs and
five local inputs. Thirty original repository blobs are unchanged; this story's scope record
and the component-delivery cache explanation are the only two differences. Neither changes
the term owners or example limitations.

The earlier three-file reservation was incomplete: `record-realization.md:8–11` broadly says
interface choices stay outside the system model. Root confirms the scoper's source citations:
`RawComponentSpec` owns reach/CLI declarations, compiled components serialize them into the
model identity, while realization separately describes physical invocation, attachment and
support. Add that existing public guide as a cited fourth reservation. Keep the opening
scope above so the correction is visible. This does not expand into a new model capability.

Binding acceptance and wave selection remain pending. The preferred-label candidate needs its
public-page reservation and exact delivery checks updated to include all three public pages;
no source implementation or executed tests are claimed by this preparation.


## Accepted terminology and interface binding — wave 13

Root accepts `docs/design/review-concept-boundaries.md` under the standing remediation approval.
The retained candidate v2 is SHA256
`c104139cc76790960c049a5e889f24d1981464aaabb7c170eca4c77877a5f1b5`; acceptance changes its
status/source references and makes its existing-contract documentation choices binding.
The fresh published base is `3464ea639258be53355f4f738c766efad2b6a3ba`, whose relevant code
and public source bytes match the independently inspected `dbe78c5` subject.

Select this story alone with the same four reservations, including all three named public pages.
Semantic reach and CLI grammar remain current authored/compiled model contracts. Physical
invocation, attachment, support and entrypoint prerequisites remain realization descriptions;
implementation identity and model identity have their documented distinct inputs. Multiple
physical entrypoint records do not establish simultaneous semantic CLI/HTTP reach or synthesis.
The glossary documents current owners and explicit supported limitations, with no new model,
persisted field, execution behavior or additive multiple-interface capability selected.

The implementor returns exact term/example-to-owner citations and confirms every inferred Scope
line. An independent document adversary checks all three changed pages against the existing
owners and complete F12 acceptance. Because this is explanatory prose, meaningful source review
and link/build checks apply; no phrase-matching test or fabricated red case is required. The
coordinator still runs the entire declared integration gate and exact public delivery before
recording completion. Earlier pending-selection statements above describe preparation history.
