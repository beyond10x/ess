---
format: aep.planning-md/3
id: story:a-not-equal-or-any-selector-beside-named-rows-is-witnessed
kind: story
status: draft
title: A not-equal or any selector beside named rows is witnessed
tags:
- ess-d368-close
relations:
- serves: vision:O2
revision: 1
---
## Finding

Found by wave D368 U4's adversary pass (`review-result:adversary-d368-u4-pass-1`), out of that story's Acceptance: a row-set selector with no equality conjunct (only `!=`, or `any:`) beside named rows refuses every branch. The refusal is honest, but such a command has no suite.

## Acceptance

- A command whose row-set selector has only `!=` conjuncts, or is `any:`, beside named rows is witnessed on its branches where an arrangement exists, or each branch is refused naming the selector.
- Committed models whose suites move are named.
