---
format: aep.planning-md/3
id: story:collections-reach-their-upper-count-boundary
kind: story
status: draft
title: An upper count bound on a collection is never sent at its accepting boundary
refs:
- provider: github
  reference: beyond10x/ess#196
revision: 1
---
## Outcome

A `.count` guard with an upper bound over a list or a map (`<= n`, `< n`) is witnessed at its accepting boundary, so a target implementing an off-by-one bound fails; and a collection nested inside the one unfolded entry of a recursive type holds one element.

## Evidence

Adversary pass 2 on the 0.41 mapwitness unit (review-result:adversary-a-mapwitness-pass-2):

- `synthesize.rs` `with_count` can shrink a text, list or map but not grow a collection, so from a 1-entry map or a 0-element list the `<= 2` guard is sent at 1 and 3, never 2; a `< 2` target passes. Cases: `adv196p2_an_upper_count_bound_on_a_map_is_witnessed_at_its_accepting_boundary`, `adv196p2_control_an_upper_count_bound_on_a_list_is_witnessed_at_its_accepting_boundary` (both `#[ignore]` with this story's id).
- `witness.rs` unfold-once: for `Map<_, List<Self>>` the map entry uses the single unfold and the inner list is `[]`. Case: `adv196p2_every_recursion_shape_synthesizes_unfolded_once` (`#[ignore]`).

## Acceptance

- the three ignored cases in `crates/verify/ess-conformance/tests/adversary_map_witness_pass2.rs` pass with `#[ignore]` removed;
- committed suites regenerate byte-identical or the change is listed in CHANGELOG.
