---
format: aep.planning-md/3
id: story:feature-request-312
kind: story
status: active
title: Suites state their empty-target assumption and act across callers
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#312
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
scope:
- confidence: cited
  path: crates/edge/ess-cli/src/main.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/conform_report.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/coverage_browser.rs
- confidence: cited
  path: crates/verify/ess-conformance/assets/coverage-admission.js
- confidence: cited
  path: crates/verify/ess-conformance/assets/coverage-player.js
- confidence: cited
  path: crates/verify/ess-conformance/assets/index.html
- confidence: cited
  path: crates/verify/ess-conformance/assets/player.js
- confidence: cited
  path: crates/verify/ess-conformance/src/admission.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/count_json.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/evidence.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/go
- confidence: cited
  path: crates/verify/ess-conformance/src/report.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/runner.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/scenario.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/existence.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/related_guard.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/set_effects.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/singleton.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/ts
- confidence: cited
  path: crates/verify/ess-conformance/src/web.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests
- confidence: cited
  path: crates/verify/ess-conformance/tests/support_versions/mod.rs
- confidence: cited
  path: crates/verify/ess-diff/src/impact.rs
- confidence: inferred
  path: docs/design/scenario-initial-state-and-cross-caller-witnesses.md
revision: 36
transitions:
- {from: "draft", to: "proposed", at: "2026-10-02T21:59:23Z", actor: "human:timo", revision: 16, executor: "agent:codex-ess-backlog", correlation: "consumer-runtime-20261002"}
- {from: "proposed", to: "active", at: "2026-10-02T21:59:24Z", actor: "human:timo", revision: 17, executor: "agent:codex-ess-backlog", correlation: "consumer-runtime-20261002"}
---
## Outcome

Suites either hold on a shared target or say they need an empty one, and some scenario acts on a row as a different caller than arranged it.

## Origin

beyond10x/ess#312, found in the #287 adversary pass; pre-existing on 0.49.0.

## Fit review

1. Need independent of syntax. A consumer must know whether a generated suite assumes an empty modeled context, and the suite must distinguish a globally shared entity from an accidental per-principal copy. Locally retained GitHub312 body (open-issues.json, issue312, created2026-10-01) supplies two controls: caller-supplied UUID reused by distinct scenarios causes already-installed on a persistent target; two callers each get a private row and incorrectly pass. Requester's proposal is behavioral, not syntax: either use fresh identities on shared targets or declare empty-target requirement in provenance/output; arrange as one actor and act as another. Brand-free reproduction is the existing demo.plant Switch/Job model in tests/adversary_287_pass1.rs25–135: InstallSwitch(switch_id), PauseSwitch(switch_id), then StartJob referring to that exact switch; two Operator callers have different principal_id values. Uuid identity reproduces the general gap, singleton Main the finite-identity boundary. No adopter document needs copying.

2. Classification. Defect in generated-suite claims plus an evidence gap, not a missing domain-language concept. The accepted isolation design §8 at docs/design/ess-closed-loop-execution-conformance-design-v0.1.md383–407 permits tenant namespaces, generated IDs or other mechanisms; it does not mandate globally empty storage. scenario.rs2479–2480 and2551 explicitly permit shared targets. Reusing a supplied identity without declaring stronger initial-state requirements conflicts with that contract. Actor and caller values already exist (scenario.rs2020–2045), but existing synthesis does not always exercise a different caller on the same row. A provenance extension is justified; a new source keyword is not.

3. Already expressible / already satisfied. Target begin_scenario/end_scenario and correlation isolate observations today (target.rs13,174; runner.rs431–465). Generated Go/TS docs already tell adapters to construct one target per scenario (go/mod.rs290; ts/mod.rs491); that is an implementation recipe, not machine-readable empty-state provenance. SuiteProvenance369–403 has no such requirement. Attributed callers already have mixed-caller refusal witnesses and role-swapped runs (synthesize/caller.rs18–27,161–223; tests/caller_values.rs235,517,534). Singleton exhaustion explicitly switches the command under test to caller2 on caller1's row (caller.rs612–668); adversary287's singleton per-caller test512 is ACTIVE, so it would be false to claim this entire half is absent. The Uuid per-caller test528, shared Uuid test543 and singleton empty-target disclosure test638 remain explicitly ignored for312. Caller synthesis's mixed assignment only fills IDs missing from the all-first suite at205–209; the ordinary swapped copy normally creates a different row. Existing attributed capability therefore does not guarantee same-row cross-caller coverage for already-reachable branches. Attribute-free actor grant rotation (grant.rs197–277) sends each command as its granted actors across scenarios and may rewrite all occurrences of that command; it does not promise the requested arranging-caller/acting-caller distinction. Authored Act.actor exists at authored.rs413/2652, so a user can manually spell a two-actor timeline today. No new authored actor spelling is warranted. Per this assignment, these idioms were inspected rather than rerun; do not record a fabricated validation result.

