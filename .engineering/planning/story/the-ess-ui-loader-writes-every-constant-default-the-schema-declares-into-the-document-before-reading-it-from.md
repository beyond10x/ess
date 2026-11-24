---
format: aep.planning-md/3
id: story:the-ess-ui-loader-writes-every-constant-default-the-schema-declares-into-the-document-before-reading-it-from
kind: story
status: active
title: The ess-ui loader writes the schema's defaults into the document before reading it, from the schema as data, one pass per owning unit, and no renderer acts differently for the ones it writes itself
tags:
- gap-p0
- gap-reports-2026-11-24
relations:
- serves: vision:O2
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-11-24T01:18:53Z", actor: "human:timo", revision: 4}
- {from: "proposed", to: "active", at: "2026-11-24T01:18:55Z", actor: "human:timo", revision: 5}
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
- **Every other default is an `Option`** that each renderer resolves in its own code. Several
  disagree between a renderer and the schema. The table below has the evidence.

The loader runs four steps (`crates/ui/ess-ui/src/lib.rs:4-14`, `load` at `:111-126`). The tree it
deserialises in step 4 is private, and it holds only what the author wrote plus the seven above.

### Every default, where it lives today, and which unit fills it

Unit key:
- **F**: this story.
- **A**: `g03-actor-from-session`.
- **M**: `g03-model-defaults`.
- **P**: `g03-placement-in-loader`.
- **B**: `g03-defaults-that-move-behaviour`.
- **–**: not filled, for the reason given.

"When" says whether the fill is always written (**always**) or only under a condition. The draft
follow-up `g03-no-renderer-fallback` reads this column: a conditional key stays `Option`.

Path key:
- `model.rs`, `expand.rs` and `path.rs` are in `crates/ui/ess-ui/src/`.
- `emit.rs` is in `crates/ui/ess-ui-react/src/`.
- `*.tmpl` is in `crates/ui/ess-ui-react/templates/runtime/`.
- `view.rs`, `app.rs` and `placement.rs` are in `crates/ui/ess-ui-tui/src/`.

