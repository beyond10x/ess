---
format: aep.planning-md/3
id: story:a-host-slot-reads-and-commands-only-what-it-declares
kind: story
status: active
title: A host slot reads and commands only what it declares, checked against the binding
summary: 'ADR section 3, check side: ess ui check --model checks a slot as it checks a generated node'
tags:
- backlog
- queue-0.91.0
- rank-9
relations:
- derived_from: design:ui-host-slots
- serves: vision:O2
- depends_on: story:a-ui-document-declares-a-host-slot
scope:
- confidence: cited
  path: crates/ui/ess-ui-check/src/model.rs
- confidence: inferred
  path: crates/ui/ess-ui-check/tests/binding.rs
- confidence: inferred
  path: crates/ui/ess-ui-check/tests/host_slots.rs
- confidence: inferred
  path: website/docs/reference/ess-ui.md
revision: 11
transitions:
- {from: "draft", to: "proposed", at: "2026-11-06T22:18:57Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-11-06T22:18:58Z", actor: "human:timo", revision: 3}
---
## Need

The accepted ADR on host components, section 3, check side: a host component gets what a
generated node at its position gets, and nothing more. The design is design:ui-host-slots,
section Check.

## Acceptance

- With `--model`, `ess ui check` checks a slot's `reads` and `commands` against the specification
  and its `ess-http/1` binding, as it checks a generated node's.
- An unknown view, an unknown command, or an operation the binding does not serve is refused by
  the same diagnostics.
- A slot with no `reads` and no `commands` is accepted and reaches no row.

## Scope

Derived 2026-11-07 by `story-scoper`. Every line is **cited** (read from the story or the tree) or
**inferred** (a reading that could be wrong).

- **Primary surface:** `crates/ui/ess-ui-check` — cited: the story's scope line names
  `crates/ui/ess-ui-check/src/model.rs`, and design:ui-host-slots, section Check, names `--model`
  and the `ess-http/1` binding.
- **Files:** `crates/ui/ess-ui-check/src/model.rs` — cited. The change lands in two places in that
  file:
  - `fn names` collects every view and command that a document node names. Both `Model::check` and
    `binding_over` read it, so one change there covers the specification check and the binding
    check.
  - `fn body_names` returns early on anything that is not `Body::Composite`, so `Body::Host` slots
    are skipped today. The page and field arms of `names` never read `page.host` or `field.host`.
- **Symbols:** `HostSlot { reads, commands }` and `Body::host()` in `crates/ui/ess-ui/src/model.rs`
  — cited. Both are read, not changed.
- **Also likely:** `Document::host_slots()` in `crates/ui/ess-ui/src/path.rs` — inferred. It is
  read, not changed.
- **Also likely:** `reading_section`, and the `body: true` flag on `Kind::View`, in
  `ess-ui-check/src/model.rs` — inferred.
- **Also likely:** tests in `crates/ui/ess-ui-check/tests/host_slots.rs` and `binding.rs` —
  inferred.
- **Documents:** the host-slot paragraph in `website/docs/reference/ess-ui.md` — inferred.
- **Confidence:** high.
- **Would collide with:** any unit editing `crates/ui/ess-ui-check/src/model.rs`.
