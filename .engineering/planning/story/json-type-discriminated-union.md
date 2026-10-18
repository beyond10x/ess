---
format: aep.planning-md/3
id: story:json-type-discriminated-union
kind: story
status: draft
title: A short-or-long wire form told apart by JSON type can be declared, or is refused by name
summary: 'Assessment of #594 (untagged String-or-struct union): deferred to a design decision'
refs:
- provider: github
  reference: beyond10x/ess#594
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Outcome

A value whose wire form is one of several JSON shapes told apart by JSON type alone — a string
short form beside an object long form — can be declared in a specification, or ESS states by name
why it cannot. Assessed from beyond10x/ess#594; deferred to a design decision, recorded in
decision-blocker:json-type-discriminated-union.

## Fit review

Tree `2c0378386`; binary run: ess 0.61.0. Reproduction `~/.cache/ess-schema-tools/repro/untagged/system.yaml`
(brand-free; not the adopter's file).

1. **Need.** An entry of a data file is either a bare name or an object with options; the two are
   distinguishable by JSON type. Declaring `routes.table.Entry` as a `union` of `String` and a
   struct is refused at parse: `system.yaml: missing field 'tag'`. Requester's proposed syntax,
   labelled as theirs: "an untagged union whose variants differ by JSON type (string vs object vs
   array), refused when two variants share a JSON type".
2. **Class.** *Gap* for a retrofit (`ess:retrofitting`): an existing format with a short and a long
   form (a compose-style `ports` entry, a manifest `bin` that is a string or a map, a CI `runs-on`
   that is a string or a list) cannot be described as it is. *Convenience* for a format the author
   is still designing: the long form alone can be declared today.
3. **Already expressible?** Only by changing the data format to the long form everywhere, or by
   authoring the JSON Schema by hand and importing it (`ess generate schema import-document`,
   `docs/design/types-only-realizations.md:184-187, 202-205`), which moves the type out of the
   specification: refused as an idiom by policy 1 of `docs/design/generic-language-features.md:468-473`.
4. **Fit.** The model has one union form, adjacently tagged, on purpose: "Tagged only, with the tag
   field named in the model. If untagged is wanted later it can be added as a distinct form with its
   own name and its own warnings" (`docs/design/ess-review-v0.1.md:192-198`); the projector's
   branch comment says the same (`crates/generate/ess-gen/src/types.rs:850-857`, "it offers no
   untagged form at all"). The generic-language design's four decisions (G1–G4) and its two grids
   are about expressions, row queries, aggregates and guard precedence
   (`generic-language-features.md:46-66`); a type body form is in none of their rows or columns, so
   no open format can carry it (policy 2, `:474-480`). Every target would need a decoder by JSON
   type: Rust and Go structural codecs, TypeScript, web, the interpreter, synthesis (one witness per
   variant), `ess verify diff` (variant added/removed/retyped), the entity runtime, generated docs,
   JSON Schema/OpenAPI (`oneOf` without a `const`), and the wire-type collision rule must say what
   counts as one JSON type (`Decimal` and `Timestamp` are strings on the wire; `Optional` adds
   `null`; a newtype inherits its representation's type; `Integer` and `Binary64` are both numbers).
5. **Second adopter.** A retrofit of an existing configuration format whose `ports` entries are
   `"8080:80"` or `{target: 80, published: 8080}`; a package manifest whose `bin` is a path or a map
   of names to paths.
6. **Cost.** A new authored form (a key on `union` or a new kind), one source format, schema
   regeneration, a diff kind, a decoder and a synthesis witness per target, a refusal per target
   that cannot decode by JSON type, and the collision rule above.
7. **Designs considered.** (a) Change nothing: the long form is always expressible — weakest for a
   retrofit. (b) As proposed: a union variant set discriminated by JSON type, refused when two
   variants share one. (c) A narrower "short form" on a struct: a struct may also be written as one
   scalar that fills one named field (`short: op`), which keeps one type, one decoder rule and no
   variant set — fits the reported case and the `ports` case, not a string-or-list case. No choice
   is made here.

## Decisions

**Defer.** The need is real for a retrofit, but the design needs a decision nobody has made: whether
ESS admits a second union form (or a struct short form), what one JSON type is for the collision
rule, and which source format carries it outside G1–G4. Recorded as
decision-blocker:json-type-discriminated-union, which blocks this story. No implementation, no
format allocation, no reply promising a syntax. Reply to the requester: deferred; the long form is
the way to write it today.

## Acceptance

Once the blocker is cleared, the chosen design's acceptance replaces this section. Until then:
`ess specify validate` on `~/.cache/ess-schema-tools/repro/untagged/system.yaml` keeps refusing with
`missing field 'tag'` — no target silently admits an untagged union.

## Scope

None until the decision. Inferred, if accepted: `crates/specify/ess-domain/src/types.rs` (union
body), `crates/specify/ess-compiler`, `crates/generate/ess-gen/src/types.rs` (`variant`),
`crates/generate/schema-contract/src/realize*`, `crates/generate/ess-synth`, `crates/verify/ess-diff`,
`crates/verify/ess-conformance` (synthesis and interpreter), `website/docs/reference/spec-versions.md`.