| construct.key | default (schema line) | today | agrees? | unit | when |
|---|---|---|---|---|---|
| Page.shell | `{first_present: [shells.app, only_shell]}` | loader `expand.rs:1226-1229` | yes | already filled | always |
| Page.layout, PageLayout | `stack` | loader `expand.rs:1222-1225` | yes | already filled | always |
| NavEntry.label | `$page.title` | loader `expand.rs:1215-1220` | yes | already filled | where the page writes `nav:` |
| list_page / report_page state | page 1, size 25 / window `{preset: last_30d}`, page 1, size 50 | kind merge `expand.rs:993-994`, `:1004-1025` | yes | already filled | on those kinds |
| Field.name | `{from: field}` | loader `expand.rs:641-657` | yes | already filled | always |
| Action.name | first of opens, does, … (`:1309-1311`) | loader `expand.rs:846-855` | yes | already filled | always |
| Widget param default | the param's `default` | loader `expand.rs:1454-1456` | yes | already filled | always |
| list.unique | false | serde `model.rs:111-112` | yes | –: inside `types`, which expansion leaves opaque (`expand.rs:29-31`) | – |
| collection.show_empty_groups | false | serde `model.rs:1371-1372` | yes | F | always |
| choice.multiple | false | serde `model.rs:1615-1616` | yes | F | always |
| Field.sortable | false | serde `model.rs:2521-2522` | yes | F | always |
| Action.optimistic, Action.bulk | false, false | serde `model.rs:3051-3055` | yes | F | always |
| Reads.pager.sizes | `[25, 50, 100]` (`sizes_default`, `:1288`) | serde `model.rs:2989-3001` | yes | F | where `pager` is written |
| State.pinned, State.sensitive | false, false | serde `model.rs:3404-3412` | yes | F | always |
| Live.match | the read's `key`, else `id` (note `:1419`) | deserialiser `model.rs:926-930`, `:1127-1129` | yes | B (P0-D16) | not on a choice's read |
| State.clear_on | `[signout, account_switch]` | serde `[]` `model.rs:3416-3417`; no renderer reads it | no: serde says `[]` | B | always |
| references.columns | `[label, type]` | serde `[]` `model.rs:2196-2197`; `references.tsx.tmpl:29-31` label ?? id, then type; `app.rs:5090-5121`, `:5193-5194` every key of the first row | no | B | always |
| form.fields | `{from: does.input_fields, order: declaration}` (`:661`) | serde `[]` `model.rs:1532-1533`; `form.tsx.tmpl:85`, `:165`, `app.rs:5010-5021` show no inputs | no | M (P0-D15) | with a model, for a command it knows |
| Document.actor | from_session (`:166`) | `Option` `model.rs:199`; `emit.rs:3322-3325` anonymous; `app.rs:4660-4662` signed out; `ess-ui-check` reads absent as from_session (`ess-ui-check/src/model.rs:442`) | no (both renderers) | A (P0-D17, P0-D18) | always |
| Region.visible | `"true"` | `Option` `model.rs:324`; `shell.tsx.tmpl:33`, `app.rs:4096-4101` | yes | F | always |
| Preload.policy | in_background | `Option` `model.rs:355`; `emit.rs:3219-3223`; the terminal ignores policy (`app.rs:1555-1570`), a defect outside P0 | yes | F | always |
| Navigation.visibility | by_actor_grants | `Option` `model.rs:452`; neither renderer reads it, a defect outside P0 | unused | F | always |
| DynamicNavEntries.label | `row.label`, corrected to `row.id` (P0-D17 B) | `Option` `model.rs:529`; `navigation.tsx.tmpl:122` and `app.rs:1062-1066` show `row.id` | after correction, yes | B | always |
| Section.collapsed | false | `Option` `model.rs:862`; `emit.rs:2450`, `core.tsx.tmpl:537`, `view.rs:720` | yes | F | always |
| Section.load | eager | `Option` `model.rs:868`; `emit.rs:2458-2462`, `core.tsx.tmpl:458`, `app.rs:1329` | yes | F | always |
| Section.title | none (note `:547`, "absent, none") | `Option` | – | –: absent means no heading; there is no value to write | – |
| SectionStates.loading | skeleton | `Option` `model.rs:985`; `core.tsx.tmpl:600` skeleton; `view.rs:730-736` one "loading…" line | no (terminal) | B | always |
| SectionStates.refreshing | keep_rows | `Option` `model.rs:987`; `core.tsx.tmpl:601` keeps rows; the terminal never reads it (`app.rs:1340`, `view.rs:730`) | yes, but the terminal ignores it | F; the terminal's behaviour is a defect outside P0 (`g03-terminal-follows-defaults`) | always |
| SectionStates.failed | `{retry: true}` | `Option` `model.rs:991`; `core.tsx.tmpl:606-607`, `view.rs:756` | yes | F | always |
| SectionStates.stale | `{mark: badge}` | `Option` `model.rs:995`; `core.tsx.tmpl:570`, `view.rs:707-714` | yes | F | always |
| SectionStates.empty | `Nothing to show yet` (note `:591`) | renderer text, for a section waiting on a `depends_on` selection | – | –: a renderer text for one lifecycle case, not a value of the key; writing `empty` would replace each renderer's ready-with-no-rows text | – |
| collection.style | table | `Option` `model.rs:1346`; `collection.tsx.tmpl:116`, `:541`; `view.rs:1414-1415` | yes | F | always |
| collection.selection | none | `Option` `model.rs:1348`; `collection.tsx.tmpl:111-114` none; the terminal marks rows in every collection (`app.rs:3216-3227`) | yes, but the terminal ignores it | F; the terminal's behaviour is a defect outside P0 (`g03-terminal-follows-defaults`) | always |
| form.save | on_submit | `Option` `model.rs:1555`; `form.tsx.tmpl:162`, `app.rs:3746`, `parity.rs:204` | yes | F | always |
| choice.style | dropdown | `Option` `model.rs:1618`; `choice.tsx.tmpl:89-91` checklist when `multiple`, and `playwright.rs:1023-1026` the same; the terminal ignores it | no (React) | B | always |
| choice.value | the read's `key`, else … else `id` (note `:691`) | `model.rs:1629-1633` at read time | yes | –: a rule over row fields and the model, applied per row; P0-D16 keeps a choice's own order | – |
| choice.label | the row's `label`, else `name`, else the value (note `:692`) | read time | yes | –: a rule over row values | – |
| filter_bar.reset | false | `Option` `model.rs:1735` (`offers_reset` `:1740-1746`) | yes | F | always |
| Header.badge.tone | neutral (note `:735`) | `Option` `model.rs:1829` (`HeaderBadge`) | yes | F | where `badge` is written |
| confirm.confirm_label | `Delete`, corrected to `Confirm` (P0-D17 B) | `Option` `model.rs:1949`; `confirm.tsx.tmpl:131` and `view.rs:2417-2420` show `Confirm` | after correction, yes | B | always |
| confirm.danger | true | `Option` `model.rs:1951`; `confirm.tsx.tmpl:79` danger; `view.rs:2421` not destructive | no (terminal) | B | always |
| metric.format | number | `Option` `model.rs:1985`; `core.tsx.tmpl:818-819`, `view.rs:2486-2500` | yes | F | always |
| metric.unit | s (`path.rs:562-565` refuses it elsewhere) | `Option` `model.rs:1987`; `core.tsx.tmpl:815`, `view.rs:2497` | yes | F | with `format: duration` |
| metric.scale | fraction | `Option` `model.rs:1989` | yes | F | with `format: percent` |
| rich_text.syntax | expression, corrected to `plain` (P0-D17 B) | `Option` `model.rs:2165`; `rich_text.tsx.tmpl:21` and `view.rs:1221` use `plain` | after correction, yes | B | always |
| references.navigates | true | `Option` `model.rs:2199`; neither renderer links a row, a defect outside P0 | unused | F | always |
| Widget.arrange | column | `Option` `model.rs:2214`; `widget.tsx.tmpl:13` column; `view.rs:997` one row | no (terminal) | B | always |
| Field.label | `{from: model.field_label}`, corrected (P0-D12) | `Option` `model.rs:2510`; no model has a field label; `fields.tsx.tmpl:117-118` name with `_` as a space; `view.rs:2771-2773` raw field | no (the renderers differ) | B | always |
| Field.as | text | `Option` `model.rs:2513`; `fields.tsx.tmpl:201`, `:204`, `view.rs:1847` | yes | F | always |
| Field.unit | ms (`path.rs:501-518`) | `Option` `model.rs:2515`, accessor `:2582-2589` | yes | F | with `as: duration` |
| Field.scale | fraction (`path.rs:520-538`) | `Option` `model.rs:2517`, accessor `:2595-2597` | yes | F | with `as: percent` |
| Field.tone | neutral | `Option` `model.rs:2533`; `tone_of` `:2565-2575` and both renderers apply neutral only beside `tone_by` | no (note) | B (P0-D13) | beside a `tone_by` |
| Field.label_from.key | id (note `:969`) | `Option` `model.rs:2797`, accessor `:2822` | yes | F | where `label_from` is written |
| Field.suggests.value | name (note `:973`) | `Option` `model.rs:2809`, accessor `:2815` | yes | F | where `suggests` is written |
| graph_editor.nodes.key | id (note `:851`) | `Option` `model.rs:2135`; `graph_editor.tsx.tmpl:146`, `app.rs:5152` | yes | F | always |
| FormGroup.collapsed, FormGroup.save | false, with_form | `Option` `model.rs:2871`, `:2875` | yes | F | always |
| text.style, text.format | body, plain | `Option` `model.rs:2299`, `:2301` | yes | F | always |
| badge.tone, icon.tone | neutral, neutral | `Option` `model.rs:2374`, `:2402` | yes | F | always |
| button.tone | secondary | `Option` `model.rs:2418` | yes | F | always |
| input.as, image.fit, divider.orientation | text, contain, horizontal | `Option` `model.rs:2458`, `:2486`, `:2494` | yes | F | always |
| Action.as | button | `Option` `model.rs:3041` | yes | F | always |
| Action.checked | derived (note `:1324`) | `toggle.rs`, at read time | yes | –: a rule over values, not a key | – |
| Reads.key | the section's `live.match`, else `id` (note `:1284`) | `Option` `model.rs:2918`, `row_key` `:957-962`; `core.tsx.tmpl:233-234`, `view.rs:1462-1477` | yes | B (P0-D16) | not on a choice's read |
| Reads.paging | none | `Option` `model.rs:2923`; React sends `paging=` only when written (`data.ts.tmpl:340-342`, `:829`) | yes; React's request would move | B | always |
| Reads.cache | `{store: {resolve: {class: view_cache}}}` (`:1291`) | `Option` `model.rs:2932`; `ReadCache.store` is a `Store` (`:3007`) | – | P | where resolution yields a store |
| Channel.scope | account | `Option` `model.rs:3232`; neither renderer reads it, a defect outside P0 | unused | F | always |
| Live.when_paged_away | count_new | `Option` `model.rs:3349`; `app.rs:2511-2516` count_new; React counts only for a read with a `filter`, written or not (`emit.rs:2381-2390`), a defect outside P0 (`g03-react-follows-defaults`) | yes, but React ignores it without a filter | F | on a section `live` block (refused on a nested node, `model.rs:1121`) |
| State.store | resolved by `PlacementProfile.resolution` (`:1563-1570`) | `Option` `model.rs:3402`; React `emit.rs:366-445`; `placement.rs:82-126` and `ess-ui-check/src/rules.rs:1392-1433` ignore `pinned` | no (pinned) | P | where resolution yields a store |
| State.scope | by store (`:1442`) | `Option` `model.rs:3409`; React keys `scope:key` only when written (`state/server.ts.tmpl:16`) | no (React key) | P (P0-D14) | where a store was resolved |
| FixtureIndex script `loop` | false (`:1344`) | a script file, not the document (`fixtures.rs:73`, `live.rs:82`) | yes | –: not in an `ess-ui/1` document | – |
| Tokens, Theme, type tokens | the built-in table (`constructs.Tokens.builtins`) | `style.rs:51-97` | yes | –: styling, out of scope by the owner's order | – |
| Degrades | the capability's first fallback | `profile.rs:1-8`, `emit.rs:2287` | yes | –: depends on what a renderer lacks, which a loader does not know | – |
| a ToneMap miss | derived (`model.rs:2574`) | one accessor in `ess-ui` | yes | –: a rule over values | – |

