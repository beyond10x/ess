---
format: aep.planning-md/3
id: story:wrong-state-branch-beside-a-subject-guard-is-witnessed
kind: story
status: draft
title: A wrong-state branch beside a subject guard is witnessed
summary: 'Synthesis''s wrong-state family skips a command that also has a subject guard: StateBesideWrongState/not-held gets no scenario and no refusal at ess/27 and ess/30, so the suite is silent on a branch the explorer reaches; explore_guards_first.rs pins the gap as expected. On origin/main since 9572af9ba. (adversary pass 2026-10-22, F2).'
relations:
- serves: vision:O2
revision: 1
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

*The conditions under which this is done, each one something a person or a check can observe. "Works
correctly" is not one of them.*

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
