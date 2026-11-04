---
format: aep.planning-md/3
id: approval-record:ess-0870-wave
kind: approval-record
status: approved
title: Wave W1 of the reconcile plan runs four units (synthesis refusals, OpenAPI publication text, value formats, choices/filters/forms) as one wave, one pull request and release 0.87.0
summary: Coordinator approval of 2026-11-04 under the operator's grant, with the W1 decision round's answers
tags:
- non-interactive
relations:
- decides: story:a-not-found-branch-after-a-guarded-move-is-refused-with-its-repair
- decides: story:a-value-invariant-on-a-type-nothing-reaches-refuses-nothing
- decides: story:the-openapi-says-published-only-for-events-the-component-publishes
- decides: story:a-value-renders-in-the-form-its-field-declares
- decides: story:choices-filter-bars-and-forms-render-what-they-declare
revision: 3
transitions:
- {from: "draft", to: "approved", at: "2026-11-04T08:47:27Z", actor: "human:timo", revision: 2, executor: "agent:ess-w1-lead"}
---
## Decision

2026-11-04, the coordinator, under the operator's grant (class C: a wave, bot commits, a bot pull
request and its merge on green, a release through ess's own process), answered the wave proposal:

> Accept all five drafts.
> Q-W: a. The plan's collections and runner steps move to W2.
> Q-A: a. Q-C: a.
> G-3: a. G-7: a. G-8: a. G-12: a. G-13: a. G-14: a. G-15: a.
> B7-1: a (B/KB/MB/GB in 1024 steps; metric `bytes` scales the same way).
> Record this in an approval record (no private ids) and move the stories to active. Release 0.87.0.

The five stories below are moved draft -> proposed -> active by the wave lead on those words; the
lead accepted none on its own.

This run is non-interactive: the wave lead is a sub-agent with no operator turn. Skill: aep
`implementing` 0.26.0, wave mode. Agents: `aep:implementor`, `aep:adversary`.

## The questions and the answers

| question | answer |
|---|---|
| Q-W, the wave's units | a: synthesis (two stories), OpenAPI publication text, value formats, choices/filters/forms. The reconcile plan's collections (A10, A21-A27) and runner steps (B01-B11, B33-B35) move to W2. |
| Q-A, a not-found after a guarded move | a: the precedence order stands; one refusal whose help names `before_row: true` and declaring the not-found first; no ESS-SYNTH-019 |
| Q-C, an invariant on a type nothing reaches | a: a note, as an unreached `Binary64` position is; a reached type keeps ESS-SYNTH-013 |
| G-3, header badge | a: `header.badge: {text, tone}`, `text` an expression (built in W2) |
| G-7, action tone | a: `tone:` on an action, the button's words (primary, secondary, danger, ghost) (built in W2) |
| G-8, field tone | a: `tone` and `tone_by` on a field, badge and plain columns; `tone_by: <map>` reads the field's own value (built in W2) |
| G-12, lists in binds | a: a YAML sequence of expressions in a bind, sent as a JSON array (built in W2) |
| G-13, help | a: `help: {text, link}` on a form group and a field (a field covers its column header) |
| G-14, choice placeholder | a: `placeholder: <text>` on a choice, also the empty dropdown option |
| G-15, reset label | a: `reset: true` or `reset: {label}` |
| B7-1, byte size | a: `as: bytes` on a field, the metric's existing word; B/KB/MB/GB in 1024 steps; metric `format: bytes` scales the same way |

## Units

Base `origin/main` 8d26c81f74. Integration branch `integration/ess-0.87.0`, tree `ess-w087-int`.
Release 0.87.0.

| unit | tree | branch | stories |
|---|---|---|---|
| U1 | ess-w087-u1 | unit/ess-0.87.0-u1 | story:a-not-found-branch-after-a-guarded-move-is-refused-with-its-repair, story:a-value-invariant-on-a-type-nothing-reaches-refuses-nothing (serial, one implementor: both land in synthesize.rs) |
| U2 | ess-w087-u2 | unit/ess-0.87.0-u2 | story:the-openapi-says-published-only-for-events-the-component-publishes |
| U3 | ess-w087-u3 | unit/ess-0.87.0-u3 | story:a-value-renders-in-the-form-its-field-declares |
| U4 | ess-w087-u4 | unit/ess-0.87.0-u4 | story:choices-filter-bars-and-forms-render-what-they-declare |

U3 and U4 share `crates/ui/ess-ui/src/model.rs`, `schemas/ui/ess-ui.schema.yaml`,
`crates/ui/ess-ui-tui/src/view.rs` and the generated `website/docs/reference/ess-ui.md`, split by
symbol: U3 owns `Field`'s `as:` vocabulary and the metric format; U4 owns `Choice`
(`placeholder`), `FilterBar` (`reset`), `FormGroup` and `Field` (`help`). The reference page is
regenerated at each merge.

## Commits this authorises

The unit commits; the merges into `integration/ess-0.87.0`; store commits (this record, scope,
evidence, close); the 0.87.0 release prepare; one bot pull request from a `queue/ess-0.87.0` head,
merged on green; tag 0.87.0 and the release through ess's own process. Nothing else.

## Go and build terms

The orchestrator relayed the host's build terms on 2026-11-04 at 08:5xZ, and the wave started on them:

| what | tell at | stop at |
|---|---|---|
| U1 | 8G target/ or 90 min | 10G or 150 min |
| U2 | 12G or 45 min | 15G or 75 min |
| U3, U4 | 4G or 120 min | 6G or 180 min |
| each gate stage | 14G | 18G |

- One ESS tree builds at a time. Every cargo call goes through the shared build slot; units never take a second slot.
- The gate runs 9 stages with the cache discarded between stages. S6 to S9 take the second slot beside one unit, and
  each starts only with at least 45G free on `/`.
- Every build uses line-tables-only debug information, no incremental compilation and 4 test threads.
- A unit's tree drops its build cache as soon as the unit merges into the integration branch.

## Ride-along: CI test shards and runner disk

The orchestrator added `story:ci-test-shards-leave-evidence-when-the-runner-runs-out-of-disk` to this wave as a
ride-along on the integration branch, not as a fifth unit: "measure where the shard's disk goes; cut it, or free
more before the build ...; add the crash-surviving sampler if it is cheap. Accept it into W1 as a ride-along; no
fifth unit." The story moves to active with this record.
