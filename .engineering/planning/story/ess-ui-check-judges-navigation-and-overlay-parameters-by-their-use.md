---
format: aep.planning-md/3
id: story:ess-ui-check-judges-navigation-and-overlay-parameters-by-their-use
kind: story
status: draft
title: ess ui check judges navigation and overlay parameters by their use
summary: A nav link to a page needing an id passes; a param bound only into a command is called unread (workbench report items 6, 10)
tags:
- backlog
- external-request
- from-consumer-report
relations:
- serves: vision:O2
revision: 1
---
## Need

Two `ess ui check` answers disagree with what the document does:

- A page that requires an id may be listed in navigation. `ess ui check` and `--model` report 0
  errors, and the link renders without an id (item 6, example `ui06`).
- An overlay parameter read only as a command input is reported as unread: `overlay_params ...
  \`params.id\` is passed to this overlay and read nowhere inside it` (item 10, example `ui10`;
  see #322, #324).

Items 6 and 10 of the 2026-11-09 workbench report.

## Acceptance

- A navigation entry to a page whose route requires a parameter is an `ess ui check` error naming
  the page and the parameter.
- An overlay parameter bound into a `does` command input counts as read; the warning stays for a
  parameter nothing reads or binds.

## Scope

- inferred: `crates/ui/ess-ui-check/src/rules.rs` (navigation, `overlay_params`), its tests.

## Fit review

Pending. An external request: it is assessed with
`.agents/skills/assessing-external-requests/SKILL.md` before this story leaves draft.

## Source

`.engineering/repro/workbench-2026-11-09/report.md`, with its examples under `examples/`.
