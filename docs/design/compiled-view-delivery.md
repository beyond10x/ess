# Compiled view delivery

A compiled IR answers what a view is, not only that one exists. A consumer reading the IR must not
have to open the source YAML beside it to learn what a view returns — a second reader of the same
specification is the thing compiling to an IR exists to prevent.

## What the IR carries

A domain lists its views by qualified name. Each name resolves to a complete declaration under the
top-level `views` key, and that declaration carries:

| field | meaning |
|---|---|
| `name` | the qualified identity, the same string the domain lists |
| `source` | the entity the view reads, itself declared under `entities` |
| `naming.wire` | the name it is answered under, when a document declares one |
| `consistency` | the guarantee the view makes, such as `read_your_writes` |
| `fields` | every field, each with its `type_ref` |

A domain's `views` list is therefore an index, never the declaration itself. Commands already carry
all of theirs; views carry theirs the same way, so a reader resolves both alike.

## Delivered, not only serialized

The declarations existed in the serialized IR before this construct; what nothing established is
that they reach an adopter through the command an adopter actually runs. `ess specify compile`
delivers the same bytes to stdout and to `--out`, and the executable witness checks a view's wire
name, source, consistency and fields in that output rather than in a library value that no CLI path
returns. A serializer that is correct in-process and a CLI that drops the field are the same defect
to a consumer, and only the second is observable from outside.

## Compatibility

No format version changes: the fields are the ones the resolved view already declared, under the
key it already used. A document that declares no `naming.wire` keeps the qualified name, and a view
with no declared consistency serializes as it did.
