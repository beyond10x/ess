---
format: aep.planning-md/3
id: story:generated-rust-and-go-behaviour-emits-repeated-and-conditional-emissions-as-the-model-declares-generated-docs
kind: story
status: draft
title: Generated Rust and Go behaviour emits repeated and conditional emissions as the model declares, generated docs state them, and the entity runtime refuses them by name
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:at-the-new-format-an-outcome-may-emit-one-event-several-times-in-order-each-with-its-own-payload-and-an
- depends_on: story:synthesis-and-the-interpreter-witness-every-emission-of-an-outcome-in-order-and-both-sides-of-a-conditional
revision: 2
---
## Need

After `ae9a-an-outcome-emits-repeated-and-conditional-occurrences`, every target meets occurrences.
A target that emitted each event once and unconditionally would pass validation and break the
suite of `ae9b`. A target that ignored the construct silently is a red flag of the fit-review skill.

## Outcome

- Generated Rust and Go behaviour (`crates/generate/ess-synth/src/rust/behaviour.rs`,
  `rust/behaviour/`, `go/behaviour.rs`, `go/behaviour/`) emits occurrences in order with their
  payloads, and evaluates `when:` on the pre-outcome subject.
- Where a predicate form is not implemented in a target, the command stays a named obligation, as
  other unimplemented guards do (`crates/generate/ess-synth/src/determined.rs`).
- The entity runtime lowering (`crates/generate/ess-entity-runtime`) refuses a model with
  occurrences by name, unless it implements them. The reference page
  `website/docs/reference/entity-runtime-lowering.md` lists the refusal.
- Generated docs (`crates/generate/ess-gen/src/docs.rs`) list each occurrence with its condition.
- AsyncAPI is unchanged: an event's emitters do not depend on how often they emit
  (`story:an-asyncapi-send-operation-names-every-emitter`).

## Acceptance

1. Red today (with `ae9a` and `ae9b` landed): the `ae9b` suite run against the generated Rust target
   of the reproduction model fails on the second `StatusChanged`. The case sits in the existing
   generated-target conformance test of `ess-synth`, chosen at implementation.
2. After: the generated Rust and Go targets pass that suite, or keep each unimplemented form as a
   named obligation that the suite reports as skipped with its reason.
3. The entity runtime refuses the model, naming the outcome and the construct.
4. Generated docs show the occurrences and conditions. Models without occurrences generate the same
   bytes: `feasibility`, `served_codec_names`, `go`, and the docs pins.

## Scope

- inferred: the generated behaviour files above, `crates/generate/ess-synth/src/determined.rs`,
  `crates/generate/ess-entity-runtime/src/`, `crates/generate/ess-gen/src/docs.rs`,
  `website/docs/reference/entity-runtime-lowering.md`

## Owned files

As in Scope, and new cases in existing `ess-synth`, `ess-entity-runtime` and `ess-gen` test targets.

## Not yours

Validation (`ae9a`). Synthesis and the interpreter (`ae9b`).

## Tests first

Acceptance 1 (Rust), then Go, then 3. Generated targets are built once per test run by the existing
harness. Name any case over 5 s.

## Gate

Each test stays under 60 s, the step under 120 s, and each job under 10 minutes.

```console
task fmt-check
cargo clippy -p ess-synth -p ess-entity-runtime -p ess-gen --all-targets --locked -- -D warnings
cargo test -p ess-synth --locked --test feasibility
cargo test -p ess-synth --locked --test served_codec_names
cargo test -p ess-synth --locked --test go
cargo test -p ess-entity-runtime --locked
cargo test -p ess-gen --locked --test openapi
task test-xtask
```

## Depends on

`ae9a-an-outcome-emits-repeated-and-conditional-occurrences`, `ae9b-synthesis-witnesses-every-occurrence-and-both-sides-of-a-condition`.

## Changelog

- Generated Rust and Go behaviour emits repeated and conditional emissions; the entity runtime
  refuses them by name; generated docs list them.

## Fit review

As `ae9a`. Question 4, targets: every target handles the construct or refuses it by name.

## Decisions

**AE-D11** (open, with the owner as AQ4).

## Size

M, about 450 lines with tests.
