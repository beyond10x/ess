---
format: aep.planning-md/2
id: task:consumer-accounting-aggregate-reference-closure
kind: task
status: active
title: Qualify aggregate reference transitions against the frozen witness
owner: ESS maintainers
relations:
- derived_from: story:consumer-accounting-baseline-never-extended
- serves: vision:O2
- depends_on: task:consumer-accounting-unchanged-behavior-admission
revision: 10
---
## Outcome

Make the existing five aggregate identities capable of proving the actual frozen-to-current
reference transition while retaining exact qualification and unresolved provenance. This is a
bounded prerequisite under story:consumer-accounting-baseline-never-extended, discovered while
preparing the four wire closures; it does not replace or reduce complete accounting.

## Demonstrated gap

aggregate.rs:313-334 rejects a changed external target when an added or removed edge crosses a
parent boundary. Frozen RawOutcome payload/sets leaves are strings; their current leaves reference
the changed RawPayloadSource definition. Both RawOutcome/properties and RawOutcome closures thus
hit that guard before child qualification. Root source comparison shows40added/1removed references
relative to the frozen witness, distinct from the unchanged refs across the two recent schema repairs.

For definitions, both endpoints are local and required even when the target schema is unchanged
(accounting_v2_tests.rs:494-529). The35changed definitions alone omit six required stable targets:
Field, Naming, Predicate, QualifiedName, StateName and TypeRef. Complete this41entry frontier with
actual paired behavior; the earlier66direct candidate identities are not a final admission cap.

## Design boundary and existing typed homes

Existing typed homes are aggregate::Row, Frontier, QualifiedCell and reconciliation::Identity.
No new persisted field or semantics is selected by this task creation. First reproduce the exact
real-witness refusal in Rust and write a binding design for bounded external reference proof.
Decide the format consequence explicitly before source changes: changed persisted proof meaning
or envelope requires a coordinated version decision under AGENTS.md. Finish the required design
review before implementing it. Never relax the old guard without equivalent explicit proof.

## Acceptance

- Retain exactly five aggregate identities, every frozen old hash, current-source matching,
  same-consumer/profile requirements and visible original unknown residual.
- Prove both local endpoints and any changed external endpoint using exact current model/shape,
  attributed cases and actual executed qualification. No prefix, fabricated cell, arbitrary JSON
  proof bag, sixth aggregate or partial reconciliation bypass.
- Reject missing/stale/foreign targets, missing or unexecuted cases, contradictory dispositions,
  incomplete/overlapping frontiers and dependency cycles. Mutation-test every new guard.
- Prepare all four real wire closures bottom-up and verify them through the production path
  against the complete reconciliation authority; a null or guessed digest is not acceptance.
- Preserve27existing Rust claim meanings, seven stable loader case IDs and all remaining parent
  gates. Run affected xtask/model/native checks, independent code examinations and required full
  task check/site-build before integration. No publication or migration is implied.

## Scope and coordination

Cited: crates/edge/ess-xtask/src/consumer_coverage/aggregate.rs, accounting_v2_tests.rs,
model_behavior.rs, reconciliation.rs, the frozen fixtures/stage1-7a6d7685-ess.schema.json and
docs/design/consumer-accounting-applicability.md. Inferred: a bounded binding design and explicitly
versioned closure authority if required. Root owns all AEP/DATA/design changes; source assignment
must coordinate with the active wire implementor and sole heavy-build token.

Root finding and exact comparisons are retained at
local-evidence:ess-evolution/waves/0004-ess-accounting/wire-behavior-evidence/aggregate-reference-adoption-finding.md.
The draft frontier file is historical proposal input, not adopted authority. No source repair,
new format, regression execution or closure qualification is claimed by this planning record.

## Executed frozen-witness reproduction

Two Rust regressions now use the actual compiled RawSpecFile JsonSchema provider and the frozen
stage1 wire witness through the existing aggregate validator. The outcome/properties parents
refuse with the exact changed-reference-endpoint reason before child qualification. The changed
definitions frontier of35 refuses as incomplete; adding Field, Naming, Predicate, QualifiedName,
StateName and TypeRef reaches the explicit missing-qualification refusal at41 entries. Both tests
pass with empty qualified-cell and executed-case inputs; they prove these refusals, not support.

