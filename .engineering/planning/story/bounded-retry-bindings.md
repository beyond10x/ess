---
format: aep.planning-md/2
id: story:bounded-retry-bindings
kind: story
status: draft
title: A binding can state an attempt bound and which failures are final
refs:
- provider: github
  reference: beyond10x/ess#165
relations:
- decomposes: epic:retrofit-findings-round-3
revision: 1
---
## Scope

Revisits `binding-delivery-guarantees.md` (no retry count) for bounds that are component behaviour: max attempts and final-failure classes on `on_failure:`.

## Acceptance

With a scripted receiver, the suite requires exactly N attempts on 5xx and one on a final 4xx.
