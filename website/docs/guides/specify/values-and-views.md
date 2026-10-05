---
title: Values, credentials and views
sidebar_position: 5
description: Value expressions, the caller's credential, inputs present after defaulting, and view consistency and paging.
---

# Values, credentials and views

## Value expressions

Source `ess/14`, introduced in 0.36.0, lets a `payload:` or `sets:` value come from more than
the input or a literal:

```yaml
- name: retried
  updates: demo.orders.Order
  instance: order_id
  emits: [demo.orders.Retried]
  payload:
    demo.orders.Retried:
      order_id: input.order_id
      previous: {subject: retries}          # the stored value before this outcome
      seq: {input: seq, else: {generated: true}}
      ref:                                  # a struct, one source per field
        id: input.order_id
        label: {generated: true}
  sets:
    retries: {increment: 1}                 # previous value plus one
    stamp: {generated: true}                # the implementation decides
    discount: 0.0                           # a Decimal literal
```

| Source | Where | Admitted when |
|---|---|---|
| `{subject: <field>}` | `payload:`, `sets:` | the outcome `moves:` or `updates:` an existing subject; `creates:` has no row before it |
| `{increment: <number>}` | `sets:` | the target is a required `Integer` (a whole number) or `Decimal`; a negative number decrements |
| `{input: <field>, else: {generated: true}}` | `payload:`, `sets:` | the input is `Optional<…>` |
| `{input: <field>, else: <literal>}` | `payload:`, `sets:` | source `ess/16`; the input is `Optional<…>` and the literal is one the target admits |
| `{related: {via: <field>, field: <field>}}` | `payload:`, `sets:` | source `ess/16`; `via` is a field of an existing subject, or `input.<field>`, typed as exactly one entity's identity; from `ess/22` also `Optional<…>` of it, or a list of two references |
| `{related: {entity: <Entity>, where: <predicate>, field: <field>}}` | `payload:`, `sets:` | source `ess/22`; `field` of the one row of `<Entity>` that `where` selects, read before the outcome, at the field's declared type the target admits; none or several selected rows supply no value. `where` reads as a [row-set guard](../../reference/predicates.md#a-guard-over-the-rows-a-selector-selects)'s; below `ess/22` the same mapping is the nested mapping it always was |
| `{caller: <attribute>}` | `payload:`, `sets:` | source `ess/16`; every actor that may invoke the command declares the attribute, at one type the target admits |
| a nested mapping | `payload:`, `sets:` | the target is a struct; every struct field has a source |
| `{generated: true}` | `sets:` | always (`payload:` has admitted it since `ess/4`) |
| `{cleared: true}` | `sets:` | the target field is `Optional<…>`; refused in `payload:`. Synthesis asserts the field is empty in the view, so a target that keeps the old value fails |

A mapping is a source when every key is one of `response`, `generated`, `cleared`, `subject`,
`increment`, `input` and `else`; any other key makes it a nested mapping. The text
`subject.note` is refused in every format: it used to compile as the literal text
`"subject.note"`.

Synthesis asserts each value where the arrangement determined what it reads: the row the
arrangement wrote, the input the scenario sent. Where it did not, a payload field is checked for
presence and type only, and a `sets:` target is not asserted on the row. Entity Runtime lowering
refuses these sources.

From source `ess/16` a fallback can be a literal instead of `{generated: true}`:

```yaml
sets:
  tier: {input: tier, else: Standard}       # the caller's tier, or Standard when none is sent
  rank: {input: rank, else: 3}
```

The literal is checked against the target exactly as a literal written there is: a variant of the
enum, text for a String-backed type, `3` over an `Integer`, `'3'` rather than `3` over a `String`.
Nothing else follows `else:` — not `input.<field>`, not `{subject: …}`. Below `ess/16` a literal
fallback is refused with `unsupported_format_version`. The outcome's synthesized scenario checks
both halves: it first sends the input and asserts the sent value, then invokes the branch again
without the input and asserts the literal. An implementation that ignores the input fails the
first check, and one that stores another default fails the second.

From source `ess/22` a value can be read from a member of a struct input, and a fallback can be
another input:

