---
format: aep.planning-md/3
id: approval-record:ess-0720-hold-for-gap-q
kind: approval-record
status: approved
title: Hold the 0.72.0 tag until gap Q is fixed and replayed
summary: 'D117: one synthesized swapped-caller refusal-policy scenario fails on the interpreter; fix, push, re-replay, then tag.'
tags:
- ess-0.72.0
- non-interactive
relations:
- decides: approval-record:ess-0720-wave-1-replay
revision: 2
transitions:
- {from: "draft", to: "approved", at: "2026-10-27T20:10:22Z", actor: "human:timo", revision: 2, executor: "agent:b10x-chef-orchestrator-ess-0720"}
---
## Decision

Relayed by the orchestrator on 2026-10-27 (decision D117). The adopter replay of 46a291a23 lost
no scenario in adopter A or adopter B. Adopter A moved from 1109/41 to 1114/34 and adopter B stayed
at 46/3. In adopter A, however, one scenario synthesized on this head fails on
`--target interpreted`. It is a binding refusal-policy cell under a swapped caller, and its second
pass counts the first pass's invocation. Decision: do not tag 0.72.0 yet.

The fix lands on the release branch:
- a neutral regression fixture built from the reduced reproduction (gap Q);
- the new head is pushed and sent to the adopter review session, which replays A, B and the
  reproduction again;
- the tag waits for that replay and for the pull request's CI.
