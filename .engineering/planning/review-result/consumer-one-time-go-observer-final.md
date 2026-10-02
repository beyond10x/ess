---
format: aep.planning-md/3
id: review-result:consumer-one-time-go-observer-final
kind: review-result
status: active
title: Independent Go observer correction review
relations:
- reviews: story:feature-request-389
revision: 1
---
approve

```findings
[]
```

Independent read-only rereview of ess-backlog-go-parity-20261002 frozen final patch go-one-time-execution.patch SHA25654860770ec725bcb7c31357752a3f655c3ec60ef65a949bb14a06a0a113337b5, against baseccf869b87a47409c0ce35b6db590c692b500fa82. All six source hashes match go-one-time-execution-sha256.txt; final report matches93643ff189949661d2a5947b109bf9418124d196a88add1b290b3a6930327943. Own test/build executions:0. No source or planning edits.

The previous successful-identity blocker is corrected at runtime.go:1770. The actual Identity callback still executes, but every protected run now replaces both successful and failed metadata with the agreed static name and empty version before any report is built. Unmarked identity failure and metadata behavior retain their original branch. The additive twentieth shared IdentitySuccess mode returns the issued plaintext in both identity fields; the author records its actual red disclosure followed by green. The test checks all captured plaintext against report and diagnostic bytes, rather than only comparing the status.

The previously noted adapter boundary is also corrected at one_time_execution.go:288: every non-nil ErrorPayload map is scanned regardless of its Error name. The author independently reproduced the original hole with two actual healthy Service commands and an adapter that copies the first returned value into the second payload while leaving the name empty. runtime_parity_go_28_35.rs:387 retains that named Go-only control, separate from the shared native fixture claims. Its real response copy avoids relying on an unrelated outcome/shape failure. The new guard covers unknown keys and values through the same bounded observation path.

Comparison with the earlier checkpoint patch confirms these are the production corrections; added test/support coverage accounts for the remaining reviewed delta. The final shared inventory includes20 protocol modes,13 exact resource boundaries,4 multi-field modes,4 actual original-byte coverage35 modes and2 shared-operation window modes. The coverage input retains its admitted parent chain; the multiple-window healthy trace distinguishes shared advancing-clock accounting from restarting each event's polling duration. The published test report distinguishes the43 native-pinned shared cases from the extra adapter-only fault and does not claim this worker executed the native producer.

Author evidence is final identical-test-byte baseline14passed/9failed followed by23passed/0failed/0ignored and strict scoped Clippy/format checks. Those counts are not my executions. The initial review's remaining source observations on private capture, shape/constraint validation, same-invocation and prior-value checks, independent logs before deduplication, measured completion, canonical resource bounds and precise status joins continue to apply. No additional concrete defect was found in this corrected snapshot.

Approval is bounded to the Go serial observer patch and its corrected privacy boundaries. Source-generated inventory completeness, integrated packages/projections, other runtime execution and release gates remain the coordinator's work; they are not silently certified here.
