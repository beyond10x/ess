---
format: aep.planning-md/3
id: story:optional-value-invariant-observation
kind: story
status: draft
title: Observe declared value invariants through Optional view positions
relations:
- decomposes: epic:downstream-reported-gaps
- informed_by: story:feature-request-293
- serves: vision:O2
- depends_on: story:browser-response-conformance
scope:
- confidence: inferred
  path: crates/edge/ess-cli/tests/optional_value_invariant_browser.rs
- confidence: inferred
  path: crates/generate/ess-synth/tests/optional_value_invariant_wasm.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/witness.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/optional_value_invariants.rs
revision: 10
---
## Outcome

Synthesize executable observations for declared value invariants reached through Optional view positions, with explicit treatment of sibling container positions, so an admitted type is not left untested merely because the view can also hold absence.

## Fit review

1. Need: a bounded Integer newtype declares value >= -10 and value <= 10000; an entity, command input, event and read-your-writes view carry Optional of that type. The type is already expressible. Actual synthesis admits the source and emits three scenarios but no value-invariant observation. No new syntax was requested. The brand-free compiled source and direct-position control are retained in the private evidence set ess-optional-invariant-observation-20261003.

2. Classification: conformance coverage gap, not a failure to inhabit an Integer bound. Refusal ESS-SYNTH-013 explicitly says no view publishes a usable position outside Optional/List/Map/Union. The implementation in crates/verify/ess-conformance/src/synthesize.rs, value-invariant position walk reaches and holds_at, intentionally does not traverse these containers because unconditional rebasing would make absence Unknown. The refusal is truthful; the all-features requirement calls for supported observation semantics rather than removal of the diagnostic alone.

