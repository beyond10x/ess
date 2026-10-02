# A row filter on a read (`filter:`, `ess-ui/1`)

beyond10x/ess#365. This page is the design; no code lands with it. Line references are to `main`
at `813b27ae341`. The ess-ui owner's fit review is on beyond10x/ess#367.

## The gap

A read names a view and binds its parameters (`Reads`, `crates/ui/ess-ui/src/model.rs:2052-2074`:
`view`, `params`, `paging`, `debounce`, `refresh`, `cache`). Nothing on it keeps some of the rows
the view answers. `visible` hides a region, a page or a node (`model.rs:300`, `551`, `681`), and
a section's `live.only_if` drops events, not rows (`model.rs:2378`,
`schemas/ui/ess-ui.schema.yaml:1268`). A filter bar writes state and narrows nothing itself
(`FilterBar`, `model.rs:1296-1316`).

The intended way to narrow a read is a view parameter. Story `served-view-params` (epic
`ui-live-apps`, not yet landed) makes the synthesized Go and Rust servers decode a view's declared
parameters from the query string and pass them to the view port; until then the Rust surface drops
the query string (`crates/generate/ess-synth/src/rust/http.rs:1291-1293`). `ess ui check --model`
already holds a read to the parameters its view declares: `read_params` refuses an undeclared one
and an unbound required one (`crates/ui/ess-ui-check/src/model.rs:359-401`).

A model cannot always declare the parameter, because synthesis refuses it:

- **A parameter over a field the row copies from a related row** (#360, `ESS-SYNTH-005`,
  `ViewUndecidable`, code at `crates/verify/ess-conformance/src/synthesize.rs:1047`). `bound()`
  sends a parameter only a value the arrangement settled on the subject's own fields, or its
  identity (`synthesize.rs:7577-7596`); one it did not settle "leaves the filter undecidable and the
  view refused by name" (`synthesize.rs:7573-7576`), pushed at `synthesize.rs:5567-5578`.
- **A parameter over an aggregate group key** (#361, `ESS-SYNTH-017`, `AggregateUnwitnessed`,
  `crates/verify/ess-conformance/src/aggregate.rs:43`). A parameter scopes an aggregate only through
  a `field == param.<name>` conjunct over a field that is not a group key
  (`crates/verify/ess-conformance/src/synthesize/aggregate.rs:893-894`); anything else is refused
  (`aggregate.rs:911-920` there).

Both refusals are taken from the issues' reports and the code paths above; neither was reproduced
for this page. #360 and #361 are the real fix. Until they land, for any filter a model chooses not
to publish, and to narrow a small list with no round trip, the narrowing happens in the renderer.

Fixture mode hides the gap. Both fixture adapters narrow rows by **any** bound parameter whose name
is a row field, declared or not (`crates/ui/ess-ui-react/templates/runtime/data.ts.tmpl:77-106`,
`crates/ui/ess-ui-tui/src/data.rs:207-241`). A page that binds `params: {partner_id: params.id}` to
a view that does not declare `partner_id` shows the right rows from fixtures. Against a server it
shows every partner's rows, and `read_params` refuses it under `--model`.

**ESS already has one client row filter.** A dynamic menu's `from_view` entries carry
`filter: <expr>` over `row` (`DynamicNavEntries`, `model.rs:510`), and React keeps the rows it holds
for (`templates/runtime/navigation.tsx.tmpl:96`). Its doc says the opposite, "Rows to skip"
(`model.rs:509`, `schema.yaml:359`, `website/docs/reference/ess-ui.md:492`); the terminal does not
apply it at all (`crates/ui/ess-ui-tui/src/app.rs:493-519`). A view spells its own predicate
`filter` too. This construct takes the same word.

## The construct

```yaml
pages:
  partners.detail:
    params: {id: PartnerId}
    sections:
      - name: orders
        component: collection
        reads: {view: orders.Recent, filter: row.partner_id == params.id}
        columns: [number, {field: placed_at, as: date}, status]
  incidents.list:
    state: {severity: {type: Severity, class: page_state}}
    sections:
      - name: bar
        component: filter_bar
        binds: [state.severity]
        choices: [{name: severity, component: choice, reads: {view: severities.All}}]
      - name: list
        component: collection
        reads: {view: incidents.Open, filter: state.severity == null or row.severity == state.severity}
```

