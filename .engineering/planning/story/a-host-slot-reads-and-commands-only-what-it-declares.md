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
revision: 6
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

- inferred: `crates/ui/ess-ui-check/src/model.rs`.
