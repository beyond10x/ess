---
format: aep.planning-md/3
id: story:check-history-reads-identity-from-response-field
kind: story
status: active
title: check-history determines a record identity taken from a response field
tags:
- adopter-report
- defect
- severity-medium
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-verdicts
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: inferred
  path: crates/edge/ess-cli/tests/check_history.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/interpret/execute.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/interpret/execute/determined.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/interpret/execute/history.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/interpret/response.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/linearize.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/linearizability.rs
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:43:13Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:43:14Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"review_outcome":1}}}
---
## Outcome

`ess verify conform check-history` determines a record's identity when the model takes it from a
response field, instead of exiting 2 with `check.model-undetermined`.

## Problem

An adopter on ess 0.56.0: `check-history` exits 2 (`check.model-undetermined`) on a history whose
record identity comes from a response field. Still so on 0.92.0.

`crates/verify/ess-conformance/src/interpret/execute/history.rs` steps each operation with
`response::Authority::default()`, which holds no issued response, so `Authority::prepare`
(`src/interpret/response.rs`) returns `None` for a creating branch whose observed identity reads the
response (`src/interpret/execute.rs`, the `Creates`/`Observed` arm). The payload evaluator
`determined::response` (`src/interpret/execute/determined.rs`) then has no actual value and answers
`NotInterpreted`, which `linearize.rs` reports as `check.model-undetermined`. An `ess-history/3`
operation records its published events, so the identity the response carried is in the history.

Minimal reproduction (ess 0.92.0):

```yaml
format: ess/18
system: demo
version: v1
domain: demo.items
types:
  - {name: demo.items.ItemId, kind: newtype, of: Uuid}
entities:
  - name: demo.items.Item
    identity: {name: item_id, type: demo.items.ItemId}
    lifecycle: {initial: Open, states: [Open, Closed], terminal: [Closed],
                transitions: [{name: close, from: [Open], to: Closed}]}
events:
  - {name: demo.items.Created, fields: [{name: item_id, type: demo.items.ItemId}]}
  - {name: demo.items.Closed, fields: [{name: item_id, type: demo.items.ItemId}]}
commands:
  - name: demo.items.Create
    input: []
    response: [{name: item_id, type: demo.items.ItemId}]
    outcomes:
      - name: created
        creates: demo.items.Item
        instance: item_id
        emits: [demo.items.Created]
        payload: {demo.items.Created: {item_id: {response: item_id}}}
  - name: demo.items.Close
    input: [{name: item_id, type: demo.items.ItemId}]
    outcomes:
      - name: closed
        moves: demo.items.Item.close
        instance: item_id
        emits: [demo.items.Closed]
        payload: {demo.items.Closed: {item_id: input.item_id}}
```

An `ess-history/3` document, one client: `Create` (input `{}`, `subject_key` `…aa`, events
`[Created {item_id: …aa}]`, outcome `created`), then `Close` (input `{item_id: …aa}`, outcome
`closed`, events `[Closed {item_id: …aa}]`).

```
$ ess verify conform check-history --path model.yaml --history right.json
right.json cannot be checked: check.model-undetermined: the model cannot decide operation …0101: an actual response field or its executable conversion is not interpreted yet
exit 2
```

The same history with `Created {item_id: …bb}` gives the same exit 2, so neither a correct nor a
wrong history is judged.

## Acceptance

- `cargo test -p ess-cli --locked --test check_history` passes a new case: `ess verify conform
  check-history --path model.yaml --history right.json` over the model and the correct
  `ess-history/3` history above exits 0 (linearizable).
- The same target passes a case where `Create` keeps `subject_key` `…aa` and its `Created` event
  publishes `…bb`: the command exits 1 and the violation names the `Create` operation `…0101`.
- The same target passes a case over an `ess-history/2` document of the same two operations with
  no `events`: it exits 0, the created identity taken from `Create`'s `subject_key` `…aa`.
- `cargo test -p ess-conformance --locked --test linearizability` passes the three histories
  above as verdicts (linearizable, a violation at `…0101`, linearizable), with no
  `CheckRefusal::Model`.

Adjacent, not a duplicate: `story:stored-field-equals-returned-response-value` (storing a returned
value with `sets:`).

## Decisions

- The identity a creating branch reads from a response field is taken from the operation's
  `subject_key` in every history version, so an `ess-history/1` or `/2` document is checked rather
  than refused: `subject_key` is "the identity of the subject the command addressed"
  (`src/history.rs`, `Operation`) in all three versions, and a `/3` operation's events are then
  judged against it.

## Scope

- cited: crates/verify/ess-conformance/src/interpret/execute/history.rs
- cited: crates/verify/ess-conformance/src/interpret/response.rs
- cited: crates/verify/ess-conformance/src/interpret/execute/determined.rs
- inferred: crates/verify/ess-conformance/src/interpret/execute.rs
- inferred: crates/verify/ess-conformance/src/linearize.rs
- inferred: crates/verify/ess-conformance/tests/linearizability.rs
- inferred: crates/edge/ess-cli/tests/check_history.rs

confidence: medium

The story moves no shared surface beyond proposing its `CHANGELOG.md` sentence.