`orders.Recent` copies `partner_id` from the order's deal, so it cannot declare a `partner_id`
parameter (#360). The second page composes a filter bar, which writes `state.severity`, with a
filter over a bounded list. `filter:` is an optional `expr` on `Reads`. It is evaluated once per row
the read answers, with `row` bound to that row and the scope the read's `params` see (page `params`,
`state`, `shell`, and `args` inside a widget body). A row is kept when the value is `true`. The rows
a section shows, counts and pages are the kept rows.

It stands on the read of a collection, board, chart, references or graph editor, and on a choice's
options (`Choice.reads`, `model.rs:1235`). `filter_place` refuses it on an action's `loads`
(`model.rs:2138`), a form's `loads` (`model.rs:1161`), a record and a metric, where it would narrow
nothing a list shows. A preload is not a `Reads` (`PreloadView`, `model.rs:340-348`, which refuses
unknown keys), so the loader already refuses `filter:` there.

`filter:` is not part of the read's identity. React keys a read by view and evaluated params
(`data.ts.tmpl:261`), the terminal by `ReadRequest::key` (`data.rs:27-34`). Two sections that read
one view with the same params and different filters share one request. A change to a value the
filter reads re-filters the held rows without reading again.

**The server never sees it.** The binding a renderer reads carries a view's path and declared
parameters only (`ViewRoute`, `crates/ui/ess-ui/src/binding.rs:43-50`). A filter is not
authorization: every row is on the wire, and the reference says so.

## Against a live server

| step | where | what happens |
|---|---|---|
| 1. params | server | evaluated and sent (`useRead`, `data.ts.tmpl:255-299`; `App::request`, `app.rs:614-632`); the view's own filter applies |
| 2. transfer | wire | every row the view and its params admit, every time the read runs |
| 3. live effects | renderer | applied to the held rows (`useLive`, `templates/runtime/live.ts.tmpl:431-517`; `apply_live`, `app.rs:1195`) |
| 4. `filter:` | renderer | keeps the rows it holds for; everything after it sees only those |
| 5. local sort, search, paging | renderer | as today, over the kept rows (`collection.tsx.tmpl:64-72`; `arranged_rows`, `page_rows`, `app.rs:901-968`) |

The read runs again after every command (`invalidate()`, `templates/runtime/actions.tsx.tmpl:52`,
`data.ts.tmpl:208-213`) and on every `refresh:` tick.

**Paging.** A client filter over a server page is wrong. The page holds `size` rows before the
filter and fewer after it, the server's `total` counts rows the filter drops, and a matching row on
page 3 is never seen on page 1. `filter:` is refused beside `paging: server`, `cursor` and `append`
(`Paging`, `model.rs:2077-2093`), and admitted with `client` and `none`. A synthesized target
cannot serve a paged view today anyway (`crates/generate/ess-synth/src/paging.rs:1-30`), and
`served-view-params` keeps paging refused.

**Totals and the empty state.** The total a section shows is the kept count. The empty state is
decided on the kept rows: React tests `data.rows.length === 0` in `SectionFrame`
(`templates/runtime/core.tsx.tmpl:417`), the terminal `result.rows.is_empty()` in `lifecycle`
(`app.rs:674`). Both read the unfiltered rows today, so both must receive the kept ones.

**Live effects.** The filter is judged on the row an event leaves behind, as the terminal already
judges `only_if` (`app.rs:1237-1242`). An `insert_top` or `insert_or_patch` row that fails it is
not inserted and, paged away, not counted by `count_new`. A shown row patched so that it fails
disappears from view but stays held, so a later patch that passes brings it back. `only_if` runs
first: `filter:` narrows what is shown, `only_if` what is taken in.

**Row keys and selection.** The filter does not change a row's key: `live.match`, else `id`, else
the row's content (`rowKey`, `core.tsx.tmpl:202-213`). A selected row the filter stops keeping
leaves the selection, so a bulk action never acts on a row the user cannot see. The React
collection keeps its own `selected` list (`collection.tsx.tmpl:46-58`) and the terminal writes ids
to selection state (`write_selection`, `app.rs:1297`); both drop the ids no longer kept.

**Exports.** `export: {params: same_as(<section>)}` copies the section's params, not its filter.
The download holds rows the section does not show. `filter_export` warns.

## Size

