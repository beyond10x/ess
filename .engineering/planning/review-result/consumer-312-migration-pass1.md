---
format: aep.planning-md/3
id: review-result:consumer-312-migration-pass1
kind: review-result
status: active
title: Suite migration independent review requiring accessor correction
relations:
- reviews: story:feature-request-312
revision: 1
---
needs-revision

```findings
[
  {
    "file": "crates/verify/ess-conformance/tests/fixtures/accessor-runtime.go",
    "line": 140,
    "category": "test weakening",
    "severity": "blocker",
    "verdict": "NEEDS-CHANGE",
    "origin": "introduced",
    "message": "TestAccessorCoverageSevenLineage downgrades /35 to /5 but retains scenario_initial_state in the embedded suite and lineage parents. Go admission rejects this provenance at runtime.go:4001-4003 before checking accessor vocabulary or lineage. Its err != nil assertion therefore passes without establishing the legacy accessor refusal that the test claims. Build a valid metadata-free legacy envelope and assert the intended vocabulary/lineage refusal, preserving numeric tokens and the parent digest relationship where exercised."
  }
]
```

Reviewed frozen patch: target/backlog-input/312-migration-review-pass1.patch, SHA-256 e2ead419a978a0065833130516168ec789a4b6c5f380c1700da9faabcb6c5238, base da2dc12b5f, 93 files. Scope was count_json.rs and tests, including support_versions/mod.rs; the three dirty documentation files were excluded.

Reviewer test/build executions: 0. Read-only source, patch and retained evidence inspection; no source edits or cache use.

The production change adds only /34 and /35 to the existing direct-response parser profile. Payload-local depth starts only at values inside expect_direct_response.response.expected; nested payload keys cannot restart the budget. Duplicate-key detection, raw numeric token handling, envelope checks and the depth-128 limit remain unchanged. The retained red-to-green depth boundary evidence is consistent with that correction.

The Rust legacy helper removes only scenario_initial_state and relabels provenance for explicit historical negative cases. Existing feature-specific assertions for paging, accessor values, string operators, delivery context and outcome shape refusal remain. Retained Go downgrade preparation uses RawMessage, preserving large numeric tokens. The periodic Go helper uses generic JSON numbers only in its small-count fixture; it changes no production numeric handling.

Historical round-three golden files remain unchanged. Their original-byte admission assertions remain, while fresh suites compare complete scenarios and the same system/specification/spec/contract provenance before requiring initial-state metadata. One-time fixture changes add empty initial-state metadata; coverage suite-byte lineage digests are regenerated with their containing suite bytes. No other production rounding or canonicalization change was found.

Retained implementor evidence inspected: group-one baseline 138 passed / 21 failed then 159 / 0; group-two pass-one green except retained replay, followed by corrected periodic 7 / 0 and retained 36 / 0; direct payload depth boundary 1 / 0; one-time contract 7 / 0 and the six other fixture-producing targets green. Those executions are implementor evidence, not independent reviewer executions. Approval is withheld only for the accessor negative-test issue above.
