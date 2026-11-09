---
format: aep.planning-md/3
id: story:the-interpreted-target-passes-a-related-run-digest-and-a-default-step
kind: story
status: draft
title: The interpreted target passes a whole-model related-run digest and default step
summary: 15 interpreted scenarios fail on DigestMismatch and an await deadline in a whole model (workbench report items 37, 38)
tags:
- backlog
- external-request
- from-consumer-report
relations:
- serves: vision:O2
revision: 1
---
## Need

On the reporter's whole model (not minimised), the interpreted target fails 15 scenarios:

- `MarkConforming/outcome/conforming` observes another model digest (`DigestMismatch`), and the view
  stays `Validated` (item 37).
- `TakeDefault/outcome/no-default` ends `nothing reached by the runner's deadline, after 50
  await(s)` (item 38).

Items 37 and 38 of the 2026-11-09 workbench report. Model only. The model is not in the repository;
the reporter holds it.

## Acceptance

- Each failure is minimised to an example in the repository, and is then a defect of the
  interpreted target or of synthesis with a regression test.
- The reporter's whole model passes those scenarios on the interpreted target.

## Scope

- unknown until minimised: `crates/verify/ess-conformance/src/interpret/`, synthesis setup.

## Fit review

Pending. An external request: it is assessed with
`.agents/skills/assessing-external-requests/SKILL.md` before this story leaves draft.

## Source

`.engineering/repro/workbench-2026-11-09/report.md`, with its examples under `examples/`.
