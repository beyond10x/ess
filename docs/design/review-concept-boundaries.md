# Logical, interface and delivery concepts

Status: accepted for story:review-glossary-boundaries in wave 13 under standing remediation
implementation approval. Current source is published 3464ea639258be53355f4f738c766efad2b6a3ba;
all relevant code and public documentation bytes match the dbe78c5 scope refresh. The original
candidate v2, SHA256 c104139cc76790960c049a5e889f24d1981464aaabb7c170eca4c77877a5f1b5, is retained.
Source report: target/review-boundaries-12/preparation/glossary-scope/report.md, SHA256
4e672add7a68de89fd7edb43ddfaa7f50eb679b0aa41efcf1a90f826a4d07d1d. Its 32 repository inputs
and two instruction/brief inputs were independently hash-verified by root. The report contains
the complete owner and example tables; it is inspection evidence, not executed test evidence.

## Chosen documentation boundary

The intended change is a reference for existing contracts. It changes no authored or persisted
format, Rust type, crate name, parser, compiler, projector, runtime or executor behavior. Use the
existing website/docs/concepts/ess.md as the public ownership reference and update the relevant
component/reach explanation in website/docs/guides/write-a-specification.md. Qualify the
interface statement in website/docs/guides/record-realization.md as physical invocation,
attachment and support metadata, linking the concept reference. This design's final
home is docs/design/review-concept-boundaries.md. No new public page, navigation entry, model,
package or validation lane is needed. These coordinator choices are accepted for this documentation unit and grounded in the
source inspections and exact examples below.

## Preferred labels and owners

Use these prose labels while retaining actual Rust/document type names where useful:

| Prose label | Existing owner |
|---|---|
| Logical component | ess-domain component model; domain ownership, command/event surface, reach and optional CLI layout |
| Deliverable descriptor | ess-deployment ComponentSpec; repository build/runtime/release inputs |
| Imported component alias | ess-composition ServiceImportSpec and ResolvedService; an exact model/component selection |
| Stack member | ess-deployment stack SystemRequirement and LockedSystem; runtime/chart release selection |
| Runtime requirement | ess-domain Topology and Workload; replicas, statefulness and resource requirements |
| Runtime workload mapping | ess-deployment runtime RuntimeSpec and Workload; desired components, processes, containers and storage |
| Implementation manifest | ess-realization RealizationSpec; selected implementation artifacts and physical entrypoint descriptions |
| Entrypoint prerequisite | ess-realization RuntimeRequirement; declared environment and credential-source requirements |
| Declared interface reach and CLI contract | ess-domain Reach and RawCommandLineSurface, retained in compiled component IR |
| Invocation and attachment | ess-realization EntryPointSpec and Invocation; argv or URL and attachment/support metadata |
| Derived HTTP contract | shared ess-gen HTTP routes, consumed by OpenAPI and current HTTP synthesis |

Do not call semantic topology deployable grouping or observed placement. Do not imply stack
resolution verifies composition aliases or typed operation payloads: its composition digest is
supplied input. Keep the composition byte-buffer and target-support limits explicit when examples
would otherwise imply those capabilities. A deliverable can map several logical components;
shared words do not make their identifiers interchangeable.

## Interface ownership decision

Document the present division as authoritative current behavior. Semantic reach and CLI layout
are interface contract declarations embedded in the authored/compiled semantic model and affect
its identity. They are not merely implementation annotations. Physical invocation, attachment and
support descriptions belong to realization, and runtime workload mapping belongs to deployment.
The glossary does not relocate any of these contracts.

Reach has exactly InProcess, Network and CommandLine, with InProcess as the default. An authored
CLI block requires CommandLine. One logical component has one reach value; duplicating its domain
owner cannot create a second surface. Current Rust/Go HTTP server generation selects Network;
CLI grammar generation consumes the CLI layout. An OpenAPI projection or native executable name
does not by itself establish a served interface or a semantic CLI.

Realization supports multiple entrypoint records, with one primary, and records may identify the
same implementation. This is supported description metadata. It does not prove that an interface
runs, that access is authorized, or that CLI and HTTP synthesis coexist for one semantic component.
The inspected examples supply no executed dual-interface claim. Do not invent a combined example
and describe it as supported end-to-end behavior.

## Example accounting

The reference must explicitly distinguish CLI-only, HTTP-only and simultaneous-interface cases.
Use the existing Clap desk-service fixture as the CLI grammar example and the Gatepass network
component/server source as the HTTP example. Attribute the finite source/test assertion only;
this change does not claim a new live execution. For simultaneous CLI/HTTP, explain the current
single-reach limitation and distinguish it from multiple physical realization entrypoint records.

Walk the Todo/Usage composition and Oracle deliverable/runtime/stack fixture as additional owner
checks. A composition alias is an imported model selection; a stack member selects releases. A
runtime workload can map order-service and dispatch-service together without redefining logical
domain ownership. Link existing public guides where useful and keep engineering reports out of
public content. Every F12 term and example gets one owner or an explicit supported limitation.

A future additive multiple-interface design would have to preserve one logical domain owner,
name interface identities and their references, decide model-identity and CLI compatibility,
and enumerate affected admission, projection and synthesis consumers. Those are design criteria;
this documentation change introduces no such entity or capability.

## Verification and completion

Before dispatch, refresh the report's relevant source and public-page hashes against the newly
integrated cache wave. Retain any intervening differences and resolve whether they affect a claim.
The implementor returns a complete term/example-to-owner table with exact source citations and
shows the changed public paragraphs. An independent document adversary checks the public claims
against those owners and the story acceptance. Do not add a test that merely searches for this
prose or mirrors its table; link/build checks and source review are the appropriate validation.

Root runs the declared repository gates and site-build against the integrated source. Publish
the green source commit, then verify the Atlas-owned deterministic source selection and rendered
Website artifact, including all three changed public pages and production provenance. The new story
status rests on those actual results. No release, tag, version, installation or default change.


## Fresh source readback

The independent refresh report is SHA256
45974a2d7c601baf1d01324c17f0aed8a528b0a6494534756d16f1bd51cb16fa. All 37 inputs were
verified by root; all model/projector/example owners are byte-identical to the original scope.
The current component-delivery explanation must remain unchanged: artifact-byte proof does not
establish publisher/signature authorization. The additional realization-guide qualification
repairs a pre-existing ambiguity identified by this refresh; no cache change introduced it.
Realization's current digest covers specification, synthesis and implementation identities,
excluding entrypoints. Explain that boundary when discussing identity; neither digest proves
execution, deployment or authorization. The fourth reservation and this wording are accepted for wave 13.
