# `presence: omitted_or_null`, and a view field's carried presence

Unit G-U30 of 0.69.0, under source format `ess/31`. Stories
`story:an-input-keeps-missing-and-null-apart` (beyond10x/ess#678, the wire half only) and
`story:a-view-field-carries-its-source-presence` (beyond10x/ess#677). Builds on
[`wire-presence-json-prefix.md`](wire-presence-json-prefix.md) § 2, which this page does not repeat.

The keep-versus-clear half of #678 (`missing: unchanged`, `sent()`, a three-valued input) belongs to
the proposed ADR `architecture-decision-record:a-missing-input-keeps-and-null-clears` and is not
built here. This page gives that ADR the wire spelling it builds on.

## 1. The third presence

```yaml
- {name: reason, type: Optional<String>, presence: omitted_or_null}
```

| `presence:` | member left out | member sent as `null` | property `required` | property schema |
|---|---|---|---|---|
| `null_when_absent` | refused | absent | yes | `anyOf [T, null]` |
| `omitted_when_absent` | absent | refused | no | `T` |
| `omitted_or_null` | absent | absent | no | `anyOf [T, null]` |
| none | absent | absent | no | `T` |

**Spelling.** `omitted_or_null`, Rust `Presence::OmittedOrNull`. It names the two spellings the
member admits and says nothing about what each means, so it stays true when the ADR gives the two
different meanings. The `_when_absent` suffix of the other two is left off for that reason: under
the ADR a `null` is no longer only "absent".

**Positions.** A third variant of the one `Presence` enum, so it is admitted wherever `presence:` is:
struct, entity, command input and response, event and error fields
(`primitive_admission::presence`). View fields and view parameters keep refusing the key (§ 2).

**Validation.** As the other two: `type_mismatch` on a field that is not `Optional<T>`. It is
admitted on an `Optional<Json>`: `null` there spells absence, as under `null_when_absent`.

**Format.** `ess/31` (`FORMAT_HISTORY` row 31, unreleased, shipping in 0.69.0). Below it the value
is refused as `unsupported_format_version` naming `ess/31`, at the field's `presence` key, as the
other `ess/31` constructs are refused (`check_empty_list_target`, `append_list`). An older reader
fails the value as an unknown variant with no version hint, which is why the gate exists. A model
without it keeps its IR bytes; the published specification schema (`schemas/generated/`) gains the
enum value.

**Model.** The value reads both spellings as one absent value, as a field without a policy does.
`defined(input.x)`, `sets:`, synthesis and every runtime are unchanged.

**Projection.** JSON Schema, OpenAPI and AsyncAPI share `ess-gen`'s `object_annotated`: the
property is the `Optional` projected whole, the `anyOf [T, null]` `nullable_field` already writes
for `null_when_absent`, and its key is not in `required`. That is the one difference from
`null_when_absent`.

**Conformance.** The value constrains no spelling, so a suite never carries it. Every place that
copies a field's policy from the IR into suite data (payload leaves, response and direct-response
observations, one-time responses, typed declarations, value-invariant observations, the
absent-first witness) reads `Presence::constrains`, which answers `None` for it. A payload with the
member left out and one with it `null` are both admitted, the suite bytes and suite major are those
of a field without a policy, and the Go and TypeScript runners, whose presence vocabulary is the
two older values, never meet it. Suite readers keep refusing `omitted_or_null` as
`InvalidPresence`.

## 2. A view field carries its source field's presence (#677)

A view field written field by field could not say `presence:`, so a `null_when_absent` entity field
was published by its view as optional and not nullable while the served row sends `null`.

The story offers two answers: the view field carries the stored field's policy, or accepts
`presence:` itself. #678's scope adds "View fields keep refusing the key". Carrying satisfies both
stories; a view field's own `presence:` would contradict the second and could disagree with the
field it projects. So the view field carries it, and `RawViewField` keeps refusing the key.

**Which fields.** From `ess/31`, in a view without `shape:` and without aggregation, a field that is
neither computed (`expr:`) nor read through a reference (`field: {related: …}`) projects its
source entity's field of the same name. Where that field declares `presence:` and the view field is
`Optional`, the compiler copies the policy onto the resolved view field (`resolve.rs`, `views`).
Every consumer of the view's fields reads it from there.

- A `shape:` view already carries its struct's presence.
- An aggregate view's group keys and a related read are not carried (§ 3).

**Format.** Carried only from `ess/31`: below it a view's resolved fields, IR bytes and published
schema do not move. A model that projects no field declaring a policy keeps its bytes at `ess/31`
too.

**Projection.** The view row schema follows the carried policy as every other message does.

**Conformance.** A view row's absent field under a carried `null_when_absent` keeps its `null`
claim (`unclaimed_absence`), and a value-invariant observation rooted at the view field carries the
policy. The interpreted target writes an absent view member as `null`, except under a carried
`omitted_when_absent`, where it leaves the member out, as the published row schema says.

## 3. Not done here

- The keep-versus-clear meaning of a missing member and `null` (the ADR above, target 0.70.0).
- `ess-diff` does not classify a changed presence policy; it falls to its residual, as before.
- An aggregate view's group key and a view field read through a reference do not carry the
  policy of the field they read.
- Generated Rust and Go view handlers are not changed. Neither generator writes
  `skip_serializing_if` or `omitempty`, so an absent value is presumably written as `null`; a
  carried `omitted_when_absent` would then be the gap those targets already have for any field that
  declares it. Not measured here.
