---
format: aep.planning-md/3
id: story:an-input-keeps-missing-and-null-apart
kind: story
status: draft
title: A command input keeps a missing member and an explicit null apart
summary: 'Three-state input: missing, null and a value, readable by guards and sets:'
refs:
- provider: github
  reference: beyond10x/ess#678
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 3
---

- A served API accepts an input member that is missing and the same member sent as `null`, and the two mean different things (for example: missing keeps the stored value, `null` clears it).
- In ESS an `Optional<T>` input that declares no `presence:` admits both spellings as one absent value; one that declares `presence:` admits only one spelling. No input type keeps "missing" and "null" apart for a guard, a `sets:` mapping or a scenario.

## Expected

A way to declare an input whose three states (missing, `null`, a value) are distinct and can each be read by guards and `sets:` (for example a clear-to-absent mapping), with the published schema and conformance payloads following it.

## Status

Needs a design page before code: it adds a state to the input model and touches `sets:`, synthesis and the projections. A minimal reproduction from the reporter is requested.

## Scope: every position that takes `presence:` (2026-10-23)

An adopter asked whether the wire half covers a struct used as a request or response body. On 0.68.0, `null_when_absent` on a struct field refuses a missing member and `omitted_when_absent` refuses `null`.

`presence:` is one `Presence` enum on `Field`, admitted at struct, entity, command input and response, event and error fields (`crates/specify/ess-domain/src/primitive_admission.rs`, `presence`). The new value is a third variant of that enum, so it applies at all of those positions. View fields keep refusing the key.

Acceptance adds, beside the command-input case:

- `a_struct_field_admits_missing_and_null`: a struct field with the new presence, used as a command's body and as a response, projects as not required and `oneOf [null, T]`, and conformance payloads accept both a missing member and `null`.

## Correction: the projected encoding (2026-10-23)

The nullable encoding is the one `nullable_field` already emits for `null_when_absent` (`crates/generate/ess-gen/src/types.rs`, `object_annotated`): `anyOf [<T>, null]`, not `oneOf [null, T]` as written above. The new presence differs from `null_when_absent` only in leaving the property out of `required`.

`a_struct_field_admits_missing_and_null` asserts, for a struct field `reason: Optional<String>` with the new presence, used as a command's body and as a response:

- the projected property is `anyOf [string, null]` and `reason` is not in the object's `required`, on both sides;
- the projected schema accepts `{}`, `{"reason": null}` and `{"reason": "x"}`;
- a conformance payload with the member missing and one with it `null` are both accepted.
