---
format: aep.planning-md/1
id: story:review-expression-typechecking
kind: story
status: active
title: Resolve complete expression paths during validation
tags:
- P1
- review-2026-09-05
relations:
- decomposes: epic:review-boundary-remediation
- serves: vision:O2
scope:
- confidence: cited
  path: crates/specify/ess-compiler
- confidence: cited
  path: crates/specify/ess-domain
- confidence: cited
  path: crates/verify/ess-conformance
- confidence: inferred
  path: docs/design/review-expression-typechecking.md
revision: 11
---
## Finding and source

F09 (P1) from `docs/reviews/2026-09-05-architecture-review.md:348`, reviewed at `fd06a4d61bfb7b4990617810655dc181d6a3ab00`. Evidence locations: `crates/specify/ess-domain/src/command.rs:1316`, `crates/verify/ess-conformance/src/input.rs:395`. Findings are attributed to the review; proposed implementation choices below are coordinator inferences.

## Acceptance

A guard such as amount.nonexistent on a Decimal is rejected during specification validation with its full typed-path diagnostic.

## Implementation boundary

Move reusable full-path resolution and operand checks to domain/compiler validation and have conformance consume the result, keeping witness search and satisfiability separate. Inventory guards and other existing expression-bearing constructs so one resolver does not diverge across consumers.

## Validation

Reject invalid nested fields and operand types; accept legal nested, optional, collection and parameter paths per the current model. Retain a type-correct expression that a particular witness engine cannot synthesize to prove validation does not promise satisfiability.

Run package-scoped checks while implementing; the integration coordinator runs every step of `task check` and `task site-build` when required by `AGENTS.md`, retaining individual exit statuses and executed-case counts.

## Compatibility and exclusions

Do not broaden the predicate language or require a universal solver.

## Scope

Derived 2026-09-05 by an independent aep-drive:story-scoper reading the complete story, graph, declarations and consumers. No files changed and no builds ran during scoping. Coordinator records the returned scope below.

