---
format: aep.planning-md/3
id: story:feature-request-194
kind: story
status: draft
title: a binding cannot invoke only when an Optional path is present (ESS-BINDING-015 leaves no way to say 'skip')
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#194
relations:
- serves: vision:O2
revision: 2
---
## Outcome

a binding cannot invoke only when an Optional path is present (ESS-BINDING-015 leaves no way to say 'skip') (beyond10x/ess#194).

## Status

Feature request, triaged 2026-09-29 as outside the 0.41 and 0.42 defect batches. Not scheduled; acceptance is written when it is.

## Reconciliation

Backlog reconciliation (coordinator, 2026-10-01).

- **Duplicate of #268 as decided:** a binding cause with `where:` over the event payload (`when: {event: E, where: <predicate>}`) expresses "invoke only when `order` is present". Close #194 when #268 ships; no separate work.
