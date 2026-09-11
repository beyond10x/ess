---
format: aep.planning-md/1
id: story:validate-go-selection-primitives
kind: story
status: implemented
title: Validate typed primitive values in Go list selection
tags:
- priority-high
relations:
- serves: vision:O2
- informed_by: story:binding-list-selection-contract
- decomposes: task:runtime-adopter-gaps-9-13
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/go/runtime.go
- confidence: cited
  path: crates/verify/ess-conformance/tests/fixtures/selection-primitives.go
- confidence: cited
  path: crates/verify/ess-conformance/tests/selection_primitive_go.rs
revision: 5
---
## Observed defect

While testing typed response values, an implementor found that the Go selection observer passes canonical source primitive names such as Boolean and Integer into a lowercase primitive validator. Its unmatched-kind path returns success, so a text value can satisfy a Boolean declaration. This is a concrete defect in the existing list-selection implementation, independently of response payloads.

## Acceptance

Normalize the finite admitted primitive vocabulary at the selection validation seam and explicitly enforce finite Binary64 values. Verify valid values and wrong-kind mutations, including an invalid unselected list tail, through the embedded Go runtime. Preserve existing accessor shape admission and response-specific behavior. No full local gate. Root owns the bounded correction and records focused evidence; independent review remains required.

## Scope

Cited: crates/verify/ess-conformance/src/go/runtime.go, selectionObservation.validateValue and primitive. Inferred: a focused embedded-Go regression fixture and Rust test launcher under crates/verify/ess-conformance/tests.

## Completion evidence

The original ten primitive cases failed on invalid values before correction and passed afterward. The independent review found two further Decimal nonfinite cases; those failed before the bounded finite check and passed afterward. The reviewer confirmed the correction statically without another run. The final integrated Go wrapper passed all twelve primitive cases, including invalid later list members, in 0.70 seconds, and strict scoped Clippy passed. The combined response-mode branch keeps its separate exact-number and closed-value admission.

<!-- public-import-provenance:begin -->
Final-state public import from the privately retained integration store, not an event-equivalent historical replay. New CLI creation/status changes occur at import time; original evidence retains its observed timestamp. Superseded bodies/scopes and original transitions remain in private journal SHA256 3bb1ff4f0a4bd3b7e283ec98fd02ad4afdeaa90ca72d490eaa280eca899e99c1. Source artifact SHA256 a641836bb6446ec420774fbc5e86c0307eb6201b81d6202573b4cff9b399e692, retained as local-evidence:runtime-gaps/publication-replay/snapshots/a641836bb6446ec420774fbc5e86c0307eb6201b81d6202573b4cff9b399e692.md. Source creation recorded at 2026-09-11T05:38:34Z. Private labels and local paths are projected to descriptive aliases.
<!-- public-import-provenance:end -->
