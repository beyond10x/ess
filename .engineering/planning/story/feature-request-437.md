---
format: aep.planning-md/3
id: story:feature-request-437
kind: story
status: draft
title: Flag relations a specification implies but does not declare
tags:
- ess-0.54.0
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#437
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
- depends_on: story:feature-request-434
- depends_on: story:feature-request-448
scope:
- confidence: inferred
  path: crates/edge/ess-cli/src/main.rs
- confidence: inferred
  path: crates/edge/ess-cli/tests/fixtures/implied-relation.yaml
- confidence: inferred
  path: crates/edge/ess-cli/tests/validate_implied_relations.rs
- confidence: cited
  path: crates/specify/ess-compiler/src/diagnostic.rs
- confidence: cited
  path: crates/specify/ess-compiler/src/resolve.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/related_guard.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/related_value.rs
- confidence: cited
  path: crates/specify/ess-domain/src/entity.rs
- confidence: cited
  path: website/docs/guides/specify/values-and-views.md
- confidence: inferred
  path: website/docs/reference/diagnostics.md
revision: 9
---
## Outcome
Resolve beyond10x/ess#437: Flag relations a specification implies but does not declare.

## Origin
beyond10x/ess#437, filed 2026-10-05; adopter feedback (2026-09-25) that missing relations were found only by review, for example which agent's record a pooled licence count is taken over.

## Fit review
1. Need: a specification that stores another entity's identity, or reads another entity's row, without declaring the relation should get an advisory, not a refusal. Minimal reproduction, written fresh (`<fit-review scratch>/probe-437/system.yaml`): `Pool` has identity `PoolId` (newtype of `Uuid`), and `Agent` has a stored field `pool: PoolId` with no `relations:`. `ess specify validate` (installed 0.52.0) prints `probe v1 — 1 file(s), valid`, and `--format json` has no diagnostics at all. Requester's proposal (theirs): a lint for (a) an identity-typed field with no relation, (b) a guard or view reading across two entities with no declared path, and (c) an aggregate counting records of an unconnected entity.
2. Class: gap. Checking relations today covers only declared relations (`validate_relations`, `crates/specify/ess-domain/src/entity.rs:1368`). Yet ESS already acts on the implied relation. When no relation is declared, a related read resolves "the entity whose identity is exactly `via_type`" (`crates/specify/ess-domain/src/command/related_value.rs:79-117`), and it refuses only the ambiguous case (`crates/specify/ess-domain/src/command/related_guard.rs:1200-1215`, "declare a `references` relation"). So the model uses a relation it was never told about, and says nothing.
3. Existing idiom: declaring the relation is the idiom (`entity.rs:807-848`, `docs/design/ess-entity-relations-design-v0.1.md` §2). Nothing reports that it is missing. Advisory severity exists, `Severity::Warning` "Legal, and probably not meant" (`crates/specify/ess-compiler/src/diagnostic.rs:17-23`), but nothing in the specification compiler produces it today, so there is no warning channel in use.
4. Fit, redesigned to what can be decided without guessing:
   - (a) A stored entity field typed exactly, or `Optional<…>`/`List<…>` of, the identity type of exactly one other entity, where that type is a declared named type (not a bare `Uuid`/`String`), and no `references` on the source or `owns` on the target carries it. Same resolver as `referenced_entity`, inverted. A bare-primitive identity is not linted, because many entities share `Uuid` and naming one would be a guess ("imports never guess", AGENTS.md Boundaries).
   - (b) A `when_related` through `via` (input or stored subject field) whose entity was settled by the identity-type fallback (`related_value.rs:106-117`) and not by a declared relation.
   - (c) Folds into (a). An aggregate groups by view fields drawn from its source entity's fields, so an unconnected group key is an (a) finding on that field. Row-set selectors `when_related: {entity, where}` are explicit queries and not relations, so they are not linted, and the guide says why.
   - Each finding is a `Severity::Warning` diagnostic with a path, a span and a hint ("declare `references` … `via: pool`"). It is printed by validate in text, and in JSON as `warnings[]` with the same shape as `diagnostics[]`. The exit status does not change. The lint lives beside `validate_relations` in ess-domain and is bridged by the compiler. No target, generator, synthesis or diff behaviour changes.
