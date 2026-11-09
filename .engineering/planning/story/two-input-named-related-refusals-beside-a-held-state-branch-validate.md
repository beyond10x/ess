---
format: aep.planning-md/3
id: story:two-input-named-related-refusals-beside-a-held-state-branch-validate
kind: story
status: draft
title: Two input-named related refusals beside a held-state branch validate
summary: Validation reports a held seat (already-bound) and an unknown seat (bound) as selected together once a second related refusal over another input joins a create-or-update command; found by the 0.70.0 unit I adversary, no adopter shape yet
tags:
- defect
- ess-0.71.0
- severity-medium
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-synthesis
- decomposes: epic:zero-open-issues-2026-11
revision: 3
---
## Problem

A create-or-update command with a held-state branch (`when_subject` + `preserves`), an
`unknown_instance:` creator and two `when_related: {exists: false}` refusals over two different
inputs is refused by `ess specify validate`: it reports the held-state branch and the creator as
selected together, though one needs a held subject and the other an unknown one, so they never both
hold. The same command with one related refusal validates, and so does the two-refusal command
without the held-state branch.

Cause (code reading): with two related rows the command takes the joint-partition path
(`crates/specify/ess-domain/src/command/related_guard.rs`, `validate_several` →
`validate_joint_partition`, around lines 2065-2218). `partitioned` (around line 2232) keeps the
`when_subject` branch and the `unknown_instance` branch, and the enumeration crosses only input
cases with related-row cases, never the subject's existence, so both count as selected
(`unanswered_case`, around line 2422, prints the refusal).

Minimal reproduction (ess 0.92.0), `two-related.yaml`:

```yaml
format: ess/31
system: demo
version: v1
domain: demo.desk
summary: A seat is bound to an existing agent at an existing desk; binding again rebinds or keeps it.
types:
  - {name: demo.desk.AgentId, kind: newtype, of: String}
  - {name: demo.desk.SeatId, kind: newtype, of: String}
  - {name: demo.desk.DeskId, kind: newtype, of: String}
entities:
  - name: demo.desk.Agent
    identity: {name: agent_id, type: demo.desk.AgentId}
    fields:
      - {name: label, type: String}
    lifecycle: {initial: Active, states: [Active], terminal: [Active]}
  - name: demo.desk.Desk
    identity: {name: desk_id, type: demo.desk.DeskId}
    fields: []
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
  - name: demo.desk.Binding
    identity: {name: seat_id, type: demo.desk.SeatId}
    fields:
      - {name: agent_id, type: demo.desk.AgentId}
    relations:
      - {name: agent, kind: references, target: demo.desk.Agent, cardinality: one, via: agent_id}
    lifecycle: {initial: Bound, states: [Bound], terminal: [Bound]}
errors:
  - {name: demo.desk.NoAgent, summary: No agent carries the identity., fields: []}
  - {name: demo.desk.NoDesk, summary: No desk carries the identity., fields: []}
events:
  - name: demo.desk.AgentAdded
    fields: [{name: agent_id, type: demo.desk.AgentId}]
  - name: demo.desk.DeskOpened
    fields: [{name: desk_id, type: demo.desk.DeskId}]
  - name: demo.desk.SeatBound
    fields: [{name: seat_id, type: demo.desk.SeatId}]
actors:
  - name: demo.desk.Operator
    may: [demo.desk.AddAgent, demo.desk.OpenDesk, demo.desk.Bind]
commands:
  - name: demo.desk.AddAgent
    input:
      - {name: agent_id, type: demo.desk.AgentId}
      - {name: label, type: String}
    outcomes:
      - name: added
        creates: demo.desk.Agent
        instance: agent_id
        emits: [demo.desk.AgentAdded]
        payload: {demo.desk.AgentAdded: {agent_id: input.agent_id}}
        sets: {label: input.label}
  - name: demo.desk.OpenDesk
    input:
      - {name: desk_id, type: demo.desk.DeskId}
    outcomes:
      - name: opened
        creates: demo.desk.Desk
        instance: desk_id
        emits: [demo.desk.DeskOpened]
        payload: {demo.desk.DeskOpened: {desk_id: input.desk_id}}
  - name: demo.desk.Bind
    input:
      - {name: seat_id, type: demo.desk.SeatId}
      - {name: agent_id, type: demo.desk.AgentId}
      - {name: desk_id, type: demo.desk.DeskId}
    outcomes:
      - name: already-bound
        when_subject:
          predicate: agent_id == input.agent_id
        preserves: demo.desk.Binding
        instance: seat_id
      - name: no-agent
        when_related: {via: input.agent_id, exists: false}
        error: demo.desk.NoAgent
      - name: no-desk
        when_related: {via: input.desk_id, exists: false}
        error: demo.desk.NoDesk
      - name: rebound
        updates: demo.desk.Binding
        instance: seat_id
        emits: [demo.desk.SeatBound]
        payload: {demo.desk.SeatBound: {seat_id: input.seat_id}}
        sets: {agent_id: input.agent_id}
      - name: bound
        unknown_instance: true
        creates: demo.desk.Binding
        instance: seat_id
        emits: [demo.desk.SeatBound]
        payload: {demo.desk.SeatBound: {seat_id: input.seat_id}}
        sets: {agent_id: input.agent_id}
views:
  - name: demo.desk.Desks
    source: demo.desk.Desk
    consistency: read_your_writes
    fields:
      - {name: desk_id, type: demo.desk.DeskId}
  - name: demo.desk.Bindings
    source: demo.desk.Binding
    consistency: read_your_writes
    fields:
      - {name: seat_id, type: demo.desk.SeatId}
      - {name: state, type: demo.desk.Binding.State}
      - {name: agent_id, type: demo.desk.AgentId}
  - name: demo.desk.Agents
    source: demo.desk.Agent
    consistency: read_your_writes
    fields:
      - {name: agent_id, type: demo.desk.AgentId}
      - {name: label, type: String}
```

