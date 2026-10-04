---
format: aep.planning-md/3
id: story:explorer-restart-suite-step
kind: story
status: draft
title: Synthesized suites can restart the target and re-check minted identities
tags:
- follow-up
revision: 1
---
## Outcome

A synthesized conformance suite can restart the target process over its durable state and then check that identities minted after the restart do not collide with ones minted before, so suites, not only the explorer, catch a counter that resets on restart.

## Origin

Follow-up split from beyond10x/ess#297 on 2026-10-04. The bundle delivered explorer restarts (opt-in `restartEvery`, a counter-reset target fails in Go and TypeScript). The issue also asks for a synthesized "create, restart, create again" scenario. That needs a restart step in a new suite format, `ConformanceTarget::restart`, the step in the Go and TypeScript runtimes and report admission, and an opt-in so models with generated identities keep their synthesized bytes.
