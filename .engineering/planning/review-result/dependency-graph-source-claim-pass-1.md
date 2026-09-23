---
format: aep.planning-md/1
id: review-result:dependency-graph-source-claim-pass-1
kind: review-result
status: active
title: Dependency graph complete source claim examination 1
relations:
- reviews: task:consumer-accounting-dependency-graph
revision: 1
---
needs-revision

# S5 source-to-claim admission review, pass 1 of 2

Review unit: the complete corrected S5 candidate ledger and its pinned source in the retained tree. I examined all 681 `Supported` rows and all five `AggregateClosureCandidate` rows against the dependency-graph implementation and the assertions in `evolution_dependency_graph_accounting.rs`. I did not run Cargo, alter source, or repeat the supplied green checks.

The reviewed artifacts match the brief:

- candidate ledger SHA-256: `f09bbd26b8540942b5de59b14e72028e35ba2e2f8cacb63008d75c349850e964`
- source test SHA-256: `10bc42b976575af2deba5606b8d74ed39f6ee1f469cee74573c04103ff5741c1`
- unchanged `graph.rs` SHA-256: `4a092fea456bee7533ff97dd0292c921dcf16b6b207a48b025f40e9e26f50883`

The ledger has 686 distinct rows and exactly matches the obligation source by model, set, index, current shape, old shape, and family. Its 681/5 disposition split is exact. The five aggregate candidates are exactly C21, C28, C29, C55, and C56, and each correctly makes no whole-parent support claim. Fixture hashes and named cases also resolve to the pinned source. Edge direction, deterministic ordered storage, the retained CC[8] view/parameter-type case, and the five aggregate classifications have no finding.

## Findings

### 1. Twenty supported rows retain the superseded unresolved-M3 control

The correction admitted A40 and A42 with the separating `unmapped_context_and_read_types_have_independent_graph_edges` case and a red/restored M3 result. Twenty other `Supported` rows still say that M3 is unresolved and that no outcome is recorded. The stale text starts at ledger line 102 and occurs on these exact rows:

| Coordinate | Model |
|---|---|
| A1 | `rust:ess_compiler::ir::ResolvedBinding/field/cause` |
| A3 | `rust:ess_compiler::ir::ResolvedBindingCause` |
| A4 | `rust:ess_compiler::ir::ResolvedBindingCause/variant/Event` |
| A5 | `rust:ess_compiler::ir::ResolvedBindingCause/variant/Event/field/0` |
| A6 | `rust:ess_compiler::ir::ResolvedBindingCause/variant/Periodic` |
| A7 | `rust:ess_compiler::ir::ResolvedBindingCause/variant/Periodic/field/0` |
| A39 | `rust:ess_compiler::ir::ResolvedPeriodic` |
| A41 | `rust:ess_compiler::ir::ResolvedPeriodic/field/contract` |
| A84 | `rust:ess_domain::binding::BindingCause` |
| A85 | `rust:ess_domain::binding::BindingCause/variant/Event` |
| A86 | `rust:ess_domain::binding::BindingCause/variant/Event/field/0` |
| A87 | `rust:ess_domain::binding::BindingCause/variant/Periodic` |
| A88 | `rust:ess_domain::binding::BindingCause/variant/Periodic/field/0` |
| A89 | `rust:ess_domain::binding::BindingSpec/field/cause` |
| A104 | `rust:ess_domain::binding::RawTrigger/field/periodic` |
| A119 | `rust:ess_domain::binding::periodic::HostInputContract/field/owner` |
| A135 | `rust:ess_domain::binding::periodic::PeriodicCause/field/host` |
| C1 | `rust:ess_compiler::ir::ResolvedBinding` |
| C13 | `rust:ess_domain::binding::RawTrigger` |
| C14 | `rust:ess_domain::binding::RawTrigger/field/event` |

This is an evidence-integrity conflict inside the admitted ledger: the rows are `Supported`, while their stated control explicitly withholds the causal conclusion. Replace the superseded text on all twenty rows. Rows whose full-object claim includes periodic context/read must cite the separating case and its M3 result; rows that claim only cause-arm behavior should narrow their assertion and control to that behavior rather than inherit the unrelated unresolved paragraph.

### 2. Cross-family parents and the selection schema arm lack their required evidence unions

Ledger line 4708 begins a finite group whose evidence observes only one of the changed branches:

