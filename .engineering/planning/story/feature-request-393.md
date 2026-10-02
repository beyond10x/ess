---
format: aep.planning-md/3
id: story:feature-request-393
kind: story
status: draft
title: An event's payload is selectable as a generated type
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#393
relations:
- serves: vision:O2
revision: 1
---
## Outcome

An event's payload is selectable as a generated type, so a producer library holds exactly the object a published event carries and the model declares those fields once.

## Acceptance

- `ess generate types --root <event>` selects the event's fields as a struct under the event's qualified name, display name and wire names, annotated `x-ess-kind: event-payload`, with the closure of the types its fields reach (`crates/generate/ess-gen/tests/event_roots.rs`).
- The selected definition's `type`, `properties`, `required` and `additionalProperties` equal those of the event's JSON Schema projection.
- `--all-events` selects every event payload and combines with `--all-types`; `--root` beside either `--all-*` selector is refused and writes nothing (`crates/edge/ess-cli/tests/model_type_events.rs`).
- An unknown root's refusal names both kinds it looked for.

## Origin

beyond10x/ess#393. Belongs to `epic:message-contract-clients` (recorded with #394's change); the edge is added once both are on `main`.

## Fit review

Per `.agents/skills/assessing-external-requests/SKILL.md`, 2026-10-03.

1. **Need.** A producer library needs the event payload as a type. `ess generate types --root <event>` answered `unknown_root: no resolved model type has this qualified name` (`crates/generate/ess-gen/src/model_types.rs`, `ModelTypes::select`, which searched `ir.types()` only). The workaround declares every payload field twice — a struct and the event — and checks equality outside ESS. Requester's proposal: `payload: <struct>` on an event, or let `generate types` select events.
2. **Class.** Gap: the payload object is in the model and no target can realize it.
3. **Already expressible?** Only by the duplicate declaration above.
4. **Fit.** No authored surface: an event already resolves to fields with wire names (`ResolvedEvent` in `crates/specify/ess-compiler/src/ir.rs`), and the struct schema mapping (`types::body`) renders the same object the event projection renders. Provenance seeds the slice with an `EventRef` for an event root, a `DeclaredTypeRef` otherwise.
5. **Second adopter.** An outbox writer that serializes `OrderPlaced` into a table column needs the same type.
6. **Cost.** One CLI flag (`--all-events`); no format bump, no diff kind, no new diagnostic beyond a reworded `unknown_root` detail.
7. **Alternatives.** (a) Change nothing: two declarations per event. (b) `payload: <struct>` on events (requester's first proposal): a format bump and a second way to declare fields, which every projection, diff and synthesis would then have to read. (c) Event roots in `generate types` (chosen): no authored surface.

## Decisions

- **accept, as selection (2026-10-03):** event names are valid roots; `--all-events` added; no `payload:` key.