4. Fit/composition. Reuse actor, caller, instance captures, existing typed value facts and scenario lifecycle. Do not add a caller_id magic input or treat equal permission sets as the same actor. Distinct same-role callers use distinct declared attribute values; distinct declared actors without attributes can use their actor names. A model with one attribute-free actor has no declared distinct principal to invent. Shared-row assertions must use the exact captured/supplied identity, preserve subject/related guards, input refusal precedence, bindings/events, view snapshots and caller-aware authorization. Re-synthesize under the mixed assignment when caller facts affect branch selection; blindly flipping actor bytes can turn a valid expected outcome into an invalid one. Related-row dependencies count as reads just like direct subjects. Preserve global aggregate/position/count composition restrictions: caller.rs574–579 and685 onward already identify runs that cannot be appended honestly. Retained replay, fixture preludes and periodic checks must not be silently dropped to make a second run fit. Existing singleton reuse is retained rather than attempting a second impossible singleton creation.

Every conformance execution path must read and preserve any new provenance contract and execute existing mixed actor/caller steps: native Runner, generated Go, generated TS, WASM execution and interpreted targets. Browser catalog/replay must visibly retain the precondition and cannot claim real backend reset. Runners may refuse an unmet adapter requirement by name; this is not permission for a language-wide feature opt-out. Structural projections/source diff/Entity Runtime source compilation need no invented source construct, because the product model does not change. Native interpreter caller/related execution is separately owned work; this story must consume that seam rather than implement a second caller evaluator.

5. Unrelated adopter. A shared inventory reservation created by WarehouseA must be visible when WarehouseB releases it; a store partitioned by credential would incorrectly report missing. A shared document lock installed by one editor must prevent another editor from acquiring the same lock. Both use existing entity identity/grants and expose exactly the per-caller-store defect. A reset or a newly generated independent row for callerB would erase the evidence. These follow the same entity/subject and related-reference semantics as the Switch/Job fixture, not a deployment-specific local policy.

6. Cost and version consequences. No ESS source-language or authored-scenario bump: no new model fact, actor syntax or caller step is needed. Explicit required suite provenance is a wire extension: Go runtime.go3981 and TS runtime.ts5834 close the provenance key set, so it cannot be silently added under an already frozen major. Reserve one coherent ordinary/coverage suite pair after the currently implemented34/35, tentatively36/37 only if coordinator confirms no competing allocation. Do not repurpose34/35 or source21, and do not borrow a future source22 bump merely because it is nearby. Old suite bytes retain legacy unspecified initial-state semantics, not a retroactive claim of shared safety or emptiness. New metadata must survive selection, parent/child exact-byte coverage transport, model digest binding and generated adapters. Adding a field to public Rust SuiteProvenance breaks external struct literals; serde default alone does not make that source-compatible. Audit constructors and generated Go public structs similarly. If new scenario IDs are needed for non-composable mixed witnesses, their closed grammar must be part of the same allocation; avoid new IDs when an existing family's observation can be safely extended.

The structured run report already carries SuiteProvenance (report.rs552), so it can retain the precondition without a second authority. Human CLI and generated-runner output must print it before execution. CountReport/2 is a closed WireReport at counts.rs176–193; do not add a field to it silently or misuse producer_profile/2 as a schema bump. Minimal proposal keeps report/2 unchanged and prints the requirement on the diagnostic/text channel while preserving it in the exact paired suite. A standalone count document wanting an explicit copy would require a separately versioned report design, outside this minimal request.

