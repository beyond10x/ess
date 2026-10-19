---
title: Binding conditions
sidebar_position: 6
description: Invoke a binding only when its event payload says so, and the expect_no_invocation scenarios that check it.
---

# Invoke only when the payload says so

`ess/22` lets an event binding carry a condition over the event payload. The binding invokes its
command only for an occurrence the condition holds for:

```yaml
when:
  event: example.messages.MessageReceived
  where: [defined(event.order), event.kind == ship]
invoke: {command: example.messages.MessageEvent}
mapping:
  message_id: event.message_id
  order_id: event.order.id
delivery: at_least_once
on_failure: retry
```

`where` reads only the event payload, as `event.<member>`, through one to three declared members
of structs and Optional structs. It admits `true`, `false`, `defined(...)`, `missing(...)`, `==`
and `!=` of a String or enum leaf against a literal, from `ess/27` `==` and `!=` of a Boolean
leaf against `true` or `false`, and `all`, `any` and `not` of those. An enum literal must name a
variant. Outcome names, command input, stored rows, the caller and the
delivery context cannot be read. Two branches that publish the same payload cannot be told apart
by a binding. Add the field its consumers need to the event.

The condition is evaluated before selections, conversions and mapping:

| it evaluates to | the binding |
|---|---|
| true | delivers as before |
| false | skips this occurrence: no invocation and no failure policy; other bindings on the event still run |
| unknown, because a comparison reads an absent Optional member | invokes nothing and reports an unmet obligation, never a silent skip |

A condition that proves an Optional member present lets a mapping read that member into a
required input. Above, `defined(event.order)` admits `event.order.id` into a required `String`.
The proof is taken from what the condition requires to be true: `defined`, and every member a
comparison reads. `any` proves only what all its sides prove. Proving a parent does not prove an
Optional member under it. Without enough proof, the mapping is refused as before.

The rules:

- Below `ess/22`, `where` is refused, naming `ess/22`.
- A periodic cause has no event and refuses `where`.
- A path the event does not declare, a union on the path, or a construct outside the list above
  is refused.

Conformance witnesses both sides. Every branch that publishes the event is a candidate trigger,
and a value it writes as a literal counts as written. A member it fills from its own input is
varied by sending another input. A member it fills from the row an input names,
`{related: {via: input.<field>, field: <field>}}`, is varied by creating that row with another
value through its creating command, which must store the field from its input unchanged and not
guard on it; the trigger must not guard on that row either. The flow, mapping, delivery and failure
scenarios publish a payload the condition holds for. Two more scenarios follow the event and
require zero invocations for the whole window:

- `condition-false` changes one compared value while every member is present. Where that is
  impossible on the same branch, it uses another branch.
- `condition-absent` publishes once for each Optional level the condition proves present, leaving
  that level out. For `defined(event.order.note)` that is one occurrence without the order and
  one with an order but no note.

A witness whose own bindings would publish the event again is refused by name, because zero
invocations could not be required of it. Each negative witness also requires every unconditioned
binding beside it on the event to invoke, since a condition skips its own binding alone. Each
scenario needs suite/36 or /37. For an event from an external channel the suite chooses the
delivered payload: one the condition holds for, one it fails for, and one per proved level left
out.

The generated Rust, Go and web targets evaluate the condition in their dispatch, before the
transformation and the invocation. An Unknown condition is that binding's unmet obligation and is
not retried. A required input copied or read from a member the condition proves present is checked
rather than unwrapped, in a selection binding too. A delivery-context field is never proved
present: the condition reads the payload only. `ess verify diff` reports an added,
changed or removed condition as `predicate-changed`, an `ess-diff/14` change.

## Compare a Boolean flag the event carries

From `ess/27` a flag on the event splits one event between bindings:

```yaml
when:
  event: example.calls.CallFinished
  where: [event.direction == inbound, event.bridged == false]
```

Write the comparison out. A bare `event.bridged` or `not event.bridged` is no comparison and is
refused, and `event.bridged == "false"` compares a Boolean with text and is refused as a type
mismatch. `condition-false` flips the flag. A selection's `where` compares a Boolean item leaf the
same way, `item.held == false`. Below `ess/27` both are refused, naming `ess/27`.

A condition reads only the event. A flag stored on another entity, such as an eligibility flag on
the role a user was assigned, is not in the payload. Read it in the invoked command with a
`when_related` guard over the row the event names, and let that command choose to do the work or
to skip it. See [read stored state in the command a binding invokes](bindings-and-components.md#read-stored-state-in-the-command-a-binding-invokes).
