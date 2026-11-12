---
format: aep.planning-md/3
id: story:an-identity-guard-is-decided-on-every-row-synthesis-arranges
kind: story
status: draft
title: An identity guard is decided on every row synthesis arranges
summary: 'Follow-up of story:identity-guard-decided-on-the-row-sent-on-every-synthesis-path: a row-set accepting branch below a row-set refusal, another input refusal in each held state, a state refusal chosen by a stored-row guard and a minted identity still send a row the identity guard does not keep (adversary pass 1, backlog)'
tags:
- backlog
- defect
- from-finding
- review-finding
- severity-low
- zero-open-issues
relations:
- decomposes: epic:zero-open-issues-2026-11
- serves: vision:O2
revision: 1
---
## Need

`story:identity-guard-decided-on-the-row-sent-on-every-synthesis-path` has synthesis decide a
`when:` guard on a command's identity input on the row it sends on the addressed-row and
held-state paths. The first adversary pass found four more paths that still send a row created
under a placeholder identity (`"pass_id"`, `"pass_id-1"`) or a minted one. The reference
interpreter then answers the guard's branch, and a correct implementation fails its own suite.

- **Row-set accepting branch below a row-set refusal.** The model has a `when_related` row-set
  refusal (`held`), with an identity-guarded accepting branch (`vip`, `pass_id == "vip"`) below it.
  The compiler admits only that order. The row is arranged by `row_set::prepare`
  (`crates/verify/ess-conformance/src/synthesize/row_set.rs`), which does not steer it, so `vip`
  observes `closed`.
- **Another input refusal sent in each held state.** The model is the story's `state.yaml` plus
  `late` (`reason == "late"`). `existence.rs` `refused_inputs_in` and `arranged_in` arrange
  placeholder rows, so the guard `pass_id != "main"` answers `frozen` in place of `late` and
  `closed`.
- **A state refusal chosen by a stored-row guard.** The model is `state.yaml` plus `stale`
  (`when_subject: {predicate: flagged == true}`). The `subject_fact::refusal_witness` branch of
  `run_state_refusal` (`synthesize.rs`) sends a pass created as `"pass_id"`.
- **A minted identity guarded by a literal.** `sent_keeps_branch` counts a target-minted identity
  as keeping the branch. No real model guards a minted identity by a literal.

## Acceptance

1. In `crates/verify/ess-conformance/tests/adversary_identity_row_sent_pass1.rs`, these cases pass
   with their assertions unchanged, each with its
   `#[ignore = "backlog: identity-row-remaining-paths …"]` removed:
   - `row_set_accepting_branch_below_a_row_set_refusal_passes`;
   - `row_set_accepting_branch_opens_the_row_its_identity_guard_selects`;
   - `state_further_input_refusal_in_each_held_state_passes`;
   - `state_refusal_beside_a_stored_row_guard_opens_main`.
2. `complete_guard_on_a_minted_identity_is_not_written_failing` either passes, or is replaced by a
   case showing that the scenario is refused `ESS-SYNTH-003` with the guard named.
3. No row pinned in `synthesis-suite-bytes-table.tsv` moves, except for a model with one of these
   shapes.
