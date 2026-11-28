---
format: aep.planning-md/3
id: story:with-model-ess-ui-check-refuses-a-fixture-row-or-a-script-payload-whose-value-contradicts-the-type-the-model
kind: story
status: draft
title: With --model, ess ui check refuses a fixture row or a script payload whose value contradicts the type the model declares for it
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
revision: 2
---
## Need

In fixture mode both renderers answer every read from the document's fixture files and play each
channel's script. With `--model`, `ess ui check` resolves the names a document writes in the
compiled model: views, commands, events, columns, row fields, enum variants of a choice. It never
reads a fixture row against the types the model declares for that view's fields, or a scripted
event's payload against the event's fields. So a fixture that holds a text where the view declares
a list, leaves out a required member of a struct field, or spells a field by a name the view does
not have passes the check. The renderers then draw the mistyped value, and a test written against
those fixtures checks a screen the served surface can never produce.

Reported by a downstream application on 0.98.0 (sixth list, item G15, rank P1): fixtures whose rows
contradicted the view types (a digest text where a list is declared, a missing struct member, a
renamed field) passed `ess ui check --model` and `ess ui test --model`. The application corrected
them by hand; nothing holds them.

What is read today (tree `bcfc51555`, read-only):

- `rules.rs` reads the fixture index and, per view, its `rows` as untyped `ess_yaml::Value`
  (`crates/ui/ess-ui-check/src/rules.rs:518-584`, `Fixtures`, `view_rows`). Only two checks look
  at them: `fixture_per_view` (a view has a fixture at all, `:1253-1260`) and
  `board_widget_kinds` (a board row's kind field, `:1197-1240`). `view_rows` reads a plain
  `rows:` only, so an answer under `by_params` is never read (the draft
  `story:board-widget-kinds-reads-every-fixture-answer`, acceptance 1, names the same reader).
- The model index keeps, per view field, its name, wire name and enum variants and nothing more
  (`crates/ui/ess-ui-check/src/model.rs:83`, `Fields`; `:117-133`, `View`; `fields_of`
  `:2018-2028`). The compiled IR it is built from has every field's `ResolvedTypeRef`
  (`crates/specify/ess-compiler/src/ir.rs:206-234`), and `compile_sources` returns it
  (`model.rs:184-225`). Events are kept by name only (`model.rs:97`).
- The model checks run as `model.check(&document, sink)` and `names::run(model, &document, sink)`
  without the fixture directory (`crates/ui/ess-ui-check/src/lib.rs:570-572`).
- The binding a renderer receives carries no row field types: `ess_ui::binding::Binding` holds the
  system, served components with view routes (path, query params with their scalar, identity,
  method, path params) and command routes, the document's names and the enums
  (`crates/ui/ess-ui/src/binding.rs:27-39`, `:265-307`). So the P0 unit
  `g03-resolved-binding` does not carry row field types either; its option P0-D11 (C) "Binding
  plus every view's row field types" was not recommended.

What passes today, read from the code above (not run): with a model view `orders.Lines {line_id:
LineId (newtype of String, prefix "ln_"), qty: Integer, status: orders.LineStatus (open | packed),
note: Optional<String>, parts: List<orders.Part>}` and `orders.Part {sku: String, count:
Integer}`, `ess ui check --model` reports no finding for any of these fixture rows of
`orders.Lines`:

| row | mistype |
|---|---|
| `{line_id: ln_1, qty: "two", status: open, parts: []}` | a text where the view declares an Integer |
| `{line_id: ln_1, qty: 2, status: open, parts: [], qtty: 3}` | an unknown member |
| `{line_id: ln_1, status: open, parts: []}` | a missing required member |
| `{line_id: ln_1, qty: 2, status: shipped, parts: []}` | an enum value outside the enum |
| `{line_id: 1, qty: 2, status: open, parts: []}` | an identity of the wrong shape (a number) |
| `{line_id: x-1, qty: 2, status: open, parts: []}` | an identity without its declared prefix |
| `{line_id: ln_1, qty: 2, status: open, parts: "9f2c…"}` | a text where a list is declared |
| `{line_id: ln_1, qty: 2, status: open, parts: [{sku: a}]}` | a struct element missing `count` |

## Outcome

With `--model`, `ess ui check` types every fixture answer of a model view, and every payload of a
fixture script, against the compiled model, under one new check id, `fixture_types`:

- **What is read.** Every answer a fixture file holds for a view the document reads with `view:`:
  a one-view file, a `views:` file, and an answer under `by_params`, through one reader that
  replaces `view_rows`. Every event of every fixture script (`fixtures.scripts`), its `payload`
  against the fields of the event it names. A `placeholder` read names no model view and is not
  typed (`unbound_placeholder` already reports it). A `derived` view takes its rows from the
  source view's fixture and is typed there.
- **What a row must be.** A mapping whose keys are the view's fields by wire name. An unknown key
  is a finding, except inside a struct declaring `undeclared_fields: ignored`. A missing key is a
  finding unless the field is `Optional`; an `Optional` field may be absent or `null`, whatever its
  `presence:`. Each value is typed by the field's declared type: Integer an integer number;
  Decimal a number or a decimal text; Binary64 a number; Boolean a boolean; String, Timestamp,
  Duration, Uuid and Bytes a text (Timestamp an RFC 3339 instant, Duration `PT…S`); an enum one of
  its wire values; a newtype its representation, and a text newtype its declared `prefix` and
  `alphabet`; a struct a mapping by these rules; a `List<T>` a sequence of `T`; a `Map<K, V>` a
  mapping of `V`; a union its tag naming a variant and its content by that variant (the wire form
  `ess-gen` projects, `crates/generate/ess-gen/src/schema.rs:76-91`); `Json` anything.
  Invariants and numeric ranges are not checked.
- **What a finding says.** One finding per mistyped value, at `fixtures/views/<view>` (or
  `fixtures/scripts/<channel>`), naming the file, the row (or event) index, the member path, what
  is written and what the model declares, for example "`fixtures/lines.yaml` row 2 of
  `orders.Lines`: `parts` is a text, and the view declares `List<orders.Part>`". At most 20
  findings per file, then one finding counting the rest.
- **Where the types live.** A new data type `ess_ui::binding::RowTypes` (per view: its fields by
  wire name with their declared shape; per event: its payload fields) and a provided method
  `ModelEnums::row_types(&self, document) -> Option<RowTypes>`, default `None`, beside
  `fixture_effects` (`binding.rs:100-118`). `ess_ui_check::Model` computes it from the IR. The
  walk that types one value against a shape is a function of `ess-ui`
  (`ess_ui::fixture_types`), over `ess_yaml::Value`, so `ess-ui` keeps no dependency but
  `ess-yaml`, and the test runner (`sx15-test-fixture-types`) calls the same walk.
- Severity per SX-D1 (default C): the shape findings are errors; the identity's `prefix` and
  `alphabet` findings are warnings. With SX-D2 (default A) the types come from the compiled IR,
  not from the route's OpenAPI schema.

## Acceptance

- First: cases in the existing target `crates/ui/ess-ui-check/tests/checks.rs` (no new test
  binary), with a model of their own (the
  `orders.Lines` view above, served by a `reached_by: network` component) and a document reading
  it with one fixture file per row of the table above. Before the change each reports
  `0 error(s)`. After: each reports exactly one `fixture_types` finding at
  `fixtures/views/orders.Lines` naming the file, the row and the member; the prefix row is a
  warning and exit 0, every other row an error and exit 1.
- A correct fixture (every field of its type, `note` absent, `note: null`, `parts: []`) gives no
  finding. A key renamed on the wire (`naming.wire`) is accepted by its wire name and refused by
  its model name; a test pins both.
- An answer under `by_params` (a one-view file holding `by_params:` and `rows:`) is typed like
  plain `rows:`; a test pins it. `board_widget_kinds` reads the same reader, so a board row inside
  such an answer is warned too (acceptance 1 of `story:board-widget-kinds-reads-every-fixture-answer`).
- A fixture script event of `orders.LineAdded {line_id: LineId, qty: Integer}` with
  `payload: {line_id: ln_9, qty: "3"}` gives one finding at `fixtures/scripts/<channel>` naming
  event 1 and `qty`; a payload leaving out a required event field gives one too. An event the
  model does not declare is not typed here (`event_in_model` refuses it already).
- A struct type with `undeclared_fields: ignored` admits an extra member; a closed struct refuses
  it. A union value with an unknown tag is refused naming the variants.
- Without `--model` nothing changes: the check runs only with a model, and every document of the
  test corpus checked without one writes the bytes it wrote before.
- The repository's documents checked with `--model` in tests
  (`crates/edge/ess-cli/tests/fixtures/ui-http/items`, `…/shelf`, and the cases of
  `crates/ui/ess-ui-check/tests/`) report no new error, or each new finding is fixed in the
  fixture in the same change and named in it.
- `schemas/ui/ess-ui.schema.yaml` lists `fixture_types` in `checks.list` (severity error; the
  message says which findings are warnings), the generated `website/docs/reference/ess-ui.md`
  shows it in the check list and in the fixtures section, `CHANGELOG.md` names it under Changed:
  "`ess ui check --model` exits 1 for a document whose fixtures contradict the model, which it
  passed before".
- Each new test is timed and expected under 5 s; none may run over 60 s, the crate's test step stays
  within 120 s, and each build-pool job within 10 minutes.

## Fit review

Verdict: **adopt, as a check (gap)**. Tree `bcfc51555`, read-only.

1. **Need.** A fixture stands in for what the served view answers; a value the view cannot answer
   makes every renderer and test downstream of it check an impossible screen. Reproduction: the
   table above, generic names.
2. **Class.** Gap: no check relates fixture values to declared types. Not a defect: no page claims
   fixtures are typed.
3. **Already expressible?** No. `field_in_model` and `row_fields` check the names a document
   writes, not the values its fixtures hold.
4. **Fit.** No new key in `ess-ui/1`. One check id beside the other `--model` checks, all errors
   (`lib.rs:165-200`). The types travel the way the fixture-effects table already travels from the
   model to the renderers (`FixtureEffects`, `binding.rs:120-200`): computed in `ess-ui-check`,
   held as plain data in `ess-ui`, so the test runner reads them without compiling a model
   (`crates/ui/ess-ui-test/src/lib.rs:338-339`). The feature, named once for the three stories
   that share it: **fixtures and live declarations are typed against the bound model**. This story
   types fixture values; `sp2-live-event-fields` (third list, item 2) checks a live event's
   declared fields against the section's view; `g07-session-script` (first report, item 7) checks
   a script's session against the read's params. Kept apart: three different defects, three
   severities (error, warning, warning plus an error), each under 600 lines; merged they would be
   split again. They share `CHECKS` (`lib.rs:109`) and `checks.list` in the schema, so they land
   one after another: `g07-session-script`, `sp2-live-event-fields`, then this.
5. **Second adopter.** Any document developed in fixture mode before its backend exists: the
   fixtures are written by hand from the model and drift when the model changes. ESS's own
   `examples/partner-portal` has fixtures but no model, so it is not reached.
6. **Cost.** One finding id in `ess-ui-check/1`; findings' `check` values grow without a format
   version (`website/docs/reference/formats.md:219`). One new data type and one provided trait
   method in `ess-ui`, not persisted in any published format (`RowTypes` is not a member of
   `Binding`). A document with mistyped fixtures that checks with `--model` today fails it after
   (exit 1 instead of 0): no format consequence, a moved exit status named in `CHANGELOG.md`.
   If P0-D11 (C) is ever chosen, `RowTypes` is the shape it would serialize.
7. **Considered.** (a) Change nothing: the application keeps correcting by hand. (b) A warning
   only (SX-D1 B): the requester's gate would not hold. (c) Validate rows against the route's
   response schema in the bound OpenAPI document (SX-D2 B): complete, with constraints, but a new
   dependency of `ess-ui-check` on `schema-contract` and validator messages instead of named
   members. Chosen: errors over the compiled shape, warnings for the identity's prefix and alphabet.

Covers sixth list, item G15. Touches third list, item 2 (`sp2-live-event-fields`) and first
report, item 7 (`g07-session-script`) as siblings of one feature, without changing them.

## Decisions

- **SX-D1. Severity.** (A) Every finding an error. (B) Every finding a warning. (C) Shape findings
  (kind, unknown member, missing member, enum value, union tag) errors; the identity's prefix and
  alphabet warnings. **Recommended: (C).**
- **SX-D2. The types' authority.** (A) The compiled IR's field types, walked by `ess-ui`'s
  function over `RowTypes`. (B) The served route's response schema from the bound OpenAPI
  document, validated with `schema-contract`. **Recommended: (A).**

## Scope

- cited: `crates/ui/ess-ui/src/binding.rs` (`RowTypes`, `ModelEnums::row_types`, beside
  `fixture_effects` `:100-118`)
- inferred: `crates/ui/ess-ui/src/fixture_types.rs` (new: the walk over `ess_yaml::Value`),
  `crates/ui/ess-ui/src/lib.rs` (module)
- cited: `crates/ui/ess-ui-check/src/model.rs` (`row_types` from the IR beside
  `fixture_effects_of` `:2127`; `impl ModelEnums for Model` `:2052`)
- cited: `crates/ui/ess-ui-check/src/rules.rs` (`Fixtures`, `view_rows` `:518-584`: one reader
  of every answer, exposed to the model checks; `board_widget_kinds` reads it)
- cited: `crates/ui/ess-ui-check/src/lib.rs` (`CHECKS` `:109`; the model checks get the fixture
  directory, `:570-572`)
- cited: `schemas/ui/ess-ui.schema.yaml` (`checks:` `:1750`), `website/docs/reference/ess-ui.md`
  (regenerated), `CHANGELOG.md`
- inferred: cases in `crates/ui/ess-ui-check/tests/checks.rs`, model and fixtures under
  `crates/ui/ess-ui-check/tests/fixtures/fixture-types/`
- Size: L, about 560 changed lines (types and walk 220, model side 120, reader and check 100,
  tests and docs 120).

## Depends on

- None hard. Lands after `g07-session-script` and `sp2-live-event-fields` (same `CHECKS` list and
  schema check list), or before them with those rebasing.
- `sx15-test-fixture-types` depends on this.

## A second report of the same gap (downstream report, finding G4; reproduced on 0.99.9)

A downstream specification of reusable building blocks (ess 0.99.2, finding G4 of one collected
report) found the same hole from its own fixtures: `ess ui check` does not check fixture rows against the
model; a row with an unknown enum value and an extra field gives 0 findings. It is not fixed in 0.99.9:
the table above is still what passes, and the report's two cases are rows of that table (an enum value
outside the enum, an unknown member).

