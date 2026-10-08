---
format: aep.planning-md/3
id: story:existence-family-sends-read-selection-plan
kind: story
status: draft
title: The existence-family synthesis sends read the selection plan
relations:
- decomposes: epic:one-selection-plan
- depends_on: story:synthesis-reads-selection-plan
revision: 1
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
