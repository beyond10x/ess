# Field wire names, presence policies, `Json` and text prefixes

Retrofit wave 2, unit `types`: beyond10x/ess#142, #139, #138 and #146, under source format
`ess/15`. Stories `story:field-wire-names-and-presence-policy` and
`story:json-values-and-text-patterns`.

## 1. A field's wire name (#142)

`Field` already flattened `Naming`, so `wire:` on a field was read. What was refused was the
nested spelling commands and events use for their own names, `naming: {wire: orderId}`.

- `Field` and `InputField` are read through `RawField` and `RawInputField`, which take both
  spellings. Writing flat keys and `naming:` on one field is refused. The field is written back
  flat, so a model's bytes and digest do not depend on the spelling.
- No format gate: the two spellings are one model.
- JSON Schema, OpenAPI and AsyncAPI already key a property by `wire_name` (`ess-gen/src/types.rs`);
  a test holds all three to it. A suite addresses fields by declared name and carries each field's
  naming, and the target adapter spells the wire key; the two spellings synthesize byte-identical
  suites.
- A view field (`RawViewField`) does not take the nested spelling.

## 2. Presence policy on an `Optional` field (#139)

```yaml
- {name: partner_ref, type: Optional<String>, presence: null_when_absent}
- {name: discount_code, type: Optional<String>, presence: omitted_when_absent}
```

**Representation.** `Presence` is carried on `Naming::presence`, because every projection already
reads a field's wire spelling from its naming and the IR carries `ResolvedField::naming`. It is never
read through `Naming`: `skip_deserializing` makes a command's, event's, type's or enum variant's
`naming:` refuse `presence` as an unknown key, and a field reads it as its own `presence:` key. It is
written back flat with the field, and skipped when absent, so bytes do not move.

**Validation** (`primitive_admission::presence`): on struct, entity, command input and response,
event and error fields. `unsupported_format_version` below `ess/15`; `type_mismatch` on a field that
is not `Optional<T>`; `conflicting_declaration` for `omitted_when_absent` on an `Optional<Json>`
(through newtypes), because `null` is a JSON value there and "never sent as null" would refuse one
of the type's own values. Views do not take `presence:`: an inline view field is read as
`RawViewField`, which refuses the key, and view parameters are the same; a view that projects a
named struct carries that struct's fields, whose policy is checked where the struct is declared.

**JSON Schema.** `null_when_absent`: the property is required and its schema is the `Optional`
projected whole, `anyOf [T, null]`. `omitted_when_absent`: optional and not nullable, which is what
an `Optional` field without a policy already published.

**Conformance.** `LeafShape::presence`. `admits(None)` is refused under `null_when_absent`,
`admits(Some(Null))` under `omitted_when_absent`. The policy is carried only on the leaf that is the
declaring field itself and under no other `Optional`: under an absent parent every leaf is absent,
and `reach_into` cannot tell that absence from the field's own. A struct-valued field has no leaf of
its own and its policy is not carried. Both omissions are weaker, never wrong.

**Witness.** A policy is observable only on an absent value. An optional input, or an optional
member of one, copied by `input.<field>` into an event field that declares a policy or has a struct
member that does (walked through newtypes and structs, not through an `Optional`), and read by no
guard, is sent absent first in every candidate: each candidate is tried with those members absent,
then filled. So whichever branch is reached, guarded or not, its first candidate publishes the
policy fields absent. A top-level input is left out; a member of a copied struct is spelled as its
policy spells absence (`null` under `null_when_absent`), because the whole struct is asserted as
the value sent. The cost: in those scenarios the copied value itself is not asserted.

**Response.** A response observation carries each response field's policy, and the comparison
refuses a `null_when_absent` field left out and an `omitted_when_absent` field sent as `null`.
An observation that carries one makes the suite `/24` or `/25`.

**Suite format.** Ordinary `ess-conformance/24`, coverage `/25` (`crate::presence`). The Go and
TypeScript runtimes unmarshal a leaf into a type that does not name `presence` and would drop it, so
they would pass the swap the policy forbids; both refuse 24 and 25 by version (their allow-lists do
not name them), and tests pin that. A pinned older suite that carries a policy is refused with
`UnsupportedVocabulary`.

