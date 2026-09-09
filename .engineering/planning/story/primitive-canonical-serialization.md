---
format: aep.planning-md/1
id: story:primitive-canonical-serialization
kind: story
status: draft
title: 'Canonical number serialization: the second stage of F08'
summary: Carry exact integers and decimals on the wire behind a format version, coordinated with the relying readers; stage one kept the bytes.
tags:
- P1
- review-2026-09-05
relations:
- decomposes: epic:review-boundary-remediation
- serves: vision:O2
- depends_on: story:review-primitive-semantics
scope:
- confidence: cited
  path: crates/generate/ess-gen/src/types.rs
- confidence: cited
  path: crates/generate/ess-synth/src/go/http.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/wire.rs
- confidence: cited
  path: crates/specify/ess-primitives/src/facts.rs
- confidence: cited
  path: crates/specify/ess-primitives/src/node.rs
- confidence: cited
  path: crates/verify/ess-conformance/assets/coverage-admission.js
- confidence: cited
  path: crates/verify/ess-conformance/src/go/runtime.go
- confidence: cited
  path: crates/verify/ess-conformance/src/input.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/report.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/witness.rs
revision: 11
---
# Canonical number serialization: the second stage of F08

## Finding and source

F08 (P1) from `docs/reviews/2026-09-05-architecture-review.md:326`. Stage one, story:review-primitive-semantics (wave 21), made fact numbers exact in-process and kept every persisted spelling, under the round-trip law its design page states (`docs/design/review-primitive-semantics.md`, "The round-trip law"): a `Number` is a fixed point of write∘read, so exactness beyond binary64 ends at the first write. review-result:primitive-semantics-adversary-wave21-pass2 F4 (pre-existing, `crates/specify/ess-primitives/src/node.rs:52`) measured the consequence: `9223372036854775296` and `9223372036854775807` are both declared-admissible `Integer`s and become one `Node::Number` on the read door, so an `expect_event` asserting one is satisfied by an implementation publishing the other. The acceptance statement's "without losing promised integer precision" does not hold for the `Integer` row until the wire carries the exact value.

## Acceptance

Two distinct declared `Integer`s (and two distinct `Decimal`s) never compare equal after a write and a read through any persisted ESS format, and every reader of those formats — the conformance runner, the Go runtime, the browser adapter, generated Rust and Go codecs, AEP's `aep-ess-evidence::adapt_json` — accepts the new spelling before any writer emits it by default.

## Implementation boundary

Reader and writer move together behind one format version (design page, "What stage two changes"): integers spelled as JSON integers, decimals as exact decimal strings where the schema already says so, binary64 unchanged. This is the migration obligation:review-contract-rollout-coordination names for F08: inventory the relying readers, the Atlas ADR, old/new compatibility with actual adapters, then the default writer in the approved order. Not a wave-21 unit.

## Scope

- `crates/specify/ess-primitives/src/facts.rs` (`Serialize for Number`), `node.rs` — cited.
- `crates/verify/ess-conformance/src/{input.rs,witness.rs,report.rs,go/runtime.go}`, `assets/coverage-admission.js` — cited by stage one.
- `crates/generate/ess-gen/src/types.rs`, `crates/generate/ess-synth/src/{rust/wire.rs,go/http.rs}` — cited by stage one.
- A format version for `ess-conformance-report/1`, suites and witnesses — inferred.
- Would collide with: any unit touching those files or the report/suite/witness formats — cited.
