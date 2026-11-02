---
format: aep.planning-md/3
id: story:an-action-declaring-several-effects-runs-them-all-or-is-refused
kind: story
status: draft
title: An action declaring several effects runs them all or is refused
summary: React runAction runs one effect per action (W1 UI1 review, pre-existing, backlog)
tags:
- backlog
- review-finding
relations:
- informed_by: review-result:security-w1-ui1-pass-1
revision: 1
---
## Need

React's `runAction` runs one effect per action (an else-if chain), so an ordinary action declaring two
effects runs only the first (pre-existing at b81deac78, read by review-result:security-w1-ui1-pass-1). W1 refuses
the combinations on account-menu entries only.

## Acceptance

- An action outside the account menu that declares more than one effect is either run in full by both
  renderers or refused by `ess ui check`, naming the action.

## Schedule

Backlog (D398): not waved; considered only after approval and after every predefined wave is released.
