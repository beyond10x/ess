---
format: aep.planning-md/3
id: story:from-ess-39-a-view-s-filter-reads-the-row-a-parameter-names-one-hop-and-walks-that-row-s-list-fields
kind: story
status: draft
title: From ess/38 a view's filter reads the row a parameter names, one hop, and walks that row's list fields
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:the-specification-format-ess-38-is-open-as-the-newest-reads-and-compiles-every-specification-exactly-as-ess
revision: 6
---
## Need

A view cannot return "the rows of one entity that a list field of another row names", where the
other row is the one a parameter identifies. The domain fact: an order holds `lines: List<Line>`, a
struct with `product_id`; "the products of order `o-1`" is a read of `Product` with parameter
`order_id`, keeping each product some line of that order names. With a nested list (an order
holding shipments, each holding lines) it is the same read one quantifier deeper.

What a view filter reads today (`website/docs/reference/predicates.md:36`,
`docs/design/language-grids.md:29`): the source row's fields and `state`, a parameter as
`param.<name>` (a fact path, `crates/specify/ess-domain/src/view.rs:1144-1147`), and from `ess/28`
one hop through a reference **of the source**, `<reference>.<field>`
(`crates/specify/ess-domain/src/view/navigation.rs:1-23`). Missing:

- reading the row a parameter names: a parameter typed as an entity's identity is a plain value,
  not a reference; G2 lists `E[id]`, a row by identity, as "not in this cut and stay refused"
  (`docs/design/generic-language-features.md:246-249`);
- a quantifier over the rows of an entity in a view filter: refused, "`exists: {in: {entity: E}, …}`
  walks the rows of `E`, which a quantifier does only in the `where:` of a `{related: …}` read and
  of a row-set `when_related:` …" (`crates/specify/ess-domain/src/expression.rs:366-374`, raised at
  `:3048-3061`).

Minimal reproduction, written fresh (not validated here; the refusal follows from the lines above):

```yaml
types:
  - name: shop.orders.Line
    kind: struct
    fields:
      - {name: product_id, type: shop.catalog.ProductId}
      - {name: qty, type: Integer}
views:
  - name: shop.catalog.ProductsOfOrder
    source: shop.catalog.Product
    consistency: read_your_writes
    params:
      - {name: order_id, type: shop.orders.OrderId}
    fields:
      - {name: product_id, type: shop.catalog.ProductId}
      - {name: name, type: String}
    filter:
      exists: {in: param.order_id.lines, as: line, that: line.product_id == product_id}
```

The requester asked, in substance, for a view with a parameter that reads through a list field of
the row the parameter names, so that a screen can list the rows that list names; their UI document
lists every row of the entity instead and marks the screen unmapped.

## Outcome

From `format: ess/39`, a view parameter whose type is one entity's identity is a reference: the
view's `filter:` reads the row it names as `param.<p>.<field>` (a declared field, the identity or
`state`), one hop, as it reads the row a source reference names. A list or map field of that row is
a collection a quantifier walks (`exists`, `forall`, nested), its binder reading a struct element's
members. Where the parameter names no row, the path is unknown and the view shows no row. The
source stays one entity and the view's rows are a subset of the source's rows: no join. Below
`ess/39` the same filter is refused with `unsupported_format_version` naming `ess/39`. A design
page records the decision; validation and the interpreted target implement it.

## Acceptance

Format (D77, 2026-11-24): this story's construct lands in `ess/38`, opened by story:the-specification-format-ess-38-is-open-as-the-newest-reads-and-compiles-every-specification-exactly-as-ess; where this text names `ess/37`, `ess/39`, "the new format", "the next source format" or a format still to be decided as the format of this construct, read `ess/38`, and where it names `ess/36` as the format below it, read `ess/37` (`ess/37` shipped empty in 0.93.0); a reproduction model keeps the format it declares.

- First: `docs/design/view-parameter-rows.example/` holds the reproduction above (validated by a
  test in `crates/verify/ess-conformance/tests/`, as `read_api_view_idioms.rs` holds its example).
  At `ess/38` `ess specify validate` refuses the filter with `unsupported_format_version` naming
  `ess/39`; at `ess/39` it validates. Today it is refused at every format.
