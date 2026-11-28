---
format: aep.planning-md/3
id: story:an-input-only-type-invariant-is-checked-by-a-refused-input
kind: story
status: active
title: An invariant on a type only command input reaches is checked through a refused input
summary: ESS-SYNTH-013 refuses a value invariant on a newtype used only in a stateless command input and asks for a view the domain cannot have; correct refusal, misleading message.
tags:
- adopter-report
- defect
- ess-0.71.0
- severity-medium
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-synthesis
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:ess-37-and-conformance-64-65-are-open-for-the-epics-constructs
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/scenario.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/value_invariant.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/input_value_invariants.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/value_invariant_reach.rs
- confidence: inferred
  path: docs/design/wrapped-value-invariant-observations.md
revision: 17
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:39:56Z", actor: "human:timo", revision: 10, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:39:57Z", actor: "human:timo", revision: 11, decided_on: {"recorded":{"review_outcome":2}}}
---
## Problem

Adopter report, 2026-10-26, ess 0.69.2; still on 0.92.0. A newtype `of: Integer` with value
invariants `[value >= 1, value <= 65535]`, used only in the input of a stateless domain's one command
(no entity, no view), passes `ess specify validate --strict-requires` and is refused by
`ess verify conform synthesize`. Minimal reproduction:

```yaml
format: ess/22
system: probe
version: v1
domain: probe.net
types:
  - name: probe.net.Port
    kind: newtype
    of: Integer
    invariants: [value >= 1, value <= 65535]
events:
  - name: probe.net.PortChecked
    fields: [{name: note, type: String}]
commands:
  - name: probe.net.CheckPort
    input:
      - {name: port, type: probe.net.Port}
      - {name: note, type: String}
    outcomes:
      - name: checked
        emits: [probe.net.PortChecked]
        payload:
          probe.net.PortChecked: {note: input.note}
components:
  - component: checker
    owns: {domains: [probe.net]}
    accepts: {commands: [probe.net.CheckPort]}
    publishes: {events: [probe.net.PortChecked]}
```

```text
$ ess specify validate --path spec --strict-requires
probe v1 — 1 file(s), valid
$ ess verify conform synthesize --path spec --out suite
refused: refusal[ESS-SYNTH-013]: type probe.net.Port has no scenario
  no view publishes a field position that can answer what `probe.net.Port` declares of every value
    - `value >= 1`
    - `value <= 65535`
  help: publish a field that holds a value of this type in some view, directly or inside a list, a map, a union or an `Optional`: every position a view publishes is observed
1 scenario(s) (0 authored), 1 refusal(s), written to suite
```

The message and help come from `value_unwitnessed` and the ESS-SYNTH-013 help in
`crates/verify/ess-conformance/src/synthesize.rs` (:2100-2125, :1602, :1769). The adopter works
around it with named refusals of its own command.

Classification: a correct refusal (no view can observe the invariant) with a misleading message. The
help asks for a view, which a stateless domain cannot have. An invariant on a type that only reaches
command input is an input check, and synthesis could exercise it by sending a value outside the
range and expecting the command's input refusal (an outcome in the `InputRefusal` phase,
`crates/specify/ess-domain/src/command/precedence.rs:54`).

## Acceptance

- `an_input_only_type_invariant_names_the_input_route`, in
  `cargo test -p ess-conformance --locked --test value_invariant_reach`: for the specification
  above, ESS-SYNTH-013 says that `probe.net.Port` reaches only command input (naming
  `probe.net.CheckPort.port`) and names what would let a scenario check it (an input refusal of the
  command), and its help does not contain the word `view`. The existing
  `an_invariant_on_a_type_a_command_input_reaches_still_refuses` in that target asserts the new
  cause in place of `ValueInvariantUnwitnessed { at: None, .. }`.
- `an_input_only_type_invariant_is_checked_by_a_refused_input`, in
  `cargo test -p ess-conformance --locked --test input_value_invariants`: where `CheckPort` declares
  an input-guarded refusal outcome whose guard holds for a `port` outside the invariants, synthesis
  writes an `ess-conformance/64` suite with scenarios sending `port: 0` and `port: 65536` and
  expecting that refusal, and `port: 1` and `port: 65535` accepted; the run reports 0 refusals and
  the suite passes `ess verify conform run --target interpreted`.
- A type a view publishes keeps its current ESS-SYNTH-013 text and suite bytes: the
  `wrapped_value_invariants` target and the suite-bytes pin targets pass with no row moved.

Added 2026-11-27 by the adopter report above (reproduced on 0.99.8). These lines refine the Acceptance
above and replace none of it:

