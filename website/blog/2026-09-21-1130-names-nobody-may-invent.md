---
title: "0.22–0.27 — what a binding promises, and the names nobody may invent"
description: >
  A binding declares one delivery attempt. A branch turns on the state a subject is already in, or
  says the field it owns holds nothing. A component declares its settings and the runtime slots are
  derived. An author names the identifier a code emitter spells, and the wire name a variant
  carries. Source formats ess/3, ess/4 and ess/5.
slug: names-nobody-may-invent
tags: [release, ess]
date: 2026-09-21T11:30:00+02:00
release_tag: "0.27.0"
release_commit: 7ce7fefbe0182d8857f80ed971ec820eda03a198
---

Six releases with one shape between them: each takes a fact that a generator was inferring, or a
consumer was assuming, and makes it something an author states. Three of them cost a source format
version — `ess/3`, `ess/4` and `ess/5` — and
[the version history](/ess/docs/reference/spec-versions) says which.

{/* truncate */}

## One attempt, and no obligation on the handler

`delivery: at_most_once` states that a binding is tried once. Loss stays possible, and **no
idempotency obligation is imposed on the handler**, which is the point: the existing default made
every consumer responsible for redelivery it might never see.

OpenAPI requires `Idempotency-Key` only for `at_least_once` commands, and conformance records
`BindingGap::DeliverySingleAttempt` instead of synthesizing a redelivery scenario that the binding
has just said will not happen. The format stays `ess/1` — this is a declaration the existing
vocabulary could carry.

## The state a subject is already in

`when_subject_state` in `ess/3` combines a declared held lifecycle state with input guards, so
**equal input applied to two different existing states is distinguished**. Without it, two branches
that differ only in where the subject started were one branch as far as the model could see.

`ess/3` also brings bounded binding accessors, binding-local `selection_inputs` with ordered
`selections`, periodic host causes and clock-reading attachments. `ess/4` declares error wire names
without merging semantic identities, and fills emitted event payloads from typed command response
fields. New deltas use `ess-diff/3` and `ess-diff/4`.

## Settings, declared once

`settings:` states each of a component's configuration inputs once, typed from the specification's
own `types:`. `ess specify runtime compile` **derives** the `ess-runtime/1` config and secret slots
from it, and refuses a hand-authored slot, or two settings binding one environment variable twice.

Absence is stated with `Optional<...>` and nothing else — a convention about an empty string is a
second reading of the same bytes.

## Three names a generator must not choose

**`naming: { code: ... }`** settles a collision the emitter cannot. A target without dotted type
names flattens `X.State` and an authored view `XState` to one identifier, and only the author knows
which should move. Only code emitters read it, so wire names, document schemas and qualified names
are untouched, and setting one **cannot break a deployed consumer**. The source format stays `ess/4`.

**`sets: {field: {cleared: true}}`** states absence. Before it a branch either left the field alone,
wrote a literal that synthesis dropped, or wrote an empty string — which is a different reading. The
field's type must be `Optional<...>` and the source must be an entity; an event payload is refused,
because an undetermined field is not the outcome's to set.

**A variant's own wire spelling**, in `ess/5`. `TypeBody::Enum` carried `Vec<String>`, so naming was
reachable from a domain, a command, an event, an error, a view and a field — and not from a variant.
A variant whose wire form is not derivable from its name could not be declared at all. The case that
asked for it: one command over three route shapes, where one variant's path suffix is empty and the
others are not. Without a declared spelling the alternatives were splitting the command to satisfy a
path, or writing the table again in every target.

A variant is now authored either as a bare name or as a mapping carrying `wire`, `display`, `summary`
and `code`. An authored empty string is a spelling rather than an absent one. Every specification and
IR document written before this keeps its bytes, and `ess-diff/5` adds three `TypeChange` cases so
that **a moved spelling no longer returns an empty delta** — which is what it did while the variant
set and the variant order both stayed the same.
