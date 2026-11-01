---
title: "0.81 — Nullable Binary64 fields in generated types, name aliases in the schema, and synthesis that follows bindings"
description: >
  0.81.0 adds no format: ess generate types realizes a nullable optional Binary64 field and refuses
  the response root of a command without a response by name, the published schema admits the id:
  and component: name aliases, and synthesis sends related-row and wrong-state scenarios to rows
  that keep their branch.
slug: nullable-binary64-and-binding-aware-synthesis
tags: [release, ess]
date: 2026-11-01T16:00:00+01:00
release_tag: "0.81.0"
---

0.81.0 adds no source, scenario or suite format, and every change in it is a fix. Two are in
`ess generate types`, one is in the published document schema, and three are in how synthesis picks
the rows a scenario is sent to. The [changelog](https://github.com/beyond10x/ess/blob/main/CHANGELOG.md)
lists every change; this note covers each of them.

{/* truncate */}

## Generated types

- An `Optional<Binary64>` field declared `presence: null_when_absent` or `omitted_or_null` is
  realized as Rust and Go, in a declared struct and in a command's input or response root. Its
  Binary64 location is the number member of the field's `anyOf [number, null]`; before, it was the
  property, and realization was refused as `model_binary64_layout`.
- The TypeScript report names its `model_binary64` obligation at that member.
- `--root <command>.response` on a command that declares no response is refused as `unknown_root`,
  naming the command and saying it declares no response, as `--all-commands` leaves it out. Before,
  it generated an empty object type.

## Name aliases in the published schema

- `schemas/generated/ess.schema.json` admits a binding's name written `id:` and a component's
  written `component:`, the aliases the parser already reads, each with the same charset as `name:`.
  Before, it refused every document that spelt a name that way.
- Writing both spellings, or neither, is refused, as the parser refuses it. Parsing is unchanged.

## Synthesis

- A `when_related:` branch is no longer sent for a related row that only a step setting off a
  binding selects, where that binding's command writes a field or the state the guard reads, or
  removes the row. Such a row is passed over for one no binding rewrites; where none is found, the
  branch is refused as `ESS-SYNTH-003` naming the step, the binding and the command it invokes.
- A `when_related:` command whose input is chosen for an external branch, where the precedence plan
  reads the related row's `exists: false` before the addressed row's existence or held state, sends
  its unknown-identity scenario and its held-state refusals beside a present related row. Before, it
  sent a reference no row carries and the target answered `exists: false` instead of the refusal.
- A model with such a command and no `wrong_state:` moves its suite bytes: its held-state refusals
  now name a present row, so the held state is what refuses them. Every other model keeps its bytes.
- A wrong-state scenario of a command reading stored fields is sent to a row its moving branch's own
  `when_subject:` holds on, and to a row holding a value in every field a sibling's stored guard
  reads, so no absent field stops selection before the wrong-state answer. Where no reachable row
  serves, the scenario is refused as `ESS-SYNTH-003` naming the guard.
- A model whose bindings write nothing a related guard reads keeps its suite bytes.
