---
format: aep.planning-md/3
id: story:the-generated-rust-server-refuses-what-its-openapi-refuses
kind: story
status: draft
title: The generated Rust server refuses what its OpenAPI refuses
summary: The Rust server admits any string as a Timestamp and ignores undeclared JSON members that its served OpenAPI refuses (#707)
tags:
- ess-batch-4
- ess-next
refs:
- provider: github
  reference: beyond10x/ess#707
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

- Over the toy specification of #707, the generated Rust server answers 400 at decode, before any
  store write, for each of the following. The answer names the path of the offending member.
  - A `Timestamp` member that is not an RFC 3339 date-time, on every command that takes one, nested
    `Optional`, `List` and struct positions included.
  - A member that a JSON object's schema does not declare, at the top level of a command input and
    inside a nested struct or list element.
- Every body the served OpenAPI admits is still admitted. A test sends the issue's valid bodies and
  their stored rows read back unchanged.
- The answer for each rejected body is the same as what the served OpenAPI predicts. A test checks
  the generated server against its own `openapi.json` for these bodies, so the two cannot drift
  apart again.
- The adopter replay at the release shows whether any adopter's generated server or suite sends such
  a body.
  - If none does, the refusal is the default and CHANGELOG lists it under Changed.
  - Otherwise, the unit ships the refusal behind a binding opt-in (D135) and records the adopter as
    an open question.

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

beyond10x/ess#707, a defect reported 2026-10-29 by a downstream session on ESS 0.77.0. Re-checked with
`ess 0.77.0` on the toy specification in the issue: `ess generate synthesize --target rust --layout
crate --http`, built and run, with requests sent through curl.

The generated Rust server decodes JSON request bodies more loosely than the OpenAPI document it serves:

- (a) A `Timestamp` (OpenAPI `format: date-time`) is read with `json::text_at` only. Any string is
  admitted, `""` and `"yesterday at noon"` included: 202, the row is stored, and the view answers the
  text back.
- (b) An object the OpenAPI declares `additionalProperties: false` ignores members it does not
  declare. This holds in a list element (`unti` beside `unit`: stored with `unit` absent) and at the
  top level of a command input. `request: {encoding: json, unknown: refuse}` on the route changes
  nothing. The only unknown-key check is `http::flat_object`, which serves `form` and `query`.

A misspelt required member is refused, but only because the required member is then missing.
Where a bad timestamp sorts in an ordered view was not observed: `order_by` over a `Timestamp` is an
obligation in 0.77.0.
