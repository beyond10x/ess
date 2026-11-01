---
format: aep.planning-md/3
id: story:related-guard-wrong-state-beside-external
kind: story
status: active
title: A related-guard command with wrong_state and an external branch gets a wrong-state scenario the interpreter passes
relations:
- decomposes: epic:one-selection-plan
- serves: vision:O2
scope:
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/related_guard.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/adversary_selection_plan_w5_u4_pass1_open.rs
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-11-01T13:05:42Z", actor: "human:timo", revision: 6}
- {from: "proposed", to: "active", at: "2026-11-01T13:05:42Z", actor: "human:timo", revision: 7}
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

## Scope

Derived 2026-11-01 by `aep:story-scoper` for wave D350 on dc60f4532. Every line is **cited** (read from the story or the tree) or **inferred**.

- **Primary surface:** `crates/verify/ess-conformance/src/synthesize/related_guard.rs` (wrong-state synthesis) — cited
- **Symbols:** `wrong_state_overlap` :1678-1797; `orders_present_related_refusal` :1659-1669; `is_related_refusal` :1639; `with_row` :4207; `missing_late` :3938; `arranges_external` :204; `leaves_external` :336; `chooses` :396 — cited
- **Also likely:** `crates/verify/ess-conformance/src/synthesize.rs` `refusal_arrangement` :15892 (gate :15925, `wrong_state_overlap` call :15927, fallback `reach(...)` :15930) — cited
- **Mechanism:** the symptom (no row arranged, `list_id` unbound) matches the fallback at :15930, reached when the :15925 gate is false or `with_row` at :1780 arranges nothing; which fires for an external branch was not run — inferred
- **Phase order:** `PresentRelated` precedes `Accepting` in `crates/specify/ess-domain/src/command/precedence.rs:45`; where `External` sits was not read — cited for the enum, inferred for its effect
- **Tests:** cases a4 :389, a4b :413, a4c :424 exist as `#[ignore]`d tests in `crates/verify/ess-conformance/tests/adversary_selection_plan_w5_u4_pass1_open.rs` asserting the passing behaviour (the acceptance's parked file is this one) — cited. The non-ignored `a4d` (`adversary_selection_plan_w5_u4_pass1.rs:405`) asserts only the accepting-`when:` variant passes — cited
- **Story drift:** "base `related_guard.rs` unchanged since 3f0e8e1539" no longer holds: five later commits touch it (newest 3db8d0d84) — cited
- **Confidence:** medium (call path and symptom read; nothing run)
- **Would collide with:** `related_guard.rs` `wrong_state_overlap`, `orders_present_related_refusal`; `synthesize.rs` `refusal_arrangement`; `adversary_selection_plan_w5_u4_pass1_open.rs`
