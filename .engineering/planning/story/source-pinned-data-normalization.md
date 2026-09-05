---
format: aep.planning-md/1
id: story:source-pinned-data-normalization
kind: story
status: active
title: Source-pinned checked data normalization across Go Rust and TypeScript
relations:
- serves: vision:O2
scope:
- confidence: cited
  path: CHANGELOG.md
- confidence: cited
  path: crates/edge/ess-cli/src/main.rs
- confidence: cited
  path: crates/edge/ess-cli/src/normalize.rs
- confidence: cited
  path: crates/edge/ess-cli/src/schema.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/normalization.rs
- confidence: cited
  path: crates/generate/schema-contract/Cargo.toml
- confidence: cited
  path: crates/generate/schema-contract/src/realize.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/diagnostic.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/check.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/eval.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/execute.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/input.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/recipe.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/rust_runtime.rs.txt
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/target.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/rust.rs
- confidence: cited
  path: crates/generate/schema-contract/tests/fixtures/normalization_rust_tests.rs.txt
- confidence: cited
  path: crates/generate/schema-contract/tests/normalization.rs
- confidence: cited
  path: crates/generate/schema-contract/tests/normalization_rust.rs
- confidence: cited
  path: docs/design/source-pinned-data-normalization.md
- confidence: cited
  path: website/docs/guides/generate-artifacts.md
- confidence: cited
  path: website/docs/reference/cli.md
revision: 4
---
## Evidence

`crates/generate/schema-contract/src/realize.rs::Plan` realizes checked shapes and reports obligations; JSON Schema default annotations are not applied. Native untagged unions use structural branch decoding, not an external application discriminator. `crates/specify/ess-domain/src/types.rs::Conversion` declares allowed from/to crossings and a reason, not executable field conversion, defaulting or branching.

Source-driven adoption established transformations that cannot be expressed by copying a schema: dispatch selected outside the settings object; distinct absent/null/default behavior; ordered preinitialization, decoding, normalization, defaulting and validation; integer-unit scaling into durations; ordered matching branches; and counts derived from distinct enabled collection categories. Handwritten consumer adapters currently own these meanings. Renaming a stored type to a normalized type would erase them.

## Outcome

ESS describes and checks source-pinned data normalization between concrete input/output representations and provides consistent supported realization across Go, Rust and TypeScript without pretending a structural schema or permitted nominal conversion implements the transformation.

## Acceptance

For an explicitly declared normalization pipeline, ESS preserves input/output identity and stage order, checks dispatch/field types/default and branch semantics, realizes supported operations consistently in all three targets, and refuses unsupported operations at their source locations before producing a successful partial adapter.

## Scope

Extend the existing qualified bundle/model selection and typed realization boundaries only after a binding design establishes concrete operations, provenance and compatibility. Inputs and normalized outputs must be separately typed. Cover external dispatch, presence/null handling, ordered defaulting and validation, exact unit conversions, branch precedence and collection-derived values from generic source-grounded fixtures. JSON Schema annotations remain annotations unless an explicit transformation stage adopts them. No generic facet registry, arbitrary behavior property bag, source-language guessing, invented lifecycle service or live data migration.

## Relation

This is the concrete normalization capability gap left open by `story:types-only-realizations`, not a replacement for its full objective. `story:schema-document-root-import` restored an already-existing enclosing type; it did not establish transformations between different application representations. Consumer-specific field inventories stay in adopter repositories; public fixtures and implementation remain generic.

## Binding Design

`docs/design/source-pinned-data-normalization.md` binds external dispatch, ordered schema-checked stages, explicit source-root identity, field/missing/null semantics, signed integer arithmetic with explicit overflow, ordered conditions, collection mapping and distinct-key counting. `ess-normalization/1` is a strict authored behavior envelope, not schema annotations or a new domain IR. Every branch must check before a sealed plan exists.

Implementation begins with typed recipe identity and reference execution in the existing schema-contract realization boundary; it must continue to common checked-plan realization in Go, Rust and TypeScript. This is sequencing, not reduced acceptance. The story remains active until all targets and source-driven normalization requirements are fulfilled. Scope: schema-contract realization submodules/tests, binding design, CLI/public documentation once the executable path is available, and changelog.

## Checked Reference Execution

`crates/generate/schema-contract/src/realize/normalize.rs` now owns the concrete
`Recipe`, `Stage`, source-pinned `Root`, typed expressions/conditions and sealed
`Plan`. `Plan::read` refuses unknown fields, duplicate dispatch/record keys and
unsupported formats. Every branch checks before a plan is returned. Adjacent roots
must agree by full canonical bundle identity and admitted name. `Plan::run` validates
input and output at every stage, evaluates requirements in order, returns no partial
successful value and never mutates caller data.

`normalize/check.rs` reuses the shared structural plan. Nullable object/list fallbacks
remain usable after explicit normalization; union checking retains presence and
declared field identity. Typed additional properties cannot bypass a named output
field's type. Unbound item scopes, unknown paths, missing required outputs and
unsupported tuple/intersection/expanded recursive shapes refuse. Schema refinements
remain runtime boundary checks. Equality is explicitly scalar null/boolean/string/
signed-integer, not target-dependent structural JSON equality. Integer operations
accept exact integral signed-64-bit representations with explicit reject/wrap
overflow. No schema number canonicalization or existing bundle bytes change.

