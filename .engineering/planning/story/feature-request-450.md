---
format: aep.planning-md/3
id: story:feature-request-450
kind: story
status: active
title: Declared reference rows
tags:
- ess-0.54.0
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#450
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
- depends_on: story:feature-request-429
- depends_on: story:feature-request-426b
- depends_on: story:feature-request-448
- depends_on: story:feature-request-445
- depends_on: story:feature-request-455
scope:
- confidence: inferred
  path: crates/generate/ess-gen/src/docs.rs
- confidence: inferred
  path: crates/generate/ess-gen/src/model_types.rs
- confidence: cited
  path: crates/generate/ess-gen/src/types.rs
- confidence: inferred
  path: crates/specify/ess-compiler/src/ir.rs
- confidence: inferred
  path: crates/specify/ess-compiler/src/resolve.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/finite.rs
- confidence: cited
  path: crates/specify/ess-domain/src/expression.rs
- confidence: cited
  path: crates/specify/ess-domain/src/types.rs
- confidence: inferred
  path: crates/specify/ess-domain/tests/enum_attributes.rs
- confidence: cited
  path: crates/verify/ess-diff/src/change.rs
- confidence: cited
  path: docs/design/enum-variant-wire-names.md
- confidence: cited
  path: website/docs/guides/specify/fields-and-invariants.md
