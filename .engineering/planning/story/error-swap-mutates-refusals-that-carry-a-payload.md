---
format: aep.planning-md/3
id: story:error-swap-mutates-refusals-that-carry-a-payload
kind: story
status: draft
title: error-swap mutates refusals that carry a payload
summary: error-swap keeps the original error's payload mapping, so every refusal with a payload is stillborn and never audited (#709)
tags:
- ess-batch-7
- ess-next
refs:
- provider: github
  reference: beyond10x/ess#709
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

- error-swap on an outcome with a `payload:` writes a mutant the model admits.
  - The payload is re-keyed to the swapped-in error where its fields can be filled the same way.
  - Otherwise the payload is dropped, where the swapped-in error has no fields.
  - Otherwise the site is listed as unavailable with a named reason, not as stillborn.
- Over the toy spec of #709, both payload-carrying refusals get a live mutant, and the synthesized
  suite kills it.
- An `unknown_instance` refusal gets a mutant that keeps it free of effects.
- Payload-less sites keep their mutant ids and verdicts.

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

beyond10x/ess#709, a mutation-audit gap reported 2026-10-30 by a downstream session on ESS 0.77.0.
Re-checked with `ess 0.77.0` on a toy spec: `mutate --target interpreted --class error-swap` gives 3
mutants, 1 killed and 2 stillborn, and exits 0.

error-swap replaces an outcome's `error:` but keeps its `payload:`, which is keyed by the original
error. The model then refuses the mutant: ESS-COMMAND-001 on a guarded or `wrong_state` refusal,
ESS-COMMAND-004 on an `unknown_instance` refusal. A hand-written mutant with the stale payload
removed validates and synthesizes. So every refusal that says what fills its error's fields has no
live error-swap mutant, and only refusals without a payload are audited.
