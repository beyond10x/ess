---
title: Refusals, fields and invariants
sidebar_position: 2
description: Why a refusable command declares outcomes, what an invariant may read, and how enums, instants, text, JSON and Binary64 fields are declared.
---

# Refusals, fields and invariants

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

Two guards may both hold of one input. A refusal with a `when:` over the input is taken before any
accepting branch whose guard it overlaps, whatever order they are written in. With `closed: open ==
false` and `id-required: ticket_id == ""`, the input `{ticket_id: "", open: false}` is refused as
`id-required`, and the generated suite sends it and requires that. An accepting branch cannot read
the identity to step aside, so this precedence is how such a command is written.

Two accepting branches may overlap as well. The first declared whose guard holds answers: with
`small: amount < 100` written before `flagged: amount > 50`, the input `{amount: 75}` takes
`small`, and the generated suite sends an input in that overlap and requires `small`. Write the
narrower branch first when it should win. An `external:` branch takes its place in the same order: written after
`small`, it is asked only for an input `small` does not claim.

### An invariant reads only what every creation sets

An entity invariant that reads a required field needs every `creates:` branch of that entity to set
the field. Without a value, `reminder_count >= 0` would hold only if the implementation happened to
pick a value that satisfies it. `validate` refuses the gap with `ESS-COMMAND-018` at the creating
outcome. The fix is to set the field there:

```yaml
- name: accepted
  creates: billing.invoice.Invoice
  instance: invoice_id
  sets:
    account_id: input.account_id
    total: input.amount
    reminder_count: "0"
```

Or declare the field `Optional<…>` if an instance may lack it. The identity, the lifecycle `state`
and `Optional` fields are never asked for. A field an invariant reads anywhere counts, including
inside `any:`.

A literal in `sets:` or `payload:` may also be written as the YAML value it means:
`reminder_count: 0` over an `Integer` and `paused: false` over a `Boolean` compile to exactly what
`"0"` and `"false"` do. An unquoted number or boolean over a text field or an enum is refused with
the repair, `quote it: label: '0'`. Over any other type it gets the same refusal as its quoted form.
A decimal such as `1.5` is never a literal, quoted or not; read it from an input.

A literal cannot say that an `Optional` field holds nothing. `metrics: none` is not read as the
field's type and synthesis drops it, and `lane_id: ""` is an empty string, which a reader tells
apart from an absent value and which a struct cannot hold at all. To empty a field, write
`{cleared: true}`; see [value expressions](values-and-views.md#value-expressions).

### Cover every declared enum value

Since 0.23.0 a command may omit its default when its input guards
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

### Order two instants

A right-hand side without a dot is a literal, so `when: ends_at > starts_at` compares `ends_at`
with the text `"starts_at"`. Validation refuses it and names the spelling that reads a field:
declare the two `Timestamp` fields in one struct and compare its members.

```yaml
input:
  - {name: window, type: rooms.booking.Window}   # struct {starts_at, ends_at: Timestamp}
outcomes:
  - name: booked
    when: window.ends_at > window.starts_at
```

A `Timestamp` is ordered by the RFC 3339 instant it names, so `+01:00` and `Z` spellings compare
correctly. Against a literal, write an instant: `when: ends_at > "2020-01-01T00:00:00Z"`. Ordering
a `Timestamp` against text that is not an instant is refused. `Duration` has no ordering yet.

### Say which characters a text may hold, and how long it may be

A `String` the implementation restricts to a character set says so with `alphabet:` on its newtype,
and a length limit is `.count`, the number of Unicode scalar values. Both need `format: ess/11`.

```yaml
types:
  - name: keypad.dial.KeySequence
    kind: newtype
    of: String
    alphabet: "0123456789*#ABCD"
commands:
  - name: keypad.dial.SendKeys
    input:
      - {name: keys, type: keypad.dial.KeySequence, example: "12#"}
    outcomes:
      - name: too-long
        when: keys.count > 64
        error: keypad.dial.UnsupportedKey
      - name: sent
        emits: [keypad.dial.KeysSent]
```

Every character of a value is one of the alphabet's, compared exactly, with no normalization or case
folding. Synthesis builds the input's witness from those characters, and a guard such as
`keys.count > 64` from texts of 65 and 64 characters. An `example:` on a command input is the value
the first witness starts from. It is not a constraint, it has to be a value of the input's type, and
only a scalar input takes one. Generated code documents an alphabet and does not enforce it, and
Entity Runtime refuses both an alphabet and a text length by name.

### Say what a text starts with

A `String` newtype whose every value starts with fixed text says so with `prefix:`. It needs
`format: ess/15`.

```yaml
types:
  - name: demo.msgs.Channel
    kind: newtype
    of: String
    prefix: "/"
```

The prefix is literal text, not a pattern. The published JSON Schema carries it as an anchored
`pattern` (`^/`), every witness synthesis builds starts with it (`/channel` for a field named
`channel`), and a literal written for the field that does not start with it is refused. Beside an
`alphabet:` every character of the prefix has to be in the alphabet, and a newtype of a newtype may
declare a longer prefix that starts with the inner one. Entity Runtime lowers the prefix to a
`starts_with` rule.

### Carry any JSON value

`Json` is a primitive for a value the specification does not structure: a body delivered as it
arrived. It needs `format: ess/15`.

```yaml
types:
  - {name: demo.msgs.Body, kind: newtype, of: Json}
```

It projects to the empty JSON Schema, which every value satisfies, and a suite compares it
structurally: an object with the same members in another order is the same value. It is never a map
key, a predicate never reads one, and no literal spells one, so a payload fills a `Json` field from
an input. Entity Runtime stores it as its own JSON field kind. The Rust code target represents it
as the generated types crate's dependency-free `json::Value`. The CLI code target reads a `Json`
flag as one JSON document into that same `json::Value`. The Go and web code targets refuse a model
that uses it, at every position, until they have a representation for it.

### Carry finite binary floating-point values

The `ess/2` format, introduced in 0.20.0, adds `Binary64` for finite IEEE-754 values. Use it
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

Binary64 values are executed through
[normalization recipes](../generate-artifacts.md#binary64-inputs-and-integer-conversion) (`ess-normalization/5`).
Standalone structural Rust/Go codecs and whole-system/conformance targets
refuse Binary64; TypeScript structural output reports the finite codec obligation.
Adding a Binary64 type to a model selected in full therefore requires checking
every intended target's support before adopting it.
