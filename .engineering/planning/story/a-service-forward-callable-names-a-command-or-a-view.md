---
format: aep.planning-md/3
id: story:a-service-forward-callable-names-a-command-or-a-view
kind: story
status: draft
title: A service_forward callable names a command or a view
summary: ess-cli/2 service_forward needs a hand-declared struct per callable; a command input or view params and rows cannot be named (0.77.0)
tags:
- ess-batch-4
- ess-next
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

- A `service_forward` callable may name a command, and takes the command's input as the specification
  declares it, or a view, and takes the view's parameters and answers its rows as declared. No
  hand-declared struct is needed.
- Over a toy component with commands and views served through `ess-http/1`, `ess specify cli`
  projects every operation, and the generated CLI forwards each one to the service. A test asserts
  this end to end.
- An existing `ess-cli/2` document that declares its own structs keeps its projection bytes. Whether
  the new form is additive to `ess-cli/2` is decided in the fit review (D135).

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

An `ess-cli/2` `service_forward` callable needs a hand-declared struct for every command and view
it forwards. Its `input` cannot name a command or a view. `ess specify cli` refuses with "unresolved
input type `<sys>.<domain>.<Command>`", "CLI input must reference a declared struct" and
"unresolved CLI type `<sys>.<domain>.<View>`". Over one component reached by the network, with 11
commands and 11 views served through `ess-http/1`, 19 of 22 operations were refused.
