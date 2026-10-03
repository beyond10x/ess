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
revision: 5
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

## Copied-key composition probe, 2026-10-03

The previously missing copied-key reproduction is now measured with installed ESS0.51.0. Brand-free source has Depot.team:String, OpenDepot setting it from input, Item.team copied through the exact typed input.depot_id relation, and ByTeam grouping by team with count and sum(cents). Adding params team:String and filter team == param.team validates, but synthesis emits2scenarios and1ESS-SYNTH-017 refusal: the parameter reads a group key. The aggregate scenario is absent. Removing only params/filter validates and synthesizes3scenarios with0refusals, including the aggregate. Both commands exit0 for valid source/synthesis; synthesis refusal is measured separately, not hidden behind the successful exit.

This isolates parameter/group-key composition from copied related-row arrangement itself. It does not yet prove actual target execution or runtime parity. Direct-key prior probe and copied-key current probe both need healthy, ignored-parameter, wrong-group, wrong-related-row and wrong-grouping targets in the implementation unit.

Private ess-aggregate-intake-20261003 retains exact sources/logs/exit files and suites. Param sourcea5dfc886c5f85136416c3060ed7082f861722fbc892ba819d9ac3683bdf546d8; control source5eaa14e5e234058b348c6b33896e476847fa23b03cb217fb3a89178bf099d9be; param suited2b3a586e85dba699bdf970b4c053e5a9f51ac2d982ea537bec3d842f8171649; control suited0fa56905123610dd442d14a484778d1dd3227146fb230c6e0bbd5f16d49e6d5. No production, format, PR or gate change. Story remains draft until exact parameter/group arrangement design and scoped implementation are recorded; this completes the missing intake probe, not the defect.

## Combined aggregate design candidate

Root has inspected actual planner state and retained aggregate-group-selection-design.md SHA25693bac2e2a70b38933cbca97bc09603e0a54c4998bd9974f58961800ef8850e81 as a private candidate for361 and362 together. It is source analysis only, not implementation or new execution evidence.

The candidate separates group selection from non-group scoping, preserves tuples/related identity bindings, arranges actual reached rows, then evaluates the full source filter independently for each query. Typed Empty initial-state authority must reach the planner before exact unscoped observations; the current late final provenance selection is insufficient by itself. It asserts exact admitted group counts/aggregate values and observable distractors, with valid nonmatching selections only where the source domain permits them. State-only and Optional absent groups receive exact observations under that same authority, while historical admitted suites retain their contract.

Deleting the two refusals alone would overwrite group values and reuse the wrong query's admission flags. New tests must first assert the missing aggregate on the current source, then execute independent healthy/mutant targets in every runtime. Existing unrelated type/filter/identity pattern limitations and browser product execution remain explicit backlog work. Proposed production scope is aggregate.rs plus a narrow initial-state initialization in synthesize.rs; that shared file requires sequencing with the integrator's282/304 work. No source edits or production worker assignment yet; independent review is pending.

## Actual current-source aggregate regression red

Root compiled and ran the private Rust current-probe against unchanged runtime046db8a680, carrier6727e07363877925aa6a0f2bb1cb47b08d2348e3. Exact diff across crates/Cargo source is empty. All four brand-free inputs pass source assembly/compilation and emit fresh suite34 with scenario_initial_state:empty.

The copied-key control without a group parameter produces3scenarios,1aggregate,0refusals. Direct group-parameter source produces5scenarios,0aggregates,1ESS-SYNTH-017; copied related group-parameter source2scenarios,0aggregates,1ESS-SYNTH-017; state-only source5scenarios,0aggregates,1ESS-SYNTH-016. After validating the control, explicit assertions requiring the three missing aggregates fail with AGGREGATE_GROUP_SELECTION_GAP and terminal101. This is actual red on current code, not an installed-old-version observation or source-authoring failure. It does not execute target queries or claim an implementation.

Private retained current-probe Rust source SHA256a9cfb9c1206ead4a5358ea156b11c136ae50272a286a28e87fc2fb3bd611f86e; actualrunloga085425b0dc5ec35c36574d2a80152f008517b15f00ed78c0f3df38d7953dd4f; exitreceipt39b8dc3fc8b44765c8e6f1adee04c5b465e555ab791cc42d0d9e810d5b64297c; manifest03cf1c4a8db2f9c77a623d7ba484b60837d531b1f5e36c90ad168755c66ace70. Source YAML, lock, emitted suites and per-case JSON results retained under private ess-aggregate-intake-20261003/current-probe and its parent. One warm one-job compile finished0.85s. Root released the servers-cache lease after completion; no source edits or remote gate.

## Executed precondition composition check

Root executed the retained current-source Rust probe binary without compiling, using copied private fixture inputs and adding a declared Open precondition to the state-only aggregate source. Binary SHA256 fb597026674fde8b15c53453370832d7b967803fcc4bc75f93424e210bbcc6aa is retained in the owned servers cache. New source SHA256 66f6384d45d4b883eda2738aa6df644b40c6f2e80bd003a7d8bdbadce08095d1 declares demo.work.Open with team=seeded and cents=17. Source admission succeeds. All five emitted state-case scenarios begin with that exact ExecuteCommand and its expected opened outcome, verified by an independent jq assertion. Thus this composition is admitted source and retained setup, not a hypothetical syntax example. It creates an aggregate source row before the scenario's own arrangement.

The current aggregate remains absent with ESS-SYNTH-016. The unchanged full probe still ends at its named AGGREGATE_GROUP_SELECTION_GAP assertion, terminal 101; no target query was executed and no healthy-target failure is claimed. Original intake and prior current-source red evidence remain unchanged. New private evidence under ess-aggregate-intake-20261003/precondition-probe/run: run.log SHA256 cbcffc5efbf262158c8eb744d9f193f6e0d89a41712cfb317d94b352eb8d7e4e; state suite c3c358f21e049a7d4f03019b8b77b104ec90e8d273c5d54b382d3972552015dd; terminal receipt 39b8dc3fc8b44765c8e6f1adee04c5b465e555ab791cc42d0d9e810d5b64297c. This strengthens the existing independent design finding: Empty only precedes setup, and exact observations require the complete retained command prefix and its subsequent effects. A revision is being developed before any implementation authorization.
