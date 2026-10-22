---
format: aep.planning-md/3
id: story:a-refusal-policy-scenario-is-not-set-up-through-a-trigger-that-needs-the-row
kind: story
status: draft
title: A refusal-policy scenario for a missing related row is not set up through a trigger that needs that row
summary: binding/refusal scenarios for a when_related exists:false refusal send a trigger that itself refuses the absent row (adopter defect 2, 2026-10-22); since 0.65.0.
tags:
- adopter-report
- defect
- ess-0.69.0
- ess-0.69.0-first
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/binding_effects.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/refusal_policy.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/refusal_policy.rs
revision: 13
---
## Need

Reported by an adopter on 2026-10-22 (defect 2 of 3, relayed by the orchestrator; adopter names kept out). A binding invokes a
command with a `when_related: {via: input.<x>, exists: false}` refusal under a refusal-selected failure policy. The synthesized
`binding/refusal/<outcome>` scenario makes the related row absent by sending the trigger with a never-defined identity, but the
trigger command itself refuses that identity, its event is never emitted and the binding never fires:

```text
expected outcome = assigned, observed outcome = role-not-found
ESS-CF-INVOCATION 0 matching invocation(s) after 50 observation(s)
```

Needed: no failing scenario. Where the only emitter of the trigger event needs the related row the refusal needs gone, the
scenario is refused by name and left out (see Decisions, 2026-10-22).

## Where it lives

- `crates/verify/ess-conformance/src/synthesize/binding_effects.rs:1300` (`related_missing`): lines 1360-1372 swap a fresh
  identity into the trigger's `from` input without checking the trigger's own `when_related` branches. Before the swap, check
  every command outcome that emits `event`; when each has a `when_related` branch over the input the event field is copied
  from, return the named gap instead of the swapped invoke. Called from
  `crates/verify/ess-conformance/src/synthesize/refusal_policy.rs:139`, which reports the gap rather than emitting the scenario.
- `crates/verify/ess-conformance/src/synthesize.rs:1869` (`BindingGap`): a variant beside `RefusalUnforcible` (:1953) naming the
  refusal and the emitting command, rendered under ESS-SYNTH-010 like every binding gap. Inferred, not traced: whether
  `related_missing` returns `Option` today, so its signature widens to carry the gap.

## Regression status

Not a 0.68.0 regression: 0.67.0 writes a byte-identical suite. Entered in 0.65.0 with `9d0531a4f`, alongside the #608 fix;
0.64.0 refuses with ESS-SYNTH-010. Does not block 0.68.0.

## Decisions

2026-10-22, taken by the orchestrator and recorded here:

1. **No new suite format in 0.69.0.** The earlier choice of a test-adapter step that publishes the trigger's event is
   withdrawn here and recorded as its own draft story, `story:a-scenario-may-publish-a-trigger-event-through-a-test-adapter`
   (tagged `ess-0.70.0`), with its format decision left open there.
2. **The case is refused by name.** When every command outcome that emits the binding's event refuses while the related row is
   absent (a `when_related` branch over the input the event field is copied from), no sequence of the model's own commands
   delivers the event with that row absent: removing the row after the trigger and before delivery races on an asynchronous
   target, and `DeliverEvent` (`crates/verify/ess-conformance/src/scenario.rs:2585-2603`) takes only events nothing
   publishes. Synthesis leaves `binding/refusal/<outcome>` out and reports a refusal under the stable code ESS-SYNTH-010 (the
   binding-gap code) whose message says the only emitter of the trigger event refuses when the related row is absent, so the
   case cannot be arranged.
3. This supersedes the 2026-10-22 decision recorded earlier that the scenario executes with the row forced absent; that
   decision required a suite format change.
4. No format change: `ess/N` and the suite format stay. The suite for such a model loses the failing scenario and gains one
   refusal; every other suite keeps its bytes.

## Acceptance

Fixture: the reproduction's model (`DefineRole` creates `Role`; `AssignRole` guards `when_related {via: input.role_id,
exists: false}` and is the only emitter of `RoleAssigned{role_id: input.role_id}`; binding `grant-on-assigned` maps
`role_id: event.role_id` into `GrantForRole`, whose `role-missing` refusal is `exists: false` on that input; policy
`drop: [role-missing], retry: {except: [role-missing]}`), written brand-free into the test.

Test: `crates/verify/ess-conformance/tests/refusal_policy.rs`, new case
`a_refusal_whose_only_trigger_needs_the_row_is_refused_by_name`, beside
`a_refusal_no_scenario_can_force_is_a_named_coverage_limitation` (:215), whose assertion shape it follows.

- **Outcome 1, synthesis.** The suite holds no `grant-on-assigned/binding/refusal/role-missing`; `synthesis.refusals` holds
  exactly one refusal for that scenario id, with code ESS-SYNTH-010, and its message names `AssignRole` as the only emitter of
  `RoleAssigned` and says it refuses when the related `Role` is absent, so the case cannot be arranged. Transition it turns on:
  the scenario moves from emitted (and failing) to refused by name.
- **Outcome 2, run.** On the interpreted target the remaining 6 scenarios all pass, none fails, and the run's report lists the
  refusal (0.68.0: 7 scenarios, 6 passed, 1 failed with `expected outcome = assigned, observed outcome = role-not-found`).
- **Failing before.** On the unfixed `related_missing` the case fails: the scenario is in the suite and no such refusal exists.
- **Mutation.** Add a second emitter of `RoleAssigned` that does not guard the role (a variant inside the same test): the
  refusal is not raised and the scenario is synthesized through today's fresh-identity swap
  (`crates/verify/ess-conformance/src/synthesize/binding_effects.rs:1360-1372`), so the refusal is tied to "only emitter".
- **No regression.** Every existing case in `refusal_policy.rs` and `binding_arranged_rows.rs` keeps its verdict; a refusal
  scenario whose trigger does not guard the row keeps its bytes.
