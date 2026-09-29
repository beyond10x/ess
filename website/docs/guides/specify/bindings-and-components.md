---
title: Components, bindings and wire names
sidebar_position: 6
description: The component, binding and topology layers; binding failure, retries, accessors, selection, periodic causes and delivery context; conversions and wire spellings.
---

# Components, bindings and wire names

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
[Verify conformance](../verify/author-scenarios.md#observe-bounded-binding-accessors).

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
[Verify conformance](../verify/author-scenarios.md#deliver-an-event-with-its-context).

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

## Crossing contexts takes a declared conversion

A binding's `mapping:` is the one place two independently-written contexts must agree about a type,
so both sides are checked. `billing.invoice.Email` and `billing.email.EmailAddress` are distinct
newtypes, and the model refuses to treat one as the other unless you say so — with a reason:

```yaml
conversions:
  - from: billing.invoice.Email
    to: billing.email.EmailAddress
    because: >-
      An invoice's customer email is a deliverable address; the email context validates it again on
      the way out, so the invoice context does not have to know how.
```

`because:` is required, and conversions are directional — declaring `Email → EmailAddress` does not
grant the reverse, which is usually the unsafe one. The reason is not decoration: it is what
`ess specify inspect` prints back at the crossing, so the person reading the binding a year later reads the
argument for it rather than reconstructing one.

## An enum variant can carry its own wire spelling

`ess/5`, introduced in 0.27.0, lets a variant declare the name it is called on the wire. Until it
existed, `variants:` was a list of strings, so a variant whose wire form is not derivable from its
name could not be declared at all — the naming had to be written again in every target.

```yaml
types:
  - name: calls.recording.Action
    kind: enum
    variants:
      - name: Stop
        wire: ""
      - name: Flag
        wire: flag
      - Tags
```

Both forms are admitted in one list. A variant that declares nothing stays a bare name, and a
variant that declares something takes the `wire`, `display`, `summary` and `code` members every
other named thing already has. An authored empty string is a spelling rather than an absent one:
`Stop` is the route with no suffix, and falling back to the variant's name would spell the route
that does not exist.

A generated JSON Schema enumerates the wire spellings; the CLI contract keeps the authored names,
which are what an operator types. Formats `ess/1` … `ess/4` refuse a variant that declares naming
with `unsupported_format_version` at `types.<type>.variants.<variant>`, and a bare list is admitted
by every format, so nothing written before this moves. `ess verify diff` reports a moved spelling as
`VariantWireNameChanged` under [`ess-diff/5`](../../reference/formats.md#change-and-conformance-records).

## A field can carry its own wire name

A field of a struct, an event or a command input that travels under another name declares it, flat
or nested the way commands and events write their own naming:

```yaml
events:
  - name: demo.orders.Placed
    fields:
      - name: order_id
        type: demo.orders.OrderId
        naming: {wire: orderId}   # the same as `wire: orderId` on the field
```

Writing both spellings on one field is refused. The model keeps the declared name, so a payload
mapping and a predicate still say `order_id`, and JSON Schema, OpenAPI and AsyncAPI key the property
`orderId`. The field is written back flat, so the two spellings are one model with one digest.

## Say whether an absent Optional is sent as null

An `Optional<T>` field says how its absent value travels with `presence:` (`format: ess/15`):

```yaml
types:
  - name: demo.orders.OrderReceipt
    kind: struct
    fields:
      - {name: partner_ref, type: Optional<String>, presence: null_when_absent}
      - {name: discount_code, type: Optional<String>, presence: omitted_when_absent}
```

`null_when_absent` makes the JSON Schema property required and nullable; `omitted_when_absent`
keeps it optional and not nullable, which is what an `Optional` field without a policy already
publishes. A suite carries the policy on the field's payload leaf, so an implementation that sends
`null` for `discount_code` or leaves `partner_ref` out fails; such a suite is written as
`ess-conformance/24` (or `/25` with coverage), which the Go and TypeScript runners execute from 0.40.0.
`presence:` on a required field, or in a command's, event's or type's own `naming:`, is refused.
