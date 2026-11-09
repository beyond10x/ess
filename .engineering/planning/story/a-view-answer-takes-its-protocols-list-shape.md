---
format: aep.planning-md/3
id: story:a-view-answer-takes-its-protocols-list-shape
kind: story
status: draft
title: A view pages by an opaque continuation with a declared default size
tags:
- adopter-report
- ess-0.70.0
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#750
relations:
- decomposes: epic:adopter-gaps-wire-and-answers
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:ess-37-and-conformance-64-65-are-open-for-the-epics-constructs
scope:
- confidence: cited
  path: crates/specify/ess-domain/src/system.rs
- confidence: cited
  path: crates/specify/ess-domain/src/view.rs
- confidence: cited
  path: crates/specify/ess-domain/src/view/paging.rs
- confidence: inferred
  path: crates/specify/ess-domain/tests/view_paging.rs
- confidence: cited
  path: crates/verify/ess-diff/src/change.rs
- confidence: cited
  path: docs/design/view-paging.md
- confidence: inferred
  path: schemas/generated/ess.schema.json
- confidence: inferred
  path: website/docs/reference/spec-versions.md
revision: 9
---
## Need

A list read pages by an opaque continuation, not by a page number: the first read sends a page
size, the answer carries the rows and a continuation, the next read sends the continuation back,
and the last page carries none. Two adopters need it: a log read newest first (`limit` 1..200,
default 50, `before=<cursor>`) and a protocol list operation (`pageSize`, `pageToken`, an answer
`nextPageToken`). `paging:` (`ess/16`) pages by page number and size only
(`crates/specify/ess-domain/src/view/paging.rs:39-50`), and a read that sends no size answers
every row, so a default page size cannot be stated (`docs/design/view-paging.md`, "Stated limits",
third item).

Minimal reproduction (0.92.0, `ess/36`): entity `example.log.Entry` (`entry_id`, `at: Timestamp`,
`text`), a view over it with `order_by: [at desc, entry_id desc]` and parameters
`before: Optional<String>`, `limit: Optional<example.log.Limit>` (`Limit` an Integer newtype with
invariants `value >= 1`, `value <= 200`):

```yaml
    paging: {cursor: before, size: limit}
```

```text
cursor was refused:
  - system.yaml: unknown field `cursor`, expected one of `page`, `size`, `first_page`, `total`
```

The same view with `page:` in place of `cursor:` validates (exit 0), so the bounded size
parameter is already expressible; the continuation and the default are not.

This story is the model half: the design, the source keys, validation, IR and diff. The answer's
wire placement, synthesis and the runtimes are the stories named under Decisions.

## Acceptance

- `docs/design/view-paging.md` gains a section "Continuation paging" stating: the keys below; that
  a continuation is opaque text the target mints, valid for the read that continues the order it
  came from; that the page after it holds the rows ranked after the last row of the page that
  answered it, in `order_by:` order with identity ascending appended where the author wrote no
  identity key (`docs/design/generic-language-features.md` § G2); that the answer carries a
  continuation exactly when rows remain; and what `default_size:` changes for a read sending no
  size.
- At the new source format, `paging: {cursor: before, size: limit, default_size: 50}` validates
  (`ess specify validate` exit 0) on the reproduction. `cursor:` and `page:` together are refused
  as `conflicting_declaration`; a `cursor:` parameter that is not `String`, a newtype of it, or
  `Optional` of either is `type_mismatch`; `total: true` and `first_page:` beside `cursor:` are
  refused by name; `default_size:` below 1, or outside the size parameter's declared invariants
  (`51` against `value <= 50`), is refused naming the bound. `default_size:` is admitted beside
  `page:` too, with the same meaning.
- Below the new format, `cursor:` and `default_size:` are each refused as
  `unsupported_format_version` naming it. A model whose views declare neither keeps its IR bytes
  (`ess specify compile` of `examples/` unchanged).
- `ess specify compile` writes the keys on `ResolvedView::paging`, omitted when absent;
  `schemas/generated/ess.schema.json` is regenerated.
- `ess verify diff` reports a moved `cursor:` or `default_size:` as the view's `paging-changed`.

## Scope

- cited: docs/design/view-paging.md
- cited: crates/specify/ess-domain/src/view/paging.rs
- cited: crates/specify/ess-domain/src/view.rs
- cited: crates/specify/ess-domain/src/system.rs
- inferred: crates/specify/ess-domain/tests/view_paging.rs
- cited: crates/verify/ess-diff/src/change.rs
- inferred: schemas/generated/ess.schema.json
- inferred: website/docs/reference/spec-versions.md
- inferred: CHANGELOG.md
confidence: medium

