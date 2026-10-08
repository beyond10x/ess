---
title: Conversions and wire names
sidebar_position: 7
description: "Declared conversions between contexts, and the wire spellings of enum variants, fields, error codes and absent Optionals."
---

# Conversions and wire names

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

## A wire name in a path is one segment

A generated HTTP route is `/{domain}/commands/{command}` or `/{domain}/views/{view}`, each part
the declaration's `naming.wire` copied verbatim. So the wire name of a domain, a command or a view
cannot contain `/`, and cannot be `.` or `..`: any of those would address another route.
`ess specify validate` refuses one as `path_segment_wire_name`, naming the declaration and the wire
name. A view is refused whether or not a network component serves it:

```text
error[ESS-VIEW-012]: view gatepass.visit.ExpectedVisits has the wire name "../.well-known/demo-configuration", which a generated path segment reads; a `/` in it, or a name that is `.` or `..`, would address another route
  `ess-domain` refuses this as `path_segment_wire_name`
  help: spell the wire name without `/`, and not as `.` or `..`
```

Dots inside a name (`orders.v1`, `.well-known`) are still one segment and are accepted. A field's
wire name is a property key, not a path segment, and is not affected.

## An error can carry its own wire code

A generated HTTP handler names a refusal in its response body. By default the name is the error's
qualified name. `naming.wire` on the error (`format: ess/4` or later) replaces it:

```yaml
errors:
  - name: gatepass.visit.InvalidVisitLength
    naming: {wire: invalid_visit_length}
    summary: The expected length of the visit is not a positive number of minutes.
```

The Rust and Go servers from `ess generate synthesize` then answer a refused `RegisterVisit` with
`"error": "invalid_visit_length"` beside `"outcome": "refused"`, where they wrote
`"error": "gatepass.visit.InvalidVisitLength"` before. Without an override both keep the qualified
name. An earlier format refuses the key with `ESS-ERROR-009`.

Only that transport code changes. The error's identity in the model, its payload type, the
outcomes that name it and the conformance assertions on it keep the qualified name. Two errors may
share one wire code, so a code alone does not say which refusal happened: a client tells them apart
by the outcome or another declared field, and ESS adds no message text or reverse lookup to do it.
`ess verify diff` reports a new or changed code, in an `ess-diff/4` document:

```text
  changes  error gatepass.visit.InvalidVisitLength: wire code `gatepass.visit.InvalidVisitLength` → `invalid_visit_length`
           error/gatepass.visit.InvalidVisitLength/wire-name-changed
```

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
