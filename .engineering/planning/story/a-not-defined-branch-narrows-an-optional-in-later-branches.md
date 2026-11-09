---
format: aep.planning-md/3
id: story:a-not-defined-branch-narrows-an-optional-in-later-branches
kind: story
status: draft
title: A `not defined` branch narrows an Optional for the branches after it
summary: A payload after an absent-optional refusal still sees Optional<String> (workbench report item 19)
tags:
- backlog
- external-request
- from-consumer-report
relations:
- serves: vision:O2
revision: 1
---
## Need

After a branch that refuses when an optional field is absent, later branches still see the field
as `Optional`: `ESS-COMMAND-002 the subject's \`note\` has type \`Optional<String>\`, and \`note\`
requires \`String\``. Same shape as #556 (closed), still present for a payload. Item 19 of the
2026-11-09 workbench report; example `19`.

## Acceptance

- A branch that refuses when a subject or input field is `not defined` narrows that field to its
  inner type in every later branch's payloads, `sets:` and guards.
- The interpreted target and synthesis agree, and a branch before the refusal keeps the `Optional`.

## Scope

- inferred: `crates/specify/ess-domain` (branch typing, the #556 narrowing), its tests.

## Fit review

Pending. An external request: it is assessed with
`.agents/skills/assessing-external-requests/SKILL.md` before this story leaves draft.

## Source

`.engineering/repro/workbench-2026-11-09/report.md`, with its examples under `examples/`.