Fields named `default` (`State.default`, `ThemeChoice.default`) and the schema's examples (`:276`,
`:1530`, `:1702`) are not defaults.

## Outcome

- **The fill step.** It runs between expansion (step 3) and deserialisation (step 4) and writes
  defaults into the tree wherever the author left the key out. It never overwrites a written value
  or an `UNMAPPED:` marker. It reads the defaults from the embedded schema as data: each
  construct's `default:` entries, `sizes_default`, and the note defaults in the table. It walks the
  tree by construct, following each field's type from `Document` down:
  - through `list`, `map`, `optional`, `one_of` and `record`;
  - into the `Composite` union by `component`, and the `Primitive` union by `primitive`;
  - extending the type walk that `Positions::scan` already does
    (`crates/ui/ess-ui/src/schema.rs:66-157`).

  So `style` is filled as `table` on a collection and as `body` on a text, never mixed.
- **Pass order.** The fill runs its passes in this order, so that each pass sees what the earlier
  ones wrote:
  1. model (`fill/model.rs`);
  2. placement (`fill/placement.rs`), `store` before `scope`;
  3. actor (`fill/actor.rs`);
  4. rules (`fill/behaviour.rs`);
  5. constants (`fill/mod.rs`), from the schema.

  A pass that adds a construct (the model pass adds `Field`s) writes it already expanded
  (`{field: x, name: x}`), because expansion has already run. The later passes then complete it.
