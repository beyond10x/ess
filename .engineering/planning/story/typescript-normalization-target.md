---
format: aep.planning-md/1
id: story:typescript-normalization-target
kind: story
status: draft
title: Execute source-pinned normalization in native TypeScript
relations:
- derived_from: story:source-pinned-data-normalization
- serves: vision:O2
- depends_on: story:normalize-positional-array-input
scope:
- confidence: cited
  path: CHANGELOG.md
- confidence: cited
  path: crates/edge/ess-cli/src/normalize.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/target.rs
- confidence: cited
  path: docs/design/source-pinned-data-normalization.md
- confidence: cited
  path: docs/design/typescript-normalization.md
- confidence: cited
  path: website/docs/guides/generate-artifacts.md
- confidence: cited
  path: website/docs/reference/cli.md
- confidence: cited
  path: website/docs/reference/formats.md
revision: 3
---
## Outcome

Implement the native TypeScript execution target of source-pinned-data-normalization
under docs/design/typescript-normalization.md. JSON text and internal bigint preserve
the existing signed/unsigned integer boundary without caller-side precision loss.
This is the coordinator's stated engineering choice, not a claim of a new operator
answer to the optional preference question.

## Evidence

The existing normalize.rs exposes only Rust/Go execution targets. normalize/target.rs
owns source retention and reports. crates/edge/ess-cli/src/normalize.rs:49 declares
only Rust and Go in its normalization target enum. Structural TypeScript generation
already exists and is a separate capability. The read-only scoper identified the
native JSON parser/serializer and number-only schema validators as insufficient for
exact integer parity.

## Acceptance

- Execute every current recipe operation through a standalone ES2022 TypeScript
  library with JSON-text input/result and located reference-compatible refusals.
- Retain token-kind numeric distinctions, signed wrapping, binary64 input policy,
  missing/null distinction, ordering and lazy evaluation.
- Validate the admitted source-schema profile without lossy bigint coercion;
  refuse every unqualified constraint before returning a partial adapter.
- Preserve complete bundle/model pins and deterministic report/file accounting;
  keep existing Rust/Go output bytes stable.
- Expose CLI generation and read-only drift checking with source-input protection.
- Native TypeScript/Node execution covers the shared fixtures, prototype-sensitive
  names and numeric/schema/error-order boundaries described by the binding design.

## Scope

- normalize.rs, normalize/target.rs and CLI src/normalize.rs (cited).
- New native TypeScript target, schema-feasibility module, runtime templates and
  normalization_typescript test harness (inferred from the scoper's implementation
  design; confirm the exact split before implementation).
- Binding design, public generation/CLI/format documentation and changelog (cited).
- Confidence: medium (inferred); execution surfaces are established, schema and
  numeric qualification still need implementation evidence.
- Sequence after the recovered base64 unit because target packaging, tests and
  shared design/documentation overlap (cited).

## Refreshed binding after raw capture

The binding design now includes the concrete standalone API/package, exact
integer and floating representation, raw capture and retained-document helpers,
Unicode-scalar ordering, prototype-safe records, and stable schema-finding
multiset presentation. The first native lane records TypeScript6.0.3 and the
actual Node/V8 engine; browser or unmeasured engine support is not inferred.

Focused numerical preparation at immutable source6c78676 established a feasible
strategy: JSON grammar first, bigint integer fast paths, guarded Number conversion,
reference decimal-key equality for exact floats, and zmij-compatible layout over
native shortest digits. Node22.23.1 matched13,455 finite patterns,28 midpoint
cases and35 existing numeric cases;29 scalar pipelines also matched. These are
private strategy experiments, not target implementation or format5 qualification.
The actual dependency graph enables serde_json float_roundtrip through jsonschema;
its inactive fallback parser must not be ported.

Implementation follows the integrated Binary64 and positional contracts. It must
refresh their checked metadata, operation variants, input policies, target-report
versions and frozen legacy sources before dispatch. Raw format4 helpers/cases are
required; seven decoded-value API cases are named as inapplicable to the text-only
TypeScript API. No required corpus branch may be silently skipped.

The native runtime validates a closed qualified source-schema profile without
coercing bigint into Number. The independent profile audit is still completing;
unsupported refinements must refuse before any artifacts. A pre-existing pinned
validator discrepancy in numeric uniqueItems is being separately recorded and
must not be silently copied or described as a TypeScript-introduced defect.
No TypeScript adapter, passing full gate, consumer migration or release is claimed.
