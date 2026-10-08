---
format: aep.planning-md/3
id: story:go-normalization-pattern-semantics
kind: story
status: archived
title: Qualify bounded ECMA-262 patterns in Go normalization
relations:
- derived_from: story:source-pinned-data-normalization
- serves: vision:O2
scope:
- confidence: cited
  path: CHANGELOG.md
- confidence: inferred
  path: Cargo.lock
- confidence: cited
  path: crates/edge/ess-cli/tests/normalization.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize.rs
- confidence: inferred
  path: crates/generate/schema-contract/src/realize/normalize/go.sum.txt
- confidence: inferred
  path: crates/generate/schema-contract/src/realize/normalize/go_runtime.go.txt
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
revision: 20
transitions:
- {from: "draft", to: "proposed", at: "2026-09-06T07:58:25Z", actor: "agent:specification-planner", revision: 9, imported: true}
- {from: "proposed", to: "active", at: "2026-09-06T07:58:25Z", actor: "agent:specification-planner", revision: 10, imported: true}
- {from: "active", to: "archived", at: "2026-10-08T09:55:13Z", actor: "human:timo", revision: 20, decided_on: {"recorded":{"test_result":1,"review_outcome":2}}}
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

Re-derived 2026-09-28 by `story-scoper` against `46e367ab2`. Each line is marked **cited** (read from the story or the tree) or **inferred** (a reading that may be wrong).

- **Status on this tree:** partially satisfied — only the exact base64 pattern is qualified; every other pattern refuses; no general ECMA-262 design/corpus — cited (`go_target.rs:14,181-187`; `generate-artifacts.md:494-497`; `CHANGELOG.md:1343-1346`)
- **Primary surface:** `crates/generate/schema-contract` Go normalization target — cited
- **Files:** `src/realize/normalize/go_target.rs:14,27,159-194` (`BASE64_PATTERN`, `check_schema_support`, `go_schema_pattern` refusal) — cited
- **Files:** `src/realize.rs:161,430-466` (`collect_patterns`) — cited
- **Files:** `tests/normalization_go.rs:197,228,265,301,404`; `tests/fixtures/normalization_base64.rs` (shared with `tests/normalization_rust.rs:22`) — cited
- **Files:** `src/realize/normalize/go_runtime.go.txt`, `go.sum.txt` if a bounded matcher is embedded — inferred
- **Files:** `tests/normalization_base64_adversary.rs` (asserts `go_schema_pattern` at :57) — cited presence; inferred change
- **Files:** new corpus fixture(s) under `tests/fixtures/` — inferred
- **Files:** `Cargo.lock` only if a Rust dependency is added — inferred
- **CLI boundary:** `crates/edge/ess-cli/tests/normalization.rs:711` — cited
- **Docs:** `docs/design/source-pinned-data-normalization.md:290-300,336-350`; `website/docs/guides/generate-artifacts.md:488-497`; `CHANGELOG.md` — cited
- **Confidence:** high on location; low on size — matcher choice and limit policy are undecided.
- **Would collide with:** any unit editing Go normalization, `realize.rs` pattern collection, schema-contract normalization tests, `ess-cli/tests/normalization.rs`, the normalization design page, the generate-artifacts guide — inferred
- **Stale in previous scope:** omitted `go_runtime.go.txt`/`go.sum.txt`; "Current Boundary" paragraph predates base64 unit `c8b8eb10a` — cited
