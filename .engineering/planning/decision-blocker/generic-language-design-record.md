---
format: aep.planning-md/3
id: decision-blocker:generic-language-design-record
kind: decision-blocker
status: cleared
title: Is the generic language design record accepted, and in which wave order?
relations:
- blocks: epic:generic-guard-precedence
- blocks: epic:generic-expression-language
- blocks: epic:generic-row-queries
- blocks: epic:generic-aggregates
revision: 2
transitions:
- {from: "open", to: "cleared", at: "2026-10-14T07:04:53Z", actor: "human:timo", revision: 2}
---
## Question

Is the design record `docs/design/generic-language-features.md` accepted as proposed, and in
which order do its waves run? It is binding once accepted, and no implementing wave of the four
features starts before that.

## Decided (the operator, 2026-10-14)

Accepted as proposed: G1 one typed expression language, G2 queries over rows, G3 aggregates over
expressions, G4 one guard-precedence order. After 0.60.0 ships, its waves run in this order, at
most five stories each: G4 with the policy, G1 values, G1 guards and time, G2, G3. The record's
open questions are answered inside the record by this repository's maintainers, before the wave
that depends on each answer.