5. Second adopter: a warehouse specification stores `bin: BinId` on `Item` without `references`. A command guards on the bin's state through `when_related: {via: bin}`, and the relation to project into OpenAPI links and docs is missing from every artifact. Same fact, different domain.
6. Cost: no `ess/23` and no `ess-conformance/N` bump. There is no authored keyword. One new diagnostic class (e.g. `019 IMPLIED_RELATION`, used by `ENTITY` and `COMMAND`), the first producer of `Severity::Warning`, and an additive `warnings` field in the validate report (not a versioned format). Specifications that relied on the fallback start printing warnings, and their output changes on stderr only.
7. Alternatives: change nothing and leave it to review, which is what failed for the reporter. Make (a) a refusal: rejected, the request says advisory, and the fallback is legal semantics today. Heuristics by field name (`*_id` matching an entity name): rejected as guessing. Put findings into #434's `completeness` object: considered. Rejected because "probably not meant" is a diagnostic about authored text with a span, not an account of what synthesis cannot hold. Chosen: Warning-severity diagnostics, exact-type rules only.

## Decisions
accept, redesigned. Add an advisory implied-relation lint as `Severity::Warning` diagnostics in `ess specify validate`. Rule (a) covers exact named identity types with no carrying relation (aggregates included through their source fields). Rule (b) covers related guards settled by the identity-type fallback. Bare-primitive identities and row-set selectors are excluded and documented. One new diagnostic class. No format bump. It depends on #434 (edge recorded): both edit `validate` and `ValidationSummary` in `crates/edge/ess-cli/src/main.rs`, and this story adds `warnings` after #434 has added `completeness`. It depends on #448 (edge recorded): both edit `related_guard.rs` and `resolve.rs` in disjoint functions, and this story lands on #448's per-declaration predicate parse.

## Acceptance
- stored_named_identity_without_relation_warns: the committed fixture `crates/edge/ess-cli/tests/fixtures/implied-relation.yaml` (the fit-review model: `Agent.pool: PoolId` with no relation) validates, exits 0, and prints one `warning[ESS-ENTITY-019]` on `Agent.pool` naming `Pool` and the `references` to declare.
- declared_relation_silences_the_warning: adding `relations: [{name: pool, kind: references, target: Pool, cardinality: one, via: pool}]` prints no warning.
- owns_on_target_silences_the_warning: an `owns` relation carried by the field prints no warning.
- optional_and_list_identity_fields_warn: `Optional<PoolId>` and `List<PoolId>` fields warn.
- bare_primitive_identity_is_not_linted: entities identified by `Uuid` produce no warning for `Uuid` fields.
- related_guard_by_fallback_warns: `when_related: {via: input.pool}` with no relation warns on the command path; with the relation declared it does not.
- warnings_do_not_change_exit_or_ir: exit 0, compiled IR bytes unchanged, `warnings[]` in `--format json` carries code, path, span and hint.
- implied_relation_docs_state_the_rule: the `### Classes` table of `website/docs/reference/diagnostics.md` has a row `` `019` | `IMPLIED_RELATION` `` whose text contains "warning" and "`references`", and the validation-name table maps `implied_relation` to `019`. `website/docs/guides/specify/values-and-views.md` has the heading `### A relation the model only implies`, placed after the relation-resolution text (:122-144). The section contains "`ESS-ENTITY-019`", "warning", "exit status", "declare a `references` relation", "bare primitive" and "row set". Neither the row nor the heading exists today. A case of that name in `crates/edge/ess-cli/tests/validate_implied_relations.rs` reads both pages and fails naming the missing row, heading or phrase.

## Scope
- crates/specify/ess-domain/src/entity.rs  cited — `validate_relations`, `RelationSpec`, lint beside it
- crates/specify/ess-domain/src/command/related_value.rs  cited — `referenced_entity` fallback reused
- crates/specify/ess-domain/src/command/related_guard.rs  cited — rule (b): a new advisory function beside `validate` (891-973), reading `read_vias` (395-405) and `related_entity_via` (431-452) unchanged; after #448 (edge recorded), which changes `RawRelatedGuard` (93-121) only
- crates/specify/ess-compiler/src/diagnostic.rs  cited — `Severity::Warning`, class table
- crates/specify/ess-compiler/src/resolve.rs  cited — a warning sibling of `bridge` (969-994) for domain advisories, called from `diagnose_locating` (952-958); `class_of` (1125-1170) maps the new validation name to `019`. #448 edits `family_of_kind` (1025-1055) and `family_of` (1095-1116) and lands first (edge recorded); #450, #458 and #445 edit other functions in this file
- crates/edge/ess-cli/tests/fixtures/implied-relation.yaml  inferred — the fit-review model, committed
- crates/edge/ess-cli/src/main.rs  inferred — `validate` (2491-2561) prints warnings in text mode; `ValidationSummary` (4853-4870) gains `warnings`; after #434
- crates/edge/ess-cli/tests/validate_implied_relations.rs  inferred — scenarios above
- website/docs/reference/diagnostics.md  inferred — new class row
- website/docs/guides/specify/values-and-views.md  cited — relation resolution text (lines 122-144)
