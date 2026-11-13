---
format: aep.planning-md/3
id: story:the-one-row-check-reads-the-instance-its-scenario-bound
kind: story
status: active
title: The one-row check after an update reads the instance its scenario bound
summary: Synthesis appends a snapshot naming an instance the scenario never bound, so conform run errors (ESS-CF-SUITE, exit 3) on a valid model after an enum gains a variant (0.90.0 to 0.94.0)
tags:
- backlog
- defect
- from-finding
- severity-high
relations:
- serves: vision:O2
- decomposes: epic:zero-open-issues-2026-11
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-11-12T22:38:35Z", actor: "human:timo", revision: 4, executor: "agent:b10x", correlation: "operator-2026-11-12-d8"}
- {from: "proposed", to: "active", at: "2026-11-12T22:38:37Z", actor: "human:timo", revision: 5, executor: "agent:b10x", correlation: "operator-2026-11-12-d8"}
---
## Defect

`ess verify conform synthesize` writes, for the updating branch that pairs with an
`unknown_instance:` creating branch on one identity, a scenario whose last step reads an instance
no earlier step bound. `ess verify conform run` then ends that scenario as an error and exits 3 on
a valid model:

```
error <domain>.JoinInstance/outcome/rejoined
    ESS-CF-SUITE: no earlier step bound the instance `operator`
```

Reported by a consumer on 0.90.0 and 0.93.0 (2026-11-12); reproduced on the 0.94.0 release binary.
In its specification the scenario passed until an enum the command's input carries got a fourth
variant: with the variant last or third the error appears, with it first the scenario passes and
another scenario leaves the suite. Nothing else about the command changed.

The synthesized scenario (0.94.0), instance references by step:

| step | kind | instance |
|---|---|---|
| 2 | `capture_instance` | `operator-2` |
| 3 | `execute_command` (leave), input `key` | `operator-2` |
| 5 | `execute_command` (join again), input `key` | `operator-2` |
| 7 | `expect_event_values`, payload `key` | `operator-2` |
| 10 | `expect_view`, field `key` | `operator-2` |
| 12 | `snapshot_subject`, subject `key` | `operator` |

With three variants all six references read `operator`.

## Cause

Steps 11 and 12 are the one-row check that `one_row` in
`crates/verify/ess-conformance/src/synthesize/existence.rs` appends to the scenario already filed
for the updating branch. It takes the instance name from a second run of the branch
(`super::run_as(models, command, updating, actors, Witness::Full)`), under the comment "synthesis
is deterministic, so this is the row the scenario's second call leaves, and the instance its
arrangement bound". The filed scenario's arrangement bound `operator-2` (an instance name is the
entity's word plus its `Distinction` number, `instance_name` in `synthesize.rs`), the second run
names `operator`, and nothing compares the two. Which pass files the scenario with the further
distinction for this model is not traced yet; the change of the enum's size moves which witness
the arrangement and the call take.

## Acceptance

1. The one-row check names the instance the filed scenario's own second call addresses (read from
   the scenario's steps), or the scenario is built once with the check in it. No step of a
   synthesized scenario reads an instance no earlier step of that scenario binds: a test walks
   every synthesized scenario of the workspace's example models and of the model below and fails
   on such a reference, before any run.
2. A minimal model reproduces the report and is a regression test: an entity with a struct
   identity and a lifecycle with a transition back to its initial state; a command with a guarded
   refusal over an enum field of its input (`actor.kind != Human`), an `unknown_instance:` creating
   branch, a moving branch on the same identity with `sets:`, and a `wrong_state:` refusal; a
   second command that moves the row out of the initial state; a read-your-writes view of the
   entity carrying its identity. The enum with three variants and with four both synthesize a
   suite that `ess verify conform run --target interpreted` passes with 0 errors.
3. Reordering the enum's variants changes no scenario id of the suite. On 0.94.0 this holds for
   the report's model: with the new variant first and with it third the run lists the same 717
   scenario ids, and variant-first passes 717 of 717. The reporter saw one scenario leave the
   suite in the variant-first case on 0.90.0; that does not reproduce on 0.94.0 and is no part of
   this story.
4. The suite bytes of every committed example model are unchanged, or the changelog names the
   scenarios that changed.

## Evidence

Reproduction on the 0.94.0 release binary and both synthesized suites: session scratch
`ess/adopter10/` (`run.sh`, `base-suite.json`, `four-suite.json`, `four-conform.log`; not in the
repository). Base: 695 scenarios, 0 errors. With the fourth variant: 717 scenarios, 716 passed,
1 error, exit 3.

## Decisions

**Accepted (2026-11-12).** Decided on the operator's behalf, under the delegation the operator
gave that day for decisions that are low risk or easy to revert. Reason given: a defect against
shipped behaviour, reproduced on the 0.94.0 release binary; an accepted story can be moved back.
The two transitions carry the correlation `operator-2026-11-12-d8` and the executor `agent:b10x`. No
review-result exists for this draft, so no `review_outcome` record was written.
