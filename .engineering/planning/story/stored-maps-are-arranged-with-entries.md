---
format: aep.planning-md/3
id: story:stored-maps-are-arranged-with-entries
kind: story
status: implemented
title: Stored Map fields are arranged only as {}
refs:
- provider: github
  reference: beyond10x/ess#240
relations:
- serves: vision:O2
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-09-29T05:37:20Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-09-29T05:37:21Z", actor: "human:timo", revision: 3}
- {from: "active", to: "implemented", at: "2026-09-29T09:58:25Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":1,"review_outcome":1,"verification":1}}}
---
## Outcome

A stored `Map` field a guard reads by its entries is arranged non-empty where the guard needs it, with keys drawn from the `input.` fields the predicate compares them with or from `example:` values, so both sides of `exists: {in: <map>, as: r, that: …}` are witnessed; the reference states what `r` binds to for a map (beyond10x/ess#240).

## Acceptance

- the #240 shape synthesizes both the refusal and the accepting default branch;
- a target that ignores the map entries fails;
- `website/docs/reference/predicates.md` states whether `r` binds to keys, values or entries, matching the interpreter and Entity Runtime;
- committed suites regenerate byte-identical or the change is listed in CHANGELOG.
