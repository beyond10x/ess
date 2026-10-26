---
format: aep.planning-md/3
id: story:an-integer-equality-guard-pair-is-exhaustive
kind: story
status: draft
title: An integer equality guard and its negation are exhaustive
summary: 'ESS-COMMAND-005: guards == 0 and != 0 over one integer are not proved exhaustive.'
tags:
- adopter-report
- ess-0.71.0
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 1
---
## Problem

Adopter report, 2026-10-26, ess 0.69.2, from a downstream repository specification (which quarantines 9 scenarios for gaps 1 and 2).

ESS-COMMAND-005: two branches guarded `n == 0` and `n != 0` over one integer are not proved exhaustive, so the command is refused although every value takes exactly one branch. Related: story:defined-and-not-defined-branches-are-exhaustive (the same refusal for `defined` / `not defined`).

Not yet reproduced: no specification came with this item.

## Acceptance

- `an_integer_equality_and_its_negation_are_exhaustive`: `== k` beside `!= k` over one integer validates for any literal k; `== 0` beside `> 0` alone still draws ESS-COMMAND-005.
