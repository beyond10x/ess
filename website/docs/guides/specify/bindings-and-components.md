---
title: Components and bindings
sidebar_position: 6
description: The component, binding and topology layers; binding failure, retries, constants, accessors and stored reads in the invoked command.
---

# Components and bindings

## Three layers above the domains

A domain says what the software *means*. Three further layers say how it is put together, kept apart
because conflating them is how a domain model turns into a description of a deployment:

| Layer | Says | Does not say |
|---|---|---|
| **component** | `invoice-service` owns `billing.invoice` and accepts four commands | whether it is a process or a module; which protocol it speaks |
| **binding** | `InvoiceCreated` causes `SendEmail` | which queue carries it |
| **topology** | the system is not correct with one instance | how many pods to start |

`reached_by:` is a closed set of `in_process` (the default), `network` and `command_line`. It
declares where a component's callers reach its surface. `examples/billing/` omits it and gets
`in_process`; `examples/gatepass/components.yaml` declares `network`, which selects HTTP server
generation in the current Rust and Go targets. Projecting an OpenAPI document alone does not run
a server.

For `command_line`, supply a `cli:` block naming the binary and placing every accepted command
exactly once at the root or in a group. Command words derive from their wire names, and flags from
input fields and their wire names. A CLI block without `command_line`, or `command_line` without a
CLI block, is refused. Reach and CLI layout belong to the authored/compiled model and participate
in its identity; physical argv or URL invocation belongs to realization.

