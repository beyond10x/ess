---
format: aep.planning-md/3
id: story:a-related-refusal-compares-the-rewritten-row
kind: story
status: draft
title: A related refusal reads back the stored fields of the row it rewrites
tags:
- ess-d368-close
relations:
- serves: vision:O2
revision: 1
---
## Finding

Found by wave D368 U3, out of that story's Acceptance: where a related refusal rewrites a row, `related_refused` compares the refusal but leaves the stored fields of the rewritten row uncompared (`refusal_policy.rs`).

## Acceptance

- The scenario reads back the rewritten row and compares its stored fields after the refusal, so a target that changes them on refusal fails.
- The models whose suites move are named.
