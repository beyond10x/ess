---
format: aep.planning-md/3
id: story:the-ess-ui-loader-writes-every-constant-default-the-schema-declares-into-the-document-before-reading-it-from
kind: story
status: draft
title: The ess-ui loader writes every constant default the schema declares into the document before reading it, from the schema as data, and no renderer acts differently for it
tags:
- gap-p0
- gap-reports-2026-11-24
relations:
- serves: vision:O2
revision: 1
---
## Need

Report item 3 asks for `ess ui load --format json` with every default filled in. The owner settled
the open question of the earlier draft: the loader fills every default. Today the schema declares
75 `default:` entries (`schemas/ui/ess-ui.schema.yaml`), plus defaults written only in notes. The
code holds them in three places and fills almost none into the document.

- **The loader writes seven** into the expanded tree: `Page.shell`, `Page.layout`,
  `NavEntry.label`, the built-in page kinds' state values, `Field.name`, `Action.name` and a widget
  parameter's default at its use site.
- **The model types supply eleven through serde** (`#[serde(default)]` or a deserialiser), after
  the tree is built, so they never appear in it. Three of these disagree with the schema:
  `State.clear_on`, `references.columns` and `form.fields` default to `[]`.
- **Every other default is an `Option`** that each renderer resolves in its own code. Eleven of
  them disagree between a renderer and the schema today. The table below has the evidence.

The loader runs four steps (`crates/ui/ess-ui/src/lib.rs:4-14`, `load` at `:111-126`). The tree it
deserialises in step 4 is private, and it holds only what the author wrote plus the seven above.

### Every default, where it lives today, and which unit fills it

Unit key:
- **F**: this story.
- **B**: `g03-defaults-that-move-behaviour`.
- **A**: `g03-actor-from-session`.
- **P**: `g03-placement-in-loader`.
- **M**: `g03-model-defaults`.
- **R**: `g03-react-follows-defaults`.
- **T**: `g03-terminal-follows-defaults` (fills nothing; it fixes behaviour).
- **–**: not filled, with the reason.

Path key:
- `model.rs`, `expand.rs` and `path.rs` are in `crates/ui/ess-ui/src/`.
- `emit.rs` is in `crates/ui/ess-ui-react/src/`.
- `*.tmpl` is in `crates/ui/ess-ui-react/templates/runtime/`.
- `view.rs`, `app.rs` and `placement.rs` are in `crates/ui/ess-ui-tui/src/`.