Each logical component currently has one reach value. Several physical entrypoint records do not
add simultaneous semantic CLI and HTTP surfaces, and duplicating domain ownership is refused.
See [Logical, interface and delivery owners](../../concepts/ess.md#logical-interface-and-delivery-owners)
for the separate contracts, identity consequences and bounded examples.

## A binding says what happens when it fails

Bindings live above the domains, in `components.yaml`:

```yaml
bindings:
  - id: notify-on-invoice-created
    when:
      event: billing.invoice.InvoiceCreated
    invoke:
      command: billing.email.SendEmail
    mapping:
      recipient: event.customer_email
      template: invoice-created
    delivery: at_least_once
    on_failure:                   # retry | drop | escalate
      escalate:
        emits: billing.email.DeliveryEscalated
```

`delivery:` and `on_failure:` are required words, not defaults — a binding that can fail silently is
the difference between specifying a system and specifying a demo. `drop` is legal: losing work is a
decision, and the decision has to be findable in the document that made it. `escalate` must name a
declared event, because "surface it to a person" is not something a generated test can observe.

`delivery:` accepts two values, and they are not two points on one scale — they say which side of
the invocation carries the risk.

| word | what it promises | what it costs |
|---|---|---|
| `at_least_once` | the command may run more than once for one event | the handler must be idempotent, and the generated OpenAPI document makes `Idempotency-Key` required on that command |
| `at_most_once` | one attempt, and nothing redelivers it | the handler is owed no repeat, and a lost attempt is `on_failure:`'s to answer |

Neither is "exactly once", and nothing here will ever spell that word: "exactly once" is what
everyone believes they have until a retry proves otherwise. Write `at_most_once` where the
invocation really is a single attempt — one HTTP call with no retry, or one whose response nobody
reads — because a specification claiming the stronger guarantee is a claim the system does not
keep. A conformance suite for an `at_most_once` binding contains no redelivery scenario, since
redelivery is the thing that word says will not happen.

## Fill a command input with a constant

A `mapping:` value without the `event.` prefix is a constant the binding writes, and it is typed
against the input it fills as `sets:` and `payload:` type a literal:

```yaml
mapping:
  leg_id: event.leg_id
  is_bridged: true        # a Boolean input
  weight: 3               # an Integer input, or a newtype of one
  share: 0.5              # a Decimal input
  template: invoice-created
```

`is_bridged: true` and `is_bridged: 'true'` are the same value: an unquoted YAML Boolean, integer
or decimal and the quoted text of one are both admitted exactly where `sets:` admits them, and
compile to the same IR. A constant that is not a value of the input is a `type_mismatch` with the
hint `sets:` gives — `weight: true` over an `Integer`, `is_bridged: 1` over a `Boolean`. Over text
or an enum the hint is `quote it`: `template: 3` meant the text `3`, and `template: '3'` says so.
Every target reads the constant typed: the conformance suite expects `true` and `3` in the invoked
command's input, not `"true"` and `"3"`, and the generated Rust and Go adapters pass a typed
constant. Other primitives — a `Timestamp`, a `Uuid` — have no literal spelling here, and take
their value from a field of the event.

## Bound a retry

`on_failure: retry` says nothing about how often. Where the count is a constant in the sender's
code rather than a deployment setting, state it (`format: ess/16`):

```yaml
on_failure:
  retry: {attempts: 3, final: [demo.ledger.Unknown]}
```

`attempts` counts invocations including the first, and is at least 2 — one attempt is `drop`.
`final` names refusals of the invoked command that end the retry at once: an outcome that carries
`error:`, by name, or the error itself. Any other failure is retried up to the bound, and after the
last attempt the event's effect is lost. The conformance suite forces a retried `external:` refusal
on every attempt and requires exactly `attempts` invocations, and forces a final one once and
requires exactly one. The generated Rust, Go and Web targets refuse a bounded retry by name,
because their retry counts no attempts.

## Choose the policy per refusal

`ess/22` lets one binding answer different refusals of its command differently. Key the policies,
and say which refusals each one takes:

```yaml
on_failure:
  drop: [wrong-state]
  retry: {outcomes: [demo.ledger.Unavailable, rejected], attempts: 3, final: [rejected]}
  escalate:
    emits: demo.ledger.RecordEscalated
    except: [wrong-state, demo.ledger.Unavailable, rejected]
```

Each of `drop`, `retry` and `escalate` appears at most once, with exactly one selector:
`outcomes: [...]`, or `except: [...]`. `drop` and an unbounded `retry` may write the list alone, as
`drop` does above. `escalate` always writes a block with `emits:`. `retry` keeps `attempts:` and
`final:`, and `final` needs `attempts:`.

Exactly one policy writes `except:`. It is the explicit fallback: it takes every refusal the other
policies do not, and every failure of an invoked command that carries no declared outcome, such as
a transport error. `except: []` takes every refusal.

A name selects as `final` does: an outcome that carries `error:`, by name, or the error, which
stands for every outcome that reports it. Above, `demo.ledger.Unavailable` selects both
`unavailable` and `busy`. A word such as `wrong_state` means nothing here unless it is the
command's own outcome name. Names are resolved first. Then every refusal of the command must have
exactly one policy.

At run time, the actual answer of each attempt chooses the policy:

| the attempt is answered | the binding |
|---|---|
| by an accepting outcome | is done |
| by a refusal under `drop` | stops; the work is lost |
| by a refusal under `escalate` | publishes the escalation event once from the attempt's actual input, and stops |
| by a refusal under a bounded `retry` | stops on a `final` refusal or once `attempts` invocations were made in all; otherwise tries again |
| with no declared outcome | does what the fallback does |

The count is the total for the occurrence. Changing from one refusal to another never restarts it.
A failure before the command could be invoked, such as an input that cannot be converted, is an
unmet obligation: no attempt, no retry and no escalation. A binding condition that does not hold is
still a skip with zero invocations.

The rules:

- Below `ess/22`, the selected shape is refused, naming `ess/22`. A universal policy keeps its
  meaning and its bytes under every format.
- A policy with both selectors, a policy without one beside one that has one, an empty `outcomes`
  list, and an `escalate` without `emits:` are refused.
- No `except:`, or two, is refused.
- A name that is no refusal of the invoked command, or that names an accepting outcome, is refused.
- A refusal two policies select, or that one selects and the fallback also takes, is refused, even
  when two names select it under one policy. A refusal the fallback leaves out and no policy takes
  is refused.
- A `final` refusal the retry itself does not select is refused.

Conformance witnesses each refusal on its own scenario, `<binding>/binding/refusal/<outcome>`: the
refusal is forced on an `external:` branch, and the scenario requires the attempt count its policy
owes, the escalation event published exactly once where it escalates (`expect_publication_count`),
and, where it does not, that the escalation event is published no times for the whole window
(`expect_no_publication`). An unbounded retry is witnessed as the universal `retry` is, through the
same arrangement of the row the command addresses. A refusal no scenario can force
is refused by name. These scenarios need suite/36 or /37. The generated Rust, Go and Web targets
and the scenario player cannot select a policy per refusal yet, so they refuse such a binding by
name (`bindings.<name>.on_failure`).

## Read a field inside an event envelope

The `ess/3` format adds bounded binding accessors. Set `format: ess/3`
in the specification header, then use two or three field segments after `event`:

```yaml
mapping:
  status: event.data.status
  text: event.data.body.text
```

Every segment names a declared field. For example, if `status` has
`wire: upstream_status`, the mapping still spells `event.data.status`; the
generated adapter uses the wire name when reading a serialized payload. Existing
single-field mappings such as `event.customer_email` keep their meaning in every
supported source format. Formats `ess/1` and `ess/2` refuse the new paths.

A path can pass through structs and newtypes. Traversing an `Optional` or a union
can leave the path unavailable: the Optional is absent, or the selected union
variant does not declare that field. Such a mapping requires an Optional command
input. A miss produces an absent input; a malformed union payload remains an
error. At least one union branch must contain the complete path, and all branches
that reach it must agree on its leaf type.

The value at the end is copied whole, including a struct, collection or Optional.
Reading `event.data.last_reason`, where `last_reason` is `Optional<String>`, differs
from traversing that Optional to reach another field. Exact source/target type
matches preserve the value; supported Optional lifting adds the required outer
presence layer. Declared conversions still govern other type crossings.

Accessors do not index or search lists or maps, compute values, or read session
context. A recipient identifier absent from the event needs its own declared
authority; adding it to an event that never carries it would misdescribe the wire.
For execution and observation limits, see
[Verify conformance](../verify/observations.md#observe-bounded-binding-accessors).

## Read stored state in the command a binding invokes

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

A `mapping:` value written as `{related: …}` is refused as an unknown field of a selection.

## Selections, periodic causes and delivery context

What a binding reads besides its event payload, and how a clock reading keeps its provenance, is on
[Selections, periodic causes and delivery context](binding-context.md). Each section moved there:

- <a id="select-ordered-records-in-a-binding"></a>[Select ordered records in a binding](binding-context.md#select-ordered-records-in-a-binding)
- <a id="declare-a-periodic-host-cause"></a>[Declare a periodic host cause](binding-context.md#declare-a-periodic-host-cause)
- <a id="read-the-channel-an-event-arrived-on"></a>[Read the channel an event arrived on](binding-context.md#read-the-channel-an-event-arrived-on)
- <a id="preserve-clock-reading-provenance"></a>[Preserve clock-reading provenance](binding-context.md#preserve-clock-reading-provenance)

## Conversions and wire names

How a value crosses from one context's type into another's, and how names and absent values are
spelled on the wire, is on [Conversions and wire names](wire-names.md). Each section moved there:

- <a id="crossing-contexts-takes-a-declared-conversion"></a>[Crossing contexts takes a declared conversion](wire-names.md#crossing-contexts-takes-a-declared-conversion)
- <a id="an-enum-variant-can-carry-its-own-wire-spelling"></a>[An enum variant can carry its own wire spelling](wire-names.md#an-enum-variant-can-carry-its-own-wire-spelling)
- <a id="a-field-can-carry-its-own-wire-name"></a>[A field can carry its own wire name](wire-names.md#a-field-can-carry-its-own-wire-name)
- <a id="an-error-can-carry-its-own-wire-code"></a>[An error can carry its own wire code](wire-names.md#an-error-can-carry-its-own-wire-code)
- <a id="say-whether-an-absent-optional-is-sent-as-null"></a>[Say whether an absent Optional is sent as null](wire-names.md#say-whether-an-absent-optional-is-sent-as-null)
