---
format: aep.planning-md/3
id: story:the-mutation-audit-reports-the-sites-it-could-not-mutate
kind: story
status: draft
title: The mutation audit reports the sites it could not mutate
summary: transition-to, emit-drop and sets-drop leave common shapes without a live mutant, and the audit still exits 0 without saying how much it covered (#711)
tags:
- ess-batch-7
- ess-next
refs:
- provider: github
  reference: beyond10x/ess#711
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

- The mutation report and manifest list every site that a selected class excluded or left without a
  live mutant, with its reason, beside the stillborn mutants. The text summary gives a coverage
  count: sites with a scored mutant out of sites considered.
- A write on a creating branch, or to an `Optional` field, that no class mutates gets a live mutant
  where one exists: sets-retarget to a literal, or sets-drop of an `Optional` field to absent. A
  transition-to on a linear lifecycle retargets to a state the model admits, or is listed as
  unavailable.
- Over the toy spec of #711, the report names each such site.
- The default exit status is unchanged (D135). An opt-in flag makes a site without a live mutant
  block exit 0, as an unavailable emit-swap site does.
- Today's killed and survived mutant ids are unchanged.

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

beyond10x/ess#711, a mutation-audit gap reported 2026-10-30 by a downstream session on ESS 0.77.0.
Re-checked with `ess 0.77.0` on a toy linear lifecycle:
- transition-to: 1 mutant, 1 stillborn (ESS-ENTITY-011).
- emit-drop: 3 mutants, 3 stillborn (ESS-COMMAND-007).
- sets-drop: ESS-MUTATE-003, with `creates_row` and `optional_field` notes on stderr only.
- The full audit exits 0 with 6 of 12 mutants stillborn.

The operators behave as documented. The gap is what the audit reports. Stillborn mutants and
excluded `sets` sites do not count against the result, and the excluded sites are absent from the
report and manifest. An emit-swap site with no alternative, by contrast, is `unavailable` and blocks
exit 0.
