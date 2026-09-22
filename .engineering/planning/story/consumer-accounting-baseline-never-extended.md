---
format: aep.planning-md/1
id: story:consumer-accounting-baseline-never-extended
kind: story
status: active
title: Reconcile changed consumer obligations without extending initial eligibility
owner: ESS model and consumer package maintainers
relations:
- serves: vision:O2
- informed_by: initiative:ess-evolution
scope:
- confidence: cited
  path: Cargo.lock
- confidence: cited
  path: crates/edge/ess-cli/Cargo.toml
- confidence: cited
  path: crates/edge/ess-cli/src/coverage.rs
- confidence: cited
  path: crates/edge/ess-cli/src/input_discovery.rs
- confidence: inferred
  path: crates/edge/ess-cli/src/input_discovery_accounting_tests.rs
- confidence: cited
  path: crates/edge/ess-cli/src/load.rs
- confidence: inferred
  path: crates/edge/ess-cli/src/load_accounting_tests.rs
- confidence: inferred
  path: crates/edge/ess-cli/src/load_mode_accounting_tests.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/accessor_cli.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/compact_conformance.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/coverage_browser.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/coverage_cli.rs
- confidence: inferred
  path: crates/edge/ess-cli/tests/evolution_coverage_accounting.rs
- confidence: inferred
  path: crates/edge/ess-cli/tests/evolution_main_cli_accounting.rs
- confidence: inferred
  path: crates/edge/ess-cli/tests/evolution_main_cli_go_runtime_expansion.rs
- confidence: inferred
  path: crates/edge/ess-cli/tests/fixtures/evolution-loader-modes
- confidence: inferred
  path: crates/edge/ess-cli/tests/fixtures/evolution_coverage_accounting/
- confidence: inferred
  path: crates/edge/ess-cli/tests/fixtures/evolution_main_cli_accounting/
- confidence: inferred
  path: crates/edge/ess-cli/tests/fixtures/evolution_main_cli_accounting/go-runtime-expansion/
- confidence: cited
  path: crates/edge/ess-cli/tests/replay_fidelity_browser.rs
- confidence: inferred
  path: crates/edge/ess-cli/tests/support/browser.rs
- confidence: cited
  path: crates/edge/ess-xtask/src/consumer_coverage/account.rs
- confidence: inferred
  path: crates/edge/ess-xtask/src/consumer_coverage/accounting_v2_tests.rs
- confidence: inferred
  path: crates/edge/ess-xtask/src/consumer_coverage/aggregate.rs
- confidence: cited
  path: crates/edge/ess-xtask/src/consumer_coverage/consumer.rs
- confidence: cited
  path: crates/edge/ess-xtask/src/consumer_coverage/enforce.rs
- confidence: cited
  path: crates/edge/ess-xtask/src/consumer_coverage/entry-classifications.json
- confidence: cited
  path: crates/edge/ess-xtask/src/consumer_coverage/executor.rs
- confidence: cited
  path: crates/edge/ess-xtask/src/consumer_coverage/feature-preservation.json
- confidence: inferred
  path: crates/edge/ess-xtask/src/consumer_coverage/fixtures
- confidence: cited
  path: crates/edge/ess-xtask/src/consumer_coverage/initial-baseline.json
- confidence: cited
  path: crates/edge/ess-xtask/src/consumer_coverage/mod.rs
- confidence: inferred
  path: crates/edge/ess-xtask/src/consumer_coverage/model_behavior.rs
- confidence: inferred
  path: crates/edge/ess-xtask/src/consumer_coverage/model_behavior_tests.rs
- confidence: cited
  path: crates/edge/ess-xtask/src/consumer_coverage/native.rs
- confidence: inferred
  path: crates/edge/ess-xtask/src/consumer_coverage/nested_execution.rs
- confidence: inferred
  path: crates/edge/ess-xtask/src/consumer_coverage/outside_boundary.rs
- confidence: inferred
  path: crates/edge/ess-xtask/src/consumer_coverage/outside_boundary_tests.rs
- confidence: cited
  path: crates/edge/ess-xtask/src/consumer_coverage/preservation.rs
