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
scope:
- confidence: inferred
  path: crates/edge/ess-xtask/src/consumer_coverage/entry-classifications.json
- confidence: cited
  path: crates/generate/ess-entity-runtime/src/lib.rs
- confidence: cited
  path: crates/generate/ess-entity-runtime/src/subset.rs
- confidence: inferred
  path: crates/generate/ess-entity-runtime/tests/component_closure.rs
- confidence: inferred
  path: website/docs/reference/entity-runtime-lowering.md
revision: 9
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

- `cargo test -p ess-entity-runtime --locked --test component_closure`: lowering component `desk`
  of the model above returns `Err` with exactly one diagnostic, at path `demo.desk.Relabel`, code
  `LoweringCode::CommandSubjectOutsideClosure`, whose message names `demo.items.Item` and says it
  is outside the component's entity closure. On today's tree the same test fails, because lowering
  returns `Ok` with no `demo.desk.Relabel` operation.
- Same test target: lowering component `items` of the same model returns `Ok` with the
  `demo.items.AddItem` operation and no diagnostic.
- `cargo test -p ess-entity-runtime --locked --test lowerable_subset` passes with
  `CommandSubjectOutsideClosure` in `LoweringCode::ALL`, a refused row naming the new test, and
  `website/docs/reference/entity-runtime-lowering.md` re-rendered to that row.
- `cargo test -p ess-entity-runtime --locked --test lowering` passes with no expected output
  changed.

## Scope

Moves no shared surface.

- cited: crates/generate/ess-entity-runtime/src/lib.rs
- cited: crates/generate/ess-entity-runtime/src/subset.rs
- inferred: crates/generate/ess-entity-runtime/tests/component_closure.rs
- inferred: website/docs/reference/entity-runtime-lowering.md
- inferred: crates/edge/ess-xtask/src/consumer_coverage/entry-classifications.json

confidence: high

`lib.rs`: `lower_command` :1974 and the `LoweringCode` enum :433. `subset.rs`: the refused rows,
`LoweringCode::ALL` and the code's name and sentence; it renders the reference page, which
`tests/lowerable_subset.rs` holds byte for byte. `entry-classifications.json` classifies every
`LoweringCode` variant for `task test-xtask`. `tests/component_closure.rs` is new and holds the
model above.

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
- The code is `CommandSubjectOutsideClosure`, named after the closure `entity_closure` computes,
  beside its siblings `CommandSpansEntities` and `StatelessCommandUnsupported`.