A client filter is fine where the view's rows are few by the shape of the domain: one partner's
orders on a record page, an operator console's per-objective lists, an on-call console's open
incidents, an aggregate with one row per group (the #361 case), a choice's options. It is wrong over
a view that grows without bound: an activity log, an event history, an audit trail. Every read moves
all of it, on every command and every refresh. Declare a view parameter there, or a view filtered to
a window.

`ess ui check` cannot warn on size. A view declares no bound: `ViewSpec` carries fields, params,
filter, aggregation, order and paging, not a cardinality
(`crates/specify/ess-domain/src/view.rs:836-888`). The schema's rule that `paging: client` requires
`view.bounded` (`schema.yaml:1141`) names a fact no model states, and no check enforces it. With
`--model` the checker can warn where a parameter exists (`filter_over_param`, below).

## Rules

New entries of `CHECKS` (`crates/ui/ess-ui-check/src/lib.rs:107-141`) and of the schema's
`checks.list` (`schema.yaml:1475`), each finding at `…/reads/filter`. The first three also hold the
menu's filter at `…/from_view/filter`, whose scope is the shell's.

| check | rule | severity |
|---|---|---|
| `filter_expr` | the value does not parse in the `expressions.forms` grammar (`crates/ui/ess-ui-check/src/expr.rs:18-28`), or its top-level form is not boolean: a comparison (`==`, `!=`), `in [..]`, `not`, `and` or `or`. A bare path is refused: the renderers disagree on truthiness, the string `"false"` being false in the terminal (`crates/ui/ess-ui-tui/src/expr.rs:94-104`) and true in React (`templates/runtime/expr.ts.tmpl:239-244`) | error |
| `filter_roots` | it reads a root other than `row`, `params`, `state`, `shell` and `args`, or a function form (`matches(params)`, `same_as(…)`, `actor.may(…)`). `actor` is refused because a filter is not authorization; `section.<name>.selection` because master-detail binds a parameter (`examples/partner-portal/ui.yaml:327`) | error |
| `filter_paths` | `params.<name>` names no param of the page or overlay, `state.<name>` no state in scope, `shell.<name>` no shell state; with `--model`, `row.<field>` names no field the view projects | error |
| `filter_place` | the read is an action's or a form's `loads`, or a record's or a metric's | error |
| `filter_paging` | the read declares `paging: server`, `cursor` or `append` | error |
| `filter_over_param` | with `--model`: a top-level conjunct `row.<f> == <e>` where the view declares a parameter its filter reads as `f == param.<p>`; the finding says to bind `params: {<p>: <e>}` | warning |
| `filter_export` | an export with `params: same_as(<section>)` names a section whose read has `filter:` | warning |

`filter_paths` on `row` needs what `Model` drops today: its `View` holds the owning context and the
parameters only (`crates/ui/ess-ui-check/src/model.rs:59-61`), while `ResolvedView` has the
projected `fields` and the `filter` (`crates/specify/ess-compiler/src/ir.rs:1466`, `1476`). Without
`--model` no `row.` path is resolved, as for every other `row.` path today: #365 says `ess ui check`
resolves the paths of `visible` against a view's fields, and on `main` it does not; `expr.rs` only
decides whether a string is an expression. No committed document uses the menu's `filter`
(`examples/partner-portal/ui.yaml:103` has none), so its new checks refuse nothing committed.

## Renderers

Both renderers apply `filter:`, in fixture and live mode alike. It runs after the `DataAdapter`
answers, so the fixture adapter's own param narrowing and the HTTP adapter's are equally upstream of
it.

**React.** `reads()` (`crates/ui/ess-ui-react/src/emit.rs:604-622`) emits `filter` into `ReadSpec`
(`data.ts.tmpl:11-17`). A section applies it to `__data` after `useLive` (`emit.rs:1729-1748`), so
`SectionFrame` and every composite under `DataContext` get kept rows. A composite that reads for
itself applies it in `useData` (`core.tsx.tmpl:339-344`), a choice after its `useRead`
(`composites/choice.tsx.tmpl:27`). `useLive`'s listener judges inserts and `count_new` against it
(`live.ts.tmpl:475-490`).

**Terminal: fails closed.** The evaluator understands `not`, `==` and `!=` only, and answers `None`
outside that (`crates/ui/ess-ui-tui/src/expr.rs:1-8`); `visible` treats `None` as shown
(`app.rs:539-544`). Before the terminal step ships, its grammar covers everything `filter_expr` and
`filter_roots` admit: `and`, `or`, `in [..]`, parentheses and literals over the admitted roots. A
checked document then never yields `None`; an unchecked one that does drops the row, because
showing another partner's orders is the defect this construct prevents. The ten callers of
`rows_of` (`app.rs`, `view.rs`) read kept rows through one function that takes the `Reads`;
`passes` (`app.rs:1348`) applies the filter after `only_if`; `lifecycle` (`app.rs:674`) counts kept
rows; the dynamic menu (`app.rs:493-519`) applies its `filter` as React does.

## Format and version

**Additive within `ess-ui/1`.** The key is optional, and every document that loads today loads to
the same `Document` and renders the same in React. An older reader (0.50.0 is the newest release,
`CHANGELOG.md:29`) refuses a document that uses it: `Reads` is `deny_unknown_fields`
(`model.rs:2051`), both renderers load through `ess_ui` (`crates/ui/ess-ui-react/src/lib.rs:127`,
`crates/ui/ess-ui-tui/src/app.rs:236`), and `ess ui check` files the refusal as `document_loads`
(`crates/ui/ess-ui-check/src/classify.rs:51-72`). No renderer ever shows every row because it did
not know the key. The terminal's menu starts applying its `filter`, which it ignored; that is the
documented meaning, not a new one. The `CHANGELOG.md` entry names the first release that reads
`Reads.filter`.

## Out of scope

A row filter in the ESS model or on the wire (that is a view parameter; #360 and #361 are the fix);
a free-form filter parameter, declined in `docs/design/view-paging.md`; a filter on an `export`;
typing the filter against field types, which `Model` does not hold; any cardinality fact on a view.

## Implementation notes

In order, each its own pull request:

1. **Model, schema, check.** `Reads` gains `filter` (`model.rs:2052`); the schema's `Reads` gains
   the field and a constraint beside `schema.yaml:1141`, its doc saying a filter is not
   authorization; the menu's `filter` doc becomes "rows kept" (`model.rs:509`, `schema.yaml:359`);
   seven checks; `Model::View` gains projected field names and the parsed filter (`model.rs:59-61`,
   `177`); the reference is regenerated with `ess ui docs`; the partner-portal example gains one
   `filter:`. Files: `crates/ui/ess-ui/src/model.rs`, `schemas/ui/ess-ui.schema.yaml`,
   `crates/ui/ess-ui-check/src/{lib,model,rules,expr}.rs`, `website/docs/reference/ess-ui.md`.
   No open unit touches them.
2. **React.** `emit.rs`, `data.ts.tmpl`, `core.tsx.tmpl`, `live.ts.tmpl`,
   `composites/{collection,choice}.tsx.tmpl`. **Collides with uilab's `ui-react-live-binding`**,
   which edits `ess-ui-react/src/{lib,emit}.rs` and the runtime `data.ts`, `actions.tsx`,
   `form.tsx`, `confirm.tsx`. The overlap is `emit.rs` and `data.ts`: start after it lands.
3. **Terminal.** `expr.rs`, `app.rs`, `view.rs`. **Collides with `ui-tui-live-binding`**
   (`ess-ui-tui` `data`, `app`, `lib`, `http`) in `app.rs`: start after it lands.
4. **Parity.** `ess-ui-test` cases (`crates/ui/ess-ui-test/tests/parity.rs`) holding both renderers
   to the same kept rows, empty state and total over a fixture, the same menu entries, and no
   `None` from any filter the check admits.

## Decided in the fit review

The ess-ui owner's fit review on beyond10x/ess#367 accepted the first draft, redesigned, and
answered its open questions; all of it is made above:

- The key is `filter:`, the spelling of the menu's row filter and of a view's predicate; the checks
  are `filter_*`. The menu's `filter` takes the same grammar, roots and fail-closed rule, and its
  "Rows to skip" doc is corrected in the code pull request.
- It stands on the five listing composites' reads and on a choice's options; it is refused on
  loads and preloads. This page reads the review's list as closed, so a record's and a metric's
  reads are refused too.
- The terminal fails closed, and its grammar covers everything the check admits.
- `filter_over_param` is a warning; `section.<name>.selection` is refused.
- The page states that the binding and the server never see the filter.

No question remains open. The code may start with step 1.
