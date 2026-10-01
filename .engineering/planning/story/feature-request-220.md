---
format: aep.planning-md/3
id: story:feature-request-220
kind: story
status: archived
title: 'the Go runner admits suites with unknown_instance: and deletes: steps'
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#220
relations:
- serves: vision:O2
revision: 3
transitions:
- {from: "draft", to: "archived", at: "2026-10-01T11:18:02Z", actor: "human:timo", revision: 3}
---
## Outcome

the Go runner admits suites with unknown_instance: and deletes: steps (beyond10x/ess#220).

## Status

Feature request, triaged 2026-09-29 as outside the 0.41 and 0.42 defect batches. Not scheduled; acceptance is written when it is.

## Reconciliation

Backlog reconciliation (coordinator, 2026-10-01).

- **Already fixed** by #188 (closed 2026-09-28): the Go runtime admits suites with `unknown_instance:` and `deletes:` (`go/runtime.go` `expectSubjectAbsent`; `tests/upsert_by_existence_go.rs`). Close #220 citing #188.
