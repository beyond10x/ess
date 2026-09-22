---
format: aep.planning-md/1
id: review-result:native-model-behavior-code-pass-1
kind: review-result
status: active
title: Native behavior diagnostic accepts stale model claims
relations:
- reviews: task:consumer-accounting-native-model-behavior
revision: 1
---
## First independent native model behavior code examination

Exact submission: 36c6842bbfdf3cd0acdce99ba9b9c24a223a5bbb; tree
9395e2d4ba8f3cc4371ffeb8e140d77fc7155cf5; parent a9b1fb015774b08a27a460b187cc05da0f25f324.
Fresh gpt-5.6-sol high reviewer returned NEEDS-CHANGE: one introduced blocker.
This is examination one of at most two for the native model behavior/accounting3 unit.

The complete original report remains unchanged at
local-evidence:ess-evolution/waves/0004-ess-accounting/native-model-review-pass-1/report.md,
SHA256 2b59a4d4788daa2630da50b94c6c91d8782417f122a99707a48d2d94269f56f2.
This coordinator-authored public index preserves its findings block verbatim. The original's
absolute workstation output manifest remains in local evidence under the approved orchestration
privacy boundary. This index is not a substitute for the original report or an approval.

## Measured failure and reachability

The reviewer added 42 lines in consumer_coverage/model_behavior_tests.rs. The unchanged test patch
is local-evidence:ess-evolution/waves/0004-ess-accounting/native-model-review-pass-1/test.patch,
SHA256 756375e998a9405f835d04bc4f9b946c71bfdaba4c4f794355072313f4128752.
candidate_rejects_a_stale_model_shape_claim and candidate_rejects_an_unknown_model_claim each
compiled and failed the intended assertion, actual exit101 (sessions5866 and13528).
The subsequent model_behavior_tests suite executed6 to8 cases:6passed/2failed, exit101/session82427.
Accounting_v3_tests passed6/6, exit0/session48844; the binary artifact boundary test passed1/1,
exit0/session17958. All reviewer processes are terminal and its lease/build token was released.

As reported, model_behavior.rs:333 receives no current model inventory. The diagnostic caller in
mod.rs:451 constructs this candidate, executes its cases, then counts its claims at mod.rs:494.
An authority mistake or later model source change therefore reaches the missing freshness guard.
The full accounting plan_v3 path separately refuses mismatches against its current model map.
This finding is diagnostic stale-authority acceptance, not a measured final accounting bypass.
The reviewer compared all27 currently adopted model/shape pairs against retained fresh Rust
inventory and found no present mismatch. No authority data change follows from this finding.

Root routes the introduced blocker to the same implementor for bounded correction. No fixed
outcome is recorded before that correction is observed. Full parent accounting and gates remain owed.

```findings
- file: crates/edge/ess-xtask/src/consumer_coverage/model_behavior.rs
  line: 333
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: model-behavior candidate construction accepts unknown or stale model/shape claim identities, so the consumer-behavior diagnostic can execute and count claims whose authority no longer matches the current extracted model inventory
```
