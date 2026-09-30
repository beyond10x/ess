---
format: aep.planning-md/3
id: story:ui-spec-checks
kind: story
status: implemented
title: An ess-ui document is checked with findings named by stable node path
relations:
- serves: vision:O2
- decomposes: epic:ess-ui-renderer-neutral-ui
- depends_on: story:ui-spec-schema
scope:
- confidence: inferred
  path: crates/ui/ess-ui-check
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-09-30T00:35:23Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-09-30T03:15:27Z", actor: "human:timo", revision: 4}
- {from: "active", to: "implemented", at: "2026-09-30T05:07:16Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":1,"review_outcome":2,"verification":1}}}
---
## Outcome

An `ess-ui/1` document is checked, and every finding names the failing node by its canonical path
with a severity, so an editor can reject a change that breaks the document.

## Acceptance

- `ess ui check --path <document> [--model <ess spec>]` runs every check the schema declares:
  references resolve (pages in navigation, opened overlays, channels, widgets, fixtures), children
  are allowed by the layer rules, names are unique among siblings, `degrades` covers every
  capability a declared renderer lacks, placeholders are reported as warnings, and — with
  `--model` — every read, does and live event exists in the ESS model and every section is readable
  by some actor.
- Output is human text or JSON (`ess-ui-check/1`) listing path, check id, severity and message; exit
  1 when any error-severity finding exists.
- A node's path does not change when a sibling is added (a test adds a sibling and compares paths).
- Each check has a test with a document that trips it.
