# A view read checks every row's shape (suite/66)

Story `a-view-read-checks-every-rows-shape`, beyond10x/ess#680.

## The gap

A view step compared only the fields its scenario named. A target that dropped a declared field
from a view's rows, or answered it with another type, passed every view step that did not name that
field. An event step already carried the event's declared shape (`PayloadShape`) and failed such a
payload. A view step carried none.

## The construct

The two view steps that hold a read to an expectation carry `shape`:

- `expect_view`, which holds the read of the `query_view` before it;
- `eventually_view`, which holds the read that ends the step, satisfied or past the deadline.

```json
{"step": "eventually_view", "view": "example.order.Orders",
 "expectation": {"expect": "contains", "fields": {"order_id": "…", "title": "…"}},
 "shape": {"order_id": {"holds": "primitive", "kind": "string"},
           "placed_at": {"holds": "primitive", "kind": "timestamp"},
           "title": {"holds": "primitive", "kind": "string"}}}
```

The shape is the view's declared fields, flattened as `PayloadShape` flattens an event: a newtype is
transparent, a struct contributes its fields under `parent.child`, an `Optional` marks every leaf
under it optional, and a list, a map and a union contribute one leaf that says only what container it
is.

The shape is checked on every row the read returns, with the event's rule (`LeafShape::admits`):

- a required leaf must be present and of its declared type;
- a declared `Optional` leaf admits absent and `null`;
- a key the view does not declare is not refused, as an undeclared event field is not.

A step whose rows do not conform fails with one diagnostic naming each row by its position and each
leaf (`row 0: placed_at was not carried`, `row 0: placed_at = 7`). An empty shape is not written and
checks nothing.

## Format consequence

A reader that predates the key would refuse it as unknown or, worse, ignore it, so a suite that
carries a row shape is ordinary `ess-conformance/66` or coverage `/67`, each implying every major
below. Every reader refuses `shape` on a view step under a lower major, before any target effect,
naming the major it requires (`a view row shape requires suite/66 or /67`). A suite with no row
shape keeps its format and its bytes.

| rule | where |
|---|---|
| `shape` on `expect_view` or `eventually_view` below `/66` | refused at admission (`UnsupportedVocabulary`) |
| a row missing a required leaf, or holding another type there | the view step fails |
| a row holding a key the view does not declare | not refused |

The Rust runner checks it (`runner.rs`, `row_shape`), and so do the generated Go runtime
(`go/runtime.go`, `rowShape`) and the TypeScript runtime (`ts/runtime.ts`, `rowShape`), with the
Rust verdicts (`tests/leaf_payloads_go.rs`, `tests/typescript_runtime.rs`, over the fixture in
`tests/support_view_row_shape`). Synthesis and authoring follow in
`synthesized-and-authored-view-reads-carry-the-rows-shape`.
