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
revision: 1
---
## Problem

A `conformances:` end must name a declared type; an event or command is refused as `unresolved_semantic_reference` (`crates/specify/ess-composition/src/lib.rs:1716-1744`). Adopters cannot prove a local event matches an owner's event. Adopter report N10, ess 0.68.0.

## Acceptance

- `an_event_conforms_to_an_owner_event`: a local event whose payload meets the reader rules of `docs/design/composition-type-conformance.md` against an owner event composes at the new composition format.
- `a_command_input_conforms`: the same for a command's input and response.
- `a_drifting_event_field_is_reported`: one field of another type is reported as drift naming the field.
- `an_older_composition_format_refuses_the_kind`: the previous format refuses an event end by name.
