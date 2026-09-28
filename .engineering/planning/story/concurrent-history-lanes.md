---
format: aep.planning-md/3
id: story:concurrent-history-lanes
kind: story
status: draft
title: A failing history is drawn as client lanes
owner: ess
relations:
- depends_on: story:linearizability-checker-over-the-interpreter
- serves: vision:O2
- decomposes: epic:concurrent-history-conformance
- depends_on: story:concurrent-explorer-runner
revision: 1
---
# Story: a failing history is drawn as client lanes

## Outcome

`ess verify conform web` renders an `ess-history/1` document and its verdict as one lane per client:
each operation's invoke–return interval, the linearization points found, and for a violation the
operation where the search failed. The page is rendered by `ess` itself, with no script or stylesheet fetched from outside.

## Acceptance

- The page for the shrunk `LostUpdate` history names the two conflicting operations and the state
  each linearization would have required.
- Rendering the same history twice gives identical bytes.

## Scope

`src/web.rs` and the web assets in `ess-gen`.
