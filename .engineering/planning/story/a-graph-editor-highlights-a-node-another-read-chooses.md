---
format: aep.planning-md/3
id: story:a-graph-editor-highlights-a-node-another-read-chooses
kind: story
status: draft
title: A graph editor highlights a node another read chooses
summary: '`highlight` is an unknown key of graph_editor (workbench report item 42)'
tags:
- backlog
- external-request
- from-consumer-report
relations:
- serves: vision:O2
revision: 1
---
## Need

A `graph_editor` cannot mark a node chosen by another read, such as the current step of a run:
`unknown field \`highlight\`, expected one of \`key\`, \`label\`, \`kind_by\`, \`opens\``. Item 42 of the
2026-11-09 workbench report; example `ui42`.

## Acceptance

- A `graph_editor` may declare which nodes to highlight from another declared read, keyed by the
  node key. React and the terminal mark those nodes.
- `ess ui check --model` holds the read and the key to the model as other reads are held.

## Scope

- inferred: `crates/ui/ess-ui` (graph_editor), `crates/ui/ess-ui-check`, the React graph runtime,
  the terminal renderer.

## Fit review

Pending. An external request: it is assessed with
`.agents/skills/assessing-external-requests/SKILL.md` before this story leaves draft.

## Source

`.engineering/repro/workbench-2026-11-09/report.md`, with its examples under `examples/`.
