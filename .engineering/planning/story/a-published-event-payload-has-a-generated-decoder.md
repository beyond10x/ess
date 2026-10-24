---
format: aep.planning-md/3
id: story:a-published-event-payload-has-a-generated-decoder
kind: story
status: draft
title: A published event payload has a generated decoder
summary: 'Adopter report row 25: no generated decoder for published event payloads.'
tags:
- ess-0.70.0
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 1
---
## Problem

An adopter generating a Rust server and adapter from a specification found no generated decoder for the events the system publishes: an adapter that consumes the published envelope decodes each payload by hand (adopter report, row 25, ess 0.67.0). The report also names the payload keys being snake_case; that half is not a gap, because the specification declares no `naming: {wire:}` for those fields, and a declared wire name is already honoured.

Generated command and view types decode from the wire (`ess generate types`, the generated `dispatch` in `crates/generate/ess-synth/src/rust/http.rs:836`); a published event's payload has no matching decoder.

## Acceptance

- `a_published_event_payload_decodes_through_a_generated_decoder`: for each event a system publishes, the generated Rust code exposes a decoder from the published envelope's bytes to the event's payload type, and a payload the system published round-trips through it.
- `an_event_decoder_honours_declared_wire_names`: a payload field declaring `naming: {wire: <name>}` is read under that name; an undeclared field keeps its model name.
- `an_event_decoder_refuses_a_payload_of_another_event`: a payload whose event name does not match is refused with the name it carried.
