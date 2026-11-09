---
format: aep.planning-md/3
id: story:entity-runtime-component-closure
kind: story
status: draft
title: Entity Runtime lowering drops a command whose subject is outside the component without a diagnostic
tags:
- defect
- follow-up
- severity-medium
- zero-open-issues
relations:
- decomposes: epic:zero-open-issues-2026-11
revision: 3
---
## Outcome

Entity Runtime lowering refuses, by name and with the reason, a command the selected component
accepts whose subject entity is outside the component's entity closure. Today it drops the command
and reports nothing.

## Problem

`lower_command` (`crates/generate/ess-entity-runtime/src/lib.rs`, around lines 1973-1976) returns
`false` without a diagnostic when the command's one subject entity has no definition:

```rust
let entity_name = targets.into_iter().next().expect("one target");
if !self.definitions.contains_key(&entity_name) {
    return false;
}
```

Definitions exist only for the closure (`entity_closure`, around line 4089: the entities of the
domains the component owns, plus relation targets, plus owners). An entity inside the closure that
lacks a definition version already carries `MissingDefinitionVersion`, so the silent path is a
subject outside the closure. `walk_refused_command` then walks the branches but adds no
command-level diagnostic, and with no other diagnostic `lower` returns `Ok(LoweredService)` without
the command.

Minimal reproduction: a valid specification (ess 0.92.0, `ess specify validate` prints `valid`,
exit 0) whose component `desk` owns a domain with no entities and accepts a command updating an
entity of another domain:

```yaml
# system.yaml
format: ess/22
system: demo
version: v1
domains: [demo.items, demo.desk]
# domains/items.yaml
domain: demo.items
types:
  - {name: demo.items.ItemId, kind: newtype, of: String}
entities:
  - name: demo.items.Item
    identity: {name: item_id, type: demo.items.ItemId}
    fields:
      - {name: label, type: String}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
events:
  - name: demo.items.ItemAdded
    fields: [{name: item_id, type: demo.items.ItemId}]
actors:
  - name: demo.items.Clerk
    may: [demo.items.AddItem, demo.desk.Relabel]
commands:
  - name: demo.items.AddItem
    input:
      - {name: item_id, type: demo.items.ItemId}
      - {name: label, type: String}
    outcomes:
      - name: added
        creates: demo.items.Item
        instance: item_id
        sets: {label: input.label}
        emits: [demo.items.ItemAdded]
        payload: {demo.items.ItemAdded: {item_id: input.item_id}}
# domains/desk.yaml
domain: demo.desk
events:
  - name: demo.desk.Relabelled
    fields: [{name: item_id, type: demo.items.ItemId}]
commands:
  - name: demo.desk.Relabel
    input:
      - {name: item_id, type: demo.items.ItemId}
      - {name: label, type: String}
    outcomes:
      - name: relabelled
        updates: demo.items.Item
        instance: item_id
        sets: {label: input.label}
        emits: [demo.desk.Relabelled]
        payload: {demo.desk.Relabelled: {item_id: input.item_id}}
# components.yaml
components:
  - component: items
    owns: {domains: [demo.items]}
    accepts: {commands: [demo.items.AddItem]}
    publishes: {events: [demo.items.ItemAdded]}
  - component: desk
    owns: {domains: [demo.desk]}
    accepts: {commands: [demo.desk.Relabel]}
    publishes: {events: [demo.desk.Relabelled]}
```

`lower_component(&ir, &"desk".into(), &options)` lowers `desk` with no diagnostic and no
operation for `demo.desk.Relabel`. The CLI does not call Entity Runtime lowering, so the case is
reached through the library only.

## Acceptance

- In `crates/generate/ess-entity-runtime/tests/`, a test lowers component `desk` of the model above
  and gets `Err`, with one diagnostic at path `demo.desk.Relabel` whose code names the case (a new
  `LoweringCode`, for example `CommandSubjectOutsideClosure`) and whose message names
  `demo.items.Item` and says it is outside the component's entity closure.
- A red run first shows today's `Ok` with no `demo.desk.Relabel` operation.
- Lowering component `items` of the same model is unchanged; existing lowering tests keep their
  output.

## Scope

- cited: crates/generate/ess-entity-runtime/src/lib.rs
- inferred: crates/generate/ess-entity-runtime/tests/ (a new test file with the fixture)

confidence: high

## Fit review

1. Need: a valid model's accepted command disappears from the lowered service without a word.
   Reproduced above, brand-free.
2. Class: defect. `build_command` (around line 1843) states that a refused command carries a
   diagnostic and that one refusal must not hide another (beyond10x/ess#231); this path carries
   none.
3. Already expressible: not applicable.
4. Fit: one more `LoweringCode` beside `CommandSpansEntities` and `StatelessCommandUnsupported`,
   the sibling command-level refusals in the same function. No authored surface.
5. Second adopter: any system whose component accepts a command over an entity owned elsewhere.
6. Cost: one lowering code; a lowering that returned `Ok` silently incomplete now returns `Err`.
7. Considered: change nothing (silent drop); widen the closure to every subject of an accepted
   command (changes which definitions a component owns, a design decision of its own). Refusing by
   name wins.

Found while implementing beyond10x/ess#231 (2026-10-04); not an adopter request.

## Decisions

- Accept as a defect: refuse by name; do not widen the closure in this story.
