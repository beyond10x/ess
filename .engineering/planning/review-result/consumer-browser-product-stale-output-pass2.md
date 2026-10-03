---
format: aep.planning-md/3
id: review-result:consumer-browser-product-stale-output-pass2
kind: review-result
status: active
title: 'Actual Firefox review: completed output survives coverage selection'
relations:
- reviews: story:browser-response-conformance
revision: 1
---
needs-revision

Bounded preliminary review, not final product approval. Root independently executed actual Firefox against a private copy of the frozen coverage response product and its exact healthy emitted-host WASM. The original player SHA256 is bceead4c48731cdb49c4899b4d857ec1e61fe667070ae539725285bfe4dda4a6 and WASM is 17a25e7e203c30fea6d1e414a88a36e4ef285e1b3fbe2725d9cf2841be7a747b. No compiler, worker-owned source mutation or worker cache was involved.

Actual sequence: Connect, Run to completion, click Select, wait for admitted selection, click Restore and wait for re-admission. Selection changes the provenance panel's selected original/parent identity, but the old results text and the same two downloadable Blob URLs remain mounted. The runtime panel says "Coverage selection admitted. Not executed."; Restore says "Rust admission complete. Not executed." while retaining the old output. The original run identity is no longer named in that runtime status. This is a completed-result association defect, distinct from the pending test of an in-flight stale completion. The original downloaded bytes were not fabricated or modified.

The named BROWSER_STALE_COMPLETED_OUTPUT probe exits 1. Private evidence in ess-browser-stale-result-review-20261003 retains probe.mjs SHA256 25609102e2e34c9b9aaf31ac82c695511427e7919304b8571d4a27d460fb3891, log 793ddfeaa2fc4f177f05d53cbe361ad9716df0d8130abaac5110876e41d6b265, DOM 90e2891e3dd87b4f2202604f8a119f63c7bfb307427f2b24e7966e1e9abafc2e, BiDi de94a0d726577be6c1e8eddc8ad46d8133553f5b964404b85d14b810ab902b64, exact original site and host, HTTP request receipts and owned-process lifecycle. Firefox terminated with SIGTERM and its recorded PID was verified absent. The worker received these receipts; no correction is yet claimed.

The existing owner will correct the association within the already accepted browser scope and add a persistent Rust-orchestrated actual-browser regression. Clear obsolete results/downloads when replacing execution selection, or retain them only with explicit immutable original-run identity. Root does not authorize weakening current selection, coverage or stale-completion acceptance, a separate PR or any publication. Full feature and final independent review remain pending.

```findings
[
{"file":"crates/verify/ess-conformance/assets/browser-player.js","line":165,"category":"boundary","severity":"blocker","verdict":"NEEDS-CHANGE","origin":"introduced","message":"setDisplay/loadOriginal and coverage selection replace current execution identity while leaving resultDisplay and old download Blob URLs attached. Actual Firefox Run -> Select changes selected provenance yet retains identical old result and links with runtime status Not executed; Restore retains them too. Clear obsolete output/downloads on execution capability replacement or label an immutable retained prior-run record with its original identity. Add persistent actual-browser completed-run -> selection/restore coverage and preserve the separate pending in-flight stale-completion requirement. Root deciding red and exact source/WASM/DOM/BiDi hashes are in the review body."}
]
```
