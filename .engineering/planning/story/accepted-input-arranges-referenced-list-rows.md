---
format: aep.planning-md/3
id: story:accepted-input-arranges-referenced-list-rows
kind: story
status: implemented
title: An accepted scenario arranges the rows a referenced list names
tags:
- defect
- severity-high
- zero-open-issues
relations:
- decomposes: epic:zero-open-issues-2026-11
- serves: vision:O2
scope:
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/related.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/referenced_list_input.rs
- confidence: inferred
  path: docs/design/referenced-input-arrangement.md
revision: 11
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:37:48Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:37:49Z", actor: "human:timo", revision: 10, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "active", to: "implemented", at: "2026-11-12T08:19:44Z", actor: "human:timo", revision: 11, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}}
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

Decided here, recorded in a new design page `docs/design/referenced-input-arrangement.md` beside
the related-row arrangement page (`docs/design/related-rows-quantifier.md`).

On any scenario that requires an accepting branch, an input written into a field carrying a
`references` relation names rows the scenario arranged before the command. Synthesis arranges them
through the target's creating route, as it does for related rows (`synthesize/related.rs`). Where
none exists, it sends an empty list (`many`), or refuses the scenario by name (`one`, or a
non-empty list). No format version moves: the scenario uses existing steps.

## Decisions

- Where a creating route exists, the accepted scenario arranges a non-empty list rather than
  sending an empty one, because an empty list would pass without exercising the reference at all.
- The refusal case is checked on the `cardinality: one` variant, because a plain reference field
  has no empty value to fall back to, so the refusal is the only result the design rule admits.

## Acceptance

1. `cargo test -p ess-conformance --locked --test referenced_list_input
   an_accepted_scenario_arranges_every_referenced_list_member`: for the reproduction, the
   accepted scenario `demo.reg.Register/outcome/accepted` sends a non-empty `sources`, and every
   identity in it is the `id` of an earlier `execute_command demo.reg.AddSource` step of that
   scenario. Red on 0.92.0.
2. Same target, `an_unarrangeable_reference_list_sends_an_empty_list`: with `AddSource` removed
   from the reproduction (no route creates a `Source`), the accepted scenario's
   `execute_command demo.reg.Register` step sends `sources: []`.
3. Same target, `an_unarrangeable_one_cardinality_reference_is_refused_by_name`: the
   `cardinality: one` variant with `AddSource` removed has no `demo.reg.Register/outcome/accepted`
   scenario, and synthesis records a refusal for it whose reason names the relation
   `demo.reg.Server.sources`.
4. Same target, `a_one_cardinality_reference_by_input_is_arranged_too`: the `cardinality: one`
   variant has an `execute_command demo.reg.AddSource` step creating the identity the accepted
   scenario sends in `sources`. Red on 0.92.0.
5. `cargo test -p ess-conformance --locked --test unknown_instance` and `--test outcome_shapes`
   pass unchanged.
6. `cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table` passes, with a row
   re-pinned only where its model writes an input into a field carrying a `references` relation,
   the cause named beside the pin.

## Scope

- inferred: crates/verify/ess-conformance/src/synthesize.rs
- cited: crates/verify/ess-conformance/src/synthesize/related.rs
- inferred: crates/verify/ess-conformance/tests/referenced_list_input.rs
- inferred: docs/design/referenced-input-arrangement.md

confidence: medium

Shared surfaces moved: re-pins suite-bytes rows (`synthesis-suite-bytes-table.tsv`). No example
under `examples/` declares a `references` relation, so no committed example suite moves.

## Estimate

Disk about 10G (ess-conformance, one `--test` at a time). Adversary: yes, because synthesis input
selection changes every accepted scenario that writes a referenced field.
