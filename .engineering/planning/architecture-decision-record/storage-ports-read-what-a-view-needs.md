---
format: aep.planning-md/3
id: architecture-decision-record:storage-ports-read-what-a-view-needs
kind: architecture-decision-record
status: accepted
title: Storage ports read what a view needs, not whole rows
summary: Proposes per-view read methods on the generated storage port, staged after view order and paging; until then the port stays row-granular and says so (row 35).
relations:
- serves: vision:O2
- informed_by: epic:downstream-reported-gaps
revision: 2
transitions:
- {from: "proposed", to: "accepted", at: "2026-10-23T19:36:16Z", actor: "human:timo", revision: 2, decided_on: {"recorded":{"approval":1}}, correlation: "ess-0690"}
---
Status: proposed (draft). Not accepted; nothing under it is dispatched before acceptance.

## Context

Generated behaviour reaches stored data through one port per entity, `<Type>Storage`, with
`get(identity) -> Option<Snapshot>`, `put(snapshot)`, `delete(identity)` and, where a view lists the
entity, `list() -> Vec<Snapshot>` (`crates/generate/ess-synth/src/rust/behaviour.rs:388-420`). Every
read returns whole snapshots. The ephemeral implementation clones the full row on each `get` and the
full table on each `list` (`crates/generate/ess-synth/src/rust/store.rs:34-45`).

Adopter repro (beyond10x/a2a, tracker row 35, ESS 0.67.0): an A2A task carries its whole message
history and artifacts. `GetTask` with `historyLength: 1` and `ListTasks` copy every task in full
before trimming (a2a `ports.rs:88-97`). A long task makes every read cost its full size, and the
adopter's lock around storage makes one large read stall others (that lock is the adopter's own).

A durable implementation of the port (a database) can read part of a row, but the port gives it
nothing to read part of: no projection, no filter, no page.

## Options

| | what it is | cost |
|---|---|---|
| A | **Per-view read methods.** For each view, the port gains a method that returns only that view's published fields for the rows the view's filter, order and page select (`fn read_<view>(&self, query) -> Vec<<View>Row>`). The ephemeral store implements it over its rows. | The port contract changes: every hand-written storage implementation must add methods (a breaking change for implementors, versioned with the generated crate's major). Behaviour and query code calls the narrower method. Largest change; matches what a database can serve. |
| B | **Field selection on `get` and `list`.** `get` and `list` take a set of fields to fill; unselected fields come back empty (an `Option` per field or a partial snapshot type). | Smaller port change, still breaking; a partial snapshot type doubles the generated types; filters and pages stay in memory. |
| C | **Keep whole snapshots; document the port as row-granular.** | Nothing to build; reads stay proportional to row size; adopters with large rows wrap the port by hand. |

## Recommendation

A, staged after the view order, page and cursor work already filed (a2a's `Tasks` view gap), because
a per-view read needs the view's order and page to be decided first. Until then C, with the port's
row granularity stated in the generated rustdoc.

## Effect

- **Format:** none; `ess/N` already states views, filters and fields. The generated crate's port
  signature is the contract that changes.
- **Synthesis:** none; the conformance suite observes answers, not port calls.
- **Generated code:** Rust first (port trait, ephemeral store, query code), then Go and TypeScript;
  a compatibility note in `CHANGELOG.md` for implementors of the port.

## Acceptance evidence

None yet. Acceptance is the operator's move of this record to `accepted`.
