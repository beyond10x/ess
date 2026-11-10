---
format: aep.planning-md/3
id: story:an-input-only-type-invariant-is-checked-by-a-refused-input
kind: story
status: draft
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
revision: 9
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

## Decisions

- The new input-route scenarios are a new suite shape, so they land in `ess-conformance/64`
  (ordinary) and `ess-conformance/65` (coverage), opened by
  story:ess-37-and-conformance-64-65-are-open-for-the-epics-constructs.

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
