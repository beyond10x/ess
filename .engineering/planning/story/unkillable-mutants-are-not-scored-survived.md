---
format: aep.planning-md/3
id: story:unkillable-mutants-are-not-scored-survived
kind: story
status: implemented
title: A mutant with an unsatisfiable guard, or on an unwitnessed outcome, is scored survived
refs:
- provider: github
  reference: beyond10x/ess#218
relations:
- serves: vision:O2
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-09-29T01:20:00Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-09-29T01:20:01Z", actor: "human:timo", revision: 3}
- {from: "active", to: "implemented", at: "2026-09-29T06:23:47Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":1,"review_outcome":2,"verification":1}}}
---
## Outcome

`ess verify conform mutate` does not score as `survived` a mutant no scenario could kill: a mutant whose guard no input satisfies (a `guard-connective` turning `any: [x == A, x == B]` into `all:`) is `equivalent` with the dead guard named, and a mutant on an outcome the baseline suite does not witness (refused at synthesis) is `unwitnessed` naming the baseline refusal (beyond10x/ess#218).

## Acceptance

- both shapes are scored as above in `ess-mutation-report/2`, in `--target` and `--emit`/`--collect`;
- a mutant on a witnessed outcome that no scenario kills is still `survived`;
- the text output and the report schema name the new reasons.
