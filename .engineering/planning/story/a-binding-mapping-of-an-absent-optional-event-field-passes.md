---
format: aep.planning-md/3
id: story:a-binding-mapping-of-an-absent-optional-event-field-passes
kind: story
status: draft
title: A binding mapping of an absent optional event field passes
summary: A binding mapping an Optional event field the trigger omits errors with 'carried no field' under omitted_when_absent or no presence (turn-2 adversary, pre-existing, backlog)
tags:
- backlog
- defect
- from-finding
- review-finding
- severity-low
- zero-open-issues
relations:
- informed_by: review-result:zi-t2-interp-adv-1
- decomposes: epic:zero-open-issues-2026-11
- serves: vision:O2
revision: 1
---
## Need

A binding that maps an `Optional` event field the trigger leaves out, where the field declares
`omitted_when_absent` or no `presence:`, makes its `…/binding/mapping` scenario report `error`:
``ESS-CF-SUITE: `example.token.TokenContributed` carried no field `note` ``. The runner reads the
mapped value as `ScenarioValue::Observed` and requires the field to be in the event
(`crates/verify/ess-conformance/src/runner.rs`, near line 3435), but an absent optional field is
omitted there by design. Under `null_when_absent` the scenario passes. Found by the turn-2
adversary (`review-result:zi-t2-interp-adv-1`); it is already in the base, since
`story:the-interpreted-target-publishes-an-absent-event-field-in-its-declared-presence` only
changes the `null_when_absent` path.

## Acceptance

- `adv_binding_mapping_of_an_absent_optional_field_passes_without_presence` and
  `adv_binding_mapping_of_an_omitted_when_absent_field_passes` in
  `crates/verify/ess-conformance/tests/adversary_t2_interp_presence_pass1.rs` run (no `#[ignore]`)
  and pass: the mapping scenario passes, with the mapped input absent.
- A mapping of a required field the event lacks still fails.

## Scope

- `crates/verify/ess-conformance/src/runner.rs` (the observed-value read of a binding mapping)
- `crates/verify/ess-conformance/tests/adversary_t2_interp_presence_pass1.rs`
