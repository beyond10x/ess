---
format: aep.planning-md/3
id: story:feature-request-363
kind: story
status: draft
title: One aggregate observation contains independently filtered measures
refs:
- provider: github
  reference: beyond10x/ess#363
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
scope:
- confidence: inferred
  path: crates/generate/ess-gen
- confidence: inferred
  path: crates/generate/ess-synth
- confidence: cited
  path: crates/specify/ess-compiler/src/ir.rs
- confidence: inferred
  path: crates/specify/ess-compiler/src/resolve.rs
- confidence: cited
  path: crates/specify/ess-domain/src/view.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/admission.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/go
- confidence: cited
  path: crates/verify/ess-conformance/src/interpret/views.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/scenario.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/ts
- confidence: inferred
  path: crates/verify/ess-conformance/tests
- confidence: cited
  path: docs/design/conditional-aggregate-measures.md
revision: 4
---
## Outcome

One aggregate view row reports differently selected counts and sums from one query observation. The pinned bundle runbook places363 after the361/362 aggregate observation authority. No implementation is complete.

## Fit review

1. Need: issue363 requests a scorecard with total, done and active counts in one row. Separate filtered views work around individual numbers but do not express one coherent aggregate response. The issue proposes nested count where and sum field/where arguments; these are requester syntax, not an adopted grammar.
2. Class: a gap in one-observation expressibility. Current view.rs::RawAggregate and compiler ir.rs::ResolvedAggregate contain only function/input/skip_absent; interpret/views.rs::project shares every partition member between all measures.
3. Existing idiom: a separate outer-filtered view per measure. The retained combined pre-history CLI validated the existing aggregate-views fixture, exit0 (sourceSHA c07b99ec1abd920a03209ccdc5c24eeed90fdae1c9ebd2657c8cbd700f2f6583; logSHA1462d5af2061c881ae090d9fe3e0fb6e9e02f6407f6f0401216857c2db3129ed). Changing count to the requested nested where was refused with unknown field where, exit1 (sourceSHA30a211a1617c6fd04ce237d9cb8e45573f022844d6d70448f2cffda7081260b7; logSHA04602178e1d1c4e326ae3bc7d3780ab0e3741feafe2fef766ec78dd4b4a07ac1). This is old combined CLI admission evidence, not a new current-candidate or target run; current74a67d7cd source retains the missing field.
4. Fit: add one sibling where beside the existing function and skip_absent, reusing typed view-row/param predicates. Apply consistently to existing function siblings; preserve group formation, independent membership, exact empty/optional arithmetic and Unknown refusal. No command input or now in view predicates. The complete proposal is docs/design/conditional-aggregate-measures.md.
5. Second adopter: an inventory summary reports all stock, available stock and reserved value for each warehouse from one observation, using the same source row/parameter semantics.
6. Cost: source22 admission and optional typed aggregate predicate, omitted for old bytes; coordinated aggregate suite38/39 reader/program support, with expression40/41 composition where needed. No new source key outside aggregate, function alias, report count or production API. Owning schema/docs/projections and actual native/generated/browser execution must be updated; old-reader refusals remain mandatory.
7. Alternatives: keep separate views (does not describe one response); nested arguments (duplicates the existing function and field grammar); chosen sibling where (preserves current arguments and uniformly extends the existing aggregate map). No ratios, having clause, time bucketing, joins or state injection.

## Decisions

Accept the need, redesigned proposal pending independent design review: sibling aggregate.where, all existing function siblings, source22 fence, independent per-measure membership after outer grouping and before absent handling. The accepted runbook authorizes resolving this design; its detailed proposal is not yet approved or implemented. Keep source21 one-time responses, older bytes and explicit unsupported targets intact. An unsupported or skipped required target cannot satisfy closure.

## Acceptance

The design's named tests cover source/version admission; measure-only view parameters; independent groups/subsets; empty and Optional arithmetic; real membership transitions and causal binding effects; closed reader/source/IR authority; and native/generated Rust/Go/browser plus Go/TypeScript runner faults. Dropped/inverted/swapped predicates, whole-view filtering, dropped zero-selected groups, stale parameters/state, Unknown-as-false and rounded large Integers must fail. Required scopes, old-reader tests, schema/projection generation, strict lint and independent implementation review remain due. Design review precedes production dispatch;361/362 authority and applicable expression slices precede integration.
