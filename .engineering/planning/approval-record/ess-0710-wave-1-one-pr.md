---
format: aep.planning-md/3
id: approval-record:ess-0710-wave-1-one-pr
kind: approval-record
status: approved
title: Wave 1 of 0.71.0 integrates every unit before one gate and one pull request
summary: 'D83: targeted tests per unit, the four units merged together, one full gate and one adversary review on the integrated branch, one PR.'
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
- {from: "draft", to: "approved", at: "2026-10-27T09:12:32Z", actor: "human:timo", revision: 2, executor: "agent:b10x-chef-orchestrator-ess-0710"}
---
## Decision

Relayed by the orchestrator on 2026-10-27 (decision D83), quoting the operator verbatim: "for ess,
make sure to just provide a single PR to gh, do not run expensive gates for every small change.
integrate all units at once".

## What it changes for wave 1 of 0.71.0

- Per unit, only that unit's own targeted tests run: the changed crate's tests that cover the story
  and its new conformance cases. No wider gate per unit.
- The four units merge into `integrate/ess-0710`, and the full gate runs once on the integrated
  branch: the CI run of the one release pull request, as `AGENTS.md` § Gate has the full gate run
  in CI and no local full gate before a release.
- Exactly one pull request to `main`, carrying the four units, the `integrate/ess-0700` commits and
  the release commit. A failure is fixed on the integration branch and only what the fix touches is
  re-run locally.
- One adversary review runs on the integrated branch, in parallel with that gate.
