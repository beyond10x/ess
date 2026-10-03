---
format: aep.planning-md/3
id: story:feature-request-361
kind: story
status: draft
title: Witness aggregate group-key parameter selection
refs:
- provider: github
  reference: beyond10x/ess#361
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Outcome

Synthesize truthful aggregate witnesses when a view parameter selects a group key, including group-key values copied from an addressed related row. Preserve both the grouping and filtering discriminators.

## Fit review

1. Need: one aggregate row for the requested group, without asking the consumer to filter unrelated groups after the query. Issue361 also reports a copied related field on that key. Brand-free Item/ByTeam source has team:String, count, group_by:[team], params team and filter team == param.team. Installed ESS0.51.0 validates it but synthesizes5 scenarios and1 precise ESS-SYNTH-017 refusal for the aggregate. Source SHA83d4aba8b3bd5933ffc4a6a308d512fd6555351045936c70a037ce70b7f66664; private probe361-group-param-closed. No new requester syntax is needed. Copied-key variant is still a required separate probe, not claimed covered by this one.
2. Class: conformance capability gap for already admitted source. Aggregate semantics partition and filter rows in docs/design/aggregate-views.md:152 and the view-level filter contract at:34. The current synthesis restriction is explicit in crates/verify/ess-conformance/src/synthesize/aggregate.rs:878–920; no source validation defect or new keyword is needed.
3. Existing expression: the exact source validates today. Removing the parameter and filtering in the reader changes which system behavior is checked; it does not witness ignored parameters. Existing copied-key arrangement machinery is in aggregate.rs:242 and:457; it must be reused rather than replaced with a second relation evaluator. The current measured source has no aggregate scenario, despite generator exit0.
4. Fit: reuse parameter/filter, group-key, exact related identity and existing aggregate evaluator authority. The parameter value and group key must describe the same row set. Include unrelated groups and a wrong-group/ignored-parameter target so a trivially single-group run cannot pass a broken reader. Keep creation/lifecycle/related source constraints and caller authority. All native/Go/TypeScript/WASM runners must execute the same admitted scenario steps; browser product support must be checked separately where it differs from the runtime. No silent exclusion of copied or Optional admitted cases.
5. Second adopter: a warehouse needs total reserved stock for one depot; a billing system needs total unpaid value for one customer. Both use a parameter selecting an aggregate group (same design semantics above), independent of the original consumer.
6. Cost: no ESS authored syntax change. Prefer existing scenario and typed view expectations; any additional persisted authority requires explicit coordinated format review in the held bundle. Synthesis witnesses, actual targets and compatibility tests change. Do not relax old shared-target assumptions implicitly; use scoped witnesses or the explicit new initial-state precondition as warranted.
7. Alternatives: change nothing and reader-filter (does not test parameter behavior); forbid a redundant parameter on a group key (unnecessarily forbids valid view selection); integrate parameter selection into existing group arrangement (chosen). Treat copied-key references through the established arrangement machinery, with an actual separate reproduction before choosing its exact implementation.

## Decisions

Accept, redesigned as an existing-source synthesis extension. No new source construct is adopted. The measured parameter case remains open; copied-key scope remains required and unverified. Do not mark implemented or close361 from this intake.

## Acceptance

Preserve the current validating source and absent aggregate witness as a red. Add actual source-admitted direct and copied-key fixtures. Healthy aggregate target passes; ignored parameter, wrong group, wrong related row and changed grouping each fail by the appropriate observation. Exercise empty/nonmatching query, at least two groups, lifecycle filtering, exact numeric aggregation and all currently admitted key types. Run the same suites in actual native/Go/TypeScript/WASM targets, preserve historical envelope admission and record refusal/answered counts.

## Scope

Cited core: crates/verify/ess-conformance/src/synthesize/aggregate.rs, existing aggregate view tests and related-guard/copied-key fixtures. Inferred new focused aggregate_group_parameters Rust test and shared actual-target controls. No production edit is authorized by this draft; record the final typed scope and red before activation. Synthesis is shared with the serial owner and requires sequencing after the frozen312 base.
