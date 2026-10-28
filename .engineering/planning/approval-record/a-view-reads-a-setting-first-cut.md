---
format: aep.planning-md/3
id: approval-record:a-view-reads-a-setting-first-cut
kind: approval-record
status: approved
title: A view may read a fixed setting and need no source (operator, 2026-10-28)
summary: 'Option A, staged, for #674; first cut reads only settings with a fixed value'
refs:
- provider: github
  reference: beyond10x/ess#674
relations:
- decides: decision-blocker:a-view-without-a-source
- decides: story:a-view-may-answer-values-no-row-holds
- informed_by: architecture-decision-record:a-command-guard-reads-a-component-setting
revision: 2
transitions:
- {from: "draft", to: "approved", at: "2026-10-28T10:54:28Z", actor: "human:timo", revision: 2, executor: "agent:ess-issues-lead", correlation: "ess-issues-1"}
---
## Decision

The operator, 2026-10-28, relayed verbatim by the orchestrator, on
`decision-blocker:a-view-without-a-source` (beyond10x/ess#674): "as you recommend."

That is option A, staged: the accepted ADR `a-command-guard-reads-a-component-setting` is extended
to views (Amendment 1, accepted with this quote). A view field may read `setting.<name>`, and
`source:` becomes optional when every field is a literal or a setting. The first cut reads only
settings with a fixed `value:`.

## Effect

- The decision-blocker is cleared.
- `story:a-view-may-answer-values-no-row-holds` is buildable, first cut only, in the format-moving
  batch with the #650 exchange work (one `ess/N` bump for both if they land together).

Decider: operator
