---
format: aep.planning-md/3
id: approval-record:d214-batch-3-consumer-blocked-first
kind: approval-record
status: approved
title: Batch 3 takes consumer-blocked defects first and ships as 0.78.0 (D214)
summary: 'U13 parent-row tenant guard, U14 cascade witness, U15 scenario reading, U11 synthesis order (#645, #693)'
relations:
- decides: story:a-guard-on-the-input-addressed-parent-row-holds-on-every-command-of-its-children
- decides: story:a-cascade-onto-rows-whose-creator-needs-the-subject-active-is-witnessed
- decides: story:a-scenario-with-a-repeated-key-is-refused
- decides: story:an-integer-literal-is-written-into-the-suite-as-an-integer
revision: 3
transitions:
- {from: "draft", to: "approved", at: "2026-10-29T16:44:34Z", actor: "human:timo", revision: 3, executor: "agent:ess-issues-lead", correlation: "ess-issues-3"}
---
# Batch 3 takes consumer-blocked defects first and ships as 0.78.0 (D214)

## Decision

The orchestrator approved on 2026-10-29, under the operator's grant, as D214: batch 3 starts on
`integration/ess-issues-3` after the `0.77.0` release, holds at most four units, takes
consumer-blocked defects first, and ships as `0.78.0` under the batch 2 rules (one local gate, one
pull request, one release, a replay against the adopters before the release).

- U13 parent-row tenant guard: `story:a-guard-on-the-input-addressed-parent-row-holds-on-every-command-of-its-children`.
- U14 cascade witness: `story:a-cascade-onto-rows-whose-creator-needs-the-subject-active-is-witnessed`.
- U15 authored scenario reading: `story:a-scenario-with-a-repeated-key-is-refused` and
  `story:an-integer-literal-is-written-into-the-suite-as-an-integer`.
- U11 synthesis order: beyond10x/ess#645 and beyond10x/ess#693, scoped on 2026-10-28.
- U12 (#694) and the two #554 synthesis stories wait for a later batch.
- A new refusal ships as a refusal only when the adopter replay finds no adopter refused; else it
  ships as a warning in this release (as D186 ruled for U10).
- A unit whose fit review finds a new source format, an ADR-level change or a breaking change with
  no opt-in form leaves the batch with a decision-blocker (D135).
- Every tree builds into its own `target/`, one ESS tree at a time.

Decider: operator
