---
format: aep.planning-md/1
id: story:fuzz-the-specification-surface
kind: story
status: draft
title: Fuzz the specification surface
summary: Anything validate accepts, every projection and every synthesis target survives — asserted rather than hoped.
owner: ess
relations:
- decomposes: epic:review-boundary-remediation
- serves: vision:O2
scope:
- confidence: cited
  path: fuzz
revision: 5
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

For every generated ess/1 document accepted by validation, the fuzz target verifies that compilation and every current generation and synthesis target terminate without panicking.

## Validation and corpus

Create the document-generation harness under `fuzz/` and retain the three defects above as mandatory regression seeds. Exercise all six current generate kinds (including the separate docs-ir path) and all four synthesis targets enumerated in the Scope section. A crash is a defect in the stage that crashed, not an invalid generator result; compilation of emitted programs is owned by the target-feasibility stories.

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

## Scope

Derived 2026-09-05 by `aep-drive:story-scoper`; scope follows the story's explicit acceptance — cited.

- **Primary surface:** `fuzz` — cited; the acceptance explicitly requires a new document-generation harness and three regression seeds here.
- **Files:** harness manifest, target, corpus, and usage instructions within the primary surface — inferred; their exact filenames are not specified.
- **Symbols exercised:** `Specification::validate`, `ess_compiler::compile`, `ess_gen::generators`, `ess_gen::docs::document`, and `ess_synth::synthesize_for` — cited; these existing public APIs supply the required pipeline stages and are evidence references rather than declared write surfaces.
- **Projection coverage:** docs, site, docs-ir, schema, openapi, and asyncapi — cited; docs-ir has a separate CLI dispatch branch and is absent from the five-entry generator registry.
- **Synthesis coverage:** Rust, Go, Web, and Clap — cited; these are the four existing `ess_synth::Target` variants.
- **Documents:** harness-local operating instructions only — inferred; the acceptance does not require public documentation changes.
- **Confidence:** medium — inferred; the new harness directory is explicit, but its build integration and concrete regression-seed documents remain unspecified.
- **Would collide with:** any unit writing the fuzz directory — inferred; dependencies on existing compiler, generator, and synthesis crates do not by themselves require edits to those crates.

## Remediation ownership and limits

Owns the F16 general document-fuzzing harness; no-panic is its property, not emitted-code compilability. Target-feasibility stories own concrete compile checks and review-consumer-coverage owns the consumer matrix. Preserve the three named regression seeds, reconstructing minimal local fixtures if their external originals are unavailable. Reassess root Cargo/Taskfile scope when choosing harness integration; this item is not in the first wave.