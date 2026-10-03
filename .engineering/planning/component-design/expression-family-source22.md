---
format: aep.planning-md/3
id: component-design:expression-family-source22
kind: component-design
status: in_review
title: Resolved source-22 expression family and occurrence clock contract
relations:
- designs: story:feature-request-233
- designs: story:feature-request-225
- designs: story:feature-request-244
- designs: story:feature-request-237
- designs: story:feature-request-200
revision: 4
transitions:
- {from: "draft", to: "in_review", at: "2026-10-03T18:53:42Z", actor: "human:timo", revision: 2}
---
## Purpose

Bind the remaining source-22 expression family before implementation: typed bare facts, one constant offset, command-time stored/related predicates, dotted input value paths, list distinctness, String UTF-8 byte length, and typed text-match operands. The accepted bundle runbook owns execution and serial integration. This design preserves approved filtered-related row-set semantics and the calendar/time-zone and external Entity Runtime deferrals.

## Initial proposal and independent review

The initial author proposal is retained as local-evidence:ess21-completion/expression-family/design-proposal.md, SHA256 `8cf552090ae25760546ab9660b82466e20e1edae7ed799113239bd6d4c7cc05d`, based on source inspected at `d35eafecf8b4ec5ff26ced16de969d4435baa7ee`. It is not approved and no implementation authority is inferred from its prose. Independent review pass 1 inspected the same relevant source at `72167e08fb2f9c9703645dd0f5664c20a080ccb9` and found five blockers: mechanically resolved-only persistence, persisted UTF-8 selector identity, exact out-of-domain Integer intermediates, occurrence-clock authority across native/history execution, and decisive distinctness equality/finite-domain controls. Exact publication report SHA256 `613ccc16005b11068f7c20fed353709c5d7a74e655f4dd7ac4daf4b0ebff6e06`.

Root must revise all five before the second and final full design review. No compiler execution or feature acceptance is claimed. Suite40/41 is a proposed coordinated allocation after existing reservations36-39; source21 remains one-time responses. All new persisted vocabulary needs explicit old-reader refusal and unchanged older bytes.

## Coordinator revision for final review

The revised complete proposal is `docs/design/expression-family-source22.md`. Review1 findings are addressed at their stated seams: (1) a distinct non-serializable/non-evaluable lexical AST with resolved-only IR/suite/digest fields, (2) persisted typed UTF-8 selectors and same-name-field/direct-binding compatibility, (3) exact wider Integer intermediates with max/min fault controls, (4) an explicit per-command decision instant throughout native/generated/browser execution and versioned history2 observation, and (5) typed distinct-key equality plus Decimal/Timestamp/large-Integer and finite Boolean/enum controls in every required lane. The revision reserves suite40/41 and history2; old envelopes and unsupported external Entity Runtime lowering remain explicit. These are coordinator design choices pending the second and final independent design review, not verified implementation.

## Final design review and coordinator receipt correction

Independent design review2 at `159a21d7f` resolved four review1 findings but identified one remaining A3 blocker: no typed path carried the exact decision instant through a failed/indeterminate completion to the recorder. The exact report is preserved in `review-result:expression-family-source22-20261003-r2` (publication SHA256 `22f3ffc3f5e72ae2832fed45377712e789230d929a47a7528950c85fc70bd3e6`).

The coordinator correction in the design binds `RecordedCommandCompletion { answer, decision_time }`, compatible recorded target/interleaved methods with default None, Atomic forwarding once, per-Pending receipt authority and recorder persistence before the answer/error split. It specifies actual returned/indeterminate/out-of-order/retained-retry and corrupted-clock controls. Two full design-review passes are exhausted. This correction is not independently approved; the exact receipt seam must pass independent implementation review and actual execution before A3 or its dependent row-set time obligations close. Keep this design in_review pending that proof rather than claiming approval from the coordinator edit.
