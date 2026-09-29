---
format: aep.planning-md/3
id: story:an-input-refusal-sits-beside-a-subject-state-branch
kind: story
status: implemented
title: An input-guarded refusal beside a subject-state branch is refused as unobservable_fact
refs:
- provider: github
  reference: beyond10x/ess#213
- provider: github
  reference: beyond10x/ess#227
relations:
- serves: vision:O2
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-09-29T01:19:56Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-09-29T01:19:57Z", actor: "human:timo", revision: 3}
- {from: "active", to: "implemented", at: "2026-09-29T09:43:50Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":1,"review_outcome":2,"verification":1}}}
---
## Outcome

An input-guarded refusal (`when:` + `error:`, naming no subject) may sit beside branches that act on an existing record through `when_subject_state:` or `when_subject`, and is answered before existence and before the held state, as the #209 precedence documents. Today `ess specify validate` refuses it as `unobservable_fact` (beyond10x/ess#213).

## Acceptance

- the #213 reproduction (`RotateSecret`: `too-short` beside `when_subject_state: Configured`) validates, compiles and synthesizes;
- synthesis witnesses the refusal with a plain send and on an arranged row in each state a sibling runs from, asserting the error and no event;
- a target answering the state before the input check fails the suite;
- committed suites regenerate byte-identical, or the change is listed in CHANGELOG.
