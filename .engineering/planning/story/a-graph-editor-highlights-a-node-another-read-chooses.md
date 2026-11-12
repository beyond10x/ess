---
format: aep.planning-md/3
id: story:a-graph-editor-highlights-a-node-another-read-chooses
kind: story
status: archived
title: A graph editor highlights a node another read chooses
summary: '`highlight` is an unknown key of graph_editor (workbench report item 42)'
tags:
- backlog
- external-request
- from-consumer-report
relations:
- serves: vision:O2
revision: 3
transitions:
- {from: "draft", to: "archived", at: "2026-11-12T21:48:19Z", actor: "human:timo", revision: 3, executor: "agent:b10x", correlation: "operator-2026-11-12-round-2"}
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

## Decisions

**Declined under the operator's procedure for external-request drafts (2026-11-12).** He set
it that day: the session that maintains the repository recommends per draft, an independent
adversarial review reads each draft, and a draft both read as drop is archived with this note.
This draft was number 37 of a second list of 13 (numbers 26 to 38); both read drop. The
transition to `archived` carries the correlation `operator-2026-11-12-round-2` and the executor
`agent:b10x`.

Reason: nothing is blocked. One page of the reporting application shows no current step; the
graph is drawn. Class (fit question 2): convenience.

Idiom: a host slot draws the graph with its highlight. A host slot declares `stands_in_for`,
`reads` and `commands` (`website/docs/reference/ess-ui.md`, the host slot's properties;
`design:ui-host-slots`). A slot that reads both the graph's view and the view that chooses the
node needs `story:a-host-slot-reads-a-list-of-views`, accepted in the same round.

Reopened by: a second requester, or a case a host slot cannot draw.

Notice to the requester: the request came with the workbench report of 2026-11-09
(`.engineering/repro/workbench-2026-11-09/report.md`). No session of its author is reachable, so
this section is the notice.
