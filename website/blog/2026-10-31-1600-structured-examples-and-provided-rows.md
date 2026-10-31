---
title: "0.78 — Structured input examples, rows a deployment provides, and lifecycle summaries"
description: >
  0.78.0 adds source format ess/33: a struct, map or Json input may carry an example, an entity may
  declare that the deployment provides its rows, and lifecycle transitions and states may carry a
  summary.
slug: structured-examples-and-provided-rows
tags: [release, ess]
date: 2026-10-31T16:00:00+01:00
release_tag: "0.78.0"
---

0.78.0 adds source format `ess/33`. A model that does not use its new keys keeps its meaning, its IR
bytes and its compiled digest. The [changelog](https://github.com/beyond10x/ess/blob/main/CHANGELOG.md)
lists every change; this note covers what `ess/33` adds and the fixes that came with it.

{/* truncate */}

## Structured input examples

- An input typed as a struct (open or closed), a `Map<…>` or `Json`, through any newtype or
  `Optional`, may carry an `example:`. Synthesis sends it at the plain instance; a struct example
  reaches synthesis through its declared members, so a guard reading one keeps the scenario that
  refutes it.
- An open struct's example keeps the members it does not declare, and the interpreted target now
  admits those members in a command input at every format.

## Rows a deployment provides

- An entity may declare `provided: true`: the deployment holds its rows, and no command of the
  specification need create one.
- Where a `when_related: {via: input.<field>}` guard reads such a row and no declared outcome
  creates it, the scenario resolves an entity fixture `{name, type, entity, state}` before it runs
  (suite formats `ess-conformance/60` and `/61`). The interpreted target creates the row; the Go and
  TypeScript runtimes pass the entity and state to their fixture provider.
- `fixture_inputs:` is refused over every input a guard reads, including the identity a related-row
  guard reads.

## Lifecycle summaries

- A transition may carry a one-line `summary:`, and a state may carry one in its mapping form,
  `{name, wire?, summary?}`. `ess verify diff` reports a moved transition summary as
  `transition-summary-changed` (`ess-diff/22`), compatible.
- A flat `summary:`, `wire:`, `display:` or `code:` on a named type is refused with a message naming
  `naming: {summary: …}`.
