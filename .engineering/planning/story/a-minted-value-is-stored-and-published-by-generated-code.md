---
format: aep.planning-md/3
id: story:a-minted-value-is-stored-and-published-by-generated-code
kind: story
status: draft
title: A value minted once is stored and published by generated code, at a top-level or nested payload field
tags:
- adopter-report
- ess-0.70.0
relations:
- decomposes: epic:downstream-reported-gaps
revision: 2
---
## Need

Source: a2a wave 3 (an adopter's server implementation, 2026-10-22). One outcome mints a timestamp, stores it on the row and publishes the same value in its event (adopter `ess/domains/tasks.yaml:1305` stores `status_timestamp: {generated: true}`; `:1314` publishes the status timestamp). The adopter wrote the payload as `{subject: status_timestamp}`; it validates, and generated code publishes the value from before the update.

## Classification

A missing feature, not a defect. The specification says `{subject: <field>}` reads the addressed entity as it was before the outcome (`website/docs/reference/spec-versions.md:166`, `website/docs/reference/formats.md:150`, `website/docs/guides/specify/values-and-views.md:22`, `:35`), and the generator does exactly that. The documented idiom for "store what the payload publishes" is the other direction, ess/27 `sets: {<field>: {payload: <Event>.<field>}}` (`docs/design/minted-value-sources.md`), and it does not serve the adopter on 0.68.0 for two reasons the design records under "Not done":

1. Generated Rust and Go do not store a payload read: the command is kept an obligation.
2. `{payload: …}` reads a top-level event field, and the adopter's timestamp sits at `status.timestamp`.

## Reproduction

Minimal spec on 0.68.0 (entity field `stamped: Optional<Timestamp>`, a moving outcome):

- `sets: {stamped: {generated: true}}` and payload `at: {subject: stamped}`: valid; generated `src/behaviour.rs:165-166` writes `next.data.stamped = self.ports.try_generate_optional_timestamp()?;` then `at: before.stamped.clone()`.
- `sets: {stamped: {payload: m.jobs.JobFinished.at}}` and payload `at: {generated: true}`: valid; generated `src/jobs.rs:381` keeps `FinishJob` an obligation: "kept an obligation by a `sets:` value read from the outcome's payload (`{payload: m.jobs.JobFinished.at}`) for `stamped`, in `moved`".
- The nested form `{payload: m.jobs.JobFinished.status.timestamp}`: refused, `error[ESS-COMMAND-001]: \`m.jobs.JobFinished.status\` is not an event this outcome emits and fills under \`payload:\`, so it publishes no \`m.jobs.JobFinished.status.timestamp\` to store`.

## Where it lives

- `before.` is chosen at `crates/generate/ess-synth/src/rust/behaviour.rs:2694-2699` and emitted for `{subject: <field>}` at `:2845-2850` (traced to the generated line). Go mirrors it in `crates/generate/ess-synth/src/go/behaviour.rs`.
- The obligation for a payload read: `crates/generate/ess-synth/src/determined.rs:620-631` (`ResolvedPayloadValue::PayloadRead`, comment "owed in this cut"), traced through the generated doc line.
- The top-level-only payload read: the `ESS-COMMAND-001` refusal in `ess-domain`, not located to a line.

## Size (estimate, inferred)

- Generated storage of a top-level payload read: lift the `PayloadRead` refusal in `determined.rs` and, in Rust `behaviour.rs` and Go `behaviour.rs`, render the payload value into a local before the row and the event. About 60 to 120 lines across `ess-synth` alone, with generated-output pins moving.
- A nested path in `{payload: …}`: `ess-domain` resolution, `ess-compiler` IR, the interpreter and conformance's `Observed` resolution, plus the generator. About 150 to 300 lines across four crates and a format note.

## Acceptance (provisional)

- One outcome can mint a value once and both store it and publish it, at a top-level or nested payload field, and generated Rust and Go implement it without an obligation.
- Conformance requires the stored row and the published payload to hold the same value.
- `ess specify validate` warns where a payload `{subject: f}` reads a field the same outcome's `sets:` writes, naming the before-value reading and the `{payload: …}` idiom.

## Fit review

The follow-up recorded in minted-value-sources.md "Not done": generated storage of a minted value,
and a nested payload path.

## Fit review

1. **Need.** A command that stores a minted value read from its own event payload
   (`sets: {f: {payload: E.f}}`, ess/27) runs in generated Rust and Go, not only on the interpreted
   target; and the value may sit below the event's top level (`{payload: E.status.timestamp}`).

2. **Class.** Generated storage: target completeness, no semantic change. Nested path: convenience
   with a small semantic extension (the path must resolve to a typed leaf).

3. **Already expressible?** The model is: ess/27 validates and the interpreter runs it
   (interpret/execute.rs:2694 and 2929 `reads_in`, from `work.published`). Generated code is not:
   crates/generate/ess-synth/src/determined.rs:654-664 keeps the command an obligation, "Generated
   code would have to order the payload's value before the row; owed in this cut". A nested path is
   refused as ESS-COMMAND-001 at crates/specify/ess-domain/src/command/minted_value.rs:304.
   Minimal reproduction:

   ```yaml
   outcomes:
     - name: placed
       creates: orders.order.Order
       sets: {placed_at: {payload: orders.order.OrderPlaced.placed_at}}
       emits: [orders.order.OrderPlaced]
       payload: {orders.order.OrderPlaced: {placed_at: {generated: true}}}
   ```

   By determined.rs:654-664, generated code keeps Place an obligation (read from code, not run in
   this review). Changing the read to
   `orders.order.OrderPlaced.status.placed_at` is ESS-COMMAND-001.

4. **Fit with grids and targets.** Synthesis already witnesses the read (`minted_row`,
   synthesize.rs around 10773, ScenarioValue::Observed{event, field}); generated code only has to
   render the payload value into a local before both the row write and the event. A nested path
   extends `PayloadRead{event, field, type_ref}` to a path; synthesis's `Observed` and the
   interpreter's `publish_reads` gain the same path. Typed responses stay out of scope: a command
   with a typed `response:` remains an obligation (rust/behaviour.rs:55-56), which is R5's
   territory.

5. **Second adopter.** The story is the ess/27 design's own deferral; the nested path is the shape
   of any status object with a timestamp.

6. **Cost.** Generation: a let-binding per minted read in Rust and Go behaviour, ordered before the
   row and the event. Nested path: a path in PayloadRead, its resolution and the format row.

7. **Alternatives.** (a) Leave generated targets as obligations: the ess/27 page names this as owed.
   (b) Flatten nested values by authoring: pushes structure into the event type. (c) Generate
   storage and admit a nested path: chosen. A validate warning where a payload `{subject: f}` reads
   a field the same outcome's `sets` writes was considered; it is a lint, not a refusal, and is left
   out of this unit.

## Decisions

**Accept, redesigned.**

- **Generated storage.** For each `PayloadRead` in an outcome's sets, generated Rust and Go mint
  the payload value once into a local, write the row from it, and publish the event with it.
  determined.rs:654-664 no longer keeps such a command an obligation unless another reason holds.
  No format change: the IR is the same.
- **Nested path.** `{payload: E.a.b}` resolves through record fields of the event type to a typed
  leaf. Refused (ESS-COMMAND-001, existing code) when a segment is not a record field, crosses a
  list or Optional, or the leaf type is not assignable. ess/31 additive row in FORMAT_HISTORY:
  "below ess/31 refused as unsupported_format_version naming ess/31; a model without it keeps its
  IR bytes". IR: `PayloadRead` carries a path (a top-level read keeps its current bytes).
- **ValidationCode.** None new.
- **Files.** crates/generate/ess-synth/src/determined.rs, rust/behaviour.rs, go/behaviour.rs;
  ess-domain command/minted_value.rs:304 and system.rs; ess-compiler PayloadRead lowering;
  ess-conformance interpret/execute.rs (`publish_reads`, `reads_in`) and synthesize.rs
  (`minted_row`, Observed path); docs minted-value-sources.md; CHANGELOG.md. Pins: `task
  test-xtask` and `cargo test -p ess-compiler --test typed_diagnostics`.
- **Relation to R5.** Separate unit (see fit/r5.md). Overlap: minted_value.rs and the later-guard
  arrangement over a stored minted value; take it after R5's first step or note the overlap.
- **Size.** M (L if the Go generator's event construction needs restructuring).

## Acceptance

In crates/generate/ess-synth tests (the determined-command tests):

- `a_stored_payload_read_is_generated_in_rust` (first red test, in
  crates/generate/ess-synth/tests/minted_value_sources.rs): the reproduction is no longer an
  obligation and the generated Rust compiles.
- `a_stored_payload_read_is_generated_in_go`.
- `a_generated_target_stores_the_published_value`: the generated Rust target passes the
  synthesized suite, including the row-equals-payload scenario.

In crates/specify/ess-domain/tests/minted_value_sources.rs:

- `a_nested_payload_read_resolves_at_ess_31`.
- `a_nested_payload_read_below_ess_31_names_the_format`.
- `a_nested_path_through_a_list_or_optional_is_refused`.
- `a_top_level_payload_read_keeps_its_ir_bytes`.

In crates/verify/ess-conformance/tests/minted_value_sources.rs:

- `a_nested_payload_read_is_witnessed_on_the_interpreted_target`.
