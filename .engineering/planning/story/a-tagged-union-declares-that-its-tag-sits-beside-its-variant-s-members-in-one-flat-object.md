---
format: aep.planning-md/3
id: story:a-tagged-union-declares-that-its-tag-sits-beside-its-variant-s-members-in-one-flat-object
kind: story
status: active
title: A tagged union may declare the flat layout, its tag beside its variant's members in one object
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- derived_from: story:a-guard-reads-a-union-tag-and-a-union-names-its-content-key
- depends_on: story:a-guard-reads-a-union-tag-and-a-union-names-its-content-key
- depends_on: story:projections-use-a-unions-declared-content-key
- depends_on: story:the-specification-format-ess-38-is-open-as-the-newest-reads-and-compiles-every-specification-exactly-as-ess
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-11-24T03:25:15Z", actor: "human:timo", revision: 6}
- {from: "proposed", to: "active", at: "2026-11-24T03:25:16Z", actor: "human:timo", revision: 7}
---
## Need

A service answers a value that is one of several shapes as one flat object. The tag is a member
beside the variant's own members, `{"kind": "parcel", "name": "north"}`, and a variant with no
members is `{"kind": "none"}`. The service cannot change this form, because older clients read it.
ESS writes and reads every union in the adjacent form, `{"kind": "parcel", "value": {"name":
"north"}}`, so a specification of that service states a wire it does not answer.

What the store already plans, and is not restated here:

