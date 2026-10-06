---
format: aep.planning-md/3
id: story:optional-value-invariant-observation
kind: story
status: draft
title: Observe declared value invariants through all wrapped view positions
relations:
- decomposes: epic:downstream-reported-gaps
- informed_by: story:feature-request-293
- serves: vision:O2
- depends_on: story:browser-response-conformance
- depends_on: story:feature-request-292
scope:
- confidence: inferred
  path: crates/edge/ess-cli/src/main.rs
- confidence: inferred
  path: crates/edge/ess-cli/tests/browser_response_conformance.rs
- confidence: inferred
  path: crates/edge/ess-cli/tests/fixtures/browser-target/src/lib.rs
- confidence: inferred
  path: crates/generate/ess-synth/tests/wrapped_value_invariant_wasm.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/admission.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/aggregate_delta.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/authored.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/count_json.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/counts.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/coverage.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/coverage_build.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/defined_aggregates.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/fixtures.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/go/mod.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/go/predicate.go
- confidence: inferred
  path: crates/verify/ess-conformance/src/go/runtime.go
- confidence: inferred
  path: crates/verify/ess-conformance/src/leaf_payloads.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/lib.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/mutate.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/now_offset.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/presence.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/quoted_predicate_format.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/runner.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/runner/page.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/scenario.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/value_invariant.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/text_match_format.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/ts/mod.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/ts/predicate.ts
- confidence: inferred
  path: crates/verify/ess-conformance/src/ts/runtime.ts
- confidence: inferred
  path: crates/verify/ess-conformance/src/value_invariant.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/view_paging.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/web.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/web_execution.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/web_execution/bundle.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/web_execution/presentation.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/witness.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/support_wrapped_value_invariants/mod.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/support_wrapped_value_invariants/models.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/support_wrapped_value_invariants/targets.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/wrapped_value_invariants.rs
- confidence: inferred
  path: docs/design/wrapped-value-invariant-observations.md
revision: 20
---
## Outcome

Synthesize executable, nonvacuous observations for declared invariants at every admitted wrapped view position: Optional, List, Map, Union, nested combinations and productive recursion. Apply the same closed assertion semantics across native, generated Go/TypeScript, WASM and the actual browser product. Legal absence or empty containers must not become failures or substitute for an actual applicable witness; no required position may disappear from inventory.

## Fit review

1. Need: a bounded Integer newtype declares value >= -10 and value <= 10000; an entity, command input, event and read-your-writes view carry Optional of that type. The type is already expressible. Actual synthesis admits the source and emits three scenarios but no value-invariant observation. No new syntax was requested. The brand-free compiled source and direct-position control are retained in the private evidence set ess-optional-invariant-observation-20261003.

2. Classification: conformance coverage gap, not a failure to inhabit an Integer bound. Refusal ESS-SYNTH-013 explicitly says no view publishes a usable position outside Optional/List/Map/Union. The implementation in crates/verify/ess-conformance/src/synthesize.rs, value-invariant position walk reaches and holds_at, intentionally does not traverse these containers because unconditional rebasing would make absence Unknown. The refusal is truthful; the all-features requirement calls for supported observation semantics rather than removal of the diagnostic alone.