- **One file per later unit, each with a hold list.** This story creates `fill/mod.rs` (the walk,
  the constant pass and the pass order) and four stub files: `fill/model.rs`,
  `fill/placement.rs`, `fill/actor.rs` and `fill/behaviour.rs`. Each stub holds an empty pass and
  a `HELD` list of the rows its unit owns (M, P, A and B in the table). The constant pass skips
  every key on a `HELD` list, in every round, and `defaults.rs` reads the same lists. A later unit
  fills its own pass, which writes the rows of its `HELD` list where their condition holds. The
  list stays, so the constant pass never writes those rows: not a `Field.tone` without `tone_by`,
  not a `form.fields` without a model, not an unresolved `store`. Each later unit edits only its
  own file, so no two units edit one hunk.
- **Four conditional defaults.** `Field.unit` and `Field.scale` are filled only where `as` is
  `duration` or `percent`. `metric.unit` and `metric.scale` are filled only where `format` is
  `duration` or `percent`. A test holds this rule to the loader's own refusals
  (`path.rs:501-576`).
- **One public surface for the filled tree.** It is
  `ess_ui::Loaded { pub document: Document, pub tree: ess_yaml::Value }`, built by
  `ess_ui::load_resolved(text: &str, model: Option<&dyn binding::ModelEnums>)` and by
  `load_resolved_path(file, model)`. `load_str`, `load_str_with` and `load_path` return
  `load_resolved(…)?.document`, unchanged in signature. Every other unit cites `Loaded` by this
  name.