| construct.key | default | today (model field; loader; renderer fallback) | agrees? | unit |
|---|---|---|---|---|
| Page.shell | `{first_present: [shells.app, only_shell]}` | loader `expand.rs:1226-1229` | yes | already filled |
| Page.layout, PageLayout | `stack` | loader `expand.rs:1222-1225` | yes | already filled |
| NavEntry.label | `$page.title` | loader `expand.rs:1215-1220`, only when the page writes `nav:` | yes | already filled |
| list_page / report_page state | page 1, size 25 / window `{preset: last_30d}`, page 1, size 50 | kind merge `expand.rs:993-994`, `:1004-1025` | yes | already filled |
| Field.name | `{from: field}` | loader `expand.rs:641-657` | yes | already filled |
| Action.name | first of opens, does, … (schema `:1309-1311`) | loader `expand.rs:846-855` | yes | already filled |
| Widget param default | the param's `default` | loader `expand.rs:1454-1456` | yes | already filled |
| list.unique | false | serde `model.rs:111-112` | yes | –: inside `types`, which expansion leaves opaque (`expand.rs:29-31`) |
| collection.show_empty_groups | false | serde `model.rs:1371-1372` | yes | F |
| choice.multiple | false | serde `model.rs:1615-1616` | yes | F |
| Field.sortable | false | serde `model.rs:2521-2522` | yes | F |
| Action.optimistic, Action.bulk | false, false | serde `model.rs:3051-3055` | yes | F |
| Reads.pager.sizes | `[25, 50, 100]` (spelled `sizes_default`, schema `:1288`) | serde `model.rs:2989-3001`, only with `pager` | yes | F (only where `pager` is written) |
| State.pinned, State.sensitive | false, false | serde `model.rs:3404-3412` | yes | F |
| Live.match | the read's `key`, else `id` (note `:1419`) | deserialiser `model.rs:926-930`, `:1127-1129` | yes | B (with Reads.key, P0-D16) |
| State.clear_on | `[signout, account_switch]` | serde `[]` `model.rs:3416-3417`; no renderer reads it | no: serde says `[]` | B |
| references.columns | `[label, type]` | serde `[]` `model.rs:2196-2197`; `references.tsx.tmpl:29-31` label ?? id (type); `app.rs:5090-5121`, `:5193-5194` every key of the first row | no | B |
| form.fields | `{from: does.input_fields, order: declaration}` | serde `[]` `model.rs:1532-1533`; `form.tsx.tmpl:85`, `:165` and `app.rs:5010-5021` show no inputs | no | M (P0-D15) |
| Document.actor | from_session | `Option` `model.rs:199`; `emit.rs:3322-3325` anonymous; `app.rs:4660-4662` signed out; ess-ui-check from_session | no | A (P0-D17) |
| Region.visible | `"true"` | `Option` `model.rs:324`; `shell.tsx.tmpl:33`, `app.rs:4096-4101` shown | yes | F |
| Preload.policy | in_background | `Option` `model.rs:355`; `emit.rs:3219-3223`; the terminal ignores policy (`app.rs:1555-1570`) | yes | F |
| Navigation.visibility | by_actor_grants | `Option` `model.rs:452`; neither renderer reads it | unused | F |
| DynamicNavEntries.label | `row.label` | `Option` `model.rs:529`; `navigation.tsx.tmpl:122` and `app.rs:1062-1066` show `row.id` | no | B |
| Section.collapsed | false | `Option` `model.rs:862`; `emit.rs:2450`, `core.tsx.tmpl:537`, `view.rs:720` | yes | F |
| Section.load | eager | `Option` `model.rs:868`; `emit.rs:2458-2462`, `core.tsx.tmpl:458`, `app.rs:1329` | yes | F |
| SectionStates.loading | skeleton | `Option` `model.rs:985`; `core.tsx.tmpl:600` skeleton; `view.rs:730-736` one "loading…" line | no (terminal) | B |
| SectionStates.refreshing | keep_rows | `Option` `model.rs:987`; `core.tsx.tmpl:601` keeps rows; the terminal never reads it and draws "loading…" (`app.rs:1340`, `view.rs:730`) | no (terminal) | F, behaviour T |
| SectionStates.failed | `{retry: true}` | `Option` `model.rs:991`; `core.tsx.tmpl:606-607`, `view.rs:756` | yes | F |
| SectionStates.stale | `{mark: badge}` | `Option` `model.rs:995`; `core.tsx.tmpl:570`, `view.rs:707-714` | yes | F |
| collection.style | table | `Option` `model.rs:1346`; `collection.tsx.tmpl:116`, `:541`; `view.rs:1414-1415` | yes | F |
| collection.selection | none | `Option` `model.rs:1348`; `collection.tsx.tmpl:111-114` none; the terminal ignores it and marks rows in every collection (`app.rs:3216-3227`) | no (terminal) | F, behaviour T |
| form.save | on_submit | `Option` `model.rs:1555`; `form.tsx.tmpl:162`, `app.rs:3746`, `parity.rs:204` | yes | F |
| choice.style | dropdown | `Option` `model.rs:1618`; `choice.tsx.tmpl:89-91` checklist when `multiple`, and `playwright.rs:1023-1026` the same; the terminal ignores it | no (React) | B |
| filter_bar.reset | false | `Option` `model.rs:1735` (`offers_reset` `:1740-1746`) | yes | F |
| confirm.confirm_label | `Delete` | `Option` `model.rs:1949`; `confirm.tsx.tmpl:131` and `view.rs:2417-2420` show `Confirm` | no (both) | B (P0-D17) |
| confirm.danger | true | `Option` `model.rs:1951`; `confirm.tsx.tmpl:79` danger; `view.rs:2421` not destructive | no (terminal) | B |
| metric.format | number | `Option` `model.rs:1985`; `core.tsx.tmpl:818-819`, `view.rs:2486-2500` | yes | F |
| metric.unit | s, only with `format: duration` (`path.rs:562-565` refuses otherwise) | `Option` `model.rs:1987`; `core.tsx.tmpl:815`, `view.rs:2497` | yes | F (only with `format: duration`) |
| metric.scale | fraction, only with `format: percent` | `Option` `model.rs:1989` | yes | F (only with `format: percent`) |
| rich_text.syntax | expression | `Option` `model.rs:2165`; `rich_text.tsx.tmpl:21` and `view.rs:1221` use `plain` | no (both) | B (P0-D17) |
| references.navigates | true | `Option` `model.rs:2199`; neither renderer links a row, whatever is written | unused | F; the missing link is outside P0 |
| Widget.arrange | column | `Option` `model.rs:2214`; `widget.tsx.tmpl:13` column; `view.rs:997` one row | no (terminal) | B |
| Field.label | `{from: model.field_label}` | `Option` `model.rs:2510`; no ESS model has a field label; `fields.tsx.tmpl:117-118` shows the name with `_` as a space; `view.rs:2771-2773` shows the raw field | no (both differ) | B (P0-D12) |
| Field.as | text | `Option` `model.rs:2513`; `fields.tsx.tmpl:201`, `:204`, `view.rs:1847` | yes | F |
| Field.unit | ms, only with `as: duration` (`path.rs:501-518`) | `Option` `model.rs:2515`, accessor `:2582-2589` | yes | F (only with `as: duration`) |
| Field.scale | fraction, only with `as: percent` (`path.rs:520-538`) | `Option` `model.rs:2517`, accessor `:2595-2597` | yes | F (only with `as: percent`) |
| Field.tone | neutral | `Option` `model.rs:2533`; `tone_of` `:2565-2575` applies neutral only beside `tone_by`, and so do `fields.tsx.tmpl:145-146` and `view.rs:63-70` | no (both) | B (P0-D13) |
| Field.label_from.key | id (note `:969`) | `Option` `model.rs:2797`, accessor `:2822` | yes | F |
| Field.suggests.value | name (note `:973`) | `Option` `model.rs:2809`, accessor `:2815` | yes | F |
| graph_editor.nodes.key | id (note `:851`) | `Option` `model.rs:2135`; `graph_editor.tsx.tmpl:146`, `app.rs:5152` | yes | F |
| FormGroup.collapsed, FormGroup.save | false, with_form | `Option` `model.rs:2871`, `:2875` | yes | F |
| text.style, text.format | body, plain | `Option` `model.rs:2299`, `:2301` | yes | F |
| badge.tone, icon.tone | neutral, neutral | `Option` `model.rs:2374`, `:2402` | yes | F |
| button.tone | secondary | `Option` `model.rs:2418` | yes | F |
| input.as, image.fit, divider.orientation | text, contain, horizontal | `Option` `model.rs:2458`, `:2486`, `:2494` | yes | F |
| Action.as | button | `Option` `model.rs:3041` | yes | F |
| Reads.key | the section's `live.match`, else `id` (note `:1284`) | `Option` `model.rs:2918`, `row_key` `:957-962`; `core.tsx.tmpl:233-234`, `view.rs:1462-1477` use `id`; a choice tries `key` first (`model.rs:1629-1633`); React writes selected rows by `row.id` (`collection.tsx.tmpl:107`) | yes, React minor | B (P0-D16), selection key R |
| Reads.paging | none | `Option` `model.rs:2923`; React sends `paging=` only when written (`data.ts.tmpl:340-342`, cache key `:829`) | yes; React's request moves if filled | B (React keeps its request) |
| Reads.cache | `{store: {resolve: {class: view_cache}}}` | `Option` `model.rs:2932`; `ReadCache.store` is a `Store` and cannot hold `resolve` (`:3007`); the terminal decides per document only (`app.rs:2770-2774`); React has no read cache | no (terminal) | P |
| Channel.scope | account | `Option` `model.rs:3232`; neither renderer reads it | unused | F |
| Live.when_paged_away | count_new | `Option` `model.rs:3349`; `app.rs:2511-2516` count_new; React counts only for a read with a `filter` (`emit.rs:2381-2390`, `live.ts.tmpl:486-501`) | no (React) | R |
| State.store | resolved by `PlacementProfile.resolution` (schema `:1563-1570`) | `Option` `model.rs:3402`; React `emit.rs:366-445` (pinned skips the section and page profiles); `placement.rs:82-126` and `ess-ui-check/src/rules.rs:1392-1433` ignore `pinned` | no (pinned) | P |
| State.scope | by store: memory, url, session_storage tab; local_storage device; server_session session; server user | `Option` `model.rs:3409`; React's server key is `scope:key` only when written (`state/server.ts.tmpl:16`) | no (React key) | P (P0-D14) |
| Tokens, Theme, type tokens | the built-in table (schema `constructs.Tokens.builtins`) | accessors `style.rs:51-97`, read by both renderers | yes | –: styling, out of scope by the owner's order; the table is published in the schema as data |
| Degrades | the capability's first fallback | `profile.rs:1-8`, `emit.rs:2287` | yes | –: depends on the renderer's lacks, which a loader does not know |
| Action.checked, a ToneMap miss | derived at read time (`toggle.rs`, `model.rs:2574`) | one accessor in `ess-ui`, called by both renderers | yes | –: a rule over values, not a key |
| FixtureIndex script `loop` | false | a script file, not the document (`fixtures.rs:73`, `live.rs:82`) | yes | –: not in an `ess-ui/1` document |

