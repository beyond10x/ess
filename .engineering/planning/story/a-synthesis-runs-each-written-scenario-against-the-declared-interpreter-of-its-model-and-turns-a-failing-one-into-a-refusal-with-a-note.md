---
format: aep.planning-md/3
id: story:a-synthesis-runs-each-written-scenario-against-the-declared-interpreter-of-its-model-and-turns-a-failing-one-into-a-refusal-with-a-note
kind: story
status: draft
title: A synthesis runs each written scenario against the declared interpreter of its model and turns a failing one into a refusal with a note
summary: 'The fourth item of the N88 fix plan: a check after synthesis that catches a scenario its own model refuses; not built with the N88 fixes, cost facts first (D125)'
tags:
- conformance
- feature
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- informed_by: story:a-synthesized-scenario-whose-setup-its-own-model-refuses-is-written-with-the-rows-its-guards-relate-or-refused-by-synthesis
revision: 2
---
## Summary

A synthesis that writes a scenario its own model refuses is found only when someone runs the suite
against the declared interpreter. The synthesis could do that itself: run each synthesized scenario
against the declared interpreter of the model it was synthesized from, and turn a failing one into
a refusal with a note. Two defects found on 2026-11-25 are such cases (group 1 and group 3 of
`story:a-synthesized-scenario-whose-setup-its-own-model-refuses-is-written-with-the-rows-its-guards-relate-or-refused-by-synthesis`:
25 of 201 scenarios of one downstream application's model fail interpreted), and a check of this
shape catches them and any later case without a fix of each.

## Need

The fourth item of the fix plan of that story (read from the diagnosis record, nothing more): in
`crates/verify/ess-conformance/src/synthesize.rs`, run each synthesized scenario against the
declared interpreter of the model it was synthesized from and turn a failing one into a refusal
with a note. The record says it catches the two groups and any later case, and that its cost is to
be measured. No other fact about the check is established: its cost and the scenarios it would turn
into refusals on the repository's fixtures are open.

## Acceptance

- A cost measurement comes first: synthesis time with and without the check on the repository's
  committed models and on a model of the size of the downstream application's (201 scenarios), and
  the number of scenarios it turns into refusals on each, recorded in this story before any build.
- With the check, a scenario of the red tests `a_link_guard_beside_a_missing_row_refusal_on_the_same_input_passes_its_own_model`
  and `a_setup_run_arranged_as_its_own_scenario_keeps_the_rows_its_guards_relate` is written as a
  refusal with a note, and the interpreted run of that suite passes.
- Suites move where the check refuses a scenario: the change names every moved suite and re-pins
  with the cause beside the pin.

## Fit review

1. **Class.** A feature that guards a class of defects; not a fix of one.
2. **Borders.** The N88 story above (the fixes it plans), and the synthesis tests that already run a
   suite against its model.
3. **Cost.** Unknown; the first acceptance item measures it. The synthesis would run the
   interpreter once per written scenario, on top of a synthesis that a regression of 0.99.0 made
   take 196 s on one model
   (`story:a-setup-run-that-falls-back-to-its-creating-commands-own-scenario-builds-that-scenario-once-per-distinct-arrangement-not-once-per-candidate-row`).

## Decisions

- Not built with the N88 fixes; cost facts first (coordinator, D125).

## Cost facts

Synthesis runs no interpreter per scenario today. One interpreted run of the whole suite of the
reported model takes 0.64 s against 194 s of synthesis (measured 2026-11-25).
