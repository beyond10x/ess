---
title: Read stored state in the command a binding invokes
sidebar_position: 6
description: "Why a binding maps an identity and the invoked command reads the row, and how binding scenarios arrange that row."
---

# Read stored state in the command a binding invokes

A binding maps what the occurrence carries: the event's payload, its delivery context, a periodic
host's context or read, and its selections. It does not read stored state, because a binding
belongs to no component and so has no store to read from: there is no store behind a mapping
(beyond10x/ess#440). When the command a binding causes needs a value held in stored state, for
example the current `bridged` flag of the call a joined leg belongs to, map the identity and let
the invoked command read the row:

```yaml
format: ess/22
system: demo
version: v1
domain: demo.calls
types:
  - {name: demo.calls.CallId, kind: newtype, of: Uuid}
entities:
  - name: demo.calls.Call
    identity: {name: call_id, type: demo.calls.CallId}
    fields:
      - {name: bridged, type: Boolean}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
events:
  - name: demo.calls.CallOpened
    fields:
      - {name: call_id, type: demo.calls.CallId}
  - name: demo.calls.LegJoined
    fields:
      - {name: leg_id, type: String}
      - {name: call_id, type: demo.calls.CallId}
  - name: demo.calls.LegMarked
    fields:
      - {name: leg_id, type: String}
      - {name: call_bridged, type: Boolean}
commands:
  - name: demo.calls.OpenCall
    input:
      - {name: bridged, type: Boolean}
    outcomes:
      - name: opened
        creates: demo.calls.Call
        instance: call_id
        sets: {bridged: input.bridged}
        emits: [demo.calls.CallOpened]
        payload:
          demo.calls.CallOpened: {call_id: {generated: true}}
  - name: demo.calls.JoinLeg
    input:
      - {name: leg_id, type: String}
      - {name: call_id, type: demo.calls.CallId}
    outcomes:
      - name: joined
        emits: [demo.calls.LegJoined]
        payload:
          demo.calls.LegJoined: {leg_id: input.leg_id, call_id: input.call_id}
  - name: demo.calls.MarkLeg
    input:
      - {name: leg_id, type: String}
      - {name: call_id, type: demo.calls.CallId}
    outcomes:
      - name: marked
        emits: [demo.calls.LegMarked]
        payload:
          demo.calls.LegMarked:
            leg_id: input.leg_id
            # Read from the stored call when MarkLeg runs, under its one snapshot.
            call_bridged: {related: {via: input.call_id, field: bridged}}
bindings:
  - id: mark-joined-leg
    when: {event: demo.calls.LegJoined}
    invoke: {command: demo.calls.MarkLeg}
    mapping:
      leg_id: event.leg_id
      call_id: event.call_id
    delivery: at_least_once
    on_failure: drop
```

The read belongs to the command for three reasons. A command reads the store once, immediately
before it selects a branch, and every guard and value of that command uses that one snapshot
([filtered related reads](https://github.com/beyond10x/ess/blob/main/docs/design/filtered-related-reads.md)).
A second read in the binding would be a second snapshot, and the mapped value and the command's
guards could disagree. A redelivery carries the context of the occurrence it repeats, and a store
read made again on redelivery could return a different value. And the command already has every
reading construct: `{related: {via: <field>, field: <field>}}` in `sets:` or `payload:`, a
`when_related` or row-set guard to choose a branch by the related row, and a stored-subject guard.
The conformance suite arranges the referenced row and asserts the value the invoked command
emits, so the read is checked where it is made.

The same holds when the command chooses its branch by the row: a flag on another entity that
decides whether the binding's work is done or skipped is a `when_related` guard of the invoked
command, not a binding condition.

```yaml
  - name: demo.seats.IssueSeat
    input:
      - {name: member_id, type: demo.seats.MemberId}
      - {name: plan_id, type: demo.seats.PlanId}
    outcomes:
      - name: no-such-plan
        when_related: {via: input.plan_id, exists: false}
        error: demo.seats.NoSuchPlan
      - name: ineligible
        when_related: {via: input.plan_id, predicate: eligible == false}
        emits: [demo.seats.SeatSkipped]
      - name: issued
        creates: demo.seats.Seat
        instance: member_id
        emits: [demo.seats.SeatIssued]
```

Where the binding maps `plan_id` straight from an event field that the triggering command copies
from its own input, the binding's `flow`, `mapping` and `delivery` scenarios create the plan through
its creating command first, so the invocation takes the accepting branch, and send the trigger
naming that plan. The accepting branch is the default where it accepts, otherwise the first
predicate branch that does. Each branch of the command keeps its own outcome scenario. If a guard
also reads the invoked command's input, or the row is named by a value the implementation mints,
the branch depends on values the scenario does not choose, and the binding's scenarios are still
refused with `ESS-SYNTH-010`.

The triggering command may choose the branch that publishes the event by a `when_related` predicate
over that same row (`when_related: {via: input.tier_id, predicate: eligible == true}`). One row then
decides both commands. The scenario arranges the row for the invoked command's branch, reads the
trigger's guards on it in declaration order, and keeps the arrangement where the branch the trigger
takes publishes the event. A refusal of the invoked command is witnessed through whichever
publishing branch the row it refuses sends the trigger down. Three cases stay refused with
`ESS-SYNTH-010`: the trigger's predicate also reads an input; the trigger reads a row of another
entity through the input the binding copies; and the trigger publishes only for rows the invoked
refusal does not refuse, in which case the refusal's message names the trigger's `when_related`
predicate.

The row the invoked command acts on may also be named by a stored field of the row the trigger
acts on. The triggering branch then publishes that field with `{subject: <field>}`:

```yaml
  - name: demo.device.RedeemToken
    input: [{name: token_id, type: demo.device.TokenId}]
    outcomes:
      - name: redeemed
        moves: demo.device.Token.redeem
        instance: token_id
        emits: [demo.device.TokenRedeemed]
        payload:
          demo.device.TokenRedeemed: {token_id: input.token_id, device_id: {subject: device_id}}
```

A binding mapping `device_id: event.device_id` into a command that updates the device gets its
`flow`, `mapping` and `delivery` scenarios: the device is arranged first, and the token is created
holding its identity. This needs the token's creating command to write `device_id` from an input,
unconverted. Otherwise the binding is refused with `ESS-SYNTH-010`, naming the stored field.

When the invoked command is a set effect that publishes an event once per changed row
(`each: changed`), the binding's `flow` and `delivery` scenarios first create one row the filter
selects. That row rests in a state the event's `when_state:` admits. Without such a row the honest
system publishes nothing. Synthesis can create the row when every input the filter reads is mapped
from a literal, or unconverted from an event field the triggering command copies from its own input.
Otherwise the two scenarios are refused with `ESS-SYNTH-010`, naming the set effect.

A `mapping:` value written as `{related: …}` is refused as an unknown field of a selection.
