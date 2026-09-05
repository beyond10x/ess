---
format: aep.planning-md/1
id: story:unique-wire-field-identity
kind: story
status: implemented
title: Reject colliding wire field names before any projection
relations:
- serves: vision:O2
revision: 4
---
## Evidence

crates/generate/ess-gen/src/types.rs::object_with inserts properties by wire name and documents that duplicate wire names can collapse. crates/generate/ess-gen/tests/model_types.rs::missing_roots_and_wire_collisions_are_refused_before_map_serialization proves that a source with two fields using wire key score still compiles; the new ModelTypes selection refuses it before serialization. This local guard does not make ordinary schema/OpenAPI/full-synthesis paths safe.

## Outcome

Wire property identity is unique before any projection can erase one field. This extends the existing typed Field/Naming contract, not a new domain noun.

## Acceptance

The model/compiler boundary accumulates source-located refusals for colliding effective wire names in every field-bearing construct that shares an object namespace, including defaults and explicit renames, while noncolliding models retain their canonical projections.

## Scope

Inspect shared field validation in crates/specify/ess-domain and crates/specify/ess-compiler, then the wire namespaces of entity identities/state, structs, command/event/error payloads and views. Do not assume all of these namespaces have identical rules. Preserve the existing model-selection refusal until the compiler establishes the invariant. Add focused negative controls and run task check.

## Relation To Current Work

Discovered while implementing story:types-only-realizations. That story now refuses selected model struct collisions locally; this story owns consistent compiler enforcement across projection consumers. No runtime application change or private adopter data is required.

## Binding Design

`docs/design/unique-wire-field-identity.md` fixes the namespaces and enforcement point. A shared domain-owned pass in `Specification::validate` checks named structs, entity identity/fields/synthetic state, command inputs, event/error payloads, view rows and separate view parameters. Exact wire-key equality is distinct from display names and host identifier spelling; nested objects are not flattened. Existing projection bytes for admitted models remain unchanged. Keep the model-selection guard as defense in depth.

Concrete implementation scope: `crates/specify/ess-domain/src/{lib,spec,wire}.rs`, compiler integration tests, existing model-realization negative control, the shared projection's stale gap comment, public field-naming documentation and changelog. No new domain format or decoder behavior.

## Local Result

`Specification::validate` now checks effective wire identities in every field-bearing JSON object namespace through `ess-domain/src/wire.rs`. Entity state is reserved before authored identity/fields; view row and parameter namespaces are separate; named view shapes are checked. Exact special keys and display names remain legal. Existing persisted formats and valid projection bytes are unchanged.

Four compiler integration tests cover seven namespaces, explicit/default name collisions, identity/state conflicts, special keys, shared names across separate objects and eight accumulated semantic source paths. The model-realization regression now proves refusal during specification assembly, before a schema map exists. The source locator recognizes `params` and `shape` as structural path segments; its existing documented ambiguity fallback remains unchanged.

`CARGO_NET_OFFLINE=true task check` exited 0 after correcting the new fixture to use the actual emits list, event-keyed payload and structured all-predicate syntax. All workspace tests, strict Clippy, format, rustdoc, smoke and projection-drift checks passed. No release or deployment is implied.


## Integration Provenance

Reconciled through AEP from wt-46ef382d9f07 at original revision 6 and status implemented. Source artifact SHA-256: 63c59c6b108c88853cc0778bdc46cb00a13ef4f36f0c88d43bde5b25885111d7. Original journal history remains with its source recovery snapshot; this store records the reconciliation as new governed operations.
