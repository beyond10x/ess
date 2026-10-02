# A row filter on a read (`where:`, `ess-ui/1`)

beyond10x/ess#365. This page is the design; no code lands with it. Line references are to `main`
at `813b27ae341`.

## The gap

A read names a view and binds its parameters (`Reads`, `crates/ui/ess-ui/src/model.rs:2052-2074`:
`view`, `params`, `paging`, `debounce`, `refresh`, `cache`). Nothing on it keeps some of the rows
the view answers. `visible` hides a region, a page or a node (`model.rs:300`, `551`, `681`), and
a section's `live.only_if` drops events, not rows (`model.rs:2378`,
`schemas/ui/ess-ui.schema.yaml:1268`).

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
for this page. Until they are fixed, and for any filter a model chooses not to publish, the narrowing
can only happen in the renderer.

Fixture mode hides the gap. Both fixture adapters narrow rows by **any** bound parameter whose name
is a row field, declared or not (`crates/ui/ess-ui-react/templates/runtime/data.ts.tmpl:77-106`,
`crates/ui/ess-ui-tui/src/data.rs:207-241`). A page that binds `params: {partner_id: params.id}` to
a view that does not declare `partner_id` shows the right rows from fixtures. Against a server it
shows every partner's rows, and `read_params` refuses it under `--model`.

## The construct

```yaml
pages:
  partners.detail:
    params: {id: PartnerId}
    sections:
      - name: orders
        component: collection
        reads: {view: orders.Recent, where: row.partner_id == params.id}
        columns: [number, {field: placed_at, as: date}, status]
```

`orders.Recent` copies `partner_id` from the order's deal, so it cannot declare a `partner_id`
parameter (#360). `where:` is an optional `expr` on `Reads`. It is evaluated once per row the read
answers, with `row` bound to that row and the scope the read's `params` see (page `params`, `state`,
`shell`, and `args` inside a widget body). A row is kept when the value is `true`. The rows a
section shows, counts and pages are the kept rows.

`where:` is not part of the read's identity. React keys a read by view and evaluated params
(`data.ts.tmpl:261`), the terminal by `ReadRequest::key` (`data.rs:27-34`). Two sections that read
one view with the same params and different `where:` share one request. A change to a value
`where:` reads re-filters the held rows without reading again.

## Against a live server

| step | where | what happens |
|---|---|---|
| 1. params | server | evaluated and sent (`useRead`, `data.ts.tmpl:255-299`; `App::request`, `crates/ui/ess-ui-tui/src/app.rs:614-632`); the view's filter applies |
| 2. transfer | wire | every row the view and its params admit, every time the read runs |
| 3. live effects | renderer | applied to the held rows (`useLive`, `templates/runtime/live.ts.tmpl:431-517`; `apply_live`, `app.rs:1195`) |
| 4. `where:` | renderer | keeps the rows it holds for; everything after it sees only those |
| 5. local sort, search, paging | renderer | as today, over the kept rows (`collection.tsx.tmpl:64-72`; `arranged_rows`, `page_rows`, `app.rs:901-968`) |

Every row of the view goes over the wire on every read, and the read runs again after every
command (`invalidate()`, `templates/runtime/actions.tsx.tmpl:52`, `data.ts.tmpl:208-213`) and on
every `refresh:` tick.

**Paging.** A client filter over a server page is wrong. The page holds `size` rows before the
filter and fewer after it, the server's `total` counts rows the filter drops, and a matching row on
page 3 is never seen on page 1. `where:` is therefore refused beside `paging: server`, `cursor` and
`append` (`Paging`, `model.rs:2077-2093`), and admitted with `client` and `none`. A paged view
cannot be served by a synthesized target today anyway
(`crates/generate/ess-synth/src/paging.rs:1-30`), and `served-view-params` keeps paging refused.

**Totals and the empty state.** The total a section shows is the kept count. The empty state is
decided on the kept rows: React tests `data.rows.length === 0` in `SectionFrame`
(`templates/runtime/core.tsx.tmpl:417`), the terminal `result.rows.is_empty()` in `lifecycle`
(`app.rs:674`). Both read the unfiltered rows today, so both must receive the kept ones.

