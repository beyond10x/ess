---
format: aep.planning-md/3
id: story:a-negated-list-literal-is-witnessed-beside-any-other-leaf
kind: story
status: draft
title: A negated list literal is witnessed beside any other leaf
summary: A not-exists literal beside a leaf the base refutes, or beside enough leaves to fill the bound, is still not witnessed (turn-2 adversary, backlog)
tags:
- backlog
- defect
- from-finding
- review-finding
- severity-medium
- zero-open-issues
relations:
- informed_by: review-result:zi-t2-listlit-adv-1
- decomposes: epic:zero-open-issues-2026-11
- serves: vision:O2
revision: 1
---
## Need

`story:a-list-literal-guard-is-witnessed-beside-other-inputs-and-negation` added candidates to the
witness search (`crates/verify/ess-conformance/src/witness.rs`, the list-fill block of
`search_uncached`): every literal list filled on top of each earlier candidate, then one candidate
per literal left out of its list, laid over the base input only, the whole addition bounded by
`added < MAX_CANDIDATES`. The turn-2 adversary (`review-result:zi-t2-listlit-adv-1`) found two
guards it still does not witness (ESS-SYNTH-003):

- `all [exists telephony, exists voice, not exists fax, level > 3]`: the leave-one-out candidates
  sit on the base input, where `level > 3` fails, and every candidate that raises `level` has every
  list full, so it holds `fax`.
- the same three list children beside eight leaves `aN < 100` that the base satisfies: the full
  fills over the 65 hosts spend the bound first, so the leave-one-out fills are never appended
  (141 candidates tried, plus eight ESS-SYNTH-022 refusals).

Both are pinned, ignored, in
`crates/verify/ess-conformance/tests/adversary_t2_list_literal_attack.rs`
(`adversary_t2_list_literal_negation_beside_base_refuted_leaf`,
`adversary_t2_list_literal_negation_beside_many_leaves_survives_the_bound`). The module's rule 4
("at most MAX_CANDIDATES distinct inputs per outcome") is also looser than it says: the block can
add up to that many on top of the walk.

## Acceptance

- Both ignored cases run (no `#[ignore]`) and pass, each with its negated-child mutant caught.
- The leave-one-out fills are laid over the same hosts as the full fill and are not crowded out by
  the bound.
- The synthesis byte pins (`synthesis_suite_bytes_table`, `synthesis_suite_bytes_table_cli_read`,
  `adversary_related_values_all_bytes`, `external_beside_held_guard`) are unchanged, or each moved
  row is named.
- Rule 4 of the module doc states the bound the search keeps.

## Scope

- `crates/verify/ess-conformance/src/witness.rs` (`search_uncached`, the list-fill block; module doc rule 4)
- `crates/verify/ess-conformance/tests/adversary_t2_list_literal_attack.rs`
