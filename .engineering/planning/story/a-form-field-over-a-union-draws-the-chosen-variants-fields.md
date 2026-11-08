---
format: aep.planning-md/3
id: story:a-form-field-over-a-union-draws-the-chosen-variants-fields
kind: story
status: active
title: A form field over a union draws the chosen variant's fields, and a column or record shows one
summary: 'A union-typed input edits as a variant choice plus that variant''s fields, layout optional via variants: (UI adopter composition report, P2)'
tags:
- backlog
- external-request
- queue-0.93.0
relations:
- serves: vision:O2
- informed_by: epic:ess-ui-renderer-neutral-ui
- depends_on: story:a-form-field-edits-a-list-of-records
scope:
- confidence: inferred
  path: crates/ui/ess-ui-check/src/model.rs
- confidence: inferred
  path: crates/ui/ess-ui-check/src/rules.rs
- confidence: inferred
  path: crates/ui/ess-ui-react/src/emit.rs
- confidence: inferred
  path: crates/ui/ess-ui-react/src/types.rs
- confidence: inferred
  path: crates/ui/ess-ui-react/templates/runtime/fields.tsx.tmpl
- confidence: inferred
  path: crates/ui/ess-ui-tui/src/view.rs
- confidence: inferred
  path: crates/ui/ess-ui/src/model.rs
- confidence: inferred
  path: docs/design
- confidence: inferred
  path: schemas/ui/ess-ui.schema.yaml
- confidence: inferred
  path: website/docs/reference/ess-ui.md
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-11-08T15:49:59Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "proposed", to: "active", at: "2026-11-08T15:50:00Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":2}}}
---
## Need

A command input often holds a value whose shape depends on a kind: a step's settings differ per step
type, a notification target is an email address or a webhook with headers, a payment method is a
card or a bank account. ESS states this as a `union` (adjacently tagged, unit variants from
`ess/22`, `docs/design/union-unit-variants.md`). An `ess-ui/1` form cannot edit such a field: it
offers no widget that shows the variant choice and then the chosen variant's fields, so the value
is edited as `as: json` text or left to a host component. Specifications that keep the value as
`Json` because no UI could edit anything typed lose the shape entirely.

Evidence: a UI adopter's collected report (2026-11-08) counts 16 gap lines closed by a sub-form
chosen by a type field over a typed shape (14 in one step-editor host component, 2 checklist lines
asking to type an opaque payload). 14 of the 16 also need a list of records
(`story:a-form-field-edits-a-list-of-records`).

Minimal reproduction (brand-free): a command `alerts.SetTarget` with input `target:
alerts.Target`, a union `tag: kind` with variants `Email: alerts.EmailTarget` (`{address}`) and
`Webhook: alerts.WebhookTarget` (`{url, secret}`), and a unit variant `Off`. A form shows `target`
only as `as: json`; `ess-ui-check` resolves no variant set for a union
(`crates/ui/ess-ui-check/src/model.rs:2037-2068` returns `None` for `ResolvedBody::Union`).

## Fit review

Tree `b33454b88`, binary ess 0.90.0.

1. **Need.** A form field over a union-typed input shows a choice of the union's variants and,
   below it, the inputs of the chosen variant's struct; it writes one union value. A column or
   record field shows a union value as its variant and that variant's fields. Requester's proposed
   syntax, labelled as theirs: `{name: settings, as: variant, by: module, shape: <type>, variants:
   {transfer: {fields: [...]}, …}}` over an opaque `Json` field.
2. **Class.** *Gap* in the UI document for a fact the model already states. Typing an opaque
   `Json` payload is not a UI gap: it is the adopter's model declaring a `union` (or a struct)
   instead of `Json`, which the language already admits.
3. **Already expressible?** Not within one form. `variant_by` generates one whole form per value of
   a field and names a separate form per value (`crates/ui/ess-ui/src/model.rs:1443`, `1476-1484`;
   schema `schemas/ui/ess-ui.schema.yaml:660`): it switches forms, it does not nest a typed part in
   one. A UI-side `shape:` naming a type for a `Json` field would type the value in the UI document
   only; conformance would check none of it (`docs/design/generic-language-features.md` § Policy,
   policy 1), so that idiom is refused.