- `story:a-guard-reads-a-union-tag-and-a-union-names-its-content-key` (active, beyond10x/ess#637)
  writes the design page `docs/design/union-variant-guards-and-content-key.md` and admits
  `content: <member>` beside `tag:`. Its fit review places the content key as one value of a union
  encoding selector, shared with an internal (flat) and an external (keyed by variant) encoding, and
  says `content:` is meaningful only under the adjacent encoding.
- `story:projections-use-a-unions-declared-content-key` (active) carries `content:` into the IR,
  makes `ess_gen::schema::union_content_key` take the resolved union instead of the tag
  (`crates/generate/ess-gen/src/schema.rs:76-78`, `types.rs:189-196`), and adds the `ess verify diff`
  kind for a changed content key.
- `story:generated-codecs-and-the-suite-use-a-unions-declared-content-key` (active) threads the
  declared key through generated Rust, Go and TypeScript, the web catalogue and the suite, under
  the next `ess-conformance` pair.

What none of them admits is the flat layout itself: the tag beside the members, with no content
member at all. The constant that fixes the adjacent layout for every target is the one those
stories replace (`crates/generate/ess-gen/src/types.rs:58`: the projection "fixes a layout ... that
the model does not state").

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

The requester's proposal, restated: a declared wire form on the union ("flat", tag inline), or that
form as the default whenever a tag is set.

## Outcome

A union may declare `layout: flat`, the flat value of the union's encoding selector; `adjacent` is
the default and keeps today's form and IR bytes. A flat union's value is one object holding the tag
member and, for a variant whose payload is a struct, that struct's members beside it. A unit variant
is the tag alone. `content:` is admitted only under `adjacent`. The compiled IR carries `layout`
only when it is `flat`, on the same union body that carries `content:`. JSON Schema and OpenAPI
write a flat branch as one object, the tag a `const` beside the struct's properties, plus
`x-ess-union-layout: flat`, through the layout-aware accessor that replaces `union_content_key`. A
generated target that does not yet read the flat form refuses the model by name until
`sl05-union-flat-codecs` lands. The format is the one the active content-key stories land in (see
Decisions).

## Acceptance

Format (D77, 2026-11-24): this story's construct lands in `ess/38`, opened by story:the-specification-format-ess-38-is-open-as-the-newest-reads-and-compiles-every-specification-exactly-as-ess; where this text names `ess/37`, `ess/39`, "the new format", "the next source format" or a format still to be decided as the format of this construct, read `ess/38`, and where it names `ess/36` as the format below it, read `ess/37` (`ess/37` shipped empty in 0.93.0); a reproduction model keeps the format it declares.

- Red first, in `crates/specify/ess-domain/tests/`: the reproduction with `layout: flat` validates
  at the format named under Decisions. Below it, it is refused as `unsupported_format_version`,
  naming the format.
- Refused by name at `<type>.layout` or `<type>.content`:
  - a flat union with a variant whose payload is not a struct (a primitive, a list, a map, an enum,
    a union, or a newtype over one of them): the members would have nowhere to sit;
  - a struct payload with a member whose wire name equals the tag;
  - `content:` beside `layout: flat` (`conflicting_declaration`);
  - a `layout` value other than `adjacent` and `flat`;
  - `layout` on a type that is not a union.
- `ess specify compile` writes `layout: flat` into the union's IR body only when declared. A model
  without it keeps its IR bytes and digest. An old-reader compatibility test shows the previous IR
  reader refusing the key by name.
- `ess generate --kind schema` writes the `parcel` branch as
  `{type: object, properties: {kind: {const: parcel}, name: …}, required: [kind, name]}`. The `none`
  branch is `{kind: {const: none}}`. Each branch is closed unless its struct declares
  `undeclared_fields: ignored`. The service's answer passes the schema, and the adjacent form fails
  it.
- `ess verify diff` reports a changed `layout` as a breaking change of the union, for callers and
  readers, in the diff kind the projections story adds for the content key.
- Generated Rust, Go, web and the conformance writer refuse a model with a flat union by name until
  `sl05-union-flat-codecs` lands.
- `docs/design/union-variant-guards-and-content-key.md` gains the flat layout as the selector's
  second value. `website/docs/guides/specify/unions.md` § "On the wire" shows both layouts.
  `website/docs/reference/spec-versions.md` and `docs/design/union-unit-variants.md` name the key.

## Fit review

Tree `bcfc51555`, read-only. Second report, G5. The reporter's answer of 2026-11-24 settled the
wire form. Revised on 2026-11-24 after a triage of double work: the content key, its IR field, its
accessor and its diff kind are the active stories'; this story adds only the flat value.

1. **Need.** A specification describes a service whose union wire is internally tagged and cannot
   change. Reproduction above.
2. **Class.** *Gap*. The adjacent layout is the projection's choice, recorded as one the model
   "does not state" (`types.rs:58`).
3. **Already expressible?** No. A struct of `Optional` fields plus an enum `kind` loses the variant
   check (`unions.md`, "when a union beats a struct of optional fields").
4. **Fit.** The form belongs to the type, because every target reads one accessor for it. The
   active guard story already put the encoding on the union as one selector, so the flat layout is
   that selector's second value. The selector's spelling is `layout:` because it must hold three
   values (adjacent, flat, and the keyed-by-variant encoding of
   `story:a-union-may-be-encoded-as-an-object-keyed-by-its-variant`), which a boolean such as
   `tag_inline: true` cannot. The requester's "default whenever a tag is set" is refused: it would
   change the wire of every existing union.
5. **Second adopter.** An event schema shared with a message bus whose payloads are
   `{"type": "created", "id": …}` objects, the common internally tagged JSON form.
6. **Cost.** New surface that changes the meaning of a value's bytes, so a specification format:
   the same one as `content:`, since both are one design decision (the union's wire layout). One
   optional IR key, absent by default. One schema extension. One diff classification in an
   existing kind.
7. **Considered.** A key in the HTTP binding (refused: the same value travels in a command body, a
   view row, a broker event and a suite); the UI reading any object with the tag (refused: a
   renderer guessing); decline.

## Decisions

**Accept, redesigned.** The flat layout is a value of the union's encoding selector.

- SL-D13 (where the layout is declared) is settled by reading: on the union, as the active guard
  story's accepted design places the encoding selector.
- SL-D14 (the spelling) is settled by reading: `layout: adjacent | flat`, because the selector must
  also carry the keyed-by-variant encoding, which `tag_inline: true` cannot, and `wire:` already
  names an enum variant's spelling (`ess/5`).
- Format: `ess/38`, the format of the content-key stories (X1 answered `ess/38`, D77, 2026-11-24).

## Scope

- cited: `crates/specify/ess-domain/src/types.rs` (`TypeBody::Union` 1251-1266, authored 1920-1929)
- cited: `crates/generate/ess-gen/src/types.rs` (layout table 45, 58; `CONTENT` 189-196),
  `schema.rs` (76-77)
- inferred: `crates/specify/ess-compiler/src/` (IR body), `crates/verify/ess-diff/src/`,
  `crates/generate/ess-gen/src/model_types.rs` (591), the by-name refusal in `ess-synth`,
  `docs/design/union-variant-guards-and-content-key.md`, `website/docs/guides/specify/unions.md`,
  `spec-versions.md`, `docs/design/union-unit-variants.md`
- Size: M, about 400 changed lines (unit 1 of 2 for the union wire half of G5).

## Depends on

- `story:a-guard-reads-a-union-tag-and-a-union-names-its-content-key` (the design page and the
  authored union's `content:` key, the same struct). This story is derived from it.
- `story:projections-use-a-unions-declared-content-key` (the IR field on the union, the accessor
  that takes the resolved union, the diff kind).
- `story:the-specification-format-ess-38-is-open-as-the-newest-reads-and-compiles-every-specification-exactly-as-ess` (opens `ess/38`).

## Source

A downstream application, 2026-11-24, second report, G5, and its answer of the same day.