- **No record of what was filled.** A filled key and the same key written by the author are one
  document: same tree, same `Document`, same digest, same behaviour in every renderer (P0-D14 B).
- **No behaviour moves.** `Document` reads the same values as before, because each F value equals
  the fallback every renderer already used. `Live.when_paged_away` changes nothing in React, which
  ignores it without a filter, written or not.
- **The UI corpus.** This story defines it, and every story of P0 that says "the UI corpus" means
  it. It is the list of repository-relative paths in `crates/ui/ess-ui/tests/corpus/CORPUS`, one
  per line, written by this story:
  - `examples/partner-portal/ui.yaml`
  - `crates/ui/ess-ui-test/tests/fixtures/fixture-command-effect/ui/desk.ui.yaml`
  - `crates/ui/ess-ui-test/tests/fixtures/adversary-t3-fixture-command-effect/ui/desk.ui.yaml`
  - `crates/ui/ess-ui-test/tests/fixtures/adversary-t3-fixture-command-effect/ui/desk-derived.ui.yaml`
  - `crates/ui/ess-ui-react/tests/fixtures/fixture-command-effect/ui/desk.ui.yaml`
  - `crates/edge/ess-cli/tests/fixtures/ui-http/items/ui.yaml`
  - `crates/edge/ess-cli/tests/fixtures/ui-http/shelf/ui.yaml`
  - `crates/ui/ess-ui/tests/corpus/every-construct.yaml`: new. It holds one instance of each
    construct in the table, with no optional key written.

  Inline documents in test sources are not in the corpus. `crates/ui/ess-ui-check/tests/fixtures/`
  holds refusal fixtures and is left out.

## Acceptance

New test file `crates/ui/ess-ui/tests/defaults.rs`. Each test is expected under 2 s and is timed;
none may run over 60 s, and the crate's test step stays within 120 s. The file joins the crate's
single test binary of `story:the-test-step-meets-the-test-budget` if that has landed; otherwise it
is a new file, which that story moves.

1. `every_f_default_is_written_into_the_tree`. Red today: there is no fill step and no `Loaded`.
   `load_resolved` of `every-construct.yaml` holds, at each instance, every F key with the
   schema's value, and the test's expected table names each one.
2. `every_default_is_placed`. The test reads three things from the embedded schema:
   - every `default:` entry and `sizes_default`;
   - every field note that matches one of the phrases `defaults to`, `(default `, `default: `,
     `Absent:`, `Absent,`, `absent,` or `absent:`.

   Each of these (construct, key) pairs must be in exactly one of three places: the F set, a unit's
   `HELD` list, or the test's `NOT_FILLED` list with its reason (the – rows above). So a new schema
   default, written as an entry or in a note, fails the test until it is placed.
3. `a_written_value_is_kept`. `style: cards`, an `UNMAPPED:` marker and `multiple: true` stay as
   written.
4. `conditional_defaults_follow_the_loaders_refusals`:
   - a `text` field gets no `unit`, and a `duration` field gets `unit: ms`;
   - a `number` metric gets neither `unit` nor `scale`;
   - every filled corpus document still loads, so the fill never writes what `path.rs:501-576`
     refuses.
5. `filling_changes_no_behaviour`. The existing test sets of `ess-ui-check`, `ess-ui-tui` and
   `ess-ui-test` pass with no expectation changed.
6. `the_fill_is_idempotent`. For every document of the UI corpus, loading the printed `tree`
   again fills nothing new: the second tree equals the first, and so does the `Document`.
7. `the_corpus_list_is_complete`. A walk of `examples/`, `crates/ui/ess-ui-test/tests/fixtures/`,
   `crates/ui/ess-ui-react/tests/fixtures/`, `crates/edge/ess-cli/tests/fixtures/ui-http/` and
   `crates/ui/ess-ui/tests/corpus/` finds every `*.yaml` file whose text holds
   `format: ess-ui/1`. Each is in `CORPUS`, and `CORPUS` holds at least 8 paths.
