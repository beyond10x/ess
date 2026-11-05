---
format: aep.planning-md/3
id: story:choices-filter-bars-and-forms-render-what-they-declare
kind: story
status: active
title: Choices, filter bars and forms render what they declare
summary: Dropdown empty option and multi-select, filter labels, on-change forms, action order; choice placeholder, reset label, help on group/field/column (A11-A13, A15, A16, A18, A19, G-13/14/15; beyond10x/ess#723)
tags:
- ess-w1-0870
- from-consumer-report
refs:
- provider: github
  reference: beyond10x/ess#723
relations:
- serves: vision:O2
- decomposes: epic:ess-ui-renderer-neutral-ui
scope:
- confidence: cited
  path: crates/ui/ess-ui-react/src/emit.rs
- confidence: cited
  path: crates/ui/ess-ui-react/templates/runtime/composites/choice.tsx.tmpl
- confidence: cited
  path: crates/ui/ess-ui-react/templates/runtime/composites/filter_bar.tsx.tmpl
- confidence: cited
  path: crates/ui/ess-ui-react/templates/runtime/composites/form.tsx.tmpl
- confidence: inferred
  path: crates/ui/ess-ui-tui/src/view.rs
- confidence: inferred
  path: crates/ui/ess-ui/src/model.rs
- confidence: inferred
  path: schemas/ui/ess-ui.schema.yaml
- confidence: inferred
  path: website/docs/reference/ess-ui.md
revision: 22
transitions:
- {from: "draft", to: "proposed", at: "2026-11-04T08:47:30Z", actor: "human:timo", revision: 11, executor: "agent:ess-w1-lead", correlation: "ess-0.87.0-w1"}
- {from: "proposed", to: "active", at: "2026-11-04T08:47:30Z", actor: "human:timo", revision: 12, executor: "agent:ess-w1-lead", correlation: "ess-0.87.0-w1"}
---
# Story: choices, filter bars and forms render what they declare

## Outcome

A person using a generated page gets choices, filter bars and forms as the document declares them:
a dropdown with a readable empty option and a real multi-select when `multiple`, labelled filter
controls, no Save button on a form that saves on change, form actions in the declared order; and an
author can give a choice a placeholder, a reset button a label, and a form group, field or column
its help text.

## Context

A downstream adopter's reviews of ESS 0.84.0 generated pages (beyond10x/ess#723), reconcile rows
A11-A13, A15, A16, A18, A19, and the grammar keys G-13, G-14 and G-15 of the W1 decision round.
Wave W1 of the ESS reconcile plan, release 0.87.0.

## Items and causes

Each item marked R was seen on 0.84.0 and not re-read on `main`: its first test is written red
before the fix, and an item that does not reproduce on `main` is recorded as not reproduced.
Templates are under `crates/ui/ess-ui-react/templates/runtime/composites/`.

| row | ids | defect | cause |
|---|---|---|---|
| A11 | adopter report | a dropdown's empty option shows the state key; filter captions are raw binding names | `choice.tsx.tmpl:109` `props.label ?? "—"`; `crates/ui/ess-ui-react/src/emit.rs` filter-bar captions (V) |
| A12 | adopter report | the filter-bar window select has no label | `filter_bar.tsx.tmpl:39` (V) |
| A13 | adopter report | `multiple` with `style: dropdown` renders a single select | `choice.tsx.tmpl:83`, `:101` (V; React half only) |
| A15 | adopter report | a list page renders a search box with no search param declared | emitter (R) |
| A16 | adopter report | a `tree` choice sends a literal placeholder value | `choice.tsx.tmpl` (R) |
| A18 | adopter report | a `save: on_change` form still shows Save | `form.tsx.tmpl:36`, `:192` (V) |
| A19 | adopter report | form actions render destructive-first, not in the declared order | `form.tsx.tmpl` (R) |
| G-14 | adopter report | a choice has no `placeholder` | `crates/ui/ess-ui/src/model.rs:1357-1380` |
| G-15 | adopter report | `reset` takes a boolean only | `model.rs:1487-1489` |
| G-13 | adopter report | `help` is read only in the page header | `model.rs:1555-1561` (Help), `form.tsx.tmpl` renders group label only |

## Acceptance

Named scenarios; each is a test function, and the story is done when each runs and passes.
Fixtures are minimal documents with invented names. The grammar scenarios follow the answers to
G-13, G-14 and G-15; a key the round declines leaves the story.

- `crates/ui/ess-ui-react/tests/choice_filter_form.rs` (new)
  - `an_empty_dropdown_option_shows_the_placeholder_or_a_neutral_word` (A11, G-14)
  - `filter_captions_use_the_declared_label` (A11)
  - `the_window_select_is_labelled` (A12)
  - `a_multiple_dropdown_is_a_multi_select` (A13)
  - `a_list_without_a_search_param_draws_no_search_box` (A15)
  - `a_tree_choice_sends_the_chosen_value` (A16)
  - `an_on_change_form_has_no_save_button` (A18)
  - `form_actions_render_in_declared_order` (A19)
  - `a_reset_button_shows_its_label` (G-15)
  - `help_renders_on_a_form_group_a_field_and_a_column_header` (G-13)
- `crates/ui/ess-ui/tests/` and `crates/ui/ess-ui-check/tests/`: the three keys load and check;
  `website/docs/reference/ess-ui.md` is a fresh render.
- `crates/ui/ess-ui-tui/tests/`: the terminal shows the placeholder, the reset label and the help
  text where it draws the same node.

## Out of scope

- A14 (a choice with many options switches to a select): needs a default threshold, a design
  question for the next round.
- The terminal half of A13 and the runner steps (reconcile W1 U4, rows B01-B11, B33-B35).
- Collections (A10, A21-A27), headers (A28-A33).

## Fit review

1. **Need.** Choices, filters and forms ignore what the document declares, and three small
   presentation facts (placeholder, reset label, help below the page) cannot be written.
2. **Class.** A-rows: defects. G-13/14/15: gaps, each an existing key extended to a sibling
   (`placeholder` exists on search and input; `help` exists on the header; a label exists on
   submit).
3. **Already expressible?** No.
4. **Fit.** Each key reuses the shape it has on its sibling (`Help {text, link}`, `placeholder:
   <text>`, `{label}`), per the round's answers.
5. **Second adopter.** Any admin console with filtered lists and settings forms.
6. **Cost.** Generated runtime and terminal changes; three additive keys in `ess-ui/1`.
7. **Considered.** Per key in the W1 decision round (G-13, G-14, G-15), each with "change nothing".

## Decisions

Accept (wave lead, 2026-11-04): defects as reported. Grammar keys decided 2026-11-04 by the
coordinator (approval-record:ess-0870-wave): G-13 (a) `help: {text, link}` on a form group and a
field, a field covering its column header; G-14 (a) `placeholder: <text>` on a choice, also the
empty dropdown option; G-15 (a) `reset: true` or `reset: {label}`.
