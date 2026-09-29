---
title: Guards and outcome selection
sidebar_position: 3
description: Select an outcome from held state or stored fields, cover inputs no guard decides, share one outcome across commands, and leave illegal moves out.
---

# Guards and outcome selection

## Select an outcome from the held subject state

`ess/3`, introduced in 0.23.0, allows `when_subject_state` beside an ordinary input predicate:

```yaml
- name: preserved
  when: incoming == Ringing
  when_subject_state: Bridged
  updates: calls.core.Call
  instance: call_id
  emits: [calls.core.Observed]
```

The surrounding model declares that event, entity and input types. Both
conditions must hold. Omitting `when` selects any admitted input in the
named held state. The state comes from the existing subject row; it is not an
extra caller field. All ordinary, state-guarded and default branches must name
the same entity and input identity. A missing row does not become an initial row.

The guard cannot accompany `creates`, `external` or `wrong_state`. Guarded moves
must start in the declared guard state. Validation checks up to 64 joint
state/input assignments for gaps and overlaps using the shared finite coverage
proof; unsupported or open input domains require a genuine default. Runtime
witnesses additionally validate their concrete inputs and invariants.

From `ess/18` (0.41.0) the guard may list several states, and a refusal may carry it without
naming a subject; it reads the subject its sibling branches name. That is how a command answers
differently in different states its moves do not start from, where `wrong_state` gives them all
one answer:

```yaml
- name: shipped
  moves: demo.ship.Order.ship
  instance: order_id
- name: already-shipped            # a re-send is accepted and changes nothing
  when_subject_state: Shipped
  preserves: demo.ship.Order
  instance: order_id
- name: gone
  when_subject_state: [Delivered, Cancelled]
  error: demo.ship.Gone
```

Every state no guard claims falls to the default, whose move must start there. The synthesized
suite arranges an order in `Delivered` and one in `Cancelled`, each refused with `Gone` and left
unchanged.

An input-guarded refusal — `when:` with an `error:`, naming no subject — may sit beside the
state-guarded branches, at every format that has `when_subject_state`. It is answered before the
record is looked up and before its state is read, so "the new value is refused whatever the record's
state" is written once:

```yaml
- name: too-short
  when: secret.count < 12
  error: demo.secrets.SecretTooShort
- name: rotated
  when_subject_state: Configured
  updates: demo.secrets.Configuration
  instance: tenant_id
  sets: {secret: input.secret}
- name: not-configured
  error: demo.secrets.NotConfigured
```

Of two refusals one input selects, the first declared answers.

The refusal still names no subject: with `updates:` or `preserves:` it is refused as
`refusal_mutated_state`. The synthesized suite sends the refused input for an identity nothing
stores, then for a record arranged in each state of the lifecycle, and requires the error, no event
and the record unchanged each time. Where a state-guarded branch also reads the input, the record in
its state is sent an input both guards admit. A target that reads the held state or looks the record
up before it checks the input fails the refusal's scenario.

## Guard an outcome by the subject's stored fields

"Express parcels over 20 kg are refused at dispatch" depends on two fields stored when the parcel
was created, not on anything the dispatch request carries. `ess/9` (since 0.34.0) states it with
`when_subject: {predicate: …}`, a predicate over the declared fields of the entity the command
addresses:

```yaml
commands:
  - name: shipping.parcel.Create
    input:
      - {name: service, type: shipping.parcel.Service}   # enum: Standard | Express
      - {name: weight_kg, type: Integer}
    outcomes:
      - name: created
        creates: shipping.parcel.Parcel
        instance: parcel_id
        sets: {service: input.service, weight_kg: input.weight_kg}
        emits: [shipping.parcel.Created]
        payload:
          shipping.parcel.Created:
            parcel_id: {generated: true}

  - name: shipping.parcel.Dispatch
    input: [{name: parcel_id, type: Uuid}]
    outcomes:
      - name: refused-overweight
        when_subject:
          predicate:
            all:
              - service == Express
              - weight_kg > 20
        error: shipping.parcel.ExpressOverweight
      - name: dispatched
        moves: shipping.parcel.Parcel.dispatch
        instance: parcel_id
        emits: [shipping.parcel.Dispatched]
```

