---
format: aep.planning-md/3
id: story:synthesis-leaves-unread-optional-inputs-absent
kind: story
status: draft
title: Synthesis fills every Optional input, with a related input's literal, so an unread optional deadline is set on every scenario
summary: Leave an Optional input absent unless the scenario's arrangement reads it
owner: timo
tags:
- adopter-request
refs:
- provider: github
  reference: beyond10x/ess#546
revision: 1
---
## Problem

https://github.com/beyond10x/ess/issues/546. Synthesis fills every `Optional` input on every synthesized step, and uses the same literal as a related `Timestamp` input. An adopter that adds an optional deadline (`expires_at: Optional<Timestamp>`) beside a required queue time (`queued_at`) gets `expires_at == queued_at` on all 108 synthesized submit steps (`2020-01-01T00:00:00Z` ×59, `…-02` ×20, `…-03` ×17, `…-04` ×12; 127 authored steps leave it absent). A conforming implementation then expires every synthesized item at its first sweep, and scenarios that never mention expiry fail by timing.

## Acceptance

- Synthesis leaves an `Optional` input absent on a step unless that scenario's arrangement needs it present: a guard, effect, invariant or later step reads it, or the branch under test requires it. Otherwise the input is omitted.
- Where it must be present, it is not the same literal as another input of the same type, unless a guard or relation requires equality.
- A minimal specification reproduces the old behaviour as a red test, and is green after.
- Existing synthesized suites in the repository change only where an optional input was filled without being read; the coordinator reviews that diff.
