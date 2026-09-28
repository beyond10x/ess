---
format: aep.planning-md/3
id: story:subject-guard-input-and-case-folding
kind: story
status: implemented
title: A subject guard compares with the input, and text compares without case
relations:
- decomposes: epic:retrofit-findings-20260927
- serves: vision:O2
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: cited
  path: crates/generate/ess-entity-runtime/src/lib.rs
- confidence: inferred
  path: crates/specify/ess-compiler/src/expression.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/command/finite.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/subject_fact.rs
- confidence: cited
  path: crates/specify/ess-domain/src/expression.rs
- confidence: cited
  path: crates/specify/ess-domain/src/primitive_admission.rs
- confidence: cited
  path: crates/specify/ess-domain/src/system.rs
- confidence: cited
  path: crates/specify/ess-primitives/src/predicate.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/go/predicate.go
- confidence: cited
  path: crates/verify/ess-conformance/src/go/runtime.go
- confidence: cited
  path: crates/verify/ess-conformance/src/scenario.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/text_match_format.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/ts/predicate.ts
- confidence: cited
  path: crates/verify/ess-conformance/src/witness.rs
- confidence: inferred
  path: schemas/generated/ess.schema.json
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-09-27T14:47:07Z", actor: "human:timo", revision: 6, imported: true}
- {from: "proposed", to: "active", at: "2026-09-27T14:47:41Z", actor: "human:timo", revision: 7, imported: true}
- {from: "active", to: "implemented", at: "2026-09-27T15:00:42Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"test_result":1}}, imported: true}
---
## Scope

- #157: `input.<field>` as an operand in a `when_subject: {predicate}` comparison.
- #140: `equals_ignore_case` / `in_ignore_case` over `String`, ASCII folding.

Design: `docs/design/value-expressions.md` § E6, E7 (accepted, not implemented).

## Acceptance

- A `when_subject` comparison of a stored field with `input.<field>` validates under the next
  source format and is refused below it; synthesis writes an equal and an unequal witness.
- The two text operators validate over `String` and its newtypes only; Rust, Go and TypeScript
  evaluators agree on ASCII folding; suites carrying them take a new suite format version; Entity
  Runtime lowering refuses them.

## Derived scope

Derived 2026-09-27 by `aep:story-scoper`. Every line is **cited** or **inferred**.

- **Primary surface:** `crates/specify/ess-primitives/src/predicate.rs` — cited: `Operand` (:284) gains an input operand, `TextOp` (:235) the two fold operators
- **Primary surface:** `crates/verify/ess-conformance/src` evaluators and synthesis — cited
- **Files:** `crates/specify/ess-domain/src/command/subject_fact.rs` (:280-330) — cited
- **Files:** `crates/specify/ess-domain/src/command/finite.rs` (:54) — inferred
- **Files:** `crates/specify/ess-domain/src/expression.rs`, `src/primitive_admission.rs` — cited (#95 changed both)
- **Files:** `crates/specify/ess-domain/src/system.rs` (:53, :98) — cited
- **Files:** `crates/verify/ess-conformance/src/scenario.rs` (`SUPPORTED_SUITE_FORMATS` :376) — cited
- **Files:** `crates/verify/ess-conformance/src/text_match_format.rs` — cited; extend rather than add a module — inferred
- **Files:** `crates/verify/ess-conformance/src/go/predicate.go`, `go/runtime.go` — cited
- **Files:** `crates/verify/ess-conformance/src/ts/predicate.ts`, `ts/runtime.ts` — inferred (TypeScript has no text operators today)
- **Files:** `src/witness.rs`, `src/synthesize/subject_fact.rs`, `src/synthesize.rs` — cited
- **Files:** `crates/generate/ess-entity-runtime/src/lib.rs` (`TextOp` lowering :3241) — cited
- **Files:** `crates/specify/ess-compiler/src/expression.rs` — inferred
- **Also likely:** `crates/infra/infra-spec/src/raw.rs`, `ess-conformance/src/{coverage,coverage_build}.rs`, `go/replay.go`, the schema, `models/toolchain/domains/specify.yaml` — inferred
- **Documents:** `docs/design/value-expressions.md` — cited; predicates, spec-versions, formats, CHANGELOG — inferred
- **Confidence:** medium — constructs and evaluators named by story and design; E6 operand path through the partition and compiler inferred
- **Would collide with:** any unit touching predicate operands/operators, source or suite format versions, the Go/TS evaluators, `witness.rs`/subject-fact synthesis, or Entity Runtime lowering
- **Not established:** where `input.<field>` is parsed in a subject predicate; whether `finite.rs` can compare two variables; where TypeScript evaluates subject predicates; the suite format numbers. #157 and #140 could be two stories.
