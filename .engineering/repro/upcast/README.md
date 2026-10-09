# Reproducer: a recorded command input in an older shape

For `story:an-older-command-input-is-read-on-replay-through-a-declared-upcast`. Recorded with
`ess 0.56.0`, run from this directory.

| file | what it is |
|---|---|
| `before.yaml` | revision v1: `catalog.items.Accept` takes `receipt: String` |
| `after.yaml` | revision v2 as requested: `receipt: catalog.items.Receipt {revision, evidence}`, plus a `stale-revision` refusal |
| `after-additive.yaml` | revision v2 written with the existing idiom: `receipt` stays `String`, `receipt_revision: Optional<Integer>` is added |
| `scenarios/recorded-v1.scenario.yaml` | the recorded invocation, an `ess-scenario/1` document written against v1 (`receipt` is text) |
| `scenarios/additive-stale-revision.scenario.yaml` | a v2 invocation under the idiom that names a stale revision |

## The failure

The recorded invocation runs against v1:

```console
$ ess verify conform run --target interpreted --path before.yaml --scenarios scenarios/recorded-v1.scenario.yaml --report-format 2
  passed catalog.items/authored/an-item-accepted-with-a-text-receipt
  5 scenarios: 5 passed, 0 failed, 0 error, 0 unsupported
```

The same invocation against v2 is refused before anything runs:

```console
$ ess verify conform run --target interpreted --path after.yaml --scenarios scenarios/recorded-v1.scenario.yaml --report-format 2
refusal[ESS-AUTHOR-015]: `catalog.items/authored/an-item-accepted-with-a-text-receipt` in recorded-v1.scenario.yaml
  the input of `catalog.items.Accept`: receipt: expected catalog.items.Receipt as a mapping, found a string
  help: write a value of the type the model declares there
exit 1
```

The adopter's replay is not an `ess` verb. It posts recorded bodies to the server that
`ess generate synthesize --layout crate` generates. For v2 that server's decoder reads `receipt`
as an object:

```console
$ ess generate synthesize --path after.yaml --layout crate --out <scratch>/gen-after
14 capabilities: 14 generated, 0 obligation(s), 0 refused
$ grep -n -A12 'pub fn decode_command_catalog_items_accept' <scratch>/gen-after/src/server/wire.rs
124:        receipt: {
125-            let at1 = json::nested(at, "receipt");
126-            let member1 = json::member_at(value, at, "receipt")?;
127-            decode_catalog_items_receipt(member1, &at1)?
```

`json::member_at` on a string answers `body.receipt: expected an object, found a string` with
status 400, the message the adopter's replay printed. The generated crate was not built here.

## What `ess verify diff` says about it

```console
$ ess verify diff --from before.yaml --to after.yaml --compatibility
  changes  command catalog.items.Accept: input `receipt` is `catalog.items.Receipt`, was `String`
           command/catalog.items.Accept/input-type-changed/receipt
           unknown for callers, readers
$ ess verify diff --from before.yaml --to after.yaml --fail-on breaking
gate --fail-on breaking: passed (0 acknowledged)
$ ess verify diff --from before.yaml --to after.yaml --fail-on breaking-or-unknown --dimension history
fails --fail-on breaking-or-unknown: system/catalog/version-changed
```

In JSON the input change reads `"history": "compatible"`. The history gate fails only on the
version change.

## The existing idiom

Keep the recorded field's type and add the new fact as an `Optional` input. The refusal reads it
only when it is present:

```console
$ ess specify validate --path after-additive.yaml
catalog v2 — 1 file(s), valid
$ ess verify conform run --target interpreted --path after-additive.yaml --scenarios scenarios --report-format 2
catalog v2 against interpreted 0.56.0 — passed
  passed catalog.items/authored/a-receipt-for-another-revision-is-refused
  passed catalog.items/authored/an-item-accepted-with-a-text-receipt
  4 scenarios: 4 passed, 0 failed, 0 error, 0 unsupported
$ ess generate synthesize --path after-additive.yaml --layout crate --out <scratch>/gen-add
13 capabilities: 13 generated, 0 obligation(s), 0 refused
```

The generated decoder reads `receipt` as text, as v1 did, and reads `receipt_revision` as
`None` when the member is absent or `null`.

## The conversion design

Declaring `conversions: [{from: catalog.items.Receipt, to: String, because: …}]` and storing
`input.receipt` whole validates but does not generate:

```console
$ ess generate synthesize --path <scratch>/after-conv.yaml --layout crate --out <scratch>/gen-conv
15 capabilities: 13 generated, 2 obligation(s), 0 refused
```

The two obligations are the `Accept` behaviour ("kept an obligation by a declared conversion into
`receipt`") and the conversion `catalog.items.Receipt -> String` ("the contract is declared; the
algorithm is not"). A conversion crosses a value on a read inside the model. It does not change
how a request body is decoded, so it cannot read an older input.
