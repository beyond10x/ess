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
revision: 7
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

## Executed precondition composition check

Root executed the retained current-source Rust probe binary without compiling, using copied private fixture inputs and adding a declared Open precondition to the state-only aggregate source. Binary SHA256 fb597026674fde8b15c53453370832d7b967803fcc4bc75f93424e210bbcc6aa is retained in the owned servers cache. New source SHA256 66f6384d45d4b883eda2738aa6df644b40c6f2e80bd003a7d8bdbadce08095d1 declares demo.work.Open with team=seeded and cents=17. Source admission succeeds. All five emitted state-case scenarios begin with that exact ExecuteCommand and its expected opened outcome, verified by an independent jq assertion. Thus this composition is admitted source and retained setup, not a hypothetical syntax example. It creates an aggregate source row before the scenario's own arrangement.

The current aggregate remains absent with ESS-SYNTH-016. The unchanged full probe still ends at its named AGGREGATE_GROUP_SELECTION_GAP assertion, terminal 101; no target query was executed and no healthy-target failure is claimed. Original intake and prior current-source red evidence remain unchanged. New private evidence under ess-aggregate-intake-20261003/precondition-probe/run: run.log SHA256 cbcffc5efbf262158c8eb744d9f193f6e0d89a41712cfb317d94b352eb8d7e4e; state suite c3c358f21e049a7d4f03019b8b77b104ec90e8d273c5d54b382d3972552015dd; terminal receipt 39b8dc3fc8b44765c8e6f1adee04c5b465e555ab791cc42d0d9e810d5b64297c. This strengthens the existing independent design finding: Empty only precedes setup, and exact observations require the complete retained command prefix and its subsequent effects. A revision is being developed before any implementation authorization.

## Complete query-time inventory direction

Read-only source proposal aggregate-group-selection-revision-proposal.md, SHA256 9f3d2556b354af6f8c37e02fcff7a16d62f33f88ff088c394d68c277f84b3bd9, confirms that Arrangement describes one row and cannot supply a complete inventory for instances, affects or binding-invoked effects. The original two-file design is insufficient. Root accepts the direction of preparing aggregate actions/query intents, selecting the exact existing precondition prefix once, installing fixture/time authority, then evaluating each query cut over the complete store. Reuse the accepted #292 shared executor/value semantics through a scenario-specific adapter; history candidate inputs and expected outcomes do not become conformance authority. No overlapping source edit is authorized before agreeing the shared interface and integrator's synthesis waves.

Dynamic fixture/observed-dependent grouping or totals remain part of this same completion bundle. Existing ScenarioValue, static Counts and literal ChangedBy cannot express all such results. A concrete closed aggregate-result observation design is being drafted, with actual occurrence capture, causal query cuts, exact arithmetic and every runtime required. No format is adopted, no version pair is reserved, and no new implementation has been dispatched by this direction record. Static-only support must not be reported as completion of the general feature; a blanket precondition refusal would not fix the reviewed defect. Generated stored values with no independent source-observable authority remain an information boundary requiring a truthful named gap rather than fabricated expected values.

Retained source-specific proposal includes prefix truncation, known/generated identity distinction, cross-row old-store effects, fixture key merging, same-event multiple occurrence captures, source binding completion and legacy compatibility controls. The proposal itself executed zero builds/tests/probes. Root's earlier admitted precondition probe is separate actual evidence. Draft dynamic authority must resolve exact scope/resource/compatibility questions and undergo independent review before production edits; the original design review finding is not yet marked fixed.

## Dynamic aggregate wire proposal and review

A concrete private aggregate-result observation candidate now exists, SHA256 64d60937c33fe7f0af2cb56a40a8d8c19b3b7fefb823b9019d547a748f472c8d. It proposes closed prepare_aggregate/aggregate_result vocabulary, exact finalized step/query-cut references, normalized semantic contracts, write-once invocation captures, all six aggregate functions, bounded exact arithmetic, full native/Go/TypeScript/WASM parity and actual historical-reader rejection. This is a design proposal, not an implemented capability or reserved format.

Root source review needs revision, immutable report SHA256 69a9e7463fbc7b95dc82347ce79663fde8efa2621db45ea903b64deb4b4ea9fd, review-result:consumer-aggregate-wire-pass1. Two concrete boundaries remain: the normalized Contract has no defined admitted reconstruction path into compiler-owned EssIr consumed by the shared executor; and binding delivery_cut requires a concrete source/target completion capability and scenario trigger absent from the current ObservedInvocation API. No unchecked IR constructor, alternate hand-written native interpreter, blanket binding refusal or fabricated quiescence is authorized. Keep dynamic fixture/observation support in the same completion bundle; static-only success is not completion.

One-time disclosure, runtime error/capability classifications, exact resource counters and occurrence provenance are explicit acceptance obligations. Transport changes require the separately assigned transport owner; #292's private executor implementation is not implicitly expanded. Browser product design is being made concrete independently. The original aggregate baseline review remains unresolved until a complete revision is independently assessed. This pass executed no builds or target probes.

## Reviewed aggregate adapter direction

The revised normalized aggregate observation design, aggregate-result-observation-wire-candidate-v2.md SHA256 a32c5a65d92d132b6356aab3c3a0b2db553f507a9ef066e761d8cbe2fabc64cf, has passed root source review with no new findings (review-result:consumer-aggregate-wire-pass2; private review SHA256 1ef6914afaa9d24bb5827e35bf1da165c972e1a9049ce7e45beb2fe1b182b20b). Both pass1 design findings are addressed: native Contract admission constructs checked RawSpecFile values, assembles and compiles through existing authorities, then checks semantic re-projection; delivery_cut/frontier are removed from admitted grammar and binding-sensitive completion remains an explicit unresolved same-bundle dependency.

This approves a concrete design direction, not an implemented adapter or complete aggregate support. Required next work is to bind a design page and exact scopes, source-to-Contract completeness and foreign validator inventories, and the shared #292 execution interface. First actual evidence must cover reconstruction round-trip and one-change invalid mutations before production expectations use the adapter. Then prove complete-prefix, direct/related/set-effect, dynamic grouping and all six aggregate functions on actual native/Go/TypeScript/WASM runtimes. A binding-affected query still needs independently verifiable causal-cut authority coordinated with the assigned transport owner; unconnected bindings do not justify refusing other queries. No static-only or missing-binding partial result closes this story.

No compiler/domain helper, transport change, format reservation or implementation worker is authorized by this review. No builds or target probes were executed for the review. The actual admitted aggregate and precondition reds remain retained unchanged, and the original incomplete-inventory finding is only fully closed by later implementation evidence.
