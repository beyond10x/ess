---
format: aep.planning-md/3
id: component-design:aggregate-observation-integration-boundary
kind: component-design
status: draft
title: Aggregate observation source ownership and shared execution dependencies
relations:
- designs: story:feature-request-361
- designs: story:feature-request-362
revision: 1
---
## Purpose

Bind the actual integration dependencies and proposed ownership for the accepted #361/#362 aggregate-result observation direction before implementation is dispatched. This records source inspection at runtime carrier c2c4f01c6cfe99c6a16db5670669fb9774ab6bb9 and the current uncommitted #292 interface. It is not a completed executable wire specification, an implemented adapter, or permission to edit the transport owner's source.

## Design already reviewed

consumer-aggregate-wire-pass2 accepted checked Contract to RawSpecFile construction, ordinary Specification assembly and compilation, semantic re-projection equality, and foreign profile validation. The private revised design SHA256 a32c5a65d92d132b6356aab3c3a0b2db553f507a9ef066e761d8cbe2fabc64cf withdraws delivery_cut/frontier from admitted grammar. Its source completeness, exact query cuts, dynamic values, all six aggregates and disclosure obligations remain required. Static-only support cannot close #361/#362.

## Actual shared native boundary

The #292 owner inspected its current source and returned shared-context-interface.md, SHA256 ba00692007103aa15962296f57c1d99c062381edf6ee9dea72177c234407fef5, retained in private ess-292-history-probe-20261003/implementation. Latest executed checkpoint there is 24/0; subsequent edits are unexecuted. No source is frozen by this manifest.

The crate-visible history API currently exposes State, Alternatives, Transition and execute. State offers lifecycle/text-state reads only; its row map is private. history::execute accepts an EssIr, state, exact input, generated slots and operation identity, then supplies history policy with no caller and open externals. Shared Context/Row/Value/Facts/import/publish remain restricted to the execute subtree. The apparent pub(crate) values are inside a private module and are not a callable aggregate API. No expected outcome is accepted by the executor; history outcome filtering belongs later in linearize.

Therefore the aggregate observer must not call history::execute as a substitute for its own actual caller/provider/response authority. After #292 freezes, a separately scoped child adapter under interpret::execute can reuse responding_core and staged take semantics, or a reviewed narrow crate-private wrapper can expose those operations. It must accept actual typed setup/inputs/caller/provider authority, return source-selected outcomes plus state, and never accept expected aggregate rows, preferred outcomes or history candidate-search state.

The private state currently addresses rows by (QualifiedName, Node). Known identities arrive from supplied inputs, imported Store or actual recorded history keys. It does not provide a symbolic address for a created row whose implementation-generated identity is unobserved. That case matters even when count/state/group facts are otherwise known: inserting a guessed Node or expected group key would invent identity authority. The aggregate adapter design must bind symbolic row identity/alias uncertainty or an actual source-admitted observation before integration. Do not expand #292 implicitly or call this problem solved by its typed unknown field values.

## Finalization order and existing hooks

Source inspection of synthesize.rs::synthesize_invocations shows aggregate generation precedes preconditions, fixtures::install, now_offset::install and fresh-format selection. synthesize() additionally performs caller synthesis, cross-caller work, withdrawal and disclosure augmentation. Thus a program cannot freeze final step indices while aggregate::aggregates initially drafts its actions. Capture the original model and exact caller-stamped finalized steps, then lower observation metadata after every insertion/withdrawal affecting the scenario. Include the retained precondition prefix exactly once; neither omit it nor replay the original untruncated prefix. The integrator owns this shared finalization change.

admission.rs::step_value and expectation own the original-byte closed vocabulary before typed admission. scenario.rs owns the persisted step and expectation enums. runner.rs::step, expect_view and Run own execution, query result association and retained scenario state. The new native module must use those hooks, retain exact QueryView/EventuallyView cut identity, and clear private state at the same scenario lifecycle boundary. runner/disclosure.rs and Run::record remain authoritative for one-time data; observer failures/budget exhaustion must not print private raw values before redaction.

interpret/views.rs::query already filters typed concrete Store rows and projects/ranks/paginates them. It cannot consume the new private abstract state by assumption. Existing aggregate.rs arithmetic remains a reuse candidate, with exact-number, Optional and group equality semantics checked rather than copied by name.

Go emission concatenates embedded files in go/mod.rs::runtime; TypeScript emission lists assets explicitly in ts/mod.rs. New persisted expectation support therefore needs emitter packaging and actual generated-package admission/execution tests as well as edits in runtime.go/runtime.ts. Existing explorer loaders are not an admitted semantic Contract compiler. Keep native differential admission vectors for every validation family reachable in the projected closure; JSON shape checks alone are insufficient.

