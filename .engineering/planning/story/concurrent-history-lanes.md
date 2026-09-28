---
format: aep.planning-md/2
id: story:concurrent-history-lanes
kind: story
status: active
title: A failing history is drawn as client lanes
owner: ess
relations:
- depends_on: story:linearizability-checker-over-the-interpreter
- serves: vision:O2
- decomposes: epic:concurrent-history-conformance
- depends_on: story:concurrent-explorer-runner
scope:
- confidence: cited
  path: crates/edge/ess-cli/src/main.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/conform_web_history.rs
- confidence: cited
  path: crates/verify/ess-conformance
- confidence: inferred
  path: crates/verify/ess-conformance/assets
- confidence: cited
  path: crates/verify/ess-conformance/src/lanes.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/lib.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/linearize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/web.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/lanes.rs
- confidence: cited
  path: website/docs/guides/verify-conformance.md
revision: 7
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

Derived 2026-09-27 by `aep:story-scoper` against `472d35fbe`. Every line is **cited** or **inferred**. Replaces the earlier line that placed the web assets in `ess-gen`.

- **Primary surface:** `crates/verify/ess-conformance` — cited (`src/web.rs`, the scenario player)
- **Files:** `crates/verify/ess-conformance/src/web.rs` — cited
- **Files:** `crates/verify/ess-conformance/assets/` — inferred; the player's embedded assets live here, not in `ess-gen` (`ess-gen/assets` holds only `default.css`, `mermaid.min.js`)
- **Files:** `crates/edge/ess-cli/src/main.rs` (`ConformCommand::Web` `:515`, `conform_web` `:3467`) — inferred, a history input argument
- **Also likely:** `crates/verify/ess-conformance/src/lib.rs` — inferred
- **Also likely:** a new ess-cli integration test for the `LostUpdate` page — inferred
- **Confidence:** medium
- **Would collide with:** any unit touching `src/web.rs`, `src/web_replay.rs`, `assets/`, or `ConformCommand` in `ess-cli/src/main.rs`
