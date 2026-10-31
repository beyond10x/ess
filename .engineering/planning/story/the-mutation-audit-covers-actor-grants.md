---
format: aep.planning-md/3
id: story:the-mutation-audit-covers-actor-grants
kind: story
status: draft
title: The mutation audit covers actor grants
summary: The mutate baseline drops the grant/denied scenarios and no class mutates may:, so a wrong grant table is invisible to the audit (#710)
tags:
- ess-batch-7
- ess-next
refs:
- provider: github
  reference: beyond10x/ess#710
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

- A grant mutation class exists:
  - grant-drop removes one command from one actor's `may:`;
  - grant-add adds one command an actor may not invoke, where a command the component accepts is
    missing from that actor's `may:`.
- The baseline keeps the `grant/denied` scenarios synthesize writes.
- Over the toy spec of #710, `mutate --target interpreted` kills the grant mutants. The baseline
  count equals the `synthesize` scenario count.
- A run with `--class` that excludes the grant classes keeps today's baseline and mutant ids. The
  manifest format moves to a new version only if it must name the new class.

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

beyond10x/ess#710, a mutation-audit gap reported 2026-10-30 by a downstream session on ESS 0.77.0.
Re-checked with `ess 0.77.0` on a toy spec:
- `synthesize` writes 2 scenarios, including `<command>/grant/denied`.
- The `mutate --emit` baseline holds 1 scenario, and the output does not mention the drop.
- None of the twelve classes edits an actor's `may:`.

The drop is deliberate (the comment above `synthesized` in `mutate.rs`, citing #265): grant scenarios
could never kill a mutant, because no class alters a grant. So the audit cannot detect a grant table
that is too wide or too narrow.
