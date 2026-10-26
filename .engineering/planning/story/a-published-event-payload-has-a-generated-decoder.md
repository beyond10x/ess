---
format: aep.planning-md/3
id: story:a-published-event-payload-has-a-generated-decoder
kind: story
status: implemented
title: A published event payload has a generated decoder
summary: 'Adopter report row 25: no generated decoder for published event payloads.'
tags:
- ess-0.70.0
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-10-26T09:18:51Z", actor: "human:timo", revision: 6}
- {from: "proposed", to: "active", at: "2026-10-26T09:18:51Z", actor: "human:timo", revision: 7}
- {from: "active", to: "implemented", at: "2026-10-26T11:30:09Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"test_result":1,"review_outcome":2}}}
---
## Problem

An adopter generating a Rust server and adapter from a specification found no generated decoder for the events the system publishes: an adapter that consumes the published envelope decodes each payload by hand (adopter report, row 25, ess 0.67.0). The report also names the payload keys being snake_case; that half is not a gap, because the specification declares no `naming: {wire:}` for those fields, and a declared wire name is already honoured.

Generated command and view types decode from the wire (`ess generate types`, the generated `dispatch` in `crates/generate/ess-synth/src/rust/http.rs:836`); a published event's payload has no matching decoder.

## Acceptance

Each line names the failing test written first, as a case in an existing test binary.

- `crates/generate/ess-synth/tests/served_publication.rs`:
  `every_published_event_decodes_back_through_the_generated_wire`: in both layouts, for every event the fixture publishes (the command's own and the binding-cascaded one), `decode_system_event(encode_system_event(e)) == e`. The generated crate builds with warnings denied.
- same binary, `an_event_decoder_reads_declared_wire_names`: a payload field declaring `naming: {wire: <name>}` is read under that name, and the model name is refused there; an undeclared field reads under its model name.
- same binary, `an_envelope_naming_an_unpublished_event_is_refused_by_that_name`: `{"event": "ops.core.Nope", "payload": {}}` is refused, and the error names `ops.core.Nope`. A payload missing a required member is refused at its path.

## Fit review

1. **Need.** An adapter that receives a published event envelope (`{"event": <qualified name>, "payload": {…}}`) needs to read it back into the synthesized crate's own event type. Today it decodes each payload by hand. Reproduction, ess 0.69.2: `ess generate synthesize --path examples/gatepass --target rust` writes `encode_event_gatepass_visit_*` (3) and `encode_system_event`, and no `decode_event_*` or `decode_system_event`. The requester proposed nothing beyond "a generated decoder". The snake_case half is not a gap: an undeclared field keeps its model name, and `naming: {wire:}` is honoured (triage, `batch0690/triage/rows-21-38.md:18`).
2. **Class.** Defect. The wire module's own contract says "Every generated named type gets both, because … a partial pair is a hole" (`crates/generate/ess-synth/src/rust/wire.rs:16-19`). An event is the one presented construct that gets only an encoder (`wire.rs:71-75`, `event_encoder` at `:333`).
3. **Already expressible?** Partly. `ess generate types --all-events --target rust --package p` emits serde `Deserialize` payload structs under the declared wire names (`ess-gen/src/model_types.rs:47-49`, beyond10x/ess#393). I ran this on gatepass: 7 types. That gives a second, standalone type, though, not the synthesized crate's event or `SystemEvent`, and it has no envelope decoder. It is no answer for an adapter beside the generated server.
4. **Fit.** No authored surface, so no grid cell is opened. The design reuses the existing per-type decoders (`decode_<type>`, `wire.rs:228`, `decode_member` at `:808`) and mirrors `encode_system_event` (`rust/http.rs:234-270`). Siblings: Go's codecs are package-private (`encodeEvent…`, `go/http.rs:776`), so a Go adapter has no public encoder either, and Go is refused by that reason. The web bridge reuses the Rust wire (`web/bridge.rs:427`). TypeScript `generate types` is structural only, by its own help text.
5. **Second adopter.** An audit consumer reads a component's published `OrderPlaced`/`OrderCancelled` envelopes from a queue and needs the same typed events the system crate defines, so it can match on them.
6. **Cost.** No format version, keyword or diagnostic. The change to the generated API is additive: new `pub fn`s in `wire`. It has one doc change: the module header stops saying decoders exist only for what enters the system (`wire.rs:16-17`).
7. **Alternatives.** (a) Change nothing, which leaves the hand decoders and the `generate types` duplicate type. (b) Point adopters at `generate types --all-events`: rejected, because that is a second type and has no envelope. (c) Generate `decode_event_<x>` and `decode_system_event` in the synthesized wire (chosen). The request is taken in substance. Its casing remark is answered "already honoured".

## Decisions

- Accept, redesigned. The synthesized Rust `wire` module emits `decode_event_<ident>(value, at)` for every event it presents an encoder for, and `decode_system_event(value) -> Result<SystemEvent, …>`, the exact inverse of `encode_system_event`. An envelope naming an event the system does not publish is refused, naming the event it carried.
- Wire names: the decoder reads each field under its declared `naming: {wire:}`, through the same member decoder commands use. No new casing rule.
- Go: not in this story, because Go's wire functions are unexported (`go/http.rs:776`) and a Go adapter has no public encoder either. The synthesize guide says so. A public Go wire surface is a separate request.
- No `ess/32`; no suite or `ess-http` format moves.

## Scope

| unit | files | confidence |
|---|---|---|
| event decoders | `crates/generate/ess-synth/src/rust/wire.rs`: `module` (`:71-75`) and a new `event_decoder` beside `event_encoder` (`:333`), reusing `decode_member`/`decode_value` (`:808`, `:836`); header doc `:16-19` | cited |
| envelope decoder | `crates/generate/ess-synth/src/rust/http.rs`: a `system_event_decoder` beside `system_event_encoder` (`:234-270`), emitted where `:176` emits the encoder | cited |
| variant table | `crates/generate/ess-synth/src/rust/system.rs` `system_event_variants` (read, not changed) | inferred |
| feasibility | `crates/generate/ess-synth/src/rust/feasibility.rs:1038` codec-prefix list gains `decode_event` | inferred |
| docs | `website/docs/guides/synthesize.md` (wire section) | inferred |

Crate: ess-synth (sizing: M). Format gate: none. Tests: `crates/generate/ess-synth/tests/served_publication.rs` (it already compiles both layouts and has a binding-cascaded event). Shares `rust/http.rs` and `served_publication.rs` with `story:generated-dispatch-and-handle-return-the-emitted-events`, so serialize the two.

confidence: high