## Proposed file ownership

This inventory records hooks and anticipated files, not a dispatch to all owners at once.

| Surface | Paths | Evidence / owner |
| --- | --- | --- |
| Finalized program producer | src/synthesize.rs; src/synthesize/aggregate.rs | Cited existing ordering/planner; aggregate owner plus shared integrator coordination. |
| Persisted vocabulary and early admission | src/scenario.rs; src/admission.rs; src/lib.rs | Cited existing hooks; closed DTO/major selection must be fully bound first. |
| Native model/program module | src/aggregate_observation.rs and src/aggregate_observation/ | Inferred new module; producer, original-byte reader, checked RawSpecFile reconstruction and semantic re-projection. |
| Shared execution child adapter | src/interpret/execute.rs; src/interpret/execute/aggregate.rs | Inferred new adapter and parent registration after #292 freeze; private row/address access needs explicit design. |
| Native runtime and disclosure | src/runner.rs; src/runner/disclosure.rs; src/aggregate.rs | Cited existing state/query/disclosure/arithmetic hooks; no duplicate native command executor. |
| Generated runtimes/packaging | src/go/mod.rs; src/go/runtime.go; src/ts/mod.rs; src/ts/runtime.ts | Cited packaging and expectation readers; new helper paths need exact scope before editing. |
| Focused Rust tests | tests/aggregate_observation_admission.rs; tests/aggregate_observation_execution.rs; tests/support_aggregate_observation/ | Inferred independent model roundtrip, malformed-byte and actual healthy/fault target corpus. |
| Actual WASM | crates/generate/ess-synth/tests/aggregate_observation_wasm.rs | Inferred test using the existing actual WASM execution route. |
| Full browser | Existing browser-response-conformance product/fixture scope, after owner freeze | Inferred test reuse; library WASM alone is insufficient. No parallel edit to that worker's tree. |

Paths beginning src/ or tests/ above are relative to crates/verify/ess-conformance. Compiler/domain remain dependencies, not implicitly editable ownership. New executable harness files are Rust. Existing generated-language runtime assets retain their established repository role. Source, evidence and format docs remain in the single held bundle; no additional delivery PR.

## Missing binding completion authority

target.rs::ObservedInvocation currently carries binding, command and input only. InvocationObservationRequest carries binding, command, correlation and a deadline. observe_invocations returns a vector, not a completion frontier, terminal outcome, causal occurrence inventory, committed effect token or no-further-retry proof. This API proves mapping observations; it cannot by itself prove the complete rows affected by bindings at an aggregate query cut.

The transport capability owner must coordinate an actual source-scoped causal boundary for relevant binding executions: which source event occurrences and descendants are included; how completion, ordering and retry effects are distinguished; how actual outcomes and source-owned generated values are observed without borrowing assertions; and how a queried projection is aligned to that boundary. Exact request/response admission, timeout/unsupported behavior, disclosure and an independent healthy/fault adapter proof are required before a wire/target extension is bound. These are required facts, not a proposed API name or authority to implement ess-transports here.

Unrelated bindings must not fence a query whose complete transitive dependency analysis proves them irrelevant. Conversely, a binding-affected query lacking completion authority cannot pass with a partial inventory. Record it as an unresolved completion dependency in the same full-support bundle. No synthetic drain opcode, guessed source completion event or blanket model-has-bindings refusal.

## Next evidence, in dependency order

1. Freeze and independently review #292, then bind the separate scenario adapter including actual caller/provider authority, typed setup, abstract rows and unknown identity behavior. Concrete/history public APIs retain their behavior.
2. Complete the closed Contract DTO and foreign validator inventory, including every reachable normal compiler validation family; prove admitted reconstruction/re-projection and one-change invalid original-byte vectors before using the program in expectations.
3. Revalidate the held suite/34 and /35 publication status with the sole integrator before binding their new vocabulary. Local inspection of the owner's e68684ef candidate shows the earlier feature-selective fresh-format code rather than the held unconditional /34 selection; this is a source comparison, not remote proof that no tag/version was published elsewhere.
4. Execute complete-prefix, static and fixture-dependent grouping, observed origins, unknown generated identity, related/set effects, count/sum/min/max/mean/count_distinct, filtered/nonmatching/empty groups and one-time faults through actual native/Go/TypeScript/WASM/full-browser routes. Keep full exactness and faulty-target detection, not only successful admission.
5. Bind and prove the relevant-binding completion capability with its separate transport owner. All source/evidence and broad required gates remain in the one integration bundle; earlier independent cases may be developed but do not close the story before this obligation is met.

All current work here was read-only source inspection; no compiler, test, target callback, remote gate or transport edit was started.
