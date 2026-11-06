---
format: aep.planning-md/3
id: story:a-value-invariant-on-a-type-nothing-reaches-refuses-nothing
kind: story
status: implemented
title: A value invariant on a type nothing reaches refuses nothing and is named
summary: ESS-SYNTH-013 on a type no command, event, error, view, entity or binding reaches becomes a note, as an unreached Binary64 already is (loom report 2026-11-04, ess 0.85.0)
tags:
- ess-w1-0870
- from-consumer-report
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-synthesis
scope:
- confidence: inferred
  path: crates/edge/ess-cli/src/main.rs
- confidence: inferred
  path: crates/specify/ess-compiler/src/binary64.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/value_invariant_reach.rs
revision: 9
transitions:
- {from: "draft", to: "proposed", at: "2026-11-04T08:47:28Z", actor: "human:timo", revision: 7, executor: "agent:ess-w1-lead", correlation: "ess-0.87.0-w1"}
- {from: "proposed", to: "active", at: "2026-11-04T08:47:28Z", actor: "human:timo", revision: 8, executor: "agent:ess-w1-lead", correlation: "ess-0.87.0-w1"}
- {from: "active", to: "implemented", at: "2026-11-05T17:37:08Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"test_result":1}}}
---
# Story: a value invariant on a type nothing reaches refuses nothing and is named

## Outcome

An author can declare the bounds of a value type that no command, event, error, view, entity,
actor, binding or conversion carries (a type that only describes a document the system reads, or
a resource schema it projects) without synthesis refusing the suite: the invariant is listed as a
note saying no run exercises it, the way an unreached `Binary64` position already is, and a type
that is reached keeps today's `ESS-SYNTH-013` refusal.

## Context

Reported 2026-11-04 by the loom repository on ESS 0.85.0; it blocks loom's story
`agent-spec-constraints-in-crd`, which wants the port range in the projected resource schema.
`crates/verify/ess-conformance/src/synthesize.rs` is unchanged between the 0.85.0 tag and `main`
at 8d26c81f74.

## Minimal reproduction (written for this story)

A newtype `demo.net.Port` of `Integer` with invariants `value >= 1` and `value <= 65535`, a struct
`demo.net.Aperture {host: String, port: Port}` and a struct `demo.net.Policy {apertures:
List<Aperture>}`, none of them named by any command input, event, error, view, entity or binding;
one unrelated entity and command so the system synthesizes. On 0.85.0:

```
refused: refusal[ESS-SYNTH-013]: type demo.net.Port has no scenario
  no view publishes a field position that can answer what `demo.net.Port` declares of every value
```

Reproduced by the wave lead on loom's own probe (138 scenarios, 1 refusal, `ESS-SYNTH-013` on
`loom.agent.Port`): the type sits in `EgressAperture -> ToolPolicy -> SessionSpec -> AgentSpec`,
which reaches only an `Optional<AgentSpec>` member of a struct no command, view or entity names.

## Problem

`ValueInvariantUnwitnessed` (`synthesize.rs:1030-1050`, raised at `:18590` and `:18894`) refuses
every value type with invariants that no view position answers, including a type nothing reaches at
all. For a `Binary64` position the same situation is a note, not a refusal (`synthesize.rs:448-454`
`Note::UnreachedBinary64`, built from `ess_compiler::binary64::unreached_locations` at
`synthesize.rs:3233-3239`): "refuses nothing; it is named, so a reader sees what the run does not
exercise". The two rules disagree for the same reach question.

## Acceptance

Named scenarios; each is a test function, and the story is done when each runs and passes.

- `crates/verify/ess-conformance/tests/value_invariant_reach.rs` (new)
  - `an_invariant_on_a_type_nothing_reaches_is_a_note` — the reproduction synthesizes with 0
    refusals and one note naming `demo.net.Port` and both invariants.
  - `an_invariant_on_a_type_a_command_input_reaches_still_refuses` — the same type used as a
    command input member, with no view publishing it, keeps `ESS-SYNTH-013` (`at: None`).
  - `an_invariant_on_a_type_a_view_publishes_unarranged_still_refuses` — the `at: Some(..)` shape
    is unchanged.
  - `an_invariant_reached_only_inside_an_optional_member_of_an_unreached_struct_is_a_note` — the
    loom shape (list inside a struct inside an `Optional` member of an unreached struct).
- `crates/edge/ess-cli` text and JSON output carry the note (one existing CLI test target that
  prints notes, named by the implementor).
- Existing `ESS-SYNTH-013` tests pass unchanged.

## Out of scope

- Witnessing an invariant through a decoder refusal at a command input (`story:generated-decoders-enforce-declared-value-constraints`, active, and a suite expectation for a rejected input that does not exist yet).
- Any change to schema or CRD projection of the bounds.

## Fit review

1. **Need.** A type's bounds are a fact the author wants projected and enforced by decoders;
   synthesis refuses it because no scenario can observe a type nothing carries. Requester's words:
   "a newtype's invariants (value >= 1, <= 65535) on a stateless type that no view publishes are
   refused with SYNTH-013".
2. **Class.** Defect against ESS's own rule for the same case (`Note::UnreachedBinary64`): an
   unreached position refuses nothing.
3. **Already expressible?** Only by dropping the invariants or by inventing a view that publishes
   the type, which changes the model to please synthesis.
4. **Fit.** One reach rule for every value type, reusing the compiler's reach walk (the one
   `binary64::reached_uses` already does) and the existing note channel. No new key.
5. **Second adopter.** A configuration file schema (`retries: 0..10`) declared in a specification so
   generated decoders hold it, read by a CLI from disk and never sent through a command.
6. **Cost.** One new note kind in the synthesis output (text and JSON); suites that refused before
   now synthesize; no suite format change for suites that synthesized.
7. **Considered.** (a) change nothing: the author drops the bounds. (b) an author-written exemption
   key: new surface for a case the reach rule decides. (c) witness through input rejection: needs a
   new suite expectation and does not apply here (nothing sends the type). (d) chosen: the
   `Binary64` rule, generalized.

## Decisions

Accept, redesigned (wave lead, 2026-11-04). Decided 2026-11-04 by the coordinator, Q-C answer (a):
an invariant on a type nothing reaches is a note, as an unreached `Binary64` position is; a reached
type keeps ESS-SYNTH-013 (approval-record:ess-0870-wave).
