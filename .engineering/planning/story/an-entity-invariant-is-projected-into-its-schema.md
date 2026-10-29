---
format: aep.planning-md/3
id: story:an-entity-invariant-is-projected-into-its-schema
kind: story
status: implemented
title: An entity's invariants reach its JSON Schema as x-ess-invariants and as the bounds a struct's already get
summary: Entity documents (schema, OpenAPI x-ess-entities) publish x-ess-invariants and lower Integer comparisons like a struct
refs:
- provider: github
  reference: beyond10x/ess#688
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
- informed_by: approval-record:d138-schema-invariants-as-scoped
scope:
- confidence: cited
  path: crates/generate/ess-gen/src/types.rs
- confidence: cited
  path: crates/generate/ess-gen/tests/agreement.rs
- confidence: cited
  path: crates/generate/ess-gen/tests/corpus/projection-digests.txt
- confidence: inferred
  path: crates/generate/ess-gen/tests/entity_invariants.rs
- confidence: cited
  path: docs/design/types-only-realizations.md
- confidence: cited
  path: generated
- confidence: cited
  path: generated/openapi/invoice-service.yaml
- confidence: cited
  path: website/data/generate.terminal.json
revision: 9
transitions:
- {from: "draft", to: "proposed", at: "2026-10-28T11:01:16Z", actor: "human:timo", revision: 7}
- {from: "proposed", to: "active", at: "2026-10-28T11:01:17Z", actor: "human:timo", revision: 8}
- {from: "active", to: "implemented", at: "2026-10-29T00:13:45Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}, executor: "agent:ess-issues-lead", correlation: "ess-issues-1"}
---
## Fit review

1. **Need.** A reader of an entity's schema can neither see nor enforce the entity's rules. Example: `format_version > 0` on an entity's Integer field. Repro: a minimal specification outside the repository (not committed). Its generated entity document `schema/entities/<entity>.schema.json` has `format_version` as `{"type":"integer"}` and no `x-ess-invariants` key. On a struct, `samples > 0` gives `"minimum": 1` (`schema/types/<struct>.schema.json` of the same repro). Requester's proposal: project entity invariants exactly as a struct's are.
2. **Class.** Defect. The module documentation says "An invariant is published verbatim under `x-ess-invariants`" (`crates/generate/ess-gen/src/types.rs:135-136`). An entity document is `types::message`, which carries no invariants (`types.rs:1303-1312`, `Message::of_entity` `:1270-1285`). `story:feature-request-394` left entities out on purpose (its fit review Q4: "Entities publish no invariants to the schema today … so the scope is structs").
3. **Already expressible?** Not equivalently. A bounded Integer newtype carries `minimum` (`types.rs:1034-1043`), but it changes the domain type and the generated API. It also does not reach `distinct:`, `forall:` or cross-field rules.
4. **Fit.** No new vocabulary. The lowering is the struct's (`integer_bounds`, `types.rs:1014-1030`) applied to `entity_fields` (`:1321-1327`). The IR already holds the invariants (`crates/specify/ess-compiler/src/ir.rs:442`). Sibling messages (command input, event, view row, error) copy some of an entity's values but are not the entity, so they get nothing. This is the same reasoning as for relations (`types.rs:1175-1177`). Targets: the schema document (`schema.rs:322-342`) and the OpenAPI model view (`openapi.rs:2204-2211`) both go through `types::message`. AsyncAPI publishes no entities. Types-only realization is unaffected: entities are not ModelTypes roots (`model_types.rs:312`). `ess verify diff` is unaffected because it works on the IR (`crates/verify/ess-diff/src/lib.rs:19-28`).
5. **Second adopter.** An inventory entity with `quantity >= 0`. A schema-validated import of stock records should refuse `-1`.
6. **Cost.** No format, no keyword, no diagnostic. Entity documents gain `x-ess-invariants`, plus `minimum`/`maximum`/`const` where an Integer comparison exists. That moves the committed entity documents under `generated/schema/entities/` (for example the billing `Invoice` document, `reminder_count >= 0`), `generated/openapi/`, the gatepass `Visit` documents (`expected_minutes > 0`), the oracle-fixture `Order` documents (`weight_grams >= 0`), and their lines in `crates/generate/ess-gen/tests/corpus/projection-digests.txt`.
7. **Alternatives.** (a) Change nothing: validators keep accepting `format_version: 0`. (b) Annotation only, without bounds: this leaves entities behind structs again. (c) Chosen: annotation plus the struct lowering, through `Message`.

## Decisions

- Accept as proposed (2026-10-28). `Message` carries the entity's invariants (empty for every other kind). `message()` publishes them verbatim and applies the shared field lowering to the entity's fields. This one function is the one `#687`, `#685` and `#686` extend, so entities get those keywords automatically.
- `ASSERTIONS` in `crates/generate/ess-gen/tests/agreement.rs` (`:332-347`) gains `minimum` (and `maximum`/`const` if the corpus publishes them). Billing now publishes `minimum`, and `:861-891` fails on any keyword it has not classified.
- If moving the committed `generated/` files changes what a consumer sees, D135 applies (`approval-record:d135-a-breaking-unit-leaves-the-batch`).

## Acceptance

- The entity document publishes `x-ess-invariants` verbatim, in declaration order: `cargo test -p ess-gen --locked --test entity_invariants an_entity_schema_publishes_its_invariants_verbatim`.
- An entity's Integer comparison becomes `minimum`/`maximum`/`const` on its property, and `==` on an Optional field becomes min and max: `cargo test -p ess-gen --locked --test entity_invariants an_entity_integer_bound_becomes_a_keyword_on_its_property`.
- A JSON Schema validator refuses `format_version: 0`: `cargo test -p ess-gen --locked --test entity_invariants a_validator_refuses_an_entity_record_its_invariants_refuse`.
- Command input, event and view-row documents of a field the entity also holds gain no invariant: `cargo test -p ess-gen --locked --test entity_invariants a_message_carrying_entity_fields_gains_no_entity_invariant`.
- The OpenAPI `x-ess-entities` fragment equals the schema document's fragment: `cargo test -p ess-gen --locked --test agreement`.
- The committed projections and digests are regenerated: `cargo xtask generate --check` and `cargo test -p ess-gen --locked --test corpus`.
- `cargo test -p ess-gen --locked --test integer_bounds` passes unmodified.

## Scope

- `crates/generate/ess-gen/src/types.rs` (Message `:1164-1312`, integer_bounds `:1005-1030`, module doc `:24-45` and `:135-140`): thread the invariants, run the lowering over entity fields.
- `crates/generate/ess-gen/tests/entity_invariants.rs`: new tests.
- `crates/generate/ess-gen/tests/agreement.rs` `:332-347`: classify `minimum`.
- `crates/generate/ess-gen/tests/corpus/projection-digests.txt`, `generated/`: regenerated. Whether the gatepass copies under `generated/go/gatepass/` and `generated/rust/gatepass/` move is inferred.
- `crates/generate/ess-gen/src/schema.rs`, `openapi.rs`: no change expected, since both call `types::message`.
- `docs/design/types-only-realizations.md` `:96-101`, `CHANGELOG.md`: the lowering now covers entities.
- Confidence: high. The defect site and both callers have been read.
- Shared with: #687, #685, #686 (the `types.rs` lowering block, `agreement.rs`, `CHANGELOG.md`, `types-only-realizations.md`). `CHANGELOG.md` is shared with every unit.

## Batch 1 integration

- Batch 1 runs U1, U2, U3 and U4 at once (`approval-record:ess-issues-batch-1`). This unit does not edit `CHANGELOG.md`, `changes/*.yaml` or `WHATS-CHANGED.md`: the implementor returns its changelog entry in its report, and the integration branch `integration/ess-issues-1` writes every entry once, after the last unit merges, so four entries under the empty `[Unreleased]` heading do not collide.
- Shared with U4 (`story:a-component-not-reached-over-the-network-has-no-http-route`): both units regenerate `generated/openapi/invoice-service.yaml` (this unit the billing `x-ess-entities`, U4 its `info.description`), the byte count in `website/data/generate.terminal.json` and the openapi and served lines of `crates/generate/ess-gen/tests/corpus/projection-digests.txt`. U1 merges first. U4 merges second, and on a conflict in any of these files the integration branch takes neither side: it regenerates them from the merged sources and commits the result, then `task projection-check` must pass.
