---
format: aep.planning-md/3
id: story:generated-rust-and-go-readers-agree-on-leading-zeros
kind: story
status: draft
title: Generated Rust and Go readers agree on leading zeros
summary: The generated Rust JSON reader accepts 01 (and 01.0 since U15); Go refuses it; found by the batch-3 adversary
tags:
- ess-next
relations:
- informed_by: story:an-integer-literal-is-written-into-the-suite-as-an-integer
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

- The generated Rust and Go readers give the same answer for every number spelling. A leading zero
  (other than `0` itself, `0.x` or `0e..`) is refused by both with the same message. A test runs a
  shared spelling table through both servers.
- The fit review decides, under D135, whether refusing `01` in Rust breaks a client that relies on
  it, and records the opt-in or notice if it does.

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

Found by the batch-3 adversary pass over the integer decoders (U15). The generated Rust JSON reader
(`crates/generate/ess-synth/src/rust/json.rs`) accepts a number with a leading zero (`01`, and since
U15 also `01.0` and `007.0e0`), while the generated Go reader refuses all of them as not JSON. JSON
forbids leading zeros, so two servers generated from one model answer the same request differently.
The disagreement over `01` predates U15.
