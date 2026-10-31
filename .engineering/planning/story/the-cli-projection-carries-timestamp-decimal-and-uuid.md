---
format: aep.planning-md/3
id: story:the-cli-projection-carries-timestamp-decimal-and-uuid
kind: story
status: draft
title: The CLI projection carries Timestamp, Decimal and Uuid
summary: The CLI projection refuses Timestamp, Decimal and Uuid, including a Uuid identity newtype, as unsupported CLI primitives (0.77.0)
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

- The CLI projection and the generated clap CLI accept `Timestamp`, `Decimal` and `Uuid` inputs and
  outputs, and a `Uuid` identity newtype. Each is parsed from and printed in the canonical spelling
  that `ess-primitives` defines. An ill-formed argument is refused before any call is made.
- A toy model with one command and one view per primitive projects and generates, and its CLI
  round-trips each value through the service. A test asserts this.

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

The CLI projection refuses `Timestamp`, `Decimal` and `Uuid` with "unsupported CLI primitive". This
includes a `Uuid` used as an identity newtype, so a command addressed by such an identity cannot be
called from the generated CLI.
