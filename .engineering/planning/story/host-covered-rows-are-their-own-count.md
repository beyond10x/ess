---
format: aep.planning-md/3
id: story:host-covered-rows-are-their-own-count
kind: story
status: implemented
title: Rows a host slot reaches are counted as host-covered, never as covered
summary: 'ADR section 2: a third coverage count in the text and JSON reports'
tags:
- backlog
- queue-0.91.0
- rank-8
relations:
- derived_from: design:ui-host-slots
- serves: vision:O2
- depends_on: story:a-ui-document-declares-a-host-slot
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: inferred
  path: crates/edge/ess-cli/tests/ui_commands.rs
- confidence: inferred
  path: crates/ui/ess-ui-check/src/hosts.rs
- confidence: cited
  path: crates/ui/ess-ui-check/src/lib.rs
- confidence: inferred
  path: crates/ui/ess-ui-check/src/model.rs
- confidence: inferred
  path: crates/ui/ess-ui-check/tests/checks.rs
- confidence: inferred
  path: website/docs/reference/ess-ui.md
- confidence: inferred
  path: website/docs/reference/formats.md
revision: 16
transitions:
- {from: "draft", to: "proposed", at: "2026-11-06T22:18:56Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-11-06T22:18:56Z", actor: "human:timo", revision: 3}
- {from: "active", to: "implemented", at: "2026-11-08T22:10:11Z", actor: "human:timo", revision: 16, decided_on: {"recorded":{"test_result":1}}}
---
## Need

The accepted ADR on host components, section 2: rows that a host slot reaches are their own count.
Today, a page with hand-written parts reports as fully covered by the specification. The design
is design:ui-host-slots, section Check.

## Acceptance

- A document with one slot that reads one view and sends one command reports those rows as
  host-covered.
- The covered count is the same as for the same document with the slot removed.
- The text report and the JSON report carry the third count and list each slot with its path,
  `host`, `reason` and `stands_in_for`.
- Every other coverage report over a UI document keeps host-covered rows out of its covered
  figure.

## Scope

Derived 2026-11-07 by `story-scoper`. Every line is **cited** (read from the story or the tree) or
**inferred** (a reading that could be wrong).

- **Primary surface:** `crates/ui/ess-ui-check` — cited (design:ui-host-slots, section Check).
- **Files:** `crates/ui/ess-ui-check/src/lib.rs` (`Sink`, `Report`, the `Json` envelope,
  `Report::render`) — cited. The third count and the slot list go into this report.
- **Files:** `crates/ui/ess-ui-check/src/hosts.rs` (`hosts::run` walks `document.host_slots()`) —
  inferred.
- **Also likely:** `crates/ui/ess-ui-check/src/model.rs` (with `--model`, the views, commands and
  fields the specification serves) — inferred.
- **Also likely:** `crates/ui/ess-ui/src/path.rs` and `crates/ui/ess-ui/src/model.rs` — inferred,
  read only.
- **Also likely:** `crates/edge/ess-cli/src/ui.rs` and `crates/edge/ess-cli/tests/ui_commands.rs`
  (JSON output pins) — inferred.
- **Tests:** `crates/ui/ess-ui-check/tests/checks.rs` and `adversary_u2_host_slots.rs` — inferred.
- **Documents:** `website/docs/reference/ess-ui.md` (host slot paragraph),
  `website/docs/reference/formats.md` (`ess-ui-check/1` row), `CHANGELOG.md` — inferred.
- **Confidence:** medium. The tree has no covered or uncovered count over a UI document yet, so
  this story probably builds the whole three-way count.
- **Would collide with:** story:a-host-slot-reads-and-commands-only-what-it-declares (same crate,
  `model.rs`).
- **Format:** `ess-ui-check/1` is serialize-only with no digest, and the 0.90.0 `backlog` was added
  as an optional key left out when empty. A third count added the same way leaves a document
  without slots byte for byte unchanged (unproven).
