---
format: aep.planning-md/2
id: review-result:s3-outside-boundary-design-pass-2
kind: review-result
status: active
title: Final finite boundary amendment design examination
relations:
- reviews: task:consumer-accounting-authored-boundaries
revision: 1
---
## Final independent amendment design examination

Existing S3 amendment design pass2 closed at handle1525, exit0. Actual reviewed design digest
6735d36921864fc858f6e3f0113309f794990c99a746610909a29aead6e6aace.
Full immutable worker report is retained in wave0010/s3-boundary-amendment-review-2.md.
Verdict approve subject to six scope-contract corrections, not a code or accounting acceptance.
Severity mapping for the store: reviewer major becomes warning, minor becomes note; the original
report remains authoritative for its original wording. The reviewer approved feasibility.
All prior blocking concerns were independently checked as corrected. This exhausts the two-pass
design budget; no third examination is scheduled.

```findings
- file: docs/design/consumer-outside-boundary-accounting.md
  line: 284
  category: acceptance
  severity: warning
  verdict: CONFIRMED
  origin: undecided
  message: Add aggregate reconciliation/2 binding; the new claim must match no closure or ExternalTargetProof.
- file: docs/design/consumer-outside-boundary-accounting.md
  line: 280
  category: acceptance
  severity: note
  verdict: CONFIRMED
  origin: undecided
  message: Name current plan_extraction resolve_v2 and plan_v5 and current plan reader call-site switches.
- file: docs/design/consumer-outside-boundary-accounting.md
  line: 283
  category: acceptance
  severity: warning
  verdict: CONFIRMED
  origin: undecided
  message: Preserve exact legacy/model partition and unique consumption through validate_model_partition_v5 and model_claim_matches_v5.
- file: docs/design/consumer-outside-boundary-accounting.md
  line: 283
  category: acceptance
  severity: note
  verdict: CONFIRMED
  origin: undecided
  message: OutsideConsumerBoundary must emit no aggregate-qualified row.
- file: docs/design/consumer-outside-boundary-accounting.md
  line: 320
  category: acceptance
  severity: note
  verdict: CONFIRMED
  origin: undecided
  message: Wrong-shape control fails at planning; use fresh shape with wrong package to observe G3 specifically.
- file: docs/design/consumer-outside-boundary-accounting.md
  line: 371
  category: documentation
  severity: note
  verdict: CONFIRMED
  origin: undecided
  message: Model has three commands/transitions; correct the two-moves prose and model comment.
```
