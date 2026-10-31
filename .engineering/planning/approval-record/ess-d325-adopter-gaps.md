---
format: aep.planning-md/3
id: approval-record:ess-d325-adopter-gaps
kind: approval-record
status: approved
title: Adopter-gap wave D325 runs as one wave, one pull request and one release
summary: Orchestrator approval D325 of 2026-10-31 under the operator's grant
tags:
- non-interactive
relations:
- decides: story:a-structured-input-may-carry-a-fixed-example
- decides: story:a-fixture-may-be-a-row-the-deployment-already-holds
- decides: story:a-transition-and-a-state-may-carry-a-summary
revision: 3
transitions:
- {from: "draft", to: "approved", at: "2026-10-31T11:28:38Z", actor: "human:timo", revision: 2}
---
## Decision

D325, 2026-10-31: the orchestrator approved one ESS wave for the open downstream gaps under the
operator's grant, with the operator's delivery rule of 2026-10-28 ("work in waves or batches. So we
fix multiple stories against one integration branch, bigger batches, multiple stories, and then we
do one PR. We do not run expensive full gates after every small change. We run targeted tests only
for what we're currently implementing, and only in the end, when everything ended up in the
integration branch, we run a local full gate and then we create the PR and auto merge it to the
default main branch.") and of 2026-10-31 ("they shall auto-merge on green pipeline"). The release
that follows the merge is the next version the store names (0.78.0).

This run is non-interactive: the coordinator is a sub-agent with no operator turn, so the stage-1
proposal stop is recorded here and the wave runs.

## Units

Triage on `main` f692aab (0.77.0), 2026-10-31: none of the six reported gaps is fixed. Order is the
orchestrator's: the consumer-blocked gaps first (G1 and G2 block a downstream deployment's
conformance suite; 45 of its 65 real-service failures are G1), then G3 and G4, then G5. At most four
units. The wave page is `docs/plan/wave-ess-d325.md`.

- U1, G1: `story:a-structured-input-may-carry-a-fixed-example`.
- U2, G2: `story:a-fixture-may-be-a-row-the-deployment-already-holds`; needs a design decision
  first, filed as a decision-blocker; the unit leaves the wave if it is not answered in time.
- U3, G3: `story:a-transition-and-a-state-may-carry-a-summary`.
- U4, G4: `story:an-authored-claim-may-compare-a-nested-field`.
- Next wave: G5 `story:identifier-guards-get-witnesses-and-authored-inputs-meet-their-type`, G6
  `story:a-binary64-value-round-trips-through-a-conformance-suite`.

Every story serves `vision:O2`.

## Commits this approval authorises

One or more commits per unit on its unit branch, the merges of green units into
`integration/ess-d325`, the store commits on that branch, the release commit, one pull request to
`main` from a `queue/` branch through the App and its merge on a green pipeline, then the release
tag and publication through the repository's own process. Not the next wave.

Decider: operator

## Revision (2026-10-31)

G2 was decided (option B, `architecture-decision-record:a-deployment-may-provide-an-entitys-rows`)
and took the fourth slot under the orchestrator's slot rule D327 (consumer-blocked work first);
`story:an-authored-claim-may-compare-a-nested-field` (G4) moves to the next wave. Three units: U1
(G1), U2 (G2), U3 (G3).
