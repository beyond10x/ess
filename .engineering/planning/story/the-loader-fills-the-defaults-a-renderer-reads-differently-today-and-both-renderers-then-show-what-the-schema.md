---
format: aep.planning-md/3
id: story:the-loader-fills-the-defaults-a-renderer-reads-differently-today-and-both-renderers-then-show-what-the-schema
kind: story
status: active
title: The loader fills the defaults the renderers disagree on, and the schema states the ones both renderers already use
tags:
- gap-p0
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:the-ess-ui-loader-writes-every-constant-default-the-schema-declares-into-the-document-before-reading-it-from
- depends_on: story:a-document-that-leaves-out-actor-runs-from-the-session-in-both-renderers-as-ess-ui-1-documents-and-the-test
- depends_on: story:the-loader-resolves-every-state-s-store-and-scope-and-every-read-s-cache-store-from-the-placement-profile-once
- depends_on: story:with-a-model-the-loader-fills-a-form-s-fields-from-its-command-s-input-fields-and-a-form-without-fields-shows
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-11-24T01:19:10Z", actor: "human:timo", revision: 4}
- {from: "proposed", to: "active", at: "2026-11-24T01:19:11Z", actor: "human:timo", revision: 5}
---
## Need

`g03-loader-fills-defaults` fills only the defaults on which every renderer already agrees with the
schema. For the rows marked **B** in its table, a renderer reads a written value but falls back to
something else when the key is absent, or the schema states a default that no ESS code has ever
had. Evidence, from that table:

- **Both renderers agree with each other against the schema** (P0-D17, settled per key by the
  operator on 2026-11-24 as B, the schema corrected):
  - `DynamicNavEntries.label`, schema `row.label`. Both show `row.id`
    (`crates/ui/ess-ui-react/templates/runtime/navigation.tsx.tmpl:122`,
    `crates/ui/ess-ui-tui/src/app.rs:1062-1066`).
  - `confirm.confirm_label`, schema `Delete`. Both show `Confirm`
    (`composites/confirm.tsx.tmpl:131`, `view.rs:2417-2420`).
  - `rich_text.syntax`, schema `expression`. Both use `plain` (`composites/rich_text.tsx.tmpl:21`,
    `view.rs:1221`).
- **One renderer disagrees with the schema and with the other renderer:**
  - `SectionStates.loading`, schema `skeleton`. The terminal draws skeleton bars only for an
    explicit `skeleton`, and otherwise one "loading…" line (`view.rs:730-736`).
  - `choice.style`, schema `dropdown`. React draws a checklist for a multiple choice with no style
    (`composites/choice.tsx.tmpl:89-91`), and the Playwright emitter mirrors this
    (`crates/ui/ess-ui-test/src/playwright.rs:1023-1026`).
  - `confirm.danger`, schema `true`. The terminal marks a confirm destructive only for an explicit
    `true` (`view.rs:2421`).
  - `references.columns`, schema `[label, type]`. serde gives `[]`
    (`crates/ui/ess-ui/src/model.rs:2196-2197`). React shows `label ?? id` and then the type
    (`composites/references.tsx.tmpl:29-31`). The terminal shows every key of the first row
    (`app.rs:5090-5121`, `:5193-5194`).
  - `Widget.arrange`, schema `column`. The terminal draws an absent value as one row
    (`view.rs:997`).
- **The renderers differ, and the schema names nothing ESS has:**
  - `Field.label`, schema `{from: model.field_label}` (`schemas/ui/ess-ui.schema.yaml:961`). No
    ESS model has a field label: the loader's model input offers enums only
    (`crates/ui/ess-ui/src/binding.rs:97-115`), and `git grep field_label` finds only the schema
    and its generated reference.
  - React shows the name with `_` as a space (`fields.tsx.tmpl:117-118`). The terminal shows the
    raw field (`view.rs:2771-2773`), except in its parity text (`view.rs:2808-2813`).
- **Keys whose fill moves nothing, if it is shaped right:**
  - `Field.tone`, schema `neutral` (`:971`). `tone_of` (`model.rs:2565-2575`) and both renderers
    apply neutral only beside a `tone_by`. Filling `neutral` everywhere would draw every terminal
    cell bold (`view.rs:50-52`).
  - `Reads.key` and `Live.match` (`:1284`, `:1419`). Each defaults to the other, else `id`. A
    choice picks the value it sends by trying `reads.key` first (`model.rs:1629-1633`), so writing
    `key: id` into a choice's read would change what it sends.
  - `Reads.paging`, schema `none`. Absent means `none` in both renderers, but React adds
    `paging=` to the request and to its cache key only when the key is written
    (`templates/runtime/data.ts.tmpl:340-342`, `:829`).
  - `State.clear_on`, schema `[signout, account_switch]`. serde gives `[]`
    (`model.rs:3416-3417`). No renderer reads the value
    (`git grep clear_on -- crates/ui` finds only the checker's enum check,
    `ess-ui-check/src/enums.rs:100`).