4. **Fit.** The tag and the shape come from the model: the variant set is the union's
   `variants`, the choice's labels their display names, the payload fields the variant struct's
   fields, unit variants show no fields. The optional per-variant field list is written
   `variants: {<variant>: [<Field>]}` (the model's own key for the set) with the same `Field`
   entries and checks as `story:a-form-field-edits-a-list-of-records` introduces for a list
   element; absent, every field of the variant in declaration order, as a form's `fields` default
   to the command input (`ess-ui.schema.yaml:650`). No `by:` and no `shape:`: the union carries its
   own tag. Siblings: form input, form group, tab, collection column, record field; a variant
   field that is itself `as: list` with `fields:` composes. Where a wire carries the variant and the
   payload under keys the union does not use today, the content key is
   `story:a-guard-reads-a-union-tag-and-a-union-names-its-content-key`; whether this adopter's wire
   needs it: I don't know. Targets: React and the terminal draw it; `ess ui check` checks it; a
   target that cannot refuses by node path.
5. **Second adopter.** An alerting console: a rule's target is `Email {address}`, `Webhook {url,
   secret}` or `Off`.
6. **Cost.** One optional key (`variants:`) on `Field`, admitted only on a union-typed field;
   additive in `ess-ui/1`; no ESS source format; findings for a variant name the union lacks and a
   field the variant struct lacks.
7. **Considered.** (a) change nothing: `as: json` text or a host component. (b) the requester's
   `as: variant` with `by:` and `shape:`: a UI-side second type system over `Json`, unchecked. (c)
   chosen: a union-typed field draws its variants from the model, with an optional `variants:`
   layout.

## Decisions

**Accept, redesigned.** The sub-form is adopted for a model `union`; the requester's `by:` and
`shape:` are refused (the union carries its tag and shape), `as: variant` is not added (the widget
follows from the field's type), and typing an opaque payload is the model's `union`/struct, not a
UI key. Depends on the list-of-records story, which introduces the nested `Field` list checked
against a struct. Design page under `docs/design/` first.

**Read side added under the grid policy.** The request's P2 asks for the write side only (a group
of fields chosen by a type field, written into one typed object). A column or record field showing
a union value as its variant and that variant's fields is added for the same reason as the
list-of-records story's read side: a nested value is opened for every field position (form input,
form group, tab, collection column, record field), not for one cell, so a value a form writes can
also be shown where it is read.

**Content key.** The submitted value uses the union's existing adjacent tagging: the tag member and
the payload under the content key `value`, or `content` where the tag is itself named `value`
(`ess_gen::schema::union_content_key`, `docs/design/union-unit-variants.md` § Wire form). This story
adds no authored content key; `story:a-guard-reads-a-union-tag-and-a-union-names-its-content-key`
is not a dependency, and if it later lets a union name its content key, the form submits under that
key as every other projection does.

## Acceptance

- With `--model`, a form field over a union input draws a choice of its variants (display names)
  and the chosen variant's struct fields, and submits `{<tag>: <variant>, value: {…}}` (the content
  key is `content` where the tag member is itself named `value`, as
  `ess_gen::schema::union_content_key` projects it); a unit variant submits the tag only (a React
  test).
- Changing the variant clears the previous variant's inputs from the submitted value.
- `variants: {Webhook: [url]}` shows only `url` for that variant; `ess ui check --model` refuses, by
  node path, a `variants` key the union does not declare, a field its variant struct lacks, and
  `variants:` on a field that is not a union.
- A variant field `as: list` with `fields:` edits inside the chosen variant.
- A collection column or record field over a union shows the variant's display name and its fields,
  in React and in the terminal renderer.
- The terminal renderer edits a union field in an `ess ui test` case.
- A document without a union-typed field keeps its generated React project byte-identical.
- The `ess-ui/1` reference documents the key and the rendering of a union field.

## Scope

- `docs/design/` — new design page (inferred)
- `crates/ui/ess-ui/src/model.rs` — `Field.variants` (inferred)
- `schemas/ui/ess-ui.schema.yaml` (inferred)
- `crates/ui/ess-ui-check/src/model.rs`, `crates/ui/ess-ui-check/src/rules.rs` (inferred)
- `crates/ui/ess-ui-react/templates/runtime/fields.tsx.tmpl`, `crates/ui/ess-ui-react/src/emit.rs`,
  `crates/ui/ess-ui-react/src/types.rs` (inferred)
- `crates/ui/ess-ui-tui/src/view.rs` (inferred)
- `website/docs/reference/ess-ui.md` (inferred)
