---
format: aep.planning-md/1
id: story:binding-list-selection-contract
kind: story
status: active
title: Define bounded binding list selection from actual reducer behavior
tags:
- priority-high
relations:
- decomposes: task:ess-gaps-measured-in-a-consumer-specification
- serves: vision:O2
- informed_by: story:binding-mapping-bounded-accessor
- depends_on: story:conformance-compact-json-output
scope:
- confidence: cited
  path: CHANGELOG.md
- confidence: cited
  path: Cargo.lock
- confidence: cited
  path: crates/generate/ess-synth/Cargo.toml
- confidence: cited
  path: crates/generate/ess-synth/src/failure.rs
- confidence: cited
  path: crates/generate/ess-synth/src/go/layout.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/go/selection.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/plan.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/feasibility.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/rust/selection.rs
- confidence: cited
  path: crates/generate/ess-synth/src/selection.rs
- confidence: cited
  path: crates/generate/ess-synth/tests/selection_reading_integration.rs
- confidence: cited
  path: crates/specify/ess-compiler/src/ir.rs
- confidence: cited
  path: crates/specify/ess-compiler/src/resolve.rs
- confidence: inferred
  path: crates/specify/ess-compiler/src/selection.rs
- confidence: cited
  path: crates/specify/ess-compiler/tests/oracle_fixture.rs
- confidence: cited
  path: crates/specify/ess-domain/src/accessor.rs
- confidence: cited
  path: crates/specify/ess-domain/src/binding.rs
- confidence: cited
  path: crates/specify/ess-domain/src/expression.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/selection.rs
- confidence: cited
  path: crates/specify/ess-domain/tests/periodic_selection_integration.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/admission.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/go/runtime.go
- confidence: inferred
  path: crates/verify/ess-conformance/src/runner.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/scenario.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/selection.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/list_selection_adversary.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/selection_reading_integration.rs
- confidence: cited
  path: crates/verify/ess-diff/src/change.rs
- confidence: cited
  path: crates/verify/ess-diff/src/delta.rs
- confidence: cited
  path: crates/verify/ess-diff/src/diff.rs
- confidence: inferred
  path: docs/design/binding-list-selection.md
- confidence: cited
  path: docs/design/binding-mapping-bounded-accessor.md
- confidence: cited
  path: schemas/generated/ess.schema.json
- confidence: cited
  path: website/docs/guides/verify-conformance.md
- confidence: cited
  path: website/docs/guides/write-a-specification.md
- confidence: cited
  path: website/docs/reference/formats.md
revision: 5
---
## Outcome

Design and implement a bounded way for a binding to select from a declared list, with explicit no-match and multiple-match behavior. This is gap 2 of task:ess-gaps-measured-in-a-consumer-specification, not an extension silently admitted by the current bounded-accessor story.

## Observed source and consumer witness

Current MappingSource admits flat EventField and Literal only (crates/specify/ess-domain/src/binding.rs:541); the active accessor design explicitly refuses list traversal. Existing typed predicate support in crates/specify/ess-domain/src/expression.rs includes collection checks and quantifiers, but those are not binding selection. Reuse compatible typed semantics where justified; do not invent a general second expression engine.

The consumer's call-leg reducer is more specific than the summary's pair of predicates: it uses case-insensitive comparisons, ordered first matches, a switch whose first branch can exclude the second for the same leg, skips nil legs, and falls back to the first id-bearing leg when no agent leg matched. A recording flag derives from both selected legs. Exact private citations are retained in local-evidence:ess-evolution-20260910/priority-wave/gap-scoping; the aggregate source task retains the original measured cost. A selector alone must not be described as completing that whole binding without those additional behaviors and authenticated context.

## Acceptance and sequencing

This story is complete when the supported call-leg selection decision table executes consistently in native Rust/Go and conformance, wrong selection/order/fallback mutants fail, and an actual reviewed consumer use or exact remaining host-owned obligations demonstrates the delivered boundary; additional context or multi-leg derivations remain explicitly open and do not become a claim that the whole call binding is complete.

Required verification:

- Before syntax or code, a binding design states the finite predicate vocabulary, declared field/type resolution, collection order, selection cardinality, missing/multiple results, null/invalid inputs, limits and source/suite compatibility consequences.
- Reproduce the actual call-leg decision table, including fallback and mixed matching legs, and distinguish selection from any still host-owned transformations. No fabricated source fields or invented third enum variant may satisfy it.
- Implement supported selection consistently in domain/compiler, descriptions, native Rust/Go, semantic diff and conformance. Unsupported derivations remain named capability obligations.
- Generated native and conformance mutation cases distinguish wrong leg, wrong first-match order, missing fallback and invalid/multiple matches. No full local or ownership gate is required by this planning record.
- Claim downstream completion only from an actual reviewed consumer binding against the delivered source, preserving authoritative context and conversion obligations.