Fields named `default` (`State.default`, `ThemeChoice.default`) and the schema's examples (`:276`,
`:1530`, `:1702`) are not defaults.

## Outcome

- A fill step runs between expansion (step 3) and deserialisation (step 4). It writes every
  default marked **F** above into the tree at every instance of its construct, wherever the author
  left the key out. It never overwrites a written value or an `UNMAPPED:` marker.
- The step reads the defaults from the embedded schema as data: each construct's `fields` with a
  `default:`, `sizes_default`, and the three note defaults named in the table. It walks the tree by
  construct, following each field's type from `Document` down:
  - through `list`, `map`, `optional`, `one_of` and `record`;
  - into the `Composite` union by `component`, and the `Primitive` union by `primitive`;
  - extending the type walk `Positions::scan` already does (`crates/ui/ess-ui/src/schema.rs:66-157`).

  So `style` is filled as `table` on a collection and as `body` on a text, never mixed.
- Four defaults are conditional. `Field.unit` and `Field.scale` are filled only where `as` is
  `duration` or `percent`, and `metric.unit` and `metric.scale` only where `format` is `duration` or
  `percent`. A test holds this rule to the loader's own refusals (`path.rs:501-576`).
- `Document` reads the same values it read before, because each filled value equals the fallback
  every renderer already used. The defaults the renderers disagree on are filled by the units
  named in the table.

