---
format: aep.planning-md/3
id: story:field-names-underscore-and-newtype-map-keys
kind: story
status: implemented
title: A field name may start with an underscore, and a map key may be a newtype
relations:
- decomposes: epic:retrofit-findings-20260927
- serves: vision:O2
- supersedes: story:field-names-wire-and-value-types
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-09-27T07:54:30Z", actor: "human:timo", revision: 2, imported: true}
- {from: "proposed", to: "active", at: "2026-09-27T07:55:02Z", actor: "human:timo", revision: 3, imported: true}
- {from: "active", to: "implemented", at: "2026-09-27T14:55:17Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":1}}, imported: true}
---
## Scope

- #141: a field name may begin with `_`.
- #143: a map key may be a newtype of an admitted key primitive (`Map<ItemId, Boolean>`).

Split from `story:field-names-wire-and-value-types` (archived); its Derived scope applies:
`crates/specify/ess-domain/src/types.rs` `field_name` :548-570, `Field::PATTERN` :329/:314, the
map-key refusal :194 (cited).

## Acceptance

- `{name: _url, type: Optional<String>}` validates; generated Rust, Go and TypeScript code compiles
  for it.
- `Map<demo.orders.ItemId, Boolean>` with `ItemId` a newtype of `String` validates, projects to the
  same JSON Schema as `Map<String, Boolean>`, and synthesizes a suite.