7. Alternatives. Change nothing / tell users to reset: rejected; hooks and README prose exist but the frozen adversary cases show the suite does not state its stronger precondition and does not test every relevant cross-caller dependency. Make all supplied identities globally fresh: insufficient; finite/singleton identities cannot be renamed, guards may pin values and authored setup fixes identity, and random per-scenario values would weaken deterministic bytes. Add a model-level global/shared-entity or new cross_caller keyword: rejected; entities are already shared unless caller-dependent semantics say otherwise, and existing actor/caller syntax is sufficient. Chosen: declare the initial-state requirement in suite provenance and broaden reuse of the existing mixed-assignment planner, with concrete mutants proving the behavior. This changes the requester's either/or into an explicit deterministic precondition and avoids promising arbitrary shared-target safety.

## Decisions

Accept, redesigned. The consumer requires an honest initial-state precondition and same-row cross-caller evidence. Reuse existing actor/caller attributes, capture identities, scenario lifecycle and typed source guards; do not add a source keyword, magic caller input or invented permission.

Coordinator adoption supersedes the fit proposal's tentative additional suite36/37 allocation: current source21/suite34–35 one-time disclosure changes are still local and unreleased, with no component PR or release tag. The repository assessment rule says not to add a version where a nearby unreleased bump can carry the change. Fold the new provenance contract into that same unreleased ordinary/coverage34–35 pair and its compatibility controls. Legacy suite1–33 bytes retain their existing unspecified initial-state semantics. Source/authored formats and CountReport/2 remain unchanged. Verify final allocations against source before implementation.

Use typed provenance scenario_initial_state: empty for newly synthesized suites. It requires an empty logical modeled-instance/event/invocation namespace before each scenario's authored setup and command steps, not deletion of physical shared storage. External provisioned resources remain explicit obligations; declared upstream setup populates this namespace after begin. Returning successfully from begin_scenario accepts this lifecycle precondition but is not proof that a physical reset occurred. Print the precondition in human-facing run diagnostics and retain it in full provenance/coverage transport; preserve report/2's closed fields. Old unspecified suites must not acquire a retroactive safety claim.

Conservatively declare this requirement on every newly synthesized suite rather than pretending to prove arbitrary persistent-target safety. For each direct/related family where two distinct eligible callers or actors are expressible and composition is valid, arrange and act on the same row under distinct callers, preserving source-selected authorization/refusal semantics. Require one deterministic pair and swapped-role discrimination where expressible, not arbitrary combinatorial caller permutations. Attribute-free actors with equal grants remain distinct. A single attribute-free actor cannot supply an invented second principal.

The actual ignored UUID partition-by-caller mutant is the deciding regression. Keep existing singleton mixed-caller controls and owner/refusal witnesses. Never reset or recreate the row between the arranging and acting caller. Preserve aggregate/order/count, fixture, binding and replay composition restrictions; if a family cannot compose, record its specific coverage reason without reporting a second independently created row as shared-row proof. Native interpreter related/caller execution is an existing dependency owned by its current workers.

This decision is within the operator's all-actionable-backlog authorization. It authorizes design and implementation after precise scope/acceptance are recorded, with real native/Go/TypeScript/WASM parity, independent review and one grouped final gate. It is not a claim that the draft implementation or tests have run.

## Acceptance