- confidence: cited
  path: crates/edge/ess-xtask/src/consumer_coverage/profiles.json
- confidence: cited
  path: crates/edge/ess-xtask/src/consumer_coverage/proposal.rs
- confidence: inferred
  path: crates/edge/ess-xtask/src/consumer_coverage/reconciliation.rs
- confidence: inferred
  path: crates/edge/ess-xtask/src/consumer_coverage/reviewed-aggregate-closures.json
- confidence: cited
  path: crates/edge/ess-xtask/src/consumer_coverage/reviewed-candidates.json
- confidence: inferred
  path: crates/edge/ess-xtask/src/consumer_coverage/reviewed-model-behavior.json
- confidence: inferred
  path: crates/edge/ess-xtask/src/consumer_coverage/reviewed-outside-boundary.json
- confidence: inferred
  path: crates/edge/ess-xtask/src/consumer_coverage/reviewed-reconciliation.json
- confidence: inferred
  path: crates/edge/ess-xtask/src/consumer_coverage/reviewed-scenario-acquisition.json
- confidence: cited
  path: crates/edge/ess-xtask/src/consumer_coverage/rust.rs
- confidence: inferred
  path: crates/edge/ess-xtask/src/consumer_coverage/scenario_acquisition.rs
- confidence: cited
  path: crates/edge/ess-xtask/src/consumer_coverage/tests.rs
- confidence: cited
  path: crates/edge/ess-xtask/src/consumer_coverage/wire.rs
- confidence: cited
  path: crates/edge/ess-xtask/src/main.rs
- confidence: cited
  path: crates/specify/ess-compiler/src/graph.rs
- confidence: cited
  path: crates/specify/ess-compiler/src/ir.rs
- confidence: cited
  path: crates/specify/ess-compiler/src/refs.rs
- confidence: cited
  path: crates/specify/ess-compiler/src/resolve.rs
- confidence: cited
  path: crates/specify/ess-compiler/tests
- confidence: cited
  path: crates/specify/ess-composition/src/lib.rs
- confidence: inferred
  path: crates/specify/ess-composition/tests/evolution_composition_accounting.rs
- confidence: inferred
  path: crates/specify/ess-composition/tests/fixtures/evolution_composition_accounting/
- confidence: cited
  path: crates/specify/ess-domain/Cargo.toml
- confidence: cited
  path: crates/specify/ess-domain/src/binding/periodic.rs
- confidence: cited
  path: crates/specify/ess-domain/src/refs.rs
- confidence: cited
  path: crates/specify/ess-domain/src/spec.rs
- confidence: inferred
  path: crates/specify/ess-domain/tests/evolution-boundary-fixtures
- confidence: inferred
  path: crates/specify/ess-domain/tests/evolution_boundary_accounting.rs
- confidence: cited
  path: crates/verify/ess-conformance/assets/coverage-admission.js
- confidence: cited
  path: crates/verify/ess-conformance/src/admission.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/authored.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/coverage_build.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/runner.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/web.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/web_replay.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/evolution_conformance_accounting.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/fixtures/evolution_conformance_accounting/
- confidence: cited
  path: crates/verify/ess-diff/tests/graph.rs
- confidence: cited
  path: docs/design/consumer-accounting-applicability.md
- confidence: cited
  path: docs/design/consumer-accounting-reconciliation.md
- confidence: cited
  path: docs/design/consumer-model-behavior-native-cases.md
- confidence: inferred
  path: docs/design/consumer-nested-execution.md
- confidence: inferred
  path: docs/design/consumer-outside-boundary-accounting.md
- confidence: cited
  path: docs/design/consumer-wire-behavior-period-parity.md
- confidence: cited
  path: docs/design/consumer-wire-container-completeness.md
- confidence: cited
  path: docs/design/review-consumer-coverage.md
- confidence: inferred
  path: ess/consumer-outside-boundary
- confidence: cited
  path: schemas/generated/ess.schema.json
revision: 50
---
## Outcome

The actual default consumer gate accepts complete, finite, reviewed accounting for current source
without extending the frozen initial unknown eligibility or suppressing behavioral execution.

