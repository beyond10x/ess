---
format: aep.planning-md/3
id: story:a-value-renders-in-the-form-its-field-declares
kind: story
status: draft
title: A value renders in the form its field declares
summary: Dates from epoch ms, audio, links, object tags, JSON and list fields, byte sizes and fixtures.dir render or send as declared (A03-A08, A41, B7-1; beyond10x/ess#723)
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
- confidence: inferred
  path: crates/ui/ess-ui-check/src/rules.rs
- confidence: cited
  path: crates/ui/ess-ui-react/src/fixtures.rs
- confidence: inferred
  path: crates/ui/ess-ui-react/templates/runtime/composites/metric.tsx.tmpl
- confidence: cited
  path: crates/ui/ess-ui-react/templates/runtime/core.tsx.tmpl
- confidence: cited
  path: crates/ui/ess-ui-react/templates/runtime/fields.tsx.tmpl
- confidence: inferred
  path: crates/ui/ess-ui-tui/src/view.rs
- confidence: inferred
  path: crates/ui/ess-ui/src/model.rs
- confidence: inferred
  path: schemas/ui/ess-ui.schema.yaml
- confidence: inferred
  path: website/docs/reference/ess-ui.md
revision: 10
---
# Story: a value renders in the form its field declares

## Outcome

A person reading a generated page sees each value in the form its field's `as:` declares: an
epoch-millisecond date as a date, an audio URL as a player, a link as a link, object tags by their
label, a byte count scaled to B/KB/MB/GB; a person editing a JSON or list field sends a parsed
value and can add and remove list entries; and a placeholder read takes its fixtures from the
declared `fixtures.dir`.

## Context

A downstream adopter's reviews of ESS 0.84.0 generated pages (beyond10x/ess#723), reconcile rows
A03-A08 and A41, plus B7-1 (byte size). No UI crate changed between 0.84.0 and `main` except by the
0.86.0 wave, which did not touch these sites. Wave W1 of the ESS reconcile plan, release 0.87.0.

## Items and causes

Each item marked R was seen on 0.84.0 and not re-read on `main`: its first test is written red
before the fix, and an item that does not reproduce on `main` is recorded as not reproduced
instead of fixed.

| row | ids | defect on `main` | cause |
|---|---|---|---|
| A03 | D4, R4-G4 | an epoch-ms number under `as: date` prints a grouped integer | `crates/ui/ess-ui-react/templates/runtime/core.tsx.tmpl:552` takes the number branch before the date branch (V) |
| A04 | D2, R4-G2, AS3-10, B2-8 | `as: audio` prints the URL | `fields.tsx.tmpl:89-118` has no `audio` case (V) |
| A05 | B5-1 | `as: link` column prints the URL as text | no `link` case in `FieldValue` (R) |
| A06 | D5, K3, B2-3 | `as: tags` over objects prints `[object Object]`; a dotted path into a list renders nothing | `fields.tsx.tmpl:98-103` `String(tag)`; `getPath` (R) |
| A07 | B4-48 | `as: json` field sends the textarea string | `FieldInput` `json` case `fields.tsx.tmpl:181-190` (R) |
| A08 | R4-G11 | `as: list` field has no add/remove editor | no `list` case in `FieldInput` (V) |
| A41 | AS3-14, E1-1 | `fixtures.dir` ignored for placeholder `fixture:` reads | `crates/ui/ess-ui-react/src/fixtures.rs:210` joins `source_dir`, not the dir resolved at `:161` (V) |
| B7-1 | B7-1 | a byte count shows as a raw integer; metric `format: bytes` prints `<n> B` unscaled | no `bytes` in field `as:` (`schemas/ui/ess-ui.schema.yaml:944`); `core.tsx.tmpl:563-564` |

## Acceptance

Named scenarios; each is a test function, and the story is done when each runs and passes.
Fixtures are minimal documents with invented names.

- `crates/ui/ess-ui-react/tests/value_formats.rs` (new)
  - `an_epoch_ms_number_under_as_date_renders_a_date` (A03)
  - `an_audio_field_renders_a_player` (A04)
  - `a_link_column_renders_an_anchor` (A05)
  - `object_tags_render_their_label` and `a_dotted_path_into_a_list_renders_its_values` (A06)
  - `a_json_field_sends_the_parsed_value_and_refuses_unparsable_text_at_the_field` (A07)
  - `a_list_field_adds_and_removes_entries` (A08)
  - `a_bytes_field_scales_in_1024_steps` and `a_bytes_metric_scales_in_1024_steps` (B7-1, after Q-B7)
- `crates/ui/ess-ui-react/tests/fixtures_dir.rs` (new or existing fixtures target)
  - `a_placeholder_read_takes_its_fixture_from_fixtures_dir` (A41)
- `crates/ui/ess-ui/tests/` and `crates/ui/ess-ui-check/tests/`: `as: bytes` loads and checks (B7-1,
  after Q-B7); `website/docs/reference/ess-ui.md` is a fresh render (`ess-ui-docs --test reference`).
- `crates/ui/ess-ui-tui/tests/value_formats.rs`: the terminal prints a scaled byte count (B7-1).

## Out of scope

- `as: duration` units and `as: date` locale or format options (A01, A02): they wait for G-1 and G-2.
- Collections, choices, forms, headers, themes.

## Fit review

1. **Need.** Values the document already types (`as:`) are shown or sent in the wrong form; one
   display form (byte size) has no word for a field.
2. **Class.** A03-A08, A41: defects (the renderer ignores what the field declares, or the emitter
   ignores a declared key). B7-1: gap, an existing word (`bytes`, metric `format:`,
   `ess-ui.schema.yaml:791`) missing on its sibling, the field.
3. **Already expressible?** No; the adopter's workaround is a local runtime patch per item.
4. **Fit.** No new surface for the defects. B7-1 reuses the metric's existing word on the field, so
   one word means one thing on both siblings; both renderers draw it.
5. **Second adopter.** An attachments list with file sizes; a call log with recordings.
6. **Cost.** Generated runtime changes; B7-1 adds one enum word to `ess-ui/1` (additive).
7. **Considered.** For B7-1: (a) `as: bytes` on the field (chosen, pending Q-B7); (b) a `format:`
   key on fields (new key for one word); (c) nothing (the reader sees raw integers).

## Decisions

Accept (wave lead, 2026-11-04): defects as reported; B7-1 redesigned from "`as: bytes` or
`format: bytes`" to the metric's existing word on the field, pending Q-B7.