Command: cargo test -p ess-xtask frozen_wire_ -- --nocapture. Actual exit0,2passed/0failed;
remaining targets filtered. Source accounting_v2_tests.rs SHA256
ae82fb4f741c64b75aca854ef379ce84e7f478514a599c9b0f758c69f75ae546.
Raw command/log/exit retained under local-evidence:ess-evolution/waves/0004-ess-accounting/wire-behavior-evidence/aggregate-reference-reproduction.
Log SHA256 b3e7b8a0998bd27904f3ea6056e31fddf39180dcf7579f086a54295f39d0dc79.
No production guard changed. A read-only scoper is mapping the bounded typed proof and format
consequence; binding design review, implementation, causal guards and full acceptance remain open.

## Proposed binding contract and caller reproduction

docs/design/consumer-aggregate-reference-closure.md proposes a closed aggregate/2 authority and
proof with exact ordered external reference records, target shapes/dispositions, complete
reconciliation and verified legacy/model execution. Accounting/3 and reconciliation/1 retain their
existing field meanings; the current v3 reader already accepts the correct disjoint execution
partition. This argument is proposed for independent review, not accepted implementation scope.

The additional v3_aggregate_model_case_routing_constructor_and_reader_disagree regression passed
with exit0,1passed/0failed. It reproduces the constructor assigning aggregate model cases to legacy
while the existing reader accepts correct routing. Source accounting_v3_tests.rs SHA256
1ee93c3ce261b2495c492583c8a5f95c0f03084168e874e367fb64f7d0b82da9; raw log SHA256
14e997ac2440d4d995c8d2ecec6b54167b0e7587edc388f179ab51040f63c56f.
Command/log/exit: local-evidence:ess-evolution/waves/0004-ess-accounting/wire-behavior-evidence/aggregate-model-routing-reproduction.

The scoped implementation touches aggregate.rs, enforce.rs, model_behavior.rs, mod.rs and their
accounting/model tests; reconciliation uses its existing typed maps. Root owns all source/DATA
coordination with the wire worker. No production guard or caller changed yet. The real local
frontiers remain3/1/41/1, with four external records owned by outcome/properties (including the
retained ExternalRef edge). Exactly five aggregate parents and all frozen witnesses remain.

## First technical design review corrections

review-result:aggregate-reference-design-pass-1 returned four blockers, preserved verbatim.
The binding proposal addresses all four before the second and final design round:

- Select ess-consumer-accounting/4 candidate/plan/qualified outputs for aggregate/2, with explicit
  aggregate_format and unchanged disjoint executor meanings. Preserve genuinely historical
  accounting/2 and /3 readers against aggregate/1 through explicit version-specific validation;
  reject cross-version and relabeled nested values. This supersedes the earlier proposal to keep
  the active outer format at /3. Reconciliation/1 and ESS product/IR formats remain unchanged.
- The private verified model token owns the exact validated immutable execution payload/digest
  alongside its claims/receipt keys. Qualification serializes only that owned value and accepts
  no second raw payload that could be swapped after verification.
- RequiredNullableShape uses a non-Option no-default custom null/string representation so omitted
  target-shape keys refuse instead of silently becoming null.
- A complete target-snapshot decision table admits Stable only for equal present shapes, behavior
  only for new/changed current targets, and Retired only for removed targets; all other classes
  refuse before payload qualification.

These are actual design amendments, not source implementation, passing guards or final review
approval. Preserve the original report and execute the second review on the revised frozen design.

## Accepted design and implementation assignment

The second and final technical design review approved with an explicit empty findings block.
Original report SHA256952b6958b43641f5b78d06775d25d453b844f36b40414b0545bda48797c0284b is
recorded verbatim as review-result:aggregate-reference-design-pass-2. All four first-round findings
were corrected; no third review or production qualification is claimed. The accepted binding
design changes only its status line after the reviewed substantive digest
c6d524e3b1a2ee69b46abbcb60467c1cc54f21ca2a9d6d0e83261395cb6866ad.

Under the approved ESS evolution initiative, implement the bounded accounting/4 and aggregate/2
contract in the existing consumer-coverage module. Preserve historical readers/fixtures, frozen
witnesses, five identities and full reconciliation. Root coordinates all DATA, design and planning;
the implementor owns only the explicitly assigned Rust files/tests. The wire implementor continues
in its separate source files; one heavy build remains serialized across both. Independent code
examinations and complete required gates follow actual implementation and source freeze.

# Stable external target under a changed consumer profile

Coordinator source inspection, 2026-09-15. This is a candidate defect awaiting an executable
reproduction, not an independent code review or observed failing test. Existing package246pass
and Clippy0 receipts remain valid for their source; they do not exercise this combination.