```yaml
input:
  - {name: opening, type: leases.pool.Opening}             # struct {generation_id, label}
  - {name: previous, type: Optional<leases.pool.Opening>}
  - {name: settings, type: leases.pool.Settings}           # struct {defaults: {label}}
outcomes:
  - name: opened
    creates: leases.pool.Lease
    instance: lease_id
    sets:
      generation_id: input.opening.generation_id           # through a required struct
      previous_generation: input.previous.generation_id    # absent where `previous` is
      label: {input: previous.label, else: input.settings.defaults.label}
```

A path names declared fields: its first segment is an input, and each further segment a member of
the struct before it, through any `Optional` and newtype around that struct. A primitive, enum,
list, map or union has no members. The value is the last member's; an absent `Optional` anywhere
before it leaves the value absent, so such a path fills an `Optional<…>` target and is refused
with `type_mismatch` for a required one. `{input: <path>, else: …}` falls back wherever any
`Optional` on the path is absent. `else: input.<path>` reads an input that must be present
whenever the request is valid — required along its whole route — and fills the target as a plain
`input.` source would; there is no second fallback. A path may supply a creation's identity, or
the address another row is read through, only where nothing on it may be absent.

The same paths work in an event's `payload:`, an error's `payload:` and a nested mapping's
leaves. Synthesis sends every path present, then sends the branch again with the shallowest
`Optional` on each path left out, and asserts the absent value and the fallback. Below `ess/22`
each form keeps the refusal it had: `input.a.b` is an input the command does not declare, and
`{input: a.b, else: …}` names more than one field. Entity Runtime lowering refuses a path by name.

From source `ess/16` a value can come from a field of the row the subject references:

```yaml
- name: dispatched
  updates: demo.shipping.Shipment
  instance: shipment_id
  emits: [demo.shipping.ShipmentDispatched]
  payload:
    demo.shipping.ShipmentDispatched:
      shipment_id: input.shipment_id
      region: {related: {via: customer_id, field: region}}   # the region of the shipment's customer
```

`via` is a field of the subject as it was before the outcome (on `creates:`, a field the
branch sets from its input, or the identity it fills from its input), or `input.<field>`, and its
type is the identity of the entity it names — exactly, not `Optional<…>` or a list. Where several
entities share that identity type, the relation on the subject field says which one: a
`references` relation of cardinality `one` that the subject declares on it, or the `owns` relation
of the subject's owner; an input is settled by the relation on the field the branch sets from it,
or on the identity the branch names its instance by. The field may be the subject's identity: an
entity keyed by `user_id` that declares `{name: user, kind: references, target: User,
cardinality: one, via: user_id}` reads the user with the same id. `field` is a field of that
entity, typed as the target admits. `{related: …}` is written alone and holds exactly `via` and
`field`; any other mapping under `related` is a nested mapping, and below `ess/16` so is this one.

From source `ess/22` the reference may be `Optional<…>`, and `via` may name a second reference —
a field of the row the first one names — as a list of two:

```yaml
- name: booked
  creates: demo.costs.CostEntry
  instance: entry_id
  sets:
    objective_id: input.objective_id
    # the outcome of the initiative of the entry's objective; absent where the objective has none
    outcome_id: {related: {via: [objective_id, initiative_id], field: outcome_id}}
```

Each reference is resolved as `via` is: the relation on the field says which entity it names, or
the one entity identified by its type. Where any reference may be absent the value may be too,
so the target must be `Optional<…>`; a required target is refused with `type_mismatch`. An absent
reference reads no row and copies an absent value. A present reference that names no row is still
a missing row, never an absent value. Two references is the limit: a list of one or of three is
refused when the document is read. Below `ess/22` an `Optional<…>` reference is refused with
`type_mismatch` and a list with `unsupported_format_version`, both naming `ess/22`. A view still
reads one entity: group by the copied field rather than by a field of another entity.

The scenario creates the referenced row between two others of its entity, points the subject at
it, and asserts that row's value, so an implementation that reads another row, the first or the
last, fails. Where the specification has an `updates:` branch that changes the field read, the
scenario runs it on the referenced row just before the branch and asserts the new value, so an
implementation that copied the value earlier fails too. A chained read gets the same between-decoys
arrangement for each entity it passes through, and a branch that changes the middle row's reference
is run on it just before the branch, so an implementation following the reference as first written
fails. For each reference that may be absent, the scenario runs the branch once more with that
reference left out and every other present, and asserts the value absent on the row it writes and
on the event (left out or `null`), so an implementation that reads absence as a missing row, or
copies some row's value anyway, fails. A reference that may be absent and that no run can leave
absent is reported as `ESS-SYNTH-020`, naming it; the scenario stands.
Below `ess/16` the source is refused with `unsupported_format_version`, and Entity
Runtime lowering refuses it. Generated Rust and Go behaviour keeps a command with a `{related: …}`
value an obligation.

