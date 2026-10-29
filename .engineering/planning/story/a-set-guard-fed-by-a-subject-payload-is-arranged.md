---
format: aep.planning-md/3
id: story:a-set-guard-fed-by-a-subject-payload-is-arranged
kind: story
status: draft
title: A set guard fed by a subject payload is arranged
summary: 'ESS-SYNTH-010 refuses a binding whose invoked set guard reads an input the trigger fills from {subject: field}, although arrangeable (#705)'
tags:
- ess-batch-7
- ess-next
refs:
- provider: github
  reference: beyond10x/ess#705
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

- Over the toy repro of #705 as written, synthesis writes the binding's flow, delivery and on-failure
  scenarios. A `{subject: field}` payload source whose field was written from an arranging command's
  input is resolved the way an input is. Each scenario passes the interpreted target and fails a
  target that skips the invoked command's move.
- A `{subject: field}` source whose field no arranging command writes from its input keeps
  ESS-SYNTH-010 with its current text.
- No existing row of the synthesis bytes tables moves.

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

beyond10x/ess#705, reported 2026-10-29 by a downstream session on ESS 0.77.0. Re-checked with
`ess 0.77.0` against the toy repro in the issue. As written: 10 scenarios and 3 refusals. With the
toggle: 13 scenarios and 0 refusals.

A binding's invoked command has a row-set guard (`when_related: {entity: ..., where: {all:
[member_id == input.member_id, ...]}, exists: true}`) whose input the trigger's event fills from
`{subject: member_id}`. Synthesis refuses the binding's flow, delivery and on-failure scenarios with
ESS-SYNTH-010, "no row it selects can be arranged before the trigger for the value the event fills",
whose help asks the trigger to copy the input from an input of its own. The case can be arranged:
the subject's `member_id` was written by a creating command from its own input, and the subject is a
row that the arrangement itself created.
