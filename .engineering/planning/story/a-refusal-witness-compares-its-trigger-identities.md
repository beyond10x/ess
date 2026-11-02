---
format: aep.planning-md/3
id: story:a-refusal-witness-compares-its-trigger-identities
kind: story
status: active
title: Binding refusal witnesses compare their trigger event's identities
relations:
- serves: vision:O2
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/bounded_retry.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/refusal_policy.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/adversary_273_pass1.rs
revision: 9
transitions:
- {from: "draft", to: "proposed", at: "2026-11-02T07:12:06Z", actor: "human:timo", revision: 8}
- {from: "proposed", to: "active", at: "2026-11-02T07:12:06Z", actor: "human:timo", revision: 9}
---
## Finding

Out of scope in wave D357 U4 (`review-result:adversary-d357-u4-pass-1`, residue): binding refusal scenarios produced by the refusal-policy witness site (`crates/verify/ess-conformance/src/synthesize/refusal_policy.rs`, `witness`, near :225) still emit a shape-only event step, which may swap in another publisher or an alternative trigger whose branch the site does not carry. The widened sweep in `crates/verify/ess-conformance/tests/adversary_273_pass1.rs` pins the seven input identities left uncompared there in `refusal_witness_residue()`.

## Acceptance

- `<binding>/binding/refusal/<outcome>` scenarios compare the identity fields of the trigger event they expect, as the other binding scenarios do since `story:binding-site-identity-comparisons`, including where the witness swaps the publisher or the trigger.
- `refusal_witness_residue()` is empty and removed; the sweep asserts no binding scenario leaves an identity uncompared.
- A target that publishes a wrong id in such a trigger event fails at least one scenario; the correct interpreted target passes all of them.

## Scope

Derived 2026-11-02 by `aep:story-scoper` for wave D368 on 6437975cb. Every line is **cited** (read from the story, the tree or a probe run of `ess` 0.82.0) or **inferred**.

- **Primary surface:** `crates/verify/ess-conformance/src/synthesize/refusal_policy.rs`, `witness` :126-237; trigger picked at :188-191, shape-only `ScenarioStep::ExpectEvent` pushed at :225-229 — cited
- **Sibling already converted:** `retried` :462 calls `super::expect_trigger_event` at :491, so only `witness` is left — cited
- **Helper:** `crates/verify/ess-conformance/src/synthesize.rs` `expect_trigger_event` :9252, `Published` :9222, `Sent` :9227, `as_run` :9234, `determined_identities` :9154, `expect_event_step` :9195 — cited; the helper itself should not need changing — inferred
- **Swapped trigger, alternative path:** `bounded_retry::clean_trigger` (`bounded_retry.rs:157-164`) drops the branch the other publisher takes; `clean_publisher` (:171, private) keeps it; `refusal_policy.rs:139` is the only caller of `clean_trigger` — cited. Call `clean_publisher` (made `pub(super)`) or widen `clean_trigger` — inferred
- **Swapped trigger, publisher path:** `unguarded_publisher` (`refusal_policy.rs:244-276`) returns without the `&ResolvedOutcome` it found (:258), which the `Published` pair needs — cited
- **Arranged-row path:** `refused_row` :286 → `binding_effects::related_refused` (`binding_effects.rs:1629`); missing-row path `binding_effects::related_missing` (`binding_effects.rs:2408`, called at `refusal_policy.rs:150`, :264) — cited
- **Tests that pin today:** `tests/adversary_273_pass1.rs` sweep :331-441 asserts `binding == refusal_witness_residue()` :430-436; the residue :443-465 has 7 entries over 5 fixtures — cited
- **Behaviour tests that stay green:** `tests/a_predicate_refusal_policy_is_forced_through_an_arranged_row.rs`, `binding_refusal_policy_swapped_caller.rs`, `a_caller_attribute_keeps_the_binding_refusal_scenario.rs`, `binding_into_updating_command.rs`, `refusal_policy.rs` — cited
- **Test changes:** delete `refusal_witness_residue`, assert the binding set empty; one wrong-id case in the style of `every_binding_scenario_fails_a_target_publishing_the_wrong_id_in_its_trigger_event` (:955) on a refusal fixture — inferred
- **Bytes rows that would move:** `tests/fixtures/synthesis-suite-bytes-table.tsv` :395, :398, :401, :402, :405 and `synthesis-component-suites-table.tsv` :83, :84, :87, :88, :91 (binding-into-updating-command, binding-refusal-policy-swapped-caller, caller-binding-refusal/attr and /plain, predicate-refusal-policy-arranged-row) — cited rows, inferred that they move. Committed `suites/`: none (no `binding/refusal/` there) — cited
- **Confidence:** high on where it lands; medium on which bytes rows move
- **Would collide with:** `refusal_policy.rs`, `bounded_retry.rs` :157-200, `synthesize.rs` :9195-9290, `adversary_273_pass1.rs`, both synthesis bytes tables (row-disjoint from other units' re-pins)