- At `ess/39`: `param.order_id.qty` where `qty` is no field of the named entity is
  `undeclared_reference`; `param.<p>.<field>` over a parameter that is no entity identity is refused
  with a message naming "a parameter typed as one entity's identity"; a second hop
  (`param.order_id.customer_id.name`) is refused as `ess/28` refuses a second hop.
- Refused by name, with the reason in the message, in the same cut as `ess/28` navigation
  (`website/docs/guides/specify/views.md:242-247`): an aggregate view, `paging:`, `absent:`, an `order_by:` key; and a view
  field `field: {related: {via: param.<p>, …}}` ("the same value on every row; read it from the
  named row's own view").
- `ess verify conform run --target interpreted` on scenarios over the example returns, for
  `order_id: o-1`, exactly the products `o-1`'s lines name, none of a decoy order's, none for a
  parameter naming no order, and the new set after a command changes `o-1`'s lines.
- A nested quantifier over a list of structs holding a list (`exists: {in: param.order_id.shipments, as: s,
  that: {exists: {in: s.lines, as: l, that: l.product_id == product_id}}}`) validates and the
  interpreter decides it; a test pins one row in, one row out.
- `docs/design/view-parameter-rows.md` (new) records the feature, the grid-R cells it opens, the
  refusals and their reasons, and the amendment to G2's sentence "a view reads other entities only
  by navigation from its source row" (`generic-language-features.md:287-289`): "or from the row a
  parameter names; never by a join". `language-grids.md:29` and `predicates.md:36` say it;
  `website/docs/guides/specify/views.md` gains a section; the format history row 39 in
  `crates/specify/ess-domain/src/system.rs` and `formats.md` name it.
- Every target that does not implement it yet refuses it by name (`g08-view-param-row-targets`
  owns those); synthesis may refuse the view by name until `g08-view-param-row-synthesis`.

## Fit review

Tree `bcfc51555` (`integration/wave-2026-11-24`), read-only. Report item 8 of a downstream
application's report of 2026-11-24.

1. **Need.** A read of one entity's rows narrowed by a list held on another row that the caller
   identifies. Reproduction above, brand-free; the request is paraphrased under Need, because its wording names the requester's domain.
2. **Class.** *Gap*. Neither a parameter's row nor a row-set quantifier is readable in a view
   filter (Need, with the refusal). Not a defect: no page promises it.
3. **Already expressible?** Not with the figure kept in the specification.
   - A list parameter (`story:feature-request-438`, implemented; `exists: {in: param.ids, …}`)
     lets the client read the order first and pass its product ids. That moves "which products
     belong to the order" out of the specification into the client, and synthesis refuses a list
     parameter on an ungrouped view (`story:a-list-parameter-selects-rows-outside-a-group-key`,
     draft, `ESS-SYNTH-017`). By policy 1 (`generic-language-features.md` § Policy) not an idiom.
   - A denormalised entity (one row per order and product) needs a create per list element beside
     the order's own write, which report item 12 reports refused.
   - The UI cannot compose two reads either: an `ess-ui/1` read's `params` bind expressions over
     page params and state, not another read's rows (`schemas/ui/ess-ui.schema.yaml:1249-1296`).
4. **Fit.**
   - Grid R, row "view `filter:`", column "related row (by reference)": `part` today, one hop
     through a source reference. This opens the same column for references held by parameters, in
     every view position: filter yes; measure `where:` and view field refused by name with a
     reason, as `ess/28` refused navigation there. Row-set reads in views (column "row set") stay
     closed; that is option B of P1-D6.
   - Vocabulary: `param.<p>` already names a parameter; `<reference>.<field>` already reads a
     referenced row; `exists/forall: {in, as, that}` already walks a list; `<binder>.<member>`
     already reads an element's member (`distinct: {…, by: <name>.<member>}`, `predicates.md:350`).
     No new keyword.
   - The source stays one entity and the row count stays at most the source's: the reason of G2's
     "never by a join" rule (aggregate scoping, `generic-language-features.md:287-289`) holds.
   - Siblings: commands read a row by identity through `when_related: {via: input.<field>}`; this
     is the view's counterpart, under the view's own spelling.
   - Targets: interpreted (this story); synthesis (`g08-view-param-row-synthesis`); generated Rust
     and Go view queries keep it as a named obligation, Entity Runtime refuses by name, `ess verify
     diff` (`g08-view-param-row-targets`).
5. **Second adopter.** "The members a team's rota names": `Person` rows, parameter `rota_id`,
   filter over `param.rota_id.shifts[*].person_id`. "The documents a release bundles": `Document`
   rows, parameter `release_id`, over `param.release_id.items`.
6. **Cost.** A new reading in `ess/39` (a parameter becomes navigable). `ess/38` is open in the
   wave of 2026-11-24 with member addresses of struct inputs in command guards: a different grid
   cell, so by policy 2 this does not join `ess/38` and waits for `ess/39`. No new keyword, no new
   diagnostic code; messages for the refusals; one design page; a diff classification only if
   `ess verify diff` classifies filter changes by path (I don't know).
7. **Considered.**
   - (a) Change nothing; the application keeps its list unmapped. Option C of P1-D6.
   - (b) Admit the `ess/27` rows quantifier (`in: {entity: E}`) in view filters with a tie to a
     parameter: opens grid R's "row set" column for views, which G2 names as the target, but its
     tie rule (`<name>.<field> == <candidate field>`, `predicates.md` § "From ess/27") would need a
     tie through a list element, and synthesis arranges whole row sets. Option B of P1-D6.
   - (c) An "unnest" view whose rows are the elements of a list field: a new kind of view with rows
     that are not the source's. Refused: breaks the one-entity source rule.
   - (d) Chosen: a parameter that is an identity is a reference, read one hop.

## Decisions

**Accept, redesigned.** What changed from the request: no view "over a list field"; the source
entity stays, the list is walked in its filter through the row the parameter names.

- **P1-D6 (open, with the owner as AQ5). How a view reaches the row a parameter names.** (A) `param.<p>.<field>`, a parameter
  typed as an identity is a reference, one hop, with list quantifiers over its fields. (B) The
  `ess/27` rows quantifier admitted in view filters, tied to a parameter. (C) Decline: the
  application keeps the list unmapped. **Recommended: (A).**

- FMT-B (decided: A, 2026-11-24): this story moves to `ess/38` with the rest of the pull request (D77: no `ess/39`); the Acceptance line on the format says so.

- AQ5: stays the owner's.

## Scope

- cited: `crates/specify/ess-domain/src/view/navigation.rs` (references a filter may navigate:
  add parameters), `crates/specify/ess-domain/src/view.rs` (`params`, filter typing),
  `crates/specify/ess-domain/src/expression.rs` (paths through a parameter; quantifier over a
  navigated list)
- cited: `crates/specify/ess-domain/src/system.rs` (format row 39; the `ess/39` gate), and the
  opening of `ess/39` itself if no other story has opened it (see Depends on)
- inferred: `crates/specify/ess-compiler/src/ir.rs` (the filter's lowered form carries the
  parameter reference)
- cited: `crates/verify/ess-conformance/src/interpret/views.rs` (evaluate the parameter's row)
- inferred: `docs/design/view-parameter-rows.md` and `.example/` (new),
  `crates/verify/ess-conformance/tests/view_parameter_rows.rs` (new)
- cited: `docs/design/generic-language-features.md`, `docs/design/language-grids.md`,
  `website/docs/reference/predicates.md`, `website/docs/guides/specify/views.md`,
  `website/docs/reference/formats.md`, `CHANGELOG.md`
- Size: L (about 600 changed lines).

## Depends on

- A story that opens `ess/39` with no construct of its own, in the shape of the wave of
  2026-11-24's `s0`; shared with the other report items that need `ess/39` (items 9 to 15, other
  groups). If none exists when this story starts, this story opens it.
- `ess/38` released or closed to constructs, so `ess/39` is the open format.
