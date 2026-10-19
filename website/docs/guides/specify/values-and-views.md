---
title: Values and credentials
sidebar_position: 5
description: Value expressions, the caller's credential and inputs present after defaulting.
---

# Values and credentials

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
| `{related: {entity: <Entity>, where: <predicate>, field: <field>}}` | `payload:`, `sets:` | source `ess/22`; `field` of the one row of `<Entity>` that `where` selects, read before the outcome, at the field's declared type the target admits; none or several selected rows supply no value. `where` reads as a [row-set guard](../../reference/predicates.md#a-guard-over-the-rows-a-selector-selects)'s, and from `ess/27` may [test each candidate's related rows](../../reference/predicates.md#from-ess27-a-test-over-each-candidates-related-rows); below `ess/22` the same mapping is the nested mapping it always was |
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

### From `ess/27`: a computed value

Source `ess/27` computes a value with an expression, `{expr: "<expression>"}`, in `sets:`, an
event's `payload:` and an error's `payload:`, and computes a view field from its source row with
`expr:` beside the field's type (beyond10x/ess#560):

```yaml
- name: recorded
  creates: demo.ledger.Incident
  instance: incident_id
  emits: [demo.ledger.IncidentRecorded]
  payload:
    demo.ledger.IncidentRecorded:
      incident_id: {generated: true}
      duration: {expr: "input.ended_at - input.started_at"}           # a Duration
      share: {expr: "coalesce(round(input.affected / input.total, 2, half_even), 0)"}
  sets:
    duration: {expr: "input.ended_at - input.started_at"}
views:
  - name: demo.ledger.Incidents
    source: demo.ledger.Incident
    fields:
      - {name: incident_id, type: demo.ledger.IncidentId}
      - {name: elapsed, type: Duration, expr: "ended_at - started_at"}  # from the row
```

| Form | Over | Gives |
|---|---|---|
| `a + b`, `a - b`, `a * b` | Integer, Decimal | an Integer over two Integers, otherwise a Decimal; exact, and unknown past 128 bits |
| `a / b` | Integer, Decimal | an `Optional<Decimal>` at six places, half-even; absent where `b` is zero |
| `round(x, digits, half_even \| half_up \| down)` | Integer, Decimal | a Decimal; a quotient inside is rounded once, at these digits and this mode |
| `abs(x)`, `min(a, b)`, `max(a, b)` | Integer, Decimal, Duration; `min`/`max` also Timestamp and String | the operand's type |
| `coalesce(a, b)` | one type | `a` where it is present, otherwise `b` |
| `t1 - t2` | two Timestamps | a Duration of elapsed seconds |
| `t + d`, `t - d`, `d1 + d2`, `d * n` | Timestamp, Duration, Integer | a Timestamp; a Duration |
| `60s`, `5m`, `1h` | — | Duration literals; there are no days |

An expression in a command outcome reads `input.<path>` and, where the outcome acts on a subject
that exists before it, `subject.<field>` as the row held it; a view field reads its source row's
fields by name. The expression is typed by what it reads and held to the field it fills:
`ended_at - started_at` into an `Integer` is refused as `type_mismatch` naming the `Duration`, and
a quotient into a required field is refused naming `coalesce(…, 0)`. `now` is refused as a value,
and so are an expression in a set effect's `sets:` and a computed field in an aggregate view.

The interpreter and synthesis compute the value with one shared evaluator. Synthesis asserts it on
the event and the row, and runs the branch again on inputs worth checking the figure on: every
input the expression reads moved apart from the others, so a target computing `started_at -
ended_at` fails; each divisor read directly from the input at `0`; and each such quotient on a
rounding tie at its digits, once on an even and once on an odd neighbour, so a target rounding by
another mode fails one. The generated Rust and Go behaviours and queries, and Entity Runtime
lowering, keep such a command or view an obligation named by the expression. Below `ess/27`
`{expr: …}` is the nested mapping it always was, and `expr:` on a view field is refused naming
`ess/27`.

### A relation the model only implies
Where a stored field is typed as exactly one entity's named identity, or a `when_related:` row is settled by that type alone, and no relation says so, `ess specify validate` prints a warning, `ESS-ENTITY-019` (`ESS-COMMAND-019` for the guard): declare a `references` relation on the field, or the target's `owns`. A warning changes neither the exit status nor the compiled model; a bare primitive identity such as `Uuid`, and a row set selector (`entity`, `where`), are not linted ([how to read it](./layout-and-validation.md#a-relation-the-model-only-implies)).

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

Views (consistency, replicas, who may read them, paging and fields derived from the lifecycle
state) have [a page of their own](views.md).
