---
format: aep.planning-md/3
id: review-result:adversary-a-mapwitness-pass-2
kind: review-result
status: active
title: Adversary pass 2, 0.41 unit mapwitness
relations:
- reviews: story:map-inputs-get-a-non-empty-witness
revision: 1
---
unit: mapwitness (#196), correction 1
verdict: green (2 notes; no blocker, no warning)
cases: executed 1431→1438, red 3
origin: introduced 2, pre-existing 0, undecided 0
wrote-outside-worktree: ~/.cache/ess-wave-n2/mapwitness/adv2/
needs-coordinator: no

Adversary pass 2 (`aep:adversary`), 2026-09-28, head 10c5d91d6 + correction 1 + `tests/adversary_map_witness_pass2.rs` (7 cases, 3 red). Held: mutual recursion through maps, recursion through a copied list, a newtype over a recursive map, a recursive list stored and read through a view; copied collections of invariant types; an update moving a copied list off the prior value. The three red cases are ignored under story:collections-reach-their-upper-count-boundary.

```findings
[{"file":"crates/verify/ess-conformance/src/synthesize.rs","line":7449,"category":"boundary","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"an upper count bound on a map (<= 2) is never sent at its accepting boundary because with_count cannot grow a map, so a < 2 target passes, matching the list twin's existing gap"},{"file":"crates/verify/ess-conformance/src/witness.rs","line":2305,"category":"boundary","severity":"note","verdict":"INFEASIBLE","origin":"introduced","message":"for Map<_, List<Self>> the map entry uses up the single unfold, so the copied list inside it is empty and no nested value is ever sent, contrary to the documented unfold-once rule"}]
```

Trend: pass 1 → 5, pass 2 → 2, carried 0. Coordinator routing: both notes deferred to story:collections-reach-their-upper-count-boundary; no correction 2.
