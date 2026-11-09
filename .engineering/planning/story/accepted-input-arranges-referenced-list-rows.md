---
format: aep.planning-md/3
id: story:accepted-input-arranges-referenced-list-rows
kind: story
status: draft
title: An accepted scenario arranges the rows a referenced list names
tags:
- defect
- severity-high
- zero-open-issues
relations:
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/related.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/referenced_list_input.rs
- confidence: inferred
  path: docs/design/
- confidence: inferred
  path: examples/
revision: 5
---
## Summary

A model declares a `references` relation carried by a field of the target's identity type — a
`List<…>` under `cardinality: many`, or a plain field under `cardinality: one` — and a creating
command writes that field from its input. Synthesis builds the accepted scenario with a fresh
identity in that input that names no row any step arranges. A target that resolves the references
it stores answers its refusal, and the accepted scenario fails. An adopter reported that the same
scenarios passed on an older release (0.26.0); not verified here.

## Problem

Still present on 0.92.0 (re-checked 2026-11-09).

- A `references` relation is structural in the model. Synthesis reads one for arrangement only when
  the relation is `cardinality: one` and carried by the entity's own identity
  (`crates/verify/ess-conformance/src/synthesize/related.rs:797-808`, `identity_reference`). No
  synthesis path reads a relation carried by a non-identity field when it builds an input.
- The accepted (`DefaultBranch`) scenario takes its input from the witness search, which only
  refutes sibling guards (`crates/verify/ess-conformance/src/synthesize.rs:9569`), so the field
  holds a fresh identity.
- The model states no `when_related` or row-set guard that refuses an unknown member, so the target
  is checking something the model does not say. Even so, the suite should not send a dangling
  reference on the branch it calls accepted, because a model that names a relation implies the
  reference names a row.

Reproduction (`ess/30`):

```yaml
format: ess/30
system: demo
version: v1
domain: demo.reg
types:
  - {name: demo.reg.ServerId, kind: newtype, of: Uuid}
  - {name: demo.reg.SourceId, kind: newtype, of: Uuid}
entities:
  - name: demo.reg.Source
    identity: {name: id, type: demo.reg.SourceId}
    fields: []
    lifecycle: {initial: Live, states: [Live], terminal: [Live]}
  - name: demo.reg.Server
    identity: {name: id, type: demo.reg.ServerId}
    fields:
      - {name: sources, type: "List<demo.reg.SourceId>"}
    relations:
      - {name: sources, kind: references, target: demo.reg.Source, cardinality: many, via: sources}
    lifecycle: {initial: Live, states: [Live], terminal: [Live]}
events:
  - name: demo.reg.SourceAdded
    fields: [{name: id, type: demo.reg.SourceId}]
  - name: demo.reg.Registered
    fields: [{name: id, type: demo.reg.ServerId}]
actors:
  - name: demo.reg.Operator
    may: [demo.reg.AddSource, demo.reg.Register]
commands:
  - name: demo.reg.AddSource
    input: [{name: id, type: demo.reg.SourceId}]
    outcomes:
      - name: added
        creates: demo.reg.Source
        instance: id
        emits: [demo.reg.SourceAdded]
        payload: {demo.reg.SourceAdded: {id: input.id}}
  - name: demo.reg.Register
    input:
      - {name: id, type: demo.reg.ServerId}
      - {name: sources, type: "List<demo.reg.SourceId>"}
    outcomes:
      - name: accepted
        creates: demo.reg.Server
        instance: id
        sets: {sources: input.sources}
        emits: [demo.reg.Registered]
        payload: {demo.reg.Registered: {id: input.id}}
views:
  - name: demo.reg.Servers
    source: demo.reg.Server
    consistency: read_your_writes
    fields:
      - {name: id, type: demo.reg.ServerId}
      - {name: sources, type: "List<demo.reg.SourceId>"}
```

`ess verify conform synthesize --path spec.yaml --out suite.yaml` writes
`demo.reg.Register/outcome/accepted` with one step, `execute_command demo.reg.Register` with
`sources: ["00000000-0000-4000-8000-f89790aeec87"]`; no `AddSource` step creates that identity.
The same model with `sources` a plain `demo.reg.SourceId` under `cardinality: one` sends a fresh
`sources` identity the same way, also unarranged.

## Design rule

Decided here, recorded in `docs/design/` beside the related-row arrangement page.

On any scenario that requires an accepting branch, an input written into a field carrying a
`references` relation names rows the scenario arranged before the command. Synthesis arranges them
through the target's creating route, as it does for related rows (`synthesize/related.rs`). Where
none exists, it sends an empty list (`many`), or refuses the scenario by name (`one`, or a
non-empty list). No format version moves: the scenario uses existing steps.

## Acceptance

1. `crates/verify/ess-conformance/tests/referenced_list_input.rs`,
   `an_accepted_scenario_arranges_every_referenced_list_member`: for the reproduction, every
   identity in the accepted scenario's `sources` is created by an earlier `AddSource` step of that
   scenario, or the list is empty. Red on 0.92.0.
2. Same file, `an_unarrangeable_reference_target_sends_an_empty_list`: with `AddSource` removed (no
   route creates a `Source`), the accepted scenario sends `sources: []` when the list type admits
   empty, and synthesis records a refusal naming the relation when it does not (a list with a
   declared minimum length).
3. Same file, `a_one_cardinality_reference_by_input_is_arranged_too`: the `cardinality: one`
   variant above has an `AddSource` step creating the identity it sends in `sources`. Red on 0.92.0.
4. `cargo test -p ess-conformance --test referenced_list_input`, `--test unknown_instance` and
   `--test outcome_shapes` pass, and the committed suites and pinned counts that move are listed in
   the change.

## Scope

- inferred: crates/verify/ess-conformance/src/synthesize.rs
- cited: crates/verify/ess-conformance/src/synthesize/related.rs
- inferred: crates/verify/ess-conformance/tests/referenced_list_input.rs
- inferred: examples/
- inferred: docs/design/

confidence: medium

## Estimate

Disk about 10G (ess-conformance, one `--test` at a time). Adversary: yes, because synthesis input
selection changes every accepted scenario that writes a referenced field.