The predicate reads the entity's declared fields and nothing else: not the input, which stays in
`when:` beside it, and — before `ess/18` — not `state`. From `ess/18` (0.41.0) it may read
`state`, the lifecycle state the row holds, to select by the state and a stored field together:
`{all: [state == Ready, hold_note != ""]}`; a branch reading it that moves must be able to move
from every state it may be selected in. The refusal names no
subject of its own and reads the parcel its sibling moves. An `Optional` field may be read; an
absent value is unknown and selects no branch, so write `not defined(field)` to select on absence.

Validation partitions closed enum fields jointly with the input, so two branches that split an enum
need no default. An open comparison such as `weight_kg > 20` needs a genuine default, here
`dispatched`. Declare an unfiltered view that projects the identity, `state` and every guarded
field: conformance arranges a parcel through `Create`'s `sets:` mappings — Express at 21 kg for the
refusal, Express at 20 kg and Standard at 21 kg for the default — observes it through that view,
and dispatches it. A `read_your_writes` view is read once; where the only such view is `eventual`,
the observation goes in an `eventually` block that waits until the view shows the arranged parcel.
A refused parcel is asserted unchanged only through a `read_your_writes` view: an `eventual` view
that has not caught up shows the old row too, so without one that check is left out.
In a state the command does not move from, no branch is selected by the stored fields, and the
refusal scenario sends the command as it does for a command without `when_subject`. The older
`when_subject: {field, equals}` form keeps `ess/6`.

## An outcome the input cannot decide says that too

Whether a mail provider accepts an address is not a function of the request. From
`domains/email.yaml`:

```yaml
- name: failed
  external: the provider rejects the recipient address
  error: billing.email.Undeliverable
```

Writing `when: false` would claim the branch is unreachable — a different statement, and a false
one. A generator reads `external` and injects a fault instead of trying to construct an input.

## One outcome for many commands

When a remote service carries out every command on the caller's behalf, every command can end the
same way: the service rejects the session credential. `ess/12` (since 0.34.0) declares that
outcome once, in a top-level `outcome_groups:` list that any file may carry:

```yaml
format: ess/12
outcome_groups:
  - name: remote-backed
    actor: calls.Agent              # or  commands: [calls.Hold, calls.Resume]
    except: [calls.Park]            # only beside actor: or domain:
    outcomes:
      - name: credential-rejected
        external: the service rejects the session credential
        error: session.Unauthenticated
        summary: The remote service refused the session.
        refs: [issue:beyond10x/ess#105]
```

A group names its members in exactly one way: an explicit `commands:` list, `actor:` for every
command that actor `may:` invoke, or `domain:` for every command that domain's files declare. Each
member gains the group's outcomes after its own, in ascending group-name order when several groups
select it. That happens before validation, so validation, conformance and every generator see
ordinary outcomes, and a group compiles to exactly what copying the outcome into each command by
hand would. A group of 28 commands therefore still adds 28 scenarios. A group's outcome is always an
external refusal: `external:` and `error:`, with an optional `summary:` and `refs:`, and nothing that
depends on the command it lands in.

`except:` is only for `actor:` and `domain:`; with an explicit list, leave the command out of
`commands:` instead. A member that already declares an outcome of the same name is refused at the
group, never silently overridden: rename one of the two, drop the command's own, or list the command
under `except:`. Two groups that would give one command outcomes of the same name are refused too,
and neither group expands into that command.

## Illegal lifecycle moves are illegal by absence

`Paid` cannot become `Cancelled` because no transition says it can. There is no forbidding rule,
because a rule would be a second place for the same truth to live, and two places eventually
disagree. The generated documentation lists the absent pairs, derived from the same transitions.
