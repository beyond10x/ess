---
format: aep.planning-md/3
id: story:feature-request-432
kind: story
status: implemented
title: 'Topology: one central write path with a read replica in each environment'
tags:
- ess-0.54.0
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#432
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
- supersedes: story:fast-line-topology-central-writer-read-replicas
scope:
- confidence: inferred
  path: crates/specify/ess-compiler/tests/replica_read_idiom.rs
- confidence: cited
  path: website/docs/guides/specify/values-and-views.md
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-10-05T13:26:28Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-10-05T13:26:29Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "active", to: "implemented", at: "2026-10-06T17:47:56Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}}
---
## Outcome
Resolve beyond10x/ess#432: Topology: one central write path with a read replica in each environment.

## Origin
beyond10x/ess#432, filed 2026-10-05; reported 2026-09-23 by an adopter specifying an identity service, carried in their specification as an unmapped requirement. Reproduced minimally in `<fit-review scratch>/probe-432/` (fresh specification, no adopter files).

## Fit review
1. Need: one logical service accepts writes at one central instance; every other environment serves reads from a local replica that lags the writer. The requester proposes no syntax, only "a way to declare a write-primary and read replicas for one component across environments" with read-your-writes at the primary and eventual reads at replicas (issue body, theirs). The issue's premise that `topology.yaml` places a component in environments is wrong: `RawWorkload` has `replicas`, `stateless` and `requires` only (`crates/specify/ess-domain/src/topology.rs:60-69`), and probe `c` (`environments: [central, edge]`) is refused: `unknown field environments, expected one of replicas, stateless, requires` (ess 0.52.0).
2. Class: local policy / deployment fact, not a domain gap. The domain fact "a read may trail the write that changed it" is already a view's `consistency: eventual`, the default (`crates/specify/ess-domain/src/view.rs:18-21`, `:75-87`). Which instance accepts writes is placement, and the model keeps placement out on purpose: a component is "a logical boundary, not a deployment decision" (`crates/specify/ess-domain/src/component.rs:3-6`); a topology workload states "correctness requirements, not process/container grouping or observed placement" (`website/docs/concepts/ess.md:52`).
3. Existing idiom: probe `a` validates on ess 0.52.0 (`directory v1 — 4 file(s), valid`): one component that owns the domain and accepts its commands; views declared `consistency: eventual`; `settings:` `role` (an enum `primary | replica`) and `write-endpoint`; topology `stateless: true` with `requires: - postgres: directory-store`. Each environment's values go into its `ess-environment/1` release binding, `config` and `endpoints` slots (`crates/generate/ess-deployment/src/environment.rs:30-46`); that the component setting and the runtime config slot are the same value is inferred, not run. The two refusals hold the idiom: probe `b` (stateful, `min: 2`) is `ESS-TOPOLOGY-004` with the hint "a store they share is a `requires:` entry" (`topology.rs:177-194`); probe `d` (a separate `directory-replica` component owning nothing) is `ESS-COMPONENT-007`. A suite asserts eventual views with `eventually`, which holds against the primary and against a replica; `check-history` judges reads at the declared consistency (`website/docs/guides/verify/explore.md:150-155`).
4. Fit: a primary/replica key on a workload would put placement into the logical model, against `component.rs:3-6` and `concepts/ess.md:52`. Across environments it would also need a relation between two `ess-environment/1` documents. Each such document binds one cluster and namespace (`environment.rs:68-77`), and no delivery document refers to another environment. Per-instance consistency would also reach the view (`consistency` is one value per view, `view.rs:75`), every runner, `check-history`, `ess verify diff` and synthesis. It has no sibling construct to share the rule with.
5. Second adopter: a product catalogue written centrally and read from caches in each region. The idiom above covers it the same way, so the need is general. A new construct is still not justified, because both adopters have the same deployment shape and neither has a domain fact the model cannot state.
6. Cost of the requested construct: source format `ess/23`, new topology keys, a cross-environment reference in `ess-environment/2`, per-target consistency in suites (`ess-conformance/N`), new diff classifications, and changes to the Rust/Go/TypeScript runners and the history checker. Cost of the idiom: one guide section and one domain test. No format bump.
7. Alternatives: (a) change nothing and document the idiom, which is chosen. (b) `topology.workloads.<c>.replication: {primary, replicas}`, refused: it is placement in the logical model, and nothing downstream could check it. (c) a view consistency per role (`consistency: {primary: read_your_writes, replica: eventual}`) plus a realization entrypoint declaring its role. This is the only design that gains a check (read-your-writes at the primary), but it is a new construct spanning views, realizations and conformance. Defer it until an adopter shows a read-your-writes defect at a primary that an `eventual` suite cannot catch. The requester's proposal is refused as written.

## Decisions
Decline, with the idiom. No authored surface. The story body is the decline record, with the probe output above. Document the idiom as the guide section `## A view served from a replica`, placed after `## A view declares its consistency` in `website/docs/guides/specify/values-and-views.md`. The section covers:
- one logical component;
- views `eventual`;
- a `role` / `write-endpoint` setting;
- a shared store as a `requires:` entry;
- per-environment values in the environment binding;
- the statement that write-primary placement and read-your-writes at a named instance are out of scope for the specification.

No format bump (`ess/22` and `ess-conformance/43` unchanged). Reply on #432 with this decision.

## Acceptance
- replica_read_idiom_validates: a specification with one component, an `eventual` view, an enum `role` setting and `stateless: true` + `requires:` validates with no diagnostic.
- stateful_replicas_without_a_shared_store_are_refused: `stateless: false`, `replicas.min: 2` reports `ESS-TOPOLOGY-004` with the shared-store hint.
- topology_environment_placement_key_is_refused: `environments:` on a workload is an unknown-field refusal naming `replicas, stateless, requires`.
- replica_only_component_is_refused: a component owning, accepting and publishing nothing reports `ESS-COMPONENT-007`.
- replica_read_guide_section_states_the_idiom: `website/docs/guides/specify/values-and-views.md` has the heading `## A view served from a replica`, directly after the section `## A view declares its consistency`. The section text contains "one component", "`consistency: eventual`", "`role`", "`requires:`", "`ess-environment/1`", "write-primary" and "out of scope". Its fenced model is the one `replica_read_idiom_validates` compiles. The case reads the page, so it fails when the heading, a phrase or the model is missing. It lives in `crates/specify/ess-compiler/tests/replica_read_idiom.rs`.
- site_navigation: `crates/edge/ess-xtask/tests/site_navigation.rs` still passes, and the existing `#a-view-declares-its-consistency` anchor still lands (`website/docs/guides/write-a-specification.md:67`).

## Scope
- website/docs/guides/specify/values-and-views.md  cited — new section after "A view declares its consistency" (:272)
- crates/specify/ess-compiler/tests/replica_read_idiom.rs  inferred — new test beside `component_settings.rs`, which compiles inline fixtures the same way; also reads the guide section
