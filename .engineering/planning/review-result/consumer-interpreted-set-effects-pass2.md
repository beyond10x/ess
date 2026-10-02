---
format: aep.planning-md/3
id: review-result:consumer-interpreted-set-effects-pass2
kind: review-result
status: active
title: Set effects exclusion correction final independent review
relations:
- reviews: task:consumer-backlog-20261002
revision: 1
---
approve

Second and final bounded source review of ess-backlog-one-time-response-20261002/target/backlog-input/interpreted-set-effects-pass2.patch SHA256 eb077670d83fe82a7683c903f21f0128c3c1edbf748ad4c3088da22c4fbd81d1. All four source hashes and the patch hash match interpreted-set-effects-pass2-source-sha256.txt. Own test/build executions:0. No source, test, AEP or remote edits. Only this review report was written.

The original blocker is corrected. Same-entity primary identity is passed as an explicit exclusion into select; its iterator removes that row before binding stored facts or evaluating the predicate. Different entities receive no exclusion, preserving the existing same-key cross-entity control. Unknown facts on eligible secondary rows still return Undecidable; they are neither ignored nor treated as false.

The new an_excluded_primary_does_not_need_the_secondary_filter_facts regression compiles the actual missing-team source and first executes Invite with only the excluded primary. It asserts the named outcome, event and primary mutation. It then creates an eligible secondary lacking team, requires Undecidable and checks exact Store preservation. Author evidence records the first half red0/1, then22/0 across the ten dedicated and twelve existing tests, with strict selected Clippy and supported formatting passing. These are author executions; no independent execution is claimed. Existing eleven handwritten mutants, #288 controls, original snapshot/count/fallback/invariant coverage and cross-entity identity test remain intact.

The related from-state order is unchanged: the established design describes a selected row outside the transition's from states being skipped. This correction does not invent a new rule treating an unknown filter on that row as false. No additional concrete finding was found in this bounded rereview. Approval covers the frozen four-file candidate and resolves the first review's source finding; it does not certify unrelated Interpreter capabilities or replace final integration checks.

```findings
[]
```
