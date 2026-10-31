---
format: aep.planning-md/3
id: decision-blocker:an-out-of-type-command-input-may-have-a-declared-error
kind: decision-blocker
status: cleared
title: May a command declare the error it answers for an input outside its type?
summary: 'G5 finding F2: keep type constraints as preconditions, or a declared error per broken constraint (every target''s decoding)'
revision: 3
transitions:
- {from: "open", to: "cleared", at: "2026-10-31T20:14:22Z", actor: "human:timo", revision: 3}
---
## Question

Does a type constraint on a command input stay a precondition, or may a command declare the error it
answers for an input outside its type?

## Today

Generated decoders refuse an out-of-type value before any outcome, and synthesis never sends one
(`predicates.md`, "an input that still breaks an invariant is never sent"). A refusal guard that
only out-of-type values meet is unreachable, so a downstream deployment loosened its identifier
type and repeated the constraint as refusal guards (wave D335 U2, finding F2).

## Options

- **A, keep.** The idiom stays: a loose input type plus refusal guards. Nothing changes.
- **B, a declared error for an out-of-type input.** Synthesis sends one value per broken constraint
  and expects that error; the Rust, Go and TypeScript decoders, the interpreter, Entity Runtime and
  the OpenAPI error responses answer it. Needs a new specification format and a suite major, and
  authored inputs would admit out-of-type values for such commands.

B changes every target's decoding and is an architecture-level decision; the wave ships A's
behaviour with a clearer ESS-SYNTH-003 message.

## Decision

**A, keep** (D336, 2026-10-31; decider: host-monitor as the operator's proxy, its D48, relayed by
the orchestrator).

An out-of-type command input stays refused by the decoder before the command: no error declaration,
no format version. No adopter asks for B. B is a breaking-format change and reopens only when a
second adopter asks for it in writing.

The wave ships the clearer ESS-SYNTH-003 message (G5 finding F2) and the checked idiom (F3).
