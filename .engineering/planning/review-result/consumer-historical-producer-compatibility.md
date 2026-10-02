---
format: aep.planning-md/3
id: review-result:consumer-historical-producer-compatibility
kind: review-result
status: active
title: Historical producer compatibility and supplied-fact controls
relations:
- reviews: story:interpreted-trust-gate
revision: 1
---
approve

```findings
[]
```

Coordinator independent source review of coverage-rust-legacy.patch SHA256 b4b3df912d5b9ac95fbbfa088668d9883cdc502f7a4727156ea103aa688fc406, committed 7d29598a24c5f6a4117ed2b4e11c28cfd57cbdcf. Own test executions: 0. The author reports existing actual Rust coverage producer0/1 before,1/0 after with24 actual exports; new actual-target control0/1 before,1/0 after; dedicated strict Clippy and formatting exit0. Evidence is target/backlog-input/coverage-rust-legacy-report.md in managed ess-backlog-one-time-response-20261002.

The immutable old suite still describes target-owned issuance time. This test-only adapter logs the original request before inserting a missing issued_at for that historical IssueInvoice path, uses a bounded per-scenario sequence, and delegates all real execution and reads to Billing. Current production Billing still refuses the missing input. Controls prove two actual invoices produce descending, distinct timestamps, original callback inputs retain their missing field, and missing identity is not repaired. No suite, scenario identity, expected count or production fallback changed.

Reviewed native additive interpreted_obligations.rs commit20ee6382d9c54e0325d7e1c72bf7a0d092c5e869 separately. The three actual-target cases retain the existing normal branch when an external alternative is unscripted, require one escalation for scripted provider failure, and refuse an unavailable absolute clock fact. This review does not claim a baseline failure for that additive clarification test. Both commits are integrated in carrier01ce4b82f.
