---
format: aep.planning-md/3
id: approval-record:ess-0700-wave-1
kind: approval-record
status: approved
title: 'Wave 1 of 0.70.0: nine units, fourteen stories'
summary: Adopter gaps reproduced on 0.69.2 and the smaller queued items, run as nine units under the orchestrator's grant, non-interactive.
tags:
- ess-0.70.0
- non-interactive
relations:
- decides: story:an-identity-guard-beside-a-row-set-validates
- decides: story:a-row-set-branch-arranges-rows-past-the-creators-predicate-guard
- decides: story:a-published-event-payload-has-a-generated-decoder
- decides: story:generated-dispatch-and-handle-return-the-emitted-events
- decides: story:identity-minting-and-view-ordering-are-public
- decides: story:a-denied-grant-over-a-command-that-writes-nothing-is-not-an-error
- decides: story:a-view-may-publish-a-decimal
- decides: story:a-binary64-field-is-admitted-by-a-conformance-run
- decides: story:a-struct-that-ignores-undeclared-fields-may-declare-none
- decides: story:a-lifecycle-state-carries-a-wire-name
- decides: story:a-dotted-variant-name-synthesizes-as-an-identifier
- decides: story:ess-generate-types-checks-drift
- decides: story:a-view-field-from-names-the-related-spelling
- decides: story:create-or-update-beside-a-related-row-refusal-synthesizes-every-branch
revision: 2
transitions:
- {from: "draft", to: "approved", at: "2026-10-26T09:19:23Z", actor: "human:timo", revision: 2}
---
## Decision

Relayed by the orchestrator on 2026-10-26: "Item: ESS 0.70.0. ... The queue:
approval-record:adr-unblocked-units-in-0700 ... and approval-record:ess-0690-cut-after-gu19 ... Add
the open gaps reported by consuming repositories ...: reproduce each from its repro path, file a
story per real gap, and include in 0.70.0 those that fit; list the rest with their story id."
Approved under the orchestrator's grant; the run is non-interactive (a sub-agent of the
orchestrator, no operator turn).

## Wave 1 of 0.70.0

Selection made by the coordinator, read pairwise from each story's `## Scope` (written
2026-10-26), not by `aep plan artifact waves`. Builds run one at a time on this machine, so units run
in sequence with at most two agents at once.

| unit | stories | main surface |
|---|---|---|
| A | an-identity-guard-beside-a-row-set-validates | ess-domain row sets, `ess/32` |
| B | a-row-set-branch-arranges-rows-past-the-creators-predicate-guard | ess-conformance synthesize/row_set.rs |
| C | a-published-event-payload-has-a-generated-decoder; generated-dispatch-and-handle-return-the-emitted-events | ess-synth rust/http.rs |
| D | identity-minting-and-view-ordering-are-public; a-denied-grant-over-a-command-that-writes-nothing-is-not-an-error | ess-conformance interpret |
| E | a-view-may-publish-a-decimal; a-binary64-field-is-admitted-by-a-conformance-run | ess-conformance replay, admission |
| F | a-struct-that-ignores-undeclared-fields-may-declare-none; a-lifecycle-state-carries-a-wire-name | ess-domain types, `ess/32`, after A |
| G | a-dotted-variant-name-synthesizes-as-an-identifier; ess-generate-types-checks-drift | ess-synth, ess-gen, ess-cli |
| H | a-view-field-from-names-the-related-spelling | ess-domain view refusal hint |
| I | create-or-update-beside-a-related-row-refusal-synthesizes-every-branch | ess-conformance related guards |

Every unit is objective `vision:O2`. Collisions read: A and F share the `ess/32` format entry (F
after A); C's two stories share `ess-synth/src/rust/http.rs` (one unit); E's two stories share
`replay.rs` (one unit). Unassessed: none.

## Commits this record authorises

One commit per unit on its unit branch, the merges of those branches into
`integrate/ess-0700`, the store commits on that branch, a pull request of it into `main` merged
through the bot route once its Gate is green, and the 0.70.0 release per `AGENTS.md`. All through
`b10x-bot[bot]`.
