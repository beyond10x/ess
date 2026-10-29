---
format: aep.planning-md/3
id: story:a-record-action-in-the-terminal-renderer-sees-its-row
kind: story
status: draft
title: A record action in the terminal renderer sees its row
summary: 'ess-ui-tui leaves row out of scope: a row.<key> bind sends null and a row-guarded action does not act; React differs (0.77.0)'
tags:
- ess-next
- ess-ui
revision: 3
---
<!-- Starting point for a `story` artifact, seeded by `aep artifact new story <name>`.
     No frontmatter here on purpose: the `---` block is written by the CLI from the id, kind, status
     and relations you gave it, and a second copy in this file would be the one that went stale.
     Delete the italic guidance as you fill each section. -->

# Story: <name>

## Outcome

*What is true for whom once this has shipped, in one sentence. If it names a component rather than a
person, it is a task — say what changes for someone.*

## Context

*Why this is worth doing now, and what it depends on. Link the epic or specification it comes from
rather than restating it; the `derived_from` relation already carries the edge.*

## Acceptance

- Over a toy `ui-spec` whose record action binds `row.<key>` and is visible on `row.state == <S>`, an
  `ess ui test` script run against the terminal renderer sends the row's key, and `act` on a
  visible action sends its command.
- The same script gives the same commands against the React target. One test runs both renderers
  over the same document and compares the commands they send.

## Out of Scope

*What a reasonable reader would expect to be included and is not — the boundary that stops this
story quietly becoming an epic.*

## Ambiguities

*Each gap this story found and did not close, classified. `inferable` — the answer is already
written down, so give the `path:line` or the artifact id that settles it.
`requires-stakeholder-input` — nobody here can decide it, so name who does, and raise that entry as
a `decision-blocker` with a `blocks` edge to this story, or it is a sentence somebody improvises
later.*

## Open Questions

*Anything still undecided belongs in `## Ambiguities` above, classified and with its citation or its
decider. Keep this section for a question that is neither — an unowned question is a story that
stalls without anybody noticing.*

## Problem

Reported 2026-10-29 on ESS 0.77.0 by a downstream session, through the orchestrator. Not yet
reproduced in this repository.

In the terminal renderer (`crates/ui/ess-ui-tui`, behind `ess ui test`), a record action cannot see
its row:

- With `bind: {<key>: row.<key>}` the action sends null for that input.
- With `visible: "row.state == <S>"` the key hint shows the action, but `act` does nothing.

The React target puts the row in scope for the same action, so the two renderers disagree on one
`ui-spec` document.
