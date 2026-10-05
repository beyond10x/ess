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

The refusal still names no subject: with `updates:` or `preserves:` it is refused as
`refusal_mutated_state`. The synthesized suite sends the refused input for an identity nothing
stores, then for a record arranged in each state of the lifecycle, and requires the error, no event
and the record unchanged each time. Where a state-guarded branch also reads the input, the record in
its state is sent an input both guards admit. A target that reads the held state or looks the record
up before it checks the input fails the refusal's scenario.

### Keep a record unchanged in any of several states

The list works on every subject branch, not only on a refusal. A re-sent `ShipOrder` that keeps the
order as it is while it is `Shipped` or `Delivered` is one branch:

```yaml
- name: shipped
  moves: demo.ship.Order.ship
  instance: order_id
  emits: [demo.ship.OrderShipped]
  payload: {demo.ship.OrderShipped: {order_id: input.order_id}}
- name: already-there              # a re-send changes nothing
  when_subject_state: [Shipped, Delivered]
  preserves: demo.ship.Order
  instance: order_id
- name: gone
  when_subject_state: Cancelled
  error: demo.ship.Gone
```

The synthesized suite arranges an order in each listed state, sends the command and requires
`already-there` with the order unchanged, so a target that keeps the order in only one of them
fails. `when_subject_state: {in: [...]}` is refused; the list is the spelling.

## Which branch answers

