---
format: aep.planning-md/3
id: story:a-composition-conformance-may-name-an-event-or-command
kind: story
status: draft
title: A composition conformance may name an event or command
summary: 'Adopter report N10: compose cannot compare events or commands as types (ess#664).'
refs:
- provider: github
  reference: beyond10x/ess#664
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 3
---
## Problem

A `conformances:` end must name a declared type; an event or command is refused as `unresolved_semantic_reference` (`crates/specify/ess-composition/src/lib.rs:1716-1744`). Adopters cannot prove a local event matches an owner's event. Adopter report N10, ess 0.68.0.

## Acceptance

- `an_event_conforms_to_an_owner_event`: a local event whose payload meets the reader rules of `docs/design/composition-type-conformance.md` against an owner event composes at the new composition format.
- `a_command_input_conforms`: the same for a command's input and response.
- `a_drifting_event_field_is_reported`: one field of another type is reported as drift naming the field.
- `an_older_composition_format_refuses_the_kind`: the previous format refuses an event end by name.

## Also: invariants in a type conformance (adopter report N17, ess#671)

- `compose_compares_invariants`: an owner type that gains `invariants: [value != "x"]`, with the digest repinned, against a local copy without it, is reported as `type_conformance_drift` naming the invariant, and compose exits non-zero.

## Also: enum variant attributes in a type conformance (adopter report N18, ess#671)

With the owner digest repinned, each change below is reported as `type_conformance_drift` naming the type, the variant and the attribute or bound, and compose exits non-zero:

- `compose_compares_variant_handled_by`: a variant's `handled_by` changes on the owner.
- `compose_compares_variant_terminal`: a variant's `terminal` changes on the owner.
- `compose_compares_variant_message_types`: a variant's `request` or `response` type reference changes on the owner.
- `compose_compares_invariant_bounds`: an invariant tightens from `>= 1` to `>= 2` on the owner.
- `compose_still_refuses_structural_drift`: wire spelling, a renamed or dropped variant, a field type, a dropped field and a presence change still exit 1.