A312-1 Fresh logical context declared: synthesized supplied-Uuid upsert suite and singleton suite carry the explicit requirement; native/Go/TS text output and full suite provenance display the same requirement. Selected coverage child keeps it and exact parent binding.
A312-2 Shared physical target, fresh namespace: correct target retaining unrelated sentinel data globally but isolating each scenario namespace passes; the sentinel survives. A target unable to open the declared context returns Unsupported before command callbacks. No test depends on deleting global state.
A312-3 Legacy admission preserved: existing suite1–33 bytes retain semantics; old reader refuses the new suite major before identity/begin/command callbacks, and malformed/missing mandatory new requirement is rejected. Report/2 bytes remain unchanged apart from ordinary suite reference/digest/count effects.
A312-4 UUID same-row mutation caught: activate/strengthen adversary_287_pass1::adv287_control_a_uuid_switch_suite_fails_a_target_keeping_one_switch_per_caller. Healthy global store passes; KeysByCaller fails a specific switched-caller scenario. Assert trace: arranging caller differs from acting caller, exact same switch identity, no second installation between them. This is the principal deciding regression.
A312-5 Singleton and swapped-role evidence preserved: existing active singleton per-caller test remains green; no duplicate singleton creation is added. Hardcoded-first-caller mutant fails when roles swap or a separate legitimate counterpart supplies that evidence. Resolve existing ignored empty-target test638 to exact provenance/output assertions, not a generic note-name search.
A312-6 Actor without attributes: two declared actors with identical grants, no attributes, creator under actorA and direct subject operation under actorB; healthy shared row passes and actor-partitioned row mutant fails. No mutation of undeclared caller attributes.
A312-7 Related/caller-guard composition: actorA creates and pauses the switch, actorB invokes StartJob referencing it; expected source refusal and unchanged row/events are checked. Existing caller-value owner/refusal scenarios235/517/534 retain their semantics and decisive mutants. A caller-owned model must get its declared refusal, not an invented global-access success.
A312-8 Real runtime parity: same serialized suites, actual target callbacks and the same healthy/partitioned/hardcoded callers across native, generated Go, generated TS and supported WASM path; compare five-category counts and diagnostic codes without normalization. Interpreted is included once its separately owned caller/related prerequisites are integrated. Browser replay is labeled as replay and preserves metadata; no claim it executed backend isolation.

## Scope

