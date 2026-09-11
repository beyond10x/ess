---
format: aep.planning-md/1
id: review-result:go-selection-primitives-adversary-pass1
kind: review-result
status: active
title: Review Go selection primitive admission
relations:
- reviews: story:validate-go-selection-primitives
revision: 1
---
unit: Go selection primitive correction / independent review1
verdict: red — confirmed static finding
cases: no new executions; inspected coordinator's retained10-case red→green evidence
origin: n/a
wrote-outside-worktree: assigned selection-review1 report only
needs-coordinator: yes — immutable finding and bounded correction
<!-- public-import-provenance:begin -->
Final-state public import from the privately retained integration store, not an event-equivalent historical replay. New CLI creation/status changes occur at import time; original evidence retains its observed timestamp. Superseded bodies/scopes and original transitions remain in private journal SHA256 3bb1ff4f0a4bd3b7e283ec98fd02ad4afdeaa90ca72d490eaa280eca899e99c1. Source artifact SHA256 5ecba2f41499fcbd6370afe6167d7193efcde5dc588190c7c452e25f3d5b9434, retained as local-evidence:runtime-gaps/publication-replay/snapshots/5ecba2f41499fcbd6370afe6167d7193efcde5dc588190c7c452e25f3d5b9434.md. Source creation recorded at 2026-09-11T05:44:35Z. Private labels and local paths are projected to descriptive aliases. This is a disclosed immutable-review publication projection, not a new review. Original review body SHA256 3856f980f5c0cca2f7abefa86934b229f022b6d76e9783150a5c12051226aca3, retained privately as local-evidence:runtime-gaps/publication-replay/snapshots/3856f980f5c0cca2f7abefa86934b229f022b6d76e9783150a5c12051226aca3-body.md. Verdict, finding identifiers, outcomes and measured counts are retained; label/path redaction may change their literal spelling.
<!-- public-import-provenance:end -->

Reviewed only selectionObservation.validateValue's primitive branch and its callees, plus selection_primitive_go.rs and fixtures/selection-primitives.go in root wt-643e4be649fc. No source/test edits or builds by reviewer.

Finding: finite-number admission remains incomplete for Decimal. The patch routes canonical Decimal through lowercased primitive("decimal", value), whose decimal arm checks only whether asNumber succeeds. asNumber accepts math.NaN() and positive/negative infinity float64 values without checking finiteness. Thus validateValue("Decimal", math.NaN(), true,...) returns nil, as does a List<Decimal> containing a valid first value and a nonfinite later value. Binary64 explicitly refuses those values in the correction; Decimal does not. Rust Node's Number::new rejects all nonfinite numbers, so this also retains Rust/Go observation parity divergence. Go target events enter the run's in-memory event list and selection evaluation directly; no JSON decode first excludes nonfinite Go values.

This hole pre-existed in the shared primitive helper. It is in scope at the newly corrected selection validation seam: the patch now relies on that helper to validate every reached primitive, and the supplied fixture claims malformed later list members refuse. Its Decimal bad-value control is a string and does not cover nonfinite numeric values. The coordinator explicitly accepted this static finding and will add two mutations before correction; reviewer did not duplicate the existing successful suite.

Smallest correction: apply the finite numeric condition to both Decimal and Binary64 at this bounded selection seam, or fix shared decimal validation only after coordinating its broader consumers. Keep existing Integer interval/fraction checks, canonical UUID/Bytes grammar and String/Boolean shape checks. Existing case-name normalization itself is correct. No further finding is asserted.

Source witnesses: runtime.go validateValue around6376; accessorShapeAdmits around4768; primitive around2746, Decimal arm2771; asNumber2578; direct event ingestion1862; selection observation dispatch4870. Rust finite Number construction: ess-primitives/src/facts.rs163.

```findings
- file: crates/verify/ess-conformance/src/go/runtime.go
  line: 6376
  category: contract-drift
  severity: blocker
  verdict: CONFIRMED
  origin: pre-existing
  message: Decimal selection values still admit NaN and infinity through the lowercase primitive helper; malformed later list members can therefore pass.
```