## Historical issue and current diagnosis

This preserves the existing issue identity. Source change 9572af9b introduced thirteen measured
ESS changes after the initial accounting freeze; later source work repaired earlier extraction
and case-validation refusals without reconciling all current model/profile cells. The old issue's
147,931 figure was current initial unknowns, not missing pairs. Current bounded analysis at
41da2281e99402602c25d8faf219fc75b1954d04 reconciles 65,336 problems: 55,590 unaccounted and 9,746
stale. There are 618 added models, 62 changed, ten removed and two changed original profiles.

The full task check in the ESS evolution scope tree failed at this existing execution-plan stage,
exit 201, before any behavioral cases. Scope/design edits introduced none of the measured source
drift. Evidence: local-evidence:ess-evolution/waves/0000-scope/consumer-accounting-analysis-result.md.

## Accepted decision

docs/design/consumer-accounting-reconciliation.md and consumer-accounting-applicability.md are
accepted under approved ESS evolution revision1. The applicability amendment passed independent
design review with zero findings, recorded as review-result:consumer-accounting-applicability-pass-1.
Finite implementation scoping is complete in
local-evidence:ess-evolution/waves/0004-ess-accounting/implementation-scope-result.md.
Existing Profile, Cell, Claim, PlannedCell and Baseline types remain the typed homes; no new product
entity or ESS dependency on AEP is introduced.

Keep eight literal acquisition profiles as separately mandatory executed obligations; retire exactly
14,504 old model tuples plus790 other removed-model tuples, without counting the80 overlap twice.
Only five aggregate identities may use AggregateClosure with fresh structural proof and qualified
same-consumer children. ShapeDelta retains visible unresolved unchanged provenance only for unchanged
profiles; changed profiles require CompleteCurrentSubtree with no unknown inheritance. The possible
395 rows are limits, not qualifications. Keep all3,482 changed-profile rows as ordinary behavioral
work; no ProfileCompatibleUnknown bridge, broader metadata rule or native-boundary relaxation.

Preserve the frozen baseline digest, all owners/follow-ups and157,677 eligible pairs;157,068 is the
earlier historical freeze. Unchanged historical unknowns remain owned by the separate existing
epic:qualify-initial-consumer-baseline.

task:consumer-accounting-v2-mechanism owns the first implementation slice: closed v1/v2 formats,
finite reconciliation, eight acquisition executions and private aggregate mechanisms. The parent
retains complete finite behavioral qualification and actual consumer-check/check/site-build gates.
Source scoping discovered that existing CaseIdentity admits only integration tests. Preserve that
old reader; new binary-unit identities belong only to the new acquisition/v2 authority and must
bind target kind/root and actual case source separately through the same exact native executor.
No code or behavioral accounting has yet been qualified by these design/scope decisions.

## Acceptance

- Add exact finite retirement/replacement without deleting or regenerating old authority. Wrong
  old/new identities, duplicate/extra/missing tuples and changed-source reuse refuse.
- Preserve unchanged eligible unknowns. Added/changed current obligations need exact Supported or
  named Refused behavior, except the unchanged six metadata rows and the five exact aggregate
  proof modes in docs/design/consumer-accounting-applicability.md.
- Keep all eight acquisition profiles in the owned inventory and require their eight separately
  executed obligations. Retire only the exact old model tuples; exclusion never drops their tests.
- AggregateClosure requires exact fresh structural proof and same-consumer child qualification.
  Changed profiles inherit no unknown; their3,482 unchanged-model rows still need finite behavior.
- Retain case/source/profile/native-runtime authority. No package alias, live prefix, panic-as-refusal,
  auto-owner or new unknown eligibility qualifies anything. Keep initial-baseline.json byte-identical.
- Use ess-consumer-accounting/2 for changed provenance and retain closed v1 history readers; add
  old-reader refusals before new envelopes/dispositions. Preserve all unrelated ESS formats.
- Review the design, demonstrate decisive reds and causal mutations, independently review the
  implementation, then run task consumer-check with zero stale/unaccounted cells and actual
  required case/guard/acquisition execution. Partial framework progress does not close the story.
