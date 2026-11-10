---
format: aep.planning-md/3
id: story:an-unbounded-retry-binding-leaves-no-trigger-scenario-unsupported
kind: story
status: draft
title: An unbounded retry binding into a possibly missing row leaves no trigger scenario Unsupported
summary: 'Unbounded on_failure: retry into a command whose target row can be missing leaves the trigger scenario Unsupported (adopter defect 3, 2026-10-22); since 0.65.0.'
tags:
- adopter-report
- defect
- ess-0.69.0
- ess-0.69.0-first
- severity-medium
- zero-open-issues
relations:
- serves: vision:O2
- depends_on: story:a-binding-trigger-acts-on-the-row-its-setup-arranged
- supersedes: story:a-delivery-under-unbounded-retry-is-not-sent-unsupported
- decomposes: epic:adopter-gaps-synthesis
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/binding_effects.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/delivery_context.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/adversary_d350_u2_pass1.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/binding_arranged_rows.rs
revision: 14
---
## Need

A binding with an unbounded `on_failure: retry` leaves a synthesized scenario `Unsupported` on the
interpreted target when the command it retries can never succeed in that scenario: the interpreter
exhausts its finite execution budget. A correct implementation would retry forever, so the
scenario can pass no target.

Two cases were filed. The first (the trigger's own outcome scenario) is fixed; the second (a
redelivery scenario) still reproduces on 0.92.0.

## Fixed: the trigger's outcome scenario (0.69.0)

Shape: `AssignRole` emits `RoleAssigned {role_id: input.role_id}`, and a binding with
`on_failure: retry` (unbounded) invokes `GrantForRole`. That command refuses an unknown role, as
`updates` with `unknown_instance` (d3c) or as `when_related {via: input.role_id, exists: false}`
(d3b). Up to 0.68.0 `AssignRole/outcome/assigned` sent a literal `role_id` that no row carries and
was Unsupported. Fixed by eb2dfa52d9, CHANGELOG 0.69.0 ("The outcome scenario of a command whose
event an unbounded `retry` binding answers names the row that binding needs"). The fix is pinned
by `crates/verify/ess-conformance/tests/binding_arranged_rows.rs::an_unbounded_retry_binding_leaves_no_trigger_scenario_unsupported`.
On 0.92.0 both shapes synthesize 7 scenarios; `DefineRole` is sent before `AssignRole`, and
`AssignRole` names the captured `role` instance; 7 of 7 pass on `--target interpreted`.

## Problem: a redelivery under unbounded retry whose retried command then refuses

Merged from `story:a-delivery-under-unbounded-retry-is-not-sent-unsupported` (found by
`review-result:adversary-d350-u2-pass-1`, O1). Still present on 0.92.0.

Minimal model (the fixture `tests/fixtures/adversary-0730/restoring-binding` with the edits of
`tests/adversary_d350_u2_pass1.rs::s6_a_binding_that_moves_the_arranged_row_back_is_followed`):
`Grant` has the lifecycle `Active → Low` (`lower`) and `Low → Active` (`raise`). `Lower` moves a
grant to `Low` and emits `Lowered {grant_id}`. `Raise` refuses `not-low` (`wrong_state`) and
otherwise moves the grant back to `Active`. Binding `raise-on-lowered`: `when: {event: Lowered}`,
`invoke: {command: Raise}`, `delivery: at_least_once`, `on_failure: retry` (unbounded).

```text
$ ess verify conform run --path . --target interpreted --report-format 2
  unsupported raise-on-lowered/binding/delivery
      ESS-CF-TARGET: cannot expose interpreted binding: binding invocation budget exhausted
  15 scenarios: 14 passed, 0 failed, 0 error, 1 unsupported
```

The delivery scenario (`synthesize.rs`, `delivery` :20241, `RedeliverEvent` :20263) sends `Lower`,
then redelivers `Lowered`. The first delivery's `Raise` has already moved the grant to `Active`, so
every attempt of the redelivered `Raise` answers `not-low`, and the unbounded retry never ends
(`interpret/bindings.rs:303`, the 65,536-invocation budget).

## Acceptance

- `cargo test -p ess-conformance --locked --test binding_arranged_rows` passes a new test
  `a_redelivery_under_unbounded_retry_is_not_sent_unsupported`: on the model above, the
  synthesized suite holds no scenario `raise-on-lowered/binding/delivery`, and the synthesis
  refusals hold one `ESS-SYNTH-010` for `raise-on-lowered` whose message names `Raise` and its
  refusal `not-low`. Before the fix the same test fails, because the scenario is in the suite.
- On the same model, `ess verify conform run --path . --target interpreted --report-format 2`
  prints `14 scenarios: 14 passed, 0 failed, 0 error, 0 unsupported`.
- `cargo test -p ess-conformance --locked --test adversary_d350_u2_pass1` passes with `story6`'s
  filter on `{binding}/binding/delivery: Unsupported` removed.
- `cargo test -p ess-conformance --locked --test binding_arranged_rows --test bounded_retry --test refusal_policy`
  passes with no assertion changed.

## Decisions

- The delivery scenario is refused, not rebuilt: after the first delivery the retried command
  refuses on every redelivered attempt, so a correct implementation retries forever and no
  arrangement of the same redelivery can make it pass; the refusal is a new `BindingGap` variant
  reported under `ESS-SYNTH-010`, beside `DeliverySingleAttempt`, and the interpreter's budget is
  unchanged.

## Scope

- cited: `crates/verify/ess-conformance/src/synthesize.rs` (`delivery` :20241, `BindingGap` :2134, its hint, message and catalogue entry)
- cited: `crates/verify/ess-conformance/src/synthesize/binding_effects.rs` (`Prepared::settled_row` :1318, the row the first delivery leaves)
- inferred: `crates/verify/ess-conformance/src/synthesize/delivery_context.rs` (`delivery`, `RedeliverEvent` :844, the same rule for delivery-context scenarios)
- cited: `crates/verify/ess-conformance/tests/adversary_d350_u2_pass1.rs` (`story6` filter)
- cited: `crates/verify/ess-conformance/tests/binding_arranged_rows.rs`

confidence: medium

The story re-pins the suite-bytes rows of models whose delivery scenario it drops, and proposes
its `CHANGELOG.md` sentence.
