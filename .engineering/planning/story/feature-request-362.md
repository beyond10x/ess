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
revision: 3
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

## Combined aggregate design candidate

Root has inspected actual planner state and retained aggregate-group-selection-design.md SHA25693bac2e2a70b38933cbca97bc09603e0a54c4998bd9974f58961800ef8850e81 as a private candidate for361 and362 together. It is source analysis only, not implementation or new execution evidence.

The candidate separates group selection from non-group scoping, preserves tuples/related identity bindings, arranges actual reached rows, then evaluates the full source filter independently for each query. Typed Empty initial-state authority must reach the planner before exact unscoped observations; the current late final provenance selection is insufficient by itself. It asserts exact admitted group counts/aggregate values and observable distractors, with valid nonmatching selections only where the source domain permits them. State-only and Optional absent groups receive exact observations under that same authority, while historical admitted suites retain their contract.

Deleting the two refusals alone would overwrite group values and reuse the wrong query's admission flags. New tests must first assert the missing aggregate on the current source, then execute independent healthy/mutant targets in every runtime. Existing unrelated type/filter/identity pattern limitations and browser product execution remain explicit backlog work. Proposed production scope is aggregate.rs plus a narrow initial-state initialization in synthesize.rs; that shared file requires sequencing with the integrator's282/304 work. No source edits or production worker assignment yet; independent review is pending.

## Actual current-source aggregate regression red

Root compiled and ran the private Rust current-probe against unchanged runtime046db8a680, carrier6727e07363877925aa6a0f2bb1cb47b08d2348e3. Exact diff across crates/Cargo source is empty. All four brand-free inputs pass source assembly/compilation and emit fresh suite34 with scenario_initial_state:empty.

The copied-key control without a group parameter produces3scenarios,1aggregate,0refusals. Direct group-parameter source produces5scenarios,0aggregates,1ESS-SYNTH-017; copied related group-parameter source2scenarios,0aggregates,1ESS-SYNTH-017; state-only source5scenarios,0aggregates,1ESS-SYNTH-016. After validating the control, explicit assertions requiring the three missing aggregates fail with AGGREGATE_GROUP_SELECTION_GAP and terminal101. This is actual red on current code, not an installed-old-version observation or source-authoring failure. It does not execute target queries or claim an implementation.

Private retained current-probe Rust source SHA256a9cfb9c1206ead4a5358ea156b11c136ae50272a286a28e87fc2fb3bd611f86e; actualrunloga085425b0dc5ec35c36574d2a80152f008517b15f00ed78c0f3df38d7953dd4f; exitreceipt39b8dc3fc8b44765c8e6f1adee04c5b465e555ab791cc42d0d9e810d5b64297c; manifest03cf1c4a8db2f9c77a623d7ba484b60837d531b1f5e36c90ad168755c66ace70. Source YAML, lock, emitted suites and per-case JSON results retained under private ess-aggregate-intake-20261003/current-probe and its parent. One warm one-job compile finished0.85s. Root released the servers-cache lease after completion; no source edits or remote gate.
