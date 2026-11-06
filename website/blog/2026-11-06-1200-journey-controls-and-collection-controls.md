---
title: "0.88 — Journeys reach every control, and a collection draws the controls it declares"
description: >
  0.88.0 gives `ess ui test` a step for every control a page declares, makes the terminal and the
  generated React app send the same reorder and toggle payloads, draws the React collection's sort,
  expand, reorder, selection and column controls, and creates a further instance with the
  specification's input examples.
slug: journey-controls-and-collection-controls
tags: [release, ess]
date: 2026-11-06T12:00:00+01:00
release_tag: "0.88.0"
---

0.88.0 changes the `ess ui test` journey runner, the terminal renderer, the generated React runtime
and conformance synthesis. No source format moves, and a UI document renders as before unless it
declares the collection controls the React runtime now draws. The gaps came from an `ess ui test`
audit ([#722](https://github.com/beyond10x/ess/issues/722)), an adopter review of the React runtime
([#723](https://github.com/beyond10x/ess/issues/723)) and a synthesis report
([#736](https://github.com/beyond10x/ess/issues/736)). The
[changelog](https://github.com/beyond10x/ess/blob/main/CHANGELOG.md) lists every change.

{/* truncate */}

## A journey reaches every control a page declares

- `mark` adds a row to its collection's selection, so a bulk action and its confirm run with `act`.
- `move: {at, to: up | down}` moves a row of a collection with a `reorder`.
- `upload: {at, file}` runs an upload action with a file named relative to the test file.
- `expect_export: {view, as?, params?}` holds when an export read the view with those params.
- `open` takes `state`, the page state a page keeps in its URL; a plain `open` starts without it.
- `act` at a section form submits it, and at a `sortable` column sorts as its header does.
- `choose` sets a form toggle, picks in a multiple choice field, a section-level choice and a filter
  bar's time window, and finds an option by its label however long it is.
- `select` at a row's `expand` shows its detail.

## The terminal and the React app send the same thing

- A reorder sends `{id, delta}` with `delta` -1 or 1 in both targets.
- A toggle nobody touched is sent as its declared default, `false` when none is declared.
- An upload is read as the browser reads it: a leading byte order mark is dropped, and text that is
  not UTF-8 is decoded with replacement characters.
- In the terminal, `h`, `l` and `i` stay the view's own keys, so a filter-bar or form action never
  takes them.

## A collection draws the controls it declares

- A client sort orders numbers by value, and mixed columns in one total order.
- A wide table scrolls inside its frame, and its row actions take a column headed `Actions`.
- Cards and lists draw the declared sort, the select-all and each row's actions.
- `expand` draws an expand control, `reorder` a drag handle that moves a row one place within its
  group, `selection: multiple` a select-all and `bulk_actions` a bulk menu.
- Item and toggle columns get their own column, and a default column subset draws only its columns.

## Synthesis

- A second, suffixed instance is created with the input examples the specification gives, and a
  value a branch decision reads stays told apart between instances.

## CI

- The go-task install retries its download, so a transient release download error no longer fails
  a test shard.