- Pass default task check and task site-build on the final local candidate. CI's configured skip
  remains separate and does not satisfy local acceptance.

## Scope

- crates/edge/ess-xtask/src/consumer_coverage/initial-baseline.json — cited; frozen read-only authority.
- crates/edge/ess-xtask/src/consumer_coverage/enforce.rs, account.rs and mod.rs — cited; typed input,
  admission, stage/provenance output and execution flow.
- crates/edge/ess-xtask/src/consumer_coverage/reviewed-candidates.json and profiles.json — cited;
  existing exact claim/profile authority; do not silently change a bounded consumer's meaning.
- crates/edge/ess-xtask/src/consumer_coverage/entry-classifications.json and feature-preservation.json
  — cited; finite helper/case mapping must remain consistent with actual source.
- docs/design/consumer-accounting-reconciliation.md and review-consumer-coverage.md — cited;
  accepted authority and existing evidence policy. The six-row metadata design stays unchanged.
- Narrow Rust reconciliation module, finite manifest and exact owner behavior tests — inferred;
  establish concrete paths/cell attribution before implementation dispatch.
- Confidence: high for the measured refusal and safe authority rules; exact behavior case groups
  still need owner scoping. No implementation worker is assigned until that scope is reviewable.

## Scope and authorization

This pre-existing correction is required by the already-approved ESS evolution full local gates.
It remains local work. Publication, release and deployment are not part of acceptance. It does not
replace the complete initiative with a smaller consumer-accounting objective.

## Acquisition production-source binding clarification

The coordinator inspection review-result:consumer-accounting-acquisition-source-inspection found
that consumer.rs declaration hashes include function signatures but omit bodies. Exact current
provider/source freshness alone therefore cannot authorize a reviewed acquisition implementation.
No scenario authority was adopted before this finding.

