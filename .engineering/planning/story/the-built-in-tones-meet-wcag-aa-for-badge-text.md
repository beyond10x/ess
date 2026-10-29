---
format: aep.planning-md/3
id: story:the-built-in-tones-meet-wcag-aa-for-badge-text
kind: story
status: draft
title: The built-in tones meet WCAG AA for badge text
summary: 'Badge text in the built-in tone table is below WCAG AA: warning 2.68, info 3.04, success 3.18, danger 3.98 to 1 (0.77.0)'
tags:
- ess-batch-6
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

- Every built-in tone's badge text and background reach at least 4.5:1 in the light and the dark theme. A test computes the contrast of each pair from the generated values.

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

Reported 2026-10-29 on ESS 0.77.0 by a downstream session, through the orchestrator. Not yet reproduced in this repository. ess-ui/1 accepts look-and-feel that the React renderer does not draw.

The built-in tone table is below WCAG AA (4.5:1) for badge text: warning 2.68:1, info 3.04:1, success 3.18:1 and danger 3.98:1.
