---
format: aep.planning-md/3
id: story:the-loader-fills-the-defaults-a-renderer-reads-differently-today-and-both-renderers-then-show-what-the-schema
kind: story
status: draft
title: The loader fills the defaults a renderer reads differently today, and both renderers then show what the schema documents for each
tags:
- gap-p0
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:the-ess-ui-loader-writes-every-constant-default-the-schema-declares-into-the-document-before-reading-it-from
revision: 1
---
## Need

`g03-loader-fills-defaults` fills only the defaults on which both renderers already agree with the
schema. For the rows marked **B** in its table, the renderers read a value when one is written
but fall back to something else when it is absent. Filling them moves behaviour, which is the point:
both renderers then show what `ess-ui/1` documents. Evidence, from that table:

- **`DynamicNavEntries.label`, schema `row.label`.** Both renderers show `row.id`
  (`crates/ui/ess-ui-react/templates/runtime/navigation.tsx.tmpl:122`,
  `crates/ui/ess-ui-tui/src/app.rs:1062-1066`).
- **`SectionStates.loading`, schema `skeleton`.** The terminal draws skeleton bars only for an
  explicit `skeleton` and otherwise one "loading…" line (`view.rs:730-736`).
- **`choice.style`, schema `dropdown`.** React draws a checklist for a multiple choice with no
  style (`composites/choice.tsx.tmpl:89-91`). The Playwright emitter mirrors this
  (`crates/ui/ess-ui-test/src/playwright.rs:1023-1026`).
- **`confirm.confirm_label`, schema `Delete`.** Both renderers show `Confirm`
  (`composites/confirm.tsx.tmpl:131`, `view.rs:2417-2420`).
- **`confirm.danger`, schema `true`.** The terminal marks a confirm destructive only for an
  explicit `true` (`view.rs:2421`).
- **`rich_text.syntax`, schema `expression`.** Both renderers use `plain`
  (`composites/rich_text.tsx.tmpl:21`, `view.rs:1221`).
- **`references.columns`, schema `[label, type]`.**
  - serde gives `[]` (`crates/ui/ess-ui/src/model.rs:2196-2197`).
  - React shows `label ?? id` and then the type (`composites/references.tsx.tmpl:29-31`).
  - The terminal shows every key of the first row (`app.rs:5090-5121`, `:5193-5194`).
- **`Widget.arrange`, schema `column`.** The terminal draws an absent value as one row
  (`view.rs:997`).
- **`Field.label`, schema `{from: model.field_label}`.**
  - No ESS model has a field label. The loader's model input offers enums only
    (`crates/ui/ess-ui/src/binding.rs:97-115`), and `git grep field_label` finds only the schema
    and its generated reference.
  - React shows the name with `_` as a space (`fields.tsx.tmpl:117-118`). The terminal shows the
    raw field (`view.rs:2771-2773`), except in its parity text (`view.rs:2808-2813`).
- **`Field.tone`, schema `neutral`.** `tone_of` (`model.rs:2565-2575`) and both renderers apply
  neutral only beside a `tone_by`. Filling `neutral` everywhere would draw every terminal cell
  bold (`view.rs:50-52`).
- **`Reads.key` and `Live.match`.** Each defaults to the other, else `id`. A choice picks the value
  it sends by trying `reads.key` first (`model.rs:1629-1633`), so writing `key: id` into a choice's
  read would change what it sends. A row without `id` would then offer nothing.
- **`Reads.paging`, schema `none`.** Absent means `none` in both renderers, but React adds
  `paging=` to the request and to its cache key only when the key is written
  (`templates/runtime/data.ts.tmpl:340-342`, `:829`).
- **`State.clear_on`, schema `[signout, account_switch]`.** serde gives `[]`
  (`model.rs:3416-3417`).

`Document.actor` is also a B row, but it changes the run of nearly every test document, so it is
its own unit, `g03-actor-from-session`.

## Outcome

The fill step of `g03-loader-fills-defaults` also writes these keys:

- `DynamicNavEntries.label`, `SectionStates.loading`, `choice.style`, `confirm.confirm_label`,
  `confirm.danger`, `rich_text.syntax`, `references.columns`, `Widget.arrange`, `Reads.paging` and
  `State.clear_on`, each with the schema's value. This is P0-D17 (A).
- `Field.label`: the field's name with each `_` written as a space, which is what React shows
  today. The schema's note and default are corrected to say so (P0-D12 A). The terminal then
  shows the filled label.
- `Field.tone: neutral`, only on a field that writes a `tone_by`. The schema's note is corrected
  to what `tone_of` does (P0-D13 A).
- `Reads.key` and `Live.match` (P0-D16 A):
  - Where one of them is written, the other is filled from it.
  - Where neither is written, both are filled with `id`, except on a choice's `reads`, which keeps
    its own value order and has that order written in the schema's note.
  - The deserialiser's fill of `live.match` (`model.rs:926-930`, `:1127-1129`) is removed.
- React sends `paging` only for a value other than `none`, so its requests and cache keys stay as
  they are.
- Both renderers now show the schema's documented behaviour for each key above.

## Acceptance

New test files: `crates/ui/ess-ui-tui/tests/schema_defaults.rs`,
`crates/ui/ess-ui-react/tests/schema_defaults.rs`, and cases appended to
`crates/ui/ess-ui/tests/defaults.rs`. Each test is expected under 5 s (React tests that run the
generated runtime under 10 s) and is timed. None may run over 60 s, and each file's step stays
within 120 s.

