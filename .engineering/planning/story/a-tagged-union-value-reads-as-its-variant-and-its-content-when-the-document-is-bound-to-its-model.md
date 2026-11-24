---
format: aep.planning-md/3
id: story:a-tagged-union-value-reads-as-its-variant-and-its-content-when-the-document-is-bound-to-its-model
kind: story
status: draft
title: A tagged union value reads as its variant and its content when the document is bound to its model
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:a-structured-value-reads-as-one-line-of-its-members-in-every-renderer
- depends_on: story:a-list-field-names-the-text-that-joins-its-items-in-every-renderer
revision: 1
---
## Need

A row field whose model type is a tagged union (`kind: union, tag: <name>`) answers on the wire as
`{"<tag>": "<variant>", "value": <content>}`, or `{"<tag>": "<variant>"}` for a variant with no
content (`website/docs/guides/specify/unions.md:67-80`). A renderer does not know which member is
the tag, so after `sl05-object-one-rule` it reads `kind: pickup, value: {name: north}`. A reader
wants the variant first and then what it holds: `pickup north`.

- `Binding` carries the routes, the names and the model enums a choice names
  (`crates/ui/ess-ui/src/binding.rs:25-40`). It carries no row field type, so no renderer can tell a
  union from an object.
- `ess_ui_check` resolves a view's row field types in the compiled model and skips a union where it
  reads enums (`crates/ui/ess-ui-check/src/model.rs:2319-2333`).
- `labels` keys only a scalar value (`crates/ui/ess-ui-react/templates/runtime/fields.tsx.tmpl:127-134`,
  `crates/ui/ess-ui/src/model.rs` `Field::value_label`).

Minimal reproduction (fresh, not run here), a model type and a column over it:

```yaml
types:
  - name: probe.depot.Place
    kind: union
    tag: kind
    variants: {pickup: probe.depot.Pickup, unknown: null}
# ess-ui: columns: [{field: place, as: map}]   row: {place: {kind: pickup, value: {name: north}}}
```

With `--model`, both renderers show the object's members (after `sl05-object-one-rule`).

The requester's proposal, restated: a union reads as its variant and its fields (`pickup north`),
or `labels` per variant.

## Outcome

With `--model` (`ess generate ui --model`, `ess ui test --model`, the terminal given a binding), a
row field whose type is a tagged union, through `Optional`, newtypes and a list, reads as its
variant's label followed by its content: a scalar content as itself, a struct content as its member
values in declaration order, joined with a space. The variant's label is the field's `labels` entry
for the variant's wire name, else the wire name itself. Without a model the value reads by the
object rule. The document gains no key: `labels` on a union field labels its variants.

## Acceptance

- Red first, in `crates/ui/ess-ui-react/tests/value_formats.rs` with a binding computed from a
  model holding the union above: the row `{place: {kind: pickup, value: {name: north}}}` renders
  `pickup north`, and `{place: {kind: unknown}}` renders `unknown`. It fails before the change.
- With `labels: {pickup: Pickup point}` the first renders `Pickup point north`.
- The terminal renders the same text from the same binding
  (`crates/ui/ess-ui-tui/tests/value_formats.rs`).
- The binding JSON carries, per view route, each union row field and its tag, under a key that is
  absent when the document reads no union field. A binding computed for a document without one
  keeps its bytes, as `enums` keeps them (`binding.rs:35-39`).
- Without `--model` the same document renders by the object rule, which is unchanged.
- `ess ui check --model` reports nothing new. `labels` on a union field naming no variant is a
  warning at the field, naming the variants.
- The `ess-ui/1` reference says how a union field reads with a model, and that `labels` labels its
  variants.

## Fit review

Tree `bcfc51555`, read-only. Second report, G5 (union half).

1. **Need.** A value that is one of several shapes is read by which shape it is. Reproduction above.
2. **Class.** *Gap*: the renderer cannot know the tag without the model.
3. **Already expressible?** No (above).
4. **Fit.** No grid applies. The binding already carries model facts a renderer needs and has no
   compiler for: enums for a choice (`binding.rs:33-39`). A union's tag is the same kind of fact.
   `labels` already gives "the words shown for a value", so it labels a variant rather than adding a
   `labels` per variant key. "Imports never guess" holds in spirit: without the model the renderer
   does not guess that a two-member object is a union.
5. **Second adopter.** A payments list holds a payee that is a person's email or a company
   reference (`examples/billing/domains/invoice.yaml:44-49`). It reads `person ana@example.org` and
   `company C-17`.
6. **Cost.** No `ess-ui/1` key and no version. The binding gains an optional key, absent when
   unused. `crates/ui/ess-ui` stays free of the model: the key is plain data filled by
   `ess_ui_check`.
7. **Considered.** (a) A document key naming the tag on the field (`tag: kind`): it states a model
   fact in the UI and can disagree with the model. (b) Guessing a union from a `{<x>, value}` shape:
   that is a guess. (c) Chosen: the binding carries the tag from the model.

## Decisions

**Accept, redesigned.** The variant is read from the model through the binding. `labels` labels
the variants. No new key in the document.

- **SL-D2. What follows the variant's label?** (A) The content's values, in declaration order,
  joined with a space: `pickup north`, as requested. (B) The content as `key: value` pairs:
  `pickup name: north`. **Recommended: A.**

What could not be established: the second report's observed value is a flat object with the tag
beside the content's members. That is not ESS's union wire, so the reporting application's own
answer would not read this way until it answers ESS's wire.

## Scope

- cited: `crates/ui/ess-ui/src/binding.rs` (`ViewRoute` 275-295)
- cited: `crates/ui/ess-ui-check/src/model.rs` (binding computation, `enum_variants` 2319-2333)
- cited: `crates/ui/ess-ui-react/templates/runtime/fields.tsx.tmpl`, `core.tsx.tmpl`
- inferred: `crates/ui/ess-ui-react/src/emit.rs` (passes the field's tag to `FieldSpec`)
- inferred: `crates/ui/ess-ui-tui/src/view.rs` (`cell_value`), `crates/ui/ess-ui-tui/src/app.rs` (binding held at 475)
- inferred: tests as named above; `website/docs/reference/ess-ui.md`
- Size: M, about 420 changed lines.

## Depends on

`sl05-object-one-rule`. Lands after `sl04-list-separator` (the same files).

## Source

A downstream application, 2026-11-24, second report, G5 (union half).
