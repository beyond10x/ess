---
format: aep.planning-md/3
id: story:existence-family-sends-read-selection-plan
kind: story
status: archived
title: The existence-family synthesis sends read the selection plan
relations:
- decomposes: epic:one-selection-plan
- depends_on: story:synthesis-reads-selection-plan
revision: 3
transitions:
- {from: "draft", to: "archived", at: "2026-10-31T23:45:02Z", actor: "human:timo", revision: 3}
---
# Story: The existence-family synthesis sends read the selection plan

## Why

`epic:one-selection-plan`. Wave 5 U3's adversary (`review-result:selection-plan-w5-u3-adversary-pass-1`)
found synthesis sends that still assume the input refusals answer before the existence step:
`is_state_input_refusal` (`crates/verify/ess-conformance/src/synthesize.rs`), the creation family's
`existing_instance:` sends, and `existence::refusals_in_each_held_state`. Under the real order they are
right; under `with_phase_order` with InputRefusal read after Existence, the interpreter answers the
existence branch and three scenarios fail (RotateSecret `too-short`, RenewLease `blank-note`, Bind
`too-short`).

## Acceptance

- The three cases parked at `~/.cache/ess-selection-plan/w5-u3-scratch/adversary_selection_plan_w5_u3_pass1.parked.rs`
  pass, as a committed test.
- The synthesis bytes tables do not move.

## Triage (D338)

Declined at the 2026-10-31 findings triage (orchestrator D338, operator approval relayed by host-monitor D53): no model reaches it under the real order; reopen if epic:one-selection-plan makes the order a model input.