## Scope

Cited: crates/specify/ess-domain/src/binding.rs and expression.rs; crates/specify/ess-compiler/src/resolve.rs and ir.rs; the current bounded-accessor design. Inferred until design: ess-synth, ess-gen, ess-conformance and ess-diff consumers of the future selection plan. The active accessor changes these same binding/compiler/conformance surfaces: implementation follows its PR and reviewed plan contract. Design discovery may occur earlier.

## Priority and authority

Priority high, design-first after immediate correctness/read-fixture work. The user requested the aggregate gaps be planned ASAP, not speculative selector implementation in the current accessor PR. No existing selection story was found; this creates the missing planned unit rather than duplicating the active accessor story.


## Integration ordering correction

The accessor relation is informed_by, not a dependency on completing its separate four-row adoption acceptance. Its reviewed source baseline must be present before touching the colliding mapping/compiler paths. The explicit depends_on compact-output relation is a scheduling constraint over their shared conformance scenario surface: deliver the smaller writer change before another new mapping representation. It does not claim list selection semantically requires compact whitespace. Reassess actual scopes at the next wave preflight; no new unit is dispatched by this planning record.


## Approved concurrent implementation

The operator's later all-gap implementation request supersedes the earlier design-only scheduling paragraph. Compact predecessor is now implemented with5 integration CLI cases. Coordinator admits local-evidence:ess-evolution-20260910/priority-wave/list-selection/design-proposal.md: ordered first/first_present with same-list occurrence exclusion, typed finite predicates and explicit binding-local input preparation at the existing exact-conversion seam. No new producer list field or duplicate conversion registry. Native Rust/Go and conformance must execute direct declared-list fixtures and the actual thirteen-row mixed-leg/fallback table; host preparation, raw enum decoding, authenticated delivery and stateful updates remain exact named obligations.

Implementor scope_accessor owns wt-09052fd37c43, branch impl/gap-list-selection, base32c765bc, lease ess-list-selection-01a089ee. Selection-only mapping/domain/compiler/native/conformance/diff symbols are separated from periodic cause/host mapping, clock named types, and coordinator subject-state commands. Root retains aggregate imports, conditional source3/suite6-7/report2/target-failure3 selectors, coverage routing, public/generated files and publication. Exact source symbol assignment before edits and returned hunk headers plus merge-tree inspection govern concurrency; no new approval/full local gate.

## Implementation admission boundary

The first implementation review reproduced an invariant-invalid unselected tail that returned a selected value. The immutable record is review-result:list-selection-implementation-adversary-1; its source-derived case reached the intended assertion after valid-source and valid-payload controls passed.

The bounded correction rejects reachable invariant-constrained struct/newtype inputs, including list aliases and nested members, when this selector implementation cannot evaluate their constraints. Source declarations remain legal; executable observation construction and native helper generation must return explicit capability refusals rather than discard their meaning. A generated prepared-input helper cannot bypass this admission. Native static refusal uses the closed SelectionConstraint cause in the coordinated unreleased target-failure/3 vocabulary. Supported unconstrained direct-list and prepared-input cases must remain executable.

Clock-reading attachments, integrated by the parallel clock story, must likewise retain their meaning or be explicitly refused at this boundary. The coordinator owns that cross-feature check at integration. This is a bounded capability decision, not proof of consumer adoption or a claim that arbitrary declared record invariants are implemented. The original red test and exact candidate evidence remain privately retained; current durable tests must assert the explicit refusal contract and the useful supported paths.

<!-- public-import-provenance:begin -->
Final-state public import from the privately retained integration store, not an event-equivalent historical replay. New CLI creation/status changes occur at import time; original evidence retains its observed timestamp. Superseded bodies/scopes and original transitions remain in private journal SHA256 3bb1ff4f0a4bd3b7e283ec98fd02ad4afdeaa90ca72d490eaa280eca899e99c1. Source artifact SHA256 30c3962364be2b7f25b65a3fdc24c33da1141f28e7a52f7b965cbb3aa5680d28, retained as local-evidence:runtime-gaps/publication-replay/snapshots/30c3962364be2b7f25b65a3fdc24c33da1141f28e7a52f7b965cbb3aa5680d28.md. Source creation recorded at 2026-09-11T00:20:57Z. Private labels and local paths are projected to descriptive aliases.
<!-- public-import-provenance:end -->
