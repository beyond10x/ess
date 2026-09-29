---
format: aep.planning-md/3
id: story:diff-classifies-a-newtype-prefix-change
kind: story
status: active
title: verify diff leaves a newtype prefix change unclassified
refs:
- provider: github
  reference: beyond10x/ess#219
relations:
- serves: vision:O2
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-09-29T01:20:01Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-09-29T01:20:02Z", actor: "human:timo", revision: 3}
---
## Outcome

`ess verify diff` names a `prefix:` change on a newtype: `type/<T>/prefix-added` (narrowed), `type/<T>/prefix-removed` (widened), `type/<T>/prefix-changed` (both), in `ess-diff/10`, instead of `system/<name>/unclassified-changed` (beyond10x/ess#219).

## Acceptance

- each of the three changes is classified with its relation in text and JSON output;
- a delta carrying one is refused below `ess-diff/10` with `unsupported_format_version`;
- a model pair without a prefix change keeps its delta bytes and format.
