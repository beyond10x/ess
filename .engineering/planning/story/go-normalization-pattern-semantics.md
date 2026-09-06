---
format: aep.planning-md/1
id: story:go-normalization-pattern-semantics
kind: story
status: active
title: Qualify bounded ECMA-262 patterns in Go normalization
relations:
- derived_from: story:source-pinned-data-normalization
- serves: vision:O2
scope:
- confidence: cited
  path: CHANGELOG.md
- confidence: cited
  path: crates/edge/ess-cli/tests/normalization.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/go_target.rs
- confidence: cited
  path: crates/generate/schema-contract/tests/fixtures/normalization_base64.rs
- confidence: cited
  path: crates/generate/schema-contract/tests/normalization_base64_adversary.rs
- confidence: cited
  path: crates/generate/schema-contract/tests/normalization_go.rs
- confidence: cited
  path: crates/generate/schema-contract/tests/normalization_rust.rs
- confidence: cited
  path: docs/design/source-pinned-data-normalization.md
- confidence: cited
  path: website/docs/guides/generate-artifacts.md
revision: 15
---
## Evidence

The standalone Go normalization target uses jsonschema/v6 v6.0.2, whose default
pattern engine is Go RE2. The reference uses jsonschema 0.52.1 with ECMA-262
translation and a bounded backtracking engine. An accepted source pattern such
as ^(?=a)a$ is not supported by RE2, and overlapping syntax alone does not prove
equivalent Unicode, anchors or character classes. Unbounded backtracking or
wall-clock-dependent success is not an acceptable deterministic replacement.

## Outcome

Qualify bounded ECMA-262 schema-pattern validation for Go normalization against
the pinned reference, or retain explicit source-located refusals for semantics
that cannot be preserved. Do not silently fall back to RE2 or skip patterns.

## Acceptance

- A binding design declares supported syntax, Unicode/anchor semantics and a
  deterministic resource-limit policy before implementation.
- Generated Go executes a shared corpus covering ordinary expressions,
  lookaround, backreferences, Unicode, anchors, empty matches and pathological
  backtracking, with reference success/refusal evidence.
- Unsupported patterns refuse before artifact publication at their bundle and
  schema pointer, including referenced definitions. Unselected schemas do not
  broaden refusals.
- Removing go_schema_pattern is supported by this evidence, not by a matcher
  merely accepting the same source text.

## Current Boundary

The resumed unit qualifies exactly the frozen model base64 pattern documented in
docs/design/source-pinned-data-normalization.md. Every other selected pattern still
refuses before publication, including nested model propertyNames constraints. The
private schema-position collector preserves structural report obligations and formats.

Implementation c8b8eb1 is committed on the integration branch, with native Go and Rust
corpus evidence. Adversary pass 1 found the previously omitted map-key pattern class;
the unchanged case was reproduced red and corrected. Pass 2 found nothing further.
Full workspace and documentation gates are running before publication. The broader
source-pinned normalization parent remains active with TypeScript, raw capture,
positional input and model binary64 capabilities tracked separately.

The incoming independent Scope section is retained as historical scoping evidence.
The typed scope now names the actual files, retiring the unused Cargo.lock and new
pattern-design-page reservations. The binding lives in the existing shared design.

## Scope

- Primary surface: `crates/generate/schema-contract` — cited; `src/realize/normalize/go_target.rs:12,100` owns Go generation and the current `go_schema_pattern` guard, which examines every selected stage input/output root and its structural obligations before constructing files. `src/realize.rs:163,444` supplies selected referenced closure and actual pattern obligations; arbitrary JSON keys named pattern are not obligations.
- Runtime and packaging within the primary surface — cited; `src/realize/normalize/go_runtime.go.txt:65,71,149` owns reusable offline validators, compiler construction and schema-refusal conversion; `go_target.rs:25,36` and `go.sum.txt` embed runtime files and exact generated dependency identities. `target.rs:120,160` retains selected schemas, bundles and deterministic report/file digests.
- Existing evidence within the primary surface — cited; `tests/normalization_go.rs:89,133,168` covers repeat generation/provenance, referenced lookahead refusal, an unselected pattern beside a usable plain root, and generated Go execution under the opt-in go-typecheck lane. `tests/fixtures/normalization_go_tests.go.txt:14` compares complete outcomes/refusals, preserves caller bytes and repeats concurrent calls. Existing Rust/reference execution is the oracle, not evidence that Go already supports patterns.
- Corpus extension within the primary surface — inferred; extend the existing fixture/harness area with shared ordinary/lookaround/backreference/Unicode/anchor/empty-match/pathological cases, generation refusal locations, selected and referenced roots, every branch/stage, deterministic repeated outcomes and pattern-free/Rust compatibility controls. Exact fixture filenames and supported syntax await binding.
- CLI publication boundary: `crates/edge/ess-cli` — cited; `tests/normalization.rs:268` currently fixes the lookahead refusal and asserts empty stdout and an absent destination. Preserve this boundary with the eventual supported/refused corpus. The package token matches existing CLI reservations; the expected edit is this focused test file, while `src/normalize.rs:117` already calls complete target generation before publication.
- Binding document: `docs/design/go-normalization-pattern-semantics.md` — inferred; the story explicitly requires a supported-syntax, Unicode/anchor and deterministic-limit design before implementation. This proposed new page is its narrow home; no binding is supplied by this scope.
- Existing design: `docs/design/source-pinned-data-normalization.md` — cited; lines 284–307 bind the Go schema validator, blanket pattern refusal, selected closure, stable finding multiset/order, pinned dependencies and native execution evidence. Update only the pattern boundary supported by the later design and measured corpus.
- Public guide: `website/docs/guides/generate-artifacts.md` — cited; lines 323–331 explicitly state every selected pattern refuses with `go_schema_pattern`; any qualified subset needs corresponding support/refusal wording.
- Dependency reservation: `Cargo.lock` — inferred; a generator-side qualification/parser dependency may require the workspace lockfile, separate from emitted Go checksums already inside schema-contract. Retire this reservation if the binding establishes no Rust dependency change.
- Confidence: high — cited; the story names the exact guard and dependency mismatch, and the source/tests identify generation, runtime, publication and documentation boundaries. Matcher selection and resource policy remain deliberately unbound.
- Would collide with: schema-contract or ess-cli package edits, either named design document, the public generation guide, or Cargo.lock — inferred; use the six exact scheduling tokens below. The CLI package and possible lockfile collide with planned OpenAPI reservations even when intended implementation symbols differ.
- Exclusions — inferred; no grammar, normalization recipe/report format, global reference-validator policy, structural-target semantics, TypeScript runtime, CLI command/options, OpenAPI logic, public CLI/format reference pages, navigation, infrastructure predicates or cross-repository delivery implementation is required by this bounded story. A later design that changes these boundaries must first refresh scope.

Independent scope recorded at published fadbc674 on2026-09-06 for complete scheduling inventory. No matcher design, implementation or status change is included. The source report and unresolved design questions remain target/review-boundaries-7/go-pattern-scope/scope-report.md. This story stays outside the selected review wave.
