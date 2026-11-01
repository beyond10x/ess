---
format: aep.planning-md/3
id: story:an-identity-guard-below-unknown-instance-is-decided-on-the-row-sent
kind: story
status: draft
title: An identity guard below unknown_instance is decided on the row sent
summary: 'Follow-up of U11 (#645, #693): a branch''s own guard on a subject-addressed identity, and a related row three moves away, still get the wrong row or a wrong refusal'
tags:
- ess-next
refs:
- provider: github
  reference: beyond10x/ess#645
- provider: github
  reference: beyond10x/ess#693
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

- A branch's own `when:` guard on the identity input of a subject-addressed row is decided on the
  identity of the row arranged. The creator is steered onto that identity, as `on_identity` does for
  #693. Where no row can meet the guard, the branch is refused with ESS-SYNTH-003, naming the guard.
- With a String identity guard and a related row three moves away, the guarded branch is written and
  passes on the interpreted target, or it is refused with a reason that names the guard. It is never
  refused as a repeated identity when a row of another identity witnesses it.
- Each red case of the batch-3 U11 adversary pass either passes on the interpreted target or is
  refused as above. No row of the synthesis bytes tables moves.

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

The batch-3 adversary pass over U11 (#645, #693) found shapes still outside it. The models are kept with
the red cases of that pass.

- An accepting branch below `unknown_instance:` whose own `when:` reads the identity input (for example
  `note_id == "keep"`, or an enum `note_id == b`) is sent a stored row of another identity. The
  interpreted target answers the wrong branch. It is the same on 0.77.0.
- Below an accepting guard `note_id != "keep"`, the deleting branch is sent a note opened as
  `"note_id"`, so the guarded branch answers. This is the #693 bug class on a subject-addressed row.
  It is the same on 0.77.0.
- With a String identity guard and a related row three moves away, the guarded branch is refused
  ESS-SYNTH-001 ("creates a second row under a value a row it created still holds"). A row of another
  identity would witness it, as the same model shows with the guard on a different input. On 0.77.0
  a failing scenario was written instead. This is introduced by U11, but it moves a wrong scenario to
  a refusal.