## Outcome

The rules pass (`crates/ui/ess-ui/src/fill/behaviour.rs`, the fourth pass) writes these keys: the
rows of that file's `HELD` list where their condition holds. The list stays, so the constant pass
never writes them (not a `Field.tone` without `tone_by`).

- **Corrected and filled, no behaviour moves** (P0-D17 B). The schema's default is corrected to
  what both renderers do, and the fill writes that value:
  - `DynamicNavEntries.label: row.id`;
  - `confirm.confirm_label: Confirm`;
  - `rich_text.syntax: plain`.
- **Filled with the schema's value, behaviour moves** in the renderer that disagreed:
  `SectionStates.loading`, `choice.style`, `confirm.danger`, `references.columns` and
  `Widget.arrange`.
- **`Field.label`** (P0-D12 A). The fill writes the field's name with each `_` as a space. The
  schema's default and note are corrected to say so.
- **`Field.tone: neutral`**, only on a field that writes a `tone_by` (P0-D13 A). The schema's note
  is corrected to what `tone_of` does.
- **`Reads.key` and `Live.match`** (P0-D16 A).
  - Where one of them is written, the other is filled from it.
  - Where neither is written, both are filled with `id`, except on a choice's `reads`. A choice
    keeps its own value order, which is written into the schema's note.
  - The deserialiser's fill of `live.match` (`model.rs:926-930`, `:1127-1129`) is removed.
- **`Reads.paging: none`.** React sends `paging` only for a value other than `none`, so its
  requests and cache keys stay as they are.
- **`State.clear_on`.** `Document` holds `[signout, account_switch]` where it held `[]`. No
  renderer clears by it, so nothing moves. Clearing by `clear_on` is a defect outside P0.
- **Before and after, per renderer.** Only the rows that move are listed. Every other key of this
  unit moves nothing in either renderer.

  | key (document leaves it out) | terminal before | terminal after | React before | React after |
  |---|---|---|---|---|
  | `SectionStates.loading` | one "loading…" line | skeleton bars | skeleton | skeleton |
  | `choice.style` on a `multiple` choice | its choice list (style ignored) | unchanged | checklist; the Playwright spec clicks checkboxes | dropdown; the spec chooses from the dropdown |
  | `confirm.danger` | not marked destructive | marked destructive | danger | danger |
  | `references.columns` | every key of the first row | columns `label` and `type` | `label ?? id`, then `type` | columns `label` and `type`; a row without `label` shows an empty label cell |
  | `Widget.arrange` | one row | a column | a column | a column |
  | `Field.label` | the raw field name (`order_id`) | the name with `_` as a space (`order id`) | `order id` | `order id` |

- **The fill is complete.** With this unit, the last fill unit, every row on a `HELD` list is
  written by its own pass where its condition holds, and the constant pass writes none of them.

## Acceptance

New test files:
- `crates/ui/ess-ui/tests/defaults_behaviour.rs`;
- `crates/ui/ess-ui-tui/tests/schema_defaults.rs`;
- `crates/ui/ess-ui-react/tests/schema_defaults.rs`.

Each test is expected under 5 s, or 10 s where it runs the generated runtime, and is timed; none may
run over 60 s, and each crate's test step stays within 120 s. Each file joins its crate's single
test binary of `story:the-test-step-meets-the-test-budget` if that has landed; otherwise it is a
new file, which that story moves.

**The filled tree** (`defaults_behaviour.rs`). The test reads `load_resolved`. Each case is red
today.

1. `corrected_defaults_are_filled`. A document leaving out the three P0-D17 keys holds
   `row.id`, `Confirm` and `plain`.
2. `moving_defaults_are_filled`. It holds `loading: skeleton`, `style: dropdown`, `danger: true`,
   `columns: [label, type]` and `arrange: column`.
3. `a_field_label_is_its_name_spelled_out`. A field `order_id` with no `label` holds
   `label: order id`. A written `label` is kept.
4. `tone_is_filled_only_beside_tone_by`. A field with `tone_by` holds `tone: neutral`, and a field
   without one holds no `tone`.
5. `key_and_match_are_filled_from_each_other`. Four cases:
   - `key: code` written gives `live.match: code`;
   - `live.match: code` written gives `key: code`;
   - neither written gives both `id`;
   - a choice's read with neither written holds neither.
