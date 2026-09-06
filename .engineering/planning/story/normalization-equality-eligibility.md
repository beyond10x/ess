---
format: aep.planning-md/1
id: story:normalization-equality-eligibility
kind: story
status: draft
title: Specify numeric equality eligibility and its format compatibility boundary
relations:
- derived_from: story:typescript-normalization-target
- informed_by: review-result:typescript-equality-binding
- serves: vision:O2
scope:
- confidence: inferred
  path: docs/design/normalization-equality-eligibility.md
revision: 2
---
# Define recipe equality eligibility with an explicit compatibility policy

## Problem

The checked expression type and runtime numeric representation currently supply
different equality authority. The checker admits ordinary comparable scalar kinds,
including Integer, and formats 5/6 additionally admit equality when both checked
operand kinds are exactly Binary64. General Number is not thereby admitted, and
Binary64 output provenance remains a separate compiler-owned contract.

The reference evaluator receives a recipe-wide format-5/6 flag. With that flag,
any two present f64-backed values compare before signed-integer eligibility is
checked. Source-schema Integer values can therefore take the floating comparison
path without having compiler-owned Binary64 kinds. This is a runtime authority
limitation, not evidence that static Number equality or implicit Binary64 output
conversion has been accepted.

Source evidence is frozen at `1f8e319cf153c348a6c434c6e74939f4aa587125`:

- [check.rs:988](https://github.com/beyond10x/ess/blob/1f8e319cf153c348a6c434c6e74939f4aa587125/crates/generate/schema-contract/src/realize/normalize/check.rs#L988)
  and [comparable:1039](https://github.com/beyond10x/ess/blob/1f8e319cf153c348a6c434c6e74939f4aa587125/crates/generate/schema-contract/src/realize/normalize/check.rs#L1039)
  establish the two distinct static admission routes.
- [execute.rs:21](https://github.com/beyond10x/ess/blob/1f8e319cf153c348a6c434c6e74939f4aa587125/crates/generate/schema-contract/src/realize/normalize/execute.rs#L21)
  validates the stage input and passes only a format-wide floating-equality flag.
- [eval.rs:362](https://github.com/beyond10x/ess/blob/1f8e319cf153c348a6c434c6e74939f4aa587125/crates/generate/schema-contract/src/realize/normalize/eval.rs#L362)
  takes the two-f64 bypass before calling the
  [signed-integer check](https://github.com/beyond10x/ess/blob/1f8e319cf153c348a6c434c6e74939f4aa587125/crates/generate/schema-contract/src/realize/normalize/eval.rs#L426).

A recorded reference CLI observation imported a source bundle with required
Integer-schema fields `a` and `b`, then evaluated equality for the literal
`{"a":1.0,"b":1.0}`:

| Recipe format | Actual reference result |
|---|---|
| 1 | Exit 1, `/branches/eq/0/value/condition`, `integer_representation` |
| 5 | Exit 0, JSON `true` |
| 6 | Exit 0, JSON `true` |

This is a reference `normalize-run` observation, not execution of a generated
Rust, Go or TypeScript library. Preserve the literal fixture and exact observed
findings as evidence. Other combinations remain proposed qualification cases
until executed.

## Binding compatibility constraint

**Frozen formats 5 and 6 keep their existing meaning.** This story does not
authorize an in-place correction that changes an admitted result into a refusal,
nor a target-specific guard that diverges from the reference. Static checked kinds
and output provenance must remain distinct from the current runtime behavior.

The repository [AGENTS.md](https://github.com/beyond10x/ess/blob/1f8e319cf153c348a6c434c6e74939f4aa587125/AGENTS.md)
requires a new format version when meaning changes. Before implementation, record
a binding decision that either retains the observed compatibility rule
or defines corrected operand eligibility behind an explicit new-format/migration
boundary. Determine the recipe format and any separately affected persisted
metadata/report consequences from their actual contracts; do not automatically
bump unrelated envelopes or assume no consequences because field layout is stable.
No future format number is selected by this draft.

## Required discovery and decision

1. Locate where checked operand authority could be retained through Plan/lowering
   and supplied to runtime equality for a new contract. Avoid deriving type
   authority from host-language floating representation. Account for equality in
   requirements, choices, nested collection conditions, optional operands and
   later stages without silently changing unrelated scalar operations.
2. State the intended new-contract behavior for exact signed Integer operands,
   typed Binary64 pairs, mixed operand kinds, optional/missing values and supported
   scalar unions. General Number equality and Binary64 output provenance require
   their own explicit decisions; neither follows from this observation.
3. Inventory all affected reference and generated Rust/Go/TypeScript equality
   paths and version-template routing. TypeScript paths must be refreshed after
   its current implementation integrates. Do not edit old templates merely to
   make all targets share a corrected implementation.
4. Bind compatibility before production changes: old format 1–6 behavior and
   existing emitted artifacts remain stable under the same generator identity;
   any corrected behavior is qualified at the selected future boundary.

## Acceptance

- The binding decision names equality authority, supported operand categories,
  optional/union treatment, exact refusal locations and the format/migration
  consequence before implementation is dispatched.
- The recorded Integer `1.0` / `1.0` case remains unchanged for formats 5/6.
  No old evaluator/checker/template patch is smuggled into another language target.
- A literal corpus distinguishes equal/unequal integral floating pairs, mixed
  integer/floating representations, signed zero, unsigned integers beyond i64,
  format families and statically refused general Number/mixed Binary64 pairs.
  Reuse the original observation as an independent expectation, not a newly
  generated golden result. Clearly label unexecuted controls.
- When a corrected contract is implemented, the reference and each supported
  generated target execute the applicable cases under both preserved and new
  format boundaries, with exact findings and retained old artifact byte witnesses.
  Native executions are reported separately from reference CLI observations.
- Documentation distinguishes static Kind admission, schema numeric admission,
  runtime comparison and output provenance. Integer arithmetic, Greater and
  schema const/enum equality do not inherit an unrequested semantics change.

## Scope status

This is a **discovery/design draft**, not an implementation-ready file scope.
The attached cited/inferred scope proposal contains three inspected reference
production files and the binding/format-governance reads. Plan metadata, format
routing and generated-target edges remain explicitly discovery-needed. Final
symbol ranges, target/test paths and any migration fixtures must be resolved
against the integrated source after the binding decision. No broad production
edit allowance, file-count estimate or automatic in-place fix is proposed.

## Current authored scope

The next authored artifact is inferred: docs/design/normalization-equality-eligibility.md. This design-only initial scope does not claim the unresolved future production scope is complete. The inspected reference files above are read evidence, not a scheduled write allowance. The retained followup-scope.json discovery packet records five cited read seams, four inferred discovery paths and unresolved target/template/test/metadata edges. Resolve and record a separate complete production scope before scheduling any implementation. This draft is informed by review-result:typescript-equality-binding; it does not block the current TypeScript parity unit, whose frozen behavior has been selected explicitly.
