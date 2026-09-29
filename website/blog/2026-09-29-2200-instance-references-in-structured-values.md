---
title: "0.43 — instance references inside structured values, one refusal order, and unstated external answers"
description: >
  0.43.0 lets an authored step name a created instance inside a list, a map or a struct, puts
  every command's refusals in one declared order, refuses an authored claim that holds only on an
  external answer the act does not state, and lets synthesis witness guards it refused before.
slug: instance-references-in-structured-values
tags: [release, ess]
date: 2026-09-29T22:00:00+02:00
release_tag: "0.43.0"
release_commit: 61295933e02664ff9b2ec894160723b03501c0db
---

0.43.0 adds the suite formats `ess-conformance/32` and `ess-conformance/33`, described in
[the version history](/ess/docs/reference/spec-versions). A suite that does not use what they add
keeps its earlier format, and committed suites regenerate byte for byte.

{/* truncate */}

## Instance references inside structured values

`{$instance: …}` may now sit inside a list, a map value, a struct member or a union payload of an
authored step, at any depth, wherever the declared type at that position is the instance's
identity type. `ring_sequence: [{$instance: a}, {$instance: b}]` is accepted for a list of
identities, so a command taking several identities can be authored. A reference at a position of
another type is refused as `ESS-AUTHOR-022`, naming the position, such as `labels[1]` or
`pair.note`. Inside an event payload or an error it is refused as `ESS-AUTHOR-021`.

A suite that carries such a value is written as `ess-conformance/32`, or `/33` with coverage. The
Rust runner resolves it; Go and TypeScript generation refuse such a suite and name the Rust runner.

## An external answer the act does not state

An authored act that names an `external:` branch under `outcome:` now tells the target which answer
to give for that one call, as a synthesized scenario does. Before, no target could satisfy the act
reliably.

An act whose error, response or event claims hold only on an `external:` answer it does not state
is refused with `ESS-AUTHOR-037`, which names every such branch as `command/branch`. That includes
answers reached through a binding's call to another command, which an act cannot state; those
claims are left to synthesis.

## One order for refusals

A refusal guarded only by the input may sit beside branches that depend on the record's held
state, so "this value is refused whatever the record's state" can be written on a command that acts
on an existing record. One order now answers every command: a related row's existence checks;
then refusals guarded by the input, first declared first; then the existence of the addressed row;
then the held state; then accepting and external branches in declaration order. The interpreter,
synthesis and the seeded explorer in the Go and TypeScript packages all follow it, as Entity
Runtime does, so a target that refuses a bad input in any state no longer disagrees with the
explorer.

## Synthesis

Synthesis now witnesses several guards it refused before with `ESS-SYNTH-003`:

- a guard over an optional field the creating command leaves absent, read as absent until a later
  command sets it;
- `exists` and `forall` over a stored map's values, and over a stored list compared with an input,
  on rows that hold several entries, so a target reading only the first or last entry fails;
- invariants over the members of a struct input, which every input synthesis sends now meets;
- equality between one input and a member of another.
