---
title: "0.89 — A route takes a bare body, a literal segment wins, and an each write takes repeated identities"
description: >
  0.89.0 lets a command route take one input field as its whole request body, lets a literal path
  segment take precedence over a variable one, adds format ess/35, in which an each write may name
  one row more than once, and stamps each generated file with its own slice's digest only. UI
  documents gain a contains search, tag suggestions, toggle checks and a header help node.
slug: bare-bodies-literal-routes-and-repeated-identities
tags: [release, ess]
date: 2026-11-06T22:00:00+01:00
release_tag: "0.89.0"
---

0.89.0 adds one source format, `ess/35`, and moves the generated-output checkpoint to
`ess-output-state/4`, which releases before 0.89.0 refuse. The HTTP and specification changes come
from a downstream specification's report ([#748](https://github.com/beyond10x/ess/issues/748)). A
model that uses none of the new spellings keeps its IR bytes. The
[changelog](https://github.com/beyond10x/ess/blob/main/CHANGELOG.md) lists every change.

{/* truncate */}

## Before you upgrade

- `ess generate` writes `.ess-output/state.json` as `ess-output-state/4`. An `ess` older than
  0.89.0 refuses that checkpoint, so every `ess` that generates into or checks such a root must be
  0.89.0 or later. 0.89.0 still reads `/1` to `/3`.
- A generated file no longer carries the `model digest` line, and the JSON Schema, OpenAPI and
  AsyncAPI documents no longer carry `source_digest`. A file committed by an earlier release is
  reported by `ess generate --check` as differing until the root is generated again.

## HTTP bindings

- A command route may take one input field as its whole request body,
  `request: {encoding: json, body: input.<field>}`, with every other input field in the path.
  OpenAPI publishes the field's own schema as the body, the generated Rust and Go servers read the
  body as that field, and the React and terminal renderers send the field's value alone.
- Two nested route templates of one component and method are admitted when one holds the literal
  at every segment where they differ, and the more specific answers:
  `/boards/{board_id}/cards/priority` beside `/boards/{board_id}/cards/{card_id}`. Crossed
  templates stay refused, and the refusal names a path both match.

## Repeated identities in list order

- From `ess/35` an `affects:` entry over an input list may take a list naming one row more than
  once, `each: {in: input.<list>, as: <name>, repeated: last_wins}`, with no `distinct:`.
- Its elements apply in list order: the last upsert's values stay, a move repeat takes its
  transition only from a state in its `from`, and a delete repeat is skipped.
- The interpreted target and synthesis apply it; generated Rust and Go keep the command an
  obligation, and `ess verify diff` reports declaring it as widening.

## Per-file digest stamps

- A generated file's stamp carries its own `contract digest` only, so a change to one entity
  rewrites the files whose model slice holds it and leaves every other file byte for byte.
- The whole-model digest is recorded once per generation, as `model` on each projection's owner in
  `.ess-output/state.json`.

## UI documents

- A client row filter may search text, `row.x contains state.search`, so a list the server sends
  whole draws its search box again.
- A tags field takes `suggests: {reads, value}` and offers the values of a read.
- `ess-ui-test/1` checks a form toggle's state with `expect: {at: …, checked: true}`, and a page
  header's help is a node a test opens and reads.
- In the React target, an overlay opened from inside another stacks on it and keeps its draft, and
  an export saves the bytes the view's server answers.
- In the terminal, a graph node action's confirm sits under its row, as the React project draws it.