## Acceptance

New test file `crates/ui/ess-ui/tests/defaults.rs`. Each test is expected under 2 s and is timed.
None may run over 60 s, and the file's step stays within 120 s.

1. `every_f_default_is_written_into_the_tree`. Red today: no fill step exists.
   - A document with one instance of each construct of the F rows, written with no optional key,
     loads.
   - The tree the loader deserialises holds, at each instance, every F key with the schema's
     value. Expose that tree to the test as `ess_ui::expanded` or `Loaded::tree`, which
     `g03-resolved-json` also uses.
2. `every_schema_default_is_named_in_the_table`. The test reads every `default:` and
   `sizes_default` entry of the embedded schema. Each one is in the fill set, or in an explicit
   list in the test of the defaults filled by A, B, P, M or R or not filled, with the reason. A new
   schema default fails the test until it is placed.
3. `a_written_value_is_kept`. `style: cards`, an `UNMAPPED:` marker and `multiple: true` are left
   as written.
4. `conditional_defaults_follow_the_loaders_refusals`. A `text` field gets no `unit`. A
   `duration` field gets `unit: ms`. A `number` metric gets neither `unit` nor `scale`. Every
   filled document still loads, so the fill never writes what `path.rs:501-576` refuses.
5. `filling_changes_no_behaviour`:
   - `ess ui check` gives the same findings for every document under `examples/` and every corpus
     document of `crates/ui/ess-ui-check/tests/`.
   - `cargo test -p ess-ui-tui -p ess-ui-test --locked` passes with no test changed, and every
     `ess-ui-test/1` corpus report is the same.
   - `cargo test -p ess-ui-react --locked` passes. A test that asserted a prop is absent from
     emitted code (the emitter writes every key that is present, for example `emit.rs:2100-2140`)
     is updated, and the cause is named beside it.
