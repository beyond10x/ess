---
format: aep.planning-md/3
id: review-result:parameterized-transport-design-20261003-r1
kind: review-result
status: active
title: Independent review of parameterized transport design
relations:
- reviews: story:feature-request-391
revision: 1
---
needs-revision

story:feature-request-391 — intersection treats repeated or source-aliasing parameters as independent wildcards, so it falsely declares usage.{x}.{x} compatible with usage.a.b when a differs from b; unify exact-token constraints by resolved source path before claiming exact stream-language intersection — docs/design/parameterized-event-channel-addresses.md:183

story:feature-request-391 — the dynamic subject map has no retirement rule for empty buckets, allowing one retained key and timer state per historical subject indefinitely; require empty-bucket retirement and verify long sequences of unique subjects after flush — docs/design/parameterized-event-channel-addresses.md:245

What I read: the complete proposed transport design, AEP fit/acceptance, retained literal-template probe, current transport compiler and publisher emitters, and the official AsyncAPI3 parameter/address sections. This is independent coordinator source review of the worker-authored design, not executable candidate review. No new build or broker execution ran.

The first finding is a direct counterexample to the proposed exact algorithm: one repeated x cannot simultaneously equal distinct static tokens a and b. Distinct placeholder names mapped to the same event path impose the same equality. Global broker stream validation is a separate rule and does not make this claimed language intersection exact.

What I could not establish: actual concurrent batching behavior and NATS validation remain implementation evidence obligations. Neither finding authorizes widening payload source types or producer context.

```findings
[
  {
    "file": "docs/design/parameterized-event-channel-addresses.md",
    "line": 183,
    "category": "design",
    "severity": "blocker",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "intersection treats repeated or source-aliasing parameters as independent wildcards, so it falsely declares usage.{x}.{x} compatible with usage.a.b when a differs from b; unify exact-token constraints by resolved source path before claiming exact stream-language intersection"
  },
  {
    "file": "docs/design/parameterized-event-channel-addresses.md",
    "line": 245,
    "category": "design",
    "severity": "warning",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "the dynamic subject map has no retirement rule for empty buckets, allowing one retained key and timer state per historical subject indefinitely; require empty-bucket retirement and verify long sequences of unique subjects after flush"
  }
]
```

