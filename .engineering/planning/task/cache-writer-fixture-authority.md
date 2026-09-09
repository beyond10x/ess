---
format: aep.planning-md/1
id: task:cache-writer-fixture-authority
kind: task
status: implemented
title: Isolate authority setup between concurrent cache test drivers
relations:
- derived_from: story:review-cache-origin
- serves: vision:O2
revision: 5
---
## Finding
Foundation run foundation-1788942852-970971, ESS attempt 7 at 113f5925, retained a failed two_real_writers_publish_one_complete_proof_without_replacement case. The losing driver reported AuthorityMismatch because recovery/etc/ess/recovery/registry.json was empty during parsing; the other driver timed out waiting for the second ORAS writer. cache_lane in crates/edge/ess-cli/tests/support/recovery_driver.rs:329 gives both processes the same arrangement; fake_recovery::publish_registry truncates its registry with std::fs::write at tests/support/fake_recovery.rs:315.

## Change and acceptance
Give each cache driver a private synthetic authority arrangement while preserving the shared OCI cache and the two real writer barrier. Both writers must still independently pass production authority and document admission. Keep original-byte proof, no-replacement, ORAS call-count and consumption assertions. No production format or runtime behavior changes. Pass the focused concurrency case, all cache-origin tests and task check before landing.

## Scope
Only the test driver arrangement in crates/edge/ess-cli/tests/support/recovery_driver.rs and this governed task record. This is one regression correction under the implemented cache-origin story, not a new decomposition.

## Verification
The focused cache-origin target passed all 24 tests, including both real concurrent writers. The complete task check subsequently exited zero with 2,561 workspace test passes and no failures or ignored tests; formatting, strict Clippy, rustdoc, examples, projections, source-support, consumer qualification, release-record consistency and action checks passed. All 22 required consumer cases have verified execution receipts. Consumer qualification retains its accepted unknowns and does not claim complete support. The session retains ess-cache-authority-full-gate.log and ess-cache-authority-consumer-summary.json. The focused result linked in the evidence journal completed at 16:05:05 UTC (the earlier evidence timestamp was rounded to the minute).

The fixture's original authority and barrier refusals are preserved in the Atlas foundation checkpoint. This correction does not complete the separate Atlas composition: its governed worker remains capacity-blocked, and no successful composition receipt is claimed here.