Keep entrypoint_sha256 and all existing profile hashes unchanged in meaning. The selected closed
row now additionally binds entrypoint_source (the exact production input_discovery.rs path),
entrypoint_ast_sha256 (its actual authored source_item_sha256 including the body), and
entrypoint_source_file_sha256 (the production module's raw source bytes including local helpers).
Retain separate owner-case AST/source hashes and normal source/tool/native checks around execution.
Candidate extraction obtains the actual production fields; full candidate/reviewed-row equality
authorizes only the reviewed body and module. No global hashing change or filesystem abstraction.

Because these are new persisted fields, the acquisition authority/candidate/plan/proof/qualified
family is version2. Refuse old/1 data instead of reinterpreting it. Preserve a closed old-reader
rejection test. The unadopted signature-only/1 proposal remains historical evidence; no unrelated
ESS format, baseline eligibility or parent accounting/v1 meaning changes. Accounting/2 names the
explicit nested acquisition format.

The correction is proved with an actual authored body containing a discarded default-directory
probe, processed by the production AST extractor and candidate builder. Its signature must remain
equal: show the old guard's acceptance gap, then corrected guard refusal. Also retain an on-disk
mutation through compiled extraction and normal guard entry, restore source, and execute all eight
normal exact cases. Editing a hash string to a made-up mismatch is not the required source control.
Root reviews and adopts only the regenerated complete eight-row authority. No incomplete matrix,
retirement-only proposal or partial execution closes the parent or qualifies missing model behavior.

## Binary model-behavior slice and format amendment

A fresh source scoper found the existing extractor already distinguishes binary target roots from
case modules and Cargo artifact selection already accepts exact target kinds. The next slice pairs
actual load::specification cases with closed model-behavior authority/execution and the native
binary-unit path. Root selected accounting/3 because new binary evidence changes nested receipt
meaning; preserve closed /1 and /2 readers/bytes and acquisition/2. The proposed binding contract is
docs/design/consumer-model-behavior-native-cases.md; technical review precedes implementation.

Additional cited scope: CLI load.rs; xtask main.rs and consumer.rs; existing preservation.rs;
the new design page. Inferred scope: model_behavior.rs, model_behavior_tests.rs,
reviewed-model-behavior.json and CLI load_accounting_tests.rs. Existing accounting/native/executor
paths remain shared and serialize under root. Root alone adopts authority/classification data.
Scoper confidence high; no writes or builds. This is the next behavior slice, not a third review
of the previously corrected mechanism. Seven actual loader cases and candidate90cells need causal
observation; unproved rows remain explicit. Complete reconciliation/aggregate authority and every
remaining behavior, including all8380replacement identities, stay under this parent acceptance.

## Complete residual delivery routing

The unchanged-cell implementation is delivered and its agreed checks are complete; root's full
affected ess-domain/ess-cli suite also exited0 (raw log SHA
3bcf58173365f16f452f56ec0588800a8b678e8fb9431e9211d1defdb561faed). Code acceptance and complete
accounting remain open. The frozen baseline remains unchanged.

Current exact partition692ca176ff0204e117db7547eef005e8d7196dd4dd5e0f0436f297de178da381
supersedes historical numerical estimates in this body:157677baseline cells partition into
133156unchanged,15926retirements and8595replacements;198112current cells include59394obligations
with no reviewed attribution. Complete identity candidate3b7996ffe41e93bdfca31962a2156112881de8ca717db70c11730cd68ee4aa68
contains24521exact historical decisions but no invented replacement claims or review authority.

One bounded source-routing assignment now maps all82current consumers to actual production/test
boundaries and identifies finite implementation units for the complete residual. This is needed
because the exact partition proves identity/counts, not behavioral assertion coverage. It cannot
introduce new baseline/applicability/aggregate exemptions, grant any claim or become another
expanding mechanism prerequisite. Stop at the complete table and exact source/check/stop contracts;
root then dispatches justified implementation. Local contract:
ess-evolution/waves/0004-ess-accounting/remaining-consumer-source-routing-contract.md.

## Loader aggregate completion contract

Approved M9 remaining S1/S2 closure work; existing story retains ownership. The three loader modes
have adopted direct behavior and successful 27-case native diagnostic execution. Five aggregate
parents per profile still need exact closure and replacement records before complete accounting.

One bounded Opus assignment authors all 15 candidate rows using the existing closed schemas,
frozen old witnesses, current same-consumer child claims and accepted reference-closure rules.
Only external evidence directory writes; canonical source/planning/authority is read only to the
worker. No builds, new checker/mechanism, baseline/policy changes or S3 amendment dependency.
Exact contract: local-evidence:ess-evolution/waves/0010-opus-accounting/loader-aggregate-completion-brief.md.
Stop at the finite complete candidate rows/handoff or exact missing structural inputs. Root owns
adoption, whole-manifest digest binding and actual guard/final-gate execution. Candidates alone
qualify nothing, and unrelated consumer groups remain mandatory. No completed worker is reopened.

## Loader aggregate completion result

Loader aggregate author CLOSED, handle 74553 exit 0. Delivered 15 candidate closure rows and 18
initial replacement records; canonical authority is not installed because complete global digest
and genuine structural/native qualification remain pending. No further worker scope is appended.

Root checked all 15 identities/residuals, 132 direct frontier case bindings and nine external
Supported proofs. Corrected three replacement reason mismatches by deriving from adopted claims;
completed the full loader reconciliation fragment: 249 decisions across three profiles (54 removed
model retirements, 180 behavior replacements, 15 aggregate replacements). Current source has 18
removed models, including eight old ExternalRef object positions; old 10-model estimates are stale.
All 621 new current models per profile are ordinary new claims, not historical reconciliation.

Root corrected exactly 132 S2 behavior-reason Cases tails in reviewed-model-behavior.json from stale
direct-file names to the already-correct per-mode case arrays. Every executable binding, case,
identity and original S1 claim is unchanged; old authority is retained. Current raw SHA
e29ee5e3659e7b1a1b587f49b0ebce34070ebfcfc013857fb43808c79f25fe81.
The earlier native receipt remains historical; no repeat diagnostic is required for prose edits.

Exact root report and NON_AUTHORITY fragments:
local-evidence:ess-evolution/waves/0010-opus-accounting/loader-aggregate-completion/coordinator/review.md.
No aggregate guard ran, no new cells qualified, no baseline eligibility changed. Current plan_v4
already selects the implemented unchanged-behavior preference; the author's stale implementation
warning must not create another task. Next: compose these records with complete remaining groups,
then execute final source/native guards and whole gates. Full M9 remains open.

## Current implementation and required-check progress

S3 finite policy design remains admitted; mechanism source compiles, old-reader rejection and all
17initial controls passed under the complete declared environment. Required package/lint checks
found fixture-coordinate/lint issues; same source contract corrected those and root reruns them.
Root-owned loader reason prose used a root-home delimiter;111reasons now use commas only, preserving
all non-reason JSON and the frozen baseline. Existing gate unchanged; qualification still pending.

S4 source checks reached all three integration targets: resolution29passed4failed on candidate
expectations/helper rendering, privateparts15passed and references16passed. Source-only correction
is delivered; final required rerun, package/lint and constructor/reference causal controls remain.
The nonexistent Naming fixture prerequisite is explicitly withdrawn in the task's corrected section.

S5 is admitted as the next existing whole group with exact source/test scope, finite candidate
accounting, causal controls and stopping condition. No new scope, baseline eligibility, native
format or additional review cycle. Complete matrix, full task check/site and local integration
remain required; these scoped observations do not close M9.

## Original S8 browser acceptance repair

Approved M9 browser-paired-replay acceptance failed in an actual Firefox run because the producer emits periodic bindings but closed readers accepted only events. The next run exposed compiler-only context/read tables in the projection. Existing counts were insufficient. Root owns a bounded three-path repair: typed periodic reader, matching browser grammar, and projection of the periodic source contract. All three paths are recorded in this story scope. Original S8 owns parity refusals, actual Firefox execution, event-only byte preservation and required package checks; root owns final integration. Stop at existing periodic behavior, no new scheduler/vocabulary/baseline scope. Exact local receipt: local-evidence:ess-evolution/waves/0010-opus-accounting/s8/coordinator-periodic-repair.md. Both patches applied and whitespace checks0; runtime acceptance pending.

## Coordinator composition checkpoint, 2026-09-18

The existing reconciliation/2 authority is being assembled from adopted exact claims and the approved 168 boundary identities. The non-authority composed candidate retains all 249 earlier loader actions exactly. Of 157677 frozen eligible tuples, 133156 are unchanged exact identities; 18337 historical decisions can be assembled from current adopted evidence, while 6184 changed historical tuples still lack an adopted replacement. These are data membership counts, not executed or qualified dispositions. The full current matrix also includes new model obligations, so 6184 is not the total remaining acceptance workload. No unknown eligibility or baseline bytes changed.

The candidate is deliberately outside the repository authority path and marked INCOMPLETE_NON_AUTHORITY_COMPOSITION; no partial manifest is admitted. The existing strict resolver consumes every changed old tuple and refuses missing decisions, so installing this incomplete manifest or rerunning the full gate unchanged would only produce a known refusal. Aggregate reconciliation digests cannot be finalized before this manifest is complete. Existing source/claim cases remain reusable only at their exact entrypoints and actual covered semantics; source drafts without accepted attribution are not automatically replacements.

Original S9 has resumed after its nested-execution prerequisite passed, owner active5; all127 saved drafts remain intact and48 accepted source paths were refreshed. Its author owns all three original Go/browser/release profiles and one final handoff, while root retains adoption and final qualification. M7 retains its existing whole acceptance contract. Two independent bounded build lanes, root no Cargo; no S10–18 fan-out, new authority engine, new review budget or additional prerequisite is opened by this composition.

Local evidence: local-evidence:ess-evolution/waves/0010-opus-accounting/reviewed-reconciliation.composed.candidate.json, SHA256 c22beb25db4e2fb6608f793a2efb486b3e9777fc5f482ee276de4b9ffe8b7ee7. Final frozen baseline remains3dd8dff59335c8a77c93c2734118566fd1b2d5165c0590d0aa9be397374a47de. Root next action is to adopt only complete accepted source/claim groups, finish exact historical replacement/aggregate bindings, then run genuine current-vector guards and the full consumer-enabled gate. Existing unresolved policy and behavioral gaps remain explicit.