One order answers every command, whichever branches it declares
([the precedence order](https://github.com/beyond10x/ess/blob/main/docs/design/cross-record-and-stored-field-guards.md#the-precedence-order)).
For a command acting on a record its input names:

1. input refusals answer first: a `when:` with an `error:` and no subject is checked before the
   record is looked up, and of two that one input selects, the first declared answers;
2. then existence: an identity no record carries takes the unknown instance answer, an
   `unknown_instance:` branch or the declared not-found refusal;
3. then the held state: `when_subject_state:` and `when_subject:` select by it, and `wrong_state`
   answers where the branch selected moves from a state its move does not start from;
4. then the accepting and external branches, in declaration order.

A `when_related:` guard adds steps of its own; the design note lists them.

An input refusal that holds only for a live record is guarded by the held state as well. "A blank
reason code is refused, but only on an active session: an unknown session is not found, and a paused
one is not active" is `when_subject: {predicate: state == Active}` beside the refusal's `when:`.
That makes it a held-state branch, answered after existence and, in any other state, after
`wrong_state`:

```yaml
format: ess/18
system: demo
version: v1
domain: demo.session
summary: A session paused with a reason code; the code is checked only on a live session.
types:
  - {name: demo.session.SessionId, kind: newtype, of: Uuid}
entities:
  - name: demo.session.Session
    identity: {name: session_id, type: demo.session.SessionId}
    fields:
      - {name: code, type: String}
    lifecycle:
      initial: Active
      states: [Active, Paused]
      terminal: [Paused]
      transitions:
        - {name: pause, from: [Active], to: Paused}
actors:
  - name: demo.session.Agent
    may: [demo.session.Start, demo.session.Pause]
errors:
  - {name: demo.session.InvalidCode, fields: []}
  - {name: demo.session.NoSession, fields: []}
  - {name: demo.session.NotActive, fields: []}
events:
  - name: demo.session.Started
    fields: [{name: session_id, type: demo.session.SessionId}]
  - name: demo.session.Paused
    fields: [{name: session_id, type: demo.session.SessionId}]
commands:
  - name: demo.session.Start
    input: []
    outcomes:
      - name: started
        creates: demo.session.Session
        instance: session_id
        sets: {code: ""}
        emits: [demo.session.Started]
        payload: {demo.session.Started: {session_id: {generated: true}}}
  - name: demo.session.Pause
    input:
      - {name: session_id, type: demo.session.SessionId}
      - {name: code, type: String}
    outcomes:
      - {name: no-session, unknown_instance: true, error: demo.session.NoSession}
      - name: invalid-code              # checked only on a live session
        when_subject: {predicate: state == Active}
        when: code == ""
        error: demo.session.InvalidCode
      - name: paused
        moves: demo.session.Session.pause
        instance: session_id
        sets: {code: input.code}
        emits: [demo.session.Paused]
        payload: {demo.session.Paused: {session_id: input.session_id}}
      - {name: not-active, wrong_state: true, error: demo.session.NotActive}
views:
  - name: demo.session.Sessions
    source: demo.session.Session
    consistency: read_your_writes
    fields:
      - {name: session_id, type: demo.session.SessionId}
      - {name: state, type: demo.session.Session.State}
      - {name: code, type: String}
components:
  - component: sessions
    owns: {domains: [demo.session]}
    accepts: {commands: [demo.session.Start, demo.session.Pause]}
    reached_by: network
```

The synthesized suite sends the blank code to an identity nothing stores and requires
`no-session`, to a paused session and requires `not-active`, and to an active one and requires
`invalid-code`. A target that checks the code before it looks the session up fails the first two.
`when_subject_state: Active` beside the `when:` would read the same, but it is refused beside
`wrong_state:` (`ESS-COMMAND-004`); write the predicate form.

### Two input refusals whose guards overlap

Declaration order is the declared precedence. Of two input refusals whose guards both hold,
the first declared answers:

```yaml
- {name: invalid-code, when: code == "", error: demo.hold.InvalidCode}
- {name: pause-refused, when: pause == true, error: demo.hold.PauseRefused}
```

`{code: "", pause: true}` takes `invalid-code`. Validation does not refuse the overlap: a text or
number guard needs an overlap analysis it does not have. Write the refusal that should win first,
or make the guards disjoint with `all:` or `not`, as in `{all: [pause == true, code != ""]}`.
The synthesized suite witnesses `invalid-code` with
`{code: "", pause: false}`, outside the later guard, and the overlap is sent once more, on its own,
requiring `invalid-code`. A target that checks `pause` first fails only that send. Where no input
separates the two guards, the overlap is the one witness.

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

## A rule stored as data is evaluated by the system

Some systems store their rules as rows, written at run time: a condition's expression, operator and
expectation, and a fold of ALL or ANY. ESS does not evaluate such a rule. The specification states
the rule's parts as typed rows and leaves the verdict to the system's evaluator, as an `external:`
outcome the suite injects:

```yaml
format: ess/16
system: demo
version: v1
domain: demo.rules
summary: Conditions stored as rows; whether a rule holds is the evaluator's decision.
types:
  - {name: demo.rules.ConditionId, kind: newtype, of: Uuid}
  - {name: demo.rules.RuleId, kind: newtype, of: Uuid}
  - {name: demo.rules.Operator, kind: enum, variants: [Equals, GreaterThan, Matches]}
  - {name: demo.rules.Fold, kind: enum, variants: [All, Any]}
entities:
  - name: demo.rules.Condition
    identity: {name: condition_id, type: demo.rules.ConditionId}
    fields:
      - {name: rule_id, type: demo.rules.RuleId}
      - {name: expression, type: String}
      - {name: operator, type: demo.rules.Operator}
      - {name: expectation, type: String}
      - {name: fold, type: demo.rules.Fold}
    lifecycle: {initial: Active, states: [Active], terminal: [Active]}
errors:
  - {name: demo.rules.RuleNotMet, fields: [{name: rule_id, type: demo.rules.RuleId}]}
events:
  - name: demo.rules.ConditionAdded
    fields: [{name: condition_id, type: demo.rules.ConditionId}]
  - name: demo.rules.RuleMet
    fields: [{name: rule_id, type: demo.rules.RuleId}]
commands:
  - name: demo.rules.AddCondition
    input:
      - {name: condition_id, type: demo.rules.ConditionId}
      - {name: rule_id, type: demo.rules.RuleId}
      - {name: expression, type: String}
      - {name: operator, type: demo.rules.Operator}
      - {name: expectation, type: String}
      - {name: fold, type: demo.rules.Fold}
    outcomes:
      - name: added
        creates: demo.rules.Condition
        instance: condition_id
        sets:
          rule_id: input.rule_id
          expression: input.expression
          operator: input.operator
          expectation: input.expectation
          fold: input.fold
        emits: [demo.rules.ConditionAdded]
        payload: {demo.rules.ConditionAdded: {condition_id: input.condition_id}}
  - name: demo.rules.TestRule
    input: [{name: rule_id, type: demo.rules.RuleId}]
    outcomes:
      - name: not-met
        external: the evaluator finds the rule's stored conditions do not hold
        error: demo.rules.RuleNotMet
      - name: met
        emits: [demo.rules.RuleMet]
        payload: {demo.rules.RuleMet: {rule_id: input.rule_id}}
components:
  - component: rules
    owns: {domains: [demo.rules]}
    accepts: {commands: [demo.rules.AddCondition, demo.rules.TestRule]}
    reached_by: network
```

The synthesized suite forces `not-met` and requires its error, and sends `TestRule` with no verdict
forced for `met`. Where the system stores each condition's result as a row field, such as
`holds: Boolean`, the ALL/ANY fold over those results is a fact the specification can state: from
`ess/22`, a `when_related` row-set guard with `forall` or `exists`. The boundary, and why ESS does
not interpret stored rules, is in
[the stored-rules design note](https://github.com/beyond10x/ess/blob/main/docs/design/stored-rules-boundary.md).

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
