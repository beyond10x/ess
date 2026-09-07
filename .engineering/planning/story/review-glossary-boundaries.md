---
format: aep.planning-md/1
id: story:review-glossary-boundaries
kind: story
status: draft
title: Disambiguate ESS logical, interface and delivery concepts
tags:
- P2
- review-2026-09-05
relations:
- decomposes: epic:review-boundary-remediation
- serves: vision:O2
scope:
- confidence: inferred
  path: docs/design/review-concept-boundaries.md
- confidence: cited
  path: website/docs/concepts/ess.md
- confidence: cited
  path: website/docs/guides/write-a-specification.md
revision: 9
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

## Scope

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
