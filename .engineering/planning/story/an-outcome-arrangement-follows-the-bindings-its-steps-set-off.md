---
format: aep.planning-md/3
id: story:an-outcome-arrangement-follows-the-bindings-its-steps-set-off
kind: story
status: implemented
title: An outcome arrangement follows the bindings its steps set off
summary: A related-guard arrangement that sets off a binding rewriting the arranged row no longer yields a scenario the interpreter fails
tags:
- ess-0.74.0
relations:
- informed_by: review-result:ess-0730-wave-adversary-pass-1
- serves: vision:O2
scope:
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/binding_effects.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/related_guard.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/adversary_0730_pass1.rs
revision: 9
transitions:
- {from: "draft", to: "proposed", at: "2026-11-01T13:05:41Z", actor: "human:timo", revision: 6}
- {from: "proposed", to: "active", at: "2026-11-01T13:05:41Z", actor: "human:timo", revision: 7}
- {from: "active", to: "implemented", at: "2026-11-01T15:20:29Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"test_result":1,"review_outcome":1,"verification":1}}}
---
## Problem

Found by the 0.73.0 adversary pass (finding F2). It already fails on the released 0.72.0.

A command `Give` refuses `not-flagged` through `when_related: {via: input.grant_id, predicate: flag == false}`.
To reach that branch, synthesis arranges a grant with `flag == false` through `Lower`. `Lower`
publishes `Lowered`, and the binding `raise-on-lowered` answers it by setting the flag back to
true. At the time `Give` is sent the grant is flagged again, so the interpreter answers `given`
and the synthesized outcome scenario `Give/outcome/not-flagged` fails. On 0.72.0 the model
synthesizes 16 scenarios, of which 15 pass and 1 fails.

Cause, read from the code: the related-guard arrangement (`synthesize/related_guard.rs`, about
:3136 and :4118) treats the row its steps leave as final. It does not follow a binding those
steps set off. `when_subject` searches likely share the gap (inferred).

## Acceptance

- `adversary_0730_pass1::f2_a_binding_that_rewrites_the_arranged_row_is_followed` asserts no
  failing scenario. Its pinned known failure is removed in the same change.
- An outcome scenario whose arrangement sets off a binding that rewrites the arranged row either
  follows that binding (and arranges a row that still refuses) or is refused by name. It never
  fails against the interpreter of the same model.

## Scope

Derived 2026-11-01 by `aep:story-scoper` for wave D350 on dc60f4532. Every line is **cited** (read from the story or the tree) or **inferred**.

- **Primary surface:** `crates/verify/ess-conformance/src/synthesize/related_guard.rs` — cited
- `related_guard.rs:3125` `arranged_at`, the present-row path `Give` takes: its related via is `input.grant_id`, so `stored::field` (`related_guard/stored.rs:44`) returns None at the early return on `:3136`, and the code continues at `:3137` — cited
- `related_guard.rs:3230` `with_row(...)`, the row the arrangement settles on; `:3300` `own_arrangement`; `:2755` `prepare_at_in` and `:2772` `arranged`, the callers' entry points — cited
- The triage pointer `:4118` falls in `send_each_without_row` (`:4095`), the `exists: false` send path, not the present-row path — cited
- **Callers:** `synthesize.rs:4728` and `:4744` in `run_as` (`:4640`) — cited for the calls; inferred that they build `Give/outcome/not-flagged`
- **Also likely:** `synthesize/binding_effects.rs:1513` `related_arranged` (calls `prepare_at_in` at `:1564`) — cited for the call; inferred to be the binding-refusal path
- **Binding awareness:** no `binding` reference in `related_guard.rs` or `related_guard/stored.rs` — cited. The only set-off check is `refusal_policy.rs:277` `refused_row` (`sets_off` `:319`, `rewritten` `:337`), which filters the binding's own refusal-policy scenario, not outcome scenarios — cited
- **Pinning test:** `crates/verify/ess-conformance/tests/adversary_0730_pass1.rs:104` `f2_a_binding_that_rewrites_the_arranged_row_is_followed` asserts `failures.keys() == ["demo.d.Give/outcome/not-flagged"]` today; fixture `tests/fixtures/adversary-0730/restoring-binding/` — cited
- **Confidence:** medium. The failing case was not traced step by step; the `when_subject` shared gap was not checked.
- **Would collide with:** `related_guard.rs` (`arranged_at`, `own_arrangement`, `arranged`, `prepare_at_in`), `refusal_policy.rs` `refused_row`, `binding_effects.rs` `related_arranged`, `synthesize.rs` `run_as`, `tests/adversary_0730_pass1.rs`

## Delivered

D350, 2026-11-01: unit 462151a7d (merged 7a9012dd1). New `related_guard/rewritten.rs`: a related row a step's binding rewrites (field, state, removal, also two bindings down) is passed over, else ESS-SYNTH-003 names step, binding and command. Scope check: `binding_effects.rs` was not needed (wrong in Scope); `run_as` building `Give/outcome/not-flagged` confirmed. Adversary pass 1: story 6 not broken (no-op). The `when_subject:` and decoy-row neighbours became `story:binding-rewrites-are-followed-for-subject-guards-and-decoy-rows`.
