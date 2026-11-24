---
format: aep.planning-md/3
id: story:a-tagged-union-value-reads-as-its-variant-and-its-content-when-the-document-is-bound-to-its-model
kind: story
status: draft
title: A tagged union value reads as its variant and its content, in the layout its model declares, when the document is bound to its model
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:a-structured-value-reads-as-one-line-of-its-members-in-every-renderer
- depends_on: story:a-list-field-names-the-text-that-joins-its-items-in-every-renderer
- depends_on: story:a-tagged-union-declares-that-its-tag-sits-beside-its-variant-s-members-in-one-flat-object
revision: 4
---
## Need

A row field whose model type is a tagged union answers in one of two layouts. Adjacent is today's
only form, `{"<tag>": "<variant>", "value": <content>}` (`website/docs/guides/specify/unions.md:67-80`).
Flat is declared by `layout: flat` in `sl05-union-flat-wire`: `{"<tag>": "<variant>", <the
struct's members>}`. A renderer knows neither the tag nor the layout. After `sl05-object-one-rule`
it reads `kind: parcel, name: north`. A reader wants the variant first and then what it holds:
`parcel north`.

- `Binding` carries the routes, the names and the model enums a choice names
  (`crates/ui/ess-ui/src/binding.rs:25-40`). It carries no row field type.
- `ess_ui_check` resolves a view's row field types in the compiled model and skips a union where it
  reads enums (`crates/ui/ess-ui-check/src/model.rs:2319-2333`).
- `labels` keys only a scalar value (`crates/ui/ess-ui-react/templates/runtime/fields.tsx.tmpl:127-134`).

The reporter confirmed on 2026-11-24 that its service answers the flat layout and cannot change it.

Minimal reproduction (fresh, not run here): the union `shop.depot.Item` of `sl05-union-flat-wire`
(`tag: kind`, variants `parcel: shop.depot.Parcel` and `none`), and a column `{field: item, as:
map}`. With `layout: flat` the row is `{item: {kind: parcel, name: north}}`. Without it the row is
`{item: {kind: parcel, value: {name: north}}}`. With `--model` both renderers show the members by
the object rule.

The requester's proposal, restated: a union reads as its variant and its fields (`parcel north`),
or `labels` per variant.

The adjacent content member is the union's declared `content:` key (story:projections-use-a-unions-declared-content-key), and `value` (or `content`) only when none is declared; SL-D13 and SL-D14 are settled by reading on 2026-11-24 as `layout: adjacent | flat` on the union.

## Outcome

With `--model` (`ess generate ui --model`, `ess ui test --model`, the terminal given a binding), a
row field whose type is a tagged union, through `Optional`, newtypes and a list, reads as its
variant's label followed by its content. The content is read in the layout the model declares:
adjacent from `value` (or `content`), flat from the members beside the tag. A scalar content shows
as itself. A struct content shows its member values in declaration order, joined with a space. The
label is the field's `labels` entry for the variant's wire name, else the wire name. Without a
model the value reads by the object rule. The document gains no key.

## Acceptance

- Red first, in `crates/ui/ess-ui-react/tests/value_formats.rs`, with a binding computed from the
  model above: the flat row renders `parcel north`, and `{item: {kind: none}}` renders `none`. The
  same model without `layout: flat`, over the adjacent row, renders `parcel north`. It fails
  before the change.
- With `labels: {parcel: Parcel}` the flat row renders `Parcel north`.
- The terminal renders the same texts from the same bindings
  (`crates/ui/ess-ui-tui/tests/value_formats.rs`).
- The binding JSON carries, per view route, each union row field with its tag and, only when flat,
  its layout, under a key absent when the document reads no union field. A binding without one
  keeps its bytes, as `enums` does (`binding.rs:35-39`).
- A value that does not match the declared layout (an adjacent object for a flat union) renders by
  the object rule. It does not guess.
- Without `--model` the document renders by the object rule, unchanged.
- `labels` on a union field naming no variant is a warning at the field, naming the variants.
- The `ess-ui/1` reference says how a union field reads with a model, in either layout.

## Fit review

Tree `bcfc51555`, read-only. Second report, G5 (union half), and the reporter's answer of
2026-11-24.

1. **Need.** A value that is one of several shapes is read by which shape it is, in the layout the
   service answers.
2. **Class.** *Gap*: the renderer knows neither the tag nor the layout without the model.
3. **Already expressible?** No (above).
4. **Fit.** The binding already carries model facts a renderer needs and has no compiler for:
   enums for a choice (`binding.rs:33-39`). A union's tag and layout are the same kind of fact. The
   layout is the model's (`sl05-union-flat-wire`), so the UI reads it rather than declaring its own.
   `labels` labels a variant, so no new key is added.
5. **Second adopter.** A payments list holds a payee that is a person's email or a company
   reference (`examples/billing/domains/invoice.yaml:44-49`). It reads `person ana@example.org`.
6. **Cost.** No `ess-ui/1` key and no version. One optional binding key. `crates/ui/ess-ui` stays
   free of the model.
7. **Considered.** The options of SL-D13, plus a document key naming the tag on the field (a model
   fact stated in the UI, which can disagree with the model).

## Decisions

**Accept, redesigned.** The variant and the layout come from the model through the binding.

- **SL-D2. What follows the variant's label?** (A) The content's values joined with a space,
  `parcel north`. (B) `key: value` pairs. **Recommended: A.**
- **SL-D13.** Where is the flat layout declared? Default A, on the union in the specification
  (`sl05-union-flat-wire`). Under SL-D13 C this story alone reads a flat object when the tag is
  known, with no declaration.

## Scope

- cited: `crates/ui/ess-ui/src/binding.rs` (`ViewRoute` 275-295)
- cited: `crates/ui/ess-ui-check/src/model.rs` (binding computation, `enum_variants` 2319-2333)
- cited: `crates/ui/ess-ui-react/templates/runtime/fields.tsx.tmpl`, `core.tsx.tmpl`
- inferred: `crates/ui/ess-ui-react/src/emit.rs`, `crates/ui/ess-ui-tui/src/view.rs` (`cell_value`),
  `crates/ui/ess-ui-tui/src/app.rs` (binding at 475), tests above, `website/docs/reference/ess-ui.md`
- Size: M, about 450 changed lines (unit 3 of 3 for the union half).

## Depends on

`sl05-object-one-rule`, and `sl05-union-flat-wire` for the flat layout. It does not need
`sl05-union-flat-codecs`: a renderer reads the answer, and the generated codecs are not on its
path. It is merged after `sl04-list-separator` and `sl01-member-client` (shared files).

## Source

A downstream application, 2026-11-24, second report, G5.
