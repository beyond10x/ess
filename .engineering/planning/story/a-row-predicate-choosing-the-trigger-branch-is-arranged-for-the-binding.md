---
format: aep.planning-md/3
id: story:a-row-predicate-choosing-the-trigger-branch-is-arranged-for-the-binding
kind: story
status: draft
title: A row predicate choosing the trigger branch is arranged for the binding
summary: ESS-SYNTH-010 refuses flow, delivery and a policy refusal when one related row decides both the trigger and the invoked command; the refusal contradicts its help (#706)
tags:
- ess-batch-4
- ess-next
refs:
- provider: github
  reference: beyond10x/ess#706
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

- Over the toy repro of #706 as written, synthesis writes the binding's flow, delivery and
  `/refusal/not-eligible` scenarios. The one row is arranged eligible for the flow and not eligible
  for the refusal, and `/refusal/tier-missing` stays refused as today. Each scenario passes the
  interpreted target and fails a target that skips the invoked command or applies the wrong policy.
- Where the help's conditions do not hold, the refusal keeps its text. Where they hold, the refusal
  is never written.
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

beyond10x/ess#706, a defect reported 2026-10-29 by a downstream session on ESS 0.77.0. Re-checked
with `ess 0.77.0` against the toy repro in the issue. As written: 10 scenarios and 4 refusals. With
the toggle: 12 scenarios and 1 refusal, the correct `/refusal/tier-missing`.

The trigger `Refresh` chooses its emitting branch by a predicate on a related row
(`when_related: {via: input.tier_id, predicate: eligible == true}`). The invoked command `Issue`
reads the same row (`not-eligible` on `eligible == false`). Synthesis refuses the binding's flow
and delivery ("whose branch is decided by an input the event fills") and the
`/refusal/not-eligible` policy scenario. That refusal's help states the condition under which
synthesis arranges such a row itself: a creating command takes each field the predicate reads as
input, and the trigger copies the row's identity unconverted from an input of its own. Both
conditions hold, so the refusal contradicts its own help. One row decides both commands, so the
case can be arranged.