- `ess specify validate --path two-related.yaml` exits 1:
  `[conflicting_declaration] command.demo.desk.Bind.outcomes: related [input.agent_id present; input.desk_id present] and input [] select 2 branches: already-bound, bound` (`ESS-COMMAND-004`).
- Without the `no-desk` branch: `valid`, exit 0.
- Without the `already-bound` branch: `valid`, exit 0.

## Acceptance

- `ess specify validate --path two-related.yaml` (the model above) exits 0.
- In `crates/verify/ess-conformance/tests/adversary_i_create_or_update.rs`,
  `a_create_or_update_beside_two_related_refusals_validates` asserts `is_ok()` (today it asserts
  the refusal), and the same model synthesizes with no refusal and every scenario of `Bind`
  (`already-bound`, `no-agent`, `no-desk`, `rebound`, `bound`) passes on the interpreter, as
  `every_branch_passes` checks.
- A joint partition whose two selected branches can both hold still refuses with
  `ESS-COMMAND-004`: the existing `select … branches` refusal tests under
  `crates/specify/ess-domain/tests/` stay green unchanged.

## Scope

- cited: crates/specify/ess-domain/src/command/related_guard.rs
- cited: crates/verify/ess-conformance/tests/adversary_i_create_or_update.rs
- inferred: crates/specify/ess-domain/src/command/precedence.rs (whether the subject-existence phase is read when two branches are counted)

confidence: medium

## Origin

Adversary pass 1, unit I of the 0.70.0 wave (`review-result:unit-i-pass-1`), finding 3
(`related_guard.rs`, boundary, pre-existing). Findings 1 and 2 of that pass (synthesis and the
`ess/30` enforced-reference proof counting an `unknown_instance` creator as refusing a missing row
first) belong to `story:a-create-or-update-writer-is-no-proof-a-stored-reference-is-present`, not
to this story. No adopter shape yet.
