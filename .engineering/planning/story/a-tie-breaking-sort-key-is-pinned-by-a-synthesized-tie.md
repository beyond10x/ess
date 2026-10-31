---
format: aep.planning-md/3
id: story:a-tie-breaking-sort-key-is-pinned-by-a-synthesized-tie
kind: story
status: draft
title: A tie-breaking sort key is pinned by a synthesized tie
summary: 'order-flip on a view''s second order_by key always survives: synthesis never arranges a tie on the first key, and --emit takes no authored scenarios (#708)'
tags:
- ess-batch-7
- ess-next
refs:
- provider: github
  reference: beyond10x/ess#708
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

- For a view ranked by more than one key, synthesis arranges, for each later key, rows that tie on
  every earlier key and differ on that one.
- Over the toy spec of #708, `mutate --target interpreted --class order-flip` kills both mutants,
  and so does the `--emit`/`--collect` path, without authored scenarios.
- A view ranked by one key keeps its suite bytes. Any change to the synthesis bytes tables is named
  in the unit and re-pinned on purpose.

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

beyond10x/ess#708, a mutation-audit gap reported 2026-10-30 by a downstream session on ESS 0.77.0.
Re-checked with `ess 0.77.0` on a toy view with `order_by: [a asc, b asc]`.
`mutate --target interpreted --class order-flip` gives 2 mutants, 1 killed, 1 survived: the flip of
the second key survives.

The two rows that synthesis arranges for the `ranked` check always differ in the first key. An
authored scenario with a tie on the first key kills the mutant under `--target interpreted
--scenarios`. `--emit` refuses `--scenarios`, so an implementation audited through `--emit` and
`--collect` keeps one permanent survivor per tie-breaker key, and nothing in the specification can
answer it.
