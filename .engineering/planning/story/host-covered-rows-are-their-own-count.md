---
format: aep.planning-md/3
id: story:host-covered-rows-are-their-own-count
kind: story
status: active
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
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-11-06T22:18:56Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-11-06T22:18:56Z", actor: "human:timo", revision: 3}
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

- inferred: `crates/ui/ess-ui-check/src/` (coverage, report), the CLI's `ess ui check` JSON
  output.
