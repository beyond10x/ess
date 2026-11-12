---
format: aep.planning-md/3
id: story:a-nested-collection-quantifier-is-witnessed-apart
kind: story
status: draft
title: A nested collection quantifier is witnessed apart
summary: 'Follow-up of story:collection-quantifiers-witnessed-everywhere: a nested forall that holds, a nested one-value list, and a nested quantifier in a when_related predicate leave inner mutants alive (adversary pass 3, backlog)'
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

`story:collection-quantifiers-witnessed-everywhere` witnesses a quantifier over a command-input
list with several elements, and a quantifier nested in the body that one element decides
(`exists l in input.lines: exists c in l.codes: …`) is held to the same. Three nested shapes are
not yet witnessed apart, so a mutant target passes every scenario:

- A nested `forall` that holds beside an outer `exists`
  (`exists l in input.tags: forall c in l.codes: c == "ok"`): the deciding element carries
  `codes: []` and the refusing scenario sends `[]`, so targets reading only the first or last inner
  element pass. An element with a mixed inner list (`["ok", "x"]`) in the refusing scenario would
  tell them apart.
- A nested list whose element type admits one value (a one-variant enum): the inner list is sent
  `[One]`, every inner mutant survives, and `unspread_lists` raises no `UnspreadInputList` because
  it walks only top-level quantifiers.
- A nested quantifier in a `when_related` predicate over a stored list
  (`exists l in lines: exists c in l.codes: c == input.application`): the inner swap survives and
  no note names the scenario.

Found by the third adversary pass over the collection-quantifier story; the changelog of that
story states these three shapes as not yet witnessed.

## Acceptance

1. `adv3_a_holding_nested_forall_beside_an_outer_exists_kills_the_inner_mutants`,
   `adv3_a_nested_quantifier_over_a_one_variant_enum_is_spread_or_says_why_not` and
   `adv3_a_nested_quantifier_in_a_related_predicate_kills_the_inner_swap_or_says_why_not` in
   `crates/verify/ess-conformance/tests/input_list_quantifiers.rs` lose their
   `#[ignore = "backlog: nested-quantifier-apart ..."]` and pass with their assertions unchanged.
2. No pinned row moves except for a model with one of those nested shapes.
