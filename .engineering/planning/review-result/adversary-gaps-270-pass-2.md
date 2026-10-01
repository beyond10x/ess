---
format: aep.planning-md/3
id: review-result:adversary-gaps-270-pass-2
kind: review-result
status: active
title: Adversary pass 2, downstream gaps unit feature-request-270
relations:
- reviews: story:feature-request-270
revision: 1
---
unit: story:feature-request-270, tree gaps-270
verdict: NEEDS-CHANGE, red 5 (introduced 4)
cases: executed 2026→2035

- warning (related_guard.rs:885): the named row was always the last accepted row, so copying from the most recent accepted row passed.
- note (:885): with one companion the named value was least or greatest, so a sort-order copy passed.
- warning (:1055): the companion had to differ in every copied field; a guard-pinned field dropped it.
- warning (:996): a companion under the named owner was constructible but never arranged.

Correction 2 (coordinator-verified): companions before and after, values below and above where ordered, at least one differing field, owner-link companions under the named owner. The note is emitted only when no companion exists, as the pass-2 assertions require.
Coordinator rerun in gaps-270: pass1 15/15, pass2 9/9, related_guard_copied_value 5/5, related_guard_owner_link 4/4, adversary_271 9/9 and 7/7, related_guard_state 4/4, adversary_229_pass2 6/6. Unit gate 2035 passed, 0 failed; no code changed on the integration branch since the base.
