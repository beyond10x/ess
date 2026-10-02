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
  path: crates/verify/ess-conformance/src/synthesize/singleton.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/ts
- confidence: cited
  path: crates/verify/ess-conformance/src/web.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests
- confidence: cited
  path: crates/verify/ess-diff/src/impact.rs
- confidence: inferred
  path: docs/design/scenario-initial-state-and-cross-caller-witnesses.md
revision: 29
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