A literal over a `Decimal` target is admitted in every format, quoted (`'0.25'`) or unquoted
(`0.25`): an optional `-`, digits without a leading zero, optionally a point and digits.

## Read the caller's credential

From source `ess/16` an actor may declare `attributes:`: typed fields its credential carries, such as
the account it acts for. A command reads one as `{caller: <attribute>}` in `payload:` and `sets:`, and
as `caller.<attribute>` in a guard, compared with an input field in `when:` or with a stored field in
`when_subject:`:

```yaml
actors:
  - name: demo.notes.AccountUser
    attributes:
      - {name: account_id, type: demo.notes.AccountId}
      - {name: agent_id, type: demo.notes.AgentId}
    may: [demo.notes.CreateNote, demo.notes.EditNote]
commands:
  - name: demo.notes.CreateNote
    input:
      - {name: text, type: String}
    outcomes:
      - name: created
        creates: demo.notes.Note
        instance: note_id
        sets: {account_id: {caller: account_id}, agent_id: {caller: agent_id}, text: input.text}
  - name: demo.notes.EditNote
    input:
      - {name: note_id, type: demo.notes.NoteId}
      - {name: text, type: String}
    outcomes:
      - name: forbidden
        when_subject: {predicate: agent_id != caller.agent_id}   # not the note's agent
        error: demo.notes.NotYourNote
      - name: edited
        updates: demo.notes.Note
        instance: note_id
        sets: {text: input.text}
```

The attribute is not an input: the credential is its authority, not the request. It is readable
only where every actor whose `may` names the command declares it, at one type; a command whose
actors disagree, or that no actor may invoke, is refused where it reads the attribute. A guard
compares a caller attribute with `==` or `!=` against a field of the same declared type and nothing
else. An input or a stored field named `caller` keeps being read as itself. Below `ess/16` actor
attributes and a `caller.` operand are refused with `unsupported_format_version`, and
`{caller: …}` is the nested mapping it always was.

A refusal the caller decides answers `403` in the generated `OpenAPI` contract, and every operation
names the caller attributes it reads under `x-ess-caller`. The conformance suite (suite/26) says
which caller sends each command, and the target sends it authenticated as that caller; a target
that cannot answers `unsupported`. Synthesis uses two callers: the refusal is sent by one caller on
the other's note, and every scenario of a command that reads the caller runs a second time with the
two callers' roles swapped, so an implementation that records one fixed account or admits one
caller by name fails. Entity Runtime lowering refuses a caller read with `CallerUnsupported`.

## An input refused when absent is present afterwards

This section has its own page: [an input refused when absent is present afterwards](./narrowed-inputs.md).

## A view declares its consistency

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

## A view served from a replica

One service may accept writes at one central instance while every other environment reads from a
local replica that trails it. Model it as one component: its commands are the writes, and its views
are declared `consistency: eventual`, so the suite asserts their reads with `eventually`. A `role`
setting says which instance an environment runs, and the store the instances share is a `requires:`
entry on a `stateless: true` workload:

```yaml
types:
  - {name: directory.accounts.Role, kind: enum, variants: [Primary, Replica]}
views:
  - name: directory.accounts.Accounts
    source: directory.accounts.Account
    consistency: eventual                                # a replica may trail the write
    fields: [{name: account_id, type: Uuid}, {name: email, type: String}]
components:
  - component: directory                                 # one component, every environment
    owns: {domains: [directory.accounts]}
    settings:
      - {name: role, type: directory.accounts.Role, required: true}
      - {name: write-endpoint, type: Optional<String>}   # absent on the primary
topology:
  workloads:
    directory: {stateless: true, replicas: {min: 2}, requires: [{postgres: directory-store}]}
```

Placed beside a domain that declares `directory.accounts.Account`, this validates. Each
environment's values belong to its `ess-environment/1` document. Which instance is the
write-primary, and read-your-writes at a named instance, are out of scope for the specification: a
component is a logical boundary, not a placement, and a view declares one consistency. A workload
has no `environments:` key; two stateful instances with no shared store are `ESS-TOPOLOGY-004`; a
second component for the replica that owns nothing is `ESS-COMPONENT-007`.