- The two refusals that go: `ESS-SYNTH-013` "has no scenario ... no view publishes a field position" for a
  type whose only position is a command input, and `ESS-AUTHOR-001` "constrained type `<T>` has no
  executable observer here" for `fixtures: {x: <T>}` where `<T>` is a constrained newtype.
- `an_input_only_constrained_type_gets_scenarios_from_its_input_position`, in
  `cargo test -p ess-conformance --locked --test input_value_invariants`, on a test model like the report's
  (`shop.blobs`: a `String` newtype with `alphabet: "0123456789abcdef"` and `value.count == 64`, one command
  taking it as its only input, one actor, one component, and a binding that declares `refusals.malformed`):
  synthesis writes no `ESS-SYNTH-013`; a scenario sends a value that breaks the alphabet and one of the
  wrong length and expects the binding's malformed answer; a scenario sends a value that meets both and
  expects `registered`; the interpreted target passes the suite.
- The same model without a binding that declares `refusals.malformed` and without an input-guarded
  refusal outcome keeps `ESS-SYNTH-013`, with the cause and help this story's first acceptance line
  already asks for (the type reaches only command input; the remedies are an input refusal, a binding
  `refusals.malformed`, or a fixture), and no word about a view.
- `a_fixture_of_a_constrained_newtype_is_accepted_and_checked_when_the_suite_runs`, in the authoring
  tests of `ess-conformance`: `ess specify validate` on the report's authored scenario
  (`fixtures: {uploaded-digest: shop.blobs.Digest}`, input `{$fixture: uploaded-digest}`) reports 0 scenario
  refusals; the suite carries the fixture with its type; a run whose provider returns a value that breaks
  the type's constraints fails that run as a provider fault (not the scenario), and a value that meets them
  passes.
- `the_unreadable_refusal_of_a_constrained_type_names_the_cause`: where `ESS-AUTHOR-001` is still raised for
  a constrained type (a record invariant on a response type, a reading-attached type), its help line names
  that cause and no longer lists the document keys.
- A type a view publishes and a constrained response type keep their text and bytes: the
  `wrapped_value_invariants` target and the suite-bytes pin targets pass with no row moved except the
  model's own new scenarios.

Added 2026-11-27 with the decisions IT-D1 = A and IT-D2 = A (both below):

- Before the release that ships `ess-conformance/64` and `/65`, the coordinator sends the planning,
  runtime and runbook consumers one line each naming the new majors. That release's `CHANGELOG.md`
  section names the change a consumer-visible format change. The check is the sent lines and the
  section's text, both read before the tag.

## Decisions

- The new input-route scenarios are a new suite shape, so they land in `ess-conformance/64`
  (ordinary) and `ess-conformance/65` (coverage), opened by
  story:ess-37-and-conformance-64-65-are-open-for-the-epics-constructs.

IT-D1, how an input-only constrained type is checked when the command has no input-guarded refusal outcome
(2026-11-27, from the adopter report). Decided A on 2026-11-27 by the operator session.

- A (decided): through the binding. Where a binding declares `refusals.malformed`, synthesis sends a
  value that breaks each constraint and expects that answer, and a value that meets the constraints; where
  none does and no refusal outcome guards the input, `ESS-SYNTH-013` stays with the input-route cause.
  Adds no authored key; scenarios change, so `ess-conformance/64` and `/65`.
- B: only this story's route (an input-guarded refusal outcome), nothing for the binding. Leaves the
  adopter's shape refused.
- C: change nothing; the adopter removes the constraint or checks it in its provider.

IT-D2, a fixture of a constrained type. Decided A on 2026-11-27 by the operator session.

- A (decided): accepted at authoring; the provider's value is checked against the type's constraints
  when the suite runs, and a value that breaks them fails the provider, not the scenario.
