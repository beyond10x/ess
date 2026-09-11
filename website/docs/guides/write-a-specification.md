---
title: Write a specification
sidebar_position: 4
description: Author an ESS document — the layout, the constructs the model insists on, and the validation errors that teach the model fastest.
---

# Write a specification

This guide covers authoring an Executable System Specification. The normative example is
`examples/billing/` in the repository — deliberately the smallest system that exercises the
current `0.13.2` model. Concepts are covered in [ESS](../concepts/ess.md); this page is about writing
one.

## Layout

```text
system.yaml            format version, the system's name, which domains it has
domains/invoice.yaml   one bounded context: types, entities, commands, events, errors, views
domains/email.yaml     a second, so cross-domain references are real rather than assumed
components.yaml        which component owns which domain, the bindings between them, conversions
topology.yaml          what the system needs at runtime to be correct
```

Those five files are `examples/billing/`, and the `5 file(s)` in every validate line below is them.

One file works too: `ess specify validate --path spec.yaml` reads a single file carrying the header
and the members, and splitting into a directory later changes nothing about invocation. A file names
at most one `domain:`, so the one-file form is for a one-domain system; `components:`, `bindings:`
and `topology:` can sit in it as well.

The header's `domains:` list is checked in both directions, and both refusals name the fix:

```text
- [undeclared_reference] system.domains: `tiny.audit` is listed as a domain of the system, and no source declares it (hint: declared domains: tiny.core)
- [conflicting_declaration] domain tiny.core: `tiny.core` is declared, and the system header does not list it (hint: the header's `domains:` says what the system has; add it there, or drop the source that declares it)
```

Point your editor at `schemas/generated/ess.schema.json` and field names are checked as you type.
The schema is generated from the same Rust types the validator runs. The repository's authoritative
offline gate is `task check`, which exercises the schema contract alongside the workspace.

## Keep sources and generated output together

Current source supports this configuration; it is unreleased. For a mixed directory, add an immediate
`ess-inputs.yaml` and pass that directory:

```yaml
format: ess-inputs/1
specification:
  - model/system.yaml
  - model/domains/invoice.yaml
  - model/domains/email.yaml
  - model/components.yaml
  - model/topology.yaml
scenarios:
  - authored/routing/first.yaml
  - authored/e2e/second.scenario
```

```text
ess-inputs.yaml
model/                  the five billing model files above
authored/               the two explicitly listed scenario files
generated/openapi/      unlisted generated YAML
generated/asyncapi/     unlisted generated YAML
output/                 unlisted compiler and runner output
```

`ess specify validate --path .` reads only the `specification` entries. Nested or renamed
headers work, and headerless fragments retain their existing meaning. The selected files must
still assemble into one valid specification. Unlisted generated files, including malformed YAML
or copies of model fragments, are never scanned. List order does not affect selection.

Both lists are required and checked for valid, distinct relative paths, even when one role is
inactive. Only files in the active role must exist. An empty active list refuses. Paths are literal,
case-sensitive UTF-8 identities: no empty or dot segments, backslashes, colons or control characters.
The selected root, manifest, listed files and intermediate directories below the root must not be
symlinks. Use real contained files. The manifest declares selection, not file ownership or authorship.

