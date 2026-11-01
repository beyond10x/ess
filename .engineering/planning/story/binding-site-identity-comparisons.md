---
format: aep.planning-md/3
id: story:binding-site-identity-comparisons
kind: story
status: active
title: Binding scenarios compare event identity fields with captured instances
tags:
- follow-up
relations:
- serves: vision:O2
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/binding_effects.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/bounded_retry.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/delivery_context.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/adversary_273_pass1.rs
revision: 10
transitions:
- {from: "draft", to: "proposed", at: "2026-11-01T20:19:30Z", actor: "human:timo", revision: 9}
- {from: "proposed", to: "active", at: "2026-11-01T20:19:30Z", actor: "human:timo", revision: 10}
---
## Outcome

Binding scenarios (flow, delivery, mapping, on-failure, bounded retry) compare the identity fields of the events they expect against the captured instances, as command scenarios do since beyond10x/ess#273.

## Origin

Follow-up from the #273 merge (2026-10-04). On the integration branch the #273 adversary sweep found four binding scenarios of the binding-arrangement fixture (#266/#267) where `Kicked.job_id` copies an input instance and no step compares it; binding synthesis sites emit no `expect_event_values`. The sweep pins those four misses until this story lands.

## Scope

Derived 2026-11-01 by `aep:story-scoper` for wave D350 on dc60f4532. Every line is **cited** (read from the story or the tree) or **inferred**.

- **Primary surface:** `crates/verify/ess-conformance/src/synthesize.rs` binding aspects: `binding_aspects` :18721-18758, `flow` :19084-19117, `mapping` :19134-19179, `delivery` :19195-19243, `on_failure` :19263-19362 — cited
- **Emission:** each pushes `ScenarioStep::ExpectEvent { payload: BTreeMap::new() }` (:19095, :19151, :19211, :19300); none calls `determined_identities` — cited
- **Command side (the beyond10x/ess#273 model):** `determined_identities` :9154-9190 and `expect_event_step` :9195-9219 (returns `ExpectEventValues` when identities exist) — cited
- **Also likely:** `synthesize/binding_effects.rs` `dropped` :3195 (empty ExpectEvent :3248); `synthesize/bounded_retry.rs` `forced_run` :256 (:270) — emission cited, being the on-failure/retry landing inferred
- **Delivered variants:** `synthesize/delivery_context.rs` :782-945 emit `EventuallyEvent`, no `ExpectEvent` — cited; whether they need identity comparison — inferred
- **Pinned by:** `tests/adversary_273_pass1.rs` :241-332, `binding.len() == 4` at :326-331 — cited; fixture `tests/fixtures/binding-arrangement.yaml` — cited
- **Confidence:** medium
- **Would collide with:** `synthesize.rs` :9154-9219 and :18721-19362; `delivery_context.rs`; `binding_effects.rs` :3195-3250; `bounded_retry.rs` :256-270; `tests/adversary_273_pass1.rs`

## Acceptance

Written by the D357 coordinator from the Outcome and Origin above (the story carried no Acceptance section).

- A binding scenario (flow, mapping, delivery, on-failure, bounded retry) whose expected event carries an identity field determined by a captured instance compares that field, as command scenarios do since beyond10x/ess#273 (`expect_event_values` with the identity, or the step form the command side uses).
- The four misses pinned by `crates/verify/ess-conformance/tests/adversary_273_pass1.rs` (`binding.len() == 4`, `Kicked.job_id` copying an input instance in `tests/fixtures/binding-arrangement.yaml`) become zero: the pin is rewritten to assert no binding scenario leaves an identity field uncompared, with `story:binding-site-identity-comparisons` in its message.
- A target that copies the wrong id into a binding's event fails at least one of those scenarios; the correct interpreted target still passes all of them.