6. `the_fill_is_idempotent`. Loading the filled tree again fills nothing new.

## Fit review

Read at tree `bcfc51555`. The request came from a downstream application on 2026-11-24, report
item 3. The earlier draft was never filed in the store. Its decision D3 was settled by the owner as
(b), "the loader fills every default".

1. **Need.** A generator other than the stock ones starts from the same values the stock
   renderers use, without re-implementing the schema's defaults. Today the only copy of most
   defaults is renderer code, and the renderers disagree (see the table).
2. **Class.** This is a *gap*: there is no resolved document. Where a renderer disagrees with the
   schema it is also a *defect*, and units B, R, T and P fix those.
3. **Already expressible?** No. The schema publishes the table, but nothing applies it, and what
   the renderers do is not that table.
4. **Fit.**
   - There is no authored key and no `ess-ui/1` version: a filled value means what the schema
     already says the absent key means.
   - It reads the schema as data, as the loader already does for short forms and built-in page
     kinds (`schema.rs:23-61`).
   - `ess-ui` keeps its single dependency.
5. **Second adopter.** A generator for a native mobile client from the same document. A linter
   that warns when a document writes a key at its default value.
6. **Cost.**
   - About 50 more keys in every loaded tree, a fixed amount per construct.
   - Emitted React code moves wherever the emitter writes a key that is present. The behaviour
     does not move.
   - No diagnostic, no migration.
7. **Considered.**
   - (a) Not filling. Rejected by the owner's order.
   - (b) A published defaults table. Rejected in the draft: nothing would hold the renderers to it.
   - (c) `#[serde(default)]` on every model field. Rejected: the values would sit in `Document`
     but not in the tree `g03-resolved-json` prints, and the schema and the model would be two
     tables.
   - (d) Chosen: a schema-driven fill of the tree.

## Decisions

None of its own. The rows marked A, B, P, M and R carry P0-D12 to P0-D17.

## Scope

- new: `crates/ui/ess-ui/src/fill.rs` (the construct walk and the fill, about 230 lines)
- cited: `crates/ui/ess-ui/src/schema.rs` (a `defaults` reader beside `positions`, 66-91),
  `crates/ui/ess-ui/src/lib.rs` (`load` 111-126: the step and the exposed tree; module doc 4-14)
- new: `crates/ui/ess-ui/tests/defaults.rs`
- possibly: assertions in `crates/ui/ess-ui-react/tests/*.rs` that a prop is absent
- `CHANGELOG.md` (Changed: the loader fills the schema's defaults)
- Overlap: `story:a-node-named-like-an-action-position-loads` (active; `expand.rs`, `schema.rs`)
  and `story:a-host-slot-in-a-reused-template-is-one-registration` (active; `expand.rs`). This unit
  adds a function to `schema.rs` and a new file, and changes no function of `expand.rs`.
- Size: M, about 480 changed lines (code about 260, tests about 220).

## Depends on

Nothing. It can be built at once, beside `g01-state-design` and `g01-app-forks`.
