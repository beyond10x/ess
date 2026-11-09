---
format: aep.planning-md/3
id: story:an-inputs-scenarios-entry-may-be-a-multi-document-file-or-a-directory
kind: story
status: draft
title: An inputs `scenarios:` entry may hold several documents or name a directory
summary: ess-inputs/2 scenarios refuses a multi-document file and a directory (workbench report item 35)
tags:
- backlog
- external-request
- from-consumer-report
relations:
- serves: vision:O2
revision: 1
---
## Need

`scenarios:` in `ess-inputs/2` takes only single-document files:

- One file with several documents: `unreadable: deserializing from YAML containing more than one
  document is not supported` (example `35b`).
- A directory: `scenarios entry "scenarios/dir" is not a regular file; list a file rather than a
  directory or special node` (example `35c`; control `35a`).

Not the same as #735. Item 35 of the 2026-11-09 workbench report.

## Acceptance

- A scenarios file may hold several YAML documents, each one scenario.
- A scenarios entry may name a directory, read in a stable order, its files as above.
- The order of scenarios in the suite does not depend on the file system.

## Scope

- inferred: the `ess-inputs` reader (`crates/specify/ess-compiler`), the scenarios loader.

## Fit review

Pending. An external request: it is assessed with
`.agents/skills/assessing-external-requests/SKILL.md` before this story leaves draft.

## Source

`.engineering/repro/workbench-2026-11-09/report.md`, with its examples under `examples/`.