revision: 10
transitions:
- {from: "draft", to: "proposed", at: "2026-10-05T20:40:14Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"review_outcome":5}}}
- {from: "proposed", to: "active", at: "2026-10-05T20:40:14Z", actor: "human:timo", revision: 10, decided_on: {"recorded":{"review_outcome":5}}}
---
## Outcome
Resolve beyond10x/ess#450: Declared reference rows.

## Origin
beyond10x/ess#450, filed 2026-10-05; reported from an adopter's catalogue of 107 static rows and 18 operator rows, each row with attributes (UI type, value type, requirements), which are modelled today as enum variants plus comments.

## Fit review
1. Need: a closed, fixed set of keys where each key carries typed attributes known when the specification is written ("`GreaterThan` takes an `Integer` value"; "plan `Team` allows 50 seats"). The attributes must be stated once, checked, readable in guards and view filters, and carried into projections. The requester's proposed shape is "rows of a declared type, read-only, addressable by key, usable in guards and views, carried into projections". Minimal reproduction: `<fit-review scratch>/probe-450/` (installed ess 0.52.0). An enum `Operator` whose variants carry `display`/`summary` validates. Synthesis writes 2 scenarios with 0 refusals for the guard `operator: {any_of: [GreaterThan, LessThan]}` standing for "takes a number". Adding `attributes: {value_type: Integer}` to a variant is refused with `unknown field 'attributes'`.
2. Class: gap. An enum variant's object form carries only `Naming` (`wire`, `display`, `summary`, `code`; `crates/specify/ess-domain/src/types.rs:651-656`, `docs/design/enum-variant-wire-names.md:21-40`). No construct states a typed per-key fact. Every guard over an attribute can be rewritten as `any_of` over the variants that have it, so guards alone are a convenience. But the attribute itself is unstated, so nothing checks that the rewritten sets agree, and no projection carries it.
3. Existing idiom: the probe's `any_of` membership plus a `summary:` per variant. It works for guards, but repeats the table in every guard and leaves it as prose. An entity of reference rows is not an idiom: ESS declares no rows, and seeds exist only for conformance arrangement, not as specification data (`docs/design/synthesis-seeds.md:3-7`).
4. Fit, redesigned onto the enum that already holds the keys, without a new row noun:
   - Declaration: the enum declares `attributes: [{name, type}]`, the same `{name, type}` shape as `fields:`. Each variant's object form gains `attributes: {<name>: <literal>}`, checked by the existing literal rule (`crates/specify/ess-domain/src/command.rs:4426` `literal_representation`). Types are scalars, String newtypes, enums and `Optional` of these. `List<…>` is refused by name in this cut. Every variant fills every non-Optional attribute.
   - Reading: in predicates (guards, invariants, view filters) a path `<enum fact>.<attribute>` resolves through `resolve_path` (`crates/specify/ess-domain/src/expression.rs:584`, segment refusal at `:813`). The compiler lowers a comparison with a literal to membership over the variants that satisfy it, so runners, suites and Entity Runtime see only `any_of` and need no new evaluator. A comparison of a fact with an attribute expands per variant within the existing finite bounds (`crates/specify/ess-domain/src/command/finite.rs:10-12`, 64 assignments / 128 nodes) and is refused by name beyond them. A 107-variant enum exceeds 64 for the coverage proof, so commands guarded on it keep a default branch (`finite.rs:151`).
   - Projections: JSON Schema/OpenAPI annotate the enum with `x-ess-attributes` (`crates/generate/ess-gen/src/types.rs:791`, beside `x-ess-alphabet` at `:301`). Generated docs get a table, and types-only Rust/Go/TypeScript get a per-variant accessor (`crates/generate/ess-gen/src/model_types.rs`, inferred). Reading an attribute as a value (`sets:`, view field) is refused by name in this cut.
   - `ess verify diff` classifies attribute declared, removed and value changed (beside `VariantAdded`/`VariantRemoved`, `crates/verify/ess-diff/src/change.rs:828-835`). A changed value that moves a guard's lowered set is breaking.
   - The boundary rules out a property bag (`AGENTS.md:18-19`). Attributes are declared and typed per enum, never free-form.
5. Second adopter: subscription plans `Basic`/`Team`/`Enterprise` with `max_seats` and `sso: Boolean`. `EnableSso` is refused when `plan.sso == false`, and a view filter selects accounts on SSO-capable plans. Unrelated domain, same construct.
6. Cost:
   - source format ess/23 for the enum `attributes:` declaration, the variant `attributes:` map and the attribute path;
   - new diagnostics, reusing `type_mismatch`, `missing_declaration` and `undeclared_reference`;
   - new diff kinds;
   - an `x-ess-attributes` annotation, plus generated accessor API (additive);
   - no suite format, because lowering to membership keeps suites carrying variant names;
   - no IR change for models that do not use it.
   Size L.
7. Alternatives: (a) change nothing: `any_of` sets repeated per guard, unchecked. (b) As proposed, a new reference-rows noun: it duplicates the enum's closed key set and the entity's fields, and needs row seeding in every target and in synthesis. Refused. (c) Chosen: typed attributes on enum variants, lowered to membership in predicates. (d) Defer: possible if the operator wants to wait for a second request. The plan-tier example suggests the need is general.

## Decisions
Accept, redesigned: typed per-variant enum attributes (ess/23). An enum declares `attributes: [{name, type}]`, each variant fills them with typed literals, and predicates read `<fact>.<attribute>`, lowered to variant membership. Projections carry them as `x-ess-attributes`, a docs table and generated accessors. Value-position reads and `List` attributes are refused by name. Rejected from the request: a separate reference-row noun. Format: `ess/23`, introduced by #429. This story depends on #429 (edge recorded), gates on `FormatVersion::V23` and does not edit `system.rs` or `spec-versions.md`: report the one sentence for the `ess/23` row under `## Changelog lines`, and the coordinator merges it into the row #429 writes. It also depends on #426b (edge recorded): #426b adds `visit_bool` to the `EnumVariant` visitor (`types.rs:778-790`), and this story extends the same variant object form after it. Further edges (recorded): after #448, whose per-declaration predicate parse is where the attribute path is read; after #445, which raises `literal_representation` (`command.rs:4426`) to `pub(crate)` and leaves its signature to this story's body change; after #455, which edits `fields-and-invariants.md:35-46` before this story adds its section further down the page. The guide section is `### Give enum variants typed attributes` in `website/docs/guides/specify/fields-and-invariants.md`, after `### Cover every declared enum value` (79-95). No ess-conformance bump.

## Acceptance
- enum_attributes_declared_and_filled_validate: an enum with `attributes:` whose variants fill each one validates; a missing, extra or mistyped value is refused by name.
- enum_attribute_guard_lowers_to_membership: `operator.takes_number == true` compiles to the same IR membership as `operator: {any_of: [GreaterThan, LessThan]}`, and synthesis witnesses both sides.
- enum_attribute_fact_comparison_expands_within_bounds: `seats.count >= plan.max_seats` expands per variant; past 128 nodes it is refused by name.
- enum_attribute_view_filter_and_invariant: the same path works in a view filter and an invariant.
- enum_attribute_value_position_refused: `sets: {x: input.plan.max_seats}` is refused naming the cut.
- enum_attributes_below_ess_23_refused: `unsupported_format_version`.
- enum_attributes_projected: JSON Schema `x-ess-attributes`, the docs table and Rust/Go/TypeScript accessors; existing enum bytes are unchanged without attributes.
- enum_attribute_diff_classified: value changes that move a lowered guard set are breaking.
- enum_attribute_guide_section_states_the_construct: `website/docs/guides/specify/fields-and-invariants.md` has the heading `### Give enum variants typed attributes`, after `### Cover every declared enum value`. The section contains "`attributes:`", "`ess/23`", "lowered to membership", "`x-ess-attributes`" and "refused" for a value-position read, and its fenced model validates. A case of that name in `crates/specify/ess-domain/tests/enum_attributes.rs` reads the page and fails naming the missing heading, phrase or invalid model.
- ess_23_row_names_enum_attributes: the coordinator's merged `ess/23` row of `website/docs/reference/spec-versions.md` names typed enum-variant attributes. Checked at integration by the coordinator, not by a unit test.

## Scope
- crates/specify/ess-domain/src/types.rs  cited — `EnumVariant` (651-656) and its visitor (778-790, after #426b), `TypeBody::Enum` (932)
- crates/specify/ess-domain/src/expression.rs  cited — `resolve_path` (584), segment refusal (813)
- crates/specify/ess-domain/src/command/finite.rs  cited — bounds (10-12, 151)
- crates/specify/ess-domain/src/command.rs  cited — the body of `literal_representation` (4426-4525) checks an attribute literal; its signature and the `pub(crate)` visibility are #445's, which lands first (edge recorded)
- crates/specify/ess-compiler/src/resolve.rs  inferred — `body` (1679-1739) carries the enum's attributes into the resolved enum; one new lowering helper rewrites an attribute comparison to variant membership and is called from `condition_of` (5142-5235, outcome guards), `entities` (3591-3649, invariants) and `views` (3698-3781, filters)
- crates/specify/ess-compiler/src/ir.rs  inferred — resolved enum body carries attributes
- crates/generate/ess-gen/src/types.rs  cited — enum schema (791), annotation beside `x-ess-alphabet` (301)
- crates/generate/ess-gen/src/model_types.rs  inferred — generated accessors
- crates/generate/ess-gen/src/docs.rs  inferred — attribute table
- crates/verify/ess-diff/src/change.rs  cited — variant change kinds (828-835)
- docs/design/enum-variant-wire-names.md  cited — variant object form being extended
- website/docs/guides/specify/fields-and-invariants.md  cited — new `### Give enum variants typed attributes` after `### Cover every declared enum value` (79-95); #455 edits 35-46 first (edge recorded)
- crates/specify/ess-domain/tests/enum_attributes.rs  inferred — scenarios above
