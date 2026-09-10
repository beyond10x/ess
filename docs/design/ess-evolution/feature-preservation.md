# Feature preservation mapping

Baseline ESS commit: abf51add80d0f083c1f701330173cc10d609ad46.

The executable authority remains crates/edge/ess-xtask/src/consumer_coverage/. In particular,
feature-preservation.json maps every baseline consumer to its retained entrypoints, execution
profile and exact reviewed requirement/case references without promoting existing unknowns.
profiles.json names exact consumer entrypoints and claim boundaries; initial-baseline.json retains
unknown eligibility; reviewed-candidates.json binds exact behavioral cases; reviewed-schema-metadata.json
and macro-guards.json retain metadata and mutation obligations. This mapping supplements those
inventories and does not reclassify a candidate as supported. Full extracted case identities and
source hashes are retained in the baseline evidence, including all recently added CLI cases.

| Existing capability / semantic source | Existing implementation and format authority | Retained evidence | Destination / compatibility acceptance |
| --- | --- | --- | --- |
| Domain types, entities, commands, outcomes, predicates, views and references | ess-domain, ess-compiler, ess-primitives; RawSpecFile, EssIr and published schemas | Specify package tests, compiler fixtures, consumer profiles authored-*, compiler-*, semantic-references and dependency-graph | Same packages and canonical bytes; add ServiceIr separately; task check and exact consumer cases |
| Structural synthesis and obligations | ess-synth; neutral plans and Rust/Go/web emission/refusal | Synthesis package fixtures/tests, generated/rust, generated/go, generated/web and billing/gatepass realizations | Retain structural outputs; ER realization is additional; synthesis-* cases plus booted service evidence separately |
| Existing Clap synthesis | ess-synth Clap target, CLI component ownership | synthesis-clap-* profiles and generated execution tests | Preserve grammar and handlers; no replacement by presentation binding |
| CLI presentation | ess-cli-contract and ess-cli-project; ess-cli/1 | Both package test corpora, CLI edge tests, cli-binding-resolution, cli-binding-rust-emission, cli-binding-process-execution | Same paths and contracts; preserve protected inputs, source parsing, process/refusal and dynamic validation cases; Connectors real bindings added separately |
| Composition and delivery | ess-composition, compiler graph | composition-compile and composition-rust-byte-transport plus package tests | Same semantic contracts and existing byte transport; no unproved service claim |
| Realization inventory | ess-realization; ess-realization/1 and /2 | realization-v1/v2 profiles, fixtures and package tests | Existing inventory remains distinct from runtime lowering; both readers retained |
| Schemas and imports | ess-gen, ess-openapi, schema-contract | generator-schema/openapi/asyncapi, shared-schema-mapping, external-schema-import; schemas/generated and generated/schema/openapi/asyncapi | Same supported semantics and explicit refusals; protobuf import is additive with source accounting |
| Documentation | ess-gen docs/site, Markdown/HTML/graph projections | generator-docs/site, docs-ir, markdown, html, graph-rendering and generated snapshots | Same canonical outputs, output containment and ownership contracts |
| Deployment | ess-deployment plus realization semantics | deployment-runtime/deployment/component/build and package fixtures/tests | Same existing plans; add typed requirements link without applying infrastructure |
| Infrastructure intention and observation | infra-domain/compiler/analyze/spec/project, ess-kubernetes | infrastructure-legacy-v1, infrastructure-qualified-v2-namespace, observed-binding-comparison and complete infra tests | Keep InfraSpec versus InfraIr and existing legacy readers; retain credential redaction, observation completeness and drift behavior |
| Conformance and reports | ess-conformance, existing Rust/Go runners and browser replay | conformance-*, coverage-*, suite-admission, browser-* and CLI coverage tests | Extend existing versioned machinery with opt-in UI observations and Flutter/Playwright emitters; preserve defaults and refusal sensitivity |
| Diff, impact and normalization | ess-diff; model-* native emission/execution/reference, external-bundle-normalization | Diff tests and all native Rust/Go/TypeScript normalization cases, source pins and schema fixtures | Preserve exact value semantics, wire names, impact and canonicalization; lowering cannot weaken them |
| Acquisition, cache and public claims | CLI acquisition, cache, release/support accounting | acquisition-*, cache-acquisition, release-report-model-qualification, support-maintenance | Same source identity, containment, artifact ownership and coverage refusals; candidate imports preserve unresolved questions |

Status for every existing row: implementation retained at the baseline; migration evidence pending.
Existing unknowns stay unknown. The current gate and exact consumer coverage determine supported and
refused cells, not this prose. New service/UI/protobuf/Flutter/runtime capabilities are outstanding.