`normalize/eval.rs` implements pure ordered execution: omission versus null, lazy
fallback and choice, signed arithmetic, list mapping with lexical item scope, and
distinct selected string-category counts. Eleven generic integration tests in
`tests/normalization.rs` exercise source identity, strict parsing, boundary validation,
explicit dispatch and aliases, stage order, caller immutability, absence/null/zero/
false/empty distinctions, map order, category counts, scalar equality and exact
integer rejection/overflow. These are reference-engine checks, not evidence of
source-adapter parity or three-target realization.

Verification: `cargo test --offline -p schema-contract --test normalization` passed
11 tests; strict package Clippy passed; `CARGO_NET_OFFLINE=true task check` exited 0;
`task site-build` exited 0. The documentation build still reports 30 dependency audit
findings (9 moderate, 21 high), unchanged from the preceding build. Public generation
documentation and the unreleased changelog explicitly state the library-only boundary.

Remaining acceptance is unchanged: common checked-plan adapter generation in Go,
Rust and TypeScript, CLI integration with deterministic provenance, source-grounded
input/output contracts and complete consumer normalization adoption. No release,
commit, push or consumer cutover was performed. Keep this story active.

## String Prefix Gap

Source-driven adapter review requires case-sensitive prefix selection to distinguish
two target representations. The current `Condition` enum in
`crates/generate/schema-contract/src/realize/normalize.rs` has scalar equality and
integer ordering but cannot express prefix matching. This is a concrete missing
normalization operation within this story, not authority to substitute a URI parser
or to enumerate a few observed values as equality branches.

Extend the binding design with `starts_with`: both operands must be present non-null
strings; comparison is case-sensitive over exact UTF-8 strings, with no trimming,
case folding, normalization or URI interpretation. Empty prefix matches every string.
Keep generic examples in this public repository. Source-specific routing names and
the immutable implementation reference remain in the adopter's normalization chapter.
The same operation must be realized by all eventual targets. Reference execution
alone remains insufficient to mark this story implemented.

## CLI And Text Boundary

`crates/edge/ess-cli/src/normalize.rs` exposes `schema normalize-check` and
`schema normalize-run`, under both area and existing flat spellings. The former
checks all branches and emits canonical recipe JSON; the latter requires explicit
branch/input selection and emits the complete resulting JSON value. Every supplied
bundle is replay-checked. Existing output preflight rejects source replacement,
symlinks and hardlinks; recipe/bundle/instance inputs are protected. This uses the
existing writer, not a new transactional filesystem or crash-rollback guarantee.
Semantic refusals happen before that writer is called.

`Plan::run_json` uses `normalize/input.rs` to inspect raw JSON values before numeric
conversion. Duplicate object keys, trailing data, excessive nesting and lossy numeric
conversion refuse before execution. Decimal significand/exponent comparison detects
rounding and underflow without using binary floating-point equality as its oracle.
Signed-range integer tokens, including negative zero, stay integral; exponent and
fraction spellings remain nonintegral representations for integer operations.
The serde_json raw_value feature does not enable arbitrary_precision or change
existing bundle serialization. The value-based `Plan::run` API still leaves decoding
precision to its caller; raw JSON consumers use the checked text edge.

The previously recorded string-prefix gap is implemented by `Condition::StartsWith`
in the shared checker/evaluator. Both operands must be present non-null strings;
execution preserves exact case, whitespace and Unicode representation. Prefix
matching does not become URI validation or finite equality dispatch.

Verification after all changes: `CARGO_NET_OFFLINE=true task check` exited 0;
`task site-build` exited 0; focused normalization suites passed 15 library tests and
5 CLI tests. The command-surface test now enumerates 50 area leaves and their aliases.
The documentation dependency audit still reports 30 findings (9 moderate, 21 high).
`git diff --check` exited 0. Public CLI reference, generation guide and unreleased
changelog document the checked workflow and its remaining boundary.

The CLI gap is now implemented locally, but this story remains active. Common
checked-plan Go/Rust/TypeScript adapters, source-derived consumer contracts and full
normalization adoption remain required. No adopter-specific partial adapter is
claimed as complete; no release, commit or push was performed.

## Standalone Rust Target

The checked plan now offers `Plan::rust(package)` as a library API. It emits an
independent Cargo data-normalization library using the same recipe, execution,
expression, input-parsing and diagnostic sources as the reference engine, with
embedded checked root schemas and no ESS or AEP runtime dependency. It validates
input/output boundaries and returns no partial successful result.

The new `ess-normalization-target/1` report records canonical recipe SHA-256,
complete bundle/root identities, exact emitted schema digests and every emitted
file digest except the report itself. The binding design describes the standalone
target and its explicit scope. Go and TypeScript normalization targets are still
pending; structural data libraries already exist in all three languages.

Raw JSON input parsing is now independent of a consumer enabling serde_json's
arbitrary_precision feature: integer tokens retain exact signed/unsigned integer
representation, fractional/exponent tokens retain the reference floating boundary,
and inexact values refuse. Generated-library tests execute the same reference cases
under default features and consumer-enabled arbitrary_precision.

Verification on the original feature worktree: 15 normalization reference tests,
2 standalone Rust target tests including both generated-crate feature modes, strict
schema-contract Clippy and the full `CARGO_NET_OFFLINE=true task check` all passed.
This does not prove source-specific application parity or all-target normalization.
No Rust adapter-generation CLI or release/publication is claimed. Keep this story
active until the unchanged three-target and consumer-normalization acceptance holds.


## Integration Provenance

Reconciled through AEP from wt-46ef382d9f07 at original revision 11 and status active. Source artifact SHA-256: 5b8b6ca27e75418c9fa0a05dbfdac625d6ae6c13aa4031f8edbef7602313434a. Original journal history remains with its source recovery snapshot; this store records the reconciliation as new governed operations.
