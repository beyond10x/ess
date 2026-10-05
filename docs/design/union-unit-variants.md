# Union unit variants (source `ess/22`)

Status: implemented for beyond10x/ess#418 (`story:union-unit-variants`). Source format `ess/22`
is shared with the other constructs of the unreleased round; this construct takes no format of its
own.

## The construct

A union variant declared with no type is a **unit variant**: it carries nothing.

```yaml
types:
  - name: demo.work.Completion
    kind: struct
    fields:
      - {name: outcome, type: String}
  - name: demo.work.Status
    kind: union
    tag: kind
    variants:
      Open:                              # a unit variant; `Open: ~` and `Open: null` are the same
      Complete: demo.work.Completion     # a payload variant, as before
```

- A union still declares at least one variant; `variants: {}` keeps its `empty_declaration`
  refusal. A union of only unit variants is admitted.
- Below `ess/22` a variant with no type is refused as `unsupported_format_version` at
  `types.<union>.variants.<label>`, naming `ess/22`. Before this change an older reader failed the
  document at parse time with `invalid type: unit value, expected a string`, with no location.
- A unit variant is always inhabited, so a union whose payload variants all recurse is buildable
  through it.

**Union or enum.** A union of only unit variants and an enum both name one of a fixed set of
labels; they differ only on the wire. An enum value is a string (`"Open"`); a union value is an
object (`{"kind": "Open"}`). Prefer an enum for a closed set of names. Declare a union when at
least one variant carries a payload, or when the value must keep the object shape because a payload
variant is expected to join it later: turning a unit variant into a payload variant keeps the tag,
whereas turning an enum into a union changes every value's JSON type.

## Model and IR

`TypeBody::Union::variants` and `ResolvedBody::Union::variants` map each label to an
`Option<…>` payload; `None` is a unit variant. The IR writes a payload as the type reference it was
before and a unit variant as `null`:

```json
"variants": {"Complete": {"kind": "declared", "name": "…"}, "Open": null}
```

So the IR, and its `spec_digest`, of every model without a unit variant keep their bytes. The suite
document's structural declarations (`selection`, `response`, `replay`, typed fields) and an
accessor plan's union operation carry `null` the same way, so no committed suite moves.

## Wire form

The existing encoding of a payload variant is adjacent tagging: the tag member, and the payload
under the content key — `value`, or `content` where the tag is itself named `value`
(`ess_gen::schema::union_content_key`). A unit variant is the same tag member and **no content
member**:

| variant | wire value |
|---|---|
| `Complete` (payload) | `{"kind": "Complete", "value": {"outcome": "shipped"}}` |
| `Open` (unit) | `{"kind": "Open"}` |

Refused, in every lane that reads one:

- a unit variant read with a content member, `null` included: `{"kind": "Open", "value": …}`;
- a payload variant read without one, unless its payload is `Optional<…>` (unchanged).

The JSON Schema branch of a unit variant pins the tag with `const`, requires only the tag and is
closed (`additionalProperties: false`), so a conforming validator refuses both shapes above.

## Lanes

| lane | unit variant |
|---|---|
| JSON Schema, `OpenAPI`, `AsyncAPI`, model types | the branch above; model-type realizations read the same branch (the TypeScript one is typechecked against both malformed shapes) |
| generated Rust | a unit enum variant (`Open,`); the codec writes and reads the tag alone and refuses a content member at `<path>.value` |
| generated Go | an empty struct variant (`StatusOpen{}`); the codec refuses a content member in the Rust reader's words |
| Web | the catalogue gives a unit variant `{"spelling": null, "type": null}`; the page reads it as absent and sends the tag alone (the page of a model without one keeps its bytes) |
| conformance synthesis | a witness for a unit variant is the tag alone. A union with a unit variant is witnessed by every variant: the branch is run once more per further variant, and where a variant does not build (a payload that recurses) the next that does is taken, a unit variant always building. A union with no unit variant keeps its one witness, its first label. A payload is written under the content key (`content` for a union tagged `value`) |
| interpreter, reference runner | values are checked by the rule above; an accessor reading through a unit variant is unavailable |
| Go and TypeScript suite runtimes | read `null` variants in declarations and accessor plans and apply the same rule |
| authored scenarios | a unit variant written with a content member is refused as an undeclared field; payload variants keep their shape-only reading |
| docs | the variant is listed as carrying nothing |
| diff | see below |
| Entity Runtime lowering | refused as `UnitVariantUnsupported`: an entity-core union variant always admits a payload member, so no exact definition exists |

The model has no predicate or `match` over union variants (a union has no predicate selectors), so
there is nothing to extend there.

## Diff

A unit variant is a variant like any other: adding one is `variant-added` (`expanded`), removing
one is `variant-removed` (`narrowed`). A unit variant that gains a payload, or a payload variant
that loses its payload, is `variant-type-changed` with `unit` on the unit side. With a required
payload every value of that variant changes its bytes in both directions, so neither revision reads
what the other writes: the change is **breaking** for callers, readers and history wherever the
union is used. An `Optional<…>` payload is read without its content member, so the tag alone is a
value of it too: a unit variant that gains one is decided as `expanded` (breaking for readers,
compatible for callers and history), and an `Optional<…>` payload variant that becomes a unit
variant as `narrowed` (breaking for callers and history, compatible for readers). A payload that
changes type between two payload variants stays `unknown`, as before.
