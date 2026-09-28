---
format: aep.planning-md/3
id: story:view-filters-witnessed-on-matching-rows
kind: story
status: draft
title: A filtered view is asserted over a row its filter matches
relations:
- decomposes: epic:retrofit-findings-20260927
revision: 1
---
## Scope

Synthesis asserts a filtered view only over a row the filter does not match, so an implementation
that applies the filter wrongly (byte-wise instead of case-insensitively, or not at all) passes. Seen
for a view filtered by `equals_ignore_case` and, as a control, by plain `source == "web"`: rows
asserted `["source"]`, no refusals. Found by adversary pass 1 of the guards unit
(`review-result:adversary-retrofit-w2-guards-pass-1`); the view-filter synthesis code was not changed
by that unit.

## Acceptance

`adv_a_folded_view_filter_is_witnessed_on_a_case_changed_row_or_refused`
(`crates/verify/ess-conformance/tests/adversary_guards.rs`, currently `#[ignore]` naming this story)
passes with the `#[ignore]` removed, and an equality-filtered view is asserted over a row the filter
matches.
