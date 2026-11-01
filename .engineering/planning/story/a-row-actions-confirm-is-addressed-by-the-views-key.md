---
format: aep.planning-md/3
id: story:a-row-actions-confirm-is-addressed-by-the-views-key
kind: story
status: draft
title: A row action's inline confirm is addressed by the view's key
summary: ess-ui-tui addresses row confirms by live.match or id and ignores reads.key, so a view without id cannot reach its open confirm (0.77.0)
tags:
- ess-batch-5
- ess-next
- ess-ui
revision: 4
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

- Over a toy view keyed by `reads.key` on a field other than `id`, with no `id` field, an `ess ui
  test` step addressing `rows/<key>/row_actions/<a>/confirm/overlay` finds the open confirm and can
  accept and dismiss it.
- A view that has an `id` field and also declares `reads.key` is addressed by `reads.key`. Views
  without `reads.key` keep their current addresses.

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

The terminal renderer addresses a row action's inline confirm by `live.match`, else by the row's
`id`, and ignores the view's `reads.key` (`row_scoped` in `crates/ui/ess-ui-tui/src/app.rs`). For a
view with no `id` field, the step address `.../rows/<key>/row_actions/<a>/confirm/overlay` reports
"is not open" while the dialog is on screen.
