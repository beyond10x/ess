---
format: aep.planning-md/2
id: decision-blocker:authored-consumer-boundary-accounting
kind: decision-blocker
status: cleared
title: Decide exact accounting for unreachable authored-consumer model pairs
relations:
- blocks: task:consumer-accounting-authored-boundaries
revision: 3
---
## Approved requirement blocked

ESS evolution M9 requires complete feature-preservation accounting under the unchanged frozen
baseline. task:consumer-accounting-authored-boundaries owns these three authored profiles.

## Exact conflict and evidence

Fresh locked/offline Cargo metadata exits 0. Traversal from ess-domain reaches 91 dependency nodes
and does not reach ess-compiler. All 56 reported compiler-only model shapes match retained current
extraction, yielding 168 exact model/consumer/profile pairs for authored-admission, authored-assembly
and authored-validation. Their entrypoints neither accept nor return compiler IR/graph types.
The accepted reconciliation contract requires Supported or named Refused for these pairs, with no
permitted outside-boundary disposition. Arbitrary YAML rejection, package assertions or downstream
compiler behavior cannot truthfully discharge them.

Exact NON_AUTHORITY evidence and concrete proposal:
local-evidence:ess-evolution/waves/0010-opus-accounting/s3-reconciliation/exact-boundary-conflict.json
and local-evidence:ess-evolution/waves/0010-opus-accounting/s3-reconciliation/decision.md.

## Input needed and stopping condition

Operator decision requested asynchronously: permit a bounded versioned outside-consumer-boundary
proof for these exact pairs, preserving visible matrix totals, unchanged baseline/unknown eligibility,
fresh source guards and mandatory behavior at real compiler consumers; or retain policy and leave
these pairs unqualified. The amendment is NOT adopted and has no authority effect before approval
and its required design/source checks. No generic N/A, wildcard, baseline extension or extra product
requirement. Stop when the explicit policy decision is recorded; it does not close implementation.

The 57th worker incompatibility, wire:RawSpecFile#/definitions, already has an accepted aggregate
mechanism and is excluded from this decision. Root owns correction of the worker brief's misleading
earliest-boundary partition language. Independent service implementation continues.

## Explicit operator decision

The operator explicitly approved the bounded proposal in this conversation: "I approve it now",
in response to the coordinator's request to approve the 168-pair S3 accounting amendment.
This clears the decision requirement. It authorizes the finite versioned outside-consumer-boundary
proof for exactly the enumerated 56 compiler-only models across the three authored profiles.

All six constraints in s3-reconciliation/decision.md remain binding: unchanged frozen baseline and
unknown eligibility, visible exact tuples and conserved totals, no wildcard or generic N/A,
fresh source/dependency/entrypoint guards with stale or reachable proofs refused, mandatory real
compiler behavior, and reviewed versioned format with old-reader rejection and meaningful controls.
Existing review budgets are not reset. This is approval to implement that amendment, not evidence
that its design/source checks or final accounting have passed. S3 remains incomplete.

Next: produce and review the finite amendment under task:consumer-accounting-authored-boundaries,
then implement its guarded authority/qualification and finish per-profile S3 attribution. Stop at
the approved finite scope; further boundary exceptions require a separate explicit decision.
