---
format: aep.planning-md/1
id: story:timestamp-clock-provenance-contract
kind: story
status: active
title: Resolve clock provenance and timestamp crossing semantics before modeling them
tags:
- priority-high
relations:
- decomposes: task:ess-gaps-measured-in-a-consumer-specification
- serves: vision:O2
- informed_by: story:elapsed-time-claims
scope:
- confidence: cited
  path: CHANGELOG.md
- confidence: cited
  path: crates/generate/ess-gen/src/types.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/go/layout.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/go/reading.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/rust/layout.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/rust/reading.rs
- confidence: inferred
  path: crates/specify/ess-compiler/src/ir.rs
- confidence: inferred
  path: crates/specify/ess-compiler/src/resolve.rs
- confidence: cited
  path: crates/specify/ess-domain/src/expression.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/reading.rs
- confidence: cited
  path: crates/specify/ess-domain/src/types.rs
- confidence: cited
  path: crates/specify/ess-primitives/src/time.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/admission.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/go/runtime.go
- confidence: inferred
  path: crates/verify/ess-conformance/src/reading.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/report.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/runner.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/scenario.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/target.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/fixtures/clock-authority-adversary.go
- confidence: cited
  path: crates/verify/ess-diff/src/change.rs
- confidence: cited
  path: crates/verify/ess-diff/src/delta.rs
- confidence: cited
  path: crates/verify/ess-diff/src/diff.rs
- confidence: inferred
  path: docs/design/timestamp-clock-provenance.md
- confidence: cited
  path: website/docs/guides/verify-conformance.md
- confidence: cited
  path: website/docs/guides/write-a-specification.md
- confidence: cited
  path: website/docs/reference/formats.md
revision: 5
---
## Outcome and priority

High-priority design for gap 7: decide and document the clock-authority/representation contract from the four measured producer/consumer cases, then implement only the agreed typed capability. No Clock fields, identities, conversion rules or format numbers are invented by this story.

## Established boundaries

Authored Timestamp has no clock parameter (ess-domain/src/types.rs:44-60), and expression admission classifies it as text (expression.rs:25-36). The date-time schema annotation is not evidence of a producer timezone (ess-gen/src/types.rs:510). Clock-free epoch milliseconds in ess-primitives/src/time.rs and the elapsed target's deliberately unspecified clock kind (ess-conformance/src/target.rs:195-219,595-605) do not establish authored clock provenance.

## Design decisions and acceptance

This story closes only after a reviewed authority/representation design is implemented as the agreed typed capability and the four measured cases have projection/conformance evidence that distinguishes justified crossings from unknown or invalid claims; a design document alone is the first checkpoint, not completed gap delivery. If an authority decision remains unavailable, retain the named unknown and keep the story open rather than implementing an assumed clock.

Required verification:

- Inspect the actual producer formatter, boot timezone pin, parser, comparisons and cross-producer subtraction. Distinguish a physical/logical source, synchronization domain, process/session, and encoding; select identity only from the authority that can establish it.
- UNMAPPED: producer timezone before bootstrap, the consumer-visible evidence for the later pin, and the authority allowing cross-producer comparisons. Unknown must remain explicit and block instant/comparability claims requiring that fact.
- Decide whether each reading is an instant or a local civil reading. A producer-local string with a literal Z must not silently become known UTC.
- Before implementation, draft the smallest concrete typed model justified by those decisions, validate it through ESS, and review the binding design. Preserve legacy Timestamp/elapsed meaning or state the exact coordinated migration and old-reader refusal.
- Define permissible comparisons/conversions and what an authored reason proves versus what needs external evidence. Existing text comparison and nominal newtypes are not a clock-aware arithmetic engine.
- Projection and conformance evidence must preserve the admitted provenance boundary; consumer vectors cover the literal-Z trap, unknown pre-bootstrap state and each justified crossing. No false claim that duration observations already solve point-in-time authority.

