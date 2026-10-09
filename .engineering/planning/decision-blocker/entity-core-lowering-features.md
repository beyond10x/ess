---
format: aep.planning-md/3
id: decision-blocker:entity-core-lowering-features
kind: decision-blocker
status: cleared
title: Entity-core has no feature for increment, cleared and Optional-to-Optional updates
relations:
- blocks: story:entity-runtime-lowers-entity-core-constructs
revision: 5
transitions:
- {from: "open", to: "cleared", at: "2026-10-08T09:55:14Z", actor: "human:timo", revision: 4}
---
# Entity-core has no feature for increment, alphabet, text count, cleared and Optional-to-Optional updates

## Narrowed (2026-10-07)

Entity Runtime 0.27.0 ships `<text>.count` on a declared `string` (entity-runtime #56) and a String
`alphabet` under `service/1` (#57). Those two are lowered by
`story:entity-runtime-lowers-alphabet-and-text-count`. This blocker now holds the three entity-core
has not shipped: `{increment: n}` (planned upstream), `{cleared: true}`, and an
Optional-to-Optional `updates:`.

## Decided

Cleared 2026-10-08: Entity Runtime 0.30.0 ships increment, cleared and `set_if_present`, the three features this blocker held (entity-runtime issue 54 closed 2026-10-07). Lowering them in ESS is ordinary story work against that release.
