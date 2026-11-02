---
title: "0.84 — Generated servers refuse what their OpenAPI refuses, and generated UIs call their binding's routes"
description: >
  0.84.0 adds no format: generated Rust servers refuse a malformed Timestamp and an undeclared JSON
  member at decode, authored success acts assert that no error came back, denied-grant scenarios
  reach ranked views and every ungranted actor, and a generated UI calls the routes its HTTP
  binding serves.
slug: generated-servers-refuse-what-their-openapi-refuses
tags: [release, ess]
date: 2026-11-03T09:00:00+01:00
release_tag: "0.84.0"
---

0.84.0 adds no source, scenario or suite format. It changes what generated servers admit, what an
authored success step checks, which denied-grant scenarios synthesis writes, and how generated UIs
reach a server. The [changelog](https://github.com/beyond10x/ess/blob/main/CHANGELOG.md) lists every
change; this note covers each of them.

{/* truncate */}

## Generated servers refuse what their OpenAPI refuses

- The generated Rust codecs refuse at decode, before anything is stored, a `Timestamp` that is not
  an RFC 3339 `date-time` and a member a closed JSON object does not declare. The server answers
  `400`, or the binding's `refusals.malformed`, naming the member's path
  ([#707](https://github.com/beyond10x/ess/issues/707)).
- Before, the server stored the text or dropped the member, while the OpenAPI document it serves
  already refused the body. A struct declaring `undeclared_fields: ignored` and a `Json` value stay
  open.
- The same decoders read the log, so `decode_system_event` refuses an envelope an older server
  wrote with a free-text `Timestamp` or an extra member in a closed struct.
- The generated Go server refuses a top-level JSON command member no input field names, on bound
  and derived routes, and reads a `Timestamp` with the same grammar as the Rust server.
- Known limit: an `Optional` member sent as `null` is still taken as absent by the generated Rust
  server, while its served OpenAPI refuses the body. It is tracked as a backlog story.

## An authored success act asserts no error

- An authored act whose `outcome:` names a branch that declares no error now also requires that no
  error came back. A target answering the success branch under its own name with an error fails the
  step; it used to pass on the outcome name alone.
- No suite format moves. A library caller assembling an authored suite by hand selects the format
  with `select_fresh_format_for` and runs it with `Runner::run_admitted`, as the CLI does.

## Denied-grant scenarios

- Synthesis writes `<command>/grant/denied` for a command acting on an existing row beside a view
  ranked by `order_by` ([#712](https://github.com/beyond10x/ess/issues/712)).
- `<view>/grant/read/denied` reads the view as every ungranted actor of the view's own domain, then
  as no actor. Before, it read one actor of any domain, so a target admitting an ungranted actor of
  the view's domain passed.

## Generated UIs call their binding's routes

- `ess generate ui` and `ess ui run --tui` take `--http <binding>`: the generated route table
  carries the method and path the `ess-http/1` binding declares, so the UI calls the routes the
  server generated for the same binding serves
  ([#713](https://github.com/beyond10x/ess/issues/713)).
- A command served on `get` is sent without a body, and React refuses a path value of `.` or `..`.

## Renderer fixes

- The terminal renderer addresses a row action's confirm by the view's row key, and a record's
  actions see the row the record shows.
- An account menu entry keeps its `sets` and `navigate` in React. `ess ui check` refuses an entry a
  renderer would drop, an entry that does not read as an action, and `sets` beside `does`,
  `navigate` or `opens`.
