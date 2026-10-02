---
format: aep.planning-md/3
id: review-result:adversary-gaps-306-pass-1
kind: review-result
status: active
title: Adversary pass 1, downstream gaps unit feature-request-306
relations:
- reviews: story:feature-request-306
revision: 1
---
unit: story:feature-request-306, tree gaps-306
verdict: NEEDS-CHANGE, red 2 (introduced 3)

- blocker: rebind accepted a copied ledger without checking the new root's files, so a copied .ess-output replaced and retired authored files.
- warning: same path with a different directory identity named a recover command that refused again.
- note: every checkout hand-off rewrote committed state.json.

Coordinator decision: a transaction copied with its root cannot be recovered at the same path; pass-1 case amended to require refusal, no recover hint and no deleted evidence.
Correction 1: digest check before relocating; in-memory binding; same-path refusal names adopt; Fixture Drop.