Reproduction with a neutral model (a `shop.channel.Items` view with fields `item_id`, `label` and
`kind`, an enum `Kind` of `Tool` and `Toy`) and a document that reads it, in
`.engineering/repro/fixture-rows-are-checked-against-the-model/` (`run.sh`, `transcript.txt`):

- `{item_id: it-01, label: Hammer, kind: Gadget, colour: red}`: `Gadget` is no variant, `colour` is no
  field of the view. `ess ui check --model`: 0 findings, exit 0.
- `{item_id: it-03, label: 5}`: a number for a String, `kind` absent. 0 findings.

What the first report does not say: `ess ui test --model` accepts both as well, including rows a test
puts under its own `fixtures:` (`transcript.txt`, last command: 2 passed). The check of this story reads
the document's fixture index, so the rows a test file replaces are not reached by it.

## Acceptance (downstream report G4)

- **G4.** On the repro's document, `ess ui check --model` reports `fixture_types` for `kind: Gadget`
  (naming the row, the member and the enum's variants), for `colour` (an unknown member), for `label: 5`
  and for the absent `kind`, and exits 1; the correct rows of the same file report nothing. Red today: 0
  findings, exit 0.
- **A test's own fixtures.** `ess ui test --model` refuses a test whose `fixtures:` hold the same rows,
  with the same finding text, and the test does not run (decision FR-D1). Red today: 2 passed.
- The other acceptance points of this story stand as written.

## Fit review (downstream report G4)

Written with `.agents/skills/assessing-external-requests/SKILL.md`. The consumer is a downstream
specification of reusable building blocks.

1. **Need, apart from the syntax.** The same as above: fixtures stand for what a view answers, and
   one that no view can answer makes a check and a test pass on an impossible screen. The reporter
   proposed no syntax.
2. **Class.** Gap, as above. The report adds no case the table lacks; it confirms the two commonest
   ones on 0.99.9.
3. **Already expressible?** No (`transcript.txt`).
4. **Fit.** The feature of this story, "fixtures and live declarations are typed against the bound
   model", with its siblings, and with the declaration-level names of
   `story:a-type-name-written-in-a-widget-or-a-state-resolves-in-the-model-as-a-page-parameter-s-does`:
   every name and value a document writes is held to the model by `ess ui check --model`. What the
   report adds is the test runner as a second place the rows enter (decision FR-D1). Targets: unchanged
   from the text above.
5. **Second adopter.** As above.
6. **Cost.** As above; a test file whose own fixtures contradict the model exits 1 under `ess ui test
   --model` where it passed.
7. **Considered.** (a) Change nothing. (b) Check the document's fixtures only: leaves a test's own
   rows unchecked, the part a journey reads. (c) Chosen: one walk for both, as the sibling plan
   `sx15-test-fixture-types` in this story says.

## Decisions (downstream report G4)

- **FR-D1, the rows of a test's own `fixtures:`.** A: `ess ui test --model` types them with the
  walk of `ess ui check` and refuses the test. B: only `ess ui check`, on the document's fixtures.
  Recommended: A. The rows a journey reads are the ones a test writes; leaving them out keeps the hole
  where the report found it. Fails closed.
- **SX-D1 and SX-D2** stand open as written above, with the recommendations given there.
