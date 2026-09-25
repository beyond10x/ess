---
format: aep.planning-md/2
id: review-result:aggregate-reference-design-pass-1
kind: review-result
status: active
title: Aggregate reference closure technical design review, pass 1
relations:
- reviews: task:consumer-accounting-aggregate-reference-closure
revision: 1
---
needs-revision
task:consumer-accounting-aggregate-reference-closure — The compatibility section must introduce a new accounting version or define version-dispatched historical accounting/2 and accounting/3 readers because replacing their embedded aggregate/1 authority and proof with aggregate/2 otherwise changes the accepted persisted value under unchanged outer format names — docs/design/consumer-aggregate-reference-closure.md:131
task:consumer-accounting-aggregate-reference-closure — The verified model token must own, borrow, or digest the exact validated execution payload so qualification can bind the accounting/3 `model_behavior_execution` value it persists rather than carrying only claims and receipt keys that can be paired with different raw evidence — docs/design/consumer-aggregate-reference-closure.md:114
task:consumer-accounting-aggregate-reference-closure — The aggregate/2 reader must specify a presence-sensitive representation or precheck for `old_target_shape` and `current_target_shape` because bare `Option<String>` fields do not distinguish omission from the explicit null required by the wire contract — docs/design/consumer-aggregate-reference-closure.md:32
task:consumer-accounting-aggregate-reference-closure — The target-proof decision table must make Stable, behavior, and Retired mutually exclusive by rejecting Supported, Refused, or AggregateClosure for an unchanged present target because the current one-way conditions otherwise admit multiple canonical proofs for the same edge — docs/design/consumer-aggregate-reference-closure.md:73
What I read: 25 artifacts (15 primary files and targeted portions of 10 local-evidence files) using `cat`, `nl -ba`, `sed -n`, `rg`, `git diff`, `git status`, `git rev-parse`, and `sha256sum`; this included the binding proposal at the supplied digest, accepted applicability/reconciliation/model-behavior designs, task record, aggregate/enforcement/model/native/caller readers, and both supplied reproduction command/log/exit triples.
What I could not establish: implementation correctness, gate status, or source acceptance; the brief prohibited builds and executable probes, and the supplied passing reproductions establish the current refusal boundaries only.
```findings
- file: docs/design/consumer-aggregate-reference-closure.md
  line: 131
  category: technical
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: The compatibility section must introduce a new accounting version or define version-dispatched historical accounting/2 and accounting/3 readers because replacing their embedded aggregate/1 authority and proof with aggregate/2 otherwise changes the accepted persisted value under unchanged outer format names
- file: docs/design/consumer-aggregate-reference-closure.md
  line: 114
  category: technical
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: The verified model token must own, borrow, or digest the exact validated execution payload so qualification can bind the accounting/3 `model_behavior_execution` value it persists rather than carrying only claims and receipt keys that can be paired with different raw evidence
- file: docs/design/consumer-aggregate-reference-closure.md
  line: 32
  category: technical
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: The aggregate/2 reader must specify a presence-sensitive representation or precheck for `old_target_shape` and `current_target_shape` because bare `Option<String>` fields do not distinguish omission from the explicit null required by the wire contract
- file: docs/design/consumer-aggregate-reference-closure.md
  line: 73
  category: technical
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: The target-proof decision table must make Stable, behavior, and Retired mutually exclusive by rejecting Supported, Refused, or AggregateClosure for an unchanged present target because the current one-way conditions otherwise admit multiple canonical proofs for the same edge
```
