---
format: aep.planning-md/3
id: story:a-generated-server-serves-a-paged-view
kind: story
status: draft
title: 'A generated server serves a paged view: its page, its size and its total'
summary: Rust, Go and web synthesis refuse every paged view; the generated route slices the declared order by the page and size parameters and answers the filtered total beside the rows
tags:
- adopter-report
- ess-0.97.1
relations:
- decomposes: epic:adopter-gaps-generated-code
- serves: vision:O2
- informed_by: story:view-paging-and-caller-filters
- informed_by: story:a-view-answer-takes-its-protocols-list-shape
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: inferred
  path: crates/generate/ess-synth/src/go/behaviour/query.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/go/http.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/go/obligation.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/go/port.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/go/store.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/paging.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/rust/behaviour/query.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/rust/http.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/rust/obligation.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/rust/port.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/rust/store.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/view_query.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/web/bridge.rs
- confidence: inferred
  path: crates/generate/ess-synth/tests/view_paging.rs
- confidence: inferred
  path: docs/design/view-paging.md
revision: 2
---
## Outcome

`ess generate synthesize` emits a Rust, Go and web workspace for a model with a paged view
(`paging: {page, size, first_page, total}`, `ess/16`). The generated server reads the page and the
size from the query string under their wire names, answers the rows the filter admits in
`order_by:` order, `size` of them from `(page - first_page) * size`, and with `total: true` answers
the number of rows the filter admits beside them: `{"rows": [...], "total": N}`, the shape the
OpenAPI projection already publishes (`docs/design/view-paging.md:173-174`). The view's synthesized
page reads pass against the generated server.

## Need

A downstream specification pages a list read by page number and size, numbered from 0, with the
total beside the rows, and wants its backend generated. Every code target refuses the whole
workspace.

Reproduced on 0.97.1 with a model of our own (`format: ess/37`, domain `demo.search`: entity
`Doc {added_ms: Integer, hit: Hit}`, command `AddDoc`, component `reached_by: network`, and the view
below; an `ess-http/1` binding routes the view to `GET /documents/page`). `ess specify validate`
exits 0.

```yaml
views:
  - name: demo.search.DocPage
    source: demo.search.Doc
    consistency: eventual
    params:
      - {name: page, type: Optional<Integer>}
      - {name: limit, type: Optional<Integer>}
    order_by: [added_ms desc, doc_id asc]
    paging: {page: page, size: limit, first_page: 0, total: true}
```

```text
$ ess generate synthesize --path system.yaml --target rust --layout crate --http http.yaml --out out/rust
rust target cannot emit this workspace
views.demo.search.DocPage.paging: this target serves a view as every row its projection holds, with no page and no total
$ ess generate synthesize --path system.yaml --target go --http http.yaml --out out/go
go target cannot emit this workspace
views.demo.search.DocPage.paging: this target serves a view as every row its projection holds, with no page and no total
$ ess generate synthesize --path system.yaml --target web --out out/web
web target cannot emit this workspace
views.demo.search.DocPage.paging: this target serves a view as every row its projection holds, with no page and no total
```

The same model without `paging:` and its two parameters synthesizes on all three targets (12
capabilities generated, 0 refused). Removing only `paging:` is refused `ESS-VIEW-003`
(`unobservable_fact`) for `page` and `limit`, as the design intends (`view-paging.md:41-43`).

## Fit review

1. **Need.** A generated server answers one page of a declared order and the filtered count. The
   requester's words (theirs): "serve a paged view in the generated server (page and size inputs,
   total in the answer)". Their server today also answers a `capped` flag; that is a different
   fact (below).
2. **Class.** Gap. The model, IR, OpenAPI, documentation, diff and conformance all carry `paging:`
   (`view-paging.md` § The construct, § Conformance, § Projections); only the code targets refuse
   it, by design of the first cut (`view-paging.md:176-178`,
   `crates/generate/ess-synth/src/paging.rs:1-38`), and the planner keeps a paged view's query an
   obligation (`crates/generate/ess-synth/src/view_query.rs:28-30`). No story makes a generated
   server serve one: `story:a-view-answer-takes-its-protocols-list-shape` (active) relied on the
   refusal ("covers the new mode with no change") and listed "an answer type for a paged view, in
   Rust and Go" among the parts that need "a story of its own"; none was filed.
3. **Already expressible?** No. Dropping `paging:` and the parameters synthesizes, but the page and
   the total leave the specification, so the suite no longer checks the figures the reader sees:
   no idiom (`docs/design/generic-language-features.md` § Policy, policy 1).
4. **Fit.** The cell is "paged view" in the code-target row; the decision opens it for every
   paging form ESS declares, not one: page-numbered with or without `total`, and the continuation
   form of `story:a-view-answer-takes-its-protocols-list-shape` when that lands. No source or
   binding key is added. The parameters already travel as declared parameters under their wire
   names (`view-paging.md:141-146`) and the generated route already decodes declared parameters
   (`story:served-view-params`, `crates/generate/ess-synth/src/rust/http.rs` `params_route`). The
   answer gains one member, `total`, which the process seam already reads
   (`crates/verify/ess-conformance/src/process/wire.rs:411-417`) and the OpenAPI schema already
   declares. Siblings: aggregate views cannot be paged (`view-paging.md:42-43`); a binding that
   moves the paging facts into headers (`story:a-view-route-carries-its-paging-facts-in-headers`,
   active) or a wrapped member (`story:a-wrapped-answer-carries-literal-members-and-paging-facts`,
   active) is served by those stories or keeps a refusal by name in generated servers; Clap
   follows or keeps its refusal by name; Entity Runtime lowers no view.
