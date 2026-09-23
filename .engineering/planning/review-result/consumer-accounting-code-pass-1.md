---
format: aep.planning-md/1
id: review-result:consumer-accounting-code-pass-1
kind: review-result
status: active
title: First accounting code examination finds omitted profiles and refusal attribution loss
relations:
- reviews: task:consumer-accounting-v2-mechanism
- reviews: story:consumer-accounting-baseline-never-extended
revision: 1
---
## Recorded independent code examination

Submission: 84d8c5f6ffc1b99b10c58c793a395ade1255da64.
Reviewer: fresh gpt-5.6-sol high, first code examination.
Returned verdict: NEEDS-CHANGE; two introduced blockers; executed 215 to 217, two red.

The original complete report is retained unchanged at
local-evidence:ess-evolution/waves/0004-ess-accounting/review-pass-1/report.md,
SHA256 d229c609b44ea650b1e79491e433b339919b5ecd1293f195b2882f0fdda524e5.
This coordinator-authored public index retains the returned findings block verbatim. It is not
the complete report: exact workstation argv and the 100-path temporary-file manifest remain in
local evidence under the approved orchestration privacy boundary. Requests for the original
reviewer to produce a public companion were refused by current agent-thread capacity; no review
or test was repeated and no finding was dropped. The original report remains authoritative for
what the reviewer returned. No approval or integration is inferred.

## Actual test evidence

The reviewer added only 67 lines to consumer_coverage/accounting_v2_tests.rs. Exact test patch:
local-evidence:ess-evolution/waves/0004-ess-accounting/review-pass-1/test.patch,
SHA256 cc9f250ff67ca4bccf8d78f51695b4f50ca4dd08ddaa4d46a7213076b97bacdf.

The two exact cases ran before the affected suite and each exited101 with one executed failure:
acquisition_profile_inventory_refuses_a_ninth_scenario_acquisition_row (session55959) and
aggregate_refused_child_requires_the_exact_qualified_refusal_boundary (session68949).
Their complete direct logs and exit files are retained beside the report.

The final affected ess-xtask suite used --no-fail-fast and exited101 (session5989): 147 unit cases
executed with145passed/2failed, then70integration cases passed. Total217executed/215passed/2failed;
three pre-existing ignored cases remain unexecuted. Complete log SHA256
02ad7f9c039f8e176e18b85289ba8816b9e9c597a2e19fc3d3913671acfcc0e8.
Earlier suite attempts remain context: exported CARGO_TARGET_DIR caused one invocation-authority
refusal; after removing it, normal fail-fast stopped before integration. Neither adds a finding.
All sessions are terminal and the reviewer's lease/build token were released.

## Reachability and scope

The profile path removes every non-model profile from the matrix, then profile_identities selects
only its expected eight rows. A ninth source row classified ScenarioAcquisition can therefore
escape both inventories. The aggregate path reduces Refused to kind/cases and loses the exact
named refusal; an authority can substitute another nonempty boundary before validation.

The original report's sentence saying both modules were absent at base is imprecise: enforce.rs
already existed. The reviewed aggregate_cells path is new in this unit. This public index does
not rewrite that original source claim. The current lack of adopted aggregate authority is the
explicit parent boundary; it does not discharge this mechanism's exact-child requirement.

Both failures must be corrected while retaining the independent assertions, frozen baseline,
closed authority formats and full parent accounting obligations. No correction is recorded yet.

```findings
- file: crates/edge/ess-xtask/src/consumer_coverage/scenario_acquisition.rs
  line: 271
  category: boundary
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: profile_identities selects the expected eight acquisition profiles without refusing an additional ScenarioAcquisition profile, allowing the extra row to disappear from both model accounting and mandatory acquisition execution.
- file: crates/edge/ess-xtask/src/consumer_coverage/enforce.rs
  line: 927
  category: contract-drift
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: aggregate_cells drops a Refused child's named boundary before aggregate validation, so an authority frontier can attribute a different nonempty refusal to the qualified current cell.
```
