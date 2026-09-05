---
title: Outlook — making ESS dependable
description: The proposed priorities, vocabulary, and evidence needed for ESS to mature.
---

# Outlook — making ESS dependable

ESS has the foundations of a useful specification toolchain: typed system models, deterministic
projections, explicit implementation obligations, and executable conformance scenarios. Its next
stage should make those foundations dependable across the full path from authored intent to
generated artifacts and execution evidence.

This outlook describes **proposed improvement priorities**, based on an architectural assessment
of the `0.18.0` source tree on 5 September 2026. It is not a release schedule, an accepted format
migration, or a claim that the capabilities below already exist. For supported behavior, use the
[status page](./where-this-stands.md) and
[limitations and trust assumptions](./limitations.md).

## What maturity should mean

A mature ESS should make it easy to answer five questions:

1. What does this specification say, and which parts are valid?
2. What can this target generate or check, and what remains unsupported?
3. What changed, and which contracts, implementations, or checks need attention?
4. Which exact model, suite, implementation, and artifacts does this evidence describe?
5. What can be safely carried forward when formats, generators, or implementations evolve?

The next milestone should be confidence in a clearly declared subset. A longer list of formats
and targets is useful only when those questions remain answerable.

## Preserve the distinctions that already help

Authored specifications, compiled models, generated outputs, and evidence serve different purposes.
An entity declaration is intent. A resolved reference is a compiler result. An OpenAPI document is
a projection of a supported contract. A conformance report describes an execution of particular
checks. None should silently become authority for the others.

The same distinction applies to infrastructure. A system's runtime requirements, a proposed
deployment, and an observed cluster are different kinds of knowledge. ESS should keep the semantic
and infrastructure models separate and introduce comparisons where concrete use cases require them.

Concrete types, explicit relations, stable semantic references, and visible obligations remain the
preferred direction. A universal registry of arbitrary properties would make it harder to say what
the system actually understands.

## First: make every omission visible

Unsupported meaning should survive every boundary. An import that cannot preserve a constraint
should account for it. A suite that omits an obligation should retain that fact when saved,
translated to another language, or executed later. A partial infrastructure observation should
remain partial in downstream analysis.

Conformance needs separate answers for **execution** and **coverage**. Passing every scenario in
a suite does not by itself establish that the suite covers every required behavior of the source
specification. Reports should identify the exact executed suite and distinguish generated checks,
authored examples, unsupported obligations, and checks outside a selected component.

The maturity criterion is simple: no consumer should need the original terminal output to discover
why a saved artifact is incomplete.

## Second: make change analysis complete as the language grows

Relations, command assignments, view parameters, ordering, and interface declarations affect what
a system promises. Adding such a construct must also establish how it participates in semantic
diff, dependency analysis, artifact provenance, and conformance.

Every meaningful model change should either produce an explained delta or conservatively request
broader review and regeneration. When ESS cannot justify a narrow impact set, it should say so.
An empty result must not conceal a field the comparison has not learned about.

This calls for tests that change one semantic property at a time and follow the consequence across
the toolchain. Snapshot stability remains valuable, but stable output alone cannot show that all
meaningful inputs were considered.

## Third: validate at the boundary where data becomes trusted

A serialized model is input when another process reads it. Its format marker, references,
identifiers, graph order, and derived claims need validation even if a compiler originally produced
it. A well-shaped JSON object is not necessarily a valid executable plan.

The same discipline should apply to file handling. Source discovery needs predictable rules for
different authored document kinds. Generated paths must stay within their output root, collisions
must be refused, and the tool should know which files it owns when regeneration removes an artifact.

For credential-bearing adapters, unexpected input must not weaken sanitization. For delivery
commands, invalid plans must be rejected before an external tool starts.

## Fourth: make semantics agree across targets

A type name should have one documented meaning, even when its wire representation differs between
languages. Integer precision, decimal values, UUID admission, optional values, collection contents,
and closed records need shared examples and counterexamples across schemas, generated codecs,
predicate evaluation, and conformance adapters.

Each synthesis target should also establish that a valid model is representable in its language.
That includes generated symbol collisions, reserved names, recursive layouts, and wire-property
names. A target should emit usable code or an explicit limitation before writing a partial tree.

Browser tools need a similarly precise promise. A scenario replay is useful for understanding an
authored example, but it is not independent execution evidence. Unsupported state or view
calculations should be visible, and the supported calculations should agree with the model.

## Fifth: separate delivery intent from delivery evidence

