---
format: aep.planning-md/3
id: story:caller-sources-keep-the-interpreted-digest
kind: story
status: implemented
title: Caller sources make the interpreted target's spec_digest differ from its suite's
refs:
- provider: github
  reference: beyond10x/ess#216
relations:
- serves: vision:O2
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-09-29T01:19:57Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-09-29T01:19:58Z", actor: "human:timo", revision: 3}
- {from: "active", to: "implemented", at: "2026-09-29T06:23:43Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":1,"review_outcome":1,"verification":1}}}
---
## Outcome

`ess verify conform run --target interpreted` runs a specification that uses actor `attributes:` and `{caller: …}` payload sources: the suite it synthesizes and the model it interprets carry the same `spec_digest` (beyond10x/ess#216).

## Acceptance

- a regression model with an actor attribute, an event field filled from `{caller: …}` and a command using it runs under `--target interpreted` without the digest refusal;
- the two digest paths hash the same compiled model, and a test pins that a model without caller sources keeps its digest;
- the refusal still fires when the suite came from a different model.