## Who may read a view

From source `ess/22` an actor's `may:` names the views it may read as well as the commands it may
invoke. There is one grant table: a view named there is read-granted, and only the actors naming it
may read it.

```yaml
actors:
  - name: desk.tickets.Clerk
    may:
      - desk.tickets.OpenTicket
      - desk.tickets.Board
  - name: desk.tickets.Watcher
    may:
      - desk.tickets.OpenTicket
```

Here the Clerk may read `Board` and the Watcher may not. A view no actor names stays open to every
caller, so a specification that names no view means what it meant before. A grant naming something
that is neither a command nor a view is refused as `undeclared_reference`. Under `ess/21` and
earlier a grant naming a view is refused with `unsupported_format_version`, naming `ess/22`. A
served component refuses a read the grant does not admit with the same `403` it answers an
ungranted command; see [synthesis](../synthesize.md).

## A view can be paged

A list endpoint that answers one page of its rows at a time declares `paging:` beside its
`order_by:`. It needs `format: ess/16`.

```yaml
views:
  - name: demo.jobs.JobList
    source: demo.jobs.Job
    consistency: read_your_writes
    params:
      - {name: type, type: Optional<demo.jobs.JobType>}
      - {name: page, type: Integer}
      - {name: size, type: Integer}
    filter: type == param.type
    order_by: [job_id asc]
    paging: {page: page, size: size, total: true}
    fields:
      - {name: job_id, type: demo.jobs.JobId}
      - {name: type, type: demo.jobs.JobType}
```

`page` and `size` name two declared parameters, each `Integer`, a newtype of it or
`Optional<Integer>`. A paged read answers `size` rows of the rows the filter admits, in `order_by:`
order, starting at `page * size`; `first_page: 1` numbers pages from 1, so the first row is at
`(page - 1) * size`. With `total: true` the answer also carries how many rows the filter admits. A
read that sends neither parameter answers every row, in order. A filter may not read a paging
parameter, and a view without `order_by:` cannot be paged: a slice of an unordered view names no
particular rows. Below `ess/16`, `paging:` is refused with `unsupported_format_version`.

The synthesized suite arranges four rows and reads two pages of one row, two pages of two rows,
and one page larger than all of them, which must hold at least those rows: each page holds exactly
its size, the total is at least the rows the scenario made, and the second page
continues the first — ranked no earlier, and not the same row. Each claim holds on a target other
users share. A free-form filter expression the caller supplies is not something `paging:` or
`params:` can declare; a closed set of filter fields can still be declared one optional parameter
at a time. [Design](https://github.com/beyond10x/ess/blob/main/docs/design/view-paging.md).

## A view field derived from the lifecycle state

A status read often answers a value that follows from the lifecycle state: `live`, true in every
state but `Closed`. A view field names a field of its source entity, so declare the value as a
stored field, write it with `sets:` on every branch that enters a state with a different value, and
state the mapping as invariants over `state`. The view projects the field:

```yaml
domain: desk.tickets
entities:
  - name: desk.tickets.Ticket
    identity: {name: ticket_id, type: Uuid}
    fields: [{name: live, type: Boolean}]          # the stored field
    invariants:                                    # one value per state, checked after every branch
      - {any: [state != Closed, live == false]}
      - {any: [state == Closed, live == true]}
    lifecycle:
      initial: Open
      states: [Open, Waiting, Closed]
      terminal: [Closed]
      transitions:
        - {name: wait, from: [Open], to: Waiting}
        - {name: close, from: [Open, Waiting], to: Closed}
views:
  - name: desk.tickets.Status
    source: desk.tickets.Ticket
    consistency: read_your_writes
    fields: [{name: ticket_id, type: Uuid}, {name: live, type: Boolean}]
```

The branch that creates a ticket writes `sets: {live: true}`, the `close` outcome writes
`sets: {live: false}`, and `wait` writes nothing because the value does not change. Synthesis checks
the invariants after every outcome, so the mapping is checked in every state some branch enters. A
branch that forgets its `sets:` still validates; the model's interpreter reports a broken invariant,
naming the one over `state`, when it runs that branch. See
[where a predicate is accepted](../../reference/predicates.md#where-a-predicate-is-accepted).