ESS can keep deterministic build and deployment planning independent of the tools that execute
those plans. Explicit Docker, registry, and Helm operations should remain identifiable boundaries.

The next improvement is clarity about the result. Matching manifest digests and required evidence
references establishes consistency; it does not establish that a signature is trusted, that a
conformance report passed the required suite, or that the intended releases are currently running.

Dependable delivery needs explicit policies for evidence verification, cache integrity, execution
receipts, partial failure, and retries. A previous desired document should not be confused with an
observation of applied state. These improvements do not require turning ESS into a continuously
running deployment controller.

## A clearer vocabulary and map

Readers should not need to know crate names to understand which document to write. The following
grouping is a proposal for documentation and discovery; it does not rename current CLI areas.

| Group | What belongs here |
| --- | --- |
| System semantics | Domains, types, entities, relations, commands, events, views, actors and bindings. |
| Interfaces and implementation | Logical component surfaces, composition, entrypoints, implementation manifests and generated code obligations. |
| Behavior and verification | Authored scenarios, compiled suites, execution reports and coverage. |
| Build and delivery | Deliverables, build/runtime descriptions, releases, stacks, environment bindings and deployment plans. |
| Observed infrastructure | Collected observations, infrastructure intent, diagnosis, drift and manifest projection. |
| Documentation and contracts | Document models, rendered documentation and independently authored schema registries. |

Within each group, show the progression from **authored input → resolved model → generated output
or evidence**. The existing `specify`, `generate`, `verify`, and `infra` CLI areas remain useful
action-oriented entrypoints.

Several names deserve particular care:

| Current term | Proposed clearer wording | Distinction |
| --- | --- | --- |
| Component | **Logical component** for domain ownership; **deliverable** for an independently released implementation | A release boundary need not equal a semantic ownership boundary. |
| Realization | **Implementation manifest** in introductory documentation | Describes implementation artifacts and entrypoints. |
| Runtime | **Runtime specification** or **resolved runtime model** | Describes how logical components are mapped to processes, containers and storage. |
| Deployment IR | **Deployment plan** | Desired release set and bindings, separate from an execution receipt. |
| Source digest | **Compiled-model digest** where that is what is hashed | Avoids implying a checksum of authored source files. |
| Conformance | **Suite**, **run**, **report**, or **coverage**, depending on the object | Passing checks and covering requirements are different facts. |

### Schema identifiers should say which contract they identify

Three identifiers should stay distinct: a document's format discriminator, a JSON Schema resource's
`$id`, and a semantic construct's qualified name. A filename is not a substitute for any of them.
Likewise, format version, system specification version, package release version, and content digest
answer different questions.

For future formats, explicit roles would improve clarity. Candidate family names include
`ess-system-spec`, `ess-composition-model`, `ess-deliverable-spec`, `ess-conformance-suite`, and
`ess-deployment-plan`. **These are proposals, not accepted format identifiers.** Existing
`ess-client-plan`, `ess-release-bundle`, `ess-stack-lock`, and `ess-conformance-report` already make
their roles clear and gain little from cosmetic renaming.

A format catalog should state each family's producer, readers, supported versions, canonical bytes,
and validation guarantees. Immutable JSON Schema resource ids can then support editors and offline
registries where needed. An editor schema should explain that checking one file's shape does not
replace whole-system semantic validation.

Any change to persisted identifiers or canonical bytes needs an explicit migration: inventory
consumers, introduce readers and migration tooling, publish immutable schemas, move consumers, and
only then change default writers. Clearer labels can be adopted sooner. Historical formats and
digests should remain meaningful.

## Evidence for the next stage

| Priority | What should become demonstrable |
| --- | --- |
| Trustworthy boundaries | Incomplete imports and suites remain visibly incomplete; malformed persisted plans and unsafe output paths are refused. |
| Complete change accounting | A change to any supported semantic construct is explained and reaches the artifacts or checks it affects. |
| Reliable target semantics | Shared value tests agree across targets, and generated programs compile for the target's declared subset. |
| Dependable delivery | Consistency, evidence trust, and execution results are separate, testable outcomes with clear recovery behavior. |
| Predictable adoption | One glossary and format catalog agree with command help, source schemas, examples, and current support documentation. |

New adapters and language targets should grow alongside this evidence. The aim is a toolchain
whose limits are explicit, whose outputs can be traced to their authority, and whose successful
results mean the same thing to authors, implementers, and reviewers.
