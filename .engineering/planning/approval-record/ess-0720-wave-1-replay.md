---
format: aep.planning-md/3
id: approval-record:ess-0720-wave-1-replay
kind: approval-record
status: approved
title: The 0.72.0 tag waits for the adopter replay of the pushed head
summary: 'D115: adopters A and B are replayed on the pushed release head on their side; the tag waits for that result and the PR CI.'
tags:
- ess-0.72.0
- non-interactive
relations:
- decides: approval-record:ess-0720-wave-1
revision: 3
transitions:
- {from: "draft", to: "approved", at: "2026-10-27T19:01:56Z", actor: "human:timo", revision: 3, executor: "agent:b10x-chef-orchestrator-ess-0720"}
---
## Decision

Relayed by the orchestrator on 2026-10-27 (decision D115). The adopter replay that D102 requires
before each tag runs on the adopter's side, against the pushed head of the release branch: the
adopter corpus stays read-only there and no clone of it is handed over. Once the adversary pass
is in, the release branch `queue/ess-0.72.0` is pushed, and its head commit is sent to the
adopter review session, which replays adopters A and B on it. The tag of 0.72.0 waits for both
that replay result and the pull request's CI. This is the same order 0.71.0 followed.
