---
format: aep.planning-md/3
id: story:explorers-decide-input-guards-before-wrong-state
kind: story
status: active
title: The generated explorer checks wrong_state before input guards
refs:
- provider: github
  reference: beyond10x/ess#235
relations:
- serves: vision:O2
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-09-29T05:37:19Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-09-29T05:37:19Z", actor: "human:timo", revision: 3}
---
## Outcome

The generated Go and TypeScript explorers decide input guards before `wrong_state`, the order Entity Runtime and `docs/design/cross-record-and-stored-field-guards.md` use, and `mutation-audit-and-model-runner.md` states the same order (beyond10x/ess#235).

## Acceptance

- a command with an input refusal and a `wrong_state:` answers the refusal for a refused input in a wrong state, in the Go and the TypeScript explorer;
- the explorer order is the single precedence order the design doc states (see story an-input-refusal-sits-beside-a-subject-state-branch);
- the mutation-audit design doc states that order;
- generated explorer code regenerates, and `cargo xtask generate --check` passes on the committed projections.
