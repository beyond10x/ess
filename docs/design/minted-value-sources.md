# Minted value sources: a present mint, and a stored value read from the payload

Design for [issue 545](https://github.com/beyond10x/ess/issues/545), 2026-10-13, planned as
`story:minted-value-present-and-shared-with-its-payload`. Status: implemented in source format
`ess/25`; both shapes keep their earlier nested-mapping reading below it.

## The need

An implementation mints a value, a sequence number for example, publishes it in an event and
stores it on the row in the same outcome. The stored field is `Optional`, because the row holds no
value before that outcome, and an invariant requires the value from a later state on:

```yaml
fields:
  - {name: connect_seq, type: Optional<ConnectSeq>}
invariants:
  - any: [{not: "state == Bridged"}, "defined(connect_seq)"]
```

Up to `ess/24` the model can say neither half:

1. `{generated: true}` into an `Optional` field lets the implementation choose absence. The
   interpreted target chooses it (absence claims least), so the model's own next transition into
   `Bridged` breaks the invariant: `ESS-CF-TARGET … violating its invariant`. Generated Rust and
   Go ask an optional generator port (`try_generate_optional_<t>`), which may answer `None`.
2. The payload and `sets:` entries are two independent mints, so nothing says the stored value is
   the published one. `sets:` refuses `{response: …}`, and there was no payload source.

The workaround, making the value a command input, makes a scenario author choose a value the
implementation chooses (the problem beyond10x/ess#544 removes from authored scenarios).

## Constructs

### `{generated: true, present: true}`

The implementation mints the value, and the value is present. Written exactly as these two keys.

| Rule | Refusal |
|---|---|
| admitted wherever `{generated: true}` is: a `payload:` field, a `sets:` entry, a nested struct leaf, an error payload field | — |
| the target is `Optional<…>` | `type_mismatch` on a required target, which is present already: write `{generated: true}` |
| `present` is `true` | `type_mismatch` on `present: false`, which says nothing `{generated: true}` does not |
| not after `else:` | the fallback of `{input: …, else: …}` already fills a present value |
| from `ess/25` | below it, the reading the shape always had (below) |

### `{payload: <Event>.<field>}`

A `sets:` entry stores the value this outcome publishes in `<field>` of `<Event>`: one value feeds
both the event and the row. `<Event>` is the qualified event name and `<field>` the last segment.

| Rule | Refusal |
|---|---|
| a `sets:` entry, or a nested struct leaf of one | `unsupported_payload_source` in a `payload:` field, an error payload or after `else:`: a payload field reads another field of the same event only through the same source |
| `<Event>` is one this outcome emits **and** fills under `payload:` | `unresolved_reference`, naming the outcome's `payload:` events |
| `<field>` is a declared top-level field of that event, given a source there | `unresolved_reference` |
| the event field's type is assignable to the stored field's, as for `input.<field>`; `T` into `Optional<T>` is present | `type_mismatch` |
| from `ess/25` | below it, the reading the shape always had (below) |

The payload field's own source may be anything a payload admits; reading a mint is the need, and
reading `input.<field>` through the payload says the same as reading the input. Where the payload
leaves the value absent (an `Optional` event field), the stored field is absent; a required stored
field over an `Optional` event field is refused by the type rule.

### Below `ess/25`

Neither shape is a source before `ess/25`, because neither is all source keywords: each is read as
the nested mapping it always was, so a struct with fields named `generated` and `present`, or one
named `payload`, keeps its meaning (the `{caller: …}` precedent, `ess/16`). Where that nested
reading is refused because the target is not a struct, the refusal's hint names `ess/25`.

## Lowering

`ess-domain` gains `PayloadSource::Generated { present }` and `PayloadSource::PayloadField { event,
field }`; `ess-compiler` lowers them to `ResolvedPayloadValue::Generated { present }` and
`ResolvedPayloadValue::PayloadField { event, field, type_ref }`. `present` is serialized only when
`true`, so every model that does not use the new shapes keeps its IR bytes.

## Execution

| Reader | Present mint | Payload read |
|---|---|---|
| interpreter (`ess-conformance`) | mints the required type and stores it present; a given or recorded slot that is absent yields no step | the value the emitted event carries in that field: the payload value is resolved once per outcome and shared, whether minted, given or recorded |
| synthesis | as `{generated: true}`: no value claimed; where a view shows the stored field, the row is required not to hold it null | the scenario of the outcome requires the row to hold the published value (`ScenarioValue::Observed` in the view row), where no arranging step emits the same event first |
| generated Rust and Go | the required generator port (`generate_<t>`), wrapped present | the command stays an obligation, named by the plan: generated code would have to order the payload before the row |
| Entity Runtime, TypeScript, Web | as `{generated: true}` where they accept it | the reading they give a source they do not support |

The synthesis claim is bounded deliberately. The runner resolves `Observed` against the first
occurrence of the event in the scenario, so a scenario whose arrangement already emitted it would
compare the row with another row's value; there no claim is made.

## Not done

- Generated Rust and Go do not yet store a payload read; doing so is a labelled later step (render
  the payload value into a local before the row and the event).
- `{payload: …}` reads a top-level event field, not a path into a struct.
- No suite format moves: `Observed` is already in the suite vocabulary and the runner resolves it in
  view rows.
