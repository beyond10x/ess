---
format: aep.planning-md/3
id: review-result:consumer-one-time-typescript-observer-pass1
kind: review-result
status: active
title: TypeScript observer aggregate callback review
relations:
- reviews: story:feature-request-389
revision: 1
---
needs-revision
unit: story:feature-request-389 — TypeScript private observer
verdict: needs-revision
cases: executed 0→0, red 0; source review only
origin: introduced
wrote-outside-worktree: none
needs-coordinator: yes — aggregate event-observation correction and a measured regression

Reviewed commit c0e02bdeaa8b1f703dfbe2e3ebdcfc7cee79f1de against 940397c6b7466f098c0667d4868791c48f167998. The frozen seven-path patch SHA-256 is 3613cbf9f892becea18cb98e30dd90b3d4476d64b0d768e56c541a2b82ee07a7; all seven source hashes in ts-one-time-observer-source-sha256.txt matched. Candidate diff: 7 files, 1000 insertions, 49 deletions. Reviewer production/test diff: none. This scratch report is the only reviewer-created file.

One blocking source counterexample remains. In crates/verify/ess-conformance/src/ts/runtime.ts:3708–3710, logCount passes each observed event to remember separately. In the same file:3800–3803, eventuallyEvent does likewise. remember at :3604 wraps only one payload in disclosureMaps, which resets the JSON byte/member/depth budget for each event. The new dedicated disclosureEvents path at :3302 correctly scans the complete payload array, but it does not cover an earlier ordinary event callback whose answer differs from later policy scans.

Concrete reproduction: use an admitted protected trace containing eventually_event after its healthy marked origin. Return two harmless event payloads of approximately 600 KiB each from that ordinary event callback, with a matching requested event. Return empty or small harmless arrays on subsequent dedicated/final log observations. Each singleton scan is within 1 MiB and the ordinary event expectation succeeds, so this path can report Passed despite observing a batch larger than 1 MiB. The accepted exact resource contract requires Unsupported for the entire aggregate array and explicitly prohibits resetting the bound for each event (canonical feature-request-389, “Exact private observation resource limits”). logCount has the same omission; its before-send path can additionally continue into a command after remember detects a per-item refusal because it does not propagate the refusal immediately.

This is introduced by the new observer integration: the older event helpers had no disclosure budget obligation. Add a complete-array check immediately after each successful ordinary ObserveEvents callback and propagate its false result before remembering, counting, issuing another target callback or asserting. Retain a real callback regression where only the ordinary callback returns the oversized batch, so a later dedicated scan cannot accidentally supply the refusal.

I inspected the complete seven-path patch, its test drivers and Rust host dispatch, and the identity, error payload, resource writer, multiple-window, resolution/error continuation, suite-ID and coverage binding call sites. No additional concrete finding is asserted. Successful protected identity is static after the actual identity callback; provided errorPayload is scanned independently of error name; the shared host executes the stateful Rust Service and sends actual callback answers rather than prerecorded reports. The declared 26 Rust parity tests, four runtime checks and 43 shared cases are author-retained results, not reviewer executions. I ran no builds, tests or target callbacks. This review does not establish source-generated feature inventory completeness or final package/gate success.

```findings
[
  {
    "file": "crates/verify/ess-conformance/src/ts/runtime.ts",
    "line": 3800,
    "category": "correctness",
    "severity": "blocker",
    "verdict": "NEEDS-CHANGE",
    "origin": "introduced",
    "message": "Ordinary eventuallyEvent and logCount observations scan each event separately through remember, resetting the aggregate JSON resource budget. A one-off batch of two harmless 600 KiB event payloads can pass when later dedicated scans are small. Scan and reject the complete payload array before remembering, counting, or continuing target callbacks; add a real ordinary-callback regression."
  }
]
```
