---
format: aep.planning-md/3
id: story:a-system-declares-which-of-its-published-events-are-outward
kind: story
status: proposed
title: A system declares which of its published events are outward
summary: 'outward: on a component''s publishes, a contract diff reads; F1 A decided'
tags:
- design
- event-bus
- issues-2026-11-26
relations:
- serves: vision:O2
- decomposes: epic:outward-events-and-an-event-bus-contract
- informed_by: story:a-component-that-publishes-nothing-keeps-the-events-its-commands-emit-unpublished
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-26T16:09:52Z", actor: "human:operator", revision: 3}
---
## Need

A component lists the events it publishes (`crates/specify/ess-domain/src/component.rs:135-137`),
and nothing says which of them the system offers to other systems. A reader the system does not
know cannot rely on an event's name or payload, and `ess verify diff` cannot call a payload change
breaking for one. Design: `docs/design/outward-events-and-event-bus.md` § 1 (branch
`design/outward-events-and-event-bus`).

## Outcome

An outward event is a published event the system offers outward. Under F1 A a component's
`publishes:` gains `outward:`, a subset of its `events:`, at the next `ess/N`:

```yaml
publishes:
  events: [gateway.calls.CallFinished, gateway.internal.Retried]
  outward: [gateway.calls.CallFinished]
```

## Acceptance

- At the new source format `ess specify validate` exits 0 on a component with `outward:`, and
  refuses an outward event the component does not publish (`undeclared_reference`) and one listed
  twice; below the format `outward:` is refused as `unsupported_format_version`.
- The compiled IR carries the outward set per component in a deterministic order; a model without
  `outward:` keeps its IR bytes.
- `ess verify diff` reports an outward event removed, or its payload changed incompatibly, as
  breaking, and making an event outward as additive.
- AsyncAPI marks an outward event's message; without `outward:` its bytes are unchanged.
- `website/docs/reference` lists the key. Each test runs within 60 s, each test step within 120 s.

## Scope

- `crates/specify/ess-domain/src/component.rs`, `crates/specify/ess-compiler/src/ir.rs`,
  `crates/verify/ess-diff/src/change.rs`, `crates/generate/ess-gen/src/asyncapi.rs`,
  `website/docs/reference/`.

## Decisions

F1 = A: a component's `publishes:` gains `outward:`, in the specification, at the next `ess/N`. The
owner, 2026-11-26, relayed by the operator session. Rejected: B, nowhere in the model (no contract
boundary for `diff`); C, a sibling `ess-outward/1` (one more document per system for one list).
