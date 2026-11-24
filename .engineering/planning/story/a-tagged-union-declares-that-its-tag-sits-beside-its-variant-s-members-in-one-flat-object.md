---
format: aep.planning-md/3
id: story:a-tagged-union-declares-that-its-tag-sits-beside-its-variant-s-members-in-one-flat-object
kind: story
status: draft
title: A tagged union declares that its tag sits beside its variant's members, in one flat object
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:the-specification-format-ess-39-is-open-as-the-newest-reads-and-compiles-every-specification-exactly-as-ess-38
revision: 1
---
## Need

A service answers a value that is one of several shapes as one flat object. The tag is a member
beside the variant's own members, `{"kind": "parcel", "name": "north"}`, and a variant with no
members is `{"kind": "none"}`. The service cannot change this form, because older clients read it.
ESS writes and reads every union in one other form, `{"kind": "parcel", "value": {"name":
"north"}}`, so a specification of that service states a wire it does not answer.

Where the form is decided today:

- **The specification states the tag only.** A union is `{tag, variants}`, with no key for where
  the content sits (`crates/specify/ess-domain/src/types.rs:1251-1266`, authored form `:1920-1929`).
- **One constant in the projection fixes the layout.** `CONTENT = "value"`, or `"content"` when the
  tag is itself called `value` (`crates/generate/ess-gen/src/types.rs:189-196`). The module says so
  itself: "the projection fixes a layout — `{"kind": "person", "value": …}` — that the model does
  not state" (`types.rs:58`). JSON Schema writes `oneOf` of adjacently tagged branches plus
  `x-ess-union-tag` (`types.rs:45`, `:342`).
- **Every target reads that one constant** through `ess_gen::schema::union_content_key`
  (`crates/generate/ess-gen/src/schema.rs:76-77`):
  - generated Rust (`crates/generate/ess-synth/src/rust/wire.rs:167-168`, `:270`);
  - generated Go (`crates/generate/ess-synth/src/go/http.rs:635-636`, `:761`);
  - the web catalogue (`crates/generate/ess-synth/src/web/catalog.rs:486`);
  - conformance (`crates/verify/ess-conformance/src/input.rs:1753`, `accessor.rs:242-250`,
    `binary64_values.rs:420`, `one_time_response.rs:402`).
- **`ess-http/1` has no say.** It refuses a union where a query value is needed and states nothing
  about a union body (`crates/specify/ess-http/src/lib.rs:5138-5155`, `:5484-5507`).
- **The UI binding carries no type** (`crates/ui/ess-ui/src/binding.rs:25-40`).

The same value travels in a command's body, in a view's rows, in an event on a broker, and in a
conformance suite. Its form is therefore a property of the type, read by every target, not a
property of one HTTP route.

Minimal reproduction (fresh, not run here):

```yaml
types:
  - {name: shop.depot.Parcel, kind: struct, fields: [{name: name, type: String}]}
  - name: shop.depot.Item
    kind: union
    tag: kind
    variants: {parcel: shop.depot.Parcel, none: null}
```

`ess generate --kind schema` writes the `parcel` branch as `{kind: "parcel", value: {name}}`. The
service's `{"kind": "parcel", "name": "north"}` fails that schema.

The requester's proposal, restated: a declared wire form on the union ("flat", tag inline), or
that form as the default whenever a tag is set.

## Outcome

From `ess/39` a union may declare `layout: flat`. Its value is one object holding the tag member
and, for a variant whose payload is a struct, that struct's members beside it. A unit variant is
the tag alone. Without `layout`, or with `layout: adjacent`, a union keeps today's form and its IR
bytes. The compiled IR carries the layout only when it is `flat`. JSON Schema and OpenAPI write a
flat branch as one object, the tag a `const` beside the struct's properties, plus
`x-ess-union-layout: flat`. A generated target that does not yet read the flat form refuses the
model by name until `sl05-union-flat-codecs` lands.

## Acceptance

- Red first, in `crates/specify/ess-domain/tests/`: the reproduction with `layout: flat` at
  `format: ess/39` validates. At `ess/38` it is refused as `unsupported_format_version`, naming
  `ess/39`.
