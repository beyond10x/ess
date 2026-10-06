---
title: Selections, periodic causes and delivery context
sidebar_position: 6
description: "What a binding reads besides its event payload — ordered selections, a periodic host cause, the channel an event arrived on — and clock-reading provenance."
---

# Selections, periodic causes and delivery context

## Select ordered records in a binding

`ess/3` adds binding-local `selection_inputs` and ordered `selections`.
For an event whose `candidates` field already has type `List<example.calls.Leg>`:

```yaml
selection_inputs:
  - name: candidates
    from: event.candidates
    as: List<example.calls.Leg>
selections:
  - name: first_agent
    first:
      in: candidates
      where: item.role == Agent
  - name: first_external
    first:
      in: candidates
      excluding: [first_agent]
      where: item.role == External
  - name: preferred
    first_present: [first_external, first_agent]
mapping:
  selected_id: {selection: preferred, path: [id]}
```

The record type must declare those fields and enum variants. Exclusion removes a
selected list occurrence by its original index, so equal-valued records remain
distinct. References point backward; `first_present` alternatives use the same
list. The result is optional and must fit the command input's declared type.
Whole records use `path: []`; field paths reuse bounded accessor rules.

Malformed records are refused before selection, including records after an early
match. Executable selection currently refuses inputs with unsupported declared
invariants or clock-reading attachments, including nested members and list
aliases, instead of dropping their constraints. Predicates use a bounded declared fragment: defined checks, typed literal
equality/inequality and Boolean combinations. Selection does not sort or invent
values. If the event carries a different representation, declare the exact
conversion to the local list type. The host prepares that value once and can
pass it to generated Rust/Go selection helpers; the local list is not a new
field on the wire.

## Declare a periodic host cause

A periodic cause belongs to a named host instance with an explicit owner and
authority. This `ess/3` binding fragment requires the declared command, component
and mapped input types:

```yaml
when:
  periodic:
    every: PT2S
    anchor: host_activation
    first: after_period
    cadence: fixed_rate
    overlap: serial_per_instance
    missed: one_pending_drop_excess
    lifetime: host_instance
    cancellation: stop_acknowledged
    host:
      owner: poll-service
      authority: authenticated-session-status
      eligibility: host_boolean
      context_fields: [{name: agent_id, type: String}]
      read_fields: [{name: status, type: String}]
invoke: {command: example.poll.Refresh}
mapping:
  agent_id: host_context.agent_id
  status: host_read.status
delivery: at_most_once
on_failure: drop
```

Context is constant for the host lifetime; reads are fresh for each eligible
occurrence. The first tick follows one period, work is serial, and excess busy
ticks coalesce to one pending tick. Stop acknowledgement means the loop and its
work have quiesced. Native generation reports `PeriodicHostRequired` until that
real host capability is supplied; it does not fabricate a scheduler or event.

## Read the channel an event arrived on

Some events do not carry their recipient in the payload. For example, a service subscribes to
`accounts/{account_id}/messages` for each account, and the recipient is the subscription the
event arrived on. `ess/18` lets an event binding declare that delivery context and read it:

```yaml
when:
  event: example.inbox.MessageReceived
  context_authority: account-messages
  context_fields:
    - {name: account_id, type: example.inbox.AccountId}
invoke: {command: example.inbox.RecordMessage}
mapping:
  account_id: context.account_id
  message_id: event.message_id
  peer: event.from
delivery: at_least_once
on_failure: retry
```

`context_fields` is a typed record separate from the payload. `context_authority` names the
external channel whose authority binds it. It is a name, not a credential, and each key
requires the other. `context.<field>` reads one declared field. The field's type must fit the
input, or a declared conversion must cross it. The host binds the context from the channel the
occurrence arrived on and supplies it with that occurrence. A redelivery carries the context
of the occurrence it repeats.

The rules:

- A context is admitted only for an event an external channel delivers. If a command outcome
  of the specification emits the event, or a binding escalates into it, the event has no
  channel, and the context is refused.
- A context mapping with no declaration is refused. It is never looked up in the payload.
  `event.channel` is not a field.
- `host_context.<field>` still belongs to a periodic host.
- Below `ess/18`, both keys and `context.<field>` are refused.

Conformance delivers the event itself, under two different contexts, and requires each
invocation to carry its own. See
[Verify conformance](../verify/observations.md#deliver-an-event-with-its-context).

## Preserve clock-reading provenance

An `ess/3` newtype can attach a reading contract while retaining its scalar wire
representation:

```yaml
- name: example.clock.LocalReading
  kind: newtype
  of: String
  reading:
    encoding: local_date_time_millis_literal_z
    origins: [{role: producer_process, offset: requires_observation}]
```

Supported encodings are offset date-time text, local millisecond text with a
literal `Z`, and exact integer Unix seconds. A literal `Z` on local text does not
establish UTC. The observation adapter supplies the process instance, clock
epoch, origin and actual formatter offset for the particular occurrence.
Comparison requires the same observed source and epoch. Unknown evidence and
cross-source calibration remain unsupported; generic input predicates cannot
silently discard the attachment.

Normalization accepts years 1970–9999 and fixed offsets within ±14:00. Offset
text allows no fraction or exactly three millisecond digits; local literal-Z
text requires exactly three. Other precision, leap seconds and inferred host
timezone settings are outside this bounded contract.
