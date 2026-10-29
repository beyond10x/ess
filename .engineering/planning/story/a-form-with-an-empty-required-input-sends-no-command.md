---
format: aep.planning-md/3
id: story:a-form-with-an-empty-required-input-sends-no-command
kind: story
status: draft
title: A form with an empty required input sends no command
summary: Neither renderer enforces required command inputs, and ess-ui-test has no step asserting that no command was sent (0.77.0)
tags:
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

- In both renderers, submitting a form whose required input is empty sends no command and marks the
  input as required.
- `ess-ui-test` gains a step that asserts no command was sent since a named point. Adding it is a
  format decision for the fit review: an additive step in `ess-ui-test/1`, or a new format version.
- A toy script submits the empty form, asserts no command was sent, fills the input and asserts the
  command with the field. It passes against both renderers.

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

Neither the terminal renderer nor the React target enforces a command's required inputs in a form.
Submitting with an empty required text input sends the command without that field. `ess-ui-test/1`
has no step that asserts that no command was sent, so a script cannot pin the refusal.