1. `an_absent_key_behaves_as_documented`. One minimal document leaves out every key above. Red
   today:
   - the terminal shows `row.label` in the dynamic navigation, skeleton bars while loading, a
     confirm labelled `Delete` and marked destructive, rich text with expression syntax, the
     references columns `label` and `type`, a widget body as a column, and a column labelled with
     its name with `_` as a space;
   - the generated React project shows `row.label`, a dropdown for the multiple choice, `Delete`,
     expression syntax and the columns `label` and `type`.
2. `a_written_key_is_obeyed`. The same document, with each key written to a value other than its
   default, shows that value in both renderers (a control).
3. `a_choice_sends_what_it_sent`. A choice whose read writes no `key` sends the same values it
   sent before this unit, in both renderers (`choice_value.rs` cases stay green).
4. `react_requests_do_not_move`. A bound read without `paging` sends the same URL as before, and
   its cache key is unchanged.
5. `ess ui check` reports the same findings for every corpus document. A `references` list whose
   model view has no `label` or `type` field is reported with `field_in_model`, at the filled
   column, and the finding names the default it came from.
6. Each existing test whose expectation moves because of one of these keys is updated, with the
   key and this story named beside it. The implementer lists them in the report.

Also:

- `schemas/ui/ess-ui.schema.yaml` corrects `Field.label`'s default and the notes of `Field.tone`
  and `Reads.key`. `website/docs/reference/ess-ui.md` is regenerated (`ess ui docs`).
- `CHANGELOG.md`, under Changed, lists each behaviour that moved, by key, as a fix to what
  `ess-ui/1` documents.

## Fit review

Read at tree `bcfc51555`. The request came from a downstream application on 2026-11-24, report
item 3, with the owner's order "the loader fills every default".

1. **Need.** One value per key for every generator, stock or own. The evidence is above: the stock
   renderers disagree with the schema on each of these keys.
2. **Class.** These are *defects*. A renderer contradicts the default `ess-ui/1` documents
   (`website/docs/reference/ess-ui.md`, generated from the schema).
3. **Already expressible?** An author can write each key out, and the renderers obey a written
   value. The defect is the absent case.
4. **Fit.**
   - There is no new key.
   - Three schema texts are corrected where the schema names something ESS has never had
     (`model.field_label`), or contradicts ESS's own typed accessor (`tone_of`) or its choice
     semantics (`Reads.key` on a choice). In each case every implementation already behaves as the
     corrected text says, so no `ess-ui/1` version moves.
5. **Second adopter.** Any document that leaves `confirm_label` out and expects the documented
   destructive confirm.
6. **Cost.**
   - Visible changes in both renderers for documents that leave these keys out, each a fix.
   - Test expectations move, each named.
   - No format version, no migration of authored documents.
7. **Considered.**
   - (a) Correct the schema to what the renderers do, for every key: P0-D17 (B).
   - (b) Fill only the keys where the renderers agree. That leaves the request unmet.
   - (c) Chosen: the schema's side, except the three corrections above, each its own decision.

## Decisions

- **P0-D12: What is `Field.label`'s default, when no ESS model has a field label?**
  - (A) The field's name with `_` as a space, filled by the loader. The schema is corrected.
  - (B) Add a field label to the specification language. That is a new `ess/39` construct, size L,
    outside the UI area.
  - (C) Leave it absent and keep the renderers' own fallbacks.

  **Recommended: (A).**
- **P0-D13: Where is `Field.tone: neutral` filled?**
  - (A) Only beside a `tone_by`, as `tone_of` and both renderers do. The schema note is corrected.
  - (B) Everywhere, as the note says. Every terminal cell is then drawn bold, and every React cell
    gets a tone class.
  - (C) Nowhere.

  **Recommended: (A).**
- **P0-D16: How are `Reads.key` and `Live.match` filled?**
  - (A) Each from the other, else `id`, except a choice's read.
  - (B) `id` everywhere. A choice then changes the value it sends.
  - (C) Only from each other, never `id`.

  **Recommended: (A).**
- **P0-D17: Where both renderers agree with each other against the schema** (`confirm_label`,
  `rich_text.syntax`, `DynamicNavEntries.label`, and `Document.actor` in `g03-actor-from-session`),
  which side does the fill take?
  - (A) The schema, as ordered. Behaviour moves in both renderers.
  - (B) The schema's default is corrected to what both renderers do. No behaviour moves, and the
    reference changes.
  - (C) Per key.

  **Recommended: (A).**

## Scope

- cited: `crates/ui/ess-ui/src/fill.rs` (from `g03-loader-fills-defaults`: more keys and the
  key/match rule), `crates/ui/ess-ui/src/model.rs` (926-930, 1127-1129: drop the deserialiser fill)
- cited: `crates/ui/ess-ui-tui/src/view.rs` (2771-2773: read the filled label)
- cited: `crates/ui/ess-ui-react/templates/runtime/data.ts.tmpl` (340-342: send `paging` only when
  not `none`)
- cited: `schemas/ui/ess-ui.schema.yaml` (961, 971, 1284), `website/docs/reference/ess-ui.md`
  (generated)
- new: the three test files named above; updated expectations in existing tests
- `CHANGELOG.md`
- Overlap: `story:a-chart-names-its-axes-and-formats-its-ticks` (active; `view.rs`) and
  `story:an-action-names-its-icon` (active; React templates). Different functions.
- Size: M, about 480 changed lines (code about 60, new tests about 300, moved expectations about
  120).

## Depends on

- `g03-loader-fills-defaults`
