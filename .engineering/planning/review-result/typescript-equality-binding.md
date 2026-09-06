---
format: aep.planning-md/1
id: review-result:typescript-equality-binding
kind: review-result
status: active
title: Qualify frozen numeric equality behavior in the TypeScript binding
relations:
- reviews: story:typescript-normalization-target
revision: 1
---
# Equality binding review

Verdict: **accept the coordinator's compatibility choice**, with an explicit
binding qualification. The source supports matching the frozen reference rather
than adding a TypeScript-only refusal. This review does not certify the mutable
TypeScript implementation or establish native Rust/Go/TS equality parity.

The reviewed binding is `docs/design/typescript-normalization.md`, SHA256
`89ddb811972b879e001deb82f0e21dd6cffeecd1261c46e05a26f665bbb9630e`, at coordinator head `bfd8bd7e8e61e970f96ee90b05c3044b492c2993`.
The three reference files below are byte-identical there and at frozen executable
source `1f8e319cf153c348a6c434c6e74939f4aa587125`. Exact source/observation hashes are in `evidence.json`.

## Why the observed behavior is reachable

| Boundary | Frozen authority | Consequence |
|---|---|---|
| Static equality admission | [check.rs:988](https://github.com/beyond10x/ess/blob/1f8e319cf153c348a6c434c6e74939f4aa587125/crates/generate/schema-contract/src/realize/normalize/check.rs#L988), [comparable:1039](https://github.com/beyond10x/ess/blob/1f8e319cf153c348a6c434c6e74939f4aa587125/crates/generate/schema-contract/src/realize/normalize/check.rs#L1039) | Formats 5/6 additionally admit two operands whose checked kind is exactly Binary64. Independently, Integer and supported scalar alternatives remain comparable. General Number does not acquire equality admission. |
| Runtime authority | [execute.rs:21](https://github.com/beyond10x/ess/blob/1f8e319cf153c348a6c434c6e74939f4aa587125/crates/generate/schema-contract/src/realize/normalize/execute.rs#L21) | Source validation runs first; evaluator Context receives a recipe-wide `binary64` boolean enabled for formats 5/6. No operand-specific checked kind is passed to equality. |
| Runtime equality | [eval.rs:362](https://github.com/beyond10x/ess/blob/1f8e319cf153c348a6c434c6e74939f4aa587125/crates/generate/schema-contract/src/realize/normalize/eval.rs#L362) | With that flag, any two present f64-backed values compare through as_f64 equality before signed-integer eligibility is checked. Remaining numeric operands pass through the existing integer check. Missing operands produce false. |
| Signed-integer eligibility | [eval.rs:426](https://github.com/beyond10x/ess/blob/1f8e319cf153c348a6c434c6e74939f4aa587125/crates/generate/schema-contract/src/realize/normalize/eval.rs#L426) | `as_i64` is required; floating representations refuse `integer_representation`, even when their numeric value is integral. |

The recorded observation imports a bundle root whose required `a` and `b` source
schemas are `integer`, then compares them. For the reported literal
`{"a":1.0,"b":1.0}`, format 1 exits 1 at
`/branches/eq/0/value/condition`, `integer_representation`; formats 5 and 6 exit 0
with `true`. These are source-schema Integer fields, not evidence that an imported
Number schema gained compiler-owned Binary64 identity. The observations record
reference CLI execution only; this review did not rerun them or inspect the
mutable TypeScript implementation.

This explains the discrepancy without changing input or type admission: the same
runtime representation can arise in an admitted Integer equality expression, and
the format-wide evaluator cannot distinguish its checked operand kinds.

## Required wording and scope boundary

The current binding's lines 193–199 correctly require compiler-owned Binary64
metadata for static typing and output provenance. However, “Runtime typeof number
is insufficient authority” must be qualified so it is not read as an instruction
to refuse the observed Integer expression in TypeScript. Suggested replacement:

> Static checking admits the additional Binary64 equality case only when both
> operand kinds are Binary64 in formats 5/6. This does not change general Number
> admission or allow Integer/Number/Decimal values to fill Binary64 outputs.
> The frozen reference evaluator has a separate runtime limitation: formats 5/6
> compare any two f64-backed operand values before integer eligibility, including
> integral floating tokens admitted by Integer source schemas. TypeScript must
> preserve that located reference behavior; it must not add a target-only type
> guard or change old format behavior. Carry compiler-owned type provenance for
> checking/lowering/output rules, while retaining this explicitly qualified runtime
> compatibility behavior pending the separate core-contract follow-up.

Also replace the format-family shorthand “enables Binary64 equality” near line
522 with “enables the existing format-wide floating equality branch,” followed by
this limitation. The decision does not widen checker/schema feasibility, introduce
coercion, change arithmetic or Greater, or authorize an old evaluator/checker or
Rust/Go template edit. Format 1 retains the observed refusal; the same conclusion
for formats 2–4 follows from execute.rs and is source-inferred, not newly measured.

Three scope pitfalls to avoid:

1. **Do not claim every Integer equality now permits floats.** The bypass requires
   both present runtime values to be floating representations. A mixed integer /
   floating pair reaches integer checking and refuses; two integers still take
   the integer path, including its signed-64 eligibility. These consequences are
   source-inferred controls, not new executions in this review.
2. **Do not replace recipe equality with schema numeric equality.** Schema
   integer/const/enum admission and this evaluator have separate rules. The observed
   integral floating pair does not authorize general Number equality, unchecked
   numeric conversions, or Binary64 output provenance.
3. **Do not present a design decision as native conformance evidence.** The supplied
   record measures one reference input in formats 1/5/6. Preserve exact older
   Rust/Go bytes and qualify native TypeScript behavior in its existing test scope;
   claim cross-target agreement only for cases actually executed. Existing native
   Rust/Go agreement was neither inspected nor rerun here.

## Separate core-contract follow-up

A separately governed core story should choose whether Integer equality must
require exact signed-integer representations in every format, or whether the
current floating-pair behavior becomes an explicit lasting compatibility rule.
If tightening is selected, investigate compiler/checker-owned per-expression
operand authority delivered to evaluation (rather than a recipe-wide switch),
including nested collection conditions, optional operands and stage boundaries.
Inventory reference and generated Rust/Go/TypeScript runtime families together.

The decision must explicitly assess recipe-format compatibility: changing an
already admitted result from `true` to refusal is observable semantic change, even
if source and schema bytes are unchanged. No recipe/report version bump is needed
merely to preserve the current behavior in this TypeScript target. A future core
change must choose its version/migration and old-template preservation policy;
this review does not preselect a number.

A minimal follow-up corpus should distinguish the observed equal floating pair,
unequal integral floating pairs, mixed integer/floating pairs, signed zero,
unsigned values beyond i64, format 1–4 versus 5/6, and statically refused general
Number/mixed Binary64 operand types. Those are proposed qualification cases, not
newly measured outcomes. Root owns filing that story; no planning record was
created or changed by this review.

All writes are limited to this assigned private review directory. No repository,
Git, AEP, worktree lifecycle, native build or implementation-file mutation occurred.
No build lane was acquired; all commands are terminal.