5. **Second adopter.** An admin console listing orders 25 per page, numbered from 1, with "page 3
   of 12" computed from the total.
6. **Cost.** No source format, IR, binding or suite format change. A model without a paged view
   keeps its generated bytes; no committed example has one (`view-paging.md:121,130`). Generator
   work, read from the code, rough: the planner (`view_query.rs`), the generated queries
   (`rust/behaviour/query.rs`, `go/behaviour/query.rs`), the view port signature and its return
   (`rust/port.rs` `view_params`, `go/port.rs` `view_params`), the route answer
   (`rust/http.rs:1654`, `go/http.rs:2012`), the store and obligation emitters of both targets, the
   web bridge (`web/bridge.rs`), and `paging.rs` itself: about 14 files in `ess-synth`, plus the
   inverted `tests/view_paging.rs`. A realization whose view query stays an obligation must
   implement the page and the total itself: a public generated-API change for a paged view only.
7. **Alternatives.** (a) Change nothing: the requester hand-writes the paged handler outside the
   generated server, and the suite's page reads check it there. (b) Serve the page and drop the
   total: refused, the declaration promises it. (c) Serve every paging form ESS declares with the
   total in the answer the OpenAPI document already publishes: chosen.

## Decisions

accept, redesigned. Adopted as the generic feature "a generated server serves a paged view", for
every paging form, rather than the requester's one form; nothing of the requester's wire beyond
`{rows, total}` is taken.

Design points the implementation writes into `docs/design/view-paging.md` § Projections (the
"Code targets" item is replaced):

- A read that sends neither parameter answers every row in order (`view-paging.md:25-28`); with
  `total: true` every answer, paged or not, carries `total`.
- A read that sends the size without the page answers the first page. A read that sends the page
  without the size is answered `400` naming the size parameter, because no default size can be
  declared yet (`view-paging.md:159-161`); a default comes with
  `story:page-numbered-paging-declares-a-default-and-a-maximum-size` (draft).
- A page below `first_page` or a size below 1 is answered `400`, as the OpenAPI `minimum` says
  (`view-paging.md:167-168`). A page past the last row answers no row and the total.
- The total counts the rows the filter admits before slicing; slicing follows ordering, with the
  identity appended where `order_by:` leaves ties (G2).
- Where the view's query stays an obligation, the generated port takes the page and the size and
  returns the rows and the total, so the hand-written realization owes both.

The requester's `capped` flag is not part of this story: a cap on the rows or on the count is
`story:a-view-may-declare-a-fixed-server-side-row-cap` (draft) and the `max_size:` of
`story:page-numbered-paging-declares-a-default-and-a-maximum-size` (draft). A total that stops
counting at a cap would contradict "the number of rows the filter admits" (`view-paging.md:26-27`),
which is why this story writes no `capped` member and the design must decide it there.

## Acceptance

- The reproduction above synthesizes on `--target rust --layout crate --http`, `--target rust`
  (workspace), `--target go --http` and `--target web`: exit 0, no cause naming `paging`.
- With four rows stored, the generated Rust and Go servers answer `GET /documents/page?page=0&limit=1`
  with one row and `"total":4`, `page=1&limit=1` with the next row in order, `page=1&limit=2` with
  rows 3 and 4, `page=0&limit=5` with four rows, no parameters with every row and `"total":4`,
  `page=-1` and `limit=0` and `page=1` alone with `400`. One `ess-synth` test per target runs the
  built server.
- The suite `ess verify conform synthesize` writes for the reproduction (its `page` expectations,
  `view-paging.md:66-78`) passes against the generated Rust and Go servers, with `first_page: 0`
  and `first_page: 1`, and `total: false`; a generated server altered to ignore `page` fails it.
- A view declaring `cursor:` (once `story:a-view-answer-takes-its-protocols-list-shape` lands), and
  a binding that places paging facts in a header or a wrapped member, either are served or keep a
  refusal naming the view; none is served as every row.
- `crates/generate/ess-synth/tests/view_paging.rs` asserts the served page instead of the refusal;
  every committed example tree keeps its bytes.

## Out of scope

- A server-side cap and a `capped` answer member (the two drafts named under Decisions).
- A default and a maximum page size (`story:page-numbered-paging-declares-a-default-and-a-maximum-size`).
- Generated clients and UI pagers (`ess-ui`), which already page through their own stories.
- `Binary64` in the same model: `story:every-code-target-carries-a-finite-binary64`. A model with
  both is refused for `Binary64` alone, because synthesis stops at the first refusal family
  (`crates/generate/ess-synth/src/lib.rs:488-493`), so the paging refusal shows only after it.

## Scope

- inferred: crates/generate/ess-synth/src/paging.rs
- inferred: crates/generate/ess-synth/src/view_query.rs
- inferred: crates/generate/ess-synth/src/rust/behaviour/query.rs
- inferred: crates/generate/ess-synth/src/rust/port.rs
- inferred: crates/generate/ess-synth/src/rust/http.rs
- inferred: crates/generate/ess-synth/src/rust/store.rs
- inferred: crates/generate/ess-synth/src/rust/obligation.rs
- inferred: crates/generate/ess-synth/src/go/behaviour/query.rs
- inferred: crates/generate/ess-synth/src/go/port.rs
- inferred: crates/generate/ess-synth/src/go/http.rs
- inferred: crates/generate/ess-synth/src/go/store.rs
- inferred: crates/generate/ess-synth/src/go/obligation.rs
- inferred: crates/generate/ess-synth/src/web/bridge.rs
- inferred: crates/generate/ess-synth/tests/view_paging.rs
- inferred: docs/design/view-paging.md
- inferred: CHANGELOG.md
confidence: medium
