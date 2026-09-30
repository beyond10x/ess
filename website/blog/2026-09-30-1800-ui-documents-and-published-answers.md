---
title: "0.47 — UI documents, and generated servers that answer with what a command published"
description: >
  0.47.0 adds ess-ui/1, a renderer-neutral UI document that loads, checks, runs in the terminal
  and generates a React project, and makes the generated Rust and Go servers answer with the
  events a command published, deliver each binding on its own, and keep request headers.
slug: ui-documents-and-published-answers
tags: [release, ess]
date: 2026-09-30T18:00:00+02:00
release_tag: "0.47.0"
---

0.47.0 adds two formats, `ess-ui/1` and `ess-ui-check/1`, described in
[the formats reference](/ess/docs/reference/formats). Specifications, suites and deltas keep their
earlier formats.

{/* truncate */}

## UI documents

An `ess-ui/1` document describes an application's shells, pages, sections, widgets, reads,
commands, live channels and where each piece of state is held, without naming a renderer. Its
schema is data, `schemas/ui/ess-ui.schema.yaml`, and the [reference](/ess/docs/reference/ess-ui)
is rendered from it by `ess ui docs`.

- `ess ui load --path <document>` loads a document and prints how many pages and addressed nodes
  it holds, or names the node that refuses it.
- `ess ui check --path <document> [--model <specification>]` runs every check the schema declares
  and, with a model, checks that each view, command and event the document names exists and that
  some actor may read each section. Each finding names its canonical node path; any error exits 1.
- `ess ui run --tui --path <document>` runs the document in the terminal, answering reads from
  its fixtures and playing live events from a fixture script.
- `ess generate ui --target react` writes a Vite + React + TypeScript project.

`examples/partner-portal/` is a complete document to start from.

## Generated servers answer with what a command published

A served command's answer now lists `published`: the events it emitted, in emit order, each as
`{event, payload}`. The OpenAPI contract types the list item by item. `SystemEvent::name` and
`wire::encode_system_event` name and encode any system event, so a runner needs no hand-kept
table. `Request.headers` keeps the request's headers in arrival order; a shell that builds a
`Request` literal must add the field.

Delivery is tracked per binding. A binding whose obligation is unmet holds the event in its own
list and tries again on a later pump; every other binding receives the event once, and an
`at_most_once` binding is never delivered twice. A command whose effect is committed but whose
events could not be delivered is answered 501, now declared on every command, and must not be
retried. The generated Go server serialises dispatch, so concurrent requests are safe.

## Fixed

`ess verify diff` no longer reports `unclassified-changed` beside an added or regrouped aggregate
view (beyond10x/ess#256).
