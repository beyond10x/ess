---
format: aep.planning-md/1
id: verification-report:normalization-main-reconciliation
kind: verification-report
status: draft
title: Preserve published composition and normalization planning history
relations:
- verifies: story:typescript-normalization-target
revision: 1
---
## Actual reconciliation

Integration commit 0c0f37a4921c42054cd3572dfd2c9a81c762f45e preserves published composition main 42a44c4324e0815ba622b8873a4c0eb5c050d7f0 and local normalization work through c930726f43cfaff3eb97e9546da670963777c413. The incoming 1353-line planning journal remains an exact byte prefix. Fifteen supported AEP CLI operations replayed 21 local semantic events with fresh agent:normalization-main-reconciliation-20260906 provenance. All five local artifact files match their frozen content/modes; 155 unrelated incoming files are unchanged. Final journal1374 lines, SHA2565267d829c4945a24a705bc2c5217b8161e5faa8f43b16b1728c00fc615ad5a24.

Validation before and after, graph and five artifact history/explain pairs all exited zero. The resulting store had160 artifacts and25 existing prose-only review warnings. Four historical section writes became explicit full-body commands preserving their exact intermediate/final content and revisions. No journal concatenation, old timestamp injection or manual artifact write occurred.

An initial verifier stopped after successful operation4 because it wrongly assumed every fresh event ID differs from the historical one. Creation IDs can be deterministic for identical semantic content: actual clock and actor were fresh. The six already appended events and their exact artifact state were verified before continuation at operation5; no command was repeated and no event was deleted or edited. The original failed verifier and all actual command exits are retained in ess-main-reconciliation/actual-replay and replay-verifier-correction.md.

The sole documentation conflict preserves incoming composition guidance followed by the TypeScript equality qualification in CHANGELOG. Automatic CLI/formats merges match the reviewed exact merged hashes. Incoming immutable composition review bodies preserve their existing terminal blank lines; those were the only git diff whitespace warnings relative to the local parent. This verification records reconciliation only, not the future complete integration gates.