**Live effects.** `where:` is judged on the row an event leaves behind, as the terminal already
judges `only_if` (`app.rs:1237-1242`). An `insert_top` or `insert_or_patch` row that fails it is
not inserted and, paged away, not counted by `count_new`. A shown row patched so that it fails
disappears from view but stays held, so a later patch that passes brings it back. `only_if` still
runs first: `where:` narrows what is shown, `only_if` what is taken in.

**Row keys and selection.** `where:` does not change a row's key: `live.match`, else `id`, else
the row's content (`rowKey`, `core.tsx.tmpl:202-213`). A selected row that `where:` stops keeping
leaves the selection, so a bulk action never acts on a row the user cannot see. The React
collection keeps its own `selected` list (`collection.tsx.tmpl:46-58`) and the terminal writes ids
to selection state (`write_selection`, `app.rs:1297`); both drop the ids no longer kept.

**Exports.** `export: {params: same_as(<section>)}` copies the section's params, not its `where:`.
The download holds rows the section does not show. `where_export` warns.

## Size

A client filter is fine where the view's rows are few by the shape of the domain: one partner's
orders on a record page, an operator console's per-objective lists, an aggregate with one row per
group (the #361 case), lookup lists. It is wrong over a view that grows without bound: an activity
log, an event history, an audit trail. Every read moves all of it, on every command and every
refresh. Declare a view parameter there, or a view filtered to a window.

`ess ui check` cannot warn on size. A view declares no bound: `ViewSpec` carries fields, params,
filter, aggregation, order and paging, not a cardinality
(`crates/specify/ess-domain/src/view.rs:836-888`). The schema's own rule that `paging: client`
requires `view.bounded` (`schemas/ui/ess-ui.schema.yaml:1141`) names a fact no model states, and
no check enforces it.
What it can do with `--model` is warn where a parameter exists (`where_over_param`, below).

## Rules

New entries of `CHECKS` (`crates/ui/ess-ui-check/src/lib.rs:107-141`) and of the schema's
`checks.list` (`schema.yaml:1475`), each finding at `…/reads/where`:

| check | rule | severity |
|---|---|---|
| `where_expr` | the value does not parse in the `expressions.forms` grammar (`crates/ui/ess-ui-check/src/expr.rs:18-28`), or its top-level form is not boolean: a comparison (`==`, `!=`), `in [..]`, `not`, `and` or `or`. A bare path is refused: the renderers disagree on truthiness, the string `"false"` being false in the terminal (`crates/ui/ess-ui-tui/src/expr.rs:94-104`) and true in React (`templates/runtime/expr.ts.tmpl:239-244`) | error |
| `where_roots` | it reads a root other than `row`, `params`, `state`, `shell` and `args`, or a function form (`matches(params)`, `same_as(…)`, `actor.may(…)`). `actor` is refused on purpose: a filter by the signed-in actor is authorization, and a client filter keeps nobody's rows off the wire | error |
| `where_paging` | the read declares `paging: server`, `cursor` or `append` | error |
| `where_paths` | `params.<name>` names no param of the page or overlay, `state.<name>` no state in scope, `shell.<name>` no shell state; with `--model`, `row.<field>` names no field the view projects | error |
| `where_over_param` | with `--model`: a top-level conjunct `row.<f> == <e>` where the view declares a parameter its filter reads as `f == param.<p>`; the finding says to bind `params: {<p>: <e>}` | warning |
| `where_export` | an export with `params: same_as(<section>)` names a section whose read has `where:` | warning |

`where_paths` on `row` needs what `Model` drops today: its `View` holds the owning context and the
parameters only (`crates/ui/ess-ui-check/src/model.rs:59-61`), while `ResolvedView` has the projected
`fields` and the `filter` (`crates/specify/ess-compiler/src/ir.rs:1466`, `1476`). Without `--model`
no `row.` path is resolved, as for every other `row.` path today: #365 says `ess ui check` resolves
the paths of `visible` against a view's fields, and on `main` it does not; `expr.rs` only decides
whether a string is an expression.

## Renderers

Both renderers apply `where:`, in fixture and live mode alike. It runs after the `DataAdapter`
answers, so the fixture adapter's own param narrowing and the HTTP adapter's are equally upstream of
it.

**React.** `reads()` (`crates/ui/ess-ui-react/src/emit.rs:604-622`) emits `where` into `ReadSpec`
(`data.ts.tmpl:11-17`). A section applies it to `__data` after `useLive` (`emit.rs:1729-1748`), so
`SectionFrame` and every composite under `DataContext` get kept rows. A composite that reads for
itself applies it in `useData` (`core.tsx.tmpl:339-344`). `useLive`'s listener judges inserts and
`count_new` against it (`live.ts.tmpl:475-490`).

**Terminal.** The evaluator understands `not`, `==` and `!=` only, and answers `None` outside that
(`crates/ui/ess-ui-tui/src/expr.rs:1-8`); `visible` treats `None` as shown (`app.rs:539-544`). It
gains `and`, `or` and `in [..]` with `where:`. A `None` from `where:` drops the row: showing another
partner's orders is the defect this construct exists to prevent. The ten callers of `rows_of`
(`app.rs`, `view.rs`) read kept rows through one function that takes the `Reads`; `passes`
(`app.rs:1348`) applies `where:` after `only_if`; `lifecycle` (`app.rs:674`) counts kept rows.

## Format and version

**Additive within `ess-ui/1`.** The key is optional, and every document that loads today loads to
the same `Document` and renders the same. An older reader (0.50.0 is the newest release,
`CHANGELOG.md:29`) refuses a document that uses it: `Reads` is `deny_unknown_fields`
(`model.rs:2051`), both renderers load through `ess_ui` (`crates/ui/ess-ui-react/src/lib.rs:127`,
`crates/ui/ess-ui-tui/src/app.rs:236`), and `ess ui check` files the refusal as `document_loads`
(`crates/ui/ess-ui-check/src/classify.rs:51-72`). No renderer ever shows every row because it did
not know the key. The `CHANGELOG.md` entry names the first release that reads it.

## Out of scope

A row filter in the ESS model or on the wire (that is a view parameter; #360 and #361 are the fix);
a free-form filter parameter, declined in `docs/design/view-paging.md`; `where:` on an `export`;
typing `where:` against field types, which `Model` does not hold; any cardinality fact on a view.

## Implementation notes

In order, each its own pull request:

1. **Model, schema, check.** `Reads` gains `where` (`model.rs:2052`); the schema's `Reads` gains
   the field and a constraint beside `schema.yaml:1141`; six checks; `Model::View` gains projected
   field names and the parsed filter (`model.rs:59-61`, `177`); the reference is regenerated with
   `ess ui docs`; the partner-portal example gains one `where:`. Files:
   `crates/ui/ess-ui/src/model.rs`, `schemas/ui/ess-ui.schema.yaml`,
   `crates/ui/ess-ui-check/src/{lib,model,rules,expr}.rs`. No open unit touches them.
2. **React.** `emit.rs`, `data.ts.tmpl`, `core.tsx.tmpl`, `live.ts.tmpl`,
   `composites/collection.tsx.tmpl`. **Collides with uilab's `ui-react-live-binding`**, which edits
   `ess-ui-react/src/{lib,emit}.rs` and the runtime `data.ts`, `actions.tsx`, `form.tsx`,
   `confirm.tsx`. The overlap is `emit.rs` and `data.ts`: start after it lands.
3. **Terminal.** `expr.rs`, `app.rs`, `view.rs`. **Collides with `ui-tui-live-binding`**
   (`ess-ui-tui` `data`, `app`, `lib`, `http`) in `app.rs`: start after it lands.
4. **Parity.** One `ess-ui-test` case (`crates/ui/ess-ui-test/tests/parity.rs`) holding both
   renderers to the same kept rows, empty state and total over a fixture.

## Open questions

1. **Where `where:` may stand.** This page puts it on `Reads`, so it also reaches an action's
   `loads` and a choice's options. Should it be refused outside collection, board, chart,
   references and graph editor?
2. **Fail closed in the terminal.** Is dropping a row on an unevaluable `where:` right, or should
   the terminal refuse the document once its grammar covers `expressions.forms`?
3. **`where_over_param` severity.** A warning here; an error would forbid a client filter wherever a
   parameter exists.
4. **`section.<name>.selection` as a root.** Refused here, since master-detail binds a parameter
   (`examples/partner-portal/ui.yaml:327`). Is there a case without one?
