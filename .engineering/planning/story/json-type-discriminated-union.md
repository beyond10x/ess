---
format: aep.planning-md/3
id: story:json-type-discriminated-union
kind: story
status: active
title: A short-or-long wire form told apart by JSON type can be declared, or is refused by name
summary: A struct may be written as its one scalar shorthand field (#594, option c, ess/27)
refs:
- provider: github
  reference: beyond10x/ess#594
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-10-17T10:08:03Z", actor: "human:timo", revision: 4}
- {from: "proposed", to: "active", at: "2026-10-17T10:08:03Z", actor: "human:timo", revision: 5}
---
## Outcome

A `kind: struct` type may name one of its fields as its shorthand, and a value of that struct may
then be written as that field's scalar alone: with `shorthand: op`, `getIssue` is the value
`{op: getIssue}`. Every target that projects or decodes the struct admits both forms and reads the
short one as the long one, or refuses the model by name. Assessed from beyond10x/ess#594.

## Fit review

Tree `2c0378386`; binary run: ess 0.61.0. Reproduction `~/.cache/ess-schema-tools/repro/untagged/system.yaml`
(brand-free; not the adopter's file).

1. **Need.** An entry of a data file is either a bare name or an object with options, told apart
   by JSON type. A `union` of `String` and a struct is refused at parse: `missing field 'tag'`.
   Requester's proposed syntax, labelled as theirs: "an untagged union whose variants differ by
   JSON type (string vs object vs array), refused when two variants share a JSON type".
2. **Class.** *Gap* for a retrofit (`ess:retrofitting`); *convenience* for a format still being
   designed.
3. **Already expressible?** Only by changing the data to the long form everywhere, or by importing a
   hand-written JSON Schema, which moves the type out of the specification.
4. **Fit.** The model keeps one union form, adjacently tagged (`docs/design/ess-review-v0.1.md:192-198`).
   A struct shorthand adds no variant set: one type, one decoder rule, and two wire forms that a
   scalar and an object tell apart.
5. **Second adopter.** A compose-style `ports` entry, `"8080:80"` or `{target: 80, published: 8080}`.
6. **Designs considered.** (a) change nothing; (b) a union told apart by JSON type; (c) a struct
   shorthand. (c) was chosen; it does not cover a string-or-list value.

## Decisions

Option (c), decided by the operator; decision-blocker:json-type-discriminated-union is cleared.

1. **Syntax.** `shorthand: <field>` on a `kind: struct` type, read beside the body as
   `undeclared_fields:` is. Its value names exactly one declared field; a list or mapping is
   refused as it is read.
2. **Format.** Source format `ess/27`. Below it the key is refused as `unsupported_format_version`
   naming `ess/27`. A model without the key keeps its IR bytes and `spec_digest`: the IR records the
   shorthands beside the types, omitted when empty.
3. **What the field must be.** A declared field of the struct, required (not `Optional`), whose
   type is a scalar on the wire: a primitive other than `Json`, `List`, `Map` and `Optional`, an
   enum, or a newtype over one of those. Every other field of the struct is `Optional`, because the
   short form fills nothing else. Each breach is refused by name at `<type>.shorthand`.
4. **Where it is admitted.** On a `kind: struct` type only. On a newtype, an enum or a union, and on
   an entity, an event, an error, a view or an actor, it is refused as `unsupported_construct`.
5. **Meaning.** The short form is the long form with only the shorthand field written. The struct's
   invariants apply to the value it normalizes to. Every writer the model generates writes the long
   form.
6. **Instance validation** is the JSON Schema projection: the struct's definition is
   `oneOf: [<the field's schema>, <the object>]`, annotated `x-ess-shorthand: <wire name>`. The two
   branches differ by JSON type, so exactly one matches. OpenAPI and AsyncAPI share the definition.
7. **Decoders.** Model type realization (`ess generate types`): Rust and Go decode either form into
   the struct and encode the long form; TypeScript declares the wire union of the scalar and the
   object, with a report obligation to normalize. Bundle realization is unchanged. Targets that do
   not decode the short form yet refuse a model that reaches a shorthand struct, by name: the
   synthesized Rust and Go servers and web app, and conformance synthesis wherever a target's
   answer would be observed (a response, an event, a view row).
8. **Diff.** `ess verify diff` classifies a shorthand added (`expanded`) and removed or moved
   (`narrowed`) on a struct.

## Acceptance

Each scenario is a named test in an existing test binary.

- `shorthand_struct_validates_at_ess_27`: a struct with `shorthand: op`, `op: String`, every other
  field `Optional`, validates under `ess/27`, and its IR names the shorthand.
- `shorthand_below_ess_27_is_refused`: the same under `ess/26` is refused as
  `unsupported_format_version` naming `ess/27`.
- `shorthand_on_a_non_struct_is_refused`: on a newtype, enum or union, `unsupported_construct`.
- `shorthand_outside_types_is_refused`: on an event, `unsupported_construct` at its key.
- `shorthand_naming_no_field_is_refused`, `shorthand_on_a_non_scalar_field_is_refused` (a
  struct, a `List`, an `Optional`), `shorthand_beside_a_required_field_is_refused`,
  `shorthand_naming_two_fields_is_refused`.
- `ir_without_shorthand_keeps_its_bytes`: a model that does not write the key keeps its IR bytes.
- `schema_projects_shorthand_as_one_of_scalar_and_object`: the JSON Schema definition is `oneOf`
  of the field's schema and the closed object, with `x-ess-shorthand`, and `ess generate schema
  validate` admits `getIssue` and `{op: getIssue, paging: token}` and refuses `7` and `[getIssue]`.
- `rust_types_decode_the_short_form`, `go_types_decode_the_short_form`: the realized library
  decodes `"getIssue"` to the struct with `op` set and encodes the long form.
- `typescript_types_declare_the_wire_union`: the declaration is the scalar or the object.
- `synthesis_refuses_a_shorthand_struct_by_name`: Rust, Go and web synthesis refuse naming
  `shorthand`.
- `diff_classifies_a_shorthand_change`: added is `expanded`, removed is `narrowed`.

## Scope

`crates/specify/ess-domain` (`types.rs`, a `shorthand` module beside `undeclared_fields.rs`,
`spec.rs`, `system.rs`), `crates/specify/ess-compiler` (`ir.rs`, `resolve.rs`),
`crates/generate/ess-gen` (`types.rs`, `docs.rs`), `crates/generate/schema-contract/src/realize*`,
`crates/generate/ess-synth`, `crates/verify/ess-conformance` (synthesis refusal),
`crates/verify/ess-diff`, `crates/edge/ess-cli` tests, `models/toolchain/domains/specify.yaml`,
`schemas/generated/ess.schema.json`, `website/docs/reference/{formats,spec-versions}.md`.
Not here: a short-form witness in conformance suites and short-form decoding in the synthesized
servers and observers.
