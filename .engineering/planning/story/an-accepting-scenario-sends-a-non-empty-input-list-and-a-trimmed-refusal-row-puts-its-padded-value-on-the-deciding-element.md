---
format: aep.planning-md/3
id: story:an-accepting-scenario-sends-a-non-empty-input-list-and-a-trimmed-refusal-row-puts-its-padded-value-on-the-deciding-element
kind: story
status: draft
title: An accepting scenario sends a non-empty input list, and a trimmed refusal row puts its padded value on the element that decides
summary: 'A regression of 95de8f1619, 0.95.0 to 0.99.1: a quantifier over an input list loses its refuting accepting scenario, and a trimmed refusal row leaves its padded value on a filler element'
tags:
- defect
- regression
- synthesis
relations:
- serves: vision:O2
- informed_by: story:collection-quantifiers-witnessed-everywhere
- informed_by: story:a-nested-collection-quantifier-is-witnessed-apart
revision: 1
---
## Defect

From 0.95.0 a synthesized conformance suite can lack the scenario that refutes a quantifier over an input list, and can send a
refuting row whose deciding element a trimming target would accept. Two findings of one regression, `95de8f1619` (conformance:
input-list quantifiers are witnessed with several elements), first released in 0.95.0 and present in every release to 0.99.1.

## Reported case

Found 2026-11-25 on a toy specification (a command with a `List` input of structs, a quantifier over the list and a guarded
branch whose refusal is decided by a text value), compared at three commits: the base `4255c8b431` (7 steps), the parent of the
regression `c8b69640d0` (9 steps) and `95de8f1619` (7 steps).

1. **Finding 1, the accepting scenario sends no list.** The refuting-side trimmed/fold witness of a quantifier body is dropped:
   `DefaultBranch` rows move the leaf at `<list>.0` of the sibling's witness (`crates/verify/ess-conformance/src/synthesize.rs:20328-20336`
   at `4255c8b431`), and that witness is now widened to `[filler, deciding, filler]` (`witness.rs`, `quantified_first`). Element 0 is a
   filler, the deciding element stays unchanged, the row still selects the refusal, and `keep()` drops it. The design says the refuting
   branch is sent the value (`docs/design/value-expressions.md:434-437`, E10). Red on `4255c8b431`:
   `an_accepting_scenario_sends_a_non_empty_list_no_element_of_which_meets_the_quantifier` in
   `crates/verify/ess-conformance/tests/input_list_quantifiers.rs`, assertion at `:1397`: "`demo.crates.PackCrate/outcome/packed` sends
   a non-empty `items` list, got none" (0.040 s).
2. **Finding 2, a trimmed refusal row whose padded value decides nothing.** The toy's invalid-part scenario at `c8b69640d0` sends
   `[{kind: "\v\f\r\n", name: parts.0.name}]`, decided by that value alone, which a target that trims spaces and tabs only accepts
   (`docs/design/value-expressions.md:431-437`). At `95de8f1619` the same row is `[{kind: "\v\f\r\n", name: parts.1.name},
   {kind: " \t ", ...}, filler]`: the deciding element still decides, so a target trimming only spaces and tabs refuses it and passes.
   The refusal's own trimmed rows (`trim_rows` on the guarded side) move element 0 of the widened witness; the padded value belongs on
   the element or elements that alone decide the guard.

## Releases that carry it

0.95.0 (first tag containing `95de8f1619`) to 0.99.1.

## Fit review

1. **Need.** A conformance suite that says a target passes while the scenario that would catch a defect is missing or does not decide
   is worse than a smaller suite: it reports a pass nothing decided.
2. **Class.** A regression with a green gate: the commit added a witness shape and lost two rows the previous shape produced.
3. **Expressible today.** No: the generated suite is what it is.
4. **Fit.** The repair is in the witness placement of `synthesize.rs` (`boundary_inputs`, the `DefaultBranch` refutation loop) and in
   `trim_rows` of the guarded side; the design (E10) already states the behaviour.
5. **Existing stories.** None covers it. `story:collection-quantifiers-witnessed-everywhere` and
   `story:a-nested-collection-quantifier-is-witnessed-apart` border it (quantifier witnesses in the same module).
6. **Cost.** The generated suite bytes change for a model that loses a refuting row today; the byte tables
   (`synthesis-suite-bytes-table`, `synthesis-component-suites-table`) were rewritten in write mode for finding 1 and no row moved
   (no repository model loses this near-miss). Release train: 0.99.x patch (a defect of a shipped release; the fix changes what a
   synthesized suite sends).

## Decisions

None open.

## Acceptance

- an_accepting_scenario_sends_a_non_empty_list: the test of Finding 1 passes: every accepting scenario of a command with a quantifier
  over a list input sends a non-empty list no element of which meets the quantifier; the toy's created scenario has 9 steps with the
  near-miss `[filler, {kind: NBSP}, filler]`.
- a_trimmed_refusal_row_puts_the_padded_value_on_the_deciding_element: for Finding 2, the invalid-part scenario of the toy sends the
  value `"\v\f\r\n"` on the element that alone decides the guard, so a target trimming only spaces and tabs accepts it and the
  scenario fails that target.
- the_byte_tables_are_unchanged_or_named: the two byte tables and `synthesis-sources.digest` move only where the change claims it,
  with the cause named beside the pin.

## Delivery

Branch `fix/an-accepting-scenario-sends-a-non-empty-input-list`, tree `ess-fix-partslist`. The commit for Finding 1 is not final
(commit follows); Finding 2 follows after the first push.

## Scope

- cited: `crates/verify/ess-conformance/src/synthesize.rs` (`boundary_inputs`, `trim_rows`, 20328-20336 at `4255c8b431`),
  `crates/verify/ess-conformance/src/witness.rs` (`quantified_first`), `crates/verify/ess-conformance/tests/input_list_quantifiers.rs`
- inferred: `CHANGELOG.md`
