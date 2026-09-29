---
format: aep.planning-md/3
id: story:a-counter-at-its-limit-is-witnessed
kind: story
status: active
title: A branch selected by a stored counter at its limit is never witnessed (ESS-SYNTH-003)
refs:
- provider: github
  reference: beyond10x/ess#226
relations:
- serves: vision:O2
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-09-29T01:20:03Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-09-29T01:20:03Z", actor: "human:timo", revision: 3}
---
## Outcome

A branch selected by a stored counter at its limit (`retries >= 3`, `attempts == max`) is witnessed: the stored-row search reaches the counter value by repeating the command that raises it, bounded by the guard's literal, and each side of the limit is witnessed. Where the limit lies beyond the search bound, the refusal names the bound (beyond10x/ess#226).

## Acceptance

- a model with a counter raised by one command and a `>= 3` refusal synthesizes both sides of the limit, and a target with the limit off by one fails;
- a limit beyond the bound is refused as ESS-SYNTH-003 naming the bound;
- committed suites regenerate byte-identical, or the change is listed in CHANGELOG.
