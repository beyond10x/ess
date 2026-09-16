---
format: aep.planning-md/1
id: blocker:live-pilot-consumer-coverage
kind: blocker
status: open
title: Live conformance entries and changed profiles lack consumer qualification
relations:
- blocks: story:compact-live-authoring
- blocks: story:callable-live-evaluator
withholds: test_result
revision: 5
---
## Observation

The full `task check` at source 65ed702ab2d5ec1162c35571d124750c106e84f5
passes formatting, strict Clippy, all workspace tests including the formerly
failing metadata tests, rustdoc, command smoke, projection and support checks.
It exits 201 at consumer-check with:

`unclassified concrete consumer entry ess_cli::bin(ess)::enum::ConformCommand/variant/Live; finite review required`

The fresh consumer inventory has 145 entries absent from entry-classifications.json,
including the new CLI boundary and compact compilation, recipe, binding and trace
modules. The refusal preserves diagnostic accounting for 2,430 model obligations
and 90 bound profiles, with 66,571 provisional stale/unaccounted messages. No
behavior cases or metadata guards executed in this refused consumer run, and no
completed qualification receipt exists. These provisional messages are not 66,571
executed test failures and must not all be attributed to this change: the original
model-source roots are unchanged from the new upstream base.

Retained evidence: ess-rebased-check.log, ess-consumer-refusal.json and
ess-unclassified-live-entries.json. Native extraction remains under the fresh
consumer run ending in run-1789545179881235728-3544796. The original full-gate
failure is preserved. Independently invoked fuzz-check, release-check and
action-check pass; they do not replace the refused lane.

## Clearing condition

Review and classify the finite new entries under docs/design/review-consumer-coverage.md.
Compare fresh upstream-base accounting with this candidate to distinguish inherited
model drift from changed live-conformance consumer profiles. Preserve the sealed
initial baseline and existing profile claims. Add exact supported/refused cases
for the new obligations and execute them through the native checker. New entries
cannot inherit package-wide support and changed profiles cannot reuse old unknown
eligibility. A corrected classification file alone is not behavioral qualification.

Run task consumer-check and the complete repository gate to zero before landing.
Feature checkpoints remain published for review; no merge or release is claimed.

## Release-base comparison 2026-09-16

A clean managed checkout of exact release/base d85f083cf0705c4c403fc3c56e09c7bf5f2ac272 reproduces consumer-check refusal under the required Rust 1.98.1 profile. It has three unclassified concrete entries: Naming::code, ess_synth::alias module and ess_synth::code_aliases re-export. Provisional accounting is 2430 model obligations, 90 bound profiles, 147506 eligible baseline-unknown cells and 66571 stale/unaccounted diagnostics, with zero executed cases/metadata guards. These are inherited accounting gaps, not 66571 failed behavioral tests. Exact refusal and extraction are retained in metrics-base-consumer-evidence. The feature adds its own live-compiler/recipe/session/temporal entries which still need finite classification and supported/refused witnesses; passing focused tests does not discharge that gate. No baseline fingerprint, exemption or profile has been refreshed to make this pass.

## Exact metrics candidate comparison

Fresh extraction of implementation 10be80e1 discovers 169 unclassified concrete entries: the same three inherited entries plus 166 feature additions. Compared with the clean d85f083c release checkout, rust-inventory.json, wire-inventory.json and provider-schema.json are exactly equal. Both report the same 66571 provisional stale/unaccounted diagnostics; the set difference is empty in both directions. No cases or metadata guards execute while extraction/classification is refused. Retain metrics-consumer-comparison.json and metrics-candidate-consumer-evidence alongside the base evidence. The new compact compiler boundary and its live helpers still need explicit finite classifications and behavioral witnesses; this task has not changed the sealed baseline or claimed that existing profile eligibility covers a new consumer.

## Follow-up execution

The operator requested coverage qualification and coordinated release after the live pilots. Continue under this blocker and the existing compact authoring/evaluator stories: explicitly bind the compact compiler and composition-retention boundary, classify the finite added entries with bounded reasons, attach exact changed/control and named-refusal witnesses, then rerun consumer-check. Preserve the accepted initial baseline. Inherited accounting drift must remain visible and requires behavioral qualification, not refreshed eligibility.

## Finite live consumer review (2026-09-16)

The 169 previously unclassified entries are explicit classifications: 166 live additions and three inherited naming/alias helpers. The compact compiler's model-retention and lowering entries form `compact-live-compilation`; public recipe expansion forms `typed-live-fixture-expansion`. Native live admission, occurrence ledger, step dispatch and target callbacks remain helpers of existing admission/runner boundaries. CLI file acquisition is orchestration, not a second compiler. None of these helper classifications grants support.

The compact changed/control witness alters original enum variants, observes the exact `fixture.routing.Changed: literal violates item.state` refusal, and recompiles both valid controls. Its two attributed enum cells are refused claims. The fixture witness proves the expanded command's actual response-field pointer ownership and capture-slot linkage, attributed only to ResolvedCommand.response. These tests pass independently; the consumer lane still refuses before same-run case execution, so it reports zero qualified cases.

Fresh consumer-check gets through classification and candidate review, then refuses with 71428 accounting diagnostics across 2430 model obligations and 92 profiles. This is the inherited 66571 diagnostics plus 4857 unaccounted cells across the two new profiles after the three finite claims. There are 10171 stale diagnostics and 61257 unaccounted diagnostics; none is an executed behavioral test failure. The sealed initial baseline and its fingerprints are unchanged. Clearing this gate requires concrete behavior/refusal evidence for the outstanding cells and an explicit treatment of stale baseline cells; refreshing old eligibility would hide the problem.

Retained evidence: dev-qualification/consumer-witnesses.log, recipe-consumer-tests.log, compact-consumer-tests.log, and the fresh consumer extraction ending run-1789556990559615426-1147970. Release and integration remain blocked.
