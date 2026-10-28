---
format: aep.planning-md/3
id: decision-blocker:a-view-without-a-source
kind: decision-blocker
status: open
title: A view that answers values no row holds
summary: 'ADR amendment for #674; recommendation A, staged'
refs:
- provider: github
  reference: beyond10x/ess#674
relations:
- blocks: story:a-view-may-answer-values-no-row-holds
revision: 1
---
# A view that answers values no row holds (beyond10x/ess#674)

On 0.74.0 a constant `expr:` on a view with a source validates; a view without a source does not
(`ViewSpec.source` is required). Components already declare `settings:` with an optional fixed
`value:`; the ADR for guards reading a setting is accepted but not built. Any of the format-changing
options amends that ADR, and an ADR amendment is the operator's (orchestrator decision D134,
2026-10-28: "#650 (WebSocket half) and #674: both need an ADR amendment, which is Timo's. File a
decision-blocker for each ... Batches proceed without them; those units wait.").

## Options

| option | what it does | cost |
|---|---|---|
| A | Extend the settings ADR to views: a field may read `setting.<name>`, and `source:` becomes optional when every field is a literal or a setting; the first cut reads only settings with a fixed `value:` | next `ess/N`; optional source in the view IR across synthesis, interpreter, generators and `ess-http`; ADR amendment; size L (first cut about B) |
| B | Constants only: `source:` optional when every field is a literal | next `ess/N`, the same IR change without settings; size M |
| C | Decline with the idiom: a constant `expr:` on a sourced view, documented in `read-api-view-idioms.md` | docs only, size S; no limits endpoint, the constant repeats per row |

## Recommendation

A, staged (first cut: literals and settings with a fixed `value:`), because a limit already has a
typed home in `settings:`.

## What waits

`story:a-view-may-answer-values-no-row-holds`. Every batch proceeds without it.

Decider: operator
