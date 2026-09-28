# A consumer names an owner's types, and proves its own copy matches

beyond10x/ess#162. Composition format `ess-composition/2`; reader assertions (#191) are
`ess-composition/3`. The source format (`ess/N`) and the suite formats do not move.

## The gap

A consumer imports an owner's model through `composition.yaml` and references its commands and
views. The owner's response shape is a type no command, event, error or view reaches, so a
reference to it was refused as `reference_outside_component`. The consumer kept a local struct with
the same fields, and a hand-written script checked that the two agreed field by field.

## The construct

```yaml
format: ess-composition/2
composition: devcenter
services:
  - {key: owner, system: demo.owner, version: v1, source_digest: <sha256>, component: status-component}
  - {key: consumer, system: demo.consumer, version: v1, source_digest: <sha256>, component: dashboard-component}
references:
  - service: owner
    semantic: {kind: type, name: demo.owner.agentstatus.StatusView}
conformances:
  - local: {service: consumer, type: demo.consumer.dashboard.AgentStatus}
    conforms_to: {service: owner, type: demo.owner.agentstatus.StatusView}
```

Two things change under `/2`:

1. **Owner-declared types are referenceable.** A `type` reference, and either end of a conformance,
   may name any type declared in a domain the selected component owns. Under `/1` the rule is
   unchanged — only types reached from the client surface — and the refusal now says that `/2`
   admits the type when it would.
2. **`conformances`** asserts that a local type has an imported type's shape. Both ends are admitted
   exactly like references (`unknown_reference_service`, `unresolved_semantic_reference`,
   `reference_outside_component`), then compared.

## The comparison

The two ends live in different compiled models and share no handle, so the comparison is
structural. Two named types conform when they are the same kind and:

| kind | rule |
|---|---|
| struct | same field names; each field has the same wire name and a conforming type; where both ends are `Optional`, the same `presence` |
| newtype | the wrapped representations conform |
| enum | same variant names, each with the same wire spelling |
| union | same tag, same variant names, each variant's shape conforming |

Primitives, lists and map keys must match exactly; named types are compared recursively, and a pair
already on the current path is taken as conforming so a recursive shape terminates; a pair reached
through two different fields is compared at each, and every drifting path is reported.

**The one tolerance:** where the imported value is required, the local one may be `Optional`, at
any position. A reader that treats a required value as optional loses nothing. The reverse — a
local value required where the imported one may be absent — is drift, because the consumer would
reject a value the owner may send. Field sets must be equal in both directions: a field the owner
declares and the consumer does not is drift, as is the opposite, since the assertion is that the
two copies agree, which is what the hand-written check verified.

Not compared: newtype `alphabet`, `prefix` and invariants, struct invariants, display names and
summaries. They constrain values, not the shape a reader decodes. This is a known limit: a local
newtype with a stricter `prefix` or `alphabet` than the owner's still conforms, although it can
reject a value the owner sends.

A difference is refused as `type_conformance_drift` on the local service. The detail names both
types and every differing field path, for example:

```text
[type_conformance_drift] `consumer` type `demo.consumer.dashboard.RenamedStatus` does not conform
to `owner` type `demo.owner.agentstatus.StatusView`: field `since`: `demo.owner.agentstatus.StatusView`
declares it and `demo.consumer.dashboard.RenamedStatus` does not; field `updated_at`: …
```

## Reader assertions (`ess-composition/3`)

beyond10x/ess#191. A consumer that mirrors an owner type only to read it off the wire differs from
it in ways the rule above refuses and a reader can afford. In one consumer 14 of 15 mirrored types
were refused for these differences alone. Under `ess-composition/3` a conformance entry may carry
`reader: true`:

```yaml
format: ess-composition/3
conformances:
  - local: {service: consumer, type: demo.consumer.dashboard.StatusSummary}
    conforms_to: {service: owner, type: demo.owner.agentstatus.StatusView}
    reader: true
```

The test for every widening is one question: can the local type reject a value the imported type
allows? If it cannot, the difference is admitted. If it can, it stays `type_conformance_drift`.

| widening | admitted | still drift |
|---|---|---|
| newtype as its representation | an imported newtype, through any chain of newtypes, read as what the chain wraps (`CallRef` → `CallId` → `String` read as `String`) | a different primitive (`Uuid` for a newtype of `String`) |
| enum as `String` | a required imported enum read as `String` or `Optional<String>`; an optional one as `Optional<String>` | an optional imported enum read as a required `String` |
| variants by wire name | local wire names include every imported wire name; variant names and extra local variants do not matter | an imported wire name the local enum lacks |
| object as a map | a value that is always a JSON object — a struct, or a map with `String` keys, directly or through newtypes — read as `Map<String, Json>` | `Json` read as any map, bare or through a newtype, `Optional`, `List` or `Map`: it may be an array or a scalar; a map whose value type is not `Json` |
| field subset | a local struct that omits imported fields; an extra local field that shares its wire name with an imported field (omitted or renamed by `wire`) and reads it as a reader; an extra local field no imported field sends, when it is `Optional` and not `null_when_absent` | a local field required where the imported one is optional or absent; an extra local field that reads an imported wire key as another type; a `null_when_absent` local field the imported struct never sends |

A reader decodes by wire name, so an extra local field is matched by wire name against every
imported field, including the ones it omits, and compared with the reader rules when the keys meet.
This holds inside union variant payloads too. Field wire names, the `presence` of fields that are
optional at both ends, map keys, lists, unions and every rule not in the table are compared as
without `reader`. Without `reader: true` nothing changes, under `/3` as under `/2`. The known limit
above carries over: a local newtype's `alphabet` and `prefix` are still not compared.

**Precondition.** A field subset holds only for a tolerant reader. `reader: true` also asserts that
the consumer's reader ignores keys it does not declare; ESS-generated closed types
(`additionalProperties: false`, `deny_unknown_fields`) do not, so a consumer reading through them
must not use `reader` for a field subset. The comparison cannot see this; the author asserts it.

**Departure from the issue.** #191 gives reading a JSON value as `Map<String, String>` as an
example. `Json` read as any map is refused here, `Map<String, Json>` included, because a JSON value
may be an array or a scalar; the issue's own rule — anything that could reject a producer value
stays drift — decides it. Only a value that is always an object is read as `Map<String, Json>`.

## Compatibility

- `SUPPORTED_COMPOSITION_FORMATS` is `[1, 2, 3]`; `cargo xtask docs` reads it, so a new major
  without a release row and a reference page is refused.
- A `/1` or `/2` document whose conformance entry carries `reader`, whatever its value (`false` and
  `null` included), is refused as `unsupported_format`, naming `/3`. `/3` keeps the key as written; an entry without it is
  serialised without it, so a `/2` entry's authored and compiled bytes are unchanged.
- A `/1` document carrying `conformances`, even `[]`, is refused as `unsupported_format`. `/1`
  documents compile exactly as before.
- The compiled composition echoes the authored marker. A `/1` composition's compiled bytes do not
  change: the owner-declared type set is not serialised, and `conformances` is omitted when empty.
- The client plan (`ess-client-plan/1`) does not change for any composition: owner-declared types
  are referenceable, not added to a service's `types` closure, so generated clients and their
  `TYPES` lists are identical for the same services and references.
- `ess specify compose` passes the key through the same closed reader; the CLI needs no change.
- The Entity Runtime, generated targets and the Go/TypeScript conformance runtimes never read a
  composition; nothing there lowers or refuses this construct.