8. Only these React test files may change expectations: `live_binding.rs`,
   `adversary_live_binding_pass1.rs`, `adversary_live_binding_pass2.rs`, `live_composites.rs` and
   `tokens.rs` in `crates/ui/ess-ui-react/tests/`. The emitter writes every key that is present
   (`emit.rs:2100-2140`), so emitted code can move while behaviour does not. Each changed
   assertion names this story. If an assertion has to change in any other file, the unit stops
   and reports it.

Gate:
- `task fmt-check`;
- `cargo clippy -p ess-ui -p ess-ui-react --all-targets --locked -- -D warnings`;
- `cargo test -p ess-ui -p ess-ui-check -p ess-ui-tui -p ess-ui-test -p ess-ui-react --locked`.

`CHANGELOG.md`, under Changed: the loader fills the schema's defaults on which every renderer
agrees.

## Fit review

Read at tree `bcfc51555`. The request came from a downstream application on 2026-11-24, report
item 3. The earlier draft was never filed in the store. Its decision D3 was settled by the owner as
(b), "the loader fills every default".

1. **Need.** A generator other than the stock ones starts from the same values the stock
   renderers use, without re-implementing the schema's defaults. The evidence received on
   2026-11-24 counts 144 lines of such re-implementation in one own generator
   (`g03-resolved-json`).
2. **Class.** It is a *gap*: there is no resolved document. Where a renderer disagrees with the
   schema it is also a *defect*, which units A, M, P and B fix.
3. **Already expressible?** No. The schema publishes the table, but nothing applies it, and what
   the renderers do is not that table.
4. **Fit.**
   - It adds no authored key and no `ess-ui/1` version: a filled value means what the schema
     already says the absent key means.
   - It reads the schema as data, as the loader already does for short forms and built-in page
     kinds (`schema.rs:23-61`).
   - `ess-ui` keeps its single dependency.
5. **Second adopter.** A generator for a native mobile client from the same document. A linter
   that warns when a document writes a key at its default value.
6. **Cost.**
   - About 50 more keys in every loaded tree, a fixed amount per construct.
   - Emitted React code moves where the emitter writes a present key. Behaviour does not move.
   - No diagnostic and no migration.
7. **Considered.**
   - (a) Not filling. Rejected by the owner's order.
   - (b) A published defaults table. Rejected in the draft: nothing would hold the renderers to it.
   - (c) `#[serde(default)]` on every model field. Rejected: the values would sit in `Document` but
     not in the printed tree, and the schema and the model would be two tables.
   - (d) Chosen: a fill of the tree, driven by the schema.

## Decisions

None of its own. The rows marked A, M, P and B carry P0-D12 to P0-D18.

## Scope

- new files:
  - `crates/ui/ess-ui/src/fill/mod.rs`: the walk, the pass order, the constant pass, about 230
    lines;
  - stubs `crates/ui/ess-ui/src/fill/model.rs`, `fill/placement.rs`, `fill/actor.rs` and
    `fill/behaviour.rs`: an empty pass and a `HELD` list each;
  - `crates/ui/ess-ui/tests/defaults.rs`;
  - `crates/ui/ess-ui/tests/corpus/CORPUS`;
  - `crates/ui/ess-ui/tests/corpus/every-construct.yaml`.
- changed files:
  - `crates/ui/ess-ui/src/schema.rs`: a `defaults` reader beside `positions`, 66-91, and the note
    phrases;
  - `crates/ui/ess-ui/src/lib.rs`: `mod fill`, `Loaded`, `load_resolved` and `load_resolved_path`
    beside `load`, 111-126; the module doc, 4-14;
  - at most the five React test files named in acceptance 8.
- `CHANGELOG.md`.
- Overlap: the active stories `story:a-node-named-like-an-action-position-loads` (`expand.rs`,
  `schema.rs`) and `story:a-host-slot-in-a-reused-template-is-one-registration` (`expand.rs`).
  This unit adds a function to `schema.rs` and changes no function of `expand.rs`.
- Size: M, about 570 changed lines: code about 290, tests about 220, corpus about 60.

## Depends on

Nothing. Round 1, beside `g01-state-design` and `g01-app-forks`, with which it shares no file.
`g03-actor-from-session`, `g03-model-defaults`, `g03-placement-in-loader` and
`g03-defaults-that-move-behaviour` depend on it.
