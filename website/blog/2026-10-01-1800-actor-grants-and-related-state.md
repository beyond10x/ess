---
title: "0.49 — Served surfaces enforce actor grants, and a related row's state"
description: >
  0.49.0 makes generated servers refuse a caller the specification does not grant a command,
  with suites that witness the refusal, adds ess/20 for a related row's lifecycle state in a
  when_related guard, and closes six synthesis and diff gaps.
slug: actor-grants-and-related-state
tags: [release, ess]
date: 2026-10-01T18:00:00+02:00
release_tag: "0.49.0"
---

0.49.0 adds the source format `ess/20`, described in
[the format history](/ess/docs/reference/spec-versions). It changes the signature of generated
servers in a specification that declares actors, so a caller of `dispatch`, `handle` or `serve`
has one change to make.

{/* truncate */}

## Served surfaces enforce actor grants

In a specification that declares actors, the generated server's `dispatch` and `handle` (Rust)
and `dispatch` (Go) take the caller your realization authenticated the request as, and `serve` /
`Serve<Component>` take an `authenticate` function that returns it. `admit` (Rust) and `Admit`
(Go) expose the check on its own.

Before a command runs, a request with no caller, or with an actor the specification does not
grant the command, gets `403 {"refused": "not granted", "actor": <name or null>}`. A command no
actor is granted is refused to every caller; views are not grant-checked. The OpenAPI contract of
a served component declares the refusal on every command.

For served components the synthesized suite adds `<command>/grant/denied`, which borrows a
scenario whose send the command accepts and requires that the target's event log grows by nothing, and
`<command>/grant/admitted/<actor>` where a granted actor would otherwise never send its command.
Authored scenarios can expect the refusal with `refused: not_granted`. How a request proves its
actor stays the realization's job.

## A related row's state in `when_related`

From `ess/20`, a `when_related` predicate may read the related row's held lifecycle state as
`state`, as a `when_subject` predicate reads the subject's:
`when_related: {via: input.candidate, predicate: state != Accepted}`. Earlier formats refuse the
path with `unsupported_format_version` naming `ess/20`. Synthesis arranges a related row of an
entity already being arranged one level deep, in its initial state; suites for earlier formats
are unchanged.

## Synthesis and diff fixes

- A value copied from a related row through the input a `when_related` guard reads gets its
  success scenario, between other accepted rows, so a target copying from the wrong one fails.
- An aggregate view grouped by a key copied from a related row gets its `<view>/aggregate`
  scenario.
- A `when_related` guard over the `via` field of an `owns` relation is witnessed on both sides.
- A scenario sending a command that reads the caller also runs with the callers swapped and a
  fresh caller-supplied identity.
- A branch with a `when:` beside a `when_subject:` gets its scenarios when a later sibling shares
  its `when:`.
- `ess verify diff` reports a declaration added or removed on one side only as that addition or
  removal, not `unclassified-changed`.
