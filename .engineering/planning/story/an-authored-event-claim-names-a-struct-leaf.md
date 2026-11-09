---
format: aep.planning-md/3
id: story:an-authored-event-claim-names-a-struct-leaf
kind: story
status: archived
title: An authored event claim may name a struct leaf, and a struct literal claims only the leaves it writes
summary: Authored struct literals compare whole (runner same_value), dotted keys are refused ESS-AUTHOR-013.
tags:
- already-fixed
- downstream
- ess-issues-3
- zero-open-issues
relations:
- informed_by: story:nested-struct-per-leaf-comparison
- decomposes: epic:zero-open-issues-2026-11
revision: 4
transitions:
- {from: "draft", to: "archived", at: "2026-11-09T23:06:49Z", actor: "human:timo", revision: 4}
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


## Verification on 0.92.0 (2026-11-09)

Result: **already-fixed**. Checked against main at 4ab2ea1449 and the 0.92.0 release binary by the
zero-open-issues run (epic `epic:zero-open-issues-2026-11`); this draft leaves the plan as archived.

Proof: verify/an-authored-event-claim-names-a-struct-leaf/repro/run.txt; crates/verify/ess-conformance/tests/leaf_payloads.rs::an_authored_payload_error_and_row_claim_may_name_a_nested_member_by_path; commit 8a4a93fb7d; CHANGELOG 0.79.0 "From `ess-scenario/6` an authored claim may name one member ... by its dotted path"

Acceptance line 1 landed in 0.79.0 (commit 8a4a93fb7d, which names this story in its Refs). On
0.92.0 an `ess-scenario/6` payload claim `{to.city: Lisbon, to.region: {...}}` passes when `zip` is
left unclaimed, and the same claim with `to.city: Porto` fails (run.txt). Acceptance line 2 (a
struct literal claims only the leaves it writes) was rejected as option A in
`story:an-authored-claim-may-compare-a-nested-field` `## Revision (wave D335)`, because it changes
the meaning of existing documents. A partial struct literal is now refused at authoring as
ESS-AUTHOR-014 rather than failing a conforming target. Close as implemented-by-D335 and keep line 2
as rejected. One small leftover: the 014 help still begins "a command is invoked with all of its
input" for a payload claim.
