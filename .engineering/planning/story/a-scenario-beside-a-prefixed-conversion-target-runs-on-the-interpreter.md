---
format: aep.planning-md/3
id: story:a-scenario-beside-a-prefixed-conversion-target-runs-on-the-interpreter
kind: story
status: draft
title: A scenario beside a binding into a prefixed conversion target runs on the interpreter
summary: 'Adversary 0.71.0 F5: with a prefix on the conversion target, the trigger and mapping scenarios are Unsupported on the interpreter, as on 0.70.0.'
tags:
- defect
- ess-0.72.0
- severity-medium
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-synthesis
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/binding_effects.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/witness.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/adversary_0710_pass1.rs
revision: 9
---
## Problem

Found by the adversary pass over wave 1 of 0.71.0 (`adversary_0710_pass1.rs`,
`u4_the_prefixed_model_s_remaining_scenarios_pass_the_interpreter`); the 0.70.0 binary gives the
same result, so it predates the wave, and 0.92.0 still does. A binding maps an event field onto an
invoked command's input across a declared conversion whose target newtype carries a `prefix:`.
Synthesis refuses the binding's flow and names the conversion (correct since 0.71.0), but the
scenarios it does write, `CreateMember/outcome/created` and `binding/mapping`, come back
`Unsupported` on the model interpreter: "binding input does not satisfy its declared type". The
scenario sends the source `GroupRef` the literal `group_ref`. The interpreter checks the mapped
value against the target's prefix and finds the unconverted source value
(`crates/verify/ess-conformance/src/interpret/bindings.rs:404`), so a scenario synthesis wrote for
a valid model cannot run on the reference target.

Minimal reproduction: the fixture `crates/verify/ess-conformance/tests/fixtures/binding-input-through-conversion/`
with one edit in `x.yaml`:

```yaml
- {name: demo.x.GroupId, kind: newtype, of: String, prefix: "g-"}
```

The binding is `enrol-on-member-created` (`when: {event: demo.x.MemberCreated}`,
`invoke: {command: demo.x.Enrol}`, `mapping: {group_id: event.group_ref}`), with the conversion
`demo.x.GroupRef -> demo.x.GroupId`. On 0.92.0:

```text
$ ess verify conform synthesize --path spec --out suite.json
refused: refusal[ESS-SYNTH-010]: binding enrol-on-member-created has no scenario `enrol-on-member-created/binding/flow`
...
5 scenario(s) (0 authored), 3 refusal(s), written to suite.json
$ ess verify conform run --target interpreted --path spec --suite suite.json --report-format 2
demo v1 against interpreted 0.92.0 — failed
  unsupported demo.x.CreateMember/outcome/created
      ESS-CF-TARGET: cannot expose interpreted binding: binding `enrol-on-member-created` has an unmet binding-input obligation and never reached its command port: cannot expose interpreted binding: binding input does not satisfy its declared type
  unsupported enrol-on-member-created/binding/mapping
      (same)
  5 scenarios: 3 passed, 0 failed, 0 error, 2 unsupported
```

## Decisions

- **`binding/mapping` is refused, naming the conversion.** Its `ExpectInvocation` asserts the mapped
  input equals the event field as sent, and what a conversion into a prefixed newtype makes of a
  value is the implementation's (the reason the flow is refused, `adversary_0710_pass1.rs:224-225`);
  `binding_effects::mapped_input` already refuses a selection or accessor across a conversion for
  the same reason.
- **`CreateMember/outcome/created` runs.** It asserts nothing about the binding, so synthesis sends
  the trigger input that reaches the mapping a witness the conversion target also admits (the
  target's prefix in front, `g-…`); the interpreter, which hands a converted value on as sent
  (`synthesize.rs:9769-9771`), then carries the binding out. The interpreter is unchanged: its check
  at `bindings.rs:404` is right.
- Where no value satisfies both types (the source declares a prefix the target's excludes), the
  trigger's scenario is refused naming the conversion rather than written to fail.

## Acceptance

- `cargo test -p ess-conformance --locked --test adversary_0710_pass1`:
  `u4_the_prefixed_model_s_remaining_scenarios_pass_the_interpreter` asserts `assert_sound` in place
  of today's pin of two `Unsupported`, and that `demo.x.CreateMember/outcome/created` is `Passed`.
- Same target, new `u4_the_prefixed_model_s_mapping_is_refused_naming_the_conversion`: the run's
  refusals hold one for `` `enrol-on-member-created/binding/mapping` `` whose text contains
  `` `demo.x.GroupRef` -> `demo.x.GroupId` ``.
- Same target, new `u4_the_prefixed_trigger_is_sent_a_value_the_target_admits`: in
  `demo.x.CreateMember/outcome/created` the `CreateMember` input `group_ref` starts with `g-`.
- Same target, new `u4_a_source_prefix_the_target_excludes_is_refused_naming_the_conversion`: with
  `demo.x.GroupRef` declared `prefix: "r-"` as well, `demo.x.CreateMember/outcome/created` is
  refused, the refusal naming the conversion, and the run has no `Unsupported` scenario.
- The reproduction above: `ess verify conform synthesize` reports `4 scenario(s) (0 authored), 4
  refusal(s)`, and `ess verify conform run --target interpreted` reports `4 scenarios: 4 passed, 0
  failed, 0 error, 0 unsupported`.
- `u4_a_conversion_at_the_trigger_payload_is_followed_and_the_suite_passes` and
  `u4_a_chain_of_two_unchanged_conversions_is_followed_and_the_suite_passes` stay green, and
  `cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table` moves no row of a model
  without a conversion into a constrained newtype.

## Scope

- cited: crates/verify/ess-conformance/tests/adversary_0710_pass1.rs
- cited: crates/verify/ess-conformance/src/synthesize.rs (`mapping` :20182)
- cited: crates/verify/ess-conformance/src/synthesize/binding_effects.rs (`mapped_input` :3407)
- inferred: crates/verify/ess-conformance/src/witness.rs (the trigger input's witness)

confidence: medium

Shared surfaces: proposes its `CHANGELOG.md` sentence and re-pins only suite-bytes rows of models
with a conversion into a prefixed newtype.
