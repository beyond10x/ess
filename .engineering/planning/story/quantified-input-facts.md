---
format: aep.planning-md/1
id: story:quantified-input-facts
kind: story
status: implemented
title: Project typed collection inputs for quantified conformance
relations:
- serves: vision:O2
- informed_by: story:ess-er-semantic-crosswalk
- informed_by: story:review-expression-typechecking
scope:
- confidence: cited
  path: crates/verify/ess-conformance
- confidence: inferred
  path: docs/design/ess-evolution/quantified-input-facts.md
- confidence: cited
  path: docs/design/ess-evolution/semantic-crosswalk.md
revision: 7
---
## Context

The ESS-to-ER crosswalk requires an independent compiled-input observer. At 568ee569, ess-conformance input.rs::project checks only outer List/Map shape and publishes no count or element facts, although ess-domain expression.rs admits count, list ordinals and value quantifiers. Bare FactStore quantifier probes do not prove compiled InputFacts support.

## Acceptance

A command compiled from ESS source projects supplied List and Map values into deterministic count and ordinal scalar facts, evaluates empty, absent, nested and scoped quantifiers with the existing three-valued semantics, and rejects malformed supplied elements before evaluation, without claiming support from the separate view-row observer.

## Design

Bind each observed collection's count, then recursively project list elements in sequence order and map values in ordered-key order. Map keys are not binder fields or direct selectors. Preserve the existing type-depth bound and optional omission/null semantics. Shared bind validates supplied collection elements for commands, payload assertions, error fields and rows; Partial still permits missing top-level fields only. Keep union and map-key codec limitations explicit. Separate input-path capabilities from existing conservative view-row capabilities. No persisted schema, scalar encoding, primitive evaluator or ER change.

## Scope

- crates/verify/ess-conformance — cited: input.rs binder, witness regression tests, and synthesis.rs view capability caller.
- docs/design/ess-evolution/quantified-input-facts.md — inferred: binding contract and limitations.
- docs/design/ess-evolution/semantic-crosswalk.md — cited: correct the unsupported pre-change InputFacts claim.

## Verification

Compile real ESS source fixtures; assert public flatten facts and actual predicate evaluation, including map value order, lexical/nested binders, empty versus missing, null elements, recursive structs/newtypes, malformed elements and depth refusal. Run affected conformance package tests and strict Clippy, plus a projection mutation. The operator forbids the full repository gate; no full gate or release claim is planned.

## Implementation evidence

The typed binder now publishes count plus recursive ordinal scalar facts through one collection helper. Map traversal uses ordered values; arbitrary keys are never fact paths. Input-path capability classification accepts legal count and List ordinal reads, while authored and synthesized view predicates retain the separate row projector's collection refusals. Missing collection counts are attributed to absent values. The semantic crosswalk corrects its earlier unsupported claim about compiled InputFacts.

Nine real-source integration tests cover compiled guards, counts, ordinal paths, map order/key collisions, nested binders and shadowing, outer/free references, empty and absent collections, null elements and Kleene dominance, accumulated shape errors, partial binding and recursive depth. The existing two collection-gap tests now expect actual input decisions; view refusal controls remain unchanged. Initial fixture errors (bare RHS words are literals; two unconditional outcomes are illegal) were corrected before counting semantic failures as evidence.

Final affected-package command `cargo test --locked --offline -p ess-conformance`: 310 passed, zero failed or ignored. Strict package Clippy on all targets and rustdoc with warnings denied passed. `cargo +1.85 check --locked --offline -p ess-conformance --all-targets` passed with existing compiler dependency dead-code warnings. Builds used a task-owned temporary target with debug info and incremental compilation disabled. No full repository gate ran, as the operator instructed.

Removing collection count publication caused `maps_bind_values_in_key_order_without_key_path_collisions` to fail: actual Unknown versus expected True. Restored source was byte-compared with its saved original, and the final package run passed after the shared traversal refactor. Logs: local-evidence:ess-evolution-20260910/quantified-inputs-package-final.log, quantified-inputs-clippy-pass.log, quantified-inputs-rustdoc.log, quantified-inputs-msrv.log and quantified-inputs-mutation.log.

Map-key codecs, union projection, Binary64 scalar facts and binder-body Unknown cause classification remain explicit existing limits. No ER lowering, service execution or adopter completion is claimed by this story. No release or full-gate evidence is claimed.