- A232 `wire:RawSpecFile#/definitions/AuthoredMappingSchema` and A233 `/anyOf` cite only the accessor-string observation although their schema includes both the string and `SelectionMapping` arms. A236 `/anyOf/1` and A237 `/anyOf/1/$ref` are the selection arm itself but still cite only the accessor-string observation. The existing selection case at source line 608 is the relevant second branch.
- C1 `rust:ess_compiler::ir::ResolvedBinding` and C9 `rust:ess_domain::binding::BindingSpec` require periodic, selection, and binding external-reference observations.
- C6 `rust:ess_compiler::ir::ResolvedMappingValue` and C11 `rust:ess_domain::binding::MappingSource` require periodic-host, selection, and event-accessor observations.
- C12 `rust:ess_domain::binding::RawBindingSpec`, C45 `wire:RawSpecFile#/definitions/RawBindingSpec`, and C46 its `/properties` parent require selection and binding external-reference observations.
- C47 `/properties/mapping` and C48 `/properties/mapping/additionalProperties` additionally require the periodic-host and event-accessor mapping observations.

The retained `core-external-refs.yaml` control writes `refs:` only on a command and a component (fixture lines 67 and 78); it contains no binding. Its test at source line 740 therefore cannot establish the `refs` member of the three binding parent shapes. This is the same nine-parent union boundary already accepted during S4 source admission, now applied to the graph entrypoint.

Attach the existing periodic, selection, and accessor graph cases to the exact rows they exercise; attach the existing selection case to A236/A237 and make A232/A233 the union of both schema arms. Add one bounded binding-refs changed/control observation proving identical nodes and edges for otherwise equal bindings, then include it in the binding parent unions. No new graph semantic is required.

### 3. Seven payload/sets schema rows cite an unrelated subject-state observation

At ledger lines 9798, 9818, and 13465-13545, these rows are attributed to the two subject-state cases:

- A489 `wire:RawSpecFile#/definitions/RawOutcome/properties/payload/additionalProperties/additionalProperties/$ref`
- A490 `wire:RawSpecFile#/definitions/RawOutcome/properties/sets/additionalProperties/$ref`
- C57-C59, the `payload` container through its nested `additionalProperties`
- C60-C61, the `sets` container and its `additionalProperties`

The subject-state fixture writes `when_subject_state` and writes neither `payload` nor `sets`. It cannot establish these schema rows. Their actual delta is the value schema changing from a string to `RawPayloadSource`, which the graph deliberately does not consume.

The existing changed/control case `where_an_emitted_payload_field_comes_from_changes_no_node_and_no_edge` at source line 676 is relevant to the payload subset and can replace the unrelated evidence there. No S5 fixture writes `sets`, so A490 and C60-C61 need one bounded sets-source changed/control observation with exact node/edge equality. Keep the accepted C55/C56 aggregate disposition unchanged.

### 4. Three changed reading parents are supported only by type-body edges

C8 `rust:ess_compiler::ir::ResolvedType` at ledger line 12537, C26 `rust:ess_domain::types::NamedType` at line 12884, and C27 `rust:ess_domain::types::RawNamedType` at line 12901 changed because the reading contract became a member of those parent shapes. Their current evidence enumerates type nodes and type-body edges but never varies the reading member. Constructor success and complete body-edge enumeration do not establish this omitted descendant.

The required observation already exists: `a_types_clock_reading_contract_changes_no_node_and_no_edge` at source line 731 compares the reading fixture with its control and asserts exact graph equality after confirming different compiled input. Add that case to all three parent rows and update their assertions to state the union of body-edge behavior and reading no-effect behavior.

## Review boundary

The supplied 30-case, 164-test, ignored-test, Clippy, formatting, and mutation outcomes were inspected as evidence and were not rerun. Native qualification and full-matrix mutation remain root-owned. The four findings above are finite ledger/source-attribution corrections; they do not request production graph changes, broaden the graph contract, or disturb the accepted five-identity aggregate group.

```findings
- file: s5/attribution-correction/non-authority-dependency-graph.json
  line: 102
  category: contract-drift
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: Twenty Supported rows retain the superseded statement that M3 is unresolved; replace that text everywhere and attach the separating context/read observation to every full-object row whose claim includes those fields.
- file: s5/attribution-correction/non-authority-dependency-graph.json
  line: 4708
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: A232-A233, A236-A237, C1, C6, C9, C11-C12, and C45-C48 lack the selection-arm or cross-family observation unions their changed shapes require; the binding parents additionally need a binding-refs no-effect pair because the cited refs fixture contains no binding.
- file: s5/attribution-correction/non-authority-dependency-graph.json
  line: 9798
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: A489-A490 and C57-C61 are payload/sets schema rows attributed to a subject-state fixture that writes neither member; use the existing payload-source no-effect case for payload and add one exact sets-source no-effect pair for sets.
- file: s5/attribution-correction/non-authority-dependency-graph.json
  line: 12537
  category: property
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: C8, C26, and C27 claim changed reading parents using only type-body evidence; include the existing reading-contract changed/control graph-equality case in each parent union.
```