6. `clear_on_and_paging_are_filled`. `clear_on: [signout, account_switch]` and `paging: none`.
7. `the_fill_is_complete`. Every row on a `HELD` list of `crates/ui/ess-ui/src/fill/` is written
   by its own pass where its condition holds, and the constant pass writes none of them:
   - for `every-construct.yaml` loaded with a model, the tree holds every (construct, key) pair of
     `every_default_is_placed` (`g03-loader-fills-defaults`) except two kinds: those on
     `NOT_FILLED`, and those whose "when" condition the document does not meet. The test names the
     second kind;
   - a `HELD` row whose condition the document does not meet stays absent, which only the
     constant pass could have written: loaded without a model, `form.fields` is absent, and a
     field without `tone_by` holds no `tone`.

**Behaviour, one test per moving key.** In `schema_defaults.rs` of each renderer, each test is red
today in the renderer that moves. A control writes the key to another value and shows that value.

8. `loading_draws_skeleton_bars` (terminal). The first load of a section with 1 s latency draws
   skeleton bars and no "loading…" line.
9. `a_multiple_choice_is_a_dropdown` (React). The emitted choice renders a dropdown. The
   Playwright spec that `ess ui test --playwright` writes for a `choose` step on it holds a
   dropdown selection and no checkbox click, asserted on the spec text.
10. `a_confirm_is_destructive` (terminal). The confirm overlay is drawn with the destructive mark.
11. `references_show_label_and_type` (both). The terminal shows the columns `label` and `type`
    only. React shows the same two columns.
12. `a_widget_body_is_a_column` (terminal). The widget's nodes are drawn one per line.
13. `a_column_is_labelled_by_its_name` (terminal). The column header for `order_id` reads
    `order id`.

**No move where none is meant:**

14. `a_choice_sends_what_it_sent`. The `choice_value.rs` cases of `ess-ui-tui` and `ess-ui-react`
    pass unchanged.
15. `react_requests_do_not_move`. A bound read without `paging` sends the URL it sent before, and
    its cache key is unchanged.
16. `corrected_defaults_move_nothing`. In both renderers, the dynamic navigation, the confirm
    label and the rich text render as before, with the screen text and emitted text asserted.
17. `ess-ui-check` findings do not move. Its test set passes with no expectation changed outside
    `model_defaults.rs` (whose filled entries gain `label`, below):
    `ess-ui-check` reads none of `references.columns` (`git grep` finds no reader in
    `crates/ui/ess-ui-check/src`), the confirm label, or the syntax.

**Moved expectations.** Only these existing files may change:
- in `crates/ui/ess-ui-tui/tests/`: `confirm.rs`, `adversary_tui_live_pass1.rs`,
  `adversary_tui_live_pass2.rs`, `http_adapter.rs`, `document_model.rs`, `literal_text.rs`,
  `row_states.rs`, `choice_value.rs` and `shared_frame.rs`;
- in `crates/ui/ess-ui-react/tests/`: `adversary_pass1.rs` and `choice_filter_form.rs`;
- in `crates/ui/ess-ui-test/tests/`: `adversary_w088_u3_pass1.rs`, `journey_controls.rs`,
  `form_toggle.rs` and `playwright.rs`;
