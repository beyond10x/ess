---
title: Views
sidebar_position: 5
description: View consistency, replicas, who may read a view, paging, and fields derived from the lifecycle state.
---

# Views

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
