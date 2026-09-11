---
format: aep.planning-md/1
id: review-result:periodic-implementation-adversary-1
kind: review-result
status: active
title: Periodic implementation adversary, round 1
relations:
- reviews: story:periodic-binding-trigger-contract
revision: 1
---
needs-revision
<!-- public-import-provenance:begin -->
Final-state public import from the privately retained integration store, not an event-equivalent historical replay. New CLI creation/status changes occur at import time; original evidence retains its observed timestamp. Superseded bodies/scopes and original transitions remain in private journal SHA256 3bb1ff4f0a4bd3b7e283ec98fd02ad4afdeaa90ca72d490eaa280eca899e99c1. Source artifact SHA256 35fb6a9b82ac3a477f067c2f110036682d803fd97064f0ce838667ba766a6ea5, retained as local-evidence:runtime-gaps/publication-replay/snapshots/35fb6a9b82ac3a477f067c2f110036682d803fd97064f0ce838667ba766a6ea5.md. Source creation recorded at 2026-09-11T03:34:56Z. Private labels and local paths are projected to descriptive aliases. This is a disclosed immutable-review publication projection, not a new review. Original review body SHA256 2f3bd689585cb0c841e8f55658ad5e388e1ee9dcd874ca281454a4e211850b3d, retained privately as local-evidence:runtime-gaps/publication-replay/snapshots/2f3bd689585cb0c841e8f55658ad5e388e1ee9dcd874ca281454a4e211850b3d-body.md. Verdict, finding identifiers, outcomes and measured counts are retained; label/path redaction may change their literal spelling.
<!-- public-import-provenance:end -->

story:periodic-binding-trigger-contract — Periodic liveness must cover the complete observed live interval, because elapsed overshoot currently hides a missing due tick. — crates/verify/ess-conformance/src/periodic.rs:351

Measured: cargo test --offline -p ess-conformance --test periodic elapsed_overshoot_cannot_hide_a_missing_live_tick exited 101 with the assertion that a missing tick in the completely observed live interval must not pass. The controlled target processed four periods, advanced its actual clock through five periods without dispatching the fifth tick, reported that complete interval, then acknowledged stop there. The runner returned success. Retained log: local-evidence:priority-wave/periodic/adversary-overshoot-red.log.

Reachability: Observation accepts elapsed_ms and complete_through_ms beyond the requested hold; Ledger::observe checks liveness only through hold_ms. The public ConformanceTarget observation seam can reach this without bypassing admission. Generated Go duplicates the same requested-hold liveness boundary.

Reviewed: typed source/IR cause, bounded Check validation, runtime ledger scope, mapping, overlap, failure, cancellation and liveness; native host refusal and prior retained Rust/Go controlled tests. This is a bounded implementation review, not production consumer adoption or a full workspace gate.

```findings
- file: crates/verify/ess-conformance/src/periodic.rs
  line: 351
  category: correctness
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: Periodic liveness must cover the complete observed live interval, because elapsed overshoot currently hides a missing due tick.
```