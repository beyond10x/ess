---
format: aep.planning-md/3
id: approval-record:ess-d335-next-adopter-gaps
kind: approval-record
status: approved
title: Wave D335 runs G4, G5, G6 and the text-inequality story as one wave, one pull request and release 0.79.0
summary: Orchestrator approval D335 of 2026-10-31 under the operator's grant; folds integration/ess-issues-3
tags:
- non-interactive
relations:
- decides: story:an-authored-claim-may-compare-a-nested-field
- decides: story:identifier-guards-get-witnesses-and-authored-inputs-meet-their-type
- decides: story:a-binary64-value-round-trips-through-a-conformance-suite
- decides: story:a-text-inequality-against-an-example-is-refuted
- decides: story:an-authored-event-claim-names-a-struct-leaf
revision: 2
transitions:
- {from: "draft", to: "approved", at: "2026-10-31T19:58:46Z", actor: "human:timo", revision: 2}
---
## Decision

D335, 2026-10-31: the orchestrator approved the next ESS wave under the operator's grant, with the
operator's delivery rule of 2026-10-28 (several stories on one integration branch, targeted tests
per unit, one local full gate at the end, one pull request auto-merged to `main`) and of 2026-10-31
("they shall auto-merge on green pipeline"). The release that follows the merge is `0.79.0`.

This run is non-interactive: the coordinator is a sub-agent with no operator turn, so the stage-1
proposal stop is recorded here and the wave runs.

## Units

At most four, on non-colliding surfaces. Base `main` aaf484ecd (0.78.0).

- U1, G4: `story:an-authored-claim-may-compare-a-nested-field`, with the first acceptance line of
  `story:an-authored-event-claim-names-a-struct-leaf` (a dotted payload key); its second line (a
  struct literal claims only the leaves it writes) is option A of the G4 fit review, rejected there
  because it changes the meaning of existing documents.
- U2, G5: `story:identifier-guards-get-witnesses-and-authored-inputs-meet-their-type`; the fit review
  per finding is the unit's first deliverable, and a finding it declines or defers leaves the unit
  with its reason.
- U3, G6: `story:a-binary64-value-round-trips-through-a-conformance-suite`; the suite contract in
  `docs/design/model-binary64.md` is the unit's first deliverable.
- U4: `story:a-text-inequality-against-an-example-is-refuted`, accepted as written.

New format majors are shared, not raced: `ess-scenario/6`, the suite pair `ess-conformance/62`
(ordinary) and `/63` (declared coverage), and `ess/34` where a specification needs it; each unit
that needs one uses that number, and the coordinator unions the entries at merge.

## Folded in

`integration/ess-issues-3` (batch 3, `approval-record:d214-batch-3-consumer-blocked-first`: U11,
U13, U14, U15, merged there and never released) is not superseded by 0.78.0 and is merged into
`integration/ess-d335` at cb3dce50c. Its rules stay: an adopter replay before the release, and a new
refusal ships as a refusal only when the replay finds no adopter refused.

## Commits this approval authorises

One or more commits per unit on its unit branch, the merges of green units into
`integration/ess-d335`, the store commits on that branch, the release commit, one pull request to
`main` from a `queue/` branch through the App and its merge on a green pipeline, then the release
tag and publication through the repository's own process. Not the next wave.

Decider: operator