In aggregate.rs verify_reconciliation, the Some(old)/Some(current) external-target branch tests
the complete reconciliation Identity with `old != current`. A CompleteCurrentSubtree row has
different old/current profile hashes even when the external target shape is identical. The branch
therefore requires a replacement whose claim matches external_replacement_matches. That helper
accepts Supported/Refused/AggregateClosure and has no Stable arm.

The accepted consumer-aggregate-reference-closure.md chooses the exclusive external proof class
from target presence/shape: identical present target shapes on Added/Removed edges require Stable.
Changed-profile complete-current mode is also admitted by the contract. A complete current local
frontier can prove its exact behavior while the external edge only proves an unchanged target;
Stable grants no new target behavior. Global reconciliation still must prove all current cells.

Proposed reproduction:

1. Build a closed aggregate/2 CompleteCurrentSubtree row with parent old/current shape and different
   consumer profile, one fully qualified current local source, no residual and an Added edge to a
   target present with the same shape in both snapshots. Its sole external proof is Stable.
2. Use the real finite reconciliation resolver for the test baseline, with exact replacements for
   the changed-profile parent, local source and stable target. Parent claim is AggregateClosure;
   the stable target's ordinary current cell is Supported by its own executed case. No synthetic
   production token is involved; this is only a unit fixture.
3. Demonstrate the structural verifier accepts the complete current local partition and stable edge.
4. Require verify_reconciliation to accept the same exact resolution. Current source is expected to
   fail with `changed external target contradicts its reconciled replacement` despite valid proof
   classes. Preserve the actual red before changing source.
5. If confirmed, compare target shapes for this external replacement guard, retaining exact profile
   binding whenever target shape changes. Keep full-global reconciliation independently mandatory.
   Mutate the target shape and remove/contradict its replacement to preserve the existing refusal.

No ESS Rust edits or build were performed during wire's source hold. An attempted read-only
followup to the completed aggregate implementor was refused because the three active child slots
were occupied; no worker was resumed. Next resume it after a slot is free, before source submission.

## Executed stable-target profile correction

The coordinator's source concern was reproduced through the real reconciliation resolver. A
CompleteCurrentSubtree row with a changed consumer profile and an Added edge to a same-shape
external target passed structural verification, then failed reconciliation with
`changed external target contradicts its reconciled replacement`. Actual red:0pass/1fail/176filtered,
exit101. The repair compares the two target shapes before requiring a matching external replacement
claim; full-global reconciliation still requires the target's current behavior replacement.

Named green:1pass/176filtered, exit0. Full ess-xtask package:177unit+70integration passed,
3ignored/0failed, exit0. Strict all-target Clippy and scoped rustfmt passed. A preserved first Clippy
run found only the new test's line-count lint; a reasoned test-only expectation resolves that lint.
Exact receipts live under local-evidence:ess-evolution/waves/0004-ess-accounting/
aggregate-reference-evidence/stable-target-profile-first-{red,green}, full-ess-xtask-stable-profile,
strict-clippy-stable-profile-2 and scoped-rustfmt-stable-profile. Coordinator read actual raw results.

Source adoption and complete accounting are still pending. Fresh consumer extraction terminated1
because the new FORMAT_V4 concrete entry lacks an explicit classification. Its actual retained
inventory yields66new accounting-helper entries and3removed aggregate identities; independent finite
classification review precedes root adoption. No baseline/claim/aggregate/reconciliation authority
was generated or silently accepted. Independent code examination and full repository gates remain.

## Exact aggregate helper classification adoption

Independent review-result:aggregate-entry-classification-pass-1 approves66 OwnedHelper additions
and3 stale removals, findings[]. Original report SHA
b103f7f8e800bbf84a203d04b3720bd22df0892e673b62604cba6d5874eab757 was recorded unchanged.
Root rechecked all four frozen production hashes and original authority339c21bd, then adopted
exact reviewed additions with expanded per-role reasons and macro_ast_sha256:null. No other
existing classification value changed; jq exact delta check exited0/true.
Current classification SHA61a93ad866a790c83c4e66625e1ae5997ad1f7493d2eb3a8cb26192be0a335bf.
The original authority and finite additions remain in external evidence. Classification does not
qualify behavior; the interrupted extraction remains exit1 and must be rerun on current source.
Baseline, behavioral claim DATA, aggregate closure and reconciliation authority remain unchanged.