- B: accepted only when the type also has an input position the suite checks (A's first route).
- C: stay refused and change only the help line to name the cause.

Consequence of IT-D1 A, decided 2026-11-27: the suite majors become `ess-conformance/64` (ordinary) and
`ess-conformance/65` (coverage), a consumer-visible format change. The acceptance names the notice the
release owes its consumers.

## Scope

This story moves the `CHANGELOG.md` shared surface and re-pins only the suite-bytes rows of
specifications whose types reach only command input.

- cited: crates/verify/ess-conformance/src/synthesize.rs
- cited: crates/verify/ess-conformance/src/scenario.rs
- inferred: crates/verify/ess-conformance/src/synthesize/value_invariant.rs
- cited: crates/verify/ess-conformance/tests/value_invariant_reach.rs
- inferred: crates/verify/ess-conformance/tests/input_value_invariants.rs
- inferred: docs/design/wrapped-value-invariant-observations.md

confidence: medium

## Adopter report, 2026-11-27 (0.99.8): an input-only constrained newtype and a fixture of it

A downstream storage specification, on 0.99.8 (`ess/37`, `contract` tier), reports the same refusal on a
shape this story's acceptance does not cover, and a second refusal beside it. Reproduced on 0.99.8 with a
minimal model (one `String` newtype with an `alphabet:` and `value.count == 64`, one command taking it as
its only input, no view, no entity, no input-guarded refusal outcome; its outcomes are the one success):

- `ess verify conform synthesize --component <api> --target ir` ends "1 scenario(s) for `<api>` (0 authored),
  1 refusal(s)" with `ESS-SYNTH-013` "type `<newtype>` has no scenario ... no view publishes a field position
  that can answer what `<newtype>` declares of every value", exit 0 (see the draft on the exit status).
- An authored scenario that takes a value of that type from a fixture (`fixtures: {x: <newtype>}`, input
  `{$fixture: x}`) is refused `ESS-AUTHOR-001` "unreadable: constrained type `<newtype>` has no executable
  observer here"; `ess specify validate` counts it as a scenario refusal. Its help line is the generic one
  about unknown document keys, which is not the cause. The refusal text comes from `typed_fields.rs`
  (`constrained type ... has no executable observer here`), the observer of a typed response or fixture
  value, which has no case for a constrained newtype outside a view.

Why a fixture is the only way the value gets in: a value that comes from outside the modelled surface (a
content digest of bytes the provider uploaded out of band) cannot be a literal, because a literal makes the
suite unrepeatable against one account (the second run finds the content present).

Related closed issues, read 2026-11-27: #499 (a typed response field of a constrained type gets no scenario;
response side, closed 2026-10-08), #600 (value invariants not observed inside a List or Optional, nor a
text's `.count` in a view row; closed 2026-10-19) and #730 (a struct with invariants in a
`read_your_writes` view; closed 2026-11-11). All three concern types a view or a response reaches; none
covers a type only a command input reaches, which is this story's, nor a fixture of a constrained type.

CHANGELOG: checked 2026-11-27 on `main`, sections 0.99.3 to 0.99.8 and `[Unreleased]`: no entry changes
`ESS-SYNTH-013` for an input-only type or admits a constrained type as a fixture.

## Fit review

Written 2026-11-27 for the adopter report in the section above, with the questions of
`.agents/skills/assessing-external-requests/SKILL.md`. Consumer: a downstream storage specification.

1. Need, apart from the proposed syntax: a command input whose type carries value constraints, and no
   view publishes it, has constraints the suite never checks, and a provider-supplied value of that type
   cannot be used in an authored scenario. Minimal reproduction: the `shop.blobs` model of the report
   (a `String` newtype `Digest` with `alphabet: "0123456789abcdef"` and `value.count == 64`, one command
   `Register` with one input of that type and a Boolean response, one actor granted it, one component
   accepting it); result as above, on 0.99.8. The reporter's proposal (theirs): scenarios from the input
   positions, a fixture of a constrained type accepted, and a help line that names the cause.
2. Class: gap. The invariant is declared and observable (a request that breaks it is answered as
   malformed by a binding that declares it), and nothing checks it; the fixture refusal is a defect of
   its help text and a gap in the observer.
3. Already expressible: partly. A named input-guarded refusal outcome on the command makes this story's
   first route apply (the existing acceptance); a literal example value works for a constrained input but
   makes a provider-owned value unrepeatable. Neither gives a value the provider supplies.
4. Fit: the grids' cell is "a position a command input reaches" in grid R: the story already decides
   it for a refusal outcome; the new case is the same cell reached through the binding's malformed
   refusal, and the fixture is the authored-scenario sibling of the same observer
   (`crates/verify/ess-conformance/src/typed_fields.rs`, `constrained type ... has no executable
   observer here`). Siblings: a response type (#499) and a view position (#600, #730) are already
   decided; the input position and the fixture position are the two left.
5. Second adopter: any specification of a protocol whose request carries a checked token or digest
   (an upload digest, a signed URL token) and whose provider owns the value; written brand-free in the
   example above.
6. Cost: no authored key. Synthesis scenarios change (new scenarios for an input-only type), so
   suites change and ride `ess-conformance/64` and `/65` as this story already says; admitting a
   fixture of a constrained type may change what the suite format carries for a fixture entry (whether
   a fixture entry's type with constraints is checkable by the run is not known: I don't know, read
   `typed_fields.rs` and the suite's fixture entry before the design page is final).
7. Alternatives: change nothing (the type is declared and unchecked; the adopter drops the constraint or
   writes the check in the provider); only this story's route through an input-guarded refusal outcome
   (leaves a command with no such outcome unchecked); the recommended design below.

Decision class: accept, redesigned (the reporter's three points are kept; the fixture value is checked
when the suite runs, not at authoring).
