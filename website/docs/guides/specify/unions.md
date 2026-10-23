---
title: Tagged unions
sidebar_position: 2
description: How a value that is one of several shapes is declared with kind union, why every union carries a tag, what each variant looks like on the wire, and when a union beats a struct of optional fields.
---

# Tagged unions

A value that is exactly one of several shapes, each with its own payload, is a **union**. An order
is open, blocked for a reason, or complete with an outcome; it is never two of those at once, and a
blocked order has a reason where a complete one has an outcome. A union says that in the model, so
the projections and the generated types hold a value to one variant and to that variant's payload.

## Declare a union

`kind: union` names the member that carries the variant (`tag:`) and lists the variants
(`variants:`), each label with the type of the payload it carries. A label with no type is a
**unit variant**: it carries nothing. Unit variants need `format: ess/22` or later.

```yaml
format: ess/22
system: demo
version: v1
domain: demo.work
types:
  - name: demo.work.Completion
    kind: struct
    fields:
      - {name: outcome, type: String}
      - {name: note, type: Optional<String>}
  - name: demo.work.Status
    kind: union
    tag: kind
    variants:
      Open:                            # a unit variant; `Open: ~` and `Open: null` are the same
      Blocked: String                  # a payload variant of a primitive
      Complete: demo.work.Completion   # a payload variant of a struct
```

`ess specify validate --path` accepts this file as it stands. The union is a type like any other:
an entity field, a command input or a view field may be typed `demo.work.Status`, and
`List<demo.work.Status>` or `Optional<demo.work.Status>` wrap it as they wrap any type.

A union declares at least one variant; `variants: {}` is refused as `empty_declaration`. A union
of only unit variants is admitted.

## A union is always tagged

There is no untagged union. A union without `tag:` is refused (`missing field` at parse time, and
an empty `tag: ""` as `empty_declaration`, whose hint says an untagged union cannot be decoded
without guessing).

The reason is the way back. An untagged union does not round-trip, because
two variants of the same shape cannot be told apart. With `Blocked: String` and a second variant
`Cancelled: String`, the untagged values of both are a bare string, and a reader handed
`"supplier late"` has no way to say which variant it was written as. JSON Schema, OpenAPI and a generated decoder would each have to guess,
and they need not guess alike. The tag makes the variant part of the value, so every reader decodes
what the writer wrote.

The tag sits beside the payload, not inside it. A payload variant carries its payload under the
content key `value`; where the tag is itself named `value`, the content key is `content`. A unit
variant is the tag alone, with no content member at all: `{"kind": "Open", "value": null}` is
refused, and so is a payload variant without its content member, unless its payload is
`Optional<…>`.

## On the wire

Each variant is one branch of a JSON Schema `oneOf`. Its branch pins the tag with `const`, requires
the tag and, for a payload variant, the content member, and is closed
(`additionalProperties: false`). The table and the definition below are generated from the
declaration above; nothing in them is written by hand.

[ess-union-wire-begin]: # (generated from the declaration above by ess generate --kind schema; regenerate with ESS_UNION_PAGE=write cargo test -p ess-cli --test union_page)

| variant | required members | wire value |
|---|---|---|
| `Blocked` | `kind`, `value` | `{"kind": "Blocked", "value": <a string>}` |
| `Complete` | `kind`, `value` | `{"kind": "Complete", "value": <a demo.work.Completion object>}` |
| `Open` | `kind` | `{"kind": "Open"}` |

The definition `ess generate --kind schema` writes for `demo.work.Status`:

```json
{
  "title": "Status",
  "x-ess-name": "demo.work.Status",
  "x-ess-kind": "union",
  "oneOf": [
    {
      "title": "Blocked",
      "type": "object",
      "properties": {
        "kind": {
          "type": "string",
          "const": "Blocked"
        },
        "value": {
          "type": "string"
        }
      },
      "required": [
        "kind",
        "value"
      ],
      "additionalProperties": false
    },
    {
      "title": "Complete",
      "type": "object",
      "properties": {
        "kind": {
          "type": "string",
          "const": "Complete"
        },
        "value": {
          "$ref": "#/$defs/demo.work.Completion"
        }
      },
      "required": [
        "kind",
        "value"
      ],
      "additionalProperties": false
    },
    {
      "title": "Open",
      "type": "object",
      "properties": {
        "kind": {
          "type": "string",
          "const": "Open"
        }
      },
      "required": [
        "kind"
      ],
      "additionalProperties": false
    }
  ],
  "x-ess-union-tag": "kind"
}
```

[ess-union-wire-end]: #

The OpenAPI and AsyncAPI documents carry the same branches. The model types and the generated Rust
and Go types read and write the same shapes: a unit variant is a unit enum variant in Rust and an
empty struct variant in Go, and each codec refuses a content member on it.

## When to choose a union

Choose a union over a struct of optional fields when the value is one of several cases and each
case has its own data. The struct form says less than it seems to:

```yaml
- name: demo.work.StatusFields
  kind: struct
  fields:
    - {name: blocked_reason, type: Optional<String>}
    - {name: completion, type: Optional<demo.work.Completion>}
```

- **Nothing says one at a time.** Both members may be present, or neither, and the model cannot
  tell which combinations are legal. A union holds exactly one variant, and the generated schema
  refuses a value that mixes two.
- **Nothing ties the data to its case.** In the struct, a `completion` beside a `blocked_reason`
  is a value like any other. In the union, an outcome exists only under `Complete` and a reason only
  under `Blocked`, each typed by its variant.
- **The case has no name.** Which case a struct value is in has to be inferred from which members
  happen to be present; a union value names it in the tag, so every reader of the value reads it
  directly.

Keep a struct when the members are independent facts that may each be present or absent on their
own: a `note` and a `due_date` that may both be set are fields, not variants.

**Union or enum.** A union of only unit variants and an enum both name one of a fixed set of labels
and differ only on the wire: an enum value is a string (`"Open"`), a union value an object
(`{"kind": "Open"}`). Prefer an enum for a closed set of names. Declare a union when at least one
variant carries a payload, or when a payload variant is expected to join later: turning a unit
variant into a payload variant keeps the tag, whereas turning an enum into a union changes every
value's JSON type.
