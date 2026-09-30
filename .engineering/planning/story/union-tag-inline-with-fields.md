---
format: aep.planning-md/3
id: story:union-tag-inline-with-fields
kind: story
status: draft
title: A union can carry its tag beside its variant's fields
relations:
- serves: vision:O2
- informed_by: epic:ess-ui-renderer-neutral-ui
- depends_on: story:acceptance-runs-as-toolchain-scenarios
revision: 1
---
## Outcome

A union type can carry its tag and its variant's fields side by side in one mapping
(`{component: collection, columns: [...], row_actions: [...]}`), so a document whose objects are
discriminated by one field generates typed readers that still refuse an unknown field.

## Acceptance

- A union declares `tag_inline: <field>` (or the chosen spelling); each variant is a struct whose
  fields sit next to the tag; the Rust target emits `#[serde(tag = "<field>")]` with each variant
  keeping `deny_unknown_fields`; the TypeScript target emits a discriminated union.
- Validation refuses a variant field named like the tag, and two variants with one name.
- A new specification format version; older versions refuse the key by name; specifications without
  it keep their bytes.

## Origin

A UI document format discriminates its 14 composite kinds by `component` with the kind's properties
inline; generated readers today either refuse every property (`deny_unknown_fields`) or would need an
untyped remainder.
