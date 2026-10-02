---
format: aep.planning-md/3
id: review-result:consumer-root-runtime-followups
kind: review-result
status: active
title: Independent review of runtime and tutorial follow-ups
relations:
- reviews: task:consumer-backlog-20261002
revision: 1
---
approve

Independent bounded source review of root-runtime-followups.patch SHA256 d9628864715cc6205942ded7594e7a40bbcf47c78b44bdd233291ead3860a713 in ess-backlog-next-20261002. Own executions:0. No builds or source/AEP changes. Reviewed the eight root-authored paths only: mutation-survivor.yaml, refusal_beside_state.rs, state_scoped_refusals.rs and five tutorial pages. The ninth path, assets/index.html, contains my own earlier one-line legacy-player correction and is explicitly excluded from this independent verdict.

No concrete finding in the reviewed scope.

The #201 update now requires both delivered/cancelled refusal scenarios to finish Passed, every check to pass, exactly two subject snapshots of demo.ship.Orders and an executed no-direct-events check. It strengthens the obsolete Unsupported expectation without changing generated scenario bodies or the existing discriminating hand-target mutants. The #204 interpreter negative at state_scoped_refusals.rs830–843 still requires Unsupported for the stored-field guard; its faithful/wrong-state-order and half-guard mutants remain unchanged pending actual implementation.

The too-short refusal update at refusal_beside_state.rs727–767 requires complete Passed execution, all checks Passed, an actual view check, subject-snapshot check and the named too-short outcome. It removes the old first-view Unsupported stopping point. Adjacent direct store/event invariance checks remain intact. This review does not claim these checks alone independently prove every native view capability.

The four mutation-survivor fixture lines explicitly supply and store IssueInvoice.issued_at using the migrated example API. They do not change the deliberate contact retarget survivor. mutate_cli.rs81–95 still requires exit1 and the named sets-retarget/billing.invoice.CreateInvoice/accepted/contact survivor; mutation assertions were not weakened.

The tutorial changes preserve their executable fence annotations and comparison harness. Source format21 matches the new source-language allocation; generated TypeScript module listing includes direct_response.ts and one_time_response.ts, matching ts/mod.rs286–291/624–625. The added optional deliverEvent declaration matches runtime.ts2007. Report/2 prose now distinguishes Unsupported from Error and strict non-success, without rewriting legacy report/1 semantics. CLI transcript scenario/refusal counts remain6/0, while file count changes13→15 and site index bytes11793→12262. Literal output-byte accuracy is supported by the author's reported tutorial execution, not an independent generation run in this review. Existing tutorial interface/exact-output assertion code is unchanged, including negative matching controls.

Author-provided evidence: native view lane29/0, actual Firefox coverage browser12/0, mutate CLI9/0 and tutorial8/0. Existing network-related npm skips remain a validation limit; no successful skipped network execution is claimed. Scoped two-test Clippy was reported in flight at handoff and is not certified complete by this source review. Approval is bounded to these source changes and does not replace the coordinator's final gates or the separate missing-interpreter-capability inventory.

```findings
[]
```
