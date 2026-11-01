---
format: aep.planning-md/3
id: story:an-authored-event-claim-names-a-struct-leaf
kind: story
status: draft
title: An authored event claim may name a struct leaf, and a struct literal claims only the leaves it writes
summary: Authored struct literals compare whole (runner same_value), dotted keys are refused ESS-AUTHOR-013.
tags:
- downstream
- ess-issues-3
relations:
- informed_by: story:nested-struct-per-leaf-comparison
revision: 1
---
## Problem

An authored scenario cannot claim one leaf of a struct in an event payload. Reported 2026-10-29
by an adopter on ESS 0.75.0. Checked on the 0.77.0 head (84671d6):

- An authored claim written as a struct literal, `payload: {status: {state: TASK_STATE_CANCELED}}`,
  compiles into the suite as the whole struct, `"payload":{"status":{"state":"TASK_STATE_CANCELED"}}`.
  The runner's `same_value` (`crates/verify/ess-conformance/src/runner.rs:3551`) compares a map
  member by member, so a member the literal leaves out (a `{generated: true}` timestamp the target
  stamps) reads as claimed absent, and the claim fails on a conforming target.
- Written with a dotted key, `status.state: TASK_STATE_CANCELED`, authoring refuses it:
  ESS-AUTHOR-013, "the payload of `a2a.tasks.TaskStatusUpdated` declares no `status.state`".
- The synthesized scenario for the same outcome writes the claim per leaf (`"status.state"` in
  `expect_event_values`) and is not affected.

Adjacent: `story:nested-struct-per-leaf-comparison` (implemented) compares a nested struct
target leaf by leaf. It does not cover an authored event claim.

## Acceptance

- An authored event payload may name a struct leaf by a dotted key, as the synthesized suite
  does, and the claim holds when the target carries other members of that struct.
- An authored struct literal claims only the leaves it writes: a member it leaves out is
  unclaimed, not claimed absent.