- `crates/specify/ess-domain` — cited; declarations, observable environments and Specification::validate.
- `crates/specify/ess-compiler` — cited; compilation admission, source diagnostics and resolved type access.
- `crates/verify/ess-conformance` — cited; existing deep-path resolver, authored expression admission and affected synthesis/witness regressions.
- `docs/design/review-expression-typechecking.md` — inferred; bind the shared checker, operand rules and capability boundary before its Rust API.
- CommandSpec::validate_guard (command.rs:1316) checks free-path roots; validate (:1487) already receives the type registry. Preserve the extra prohibition on an outcome guard reading the command-input identity naming its subject — cited.
- EntitySpec::validate (entity.rs:826) and check_nested (:925) partly check nesting, but primitive/collection/non-struct boundaries stop without a complete diagnostic and state bypasses deeper checks. The environment includes identity, fields and the synthesized lifecycle enum — cited.
- NamedType::check_invariants (types.rs:509) checks roots before a registry exists. Struct invariants see fields; newtypes see synthetic value; enums/unions have no invariant field — cited.
- ViewSpec::validate (:475), validate_params (:653), validate_filter_values (:710) separately check roots, parameters and enum literals. Filters see all source observables, including unprojected fields, plus param.<name>. Nested parameter reads currently evade typing and parameter-use accounting — cited.
- Specification::assemble (spec.rs:219) calls validate (:266), which builds the registry including lifecycle enums and delegates to owners. Add registry-aware named-type invariant validation there — cited.
- compile_locating (resolve.rs:742) revalidates before IR construction, protecting direct compilation of programmatically altered specifications. bridge (:543) already maps UnobservableFact and TypeMismatch to source diagnostics — cited.
- Conformance input.rs:395 owns resolve_path. Consumers include input refusal explanations, authored view satisfies (authored.rs:2023) and synthesis checks (:3127,3379,3398). Authored assertions are an additional admission surface whose environment is projected row fields. ViewExpectation::Satisfies (scenario.rs:2046) persists Predicate; preserve its shape and Unknown-does-not-pass runtime behavior — cited.
- Predicate already has Always/Never, All/Any/Not, comparisons, Truthy, Defined, AnyOf/NoneOf, Forall/Exists. Operands are FactPath or Bool/Number/Text. A bare dotted RHS is a path, an undotted bare word a literal. No grammar extension — cited.
- Predicate::fact_paths omits bound paths and quantified_collections omits collections rooted in outer binders. Recursively walk the existing tree with lexical binder environments; neither flattened list is sufficient — cited.
- Existing conformance traversal transparently unwraps Optional/newtypes, consumes struct fields and ends at primitive/enum scalar leaves. Entity validation admits List/Map quantification; union tag/member selection is not an established conformance path contract — cited.
- Bind existing collection conventions before implementation: primitives document .count, numeric element paths and nested binder rebinding, while conformance cannot project collection elements. Decide ESS interpretation and Map element meaning without inventing bracket/map-key syntax or union narrowing — inferred.
- Define compatibility for both fact operands, literals, membership lists and ordering separately from assignment. Number covers Integer and Decimal; existing Integer quantity == 0.5 is deliberately type-correct but unsatisfiable, and must remain admitted — cited.
- Settle nominal versus representation compatibility, enum membership, text-backed primitives, ordering without a scale and aggregate Defined/Truthy. Truthy currently has Bool/Number/Text semantics, not Boolean-only; text ordering may remain Unknown without scale — inferred.
- Terminate transparent named cycles without rejecting all recursive declarations. A finite path through recursive optional structs differs from a non-consuming newtype cycle; witness depth is not a specification type rule — inferred.
- Minimal reuse: ess-domain owns one semantic path/predicate checker behind a narrow read-only type-environment interface. Adapt domain declarations and compiler resolved types; conformance consumes the compiler adapter. Keep parsing in primitives, type policy in domain, handles/diagnostics in compiler and witness search/projection in conformance. No domain→conformance dependency, second semantic resolver or mirrored persisted Predicate — inferred.
- Shared results separate malformed paths/operands from legal paths unavailable to a projection. Conformance may retain scalar projection capability classification but consumes shared resolution. Missing optional values, scales and exhausted search remain runtime/conformance outcomes — inferred.
- Diagnostics retain owner location, complete original path, failing segment, resolved type and operand types. Locator uses textual needles and can locate declarations, not guarantee exact predicate leaf lines; test honest attribution — cited.
- CLI load.rs:105 bridges assembly failures and main.rs:1729/1768 renders text/structured errors and fails validate. No CLI source edit is established. ess-primitives language/evaluator and infra-spec's separate workload vocabulary are evidence-only; primitive representation, infrastructure predicates, assignment and solver work are excluded — cited/inferred.
- Red-first cases: Decimal amount.nonexistent at assembly; all owners; direct compiler revalidation; missing nested fields, continuation through scalar/enum, wrong comparison/membership kinds, undeclared/nested params, malformed quantifier target and invalid binder body — inferred.
- Positive cases: structs, transparent newtypes/optionals, source-only view fields, params, lifecycle enums, scalar truthiness, List/Map quantifiers, nested binders and free references therein. Test both operands without accidental RHS parser assumptions — inferred.
- Keep Integer versus 0.5 unsatisfiable case. Add satisfiable Decimal interval all:[amount > 0.1, amount < 0.2]: validation/compilation pass and explicit0.15 satisfies; current candidate ladder uses literals, ±1,0,-1 and base1 and misses that witness. This proposed execution result was not measured by the scoper — inferred.
- Migrate malformed TaxOrder amount.vat in conformance tests/witness.rs and tests/synthesis.rs:502 from late synthesis expectations to early admission. Retain defensive runtime Unknown coverage using directly supplied predicates against valid IR — cited.
- Baseline and isolated regressions first, then offline locked package tests, formatting and strict Clippy for the three packages. Preserve prior valid canonical bytes. Coordinator owns full integration and site gate — inferred.
- Confidence: high — cited; all ESS predicate owners, the defect and compiler/conformance paths identified. No complete normative operand/collection matrix, exact API/module names, measured baseline or precise predicate-leaf locations was established. Binding design must settle these before code.
- Collisions: domain command/entity/type/view/spec validation, compiler IR/resolve and diagnostics, conformance input/authored/synthesis/witness. Use all three exact package tokens and the proposed design token. CLI/primitives stay evidence-only until an actual edit is established — inferred.


## Bound Wave7 implementation

The coordinator accepts docs/design/review-expression-typechecking.md before source edits under the user's standing approval for all remediation waves. The complete original Scope above is retained. The following independent refresh remains attributed proposal text; its concrete design/scope/acceptance choices are now accepted for implementation. Planned cases are not executed evidence. Source pin e113a65 is contained in published combined fadbc674, whose complete ten-lane gate passed1768 Rust cases at2026-09-06T00:53:01Z. That gate is the opening baseline, not a result for this implementation.

# Scope refresh delta — review-expression-typechecking

Read-only refresh initially at coordinator `1b1c4a3a424e3304e503747422a0c3d10d86928a`, now verified against fresh main `e113a65a0bac63e77cd17f43fa280a5bf56c93f9`, story revision 7. Preserve the existing detailed Scope in place; `scope-as-read.md` is its verbatim snapshot. This file proposes an appended refresh rather than replacement. No tracked/store/ref edits or builds ran.

## Source drift since the review baseline