The shape is built in `synthesize.rs` (`payload_shape`/`describe`, `mark_presence`). The Rust
execution reader admits suite majors 24 and 25 (25 requires coverage).

## 3. `Json` (#138)

- `Primitive::Json`, spelled `Json`, admitted from `ess/15` (`primitive_admission::reference`).
- Never a map key: refused by the parser, by `MapKeyNewtypes` and by admission.
- Never read by a predicate: the expression typer shapes it `Shape::Json`, which has no selectors
  and no scalar, so a comparison or a path into it is refused. `ScalarKind::of` answers `None`.
- No literal spells one: a payload or `sets:` literal for a `Json` field is refused; a payload fills
  it from an input.
- JSON Schema: the empty schema.
- Conformance: `Holds::Primitive { kind: Json }` admits every value; a `Json` input projects to no
  fact. The witness is a one-member object keyed and valued by the path (`{"body": "body"}`,
  `body-1` for a further instance), so two fields and two instances differ, and a copied payload is
  compared with `Node` equality, which is structural.
- Entity Runtime: `FieldKind::Json`, entity-core's own unchecked JSON kind; no spelling requirement.
- Code targets (Rust, Go, web, CLI) refuse a model that uses `Json` at every position
  (`failure::json`, `MissingRepresentation`), as they refuse `Binary64`. The emitted Rust workspace
  builds with zero third-party crates, so `serde_json::Value` is not available to it; a
  dependency-free wrapper over the exact JSON text needs a renderer in the fixed emitted `json`
  module, which changes template bytes every model shares. That, and Go `any`, is a follow-up.

## 4. `prefix:` on a `String` newtype (#146)

```yaml
- name: demo.msgs.Channel
  kind: newtype
  of: String
  prefix: "/"
```

- `TypeBody::Newtype::prefix` and `ResolvedBody::Newtype::prefix`, skipped when absent; a prefix
  makes a type constrained (`is_constrained`).
- Validation: `ess/15`; not empty; only over `String` through newtypes and not through an
  `Optional`; nested prefixes extend one another; every character of the effective prefix (the
  longest in the chain) is in the effective alphabet, reported where the prefix or alphabet that
  brought them together is declared.
- Literals: an input `example:` and a payload or `sets:` literal that does not start with the
  effective prefix are refused.
- JSON Schema: `pattern: ^<prefix>` with the ECMA-262 syntax characters escaped; a literal is
  certainly right where a character class for an alphabet is not.
- Witness: the prefix is put in front of the path text, then the alphabet maps the whole text,
  which keeps the prefix because its characters are in the alphabet. A count resize cycles the
  prefixed text, so it keeps the prefix in front; a candidate that does not start with it is refused
  by value admission (`setup_body`) with every other value the type refuses.
- Entity Runtime: a nominal `starts_with` rule. Each layer of a chain lowers its own rule under the
  field's path; the outermost keeps the plain name and an inner layer's rule, prefix or invariant,
  is suffixed with the declaring type, then with a counter while the name is still taken (two paths
  can sanitize alike), so no two rules of one definition share a name. A model with no collision
  keeps its rule names.
- Generated Rust and Go document it; the web catalogue lists it.

## Not done here

- `ess-diff` does not classify a changed presence policy; it falls to its residual. A changed prefix
  is `prefix-added`, `prefix-removed` or `prefix-changed` in `ess-diff/11` (beyond10x/ess#219);
  `ess-diff/10` shipped in 0.41.0 without them, so a `/3`–`/10` writer and reader refuse them.
- The prefix relation compares the *declared* prefixes, not the effective ones. An outer newtype
  that declares, drops or restates a prefix its inner layer already imposes is reported as
  narrowed or expanded although the values it admits do not move. The direction is never
  inverted. An effective-prefix relation would need the change to carry the effective prefix,
  because the reader re-derives the relation from the change's own content.
- `website/docs/reference/formats.md` and `spec-versions.md` do not list suite formats 24 and 25,
  and `FORMAT_RELEASES` / the toolchain model do not either.
- A periodic binding's context fields take `presence:` without its checks.