Implementation owner scope_aggregate in a new managed tree from carrier4bc5660e4, with sole ownership of the new design and synthesis/provenance/reader changes. Root remains AEP and integration owner. Existing native related/caller/effects/absent workers own interpret.rs and interpret/**; this story consumes their completed execution and must not edit those paths.

Cited by source assessment62f0b66bf64367b708969c923be116556fe7b683c99dd6f7ebb29a1847c5d8b2: scenario.rs SuiteProvenance369–403 and suite format registry; synthesize/caller.rs18–27,161–223,574–685 and grant.rs197–277; Go runtime closed provenance3981 and TypeScript runtime5834; runner.rs431–465 and report.rs552; tests/adversary_287_pass1.rs512–638; existing caller_values and coverage transport tests. New docs/design/scenario-initial-state-and-cross-caller-witnesses.md is inferred as the binding design home and must be written before production code.

Exact CLI and browser display entry points are to be established before edits. Add their precise paths to the handoff and report discoveries to root for machine scope. No CountReport/2 field, source syntax or source diff projection changes. Reuse unreleased suite34–35 by the recorded decision. Required fixture/old-reader/actual-target mutation and all supported language execution tests travel with the implementation. Target factories/hooks own logical isolation and must preserve unrelated physical data. Builds use only this worker's idle synthesis cache, jobs2/debug0/incremental0 and external TMPDIR; pause below8GiB. Independent review and final integration evidence are required; no component PR or remote gate.

## Reviewed core integrated, remaining caller-sensitive synthesis

Owner bot commit c3556bf49ed0a5372536c210c3e24a5d35833a7d is integrated as 13e40c33f9. Frozen reviewed source daf8d1c7f42c8ddd455bc9b7944000c66dde6e99900e23c1ef66c27d704d640f; independent pass2 and pass3 approvals retained as consumer-initial-state-pass2 and consumer-initial-state-pass3. Owner measured34 focused native/Go/TS/WASM tests,2 CLI/Firefox tests,2 strict TypeScript packaging checks, final live Go/TS parity1/0; scoped lint/fmt exit0. Go parity used installed Go1.27.0-X:nodwarf5, not the pinned gate compiler. Report retained in owner target/backlog-input/312-implementation-report.md SHA55110c33037cf257a5af0c444f326228cce26f84c6591aa4b326113c65e434d2.

First review caused actual source correction for attribute-free and attributed caller-insensitive same-command arrangements. Caller-sensitive upsert remains unfinished, so no claim that the full finding or story is resolved. Root owns canonical producer regeneration, format expectation migration and integrated checks. Full story stays active through remaining acceptance.

Owner read-only continuation brief target/backlog-input/312-hybrid-continuation-brief.md SHA45c153c7cc6f7ed127a3ba03dc263dc25f606463f6f56384ad2d805de32bc85b identifies a narrow caller-valued hybrid when all rewritten First/Second conditions compare equal. General caller-sensitive guards require separate arrangement and acting command interpretations per invocation: evaluate First creation eligibility with First input/absent-or-related facts, then Second acting eligibility against the actual typed First-created row. Changing actor bytes after selecting an outcome is not acceptable. Root requests concrete full-remaining design/scoping before authorizing further production edits; this is continuation of existing story acceptance, not a reduced acceptance set.

# #312 continuation: caller-sensitive same-command arrangements

Read-only design, 2026-10-03. Based on committed c3556bf49 (integrated by root as 13e40c33f9). No production edits or new runtime results are claimed. The earlier hybrid brief remains retained at initial-state tree target/backlog-input/312-hybrid-continuation-brief.md; this general design supersedes its guard-equality restriction.

## Decision

Use two complete typed interpretations of the same source model, one for arranging invocations and one for acting invocations. Do not splice First creation writes into a Second command. The full First command, including all competing guards, inputs, related-row requirements and writes, must select each arrangement outcome. The full Second command must select the acting outcome against the row actually left by First. Both interpretations retain identical command names and model structure; caller rewriting changes only typed value sources and predicate operands.

The current planner already separates most of these concerns. Its input `ir` supplies arrangement drivers; its separate `command` and `outcome` arguments supply the branch under test. This permits reuse of its guarded searches rather than introducing an interpreter or guessing new assertions.

## Concrete seam and typed scope

Add a private invocation-planning context near `synthesize::caller` with explicit arrangement and acting model references, plus a typed role/credential assignment. Illustrative names: `InvocationModels { arrangement, acting }` and `InvocationPhase::{Arrange, Act}`. An ordinary synthesis delegates through a context with both references equal. A mixed run uses First's complete rewritten IR for arrangement and Second's complete rewritten IR for acting. No serialized scenario member, format bump, runtime capability or interpreter change is required.

Separate scenario-family enumeration from arrangement lookup: the focused family enumerates the acting model's commands/outcomes while its searches receive the arrangement model and the explicit acting command/outcome. Keep original source provenance and unchanged type/entity/view definitions. Make this distinction visible in the helper/API; do not silently pass differently interpreted references into a parameter still described as one model.

1. `src/synthesize/caller.rs`: reuse `Callers`, `written`, `mark`, `append_independent`, and identity drawing. Replace the caller-sensitive blanket note in `cross_single_command` with focused per-invocation synthesis. Independently plan First->Second and Second->First. Do not derive the reverse run by changing credentials on an existing run. Enumerate all source acting outcomes, including outcomes absent from the all-First suite, and retain each under the outcome actually selected. Never relabel a source refusal as a successful update.
2. `src/synthesize.rs`: expose the model split at `synthesize_plain`/focused family dispatch, `outcome_scenario`, `exercise_as`, `run_as`, `exercise_run`, lifecycle/state-refusal/invariant entry points. The structural `Run { setup, invoke, before_settled, settled }` is the main role boundary. `prepare_subject`, `arrange_first`, `arrange`, `advance`, `created`, `created_owned`, and `invoke` continue searching First's drivers. `reach`, `selected_in_state`, `arranged_as`, and assertion construction receive Second's explicit command/outcome. `settled` computes Second's writes from First's actual settled facts.
3. `src/synthesize/subject_fact.rs`: preserve the existing split. `search_under` obtains creators/drivers from `ir.drivers()`; `arranged_row`/`prepare` close over the explicit acting command; `reach_linked`/`selects` evaluate that command against each actual First arrangement. `creations` already validates alternate candidate inputs with `input_selects` on the creator command before accepting them. Keep that check on First's creator. `boundaries`, `send_for_row`, refusal witnesses and absent-row helpers emit additional acting invocations and must carry the explicit Act role when they assemble those steps.
4. `src/synthesize/related_guard.rs`: likewise `prepare`/`prepare_at`/`arranged_at` search related and subject rows through First IR, while `selects` examines the explicit Second command and its input predicates. Creator `drive`/`drive_on` must continue using First's full Driver, including First's competing input/related predicates. The helper near `search_rows` that emits direct command steps needs an explicit role for whichever phase called it.
5. `src/synthesize/existence.rs`: this is the genuine exception requiring an API correction, not merely passing a different IR. `segment` currently takes one command for both its `creating_input`/Driver and its later refused invocation. Split it into a First creator reference and a Second acting reference. `creations`, `creating_input`, `identity_at`, `created`, and any reconstruction of the original creating invocation use First. Select the refusal/input against Second, then bind the exact same identity. `existing_instance`, `refusals_on_a_stored_row`, `arranged_refusal`, and `held_state_refusals` must preserve this distinction. `recreation`/`recreated` contains a later replay of the creation: that invocation is First again, not automatically Second because it is third in the vector.

Credentials must be attached while these boundaries are still known. Mark arrangement blocks as Arrange and acting blocks as Act before flattening `Run::steps`; also do so for the direct emitters in subject_fact, related_guard and existence. Additional view-neighbor setup belongs to Arrange; an actual repeated command whose post-state behavior is under test belongs to Act; recreation belongs to its explicit creator role. A small typed helper can stamp the existing `actor`/`caller` fields, so no private phase markers need to escape into the public suite. Do not recover phase from command name or ordinal after flattening. Thread the context only to family/block composition and direct invocation emitters; the bounded row-search engine can continue taking the arrangement IR without a new generic planner abstraction.

Audit supplementary paths `view_expectations`, `deletion_witness`, `run_state_refusal`, `run_replay`, `subject_fact::boundaries` and related boundaries before declaring this general: their extra calls must have the interpretation that generated their inputs/assertions. This is an implementation checklist, not permission to suppress those features on mixed scenarios.

## Why guarded upserts are sound under this split

Example: add `bad-principal` input refusal `input.label != caller.principal` before PutItem's existing updated/unknown-instance-created pair; store `{caller: principal}` in label. First creates with label equal to First.principal. Second acts on that exact item_id with label equal to Second.principal. First creation cannot borrow Second's truth: its `reach`/creation candidate check sees First's full command. Second's selection cannot borrow First's truth: its explicit command is rewritten with Second's principal. The captured identity links the two invocations while the settled label initially remains First.principal. The resulting payload/state expectation is Second.principal only after Second's successful write.

A stored-owner variant may instead select a source refusal for Second (`when_subject: ... != caller.principal`). The planner must retain the First row and select that refusal, then assert no events and preservation of First's row. If Second can reach the update only after changing an admitted input, the existing candidate search may choose that input. It must never change a previously stored First fact merely to make the desired update reachable.

For related guards, First's creator arranges and validates its own related prerequisites. The Second acting search sees the actual arranged related facts, including owner bindings. Unknown facts stay unknown and produce the existing bounded-search explanation. A guarded source form must compile first; do not claim unsupported combinations of condition syntax as admitted test cases.

Witness cache safety is already present: `src/witness_memo.rs::held` requires pointer identity with the command stored in that IR. A Second explicit command searched against First IR cannot hit First's memoized answer. Preserve this predicate and add a regression which would fail if First and Second input guard witnesses were confused. Optimizing the Second cache is unnecessary for the first implementation.

## Red-capable fixtures and actual execution matrix

Keep the measured core tests/evidence unchanged as history. Add focused Rust regression tests (or extend `tests/adversary_287_pass1.rs` and the existing paired fixture harness) which first compile the admitted source and then fail on the current blanket same-command gap. New executable test code is Rust; generated Go/TS fixture text follows the existing repository template convention.

Required source cases:

- Minimal caller-valued PutItem from the existing `caller_valued_upsert_records_the_remaining_per_invocation_witness_gap`: same item_id, First value at creation, Second value at update, independently reversed roles.
- Input-guarded PutItem as above: require different First/Second principal-matching input labels, including a competing input refusal before existence. Assert every invocation's supplied label agrees with its own credential; run against a target that evaluates the actual source guard.
- Stored-field caller-sensitive upsert: create under First ownership; source-selected Second refusal, row unchanged; and an admitted branch/input allowing a successful cross-caller update, where source permits it. These are separate source behaviors, not a requirement to force update through an ownership refusal.
- Related-row guarded upsert: First creator prerequisites plus Second acting selection, with a negative related predicate/input boundary. Reuse an admitted existing related-guard fixture and add the caller-valued source/input guard; compile acceptance before claiming coverage.
- Create-or-refuse BookSlot: First's guarded creation, Second's existing_instance refusal, input-refusal precedence on a stored row, and recreation using the correct creator credentials.
- Optional/default input and a further stored/related boundary: prove supplementary runs retain phase assignments rather than only the main two calls. Existing attribute-free/caller-insensitive same-command cases remain green.

For the same serialized synthesized suites, run native Rust Runner, generated Go runner, generated TypeScript runner and actual WASM/browser bridge. Healthy target must pass every selected scenario with zero skipped/refused runtime outcomes. Fault modes should include: partitions rows by caller (Second incorrectly creates/loses the row); permits only one hard-coded caller; writes a hard-coded principal; ignores the source stored-owner refusal; evaluates a creator's guard using the later caller; and cannot honor the empty logical namespace. Compare exact outcome/status counts and diagnostic code sets symmetrically, as the core paired test now does. Inspect exact identity equality, source outcome references, credential payloads and First/Second stored values, not only log membership.

The existing actual CLI/Firefox initial-state-display checks need rerunning only if their output/fixture changes. Metadata format stays34/35. Required release Go toolchain remains a gate obligation; local Go1.27 results must not be presented as Go1.25.10 validation.

## Restrictions: semantic facts versus implementation gaps

- No two distinguishable eligible credentials under the declared types/grants: a semantic absence of the requested pair, with precise actor/type evidence. Attribute presence alone is never this reason.
- Source ownership guards intentionally refuse the second caller: an executable mixed refusal witness, not missing coverage or permission to fabricate success.
- Singleton identity or exhausted finite identities: can forbid appending a second independent arrangement in the same scenario. It does not forbid the first mixed shared-row witness. Keep the existing distinct unswapped explanation when reversal cannot compose.
- Aggregate/count/rank assertions, retained replay, fixture prelude and periodic checks currently block `append` because the second run cannot safely reuse absolute observations/state. This is an existing composition limitation, not proof that mixed invocation synthesis is semantically impossible. Preserve a correct mixed first run and precise unswapped note; do not call these unsupported caller semantics.
- Search-budget exhaustion, unknown stored values, cyclic/unavailable arrangement routes and missing phase plumbing are implementation/witness limitations. Name the actual cause and bounds; do not assert mathematical unreachability. Full #312 cannot be closed merely by replacing the blanket caller-sensitive note with another implementation-gap note for these now-scoped admitted cases.
- Browser step admission and current source-format admission remain closed. This change adds no blind browser step enablement and no interpreter edits.

## Completion boundary for this continuation

Root records this design and authorizes production work before edits. Implement and obtain actual red/green for the admitted cases above, preserve previous evidence, freeze source for independent review, run focused strict lint and actual paired runtimes, and bot-commit. Parent remains sole AEP/integration/release writer. The core commit is valid and integrated; #312 remains nonterminal until this scoped caller-sensitive gap and its review findings are resolved.

## Coordinator acceptance

Accepted continuation of existing active story under the full-backlog mandate. Owner recover_312 resumes its managed initial-state tree from c3556bf49; root is migrating fixtures in the integration tree, so owner must not edit canonical fixtures or unrelated format-only tests. Implementation scope is the five synthesis modules above, existing adversary_287_pass1 and support_initial_state runtime tests, and the binding initial-state design page. Root records any additional cited helper scope before source expansion. Phase distinction must remain explicit through auxiliary invocations and must not be recovered from command name/ordinal. Retain a real compiler-admitted red for every source family before production edits, then the actual four-runtime matrix and independent review. Existing semantic composition restrictions remain precise notes, not an excuse to omit the requested mixed first witness. No new PR, remote gate or release in this unit; root owns AEP, integration and publication. All committed executable source Rust; existing generated-runtime templates remain in their established languages. Existing bounded build/cache lease and 8GiB floor rules apply.

## Admitted same-command set-effect witness

The continuation established a compiled source-level counterexample, rather than extending scope for rejected syntax. In set-effects.yaml, Open accepts bulk:Boolean. opened when bulk==false creates Session and stores caller.principal; noted when bulk==true updates the filtered team set from input.note. Current synthesis gives the creating Open and the bulk-action Open the same principal-262144. Exact paired witness caller_sensitive_set_command_arranges_under_another_invocation fails. Owner log312-general-target3.log SHA d709d07f3752f280fa92c58a97d6e58d2a2e041b16e69869f52328db3150ca84 preserves the admitted failing case.

Extend the existing First-arrangement/Second-action design to synthesize/set_effects.rs. Scope is its context/enumeration, direct ExecuteCommand seam and setup blocks: reuse typed phase authority and the First-owned creator requirements, never infer role from command name or step ordinal. Preserve existing set-effect guard selection, source admission, filter and write semantics. Test the actual creation/action pair, both directions, and retain existing healthy/fault runtime witnesses. This is continuation of story312, not a new syntax feature. Caller-valued bulk writes and when_related+unknown_instance combinations rejected by the compiler remain excluded; record the exact semantic refusals, do not invent support for them.

## Continued migration and cross-runtime evidence

Expanded migration patchf7cf54eef26b27f6e00c74b62323cd9f0ceec3660604d61c3a05b8c4da764ce8 independently approved by recover_caller, findings empty, reviewer executions0; report44343b9342f17e48c7d549a20c2399051bcd5046f76df34cb9437e16a7283e9e. Own accessor correction is separately independently approved by root, not self-reviewed. Immutable review artifacts retain both initial finding and correction. Canonical suites and README are included; TS parity correction is separately frozen80341f5e5269c087cd8c4dd817ce2b16f3c9a409a14f5c5a77030a380195ca8e for final review.

Observed integration logs under ess-backlog-next-20261002/target/backlog-input: group3 baseline18passed15failed, corrected40passed0failed across9 binaries including21RelatedField tests; group4 completed33passed0failed across4 binaries. count_reports11/0 and Go runtime parity23/0 passed. TypeScript initial4/24 was stale version, legacy metadata and missing installed Node types; first correction27/1 exposed an old deep-fixture no-op version replacement, then typed provenance rewrite preserved the depth witness. Final TS parity28/0 exit0 with ESS_TYPES_NODE pointing at the already installed definitions. Latest distinct targeted groups total135passing cases, not a full-package or full-backlog count. Scoped fmt exit0. Strict all-target lint found one existing ignored-unit-pattern in one_time_recording; exact spelling corrected from _:() to ():(), rerun pending.

Caller-sensitive312 continuation established admitted same-command creation/bulk-action red and now owns bounded set_effects.rs phase context. Extended actual WASM matrix passed. Exact Go/TS diagnostic comparison exposed missing ESS-CF-NO-EVENT and ESS-CF-VIEW prefixes in the exercised no-event/snapshot-multiple-row branches; bounded existing runtime-template correction approved within story's cited go/ts scope. No broad diagnostic or source-admission change authorized. Frozen final caller matrix/review remains pending.

Disk pressure is shared with unrelated sessions. Root removed only completed own ess-cli compiler outputs958MiB. Worker reclaimed exact verified idle served-entry target/debug and target/tmp allocated4.64GiB, retaining/hash-verifying64 evidence files and archiving scratch logs. No worktree/source/other-session output deleted. Cargo-clean refused the older cache's missing CACHEDIR.TAG; exact prior-authorized compiler directories were inspected and cleaned as disposable storage, not lifecycle retirement. Observed free space recovered to roughly16GiB.

## Migration committed locally

Root committed the independently reviewed migration and canonical suites as8b3f128a0 in batch/consumer-runtime-20261002. Final TypeScript review fc5161f89dbe4c185ace8d68cf7ec200c16d760995555c017e6e553866bb4f55 approves80341f5e5269c087cd8c4dd817ce2b16f3c9a409a14f5c5a77030a380195ca8e with no findings and reviewer executions0. One behavior-neutral unit-pattern spelling correction in one_time_recording accompanies the measured strict-lint fix. Final cargo clippy -p ess-conformance --all-targets --locked -- -D warnings exited0; scoped formatting and diff checks exited0. RelatedField integrationeaf7fde99a remains independently reviewed and its21 tests passed in rootgroup3.

This is a local integration checkpoint. No branch published, no required full-package/release gate waived, no source release claimed. Native response story is active with explicit transaction authority and scope; nested observer story remains draft required work. General caller-sensitive312 is still being finalized across native/Go/TS/WASM and will be reviewed separately. Current primary tree remains untouched; integration tree retains only pre-existing dirty changelog/one-time-response documentation after this source commit.
