---
format: aep.planning-md/1
id: story:fuzz-the-specification-surface
kind: story
status: draft
title: Fuzz the specification surface
summary: Anything validate accepts, every projection and every synthesis target survives — asserted rather than hoped.
owner: ess
revision: 1
---
# Fuzz the specification surface

## Outcome

`ess validate` accepting a document means every projection and every synthesis target survives it.
Today that is a hope, and three defects in one afternoon were all the same shape.

## Why this, rather than fuzzing inputs

**Input generation already exists and is deliberately not random.**
`crates/verify/ess-conformance/src/witness.rs` states the rule it is built under: "Never generate an
arbitrary value and claim it satisfies an outcome predicate unless the generator can prove or
evaluate that it does." Candidates come from the guard's own literals, one either side, in a bounded
deterministic order. Randomising that would weaken it — an arbitrary value that happens to satisfy a
guard proves nothing about the guard.

The unguarded surface is the **document**, not the input. Three defects found while writing one
consumer specification, and every one of them is `validate` admitting something a later stage cannot
survive:

| defect | validate | downstream |
|---|---|---|
| a system-level `types:` entry | accepts | all three synthesis targets panic — `"…" is not a declaration this layout knows` |
| a binding mapping `T` into `Optional<T>` | accepts | emitted Rust does not compile, `E0308`, three occurrences |
| a specification with no `on_failure: retry` binding | accepts | the web bridge calls a `redeliver` the rust target does not emit |

The first was reproduced from a two-line addition to a valid document. A fuzzer over documents would
have found it in minutes, and none of the three needed a clever input.

## Acceptance

A `fuzz/` target that generates `ess/1` documents and asserts one property: **for every document
`ess validate` accepts, `compile`, all six `generate --kind`s and all four `synthesize --target`s
terminate without panicking.** A crash is a defect in whichever stage crashed, never in the
generator.

The three defects above become regression seeds, so the corpus starts with cases known to have been
wrong.

## What is already here

- `proptest 1.11.0` is a dev-dependency of `ess-compiler`, so property testing is not a new tool in
  this workspace.
- There is no `fuzz/` directory. `cargo-fuzz` and `arbitrary` are not dependencies anywhere.
- `sipx` carries a `fuzz/` directory, which is worth reading for the shape a b10x repository uses
  before inventing one here.

## What this is not

Not a replacement for the gate. `task check` proves the committed examples work; a fuzzer proves the
*unwritten* documents do not crash, which is the half no example can cover. And not a test of
whether a specification is any good — a document can be meaningless, survive every stage, and that
is a pass.
