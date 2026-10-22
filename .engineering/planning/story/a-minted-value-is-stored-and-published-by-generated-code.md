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
revision: 1
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
