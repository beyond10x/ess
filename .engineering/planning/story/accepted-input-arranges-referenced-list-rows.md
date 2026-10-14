---
format: aep.planning-md/3
id: story:accepted-input-arranges-referenced-list-rows
kind: story
status: draft
title: An accepted scenario arranges the rows a referenced list names
revision: 1
---
## Summary

An adopter's model declares a `references` relation of `cardinality: many` carried by a `List<…>` field of the target's identity, written from an input of the creating command. Synthesis builds the accepted scenario with a one-element list whose member names no row any step arranges. A target that resolves the references it stores answers its refusal, and the accepted scenario fails. The adopter reports that the same scenarios passed on an older release (0.26.0); not verified here.

## What main does (origin/main 260a3d1e3)

- A `references` relation is structural in the model. Validation reads it (`crates/specify/ess-domain/src/entity.rs:1553`, `:1708-1731`), and synthesis reads one only for a `cardinality: one` identity carrier (`crates/verify/ess-conformance/src/synthesize/related.rs:777-786`, `interpret/execute/values.rs:185-191`). No synthesis path reads a `many` relation carried by a list field when it builds an input (inferred from a search of `synthesize.rs`, `synthesize/related.rs` and `synthesize/subject_fact.rs:1853`; that search found no other reader).
- The accepted (`DefaultBranch`) scenario takes its input from the witness search (`synthesize.rs:11090`), so the list holds a fresh identity.
- A guard that refuses an unknown member would be a `when_related` / row-set guard. The model states none, so the target is checking something the model does not say. Even so, the suite should not send a dangling reference on the branch it calls accepted, because a model that names a relation implies the reference names a row.

## Reproducer (ess/24, neutral)

```yaml
format: ess/24
system: {name: demo}
domains:
  - domain: demo.reg
    types:
      - {name: demo.reg.ServerId, newtype: Uuid}
      - {name: demo.reg.SourceId, newtype: Uuid}
    entities:
      - name: demo.reg.Source
        identity: {name: id, type: demo.reg.SourceId}
        fields: []
      - name: demo.reg.Server
        identity: {name: id, type: demo.reg.ServerId}
        fields:
          - {name: sources, type: "List<demo.reg.SourceId>"}
        relations:
          - {name: sources, kind: references, target: demo.reg.Source, cardinality: many, via: sources}
    commands:
      - name: demo.reg.AddSource
        input: [{name: id, type: demo.reg.SourceId}]
        outcomes:
          - {name: added, creates: demo.reg.Source, instance: id}
      - name: demo.reg.Register
        input:
          - {name: id, type: demo.reg.ServerId}
          - {name: sources, type: "List<demo.reg.SourceId>"}
        outcomes:
          - name: accepted
            creates: demo.reg.Server
            instance: id
            sets: {sources: input.sources}
```

Today `ess verify conform synthesize` writes `demo.reg.Register/outcome/accepted` with `sources: ["<fresh uuid>"]` and no `AddSource` step before it (the shape the adopter's scenario shows). The exact spelling of the system/component header follows the repository's examples; the implementor adjusts it so the file validates.

## Acceptance

1. `crates/verify/ess-conformance/tests/referenced_list_input.rs`, `an_accepted_scenario_arranges_every_referenced_list_member`: for the reproducer, every identity in the accepted scenario's `sources` is created by an earlier step of that scenario (`AddSource`), or the list is empty. Red on main.
2. Same file, `an_unarrangeable_reference_target_sends_an_empty_list`: with `AddSource` removed (no route creates a `Source`), the accepted scenario sends `sources: []` when the list type admits empty, and synthesis records a refusal naming the relation when it does not (a list with a declared minimum length).
3. Same file, `a_one_cardinality_reference_by_input_is_arranged_too`: a `cardinality: one` reference carried by a plain field set from input, arranged in the same way. If main already arranges it, the test pins that and the story says so.
4. `cargo test -p ess-conformance --test referenced_list_input`, `--test unknown_instance` and `--test outcome_shapes` pass, and the committed suites and pinned counts that move are listed in the change.

## Design rule (decided here, recorded in `docs/design/` beside the related-row arrangement page)

On any scenario that requires an accepting branch, an input written into a field carrying a `references` relation names rows the scenario arranged before the command. Synthesis arranges them through the target's creating route, as it does for related rows (`synthesize/related.rs`). Where none exists, it sends an empty list (`many`), or refuses the scenario by name (`one`, or a non-empty list). No format version moves: the scenario uses existing steps.

## Scope

- `crates/verify/ess-conformance/src/synthesize.rs`: accepted-input construction (cited: `:11090` strategy text; the input path is inferred).
- `crates/verify/ess-conformance/src/synthesize/related.rs`: the arrangement helpers to reuse (cited `:777-786`; reuse inferred).
- New test file `tests/referenced_list_input.rs`. Committed suites under `examples/` may move (inferred).
- Shared surfaces with wave l: `synthesize.rs`, near `read-only-command-answers-an-unknown-instance` (`synthesize.rs:11555`), on a different function. It does not touch `explore.go` or `mutate.rs`.

## Estimate

du about 10G (ess-conformance, one `--test` at a time). Adversary: yes, because synthesis input selection changes every accepted scenario that writes a referenced field.
