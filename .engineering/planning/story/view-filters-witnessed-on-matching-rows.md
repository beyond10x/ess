---
format: aep.planning-md/2
id: story:view-filters-witnessed-on-matching-rows
kind: story
status: active
title: A filtered view is asserted over a row its filter matches
relations:
- serves: vision:O2
- decomposes: epic:retrofit-findings-20260927
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/witness.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/adversary_guards.rs
revision: 6
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

## Derived scope

Derived 2026-09-27 by `story-scoper`. Every line is **cited** or **inferred**.

- **Primary surface:** `crates/verify/ess-conformance` row-level view assertions — cited
- **Test:** `tests/adversary_guards.rs:185-230` (`#[ignore]` at 189) — cited
- **Decision site:** `synthesize.rs` `view_expectations` (3842-3975) picks Contains/Excludes from whether `shows` admits the row as set up — cited
- **Symbols:** `shows` (4680-4743), `arrange_beside` (4564-4585, empty `settled` at 4576), `rows_shown` (4542) — cited
- **Also likely:** `arrange_first` (2531); `witness.rs` `swap_ascii_case` (916), `fold_literals_at` (1047); a new equality-filter control test — inferred
- **Confidence:** medium
- **Would collide with:** `synthesize.rs` view-assertion or arrangement code; `witness.rs` fold helpers. Calls into `shows`, which `synthesize/aggregate.rs:1177` also calls
