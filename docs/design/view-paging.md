# View paging (`paging:`, ess/16)

Story `view-paging-and-caller-filters`, beyond10x/ess#174.

## The gap

A list endpoint takes `page` and `size` and answers one page of its ordered rows, often with the
total count beside it. ESS 0.37.0 could declare the parameters but not what they do: a view
parameter had to be read by `filter:`, which selects rows one at a time, so `page` and `size` were
refused as `unobservable_fact` (`ViewSpec::validate_params`). The refusal was right. Nothing in the
view gave the two parameters a meaning.

## The construct

```yaml
params:
  - {name: type, type: Optional<demo.jobs.JobType>}
  - {name: page, type: Integer}
  - {name: size, type: Integer}
filter: type == param.type
order_by: [job_id asc]
paging: {page: page, size: size, first_page: 0, total: true}
```

A paged read answers the rows the filter admits, in `order_by:` order: `size` of them, starting at
`(page - first_page) * size`. With `total: true` the answer also carries the number of rows the
filter admits. A read that sends neither parameter answers every row, in order. That is part of
what `paging:` declares, and it is what a scenario's other reads of the view send.

| rule | code |
|---|---|
| `paging:` below `ess/16` | `unsupported_format_version`, at `view.<name>.paging` (`ViewSpec::paging_admission`, called beside `absent_value_admission` in `primitive_admission::specification`) |
| no `order_by:` | `missing_declaration`: a slice of an unordered view names no particular rows |
| `page` or `size` names no declared parameter | `undeclared_reference` |
| a named parameter is not `Integer`, a newtype of it, or `Optional` of either | `type_mismatch` |
| `page` and `size` name one parameter | `conflicting_declaration` |
| `filter:` reads a paging parameter | `conflicting_declaration`: it would both select rows and slice them |
| `first_page` other than 0 or 1 | `unsupported_construct` |
| an unknown key in `paging:` | refused by the reader (`deny_unknown_fields`) |

The two parameters `paging:` names are exempt from the `unobservable_fact` refusal; every other
declared parameter still has to be read by the filter. An aggregate view cannot declare `order_by:`,
so it cannot be paged either.

`ResolvedView::paging` carries the block unchanged, omitted when `None`, so the IR bytes and digest
of every model without a paged view are unchanged. `schemas/generated/ess.schema.json` gains
`Paging` (regenerated with `cargo xtask schema`).

## Declined: the caller-supplied filter expression

#174 also asks for a free-form `filter` parameter in the implementation's own query language. This
cut does not implement it. The acceptance names only the page length, the slice and the total; #174
defers the filter's semantics itself ("would need a declared grammar"); a parameter nothing selects
on is exactly what `validate_params` refuses and `SemanticViewRequest::params` documents as refused;
and synthesis (`bound()`) cannot settle what a free-form value selects. A closed set of filter
fields can still be declared one optional parameter at a time, as #174 notes.

## Conformance

### What the page reads claim, and why each survives a shared target

A paged view is a ranked view, and the scenario that asserts its order arranges rows the caller's
parameters admit (`arrange_ranked`): four for a paged view (`paging::rows_wanted`), where an order
needs two. If only two or three can be arranged the order is still witnessed and the two-row pages
below are skipped; fewer than two is `ESS-SYNTH-014` as before. After the `ranked` expectation,
`synthesize/paging.rs` reads the view again:

1. the caller's parameters plus `page: first_page, size: 1`, requiring a `page` expectation with
   `rows: 1` and, where `total: true`, `total_at_least: <rows arranged>`; then `snapshot_view`;
2. the caller's parameters plus `page: first_page + 1, size: 1`, requiring `rows: 1`, the same
   floor, and `follows: {order_by: <the view's>, distinct_by: <see below>}`;
3. with four rows or more, the same two reads with `size: 2` (`rows: 2`, the first snapshotted, the
   second following it): the second starts at row 2, where a target that reads the page number as a
   row offset (`OFFSET :page`) starts at row 1, inside the first page;
4. the caller's parameters plus `page: first_page, size: <rows arranged> + 1`, requiring
   `rows: <rows arranged>, at_least: true` (at least those rows, at most the size) and the same
   floor. On a target nobody else writes to this is a partial last page, which a target that
   answers no partial last page answers empty.

`distinct_by` is the entity's identity where the view projects it. Otherwise it is the projected
fields of which the scenario knows a literal value for every one of its rows, where no two of its
rows agree on all of them; otherwise it is empty, and the continuation is asserted by its order
alone (a stated limit below). The paging parameters are never bound by name on any other read:
`bound()` skips them, so an input or a field called `size` does not half-page the unpaged reads.

A read-your-writes view is read with `query_view`, and its unpaged read is issued once more after
the pages, so an expectation appended later for the same view is about every row again. An
eventual view uses `eventually_view` for each page.

| claim | why rows another user made cannot break it |
|---|---|
| the page holds exactly one row | the scenario's two rows already fill the first two pages; other rows only add rows |
| total ≥ rows arranged | other rows only add to it |
| page 2 continues page 1: ranked no earlier than page 1's row, and not the same row | both pages are slices of one order; other rows shift which rows they hold, never the order between two pages |