Runtime synchronization, timezone databases, calendars/leap-second arithmetic and a general temporal query engine are not implicit scope. The concrete representation remains a design prerequisite, not an implementation-ready assumption.

## Historical source

Archived argument: story:a-timestamp-names-the-clock-it-was-read-from. Its original acceptance is evidence to assess, not a legal lifecycle path to reactivate.


## Provenance and delivery

Decomposes task:ess-gaps-measured-in-a-consumer-specification under initiative:ess-evolution. The operator prioritized these gaps on 2026-09-11. Source-only scoping at dcdc3343 is retained in local-evidence:ess-evolution-20260910/priority-wave/gap-scoping/time-size.md; original archived argument bytes and hashes are in gap-scoping/source/manifest.json. Original archived stories remain terminal and untouched; this is an explicitly distinct implementation/design unit, not an illegal unarchive. Planning does not claim implementation or a rerun of consumer measurements. Integrate overlapping CLI/compiler/conformance changes after the active accessor candidate; source overlap does not depend on finishing its separate downstream adoption proof. No full local workspace/ownership gate or unchanged reruns. Consumer-only PR pipeline failures remain accepted, with core feature checks retained.

## Scope

- Cited: crates/specify/ess-domain/src/types.rs.
- Cited: crates/specify/ess-domain/src/expression.rs.
- Cited: crates/specify/ess-primitives/src/time.rs.
- Cited: crates/generate/ess-gen/src/types.rs.
- Cited: crates/verify/ess-conformance/src/target.rs.
- Inferred: docs/design/timestamp-clock-provenance.md.
- Inferred: crates/specify/ess-compiler/src/ir.rs.
- Inferred: crates/specify/ess-compiler/src/resolve.rs.


## Approved bounded implementation

The operator requests concurrent implementation, not another design-only handback. Coordinator admits the reading-contract proposal in local-evidence:ess-evolution-20260910/priority-wave/clock-design/design-proposal.md: optional closed named-newtype attachment, three explicit encodings, observed process-instance/epoch/occurrence identity, and independently observed formatter offset. A trusted adapter supplies actual correlated facts; it never supplies expected normalized output or a comparison verdict. Same-observed-source/epoch coordinate normalization and ordering is supported; unknown/literal-Z/erased-origin and uncalibrated cross-source claims remain explicit refusals. At least the justified offset-text/epoch/local-fixed-offset positive vectors must execute in native Rust/Go and conformance. No all-refusal completion or invented global clock trust.

Implementor scope_literal owns wt-6b9e1be0a97b, branch impl/gap-clock-provenance, base32c765bc. Domain named-type attachment and clock-specific expression checks, compiler named-type metadata, schema annotations, native helper/layout and conformance reading-only branches are separated from periodic binding causes, list mapping and root's subject-state command guards. Minimal existing-syntax ESS model and binding design precede source edits. Exact ownership and returned shared hunks are required in the brief; aggregate import/format/projection wiring stays coordinator-owned. Conditional unreleased source3/suite6-7/target-failure3 is shared across this one PR, with report2 and exact old-reader refusals. Existing Timestamp/elapsed bytes remain unchanged.

<!-- public-import-provenance:begin -->
Final-state public import from the privately retained integration store, not an event-equivalent historical replay. New CLI creation/status changes occur at import time; original evidence retains its observed timestamp. Superseded bodies/scopes and original transitions remain in private journal SHA256 3bb1ff4f0a4bd3b7e283ec98fd02ad4afdeaa90ca72d490eaa280eca899e99c1. Source artifact SHA256 5b9a6459deb5390824ca701e7cd11ac473e88c88e3b690852803d6f3ae66976c, retained as local-evidence:runtime-gaps/publication-replay/snapshots/5b9a6459deb5390824ca701e7cd11ac473e88c88e3b690852803d6f3ae66976c.md. Source creation recorded at 2026-09-11T00:24:42Z. Private labels and local paths are projected to descriptive aliases.
<!-- public-import-provenance:end -->
