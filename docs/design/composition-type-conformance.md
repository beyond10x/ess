# A consumer names an owner's types, and proves its own copy matches

beyond10x/ess#162. Composition format `ess-composition/2`; the source format (`ess/N`) and the
suite formats do not move.

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

## Compatibility

- `SUPPORTED_COMPOSITION_FORMATS` is `[1, 2]`; `cargo xtask docs` reads it, so a new major
  without a release row and a reference page is refused.
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
