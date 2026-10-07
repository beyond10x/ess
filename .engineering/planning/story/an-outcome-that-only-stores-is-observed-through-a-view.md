---
format: aep.planning-md/3
id: story:an-outcome-that-only-stores-is-observed-through-a-view
kind: story
status: draft
title: An accepted outcome that stores a row and emits nothing is admitted when a declared view observes it
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#492
relations:
- serves: vision:O2
revision: 1
---
## Outcome

Gap 2 of https://github.com/beyond10x/ess/issues/492: an outcome with `creates:` or `updates:` and
no event and no error is admitted when a declared view of that entity can observe the change, and
synthesis checks it by reading that view. Today `ESS-COMMAND-007` refuses it as unobservable;
`accepts: nothing` (#144) declares a no-op, which this is not.

Spec first; fit review owed.
