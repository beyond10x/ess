---
format: aep.planning-md/1
id: task:consumer-accounting-native-model-behavior
kind: task
status: implemented
title: Execute reviewed binary model behavior through the native accounting path
owner: ESS maintainers
relations:
- derived_from: story:consumer-accounting-baseline-never-extended
- serves: vision:O2
revision: 7
---
## Outcome

Pair actual CLI loader behavior cases with closed model-behavior authority and native binary-unit
execution, consumed through accounting/3 without extending historical eligibility or reinterpreting
old accounting/receipt formats. This is one sequential implementation slice under the existing
accounting parent. The previous bounded mechanism task is implemented and remains unintegrated.

## Binding proposal and typed home

docs/design/consumer-model-behavior-native-cases.md fixes the proposed source/target/case authority,
execution stage and accounting/3 contract. specification:consumer-accounting-binary-case-source-boundary
records fresh source scoping and the required format consequence. Existing CellIdentity,
reconciliation::Identity and case/source/native coordinates remain the typed homes; concrete closed
Rust wrappers own the new formats. No AEP dependency, generic evidence registry or product entity.
Independent technical design review precedes implementation. Root selected /3 because binary
receipts change the nested admitted value/reference meaning of /2; the old readers remain closed.

## Acceptance

- Before new readers, add old-reader refusal fixtures for new envelopes/bin identities and preserve
  legacy authority and receipt bytes. Implement closed model-behavior/1 authority, execution/1 stage
  evidence, case/1 receipts and accounting/3. Reject duplicate raw JSON keys and unknown fields.
- Bind exact case identity, binary Cargo root, defining test module and AST/file bytes separately.
  Bind every claimed current cell/profile to its exact production declaration/body/module authority.
  No signature-only approval, renamed integration target, copied loader or fabricated execution.
- Use one private native execution path with target-kind-aware cache, exact Cargo artifact admission,
  retained native images and normal libtest listing/execution/freshness. Preserve old integration
  behavior and acquisition/2. Cases must actually execute1/pass1/fail0/ignore0.
- Preserve exact authority/candidate/plan/source/provider digest propagation. Accounting/3 uses a
  separate typed new-evidence member, requires disjoint legacy/binary IDs and exact combined
  planned/executed equality, and consumes every new claim exactly once. Existing metadata,
  reconciliation, acquisition and aggregate checks stay mandatory.
- Add diagnostic consumer-behavior execution through those same production functions. It emits
  zero qualified cells and never an ACCOUNTED result; missing/stale authority or unexecuted cases
  fail. It cannot replace full consumer-check or admit a partial reconciliation manifest.
- Implement seven actual load::specification cases covering reading contracts, component settings,
  subject-state selection, response payload ownership, error naming, periodic host mapping/delivery,
  and selection/event accessors. Observe returned IR or precise diagnostics from the real loader.
  Retain named causal controls/mutations. Do not separately reparse a fixture to claim loader behavior.
- Produce exact reviewed-claim candidates and causal evidence for root. The90cell preparation is
  not an adopted set: any unobservable/unproved candidate remains explicitly unclaimed. Aggregate
  parents use separate complete closure authority; no behavior blanket or new baseline unknowns.
- Pass actual native diagnostic execution, complete affected xtask/CLI suites, strict affected
  Clippy and formatting, with real selected/executed counts and source-stable receipts. Root reviews
  final authority/classification data. Independent code examinations follow exact local submission.

## Scope and ownership

Inherit the parent story's typed scope. Primary files are account/enforce/proposal/native/executor/
mod/preservation/consumer under consumer_coverage plus xtask main.rs, CLI load.rs and the binding
design. Inferred additions are model_behavior.rs, model_behavior_tests.rs, reviewed-model-behavior.json
and CLI load_accounting_tests.rs. Existing Cargo binary declaration needs no manifest change.
One source implementor owns this paired slice; no parallel writer touches accounting/native/CLI
module declarations. Root alone owns AEP/design/authority JSON/classification/preservation data,
commits and integration. Source scoper confidence high; no source or build was produced by scoping.