Compared source to the cited original review commit `fd06a4d61bfb7b4990617810655dc181d6a3ab00`.

```text
 crates/specify/ess-compiler/src/graph.rs           |  74 +++++-
 crates/specify/ess-compiler/src/resolve.rs         |   2 +
 crates/specify/ess-compiler/tests/wire_fields.rs   | 171 ++++++++++++
 crates/specify/ess-domain/src/lib.rs               |   1 +
 crates/specify/ess-domain/src/spec.rs              |   2 +-
 crates/specify/ess-domain/src/wire.rs              | 102 +++++++
 crates/verify/ess-conformance/src/evidence.rs      | 295 ++++++++++++++++++++-
 .../tests/report_reader_adversary.rs               | 194 ++++++++++++++
 8 files changed, 836 insertions(+), 5 deletions(-)
```

- **Cited actual drift:** Specification::validate still begins at spec.rs:266 but now initializes errors from `crate::wire::validate(self)`. Preserve that validation and its accumulation when inserting the registry-aware invariant pass. New wire.rs validates effective wire namespaces, including entity state/identity and view params/shape fields; expression lookup continues to use semantic field names. No wire.rs edit is needed.
- **Cited actual drift:** resolve.rs adds `params` and `shape` to diagnostic STRUCTURAL needles. Preserve those entries. compile_locating is now :744 (formerly :742); bridge remains :543 and class mapping :602–605. No locator replacement or exact-leaf-location promise follows from this drift.
- **Cited actual drift outside this implementation:** compiler graph.rs and wire_fields.rs tests changed; conformance evidence.rs/report_reader_adversary.rs changed. They do not alter the expression declarations, checker/evaluator call sites or witness candidate algorithm and stay outside the proposed edits.
- **Cited unchanged source:** command.rs, entity.rs, types.rs, view.rs, primitives predicate.rs/facts.rs, compiler ir.rs, and conformance input.rs/authored.rs/synthesize.rs/witness.rs have no diff over this range. The original complete inventory still applies.
- **Citation refresh only:** current NamedType::check_invariants is types.rs:512, not :509; CLI validate is main.rs:1782 and compile :1809. load.rs assembly bridge remains :105. These line updates do not add CLI scope.

## Binding refinements, not claims of concurrent code drift

The additional main refresh from 1b1c4a3 to e113a65 changes no byte anywhere in ess-domain, ess-compiler, ess-primitives or ess-conformance: all four complete directory tree objects match. New Go normalization implementation and CLI exposure are outside those tokens. CLI validate/compile remain at main.rs:1782/:1809; assembly diagnostics remain load.rs:105/:108. Exact tree IDs are retained in source-correspondence.md. No further expression scope drift results.

The original Scope explicitly left these choices inferred. The separate binding proposal settles them without deleting their inventory lines:

1. Same evaluator-kind compatibility for comparison, distinct from nominal assignment; enum fact/literal membership checks; Text ordering stays type-correct without a scale; Number includes Integer == 0.5.
2. Scalar Truthy and Defined only; optional scalar presence remains runtime behavior.
3. List count/canonical ordinal selectors; Map count and value quantification, with no direct Map key/ordinal selector or entry record; lexical binders and parameter use are checked through the actual AST.
4. Non-progress-cycle detection by type identity plus path position; no semantic 32-step path limit or new global recursive-type rejection.
5. A domain checker and resolved compiler adapter return typed reads plus structural projection requirements; conformance consumes them without claiming collection projection support.
6. Existing compiler diagnostic envelope/heuristic location limits remain; one new in-memory authored operand-error cause (proposed ESS-AUTHOR-035) distinguishes bad typing from existing unreadable-path cause 026.

## Exact scheduling and file delta

Keep the four existing machine-readable scope tokens unchanged:

- `crates/specify/ess-domain` — cited
- `crates/specify/ess-compiler` — cited
- `crates/verify/ess-conformance` — cited
- `docs/design/review-expression-typechecking.md` — inferred

Within those tokens, inferred concrete additions are `ess-domain/src/expression.rs`, `ess-compiler/src/expression.rs`, their existing lib.rs exports, and focused expression tests in the three packages. Existing command/entity/types/view/spec owner files and conformance input/authored/synthesize plus witness/synthesis fixture tests remain in the original assignment.

Compiler ir.rs/resolve.rs/diagnostic.rs stay in the collision inventory; consume public resolved types and preserve bridge revalidation without requiring an IR/schema change. Conformance scenario.rs and runner.rs are evidence/compatibility surfaces, not a request to change persisted predicates or execution. No new CLI, primitives, infra-spec, wire, graph, solver, projection or format token is proposed.

Root should bind the reviewed design through the store and owned design document before dispatch; this proposal creates neither an approval nor a lifecycle claim.
