---
format: aep.planning-md/3
id: story:a-binding-trigger-acts-on-the-row-its-setup-arranged
kind: story
status: draft
title: A binding scenario sends its trigger against the row the setup arranged
summary: Binding delivery/flow/mapping scenarios arrange one row and trigger on another when the trigger guards when_related on the destination input (adopter defect 1, 2026-10-22); since 0.65.0.
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
  path: crates/verify/ess-conformance/src/synthesize/binding_effects.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/binding_arranged_rows.rs
revision: 8
---
## Need

Reported by an adopter on 2026-10-22 (defect 1 of 3, relayed by the orchestrator; adopter names kept out). A binding's synthesized
`binding/delivery`, `binding/flow` and `binding/mapping` scenarios arrange one row into the accepting state and then send the
trigger against another. The trigger's creating command guards `when_related` on the same input, answers its refusal, and the
scenario fails with no implementation able to pass it:

```text
ESS-CF-OUTCOME expected outcome = reset-issued, observed outcome = externally-managed
```

Needed: the trigger runs against the row the setup arranged, and the binding's destination is that row (see Decisions,
2026-10-22; refusal by name is not accepted).

## Reproduction

Brand-free fixture: two `CreateAccount` calls captured as `account` and `account-<n>`, `MoveToLocal(account-<n>)`, then
`RequestReset(account_id = account)`; the binding's destination is a `{subject: field}` payload source. 15 scenarios, 12 passed,
3 failed on 0.68.0. The field is copied straight from the creating command's input, so this is not past the #611 boundary; the
cause is the `when_related` guard on that input.

## Where it lives

- `crates/verify/ess-conformance/src/synthesize/binding_effects.rs:1571` (`stored_in_setup`): at :1622 the input the creator's
  `sets:` copies into the stored field is overwritten with the destination identity arranged separately by `eligible_row`
  (called from `stored_destination`, :1522). When the creator (the outcome found at :1604-1609) has a `when_related` branch
  whose `via` is `input.<from>`, keep the trigger's own value (`held`, :1588) and make it the destination identity instead.
- `stored_destination` (:1510): in that case drop the separate arrangement (`arrangement.steps`, :1536) and build the
  destination `row` (:1543) from the trigger-held identity and the state the trigger's arrangement left it in. Inferred, not
  traced: when that state is not one `bound_branch` admits for the invoked command, the implementor moves it on with the
  same transition search `eligible_row` (:1632) uses; the reproduction's `Account` has one state, so it never arises there.

## Regression status

Not a 0.68.0 regression: 0.67.0 writes a byte-identical suite. Entered in 0.65.0 with `9d0531a4f` ("binding and row-set scenarios
arrange the rows they read"); 0.64.0 refuses with ESS-SYNTH-010. Does not block 0.68.0.

## Decisions

2026-10-22, taken by the orchestrator and recorded here:

1. **The scenario executes.** The binding's `delivery`, `flow` and `mapping` scenarios send the trigger against the row the
   setup arranged, and the binding's destination is that same row. Refusing the scenario by name (ESS-SYNTH-010, the 0.64.0
   behaviour) is not an accepted outcome; the title's earlier "or is refused by name" is withdrawn.
2. **The destination is the row the trigger's arrangement already holds.** When the creating command of the trigger's row
   (here `RequestReset`) has a `when_related` branch over the input that `sets:` copies into the stored field
   (`input.account_id`), `stored_in_setup` does not swap a separately arranged identity into that input. The identity the
   trigger's arrangement sent there (and moved into a state the creator accepts) is the destination.
3. No format change: `ess/N` and the suite format stay; only the setup of the affected binding scenarios changes bytes.
4. `story:an-unbounded-retry-binding-leaves-no-trigger-scenario-unsupported` depends on this story (same file, same
   "arrange the row the trigger acts on" rule).

## Acceptance

Fixture: the reproduction's model (two entities `Account` and `Reset`, binding `store-secret-on-changed` from
`SecretChanged` into `StoreSecret`, destination `{subject: account_id}` copied by `RequestReset`'s `sets:` from its input,
which `RequestReset` guards with `when_related` twice), written brand-free into the test.

Test: `crates/verify/ess-conformance/tests/binding_arranged_rows.rs`, new case
`a_subject_field_destination_its_creator_guards_when_related_is_the_row_the_trigger_arranged`, beside
`a_binding_target_named_by_a_subject_field_is_arranged_before_the_trigger` (:734).

- **Outcome 1, synthesis.** The suite holds `store-secret-on-changed/binding/delivery`, `/flow` and `/mapping`, none refused.
  In each, the setup creates exactly one `Account` that `RequestReset` and the binding both name: `MoveToLocal` and
  `RequestReset` take the same captured instance, and no second `CreateAccount` arranges a separate destination
  (assert with `steps_of` and `position`). Transition it turns on: `Account` source `external` to `local` by `MoveToLocal`,
  before `RequestReset` reads it.
- **Outcome 2, run.** On the interpreted target all 15 scenarios pass (0.68.0: 12 passed, 3 failed with
  `ESS-CF-OUTCOME expected outcome = reset-issued, observed outcome = externally-managed`). Transition: `RequestReset`
  answers `reset-issued` (a `Reset` row in `Issued`), `ConsumeReset` moves it `Issued` to `Consumed` and emits
  `SecretChanged` carrying that account, and the binding's `StoreSecret` moves that account's `stored` from `false` to `true`.
- **Failing before.** On the unfixed `binding_effects.rs` the new case fails on outcome 2 with the three binding scenarios in
  `not_passed`; the implementor records that red run before the fix.
- **Mutation.** A target that drops the binding (the pattern of `a_dropped_binding_fails_the_subject_field_flow`, :768)
  fails `binding/flow` on the fixed suite, so the green run is not vacuous.
- **No regression.** Every existing case in `binding_arranged_rows.rs` and `refusal_policy.rs` keeps its verdict; a suite for a
  model whose creator has no `when_related` on that input keeps its bytes.
