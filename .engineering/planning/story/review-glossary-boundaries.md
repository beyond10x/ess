---
format: aep.planning-md/1
id: story:review-glossary-boundaries
kind: story
status: implemented
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
revision: 17
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


## Preimplementation scope (retained)

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


## Scope

Confirmed by the implementor at `c99776eb14173b467143525947cd0933b983b459`, with the complete source attack recorded as `review-result:glossary-source-pass1`. Root independently verified the returned source and input inventories before integration.

- **Primary public reference:** `website/docs/concepts/ess.md` — cited; the implemented glossary names logical, interface and delivery owners and accounts for all original naming proposals and example families.
- **Authoring explanation:** `website/docs/guides/write-a-specification.md` — cited; states all three semantic reaches, the model-owned CLI contract and its distinction from physical invocation.
- **Realization explanation:** `website/docs/guides/record-realization.md` — cited; describes invocation, attachment and support choices separately from model reach/CLI layout and names the realization digest boundary.
- **Binding:** `docs/design/review-concept-boundaries.md` — cited; the accepted file exists at opening `38cfefc`, SHA256 `271910e53562be0e018db7b39fa9d52f2aee7333201d6bf437f7793cd7a563cd`. It was read before implementation and remained unchanged. The earlier inferred absence describes the frozen scoping subject; its resolution is explicit here and in the implementor confirmation table.
- **Read-only owners:** existing Rust model, compiler, composition, realization and deployment sources and all example fixtures — cited; none changed in this documentation unit.
- **Boundaries:** existing contracts only, with one semantic reach per logical component and no combined CLI/HTTP synthesis implied by multiple physical entrypoint descriptions. No persisted type/format or execution behavior changed — cited.
- **Confidence and collisions:** high, cited; the observed three-page diff and existing binding establish these exact four reservations. Earlier whole-website and incomplete three-file reservations remain above as corrected history.

## Reviewed implementation

The bot-authored unit `c99776eb14173b467143525947cd0933b983b459` was integrated as `ecb7efc22ad9b19b85ef4debd8143491d6a66ef3`. The independent document attack returned an empty findings block, with all 50 source inputs and 80 command receipts verified by root. Its exact report SHA256 is `035d6d1a4fcfe077dc9d03dd929be245e16358c36f4806583638d2c40198db59`.

The implementor report is SHA256 `7a3d758cf5e47267ec0545e5ce3c5020c7f690e75945f1ea6df52ac958dd9dbb`. It confirms each inferred Scope line, all 13 original naming-proposal rows and the bounded CLI, HTTP, simultaneous-entrypoint, billing, Todo/Usage and Oracle examples. The unchanged Markdown-only output claim remains F16 under the existing `story:review-public-support-claims`; this glossary wave adds no follow-up story.


## Completion evidence — wave 13

The complete first integration attempt at `ecb7efc22ad9b19b85ef4debd8143491d6a66ef3` returned zero for all ten lanes: fmt-check, clippy, test, doc-check, example-check, projection-check, release-check, action-check, site-build and planning. The workspace test lane executed 2,093 passing cases, zero failures and zero ignored cases across 189 summaries in 331.025796699 seconds. Tracked bytes and the full frozen toolchain manifest remained unchanged. The final gate receipt is SHA256 `49e1579d54cd6afb39a52a65821bc20cccf3a8cf4ad876a9c61830614086d9fd`; the test log is `0672624ac305c405718c3128ba79db606dd12a6e7aa46131ffc254fd9ef03419`.

The exact source commit is published on remote main. CI [34078983021](https://github.com/beyond10x/ess/actions/runs/34078983021), documentation validation 34078983036 and source bundle 34078983078 completed successfully for that commit. Atlas publication 34079580209 completed successfully under both bot actors at clean authority `d10b7484d64c28830774c9dae0ec531fcc47acb2`. Its artifact 10003271836 has archive SHA256 `d81a24f8f10523c586df4bf1cbdd2e905ed13ce44c62fc5483c461c849ad01da`; the immutable source-set SHA256 is `08d8cb4d6250b7b8917c0953949841e1e073156adca4329713ffb0d1525129a0` and names exact ESS `ecb7efc`, AEP `658cf76e6371b1628f6de69548e724b52803f5c2` and Website runtime `fc4571534765c098ed861bc326da4d3da0d1df63`.

The independent local immutable-artifact check exited zero and reported 357 routes and 1,332 files. The Website gate exited zero in 140.969673426 seconds with 99 passing tests and zero failures, skipped or cancelled tests. Both provenance endpoints and all three changed public page responses returned HTTP 200 and matched the artifact byte-for-byte. Final Atlas `docs verify-pages` exited zero in 155.095452445 seconds, reporting 37 repository states, 26 Pages repositories and 52 delivery routes.

The whole Atlas fence retains the same unrelated baseline failures as wave 12: AgentIDE's manifest under the old collector, the primary Website checkout's old Docs System pin, and Widgets' missing Serves section. Its 149 Rust tests passed. These are recorded separately; no organization-wide convergence claim is made. The exact glossary source, artifact, Website and live-delivery checks above passed.

Complete gate/publication receipts remain under coordinator `target/review-boundaries-13`, and source review is committed in `review-result:glossary-source-pass1`. The whole unit scratch has been archived and independently read back: 328 native entries, 2,051,916 regular payload bytes, archive SHA256 `01e50a14b3a6f836775a75f910cb8dbda145fecae0ab2831b36a11a0ce63ba58`. Managed lifecycle cleanup is recorded in the wave page after final retention checks. No release, tag, installation or actual deployment execution was performed by this wave.
