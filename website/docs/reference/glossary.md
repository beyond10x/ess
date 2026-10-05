---
title: Glossary
sidebar_position: 6
description: One line for each term the ESS documentation uses, with the page that defines it.
---

# Glossary

Each term has one line here and a link to the page that defines it. The linked page is the
authority; this list only helps you find it.

| Term | Meaning | Defined in |
|---|---|---|
| Actor | Who may invoke which commands. | [The model](../concepts/ess.md#the-model) |
| Aggregate view | A view with `group_by:` that reports counts, sums and extremes over one entity's rows. | [Aggregate views](../guides/specify/aggregate-views.md) |
| Ambient precondition | A command every command of the system runs inside, such as an open session. | [Ambient preconditions](../guides/specify/commands-and-outcomes.md#a-system-can-run-inside-ambient-preconditions) |
| Authored scenario | A conformance scenario a person wrote as an `ess-scenario/*` document, joined to a suite only when `--scenarios` names it. | [Author scenarios](../guides/verify/author-scenarios.md#select-authored-scenarios-explicitly) |
| Binding | An event-to-command reaction across contexts, including what happens when it fails. | [Bindings](../guides/specify/bindings-and-components.md#a-binding-says-what-happens-when-it-fails) |
| Command | The only way state changes; it declares its input and its outcomes. | [The model](../concepts/ess.md#the-model) |
| Compiled model (IR) | The normalized intermediate representation every name resolves in; compiling the same source twice gives the same bytes. | [The pipeline](../concepts/ess.md#the-pipeline) |
| Component | A unit of ownership over domains, which can become a module or a process without changing what it owns. | [Logical, interface and delivery owners](../concepts/ess.md#logical-interface-and-delivery-owners) |
| Concurrent history | A recorded run of several clients, with each call's invoke and return instants, checked against the model. | [Check a concurrent history](../guides/verify/explore.md#check-a-concurrent-history) |
| Consistency | A view's declaration of whether a generated assertion reads it immediately or eventually. | [View consistency](../guides/specify/values-and-views.md#a-view-declares-its-consistency) |
| Conversion | A declared reason for a binding to carry a value from one context's type into another's. | [Conversions](../guides/specify/wire-names.md#crossing-contexts-takes-a-declared-conversion) |
| Declared coverage | An opt-in suite form that retains its declared scope, the selected scenarios and every refusal, so a report can qualify what it covers. | [Declared coverage](../guides/verify/runners.md#opt-into-declared-coverage) |
| Deliverable descriptor | An `ess-component/1` document naming a deliverable's source paths and release units. | [Component delivery](../concepts/component-delivery.md) |
| Digest | A hash identifying only the bytes its producer defines; several digest kinds exist. | [Which digest is this?](../reference/formats.md#which-digest-is-this) |
| Disposition | What synthesis does with one capability: generate it, name it as an obligation, or refuse it with a reason. | [The synthesis plan](../guides/synthesize.md#the-plan-every-capability-gets-exactly-one-disposition) |
| Display name | The human-readable name generated documentation and a UI show. | [Names](../guides/specify/layout-and-validation.md#names) |
| Domain | One bounded context: its types, entities, commands, events, errors and views. | [Layout](../guides/specify/layout-and-validation.md#layout) |
| Entity | Identity-bearing state with a lifecycle, invariants and declared relations. | [The model](../concepts/ess.md#the-model) |
| Entrypoint | One physical way into an implementation, an argv or an HTTP(S) URL, recorded in a realization. | [Record a realization](../guides/record-realization.md) |
| Error | A declared domain refusal. | [The model](../concepts/ess.md#the-model) |
| Event | A fact a command's outcome emits. | [The model](../concepts/ess.md#the-model) |
| Explorer | Seeded random command sequences run against your target and checked after every step. | [Explore](../guides/verify/explore.md#explore-random-command-sequences) |
| Format version | The number in `format:` that moves when meaning, identity or the persisted shape changes. | [Format version history](../reference/spec-versions.md#when-the-number-moves) |
| Guard | A predicate on an outcome that decides when that branch is taken. | [Guards and outcome selection](../guides/specify/guards-and-predicates.md) |
| Impact | What a semantic delta invalidates: constructs, generated artifacts or suite scenarios. | [Find the work owed again](../guides/track-change.md#find-the-work-owed-again) |
| Interpreted target | The built-in target that runs a suite against the specification itself. | [Run a supported target](../guides/verify/runners.md#run-a-supported-target) |
| Invariant | A rule on a type or entity that travels into every projection. | [Invariants](../guides/specify/fields-and-invariants.md#an-invariant-reads-only-what-every-creation-sets) |
| Locator | The name anything outside the specification uses to point at a construct. | [Names](../guides/specify/layout-and-validation.md#names) |
| Mutant | A copy of the specification with one rule changed, used to test whether a suite notices. | [Mutation audit](../guides/verify/mutation-audit.md#audit-the-suite-with-specification-mutants) |
| Normalization recipe | An `ess-normalization/*` document naming the ordered stages that turn external input into model values. | [Explicit normalization](../guides/generate-artifacts.md#explicit-normalization) |
| Obligation | A capability the specification does not fully determine, implemented by hand in a realization. | [Realizations](../guides/synthesize.md#realizations-the-humans-half) |
| Outcome | One declared result of a command, including each refusal branch. | [Declared outcomes](../guides/specify/fields-and-invariants.md#a-command-that-can-be-refused-says-so) |
| Predicate | A condition over facts, written as a compact string or a structured mapping. | [Predicates](../reference/predicates.md) |
| Projection | A generated artifact derived from the compiled model: documentation, site, JSON Schema, OpenAPI or AsyncAPI. | [Projections](../concepts/ess.md#projections-ess-generate) |
| Qualified name | The dotted name only the specification itself uses, such as `billing.invoice.CreateInvoice`. | [Names](../guides/specify/layout-and-validation.md#names) |
| Reach | Where a component's callers are: `in_process`, `network` or `command_line`. | [Reach, entrypoints and identity](../concepts/ess.md#reach-entrypoints-and-identity) |
| Realization | A document naming the immutable artifacts that implement an exact specification and how they are entered. | [Record a realization](../guides/record-realization.md) |
| Report | The standalone record of what a run's scenarios observed against a named target and specification digest. | [What the report proves](../guides/verify/runners.md#what-the-report-proves) |
| Scenario | One semantic check in a suite: steps against a target and the result they require. | [The conformance suite](../concepts/ess.md#the-conformance-suite-ess-verify-conform) |
| Semantic delta | What moved in meaning between two revisions of a specification. | [Compare revisions](../guides/track-change.md#compare-revisions) |
| Suite | The deterministic set of scenarios a specification requires of an implementation. | [Synthesize a suite](../guides/verify/synthesize-a-suite.md#generate-the-suite) |
| Survivor | A mutant no scenario notices: a declared rule the suite does not pin down. | [Mutation audit](../guides/verify/mutation-audit.md#audit-the-suite-with-specification-mutants) |
| Synthesis plan | The language-neutral plan that gives every capability of a specification one disposition. | [The synthesis plan](../guides/synthesize.md#the-plan-every-capability-gets-exactly-one-disposition) |
| System | The root of a specification: its name, version and domains. | [The model](../concepts/ess.md#the-model) |
| Target | The implementation a suite runs against, built in or your own. | [Runners and reports](../guides/verify/runners.md) |
| Topology | Each component's runtime requirements: replica bounds, statefulness and required resources. | [The model](../concepts/ess.md#the-model) |
| Value expression | A `payload:` or `sets:` value taken from more than the input or a literal. | [Value expressions](../guides/specify/values-and-views.md#value-expressions) |
| View | A read model: what it shows, filtered by what. | [The model](../concepts/ess.md#the-model) |
| Wire name | The name HTTP paths, topics and generated JSON use. | [Names](../guides/specify/layout-and-validation.md#names) |
