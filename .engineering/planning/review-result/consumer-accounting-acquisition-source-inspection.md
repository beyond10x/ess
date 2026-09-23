---
format: aep.planning-md/1
id: review-result:consumer-accounting-acquisition-source-inspection
kind: review-result
status: active
title: Coordinator inspection of acquisition production source authority
relations:
- reviews: task:consumer-accounting-v2-mechanism
revision: 1
---
needs-revision
task:consumer-accounting-v2-mechanism — Acquisition authority binds the authored function signature but not its production body or module bytes; a freshly compiled provider can retain the reviewed hashes after a body-only default probe. Bind the actual production source to reviewed authority and prove refusal with a real source mutation. — crates/edge/ess-xtask/src/consumer_coverage/scenario_acquisition.rs:161

Coordinator inspection by Astra, not an independent implementation attack. No runtime failure is
claimed yet. Root read all eight proposed rows, all150lines of their direct owner tests, the
candidate/planning/qualification code, native source/binary freshness checks and the inventory's
actual hash function. Candidate and proposed authority rows compare equal; all eight profile/case
identities are unique and current profile hashes match. The proposed authority is NOT adopted.

consumer.rs:144-153 hashes function attributes, visibility and signature, excluding the body.
scenario_acquisition::candidates selects that declaration_sha256. Consumer profile fingerprints
also use declaration hashes. The two omitted owner cases assert an empty return, which a discarded
filesystem probe need not change. Native Session::authority checks current source against the same
freshly compiled provider; it does not compare production source against a reviewed fixed body.
Therefore mutating an authority row to an arbitrary different signature hash would test a mismatch
the real body mutation need not create. It is not the required causal default-probe control.

The accepted design already requires the exact acquisition source boundary, refusal if the route
gains model consumption, and causal default-probe sensitivity. Its concrete implementation must
retain signature/profile identity while also binding the actual authored source item and the
bounded production input_discovery.rs module bytes to root-reviewed authority. Do not globally
change existing declaration/profile hashes or extend frozen baseline eligibility. No filesystem
abstraction or product behavior change is required to fix source authorization.

Required evidence: an actual body-only discarded probe leaves the old signature unchanged; show
the old guard's sensitivity gap, then the corrected candidate/authority guard refuses that same
source mutation. Preserve the ordinary exact cases and fresh-source/native-image checks. Restore
source and rerun normal acquisition execution; no incomplete full-matrix run becomes accepted.

```findings
- file: crates/edge/ess-xtask/src/consumer_coverage/scenario_acquisition.rs
  line: 161
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: Acquisition authority binds the authored function signature but not its production body or module bytes; a freshly compiled provider can retain the reviewed hashes after a body-only default probe. Bind the actual production source to reviewed authority and prove refusal with a real source mutation.
```
