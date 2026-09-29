---
format: aep.planning-md/3
id: story:overlapping-accepting-guards-have-declared-precedence
kind: story
status: implemented
title: Two overlapping accepting guards leave the answer for the overlap undeclared
refs:
- provider: github
  reference: beyond10x/ess#217
relations:
- serves: vision:O2
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-09-29T01:19:59Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-09-29T01:19:59Z", actor: "human:timo", revision: 3}
- {from: "active", to: "implemented", at: "2026-09-29T06:23:44Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":1,"review_outcome":2,"verification":1}}}
---
## Outcome

Which accepting branch answers an input that two accepting guarded branches both select is declared and tested: the first declared branch whose guard holds answers, documented in `docs/design/input-guard-overlap-precedence.md`, and synthesis witnesses each decidable overlap with an input inside it (beyond10x/ess#217).

## Acceptance

- the #217 shape (`small: amount < 100`, `flagged: amount > 50`) synthesizes a scenario sending an input in the overlap and expecting `small`;
- a target answering with the later branch fails it;
- the interpreter and Entity Runtime select the first declared branch (a test pins each);
- models without overlapping accepting guards keep their suite bytes.
