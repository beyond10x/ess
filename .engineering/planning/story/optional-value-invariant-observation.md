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
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
revision: 2
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
