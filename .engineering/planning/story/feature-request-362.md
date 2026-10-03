---
format: aep.planning-md/3
id: story:feature-request-362
kind: story
status: draft
title: Witness aggregate grouping by lifecycle state
refs:
- provider: github
  reference: beyond10x/ess#362
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Outcome

Synthesize exact aggregate rows grouped only by lifecycle state under the suite's explicit initial-state contract, without requiring an artificial extra domain key.

## Fit review

1. Need: count the items in each reached lifecycle state. The requester supplies no new syntax. Minimal Item with Open/Done lifecycle and ByState(group_by:[state],count) validates on installed ESS0.51.0. Synthesis produces5 scenarios and1 ESS-SYNTH-016 refusal, no aggregate witness. Private probe362-state-only-closed source SHA b9db0ba099242296ab75d26ffd452663070fb7c0f74138cd9bb513f206d49c67.
2. Class: synthesis capability gap for admitted state-valued grouping. docs/design/aggregate-views.md:152–174 defines grouping and equality, including state. aggregate.rs:935 already represents state keys;:978 refuses groups without scoped String/Uuid keys or parameters unless the view is an additive ungrouped delta.
3. Existing expression: validates as written. Adding team to the grouping changes the result and requires consumer aggregation. Ungrouped count/sum uses snapshots/deltas (aggregate.rs:2903); this is an established alternative mechanism but does not implement grouped rows. Existing grouping by state plus a scope is useful prior coverage, not a solution for state-only grouping.
4. Fit: preserve lifecycle reachability and exact aggregate evaluation. The refusal was justified by a shared persistent target, while story312 now declares empty logical scenario state for freshly synthesized34/35 suites. Use that explicit contract only where actually carried; never retrofit it into historical suites. Compose creation, transitions, empty groups, filter truth and all aggregate operators through existing evaluator authority. Native/Go/TypeScript/WASM and browser original-authority admission must agree. A begin callback accepts isolation, but does not prove physical reset; faulty retained-row behavior needs an actual target control.
5. Second adopter: warehouse reservations grouped Pending/Committed/Released and support cases grouped Open/Resolved both require state counts without a tenant discriminator. These are the same lifecycle/grouping semantics, not consumer policy.
6. Cost: no new ESS syntax. The already unreleased34/35 initial-state contract may provide sufficient authority; verify before implementation and allocate no additional format by assumption. Existing shared-target docs and assertions need explicit reconciliation. Add grouped-state actual target/mutation tests, not a hardcoded expected count.
7. Alternatives: change nothing/add artificial key (changes domain query); grouped before/after deltas (more persisted observation and complex absent-group/non-additive semantics); exact arrangement under explicit empty-state authority (preferred for current fresh suites). Historical suites preserve their existing semantics. The preferred choice still needs a red/green run on current source before acceptance as implemented.

## Decisions

Accept, redesigned around existing admitted grouping and explicit suite isolation. Do not simply delete AggregateUnscoped: prove scenario setup, absolute observations and empty groups remain truthful. This draft does not claim312 itself fixed362.

## Acceptance

Measure missing ByState/aggregate on current source. Healthy target executes every reached group; ignore-state, count-all-rows, drop-group, retain-prior-scenario and stale-transition targets fail. Cover one state, multiple states, empty groups, filters and count/sum plus existing admitted non-additive aggregations. Generated native/Go/TypeScript/WASM run the same actual suites; legacy metadata remains unchanged and old readers refuse new authority as required.

## Scope

Cited aggregate planner and tests under crates/verify/ess-conformance/src/synthesize/aggregate.rs and tests/aggregate_views.rs. Inferred tests for isolated state groups and relevant initial-state actual target controls. Coordinate sequencing with361 on the same planner, final312 integration and serial synthesis ownership. No implementation dispatched yet.
