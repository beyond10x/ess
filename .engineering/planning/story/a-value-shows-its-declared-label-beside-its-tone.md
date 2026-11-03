---
format: aep.planning-md/3
id: story:a-value-shows-its-declared-label-beside-its-tone
kind: story
status: active
title: A value shows its declared label beside its tone
summary: Boolean and enum values on columns and badges show labels the document declares (labels:, label_maps), in React and the terminal (D17, B7; beyond10x/ess#723)
tags:
- ess-ui-adopter-wave-1
- from-consumer-report
refs:
- provider: github
  reference: beyond10x/ess#723
relations:
- serves: vision:O2
scope:
- confidence: cited
  path: crates/ui/ess-ui-check/src/classify.rs
- confidence: cited
  path: crates/ui/ess-ui-check/src/lib.rs
- confidence: cited
  path: crates/ui/ess-ui-react/src/emit.rs
- confidence: cited
  path: crates/ui/ess-ui-react/templates/runtime/fields.tsx.tmpl
- confidence: cited
  path: crates/ui/ess-ui-react/templates/runtime/primitives/badge.tsx.tmpl
- confidence: cited
  path: crates/ui/ess-ui-tui/src/app.rs
- confidence: cited
  path: crates/ui/ess-ui-tui/src/view.rs
- confidence: cited
  path: crates/ui/ess-ui/src/expand.rs
- confidence: cited
  path: crates/ui/ess-ui/src/lib.rs
- confidence: cited
  path: crates/ui/ess-ui/src/model.rs
- confidence: cited
  path: schemas/ui/ess-ui.schema.yaml
- confidence: cited
  path: website/docs/reference/ess-ui.md
revision: 14
transitions:
- {from: "draft", to: "proposed", at: "2026-11-03T16:24:10Z", actor: "human:timo", revision: 11}
- {from: "proposed", to: "active", at: "2026-11-03T16:24:11Z", actor: "human:timo", revision: 12}
---
# Story: a value shows its declared label beside its tone

## Outcome

A person reading a generated list or detail page sees the words the document declares for a
boolean or enum value (`Active`, `In progress`) instead of the raw value (`true`, `in_progress`),
in the React app and in the terminal app, while the value's tone still comes from the raw value.

## Context

A downstream adopter's review of ESS 0.84.0 generated pages (beyond10x/ess#723, items D17, B7, B2).
Wave 1 of the ESS UI-adopter waves (D418, D420), release 0.86.0.

## Acceptance

Named scenarios; each is a test function, and the story is done when each runs and passes.

- `crates/ui/ess-ui/tests/value_labels.rs`
  - `a_field_takes_labels_written_out` — a column `{field: state, as: badge, labels: {in_progress: In progress}}` loads, and the typed field carries the map.
  - `a_field_takes_labels_by_name` — `labels: run_state` resolves to the document's `label_maps.run_state`, as `tones:` resolves to `tone_maps`.
  - `a_badge_takes_labels` — a `badge` primitive with `labels:` loads, both forms.
  - `boolean_values_are_keyed_true_and_false` — `labels: {"true": Active, "false": Inactive}` loads and a `true` value finds `Active`.
- `crates/ui/ess-ui-check/tests/value_labels.rs`
  - `an_unknown_label_map_name_is_an_error` — `labels: nope` with no `label_maps.nope` is refused by a named rule (`label_map_refs`) at the field's path.
  - `an_unused_label_map_is_a_warning` — a `label_maps` entry no field or badge names warns (`label_map_unused`).
- `crates/ui/ess-ui-react/tests/value_labels.rs`
  - `a_badge_column_shows_the_label_and_keeps_the_raw_value_class` — the generated runtime and page render `In progress` for `in_progress`, and the badge's class still names the raw value (`ui-badge-in_progress`).
  - `a_boolean_column_shows_its_labels` — `true` renders `Active`, `false` renders `Inactive`.
  - `a_value_with_no_label_shows_the_raw_value` — a value the map does not name renders as today.
  - `a_choice_column_shows_its_option_labels` — a column `as: choice` whose `choice` declares fixed `options` with `label` shows the option's label for the row value (B7).
- `crates/ui/ess-ui-tui/tests/value_labels.rs`
  - `a_badge_column_shows_the_label` — the terminal renders `[In progress]` for `in_progress`.
  - `a_badge_primitive_shows_the_label` — a badge primitive with `labels:` renders the label.
- The reference page (`website/docs/reference/ess-ui.md`) is a fresh render of the schema (`ess-ui-docs --test reference`).

## Out of Scope

- `tone_by` on a column (C18, `story:a-badge-field-takes-a-tone`, a draft) and any change to how tones are chosen; the next UI-adopter wave.
- Labels derived from field names or a built-in word list: the runtime carries no vocabulary.
- Labels taken from the ESS model's enum declarations: an ESS enum declares no display text
  (`crates/specify/ess-domain/src/types.rs:644-700`, variants carry a name, a wire spelling and
  typed attributes only).

## Problem

A column `as: badge` prints `formatValue(value)` (`crates/ui/ess-ui-react/templates/runtime/fields.tsx.tmpl:82-85`);
a `badge` primitive prints its text; the terminal prints `[<value>]` (`crates/ui/ess-ui-tui/src/view.rs:1424`).
The document can map a value to a tone (`tone_maps`, `crates/ui/ess-ui/src/model.rs:231-233`;
`tone_by`, `model.rs:2075-2087`) but not to a label, so `true` and `in_progress` reach the reader.

## Fit review

1. **Need.** A value has a word for people that differs from its stored form, and the document
   cannot say it. Requester's syntax: "value labels beside tones (`labels: {"true": Active, "false": Inactive}`)".
   The adopter's reference fix derived words from the field's name in the runtime and says the real
   fix is a label per value in the grammar.
2. **Class.** Gap: no construct maps a value to display text (`schemas/ui/ess-ui.schema.yaml:937`,
   `:1065-1081`).
3. **Already expressible?** No. `label_from` (`model.rs:2215-2231`) reads a related view, not a fixed
   map; a `choice` option label applies only to the choice control.
4. **Fit.** Mirrors the tone chain: a document-level named map (`label_maps` beside `tone_maps`) and a
   per-node key (`labels:` beside `tone_by`), written out or by name, checked like
   `tone_map_refs`/`tone_map_unused` (`ess-ui.schema.yaml:1673-1674`). Applies on both siblings that
   show a value as a pill (a field and a badge primitive) and on both renderers.
5. **Second adopter.** An order list showing `pending_payment` as "Awaiting payment" and a boolean
   `archived` as "Archived"/"Live".
6. **Cost.** Two optional keys in `ess-ui/1` (additive; documents without them load and render as
   before), two check rules, runtime and terminal rendering. No ESS model format change.
7. **Considered.** (a) change nothing: rejected, the reader sees raw values. (b) `{tone, label}`
   entries in `tone_maps`: rejected, changes the type every `tone_by` reads and couples labels to
   tones on nodes that have no tone (a column). (c) derive words in the runtime (the adopter's
   reference fix): rejected, a vocabulary in generated code cannot be checked or translated.
   (d) chosen: `labels:` plus `label_maps`.

## Decisions

Accept, redesigned (2026-11-03, wave lead under D420): the requester's `labels:` map is taken as the
written-out form; a named form (`label_maps`) is added so a label set is declared once, as tones are.
B7 ("Enum labels declared in the spec are not applied to badges", quoted from the adopter list) is
read as labels the document declares, including a choice column's option labels; whether it meant
something else is unclear and was not asked.

## Delivered

UI-adopter wave 1 (D418, D420), release 0.86.0.

- Unit `fe9b8046d8` merged at `27fb9d4b22`; fix round 1 `4f160cbe6f` merged at `2a329a6b5c`.
- `labels:` takes a map written out or the name of a `label_maps` entry, resolved in `model.rs` and `expand.rs` as `tone_maps` are. It is a value map only in a mapping with `field:` text or `primitive: badge`, so a node or entry named `labels` elsewhere loads as before.
- The React runtime (`fields.tsx`, `badge.tsx`) and the terminal (`view.rs`) show the label, and a badge keeps the raw value in its class and tone. A choice column with fixed `options` shows the option label.
- `ess ui check` reports `label_map_refs` (error) and `label_map_unused` (warning).
- Every named acceptance scenario passes. Adversary pass 1 needed revision and was fixed in round 1; pass 2 approved, and its 14 cases are kept (`adversary_w086_u1_pass2.rs`).
- Scope: the cited lines are confirmed by the diff. `style.rs` was not touched; `classify.rs`, `lib.rs` and `badge.tsx.tmpl` were added.
- Out of scope: a node named `tone_by` that is not a tone is `story:a-node-named-tone-by-is-not-a-tone` (backlog).
