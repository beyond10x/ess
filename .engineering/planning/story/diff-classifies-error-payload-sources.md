---
format: aep.planning-md/3
id: story:diff-classifies-error-payload-sources
kind: story
status: implemented
title: The diff classifies error payload sources per outcome
refs:
- provider: github
  reference: beyond10x/ess#253
relations:
- serves: vision:O2
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-09-30T01:33:23Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-09-30T01:33:23Z", actor: "human:timo", revision: 3}
- {from: "active", to: "implemented", at: "2026-10-06T09:42:49Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":2}}}
---
## Outcome

`ess verify diff` classifies an added, changed or removed error payload source per outcome (beyond10x/ess#253) instead of one opaque system-level change.

## Acceptance

- Adding `payload: {<Error>: {...}}` to a refusal outcome yields one `command/<command>/outcome-error-payload-added/<outcome>` change per outcome, classified additive; changing a source and removing one are classified too (removing: breaking for a consumer that reads the field? decide from the existing event-payload classification and mirror it).
- A specification with 19 such outcomes yields 19 classified changes and no unclassified change.
- Diff output formats and the change catalogue documentation name the new kinds.