Without an immediate manifest, existing recursive model discovery still requires `system.yaml`
and reads every lowercase `.yaml`/`.yml` below the directory. Use a separate model input directory
for that layout. An explicit file is always one source, independent of its extension; it never
searches its parent for configuration. A malformed or unsupported `ess-inputs.yaml` refuses without
falling back: rename a legacy source with that reserved filename or adopt this configuration.
See the [complete input format](../reference/formats.md#directory-input-configuration).

## Validate early, read the refusals

The unreleased `ess/2` format adds `Binary64` for finite IEEE-754 values. Use it
when a source contract requires binary floating-point rounding and signed zero:

```yaml
format: ess/2
system: sample
version: v1
domains: [sample.settings]
domain: sample.settings
types:
  - name: sample.settings.Ratio
    kind: newtype
    of: Binary64
```

`Integer` retains exact signed integer identity; `Decimal` retains its patterned
string representation. Neither implicitly assigns to `Binary64`. Map keys cannot
be Binary64. Format 1 refuses the new primitive, including fields in headerless
fragments. Authored numeric predicates may compare Binary64 to numeric literals,
using the existing Number predicate rules; that comparison does not construct a
floating value.

The qualified executable boundary is [format-5 normalization](generate-artifacts.md).
Standalone structural Rust/Go codecs and whole-system/conformance targets currently
refuse Binary64; TypeScript structural output reports the finite codec obligation.
Adding a Binary64 type to a model selected in full therefore requires checking
every intended target's support before adopting it.

```shell-session
$ ess specify validate --path examples/billing
billing v3 — 5 file(s), valid
```

Break a reference and the refusal names what was available. Take a copy of `examples/billing/`,
misspell `InvoiceCreated` as `InvoiceRaised` in the `accepted` branch's `emits:` and `payload:` —
those two occurrences only, not every one in the file — and run it:

```shell-session
$ COPY=$(mktemp -d)/billing && cp -r examples/billing "$COPY"
$ # in $COPY/domains/invoice.yaml, rename `InvoiceCreated` to `InvoiceRaised` in the
$ # `accepted` outcome's `emits:` list and its `payload:` key
$ ess specify validate --path "$COPY"
…/billing was refused:
  - [undeclared_reference] command.billing.invoice.CreateInvoice.outcomes.accepted.emits: `billing.invoice.InvoiceRaised` is not a declared event (hint: declared events: `billing.email.DeliveryEscalated`, `billing.email.EmailSent`, `billing.invoice.InvoiceCancelled`, `billing.invoice.InvoiceCreated`, `billing.invoice.InvoiceIssued`, `billing.invoice.InvoicePaid`)
  - [undeclared_reference] command.billing.invoice.CreateInvoice.outcomes.accepted.instance: outcome `accepted` of `billing.invoice.CreateInvoice` acts on the instance named by `invoice_id`, which is no field of an emitted event of it (hint: the field of an emitted event must be typed `billing.invoice.InvoiceId` — declared: none are declared)
  - error[ESS-COMMAND-001]: `billing.invoice.InvoiceRaised` is not a declared event
  … structured diagnostics continue with source locations and repair hints …
```

Every problem is reported in one run — one typo, two consequences, both stated, and the exit code is
`1`. The second is the more useful of the two: the branch says it creates an invoice and publishes
its identity in an emitted event, and the misspelling took away the event that was carrying it.

## What the model insists on

These are the authoring decisions that surprise people coming from OpenAPI-first or prose designs.
Each exists to keep a generated test honest. Every block below is an excerpt of
`examples/billing/`, abridged to the construct being explained.

### A command that can be refused says so

Not an `emits` list — **outcomes**. From `domains/invoice.yaml`:

```yaml
outcomes:
  - name: accepted
    when: amount.amount > 0
    creates: billing.invoice.Invoice
    instance: invoice_id
    emits:
      - billing.invoice.InvoiceCreated

  - name: rejected
    error: billing.invoice.InvalidAmount
```

A command with a precondition has at least two results. A specification recording only the happy one
generates a suite that never checks the branch where the money does not move.

### Cover every declared enum value

Current unreleased source accepts a command without a default when its input guards
select exactly one outcome for every value of a required, closed enum. For example,
if `status` has the declared variants `Ready` and `Stopped`, the two guards
`status == Ready` and `status == Stopped` cover that input. Synthesis uses the same
declared domain to construct a witness for each reachable branch. An omitted value
or overlapping guards produce a concrete failing assignment.

This proof is bounded to 64 joint assignments and 128 predicate nodes. Required
enum fields, transparent wrappers, equality, membership and supported Boolean
combinations participate. Every referenced input must fit that finite domain;
Optional paths, open types, unsupported expressions and unknown results retain the
requirement for a default. Existing defaults and external outcomes keep their behavior.

### Select an outcome from the held subject state

Unreleased `ess/3` allows `when_subject_state` beside an ordinary input predicate:

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

### An outcome the input cannot decide says that too

Whether a mail provider accepts an address is not a function of the request. From
`domains/email.yaml`:

```yaml
- name: failed
  external: the provider rejects the recipient address
  error: billing.email.Undeliverable
```

Writing `when: false` would claim the branch is unreachable — a different statement, and a false
one. A generator reads `external` and injects a fault instead of trying to construct an input.

### Illegal lifecycle moves are illegal by absence

`Paid` cannot become `Cancelled` because no transition says it can. There is no forbidding rule,
because a rule would be a second place for the same truth to live, and two places eventually
disagree. The generated documentation lists the absent pairs, derived from the same transitions.

### A command says what it answers when invoked in the wrong state

One key and one error name — everything else is derived:

```yaml
- name: issued
  moves: billing.invoice.Invoice.issue      # `issue` runs from [Draft]
  instance: invoice_id
  emits:
    - billing.invoice.InvoiceIssued

- name: wrong-state
  wrong_state: true
  error: billing.invoice.InvoiceStateConflict
```

`wrong_state:` names no state: `issue` already declares it runs from `Draft`, so the refused states
are derived. The `error:` is required — without it a generated scenario could only assert that
*nothing happened*, which also passes against an implementation refusing for the wrong reason.

### An event's values need a declared source

`emits:` declares which facts a branch announces; `payload:` declares what fills their fields:

```yaml
- name: accepted
  when: amount.amount > 0
  creates: billing.invoice.Invoice
  instance: invoice_id
  emits:
    - billing.invoice.InvoiceCreated
  payload:
    billing.invoice.InvoiceCreated:
      account_id: input.account_id
      customer_email: input.customer_email
      amount: input.amount
```

Without this, an implementation announcing an amount nobody submitted contradicts nothing. In source formats 1–3, the block
is optional per field: `invoice_id` has no line because the identity is the implementation's to assign.

Source `ess/4` requires every field of each emitted event to have a mapping. Use
`invoice_id: {generated: true}` for an explicitly implementation-generated identity.
Events with no emitting outcome remain valid; they may have an external producer.

A command may declare a closed typed `response` record and map one of its fields with
`item: {response: item}`. This reads the returned response; the string `response.item`
retains its historical literal meaning. Input mappings keep their existing spelling.
Missing fields, unknown response members and incompatible types are refused. Conformance
checks compare mapped values with the actual response from the same command invocation.

### A view declares its consistency

`consistency: eventual` on a view is what decides that a generated assertion is `eventually` rather
than immediate. Getting it wrong produces a suite that passes on a laptop and flakes in CI.

When several views expose the same row, declare the row once as a named struct and reference it with
`shape:`:

```yaml
types:
  - name: todo.list.ItemRow
    kind: struct
    fields:
      - name: item_id
        type: todo.list.ItemId
      - name: list_id
        type: todo.list.ListId
      - name: state
        type: todo.list.Item.State

views:
  - name: todo.list.ItemById
    source: todo.list.Item
    shape: todo.list.ItemRow
    consistency: read_your_writes

  - name: todo.list.ListItems
    source: todo.list.Item
    shape: todo.list.ItemRow
    consistency: read_your_writes
```

A view declares exactly one of `shape` or inline `fields`. The named type must be a struct, and ESS
still checks each of its fields against the source entity. Compiled IR carries both the shape handle
and the checked expansion; OpenAPI uses the handle as a real `$ref`, so the row schema is emitted
once rather than copied per view.

### A binding says what happens when it fails

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

### Read a field inside an event envelope

The unreleased `ess/3` format adds bounded binding accessors. Set `format: ess/3`
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
[Verify conformance](verify-conformance.md#observe-bounded-binding-accessors).

### Select ordered records in a binding

Unreleased `ess/3` adds binding-local `selection_inputs` and ordered `selections`.
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

### Declare a periodic host cause

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

### Preserve clock-reading provenance

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

### Crossing contexts takes a declared conversion

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
See [Logical, interface and delivery owners](../concepts/ess.md#logical-interface-and-delivery-owners)
for the separate contracts, identity consequences and bounded examples.

## Check what you just wrote resolved

`ess specify compile --path <specification> --format json` emits complete top-level `views`,
including each view's `source`, `fields`, `consistency` and `naming.wire`. A domain's `views`
list contains references into that map. Adapters should consume those declarations directly;
no second YAML reader is needed. `--out <file>` writes the same canonical JSON bytes.

`ess specify validate` says the document holds together. `ess specify inspect` shows what one
declaration *became*,
with every reference in it resolved — the fastest way to find out whether the thing you meant is the
thing the model read:

```shell-session
$ ess specify inspect --path examples/billing billing.invoice.CreateInvoice
commands:
  domain: billing.invoice
  input:
  - name: account_id
    type_ref:
      kind: declared
      name: billing.invoice.AccountId
  - name: customer_email
    type_ref:
      kind: declared
      name: billing.invoice.Email
  - name: amount
    type_ref:
      kind: declared
      name: billing.invoice.Money
  name: billing.invoice.CreateInvoice
  naming:
    display: Create invoice
    wire: create-invoice
  outcomes:
  - condition:
      kind: when
      predicate: amount.amount > 0
    name: accepted
    test_strategy: construct_input
  - condition:
      kind: otherwise
    error: billing.invoice.InvalidAmount
    name: rejected
    test_strategy: default_branch
```

`kind: otherwise` is the line to read: the specification names no condition for that branch, and the
model derived one. `construct_input` and `default_branch` say how a generated scenario will reach
each branch, before any suite exists. The actual YAML includes the resolved payload and subject
records as well; the excerpt above keeps only the fields relevant to that question.

On a binding it resolves the crossing as well, reason included:

```shell-session
$ ess specify inspect --path examples/billing notify-on-invoice-created
bindings:
  command: billing.email.SendEmail
  delivery: at_least_once
  escalation: billing.email.DeliveryEscalated
  event: billing.invoice.InvoiceCreated
  failure: escalate
  mapping:
  - conversion: An invoice's customer email is a deliverable address; the email context validates it again on the way out, so the invoice context does not have to know how.
    target: recipient
    target_type:
      name: billing.email.EmailAddress
    value:
      field: customer_email
      kind: event_field
      type_ref:
        name: billing.invoice.Email
  name: notify-on-invoice-created
```

`--kind` is only needed when one name is used in two namespaces; the seven it accepts are `domain`,
`type`, `command`, `event`, `error`, `binding` and `component`.

## Names

| Name | Example | Who reads it |
|---|---|---|
| qualified name | `billing.invoice.CreateInvoice` | the specification, and only it |
| wire name | `create-invoice` | HTTP paths, topics, generated JSON |
| display name | `Create invoice` | generated documentation, a UI |
| locator | `ep://acme/billing/ess-command/billing.invoice.CreateInvoice` | anything outside |

Conflating any two costs a rename later: an HTTP path that changes because someone improved a domain
term is an outage caused by a wording fix.

Field `wire` overrides change JSON property keys, not logical field identities.
Effective wire keys must be distinct within each object: structs, command inputs,
event/error payloads, entity identity/fields/state and view rows. View parameters have
their own namespace. An entity field or identity cannot use the reserved `state` wire
key. Validation accumulates collisions before any projection can overwrite a property;
display names and identical keys in separate objects do not conflict.

## Next

* [Verify an implementation](./verify-conformance.md) — generate the suite this specification
  obliges, run it, and turn the result into evidence.
* [A specification and its contracts](../examples/specification-to-contracts.md) — the billing
  example's source next to its generated output.