3. Existing expression: the source uses admitted ess/16, a named Integer newtype, existing invariants, existing Optional and existing view fields. The actual pinned CLI successfully compiled and synthesized both source and control. Replacing every Optional<demo.items.Bounded> with demo.items.Bounded, consistently across the model, adds demo.items.Bounded/invariant/at/demo.items.Items/note and changes counts to four scenarios, zero refusals. That replacement changes the consumer contract and is a diagnostic control, not an acceptable fix. Story synthesized-inputs-satisfy-invariants-over-nested-members addresses input/guard witnesses (#234); it does not cover this missing view observation. Story feature-request-293 changes generated explorer draws, not fixed-suite observation synthesis.

4. Composition: retain the distinction between absent Optional and present false/zero/empty values. A design must prove a nonvacuous present witness and apply the invariant to that value, while permitting legal absence; merely adding a conditional that always sees absence is insufficient. Audit named wrappers, nested structs and Optional nesting, and explicitly inventory List/Map/Union positions refused by the same walk. Each supported observation must execute through native, generated Go/TypeScript and the full browser runner with the same assertion semantics. No source, suite or predicate format change is selected yet; any required format migration must be bound before implementation. Runtime history validation and explorer tests cannot substitute for fixed-suite target observations.

5. Second adopter: an inventory item has an optional bounded replenishment threshold; a reservation has an optional positive hold duration. Both require checking the declared value constraint whenever the optional value exists, without making the field mandatory.

6. Cost: known production seam is synthesize.rs (value-invariant position inventory, arrangement and assertion rebasing), with Rust-driven conformance tests and actual target controls. Existing predicate/step readers and all runtime emitters must be audited before fixing scope. A new syntax or format is not yet justified. Full affected surface and container handling remain design work, not an implied two-line traversal change.

7. Alternatives: keep the explicit refusal (honest but incomplete); make the field mandatory (changes the contract, rejected); restate every named-type invariant as an entity invariant (possible diagnostic workaround, duplicates meaning and does not cover the original type obligation); or arrange actual present witnesses and generate correctly guarded/quantified observations using the existing type structure (preferred direction, pending binding design and admitted cross-runtime proof).

## Decisions

Accept, redesigned: use the independently approved closed ValueInvariants assertion and finite position graph in docs/design/wrapped-value-invariant-observations.md. Conditional rebasing of the legacy Satisfies predicate is insufficient because absence-only observations can pass vacuously and untyped fact paths do not preserve container semantics. Preserve existing direct-only assertion identities and bytes.

The binding semantic design is local commit 4c78066fde612a556d6a00aa653fe39bb48f1785 on design/wrapped-value-invariants-20261003, based on frozen runtime c2c4f01c6cfe99c6a16db5670669fb9774ab6bb9. File SHA256 b58b77b9ef68abe18aabd681d13144a5ac6e8ecc1921d8fd6da543d669aad29e. From Contract and evidence onward it is byte-identical to approved candidate v3; only the adoption preamble changed. Both author and committer are the bot. No production implementation, remote publication or format reservation occurred.

Ordinary36/coverage37 remain proposed numbers pending integration-owner catalog reconciliation and exact generated-carrier scoping. Story stays draft with dependencies on browser-response-conformance and feature-request-292; do not edit those owners' active shared surfaces. The existing single held-bundle integrator owns eventual delivery. Prior exploratory scope, cost and candidate paragraphs below are retained history and are superseded by this decision and the current typed scope.

## Acceptance

Complete all eleven groups in docs/design/wrapped-value-invariant-observations.md, Actual acceptance matrix, with admitted source and independent healthy/fault targets across native, generated Go, generated TypeScript, WASM and actual CLI/browser routes. This includes every wrapped and recursive position, nonvacuous occurrence/row selection, exact typed/lexical facts, deterministic cross-runtime logical work traces and N-1/N/N+1 boundaries, forged authority rejection before callbacks, old-reader refusal and historical-byte preservation, and incomplete-inventory refusal through every serialization/reconstruction/packaging path. Retain the original mixed-position red and controls proving missing observers and bad later members are detected. A named refusal or library-only WASM run does not establish full browser feature support. No matrix group is yet implemented or discharged by design approval.

## Evidence

Both actual CLI invocations exited 0; synthesis refusals are reported counts and diagnostics, so exit 0 alone is not a green coverage result. Baseline: three scenarios, one ValueInvariantUnwitnessed at None; direct-position control: four scenarios, zero refusals. No target was executed by this discovery probe.

| Artifact | SHA256 |
| --- | --- |
| Pinned CLI used for both invocations | d397e76cd811915e6efb9bfd33e0a7c7483ce65d9518ff229de0cbcab56fd23b |
| Original Optional source | b2079d4917bcc6e7901a4cf43fc7f49a0f60439186fe5f4b9117813a5896c6e2 |
| Original synthesis log | 8304f1905598c3061872ed969a1c119df0b0699c1dfedca5a7a33d2121fa00ce |
| Original suite | 8516ef666391f9c2e203fb2b4873b522f3004848b5a617a10a1d53dc5ba74ee7 |
| Direct-position control source | 27a08f789426bf952494189f48d61aee9fa80bca0643560d0ac10bd923cedd0d |
| Control synthesis log | c7947dd662413f2956264dc9c6141257ddc4b77dac8a4d05b0bbe5bba5ed8733 |
| Control suite | 8b65bd408b31f8b7bbcf1f42846c19b4e93dfcde3b79f410b0827688685ecebd |

The actual command was ess verify conform synthesize --path source.yaml --out suite.json, then the same invocation for direct-control.yaml and direct-control-suite.json. The pinned CLI predates the #293 production changes; that unit changes explorer assets only. This establishes an existing source-observation gap, not a regression introduced by Optional random drawing.

## Mixed-position silent omission regression

Root independently executed a stronger source control after read-only scoping identified that positions_of omits Optional/List/Map/Union before inventory. The original Optional note remains; the entity and view additionally carry a required_note of the same Bounded type, set to concrete zero in each source outcome. Actual retained CLI synthesis succeeds with four scenarios and zero refusals. Its only value-invariant scenario is demo.items.Bounded/invariant/at/demo.items.Items/required_note; the note position has neither an invariant scenario nor a coverage refusal. A named inventory assertion fails with OPTIONAL_INVARIANT_POSITION_OMITTED_WITH_ZERO_REFUSALS (exit1). No target execution is claimed.

Private evidence ess-optional-invariant-observation-20261003/mixed-position:

| Artifact | SHA256 |
| --- | --- |
| model.yaml | b1e4d4cd70cf265481164ef4a8feb8f06f113f626a098c55985dadb8625a5555 |
| suite.json | 3fc4056059eb2f9fa920f87491dcb9c4fe3d2673ff274b581a356b1bfd91f60e |
| synthesis.log | e3621f5fef2734ca0febaa62c72ba3f1a65a6daf7fdda84162097a32905c2cae |
| obligation-red.log | 090e63f6c603751b5a4fe99650ba8417c09a32ccc0a1348c4bb42321eeb2db37 |

The pinned CLI hash remains d397e76cd811915e6efb9bfd33e0a7c7483ce65d9518ff229de0cbcab56fd23b. First invocation used the parent working directory and failed to find model.yaml; its log/exit are retained at the evidence root, not counted as the product red. The corrected invocation in mixed-position produced the stated suite. No compilation occurred.

This strengthens the required inventory rule: a supported direct position must not hide omitted wrapped positions of the same type. Fixing only the at=None refusal case is insufficient. Legal absence must remain legal while a present witness is actually asserted, and sibling container dispositions must be per position rather than inferred from type-level presence of any one successful scenario.

## Scope

Current machine-readable scope follows the reviewed full-feature v3 design: 43 explicit paths, recorded as inferred planned implementation/audit surfaces rather than an implemented diff. Three earlier Optional-only test paths are retired in favor of shared full-feature tests and the existing browser host. The four web execution/producer paths are coordination and serialization/admission audit scope; change them only if the shared checks prove insufficient, preserving target ABI and the browser owner's implementation. Exact generated format/catalog carriers remain to be enumerated after number reconciliation. The dependencies on browser-response-conformance and feature-request-292 prevent scheduling conflicting work.

Binding design: docs/design/wrapped-value-invariant-observations.md. The CLI scope entries enumerate the exact paths and supersede the original exploratory scoper's Optional-only list. Core assertion/graph and synthesis modules, exhaustive feature visitors, Rust-driven target tests and existing embedded runtime assets are included. No transport implementation or interpreter command-evaluator change is authorized by this story.

## Scope uncertainties and full-feature boundary

The scoper changed and executed nothing. The mixed-position red is root execution, independently linked in the evidence section. Proposed scope is medium-confidence, not implementation authorization.

List/Map/Union remain required parts of the full all-features objective, not accepted permanent exclusions. The initial Optional scope does not claim to cover them. Source inspection found native runner::row_facts currently publishes sequence presence only, while generated Go/TypeScript predicate fact projectors additionally publish counts/elements. Typed input map projection uses values in key order, whereas untyped runtime maps are flattened by member; input quantifier support therefore does not establish view-observation parity. Union path resolution and first-variant witness generation need variant-aware design. These are source-grounded concerns, not executed cross-runtime red/green results. Their implementation needs an explicit expanded or sibling scope before dispatch; recording them does not discharge full feature support.

For Optional, holds_at proves some row exists, not that the constrained value is present. Binding design must handle cleared/generated/converted values, filters, shared rows and eventual consistency without an absence-only pass. Existing Contains/Excludes might establish nonvacuity for identifiable arrangements; general sufficiency and compatibility are not established. Browser acceptance depends on the separately owned full execution product; declaration navigation or library WASM alone cannot prove it.

## Complete wrapped-position design candidate awaiting review

The existing read-only scoper produced a private complete wrapped-position design candidate, wrapped-value-invariant-design-candidate.md SHA256 c0956c68e84c40fa7b6797808dd08cef8d1aee4977b67514bfe5e408d3e8191e, reading this story revision 9 and runtime c2c4f01c6cfe99c6a16db5670669fb9774ab6bb9. Root rehashed it and read the proposed direction, scope and remaining decisions. No production source, AEP state, compiler or target execution was changed by that scoping work. The prior mixed-position root red remains unchanged.

The candidate proposes a closed typed ValueInvariants view expectation, preserving historical Satisfies semantics, with universal checks for every reached constrained value plus explicit actual nonvacuity. It covers Optional/List/Map/Union and recursive schema graph sites, source-preserving arrangement goals, per-position truthful refusal inventory and a cross-runtime independent healthy/fault matrix. This is materially wider than the earlier Optional-only inferred scope. It is a candidate for review, not an adopted binding design, new format reservation or source implementation authorization.

Root must still decide recursive site/back-edge identity, witness attribution where a view has no unique selector, exact format compatibility/allocation, shared byte/work budgets and DTO ordering, and browser bridge integration. No held version is assumed amendable merely from its number or release timing. Existing source refusals are not permanent feature exclusions; listing them without the required actual cross-runtime observations would not complete the user's all-feature requirement. Store scope will be expanded only after a coherent reviewed design is adopted, before production edits.

## Review direction for complete wrapped obligations

Root has requested a concrete second private design candidate before independent review or binding adoption. Keep the proposed typed expectation and universal checking of all applicable observed values, paired with an actual occurrence witness. The semantic subject remains the nominal invariant type at a view position, as current ScenarioId::ValueInvariant documents; it does not silently become a claim about every arranged row's provenance.

For a view without a source-proved unique selector, use an explicit any-observed-occurrence witness claim rather than a blanket absence-of-identity refusal. Where identity or a unique query selector is source-proved, bind the occurrence to that arranged row with a separate closed witness form. Neither form may infer uniqueness or confuse observed-value coverage with row-provenance proof. The revised candidate must make both forms and their admission rules exact.

Keep the held 34/35 authority unchanged for this independent feature. Root's local source inspection finds current main e68684ef supports suite majors 1 through 33 and held runtime c2c4f01c6 supports 1 through 35. The next pair 36/37 is a design proposal only, pending exact catalog/scope adoption; no format has been reserved or implemented here. The revised candidate must resolve finite recursive site/back-edge identity, canonical DTO and Map ordering rules, quantifier scope, source-arrangement versus observed authority, and truthful resource-exhaustion inventory. Resource limits must be justified against existing profiles, not chosen as an arbitrary narrower feature subset. This direction is not source implementation authorization; all new cross-runtime behavior still needs actual admitted healthy/fault evidence.

## Complete v2 candidate enters independent review

The private complete wrapped-invariant design v2 is frozen at SHA256 ca56df7e4687bd8e4402afe8380cff3c647a91b5c785e355652ac13d128c63c3. V1 remains unchanged. Root read the complete candidate. It specifies closed any_observed_occurrence and arranged_row witnesses, projected-identity/identity-query selector admission, finite typed path sites and recursive edge/site pairs, exact canonical identity, typed facts and quantifier context, deterministic work accounting and incomplete-inventory refusal. Ordinary 36/coverage 37 remain proposed only; held 34/35 are unchanged and no format is reserved.

The browser implementor now independently reviews the candidate against actual held runtime and browser bridge source, focusing concrete semantic contradictions, pre-callback admission, resource determinism, internal incomplete marker and exact integration seams. This read-only review runs beside the history owner's sole compilation lane. No wrapped-invariant production change or scope expansion is authorized until review and binding adoption. Required actual native/Go/TS/WASM/product-browser healthy/fault acceptance remains unchanged; the candidate is not executed support.

## Corrected v3 awaits independent finding verification

Candidate v3 is frozen at SHA256 7d68859b7b16edc3ee0e0448229353e45ee06c59e8b79b25e6e65fbbfebddf39. Root read its complete diff against reviewed v2. It adds normalized semantic numeric work spelling, explicit logical shape/observation/fact/predicate event charges independent of physical passes, and custom Serialize refusal plus constructor/field-copy/coverage/mutation/CLI/emitter audit for incomplete inventories. The actual browser Loaded/Runner/presentation seams and existing Rust browser test/fixture files replace the obsolete proposed new browser harness. Provenance checking remains explicitly separate from authenticating every assertion DTO against IR.

The original independent reviewer will re-review the three WVI-D2 findings after its bounded source-only browser authoring checkpoint. V1 and v2 remain immutable. This is not production implementation authorization, a format reservation, review-outcome clearance or executed all-runtime support. The browser dependency and shared source sequencing remain explicit; the next implementation wave must reconcile actual overlapping witness/reader source and the full held bundle before dispatch.

## Independent v3 design approval

Independent bounded review of candidate wrapped-value-invariant-design-v3.md SHA256 7d68859b7b16edc3ee0e0448229353e45ee06c59e8b79b25e6e65fbbfebddf39 is recorded verbatim as review-result:consumer-wrapped-invariant-design-v3. Private report SHA256 32fb384b30fc9adc22d15e2c57568ad2930e826461b84e6e298008d4dff2e21a; root read and rehashed it. Verdict approve, findings [], execution count zero.

The revision resolves the three v2 design findings through semantic canonical numeric work, a physical-pass-independent logical S/O/F/P debit schedule, and refusal of incomplete suites at generic serialization and reconstruction boundaries. The review outcome is a design correction only. It does not establish runtime parity, measured bounds, old-reader refusal, arrangement completeness or browser completion. Story remains draft, dependent on the active browser product; binding design placement, typed scope and shared-file sequencing remain required before implementation. No format number is reserved and no production source has been changed for this story.
