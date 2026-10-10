---
format: aep.planning-md/3
id: story:related-guard-wrong-state-beside-external
kind: story
status: draft
title: A related-guard command with wrong_state and an external branch gets a wrong-state scenario the interpreter passes
relations:
- decomposes: epic:one-selection-plan
revision: 1
---
# Story: A related-guard command with `wrong_state:` and an external branch gets a passing `wrong-state` scenario

## Why

Found by wave 5 U4's adversary (`review-result:selection-plan-w5-u4-adversary-pass-1`, F1) and confirmed
pre-existing: the coordinator ran the same cases against the base `related_guard.rs` (unchanged since
`3f0e8e1539`) and they fail identically. On a validating ess/22 model (the DESK pick model with
`when_related:`, `wrong_state:` and an external branch `unlisted`), the synthesized `wrong-state`
scenario arranges neither a list nor a pick and sends an unarranged `list_id`; the interpreter answers
`no-list`, so the suite fails. Replacing the external with an accepting `when:` passes (case a4d).

Also from the same review, pre-existing and only visible to tests: several-rows searches ask `selects`
over a one-row projection whose plan reads the predicate refusal among the accepting branches rather than
at step 5 (F2); `orders_present_related_refusal` returns true for a stored reference or several rows with
no related refusal declared, an arrangement rule under an order name (F5).

## Acceptance

- Cases a4, a4b, a4c parked at `~/.cache/ess-selection-plan/w5-u4-scratch/adversary_selection_plan_w5_u4_pass1.parked.rs`
  pass as a committed test.
- The synthesis bytes tables move only for the models this fixes, each named.
