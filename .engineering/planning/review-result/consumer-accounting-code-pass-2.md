---
format: aep.planning-md/2
id: review-result:consumer-accounting-code-pass-2
kind: review-result
status: active
title: Consumer accounting code review pass 2
relations:
- reviews: task:consumer-accounting-v2-mechanism
revision: 1
---
# Second accounting mechanism code examination

This is the coordinator's public index of the complete retained reviewer report, not a rewritten
claim that the report used sanitized command arguments. The full original report includes local
workstation paths and remains in the local initiative evidence, SHA256
ea9f8ffee3e38079c43e2d4e50b026bf05c4f2a13aa21f022d2886420fc81d4c.
Submission:4aea87eec96a41ef4bca9160e13415d3be50f5d3; mechanismbase41da2281e99402602c25d8faf219fc75b1954d04.
Verdict:NEEDS-CHANGE; one introduced blocker. Exact new case executed1/failed1/exit101 before
complete affected suite218executed/217passed/1failed,three pre-existing ignored,exit101.
Only test file changed,46insertions; patchSHA256
311fdbe643a36acc9eb1e1102e3af5684391ef9abb19afe7afa0417fff954d3b.
ExactlogSHA256c18bcb7ab4b1b8e69f5649587e3e2599570a45ac0de8c15df7c371f0f7262e4d;
suitelogSHA2564bd94a8988408c6519e33a8fac0cd1e517c8ff83d23cfe22927530592276c983.
Both first-pass regressions pass. Finding originated in the original mechanism commit84d8c5f6,
not the first correction. No aggregate authority is adopted, so the normal gate refuses before
this path today; production uses this validator and receipt when authority becomes available.
All sessions terminal; reviewer released its lease and token. No third examination is opened.

The following machine findings are verbatim from the complete reviewer report:

```findings
- file: crates/edge/ess-xtask/src/consumer_coverage/aggregate.rs
  line: 566
  category: contract-drift
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: the persisted aggregate proof omits the exact reviewed manifest digest and v2 qualification accepts it by format and closure count, so distinct aggregate authorities can produce interchangeable qualified evidence.
```
