---
format: aep.planning-md/3
id: story:external-requests-are-assessed-before-adoption
kind: story
status: archived
title: An adopter request is assessed for fit before it is adopted
relations:
- serves: vision:O2
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-09-30T13:07:51Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-09-30T13:07:51Z", actor: "human:timo", revision: 3}
- {from: "active", to: "archived", at: "2026-10-08T09:55:13Z", actor: "human:timo", revision: 4}
---
## Outcome

A change an adopter asks ESS to make is assessed for fit before it is adopted: the underlying need is
separated from the requester's proposed syntax, checked against what ESS can already express and
against the constructs it will meet, and the decision (accept as proposed, accept redesigned,
decline with the idiom, defer) is recorded before a story is dispatched.

## Acceptance

- `.agents/skills/assessing-external-requests/SKILL.md` states the seven fit questions, the four
  decisions and the red flags; `AGENTS.md` points to it from "Where work is tracked".
- Every story under `epic:downstream-reported-gaps` carries a `## Fit review` and a `## Decisions`
  section before its wave is dispatched; wave 1 (257, 265, 271), dispatched before this story,
  gets them before it merges.
- The externally driven ESS changes of 2026-09-22 to 2026-09-30 are audited against the fit
  questions; every change judged `revise` or `deprecate` has its own story, and the report is
  cited here.

## Origin

Operator request, 2026-09-30: requests from downstream sessions were being turned into stories and
code largely as proposed, which risks inconsistencies in the authored language and formats.
