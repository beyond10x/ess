---
format: aep.planning-md/3
id: story:feature-request-306
kind: story
status: implemented
title: Committed generated output regenerates in another checkout
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#306
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-10-01T16:38:14Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-01T16:38:15Z", actor: "human:timo", revision: 3}
- {from: "active", to: "implemented", at: "2026-10-02T11:41:31Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":1,"review_outcome":2,"verification":1}}, executor: "agent:codex-ess-backlog", correlation: "consumer-backlog-20261002"}
---
## Outcome

Committed generated output regenerates in any checkout of the repository.

## Acceptance

- An Idle output state is rebound to a moved or second checkout, and generation runs.
- A non-Idle state still refuses a mismatched root, naming the recorded path.

## Origin

beyond10x/ess#306, reported downstream on 0.48.0.

## Fit review

- Class: defect. Committed `.ess-output` is a documented layout (`website/docs/start/runners/go.md:59`), but it works in one checkout only. No new authored surface.

## Decisions

- **accept, redesigned** (coordinator, 2026-10-01): rebind an Idle checkpoint; keep the strict binding for in-flight transactions. Priority 1 (base defect); ships in the next release.

## Reconciled released implementation, 2026-10-02

Tag0.51.0 (peeled0347ffa222939e3791e574d2dbe42d4b4b02d979) contains release integration482bc33c251859d81d52b52d547c7fc04001ec98 and is an ancestor of current mainb4da64e38b770fe74103409fe1fef7ae6ca214f4. Output-ownership implementation and relocation tests have no diff between release/tag/main. GitHub306 closed2026-10-02T02:59:04Z.

The delivered contract is settled Idle relocation only, with owned paths, digests, aliases and ancestors checked. In-flight moved roots and same-path copied directory identities refuse; unchanged no-op checks/recovery remain bound in memory and first actual write records the new root. Recorded modes keep a different-umask clone clean. Differing files require the printed adoption route, including moving them aside first. These corrections are recorded in adversary-gaps-306-pass-1 and pass-2 and tested in ownership_relocation, ownership_relocation_adversary and ownership_relocation_adversary_p2.

Existing immutable evidence at2026-10-01T19:18:37Z records coordinator rerun: pass2 10/10 and output_ownership51/51. The prior881-case CLI run predates the route amendment and is not claimed as final full-suite evidence. The new test_result record below classifies the already recorded observed test run under the lifecycle's required kind, preserving its original instant and reference; no new test execution is claimed by this reconciliation.