What is not claimed — which rows the pages hold, and the exact total — would be a claim about the
other users of the target (§8), which this crate does not make (the reasoning that makes `counts` a
floor).

| a wrong implementation | what fails |
|---|---|
| ignores paging (every row) | `rows: 1` |
| ignores `page` (always the first page) | `follows`: the same row again |
| numbers pages from 1 where the view says 0 (page 0 and 1 both the first) | `follows`: the same row again |
| answers no partial last page | the last read holds no row |
| reads the page number as a row offset | the second two-row page repeats a row of the first |
| slices before ordering | `follows`: page 2 ranks before page 1 |
| answers the page length as the total, or no total | `total_at_least` |

`crates/verify/ess-conformance/tests/view_paging.rs` runs each against an in-memory target, and a
correct target that shares its store with rows ranked before and after the scenario's passes.

### The companion search binds the caller's parameters

The #174 repro filters by its caller's parameter, `type == param.type`. `arrange_toward` maps a
filter onto the creating command's input and asks for inputs that satisfy it, and `param.type` is a
path no input carries: no second row was found, and every ranked view filtered by a parameter was
refused `ESS-SYNTH-014` (`OrderUnwitnessed`). `paging::with_bound_params` now substitutes every
parameter the scenario bound to a literal before that search, and falls back to the unbound search;
the row found is still checked against the view as the caller reads it. No committed model has a
parameter-filtered view, so every committed suite keeps its bytes.

### Views read whole

Every witness that reads a view whole — a deletion, a preserved or unchanged row, a one-row
snapshot, an identity lookup — selected views with no parameter. It now selects them by
`paging::read_whole`: no parameter, or only the paging parameters, which a read may omit and then
answers every row. Seven sites: `synthesize.rs` (identity lookup, deletion witness, preservation),
`synthesize/existence.rs` (`one_row_view`) and `synthesize/subject_fact.rs` (three). No committed
model has a paged view, so every committed suite keeps its bytes.

### The suite vocabulary

`ViewExpectation::Page { page, size, rows, at_least?, total_at_least?, follows? }`, tag `page`, is round-3
vocabulary: a suite carrying it takes `ess-conformance/26` or `/27` (`crate::view_paging`, joined to
the pair like `aggregate_delta`: `select_fresh_format`, `coverage_version`, admission). Admission
refuses it under an older label, and refuses a page that is no claim: size 0, more rows than the
size, a total floor below the page's own rows, or a continuation with no order. The runner resolves
`follows` against the run's snapshot of the same view (`runner/page.rs`), as `changed_by` is.

The page and the size travel as the view's declared parameters, under their declared names, in
`SemanticViewRequest::params`: a target reads them as it reads any parameter, and an HTTP adapter
sends them as query parameters without knowing what they mean. Only the total needed a field:
`SemanticViewResult::total`, `None` for every answer that carries none (the hand-written `billing`
and `oracle-fixture` targets in `reference.rs` answer no paged view, and the `interpreted` target
answers nothing).

The Go and TypeScript runtimes execute the expectation at `/26` and `/27` (beyond10x/ess#188).

### Stated limits

- **A concurrent writer between the two page reads** can move a row across the page boundary and
  fail a correct shared target, as it can move `changed_by`. §8 isolation asks a target not to.
- **A view that projects no identity and whose rows the scenario cannot tell apart** is asserted
  without `distinct_by`: a target that answers the first page for every page passes it. Two rows
  identical in every projected field cannot be told apart by any reader of the view. A correct
  shared target can also fail `distinct_by` over every projected field if another user made a row
  identical, in every projected field, to one of the scenario's.
- **The other reads send no paging parameter**, which `paging:` declares to answer every row. An
  implementation that pages a read without parameters by a default size contradicts that
  declaration, and its specification should not declare `paging:` until a default can be written.

## Projections

- **OpenAPI** (`ess-gen/src/openapi.rs`): every view operation takes every declared parameter as a
  query parameter under its wire name. A filter parameter is required unless declared `Optional`,
  with its declared type as schema (its named types join the document). The page and size are
  optional integers (`minimum` `first_page` and 1). The response description says what page and
  size select, written with their wire names, and whether a total is answered. The network
  surface's description names views with `params:` as the exception to "no filter parameter";
  committed examples declare none, so their bytes are unchanged. A view without `paging:` keeps
  its bytes.
  The response schema's `rows` says it is one page in the declared order, and with `total: true`
  it gains an optional integer `total`; the response description begins "One page of the rows".
- **Documentation** (`ess-gen/src/docs.rs`): one sentence after the order sentence.
- **Code targets** (`ess-synth`): a generated view handler serves every row its projection holds, so
  Rust, Go, Web and Clap refuse a paged view by name (`views.<name>.paging`,
  `MissingRepresentation`), as they refuse `Json`.
- **Entity Runtime** lowers no view, so there is nothing to refuse.
- **Semantic diff**: `paging-changed` on a view, each side `null` or the paging contract, in
  `ess-diff/9` (`SUPPORTED_DELTA_FORMATS`, `FORMAT_RELEASES` row `None`). An `ess-diff/8` reader
  refuses such a delta; a delta that moves nothing about paging keeps its earlier format. `paging`
  is removed from the residual comparison, so the change is not also reported as unclassified.
