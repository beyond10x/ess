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
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
- depends_on: story:a-binding-trigger-acts-on-the-row-its-setup-arranged
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/binding_effects.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/binding_arranged_rows.rs
revision: 7
---
## Need

Reported by an adopter on 2026-10-22 (defect 3 of 3, relayed by the orchestrator; acknowledged in the 0.65.0 verification reply,
no issue filed). A binding with unbounded `on_failure: retry` into a command whose target row can be missing leaves scenarios
Unsupported:

```text
unsupported <Trigger>/outcome/<accepting>
ESS-CF-TARGET: cannot expose interpreted binding: unbounded retry exceeded the finite execution budget
```

Needed: synthesize the trigger's scenarios with the target row arranged present for the accepting path (see Decisions,
2026-10-22; refusal by name is not accepted).

## Reproduction

Two brand-free variants on 0.68.0, both 7 scenarios, 6 passed, 1 unsupported: a `when_related exists: false` refusal with
`on_failure: retry`, and an `updates` command with `unknown_instance`. A trigger that guards the row's existence passes 6/6.
The adopter's affected scenario was `binding/mapping`; the reproduction shows only the trigger's own outcome scenario
Unsupported, so the adopter's case may differ.

## Where it lives

Traced 2026-10-22 (code read plus runs of the 0.68.0 binary; the fix itself is inferred, not built).

- The Unsupported scenario is built by `outcome_scenario_noting` (`crates/verify/ess-conformance/src/synthesize.rs:3062`) through
  `exercise_as` (:3492), `run_as` (:3931) and `exercise_run_in` (:3621). Its setup comes from `arranged_as` in `run_as`
  (:4067-4099), which arranges only the trigger's own needs and looks at no binding the emitted event sets off. So the
  trigger is sent with a literal identity nothing stores (in d3c `AssignRole{role_id: <literal uuid>}`), the retrying
  `GrantForRole` refuses every attempt, and `crates/verify/ess-conformance/src/interpret/bindings.rs:199-240` reports the
  exhausted budget (:237, :261).
- `binding/mapping` for the same binding already arranges the row: `DefineRole`, `capture_instance role`, then
  `AssignRole{role_id: instance role}`, through `binding_effects::prepare`
  (`crates/verify/ess-conformance/src/synthesize/binding_effects.rs:983`), today called only from `binding_aspects`
  (`synthesize.rs:17055`). For d3c (`updates` with `unknown_instance`) `prepare` goes `TriggerInput`, `arranged_destination`
  (:1448), `eligible_row` (:1628); for d3b (`when_related exists: false`) through `related_decided` (:1088).
- **Fix site:** `exercise_run_in` (`synthesize.rs:3621`), before `binding_effects::settle_run` (:3637), so every witness run
  gets it. For each binding whose `when.event` is in `outcome.emits`, with failure `retry`, no retry bound and no
  `refusal_policy`: call `binding_effects::prepare(ir, binding, ir.command(&binding.command), outcome, &run, actors)`,
  replace `run.setup` and `run.invoke` with the prepared ones (append its `capture`), and set `run.input[field]` to the
  arranged instance so the payload assertion on the trigger's event names the arranged identity, not the old literal.
  `expect_no_event` for the invoked command's event stays valid (`not_emitted` reads only the trigger's direct events,
  `synthesize.rs:7914-7926`).

## Regression status

Present on 0.65.0 through 0.68.0; 0.67.0 and 0.68.0 give identical suites. Does not block 0.68.0.

## Decisions

2026-10-22, taken by the orchestrator and recorded here:

1. **The scenario executes.** The trigger's own `<Trigger>/outcome/<accepting>` scenario arranges the row its unbounded-retry
   binding acts on before sending the trigger, the same way `binding/mapping` already does. Refusing by name, or a message
   asking for `external:`, is not an accepted outcome; the earlier "or refuse by name" is withdrawn.
2. **Fix site traced** (see Where it lives): `exercise_run_in` reuses `binding_effects::prepare` unchanged; no new arrangement
   code.
3. **Depends on** `story:a-binding-trigger-acts-on-the-row-its-setup-arranged`: same file and rule, and a binding whose
   destination is a stored field of the trigger's row reaches `prepare` through `stored_destination`, which that story fixes.
   The two reproductions here (d3b, d3c) do not take that route.
4. No format change: only the setup of the affected outcome scenarios changes bytes. Verified on 0.68.0: only the trigger's
   outcome scenario is Unsupported; every binding scenario, `binding/mapping` included, passes, so the adopter's affected
   `binding/mapping` is not reproduced and is not in this story's acceptance.

## Acceptance

Fixtures: the two failing reproductions, brand-free, written into the tests: d3b (`GrantForRole` refuses with
`when_related {via: input.role_id, exists: false}`, binding `on_failure: retry` unbounded) and d3c (`GrantForRole` `updates`
a `Role` with `unknown_instance`). On 0.68.0 each gives 7 scenarios, 6 passed, 1 unsupported:
`demo.access.AssignRole/outcome/assigned`, `ESS-CF-TARGET: ... unbounded retry exceeded the finite execution budget`.

Test: `crates/verify/ess-conformance/tests/binding_arranged_rows.rs`, two new cases using `witnessed` (:584) and `position`
(:599): `an_unbounded_retry_trigger_outcome_arranges_the_row_its_binding_updates` (d3c) and
`an_unbounded_retry_trigger_outcome_arranges_the_related_row_its_binding_reads` (d3b).

- **Outcome 1, synthesis.** In `demo.access.AssignRole/outcome/assigned`, `DefineRole` is sent before `AssignRole`, and
  `AssignRole`'s `role_id` is the captured instance `role`, not a literal. Transition it turns on: a `Role` row exists before
  the trigger is sent.
- **Outcome 2, run.** On the interpreted target every scenario passes, 7 of 7 for each fixture. Transition: the binding's
  `GrantForRole` accepts on its first attempt (d3c `granted` on the arranged row; d3b the accepting branch over a present
  related row), so no retry runs and the budget is never reached.
- **Failing before.** On the unfixed `exercise_run_in` both cases fail: the scenario is `Status::Unsupported` with the budget
  message, and the `position` assertion panics because `DefineRole` is not sent.
- **Mutation.** Removing the `prepare` call from `exercise_run_in` turns both cases red exactly as under Failing before, and a
  target that drops the binding still fails `grant-on-assigned/binding/flow` on the fixed suite.
- **No regression.** d3a (the trigger guards the row's existence) keeps 6 of 6 passed and its suite bytes; every existing case
  in `binding_arranged_rows.rs`, `bounded_retry.rs` and `refusal_policy.rs` keeps its verdict.