## Fit review

1. **Need.** A read answers one page of a declared order and a continuation to the next, with a
   declared default size. Requester's syntax (theirs): `before`/`after` parameters over an ordering
   key, a `limit` with bounds and a default. The earlier adopter's (a protocol list operation):
   `pageSize`, `pageToken`, `nextPageToken`. Reproduction above, `run.txt` steps 1 and 2.
2. **Class.** Gap. No key states a continuation (`view/paging.rs:39-50`, `deny_unknown_fields` at
   line 38), and a default size is called unstatable by the design itself (`view-paging.md`,
   "Stated limits": "its specification should not declare `paging:` until a default can be
   written").
3. **Already expressible?** Partly. The size bound is (a newtype with invariants on the size
   parameter validates, `run.txt` step 1). A keyset idiom (`filter: at < param.before` with
   page/size paging) is refused: a filter reading a paging parameter is `conflicting_declaration`
   (`view-paging.md:37`), and a `before` that is an order-key value is not the opaque cursor either
   adopter serves. Moving the cursor into an `UNMAPPED:` marker, the requester's workaround, takes
   the figure out of the suite (policy 1, `generic-language-features.md:588-593`).
4. **Fit.** Grid R row "view param" is unchanged: paging parameters stay exempt from the filter
   (`view-paging.md:41-43`). It extends the one existing construct, `paging:`, with one alternative
   key (`cursor:` beside `page:`) and one key that applies to both modes (`default_size:`), so the
   page-number mode gains the default too rather than leaving one cell open. The order is G2's
   total order (identity appended), so a continuation is defined on every ordered view. Siblings:
   aggregate views stay unpaged (`view-paging.md:42-43`); generated Rust, Go, Web and Clap already
   refuse every paged view by name (`crates/generate/ess-synth/src/paging.rs:18-38`), which covers
   the new mode with no change; Entity Runtime lowers no view (`view-paging.md`, "Projections").
5. **Second adopter.** Already two: the log read of this issue and the protocol list operation of
   the earlier report on this story. A third, brand-free: an audit trail read oldest first,
   `after=<token>`, 100 per page.
6. **Cost.** Source format: the next one open (`ess/37` unless another decision allocates it
   first). Two new keys inside `paging:`, no new diagnostic code. IR fields omitted when absent.
   `paging-changed` already compares the whole block, so no new diff kind. No generated-code change
   (refused by name today).
7. **Alternatives.** (a) Change nothing: the paging stays in a comment. (b) The requester's
   keyset `before`/`after` over an order key: refused, because it fixes the cursor to a key value
   both adopters keep opaque, and needs a second key per direction. (c) An opaque continuation in
   the declared order, plus a default size for both modes: chosen. A reverse page is a second view
   with the opposite `order_by:`.

## Decisions

accept, redesigned. The request's `before`/`after` over an order key becomes one opaque
continuation in the declared total order (`cursor:`), and the default becomes `default_size:` for
both paging modes. Changed from the request: no keyset value on the wire, no second direction.
Where the continuation travels on the wire (a header, a body member) is the binding's, not the
model's.

The rest of this story's earlier text is split out, as its 2026-11-02 reconcile note required. Each
of these needs a story of its own with a fit review:

- Renaming the member that carries the rows. The generated route always answers `{"rows": [...]}`
  (`json::member(&mut body, "rows")`, `crates/generate/ess-synth/src/rust/http.rs:1413`).
- Row-shaping parameters (`historyLength`, `includeArtifacts`): no parameter can omit or truncate a
  view field.
- Lifecycle state wire values. A state is written by its name, and whether a state takes `wire:` as
  an enum variant does was not tested.
- A generated request type for a view's parameters, and an answer type for a paged view, in Rust
  and Go. The Rust view query takes parameters as function arguments
  (`crates/generate/ess-synth/src/rust/port.rs`, `view_params`).

Retitle this story to "A view pages by an opaque continuation with a declared default size".

Follow-ups:

- `story:a-view-route-carries-its-paging-facts-in-headers`
- `story:conformance-follows-a-views-continuation-to-its-last-page`
- `story:a-value-outside-its-declared-type-is-answered-malformed`
- `story:the-go-and-typescript-runtimes-follow-continuations-and-check-malformed-answers`

## Source

github:beyond10x/ess#750
