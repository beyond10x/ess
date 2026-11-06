---
format: aep.planning-md/3
id: approval-record:ess-0880-wave
kind: approval-record
status: approved
title: ESS 0.88.0 runs four units (suffixed-instance synthesis, journey steps, search box and go-task install, collection controls) as one wave, one pull request and release 0.88.0
summary: Coordinator approval of 2026-11-05 under the operator's grant, with the unit order and build terms
tags:
- non-interactive
relations:
- decides: story:a-suffixed-instance-is-created-with-the-inputs-the-specification-gives
- decides: story:a-journey-step-reaches-every-control-a-page-declares
- decides: story:a-read-given-the-search-state-record-draws-the-search-box
- decides: story:ci-go-task-install-survives-a-release-download-error
- decides: story:a-collection-draws-the-controls-it-declares
revision: 2
transitions:
- {from: "draft", to: "approved", at: "2026-11-05T18:11:25Z", actor: "human:timo", revision: 2}
---
## Decision

2026-11-05, the coordinator, under the operator's grant (class C: a wave, bot commits, a bot pull
request and its merge on green, a release through ess's own process), answered the wave proposal:

> Go for ESS 0.88.0 as proposed: U1, U3, U4, U2 in that order.
> I accept the three drafts: a-collection-draws-the-controls-it-declares,
> a-journey-step-reaches-every-control-a-page-declares and
> a-read-given-the-search-state-record-draws-the-search-box. Move them to active.
> File the 13 synthesis issues (#726-735, #738-740) as drafts in this wave's store commit, each with
> a one-line verdict, for me to accept later.
> On green: one bot PR, merge on green (squash), tag 0.88.0 after main is green, release, verify, the
> announcement file. The 0.87.0 store commits ride in this PR.

The go-task story was accepted by the coordinator earlier the same day. The wave lead moves the four
stories draft -> proposed -> active on those words and accepts none on its own.

This run is non-interactive: the wave lead is a sub-agent with no operator turn. Skill: aep
`implementing`, wave mode. Agents: `aep:implementor`, `aep:adversary`, at most 3 at once.

## Units

| unit | story | surface |
|---|---|---|
| U1 synthesis | a-suffixed-instance-is-created-with-the-inputs-the-specification-gives | ess-conformance synthesis; the layout-and-validation guide |
| U3 journey steps | a-journey-step-reaches-every-control-a-page-declares | ess-ui-test, ess-ui-tui |
| U4 small | a-read-given-the-search-state-record-draws-the-search-box; ci-go-task-install-survives-a-release-download-error | ess-ui-react `read_takes`; ci.yml and ci_lanes |
| U2 collections | a-collection-draws-the-controls-it-declares | ess-ui-react collection and styles |

## Terms

- Cargo limits per unit: tell at 20, stop at 40 cargo minutes; stop at the stated peak plus 2G
  (U1 10G, U3 7G, U4 5G, U2 5G). Slot waits do not count.
- One cargo at a time among the coordinator's agents. A pre-check once per unit and per gate stage:
  25G free on `/` and the trees budget, else stop.
- Gate: the staged stages measured in 0.87.0 plus the Checks job's clippy invocation, discard
  between stages; CI runs the full suite on the pull request.
- Merge: squash through the bot; the tag goes on the squash commit after `main`'s push run passes.
