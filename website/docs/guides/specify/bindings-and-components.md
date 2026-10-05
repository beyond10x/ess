---
title: Components and bindings
sidebar_position: 6
description: The component, binding and topology layers; binding failure, retries, accessors, selection, periodic causes and delivery context.
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

## Conversions and wire names

How a value crosses from one context's type into another's, and how names and absent values are
spelled on the wire, is on [Conversions and wire names](wire-names.md). Each section moved there:

- <a id="crossing-contexts-takes-a-declared-conversion"></a>[Crossing contexts takes a declared conversion](wire-names.md#crossing-contexts-takes-a-declared-conversion)
- <a id="an-enum-variant-can-carry-its-own-wire-spelling"></a>[An enum variant can carry its own wire spelling](wire-names.md#an-enum-variant-can-carry-its-own-wire-spelling)
- <a id="a-field-can-carry-its-own-wire-name"></a>[A field can carry its own wire name](wire-names.md#a-field-can-carry-its-own-wire-name)
- <a id="an-error-can-carry-its-own-wire-code"></a>[An error can carry its own wire code](wire-names.md#an-error-can-carry-its-own-wire-code)
- <a id="say-whether-an-absent-optional-is-sent-as-null"></a>[Say whether an absent Optional is sent as null](wire-names.md#say-whether-an-absent-optional-is-sent-as-null)
