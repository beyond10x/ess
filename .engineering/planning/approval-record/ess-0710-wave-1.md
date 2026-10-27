---
format: aep.planning-md/3
id: approval-record:ess-0710-wave-1
kind: approval-record
status: approved
title: 'Wave 1 of 0.71.0: four adopter-reported synthesis gaps'
summary: Batch 2c K and M, 2d N and batch 2 item A, run under the orchestrator's grant, non-interactive.
tags:
- ess-0.71.0
- non-interactive
relations:
- decides: story:a-set-update-arranges-rows-whose-filtered-value-the-creator-copies-from-a-related-row
- decides: story:a-set-update-witness-varies-the-creators-input-against-the-written-value
- decides: story:an-affects-row-is-arranged-past-a-creators-emitting-not-found-branch
- decides: story:a-converted-binding-input-arranges-the-related-row
revision: 2
transitions:
- {from: "draft", to: "approved", at: "2026-10-26T21:52:52Z", actor: "human:timo", revision: 2}
---
## Decision

Relayed by the orchestrator on 2026-10-26 (21:49Z): "Item: ess 0.71.0 wave 1. ... Units (at most 4;
pick in this order, each an accepted story in the store; the stories are drafts filed today by the
0.70.0 coordinator, accept them under the grant)". Approved under the orchestrator's grant (class C:
wave, bot commits, PR, merge, release). The run is non-interactive: a sub-agent of the orchestrator
with no operator turn.

## Wave 1 of 0.71.0

Selection fixed by the orchestrator's order; scope read pairwise from the refusal sites (the
`aep plan artifact waves` verb was not run: the four units were named, not selected).

| unit | story | repro | main surface (inferred from the refusal text) |
|---|---|---|---|
| U1 | a-set-update-arranges-rows-whose-filtered-value-the-creator-copies-from-a-related-row | repro-r3a-r1 2/1, repro-r3a 5/1, guarded-creator inst.yaml 5/1 | ess-conformance `synthesize/set_effects.rs` rows witness (`NO_MATCHING`, `UNARRANGED` help) |
| U2 | a-set-update-witness-varies-the-creators-input-against-the-written-value | r3a-work/t1_true 1/1, t2_creator_input 1/1 | same rows witness as U1 |
| U3 | an-affects-row-is-arranged-past-a-creators-emitting-not-found-branch | repro-r3b 3/1 (r3b-fixed 4/0) | `set_effects.rs` affects segment (`UNDRIVEN`) |
| U4 | a-converted-binding-input-arranges-the-related-row | repro-r1 5/3 | `synthesize.rs` binding refusal (ESS-SYNTH-010) |

Counts are selected scenarios / refusal occurrences from `ess verify conform synthesize --suite-format
5 --compact`, measured with ess 0.69.2 on 2026-10-26.

Batch 2 item A (`a-converted-binding-input-arranges-the-related-row`, repro-r1) is taken over item B
(`a-binding-whose-trigger-shares-the-row-guard-flows`, repro-r1b): B's acceptance keeps
`binding/refusal/group-missing` refused, so its repro cannot reach 0 refusals, and A is the
smaller defect (one conversion to follow).

Collisions read: U1 and U2 share the set update rows witness, so they run as one unit tree with
two stories. U3 shares `set_effects.rs` with U1/U2 in a different region (affects segment); it is
forked after U1/U2 merges. U4 is disjoint. Unassessed: none. Every story serves `vision:O2`.

Pre-flight: 124G free on `/` at 21:49Z; one ess tree builds at a time (builds serialized with a
lock file). Wave cap: 4 units or 4 h from 21:50Z; wall budget 3 h.

Commits this record authorizes: one or more per unit on its branch, the merges into
`integrate/ess-0710`, the store commits, the 0.71.0 release commit, the release PR, its merge
through the App, the tag and the publication, all through `b10x-bot[bot]`.