- Refused by name at `<type>.layout`:
  - a flat union with a variant whose payload is not a struct (a primitive, a list, a map, an enum,
    a union, or a newtype over one of them): the members would have nowhere to sit;
  - a struct payload with a member whose wire name equals the tag;
  - a `layout` value other than `adjacent` and `flat`;
  - `layout` on a type that is not a union.
- `ess specify compile` writes `layout: flat` into the union's IR body only when declared. A model
  without it keeps its IR bytes and digest. An old-reader compatibility test shows the `ess/38` IR
  reader refusing the key by name.
- `ess generate --kind schema` writes the `parcel` branch as
  `{type: object, properties: {kind: {const: parcel}, name: …}, required: [kind, name]}`. The
  `none` branch is `{kind: {const: none}}`. Each branch is closed unless its struct declares
  `undeclared_fields: ignored`. The service's answer passes the schema, and the adjacent form fails
  it.
- `ess verify diff` classifies a layout change as breaking for every reader. It is not left
  unclassified.
- Generated Rust, Go, web and the conformance writer refuse a model with a flat union by name until
  `sl05-union-flat-codecs` lands.
- `website/docs/guides/specify/unions.md` § "On the wire" shows both layouts.
  `website/docs/reference/spec-versions.md` (the `ess/39` row) and `docs/design/union-unit-variants.md`
  name the key.

## Fit review

Tree `bcfc51555`, read-only. Second report, G5. The reporter's answer of 2026-11-24 settled the wire
form.

1. **Need.** A specification describes a service whose union wire is internally tagged and cannot
   change. Reproduction above.
2. **Class.** *Gap*. The adjacent layout is the projection's choice, recorded as one the model
   "does not state" (`types.rs:58`). A model that must state it has no way to.
3. **Already expressible?** No. A struct of `Optional` fields plus an enum `kind` loses the variant
   check, the guide's argument against that idiom (`unions.md`, "when a union beats a struct of
   optional fields").
4. **Fit.** The form belongs to the type, because every target reads one constant for it
   (above). So it is declared on the type, beside `tag`. The sibling precedent is `shorthand:` on a
   struct (`ess/27`), which also chose a second wire spelling of a type and made every writer
   handle it. The requester's "default whenever a tag is set" is refused: it would change the wire
   of every existing union.
5. **Second adopter.** An event schema shared with a message bus whose payloads are
   `{"type": "created", "id": …}` objects, the common internally tagged JSON form.
6. **Cost.** A specification format: it is new surface, and a declared layout changes the meaning
   of the value's bytes. By policy 2 it does not join `ess/38` (whose row is the member address of
   a struct input). It joins `ess/39`, opened by the first report's `g15-ess39-opens`. One
   optional IR key, absent by default. One schema extension. One diff classification. Whether a
   conformance suite carrying a flat value needs the next suite major is decided in
   `sl05-union-flat-codecs`.
7. **Considered.** See SL-D13: a key in the HTTP binding; the UI reading any object with the tag;
   decline.

## Decisions

**Accept, redesigned.** A declared `layout` on the union, with today's form the default.

- **SL-D13.** Where is the flat form declared? Default A, `layout: flat` on the union from `ess/39`.
- **SL-D14. The key's spelling.** (A) `layout: adjacent | flat`. (B) `tag_inline: true`.
  (C) `wire: flat`, which is refused because `wire:` already names an enum variant's spelling
  (`ess/5`). **Recommended: A.**

## Scope

- cited: `crates/specify/ess-domain/src/types.rs` (`TypeBody::Union` 1251-1266, authored 1920-1929)
- cited: `crates/generate/ess-gen/src/types.rs` (layout table 45, 58; `CONTENT` 189-196), `schema.rs` (76-77)
- inferred: `crates/specify/ess-compiler/src/` (IR body), `crates/verify/ess-diff/src/change.rs`,
  `crates/generate/ess-gen/src/model_types.rs` (591), the by-name refusal in `ess-synth`,
  `website/docs/guides/specify/unions.md`, `spec-versions.md`, `docs/design/union-unit-variants.md`
- Size: M, about 480 changed lines (unit 1 of 3 for the union half of G5).

## Depends on

`g15-ess39-opens` (first report, P2).

## Source

A downstream application, 2026-11-24, second report, G5, and its answer of the same day.
