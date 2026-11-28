---
format: aep.planning-md/3
id: story:openapi-import-carries-a-discriminated-one-of-as-a-union-and-an-all-of-of-objects-as-one-object
kind: story
status: draft
title: OpenAPI import carries a oneOf with a discriminator as a union and an allOf of objects as one object, and refuses the rest by name
summary: ess infra import openapi refuses every composition but the nullable pair; a public contract was refused at 51 positions (downstream report 2026-11-26, gap 8; OI-D1 to OI-D3)
tags:
- external-request
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- informed_by: story:openapi-import-admits-open-objects-as-json
- informed_by: story:a-union-may-be-encoded-as-an-object-keyed-by-its-variant
revision: 1
---
## Need

Reported 2026-11-26 by a downstream application on 0.99.2 (gap 8 of a report of ten).
`ess infra import openapi` refuses every `allOf`, `anyOf`, `oneOf` and `not` except a nullable pair
of one schema and `{type: 'null'}` (`crates/generate/ess-openapi/src/lib.rs:1518-1525`;
`docs/design/review-openapi-accounting.md:189-190`). A public vendor's OpenAPI contract that models
polymorphic bodies as a `oneOf` with a `discriminator` and shared members as an `allOf` of a `$ref`
and an inline object is refused at 51 such positions, so the operations that use them cannot be
imported. The decision recorded on `story:openapi-import-admits-open-objects-as-json` keeps "an
untagged `oneOf` of objects" refused by name; the tagged form and `allOf` were not decided.

## Fit review

1. **Need.** Import the two composition forms public contracts use most: a choice among object
   shapes told apart by a named member, and an object built from shared parts. The reporter proposed
   no syntax.
2. **Class: gap.** The import's schema model has no union and no merged object: `InterfaceSchema`
   holds references, scalars, arrays, tuples, objects and the optional pair
   (`crates/generate/ess-openapi/src/lib.rs`, `pub enum InterfaceSchema`).
3. **Already expressible?** Only by writing the specification by hand, which is the retrofit path
   without the import.
4. **Fit.** An ESS union is always tagged, with the tag beside the payload under the content key
   `value` (`website/docs/guides/specify/unions.md:47-63`). An OpenAPI `discriminator` puts the tag
   among the variant's own members, so a discriminated `oneOf` is an ESS union only with a wire
   encoding that places the tag there. That is the same question as the draft
   `story:a-union-may-be-encoded-as-an-object-keyed-by-its-variant` (a union keyed by its variant):
   the union's wire encoding, decided once for both. An `allOf` of objects is one object whose members
   are the parts' members. Siblings: the projection writes back what the import reads; a union
   already projects as a `oneOf` with a `const` tag per branch (`unions.md:69`).
5. **Second adopter.** A payments API whose payment method is a card or a bank transfer by `type`;
   an event API whose envelope is told apart by `event_type`.
6. **Cost.** The `ess-openapi-import/1` envelope gains schema kinds, a change of its persisted
   shape (OI-D3). An internally tagged union encoding is new authored surface, shared with the union
   encoding story. One accounting row per form in `review-openapi-accounting.md`.
7. **Designs.** (a) Change nothing. (b) Import a discriminated `oneOf` as a struct of the
   discriminator and every variant's members `Optional`, with a gap: loses that exactly one variant
   is present. (c) Import it as a union with the tag among the variant's members, and an `allOf` of
   objects as one object: chosen, pending OI-D1.

## Decisions

Each option list names its default, the recommended option A. The untagged `oneOf`, every other
`anyOf`, and `not` stay refused by name (the 2026-10-23 decision).

- **OI-D1, a discriminated `oneOf`.** A: a `oneOf` with `discriminator.propertyName` whose every
  member is a `$ref` to an object requiring that member as a string constant imports as a union whose
  tag sits among the variant's members, the encoding designed together with the union-encoding story
  (recommended). B: the flattened struct of design (b), with a gap. C: keep refusing.
- **OI-D2, `allOf`.** A: an `allOf` whose parts are objects or `$ref`s to objects imports as one
  object holding every part's members; a member two parts declare with different schemas is refused
  at its path (recommended). B: keep refusing.
- **OI-D3, the envelope.** A: `ess-openapi-import/2`, written only for a document that uses a new
  form, `/1` otherwise (recommended). B: `/2` for every document.

## Acceptance

1. A minimal contract with a discriminated `oneOf` of two objects and an `allOf` of a `$ref` and an
   inline object imports with no refusal, and the envelope passes its replay check.
2. Projecting the import and importing the projection give an equal IR.
3. An untagged `oneOf`, an `allOf` whose parts declare one member differently, and a discriminator
   naming a member a variant does not require are each refused at their path, naming the rule.
4. A contract without these forms imports to the bytes it did.
5. `docs/design/review-openapi-accounting.md` has a row for each form.

Every new test runs under 5 s; none may pass the 60 s bound, and the test step stays within 120 s.
