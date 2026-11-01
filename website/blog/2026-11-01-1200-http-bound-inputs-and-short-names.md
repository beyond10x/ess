---
title: "0.80 — HTTP-bound inputs and short names in generated types, and input-type rules in Entity Runtime"
description: >
  0.80.0 adds no format: ess generate types takes --http, so a command's input type is the request
  body its bound OpenAPI document publishes, --names short combines with --all-commands and
  --all-views, and Entity Runtime lowering keeps an unstored input type's rules.
slug: http-bound-inputs-and-short-names
tags: [release, ess]
date: 2026-11-01T12:00:00+01:00
release_tag: "0.80.0"
---

0.80.0 adds no source, scenario or suite format. It changes what `ess generate types` writes under
two flags, what Entity Runtime lowering does with a command input's type, and how synthesis decides
an identity guard. The [changelog](https://github.com/beyond10x/ess/blob/main/CHANGELOG.md) lists
every change; this note covers each of them.

{/* truncate */}

## Generated types for an HTTP binding

- `ess generate types` takes `--http <binding>` (and `--refines`), compiled as
  `ess generate --kind openapi --http` compiles it (beyond10x/ess#656).
- Under it each `<command>.input` is the request body the bound OpenAPI document publishes, without
  the fields its route carries in the path, so the Rust, Go and TypeScript types decode that body.
- A command whose route publishes no body (encoded `query`, or every field in the path) has no input
  root: `--root <command>.input` is refused as `unpublished_body`, naming the command and why, and
  `--all-commands` leaves it out.
- Responses and view rows are unchanged, and without `--http` every generated file keeps its bytes.

## Short names for every command and view

- `ess generate types --names short` combines with `--all-commands` and `--all-views`
  (beyond10x/ess#409).
- A command's input or response and a view's row are named by their owner's last segment and the
  member: `demo.notes.Create.input` is `CreateInput` and `demo.notes.ByOwner.row` is `ByOwnerRow`.
  Before, every input shortened to `Input` and every row to `Row`, and two commands or views were
  refused as `short_name_collision`.
- Declared type and event short names are unchanged, and two commands of one name in two domains
  are still refused, naming both.

## Input-type rules in Entity Runtime

- Entity Runtime lowering no longer drops the `invariants:` and `prefix:` of a command input's type
  unless every accepting branch stores the input in a field of that type.
- An update holds them as operation preconditions over `$args.input.<field>`, lowered as a guard is,
  so an ordering over a `Timestamp` becomes `before`/`after`.
- A creation, which has no precondition in entity-core, and an ordering over text, which entity-core
  cannot decide, are refused as `InputInvariantUnsupported`, naming the input.

## Synthesis fix

- A `when:` guard on the identity input of a row a branch is sent for, below a non-creating
  `unknown_instance:` or on the subject the branch acts on, is decided on the identity of the row
  the scenario arranges (beyond10x/ess#645, beyond10x/ess#693).
- The row's creator is steered onto an identity that keeps the branch; where none does, the branch
  is refused as `ESS-SYNTH-003` naming the guard. Before, the scenario was sent a row of another
  identity and the target answered another branch.
- A related row's boundary run that would create a second row under such an identity gives way to
  the next branch answering there, where the whole scenario was refused as `ESS-SYNTH-001`.
