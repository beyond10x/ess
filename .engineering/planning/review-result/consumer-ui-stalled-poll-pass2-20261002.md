---
format: aep.planning-md/3
id: review-result:consumer-ui-stalled-poll-pass2-20261002
kind: review-result
status: active
title: Review of stalled polling with a shared non-polling reader
relations:
- reviews: story:feature-request-365
revision: 1
---
## Outcome

One introduced cache-composition finding confirmed. Coordinator own executions0. The implementor's deterministic clock/interval regression in30-stalled-red.log observed one request instead of the required second request at the exact STALLED deadline with two subscribers, only one polling. Correction is in progress; this record claims no resolution or final package verdict.

```findings
- file: crates/ui/ess-ui-react/templates/runtime/data.ts.tmpl
  line: 472
  category: correctness
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: A stalled poller releases only its subscription while a non-polling consumer retains the same unresolved request. Its refresh rejoins that same in-flight promise, so usePoll never starts the replacement request promised at its STALLED deadline. A forced restart must preserve the other subscriber's cancellation authority.
```