3. Existing expression: the source uses admitted ess/16, a named Integer newtype, existing invariants, existing Optional and existing view fields. The actual pinned CLI successfully compiled and synthesized both source and control. Replacing every Optional<demo.items.Bounded> with demo.items.Bounded, consistently across the model, adds demo.items.Bounded/invariant/at/demo.items.Items/note and changes counts to four scenarios, zero refusals. That replacement changes the consumer contract and is a diagnostic control, not an acceptable fix. Story synthesized-inputs-satisfy-invariants-over-nested-members addresses input/guard witnesses (#234); it does not cover this missing view observation. Story feature-request-293 changes generated explorer draws, not fixed-suite observation synthesis.

4. Composition: retain the distinction between absent Optional and present false/zero/empty values. A design must prove a nonvacuous present witness and apply the invariant to that value, while permitting legal absence; merely adding a conditional that always sees absence is insufficient. Audit named wrappers, nested structs and Optional nesting, and explicitly inventory List/Map/Union positions refused by the same walk. Each supported observation must execute through native, generated Go/TypeScript and the full browser runner with the same assertion semantics. No source, suite or predicate format change is selected yet; any required format migration must be bound before implementation. Runtime history validation and explorer tests cannot substitute for fixed-suite target observations.

5. Second adopter: an inventory item has an optional bounded replenishment threshold; a reservation has an optional positive hold duration. Both require checking the declared value constraint whenever the optional value exists, without making the field mandatory.

6. Cost: known production seam is synthesize.rs (value-invariant position inventory, arrangement and assertion rebasing), with Rust-driven conformance tests and actual target controls. Existing predicate/step readers and all runtime emitters must be audited before fixing scope. A new syntax or format is not yet justified. Full affected surface and container handling remain design work, not an implied two-line traversal change.

7. Alternatives: keep the explicit refusal (honest but incomplete); make the field mandatory (changes the contract, rejected); restate every named-type invariant as an entity invariant (possible diagnostic workaround, duplicates meaning and does not cover the original type obligation); or arrange actual present witnesses and generate correctly guarded/quantified observations using the existing type structure (preferred direction, pending binding design and admitted cross-runtime proof).

## Decisions

Accept the coverage need under the standing full-backlog/all-features instruction. Record it separately from #293 so the exact fixed-suite refusal is not hidden by its passing exploration tests. This draft schedules design and implementation work; no implementation is claimed or dispatched. The finite-domain and recursive witness limitations in #293 remain separate retained limitations, not outcomes of this new item. No GitHub issue, PR or remote gate was created.

## Acceptance

For admitted Optional value-invariant positions, a synthesized suite obtains an actual present witness, passes an independent healthy target including legal absence, and rejects a target returning an invariant-breaking present value in every conformance runtime; sibling container positions have explicit evidence-backed dispositions and no silently omitted obligation.

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

Derived 2026-10-03 by `story-scoper`, reading canonical revision 2 at `a552b9434` and runtime source at `c2c4f01c6` — cited.

- **Primary surface:** value-invariant observation synthesis — cited.
- **Files:** `crates/verify/ess-conformance/src/synthesize.rs:10808` — cited; `value_object_invariants`, `positions_of`, `reaches`, `holds_at`, `rebased`, and `assert_satisfied` own obligation discovery, arrangement and assertions.
- **Required behavior:** inventory wrapped positions even when another direct position already succeeds; establish a present constrained value, permit legal absence elsewhere, and retain a specific unresolved obligation when presence cannot be established — cited from acceptance and the existing omission path.
- **Also likely:** `crates/verify/ess-conformance/src/witness.rs:262` — inferred; presence-directed arrangement may need to extend candidate search while preserving guard selection, type/entity invariants, aliases and existing resource bounds. Existing Optional witnesses already prefer present values, but that does not prove the resulting view position remains present.
- **Tests:** `crates/verify/ess-conformance/tests/optional_value_invariants.rs` — inferred new Rust-driven native and generated Go/TypeScript regression surface, including independent healthy, absent-only, invalid-present and mixed-position controls.
- **WASM tests:** `crates/generate/ess-synth/tests/optional_value_invariant_wasm.rs` — inferred new regression using the repository’s actual WASM test convention; library execution must remain distinct from product-browser execution.
- **Browser tests:** `crates/edge/ess-cli/tests/optional_value_invariant_browser.rs` — inferred new real CLI/browser test surface, dependent on a browser target-execution bridge; current declaration replay cannot satisfy this acceptance.
- **Documents:** a binding observation/compatibility design is required before implementation; no document path or format change has been selected — inferred.
- **Confidence:** medium — the omission and arrangement seams are established, but general nonvacuous observation, container semantics and browser execution still require design decisions — inferred.
- **Would collide with:** value-invariant synthesis, shared arrangement/candidate generation, and browser conformance integration-test surfaces — inferred.
- **Safety fact:** existing predicate syntax and runtimes already distinguish Optional absence from present scalar/struct values; existing Satisfies requires a nonempty view but does not require a present value at the constrained position. Walked through holds_at → assert_satisfied and the runtime evaluators: level 3, unproven by this read-only pass. A guarded invariant alone is insufficient — cited.

## Scope uncertainties and full-feature boundary

The scoper changed and executed nothing. The mixed-position red is root execution, independently linked in the evidence section. Proposed scope is medium-confidence, not implementation authorization.

List/Map/Union remain required parts of the full all-features objective, not accepted permanent exclusions. The initial Optional scope does not claim to cover them. Source inspection found native runner::row_facts currently publishes sequence presence only, while generated Go/TypeScript predicate fact projectors additionally publish counts/elements. Typed input map projection uses values in key order, whereas untyped runtime maps are flattened by member; input quantifier support therefore does not establish view-observation parity. Union path resolution and first-variant witness generation need variant-aware design. These are source-grounded concerns, not executed cross-runtime red/green results. Their implementation needs an explicit expanded or sibling scope before dispatch; recording them does not discharge full feature support.

For Optional, holds_at proves some row exists, not that the constrained value is present. Binding design must handle cleared/generated/converted values, filters, shared rows and eventual consistency without an absence-only pass. Existing Contains/Excludes might establish nonvacuity for identifiable arrangements; general sufficiency and compatibility are not established. Browser acceptance depends on the separately owned full execution product; declaration navigation or library WASM alone cannot prove it.

## Complete wrapped-position design candidate awaiting review

The existing read-only scoper produced a private complete wrapped-position design candidate, wrapped-value-invariant-design-candidate.md SHA256 c0956c68e84c40fa7b6797808dd08cef8d1aee4977b67514bfe5e408d3e8191e, reading this story revision 9 and runtime c2c4f01c6cfe99c6a16db5670669fb9774ab6bb9. Root rehashed it and read the proposed direction, scope and remaining decisions. No production source, AEP state, compiler or target execution was changed by that scoping work. The prior mixed-position root red remains unchanged.

The candidate proposes a closed typed ValueInvariants view expectation, preserving historical Satisfies semantics, with universal checks for every reached constrained value plus explicit actual nonvacuity. It covers Optional/List/Map/Union and recursive schema graph sites, source-preserving arrangement goals, per-position truthful refusal inventory and a cross-runtime independent healthy/fault matrix. This is materially wider than the earlier Optional-only inferred scope. It is a candidate for review, not an adopted binding design, new format reservation or source implementation authorization.

Root must still decide recursive site/back-edge identity, witness attribution where a view has no unique selector, exact format compatibility/allocation, shared byte/work budgets and DTO ordering, and browser bridge integration. No held version is assumed amendable merely from its number or release timing. Existing source refusals are not permanent feature exclusions; listing them without the required actual cross-runtime observations would not complete the user's all-feature requirement. Store scope will be expanded only after a coherent reviewed design is adopted, before production edits.