## Limits

This task does not close all8380replacement behaviors, remaining unaccounted cells, complete
reconciliation/aggregate adoption or the parent gate. The parent still requires actual default
consumer-check, task check and site-build, without skips or a smaller matrix. All source remains
unintegrated until the applicable full repository gate passes. No publication/release/deployment.

## Technical design review correction

review-result:native-model-behavior-design-pass-1 found one exact-equality gap. The binding design
now requires executed selected_cases/cases/claims to equal the plan, the plan to equal reloaded
source-validated authority, and every receipt key/identity/target/module/AST hash to match its exact
planned case. Per-receipt digests match independently computed run bindings. Preserved plan digests
cannot authorize changed repeated content or swapped receipts. Decisive mutation checks cover each
field family through the same qualification guard. Final technical design review precedes source
implementation; no authority or coverage is adopted by this correction.

## Design acceptance and implementation dispatch

Final review-result:native-model-behavior-design-pass-2 approves with zero findings after the one
first-pass equality correction. The complete reports are immutable; no third design review is
required. Root accepts the binding implementation contract within approved ESS evolution scope.
The design's acceptance-status-only amendment leaves its reviewed semantics unchanged. Proceed
through proposed to active for the paired executor/loader implementation; no code, authority data,
coverage receipt or full-parent acceptance is claimed by the design approval.

## Bounded implementation completion

Implementation and correction are submitted at36c6842bbfdf3cd0acdce99ba9b9c24a223a5bbb and
87c266c0a3c3b90f54dbb3c072d913e160ccb894. Final review-result:native-model-behavior-code-pass-2
returned nothing found; the exact two-review ledger is carried0/new0/resolved1. The first stale
model/shape diagnostic finding was corrected in production and recorded fixed. Both independent
stale/unknown assertions and the final later-stale-claim assertion are retained unchanged in meaning.
There is no third code examination for this unit.

Closed authority/execution/accounting3 readers and old-reader refusal/byte-preservation cases pass.
Source/target/module/AST binding and exact repeated content, disjoint legacy/model partitions and
claim-to-cell consumption pass through the real v3 planner and qualification guards. Seven actual
loader cases and their causal controls support27 reviewed Rust claims; root adopted their exact
authority and infrastructure classifications. No wire claim or aggregate closure was adopted.

Actual complete affected suite results: xtask233passed/0failed/3existingignored after correction;
CLI566passed/0failed/5existingignored from the unchanged CLI submission. Strict affected Clippy,
formatting and diff checks passed. Final independent focused results:10model cases,6accounting3
cases and1binary artifact boundary case passed. After adopting the final29-line test patch, root
executed its exact test in the implementation tree:1passed/0failed/0ignored, exit0. This test-only
addition and planning records do not alter the reviewed production implementation. No full-suite
count including the final addition is asserted.

The actual source-stable normal diagnostic retained for correction87c266c0 executed7cases for
27claims, every keyed receipt1/1/0/0, with0qualifiedcells and DIAGNOSTIC_ONLY. All22 native
commands exited0. Root and reviewer checked independent source/provider/authority/candidate/plan
digests and the retained native image. These are evidence for that submitted source; later test
and planning edits do not make the historical receipt current for a different source vector.
Current qualification must always rerun the production freshness and execution path.

Evidence remains under local-evidence:ess-evolution/waves/0004-ess-accounting/:
native-model-correction-pass-1-evidence/{implementation-result.md,coordinator-verification.md},
native-model-review-pass-2/{report.md,test.patch,adopted-test.log,adopted-test.exit}, and the
original native-model-behavior-evidence suite/causal-control logs. Source hashes, actual process
exits, diagnostic digests and review report hashes are recorded there and in immutable reviews.

This completes the acceptance of this bounded implementation task only. All source remains locally
submitted and unintegrated. The parent retains all other replacement/profile obligations, wire and
aggregate evidence, complete reconciliation, default consumer-check, task check and site-build.
No ACCOUNTED result, full repository gate, publication, release or migration cutover is claimed.