- the round-2 files of `g03-model-defaults`: `crates/ui/ess-ui-check/tests/model_defaults.rs`
  (test 1's three entries gain `label: order id`, `label: note`, `label: priority`) and
  `crates/ui/ess-ui-test/tests/model_defaults.rs` (test 3's step expects
  `text: [order id, note, priority]`, since the terminal draws form inputs through `label_of`,
  `view.rs:2197`, `:2771-2773`);
- `examples/partner-portal/tests/partners-list.yaml`, `live.yaml` and `stale.yaml`.

They were found by `git grep` for documents with `component: confirm`, `widgets:`,
`component: references`, `multiple: true` or a snake_case field name, in those directories. Each
moved line names its key and this story. A move in any other file stops the unit and is reported.

Gate:
- `task fmt-check`;
- `cargo clippy -p ess-ui -p ess-ui-tui -p ess-ui-react --all-targets --locked -- -D warnings`;
- `cargo test -p ess-ui -p ess-ui-tui -p ess-ui-react -p ess-ui-test -p ess-ui-check -p ess-ui-docs --locked`;
- `task test-xtask`.

Also:
- `schemas/ui/ess-ui.schema.yaml` corrects five texts:
  - the defaults of `DynamicNavEntries.label`, `confirm.confirm_label`, `rich_text.syntax` and
    `Field.label`;
  - the notes of `Field.tone` and `Reads.key`.

  `website/docs/reference/ess-ui.md` is regenerated by `cargo run -q -p ess-cli -- ui docs`.
- `CHANGELOG.md`, under Changed:
  - "The terminal draws skeleton bars for a loading section, marks a confirm destructive, shows a
    references list's `label` and `type`, draws a widget body as a column, and labels a column by
    its name with `_` as a space, where the document leaves these keys out, as `ess-ui/1`
    documents."
  - "**Breaking for a generated React project**: a multiple choice without `style` is a
    dropdown, and a references list without `columns` shows `label` and `type`; write `style:
    checklist` or `columns` for the earlier look."
  - "The `ess-ui/1` reference now states the defaults both renderers already used: a dynamic
    navigation entry's `label` is `row.id`, a confirm's `confirm_label` is `Confirm`, a rich
    text's `syntax` is `plain`, and a field's `label` is its name with `_` as a space."

## Fit review

Read at tree `bcfc51555`. The request came from a downstream application on 2026-11-24, report
item 3, with the owner's order "the loader fills every default".

1. **Need.** One value per key for every generator, stock or own. The evidence is above.
2. **Class.** The moving keys are *defects*: one renderer contradicts the documented default and
   the other renderer. The corrected keys are *documentation defects*: the schema states what no
   implementation does.
3. **Already expressible?** An author can write each key, and the renderers obey a written value.
   The defect is the absent case.
4. **Fit.**
   - No new key.
   - No `ess-ui/1` version, for these reasons:
     - The three P0-D17 keys are corrected to what both renderers already do, so no accepted
       document changes what it does.
     - The old `Field.label` default named `model.field_label`, which no model has, so no
       document could have had that meaning. The terminal's label moves, which brings it to
       React's.
     - `Field.tone` and `Reads.key` are notes corrected to ESS's own typed accessor (`tone_of`)
       and to the choice semantics, which every implementation already follows.
     - The moving keys move toward the documented meaning, so the meaning of `ess-ui/1` does not
       change.
5. **Second adopter.** Any document that leaves `danger` out and expects the documented
   destructive confirm.
6. **Cost.**
   - Visible changes in the terminal and in generated React projects for documents that leave the
     moving keys out, each named in the changelog.
   - The test expectations move in the named files.
   - No format version, no migration of authored documents.
7. **Considered.**
   - (a) The schema's side for every key, P0-D17 A. Rejected by the operator for the three keys
     where both renderers agree: nearly every test document's two renderers would disagree with
     their old behaviour.
   - (b) Fill only the keys where the renderers agree. That leaves the request unmet.
   - (c) Chosen: correct the schema where both renderers agree, and follow the schema where they
     disagree. Three further texts are corrected, each under its own decision.

## Decisions

- **P0-D12: What is `Field.label`'s default, when no ESS model has a field label?**
  - (A) The field's name with `_` as a space, filled by the loader. The schema is corrected, and
    the terminal moves to what React shows.
  - (B) Add a field label to the specification language. That is a new `ess/39` construct, size L,
    outside the UI area.
  - (C) Leave it absent and keep the renderers' own, different fallbacks.

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
  `rich_text.syntax`, `DynamicNavEntries.label`), which side does the fill take? `Document.actor`
  is decided apart: `g03-actor-from-session` and P0-D18.
  - (A) The schema. Behaviour moves in both renderers, so nearly every test document's two
    renderers would disagree with their old behaviour.
  - (B) The schema's default is corrected to what both renderers do. No behaviour moves, and the
    reference changes.

  **Settled: (B)**, by the operator on 2026-11-24.

## Scope

- `crates/ui/ess-ui/src/fill/behaviour.rs`: the rules pass, which writes its `HELD` rows; the list
  stays.
- `crates/ui/ess-ui/src/model.rs`: drop the deserialiser fill at 926-930 and 1127-1129.
- `crates/ui/ess-ui-tui/src/view.rs`: 2771-2773, read the filled label. The skeleton, the danger
  mark and the arrangement already obey a written value (730-736, 2421, 997), so they need no
  edit.
- `crates/ui/ess-ui-tui/src/app.rs`: 5090-5121 and 5193-5194, draw the written `columns`.
- `crates/ui/ess-ui-react/templates/runtime/data.ts.tmpl`: 340-342, send `paging` only when not
  `none`.
- Read only, behaviour moves through the filled value: `composites/choice.tsx.tmpl`,
  `composites/references.tsx.tmpl` and `crates/ui/ess-ui-test/src/playwright.rs`.
- `schemas/ui/ess-ui.schema.yaml`: the five texts above.
- `website/docs/reference/ess-ui.md`: generated.
- new: the three test files above. Changed: at most the files listed under "Moved
  expectations".
- `CHANGELOG.md`.
- Overlap: the active stories `story:a-chart-names-its-axes-and-formats-its-ticks` (`view.rs`) and
  `story:an-action-names-its-icon` (React templates) edit other functions.
- Size: L, about 560 changed lines: code about 70, new tests about 340, moved expectations about
  150.

## Depends on

- `g03-loader-fills-defaults`: `fill/behaviour.rs` and `Loaded`.
- `g03-placement-in-loader`: the schema and the generated reference, round 3.
- `g03-actor-from-session`: `app.rs` and `data.ts.tmpl`, round 2.
- `g03-model-defaults`: the schema, round 2. Test 7 needs every other fill unit landed.

Round 4, alone.
