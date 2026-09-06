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
  path: crates/generate/schema-contract/src/realize/go.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/check.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/eval.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/execute.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/go.sum.txt
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/go_collection.go.txt
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/go_condition.go.txt
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/go_expression.go.txt
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/go_input.go.txt
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/go_numeric.go.txt
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/go_runtime.go.txt
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/go_target.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/input.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/numeric.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/recipe.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/rust_runtime.rs.txt
- confidence: inferred
  path: crates/generate/schema-contract/src/realize/normalize/source.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/target.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/rust.rs
- confidence: cited
  path: crates/generate/schema-contract/tests/fixtures/normalization_go_tests.go.txt
- confidence: inferred
  path: crates/generate/schema-contract/tests/fixtures/normalization_model.rs
- confidence: cited
  path: crates/generate/schema-contract/tests/fixtures/normalization_numeric.rs
- confidence: cited
  path: crates/generate/schema-contract/tests/fixtures/normalization_rust_tests.rs.txt
- confidence: cited
  path: crates/generate/schema-contract/tests/fixtures/normalization_v1.rs
- confidence: cited
  path: crates/generate/schema-contract/tests/fixtures/normalization_v2.rs
- confidence: cited
  path: crates/generate/schema-contract/tests/normalization.rs
- confidence: cited
  path: crates/generate/schema-contract/tests/normalization_go.rs
- confidence: inferred
  path: crates/generate/schema-contract/tests/normalization_model.rs
- confidence: cited
  path: crates/generate/schema-contract/tests/normalization_numeric.rs
- confidence: cited
  path: crates/generate/schema-contract/tests/normalization_rust.rs
- confidence: cited
  path: crates/generate/schema-contract/tests/normalization_v2.rs
- confidence: cited
  path: docs/design/source-pinned-data-normalization.md
- confidence: cited
  path: website/docs/guides/generate-artifacts.md
- confidence: cited
  path: website/docs/reference/cli.md
- confidence: cited
  path: website/docs/reference/formats.md
revision: 15
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

## Additional Source-Fidelity Gaps After 0.19.0

The adopter's continued immutable-source audit identified operations that the
current recipe cannot represent. This is added to the existing unchanged
three-target acceptance, not filed as a smaller replacement for it.

1. Explicit floating representation and conversion order. A source decoder admits
   ordinary JSON numbers into binary64, clamps some values, multiplies by a unit
   scale, truncates toward zero into a signed duration, then performs another
   integer scale. Expr::Arithmetic and Condition::Greater currently admit only
   signed integers. normalize/input.rs intentionally rejects lossy JSON conversion.
   Neither exact decimal multiplication nor silently broadening that text reader
   preserves the observed pipeline. The binding must declare numeric decoding,
   rounding, operation order, truncation and range behavior explicitly; retain
   the existing exact policy for recipes that did not request another policy.
   Out-of-range host conversions need a supported contract or a named refusal,
   not a guessed portable result.

2. Ordered string construction. A source conversion maps list entries to exact
   strings, concatenates them in source order (including duplicate and empty
   contributions), and conditionally surrounds the result with literal text.
   Expr currently has literals and prefix comparison, but no concatenation/join.
   Add concrete typed operations with no hidden sorting, deduplication, regex
   interpretation or escaping. Empty-list and empty-string behavior must be bound.

3. Filtered indexed collection construction. A source conversion conditionally
   prepends one element, filters later entries, and constructs a key from each
   entry's original zero-based index. Expr::Map preserves every input element and
   Scope exposes only input/item values; there is no index, filtered map, list
   concatenation or integer-to-string operation. Preserve original indices after
   filtering and specify lexical index scope for nested collections. Do not
   replace the behavior with dense renumbering or missing list elements.

4. Non-JSON runtime behavior must retain a typed home. A source converter constructs
   ordered equality-match callbacks excluded from JSON; first match wins and
   duplicate keys remain meaningful. A structural JSON snapshot loses them.
   Define a concrete declarative equality-dispatch representation and its lowering
   boundary before claiming complete normalization. Do not serialize arbitrary
   source code, introduce generic callback property bags, or claim a native closure
   is equivalent to an unbound string. The declaration and execution obligations
   must remain visible until the target realizes them.

Implementation must first revise docs/design/source-pinned-data-normalization.md,
including strict old-reader compatibility and format-version consequences. The
currently published ess-normalization/1 is not silently extended with changed
number admission or operation meanings. Every admitted operation must be checked
and realized consistently in the reference engine and Go, Rust and TypeScript;
unsupported operations must refuse before successful artifact publication.

Code evidence: crates/generate/schema-contract/src/realize/normalize/recipe.rs
declares the closed Expr/Condition/Scope variants; normalize/input.rs enforces the
exact JSON boundary; normalize/check.rs checks only those variants; target.rs
currently offers the standalone Rust target. Source-specific field names and
immutable implementation citations remain in the adopter's normalization chapter.
No consumer conversion parity or implementation of these additions is claimed.

## Ordered Operations Checkpoint

The binding design now declares ess-normalization/2 for seven concrete additions:
concat, join, integer_string, concat_lists, item_index, select_map and find.
The reference checker/evaluator and standalone Rust target implement them. String
and list construction preserves source order, duplicates and empty values. Filtered
mapping retains original indices; nested collection scopes rebind the index.
First-match selection skips later items and evaluates fallback only on no match,
in the enclosing scope. Every operand still checks before execution.

The version 1 checker refuses each new operation at its expression pointer with
operation_version. Existing version 1 canonical bytes are unchanged: its generated
Rust fixture canonical recipe has SHA-256
8c99019a9eb7111f0e025772d1c29a82acea2215c1fcaf0b6200f825bcca611b,
independently reproduced byte-for-byte by the published 0.19.0 reader and pinned
in tests/normalization_rust.rs. The published reader refuses the version 2 fixture
with recipe_syntax: unknown variant `join`. New readers retain strict unknown-format
refusal and do not broaden numeric admission.

Verification on the final local code:
- CARGO_NET_OFFLINE=true CARGO_BUILD_JOBS=4 task check: exit 0, including formatting,
  strict Clippy, workspace tests, rustdoc, command smoke checks and projection checks.
- CARGO_BUILD_JOBS=4 task site-build: exit 0, including the browser/WASM lab and
  Docusaurus build.
- Normalization suites: 15 existing reference tests, six version 2 checks and three
  standalone Rust target tests. Generated crates execute both old/new recipes under
  default and consumer-enabled serde_json/arbitrary_precision feature configurations.
  The new target runs 33 cases, including all three-item filter combinations,
  first-match versus selected overflow, empty collections and integer boundaries.
- git diff --check: exit 0. Accidental workspace-formatter changes to generated
  sample projections were removed; no unrelated generated bytes remain changed.

New operation coverage is reference plus Rust only. Binary64 decode/scale/truncate,
Go and TypeScript normalization targets, adapter-generation CLI and the concrete
declarative runtime dispatch/consumer lowering contract remain open. Generic public
documentation labels version 2 unreleased. No new release, commit, push or source
adapter parity is claimed by this checkpoint. This story remains active with its
original three-target acceptance unchanged.

## Declared Binary64 Conversion Checkpoint

The unreleased version 2 design now additionally binds branch-specific numeric
input paths and ordered binary64-to-integer conversion. The optional binary64_inputs
map is omitted from existing recipe serialization, so the released version 1
canonical fixture digest remains unchanged. Version 1 refuses the declaration,
including an empty map; null is not accepted as an absent map. Typed field/items
paths are checked against each branch's first source root, including duplicate,
unknown branch/field and nonnumeric-leaf refusals. Nullable/optional containers
retain their actual presence.

Only selected raw numeric tokens decode with standard binary64 nearest-even
rounding. Other tokens retain exact admission, including otherwise unused values.
Signed underflow is preserved; infinity refuses. binary64_to_integer executes
finite multiply/minimum/maximum steps in declaration order, preserving signed-zero
clamp semantics, then explicitly rejects outside [-2^63,2^63) before truncating.
Later integer arithmetic still independently declares reject or wrap. Authored
constant tokens remain strings, checked before any branch execution.

The reference engine, normalize-check/normalize-run CLI and standalone Rust target
share this implementation. Generated Rust also exposes normalize_value for a
decoded-value input, with the same caller-owned precision qualification as Plan::run.
No source runtime or general schema numeric serialization was changed.

Verification:
- CARGO_NET_OFFLINE=true CARGO_BUILD_JOBS=4 task check: exit 0.
- CARGO_BUILD_JOBS=4 task site-build: exit 0.
- Five numeric integration tests, two numeric unit tests and actual generated Rust
  execution of 36 numeric cases under default and arbitrary_precision JSON features.
  Existing 15 reference cases, six ordered-operation checks and the version 1
  canonical SHA-256 regression remain green. CLI checks exercise fractional scaling,
  range refusal, version refusal and preservation of a preexisting output file.
- An independent Go standard-library decoding/conversion run confirmed selected
  bit patterns and pipeline values. In particular, 1.001 times 1000 truncates to
  1000; 9223372036854774 times 1000 rounds to 9223372036854773760, and its subsequent
  wrapping integer scale by 1000000 yields -2048000000. This corrected an initially
  incorrect hand-calculated test expectation; it was not changed merely to match
  the new evaluator. In-range binary64 cast behavior only is claimed.
- git diff --check: exit 0.

The numeric operation gap is addressed for the reference, CLI and Rust realization.
Go/TypeScript normalization targets, adapter-generation CLI, the concrete runtime
dispatch lowering contract and complete source-adapter adoption remain required.
Host-dependent out-of-range conversions are explicitly qualified, not guessed.
Keep the story active; this checkpoint is not the completed three-target outcome
and has not cut a new release.

## Standalone Go Checkpoint

Plan::go(package, module) now emits a standalone Go 1.26 library from the sealed
recipe. Expressions and conditions lower to typed function bindings. The module
retains canonical recipe, qualified bundles, checked root schemas and every
generated-file digest in normalization-target/1 with explicit Go configuration.
New compiles offline validators; Normalize accepts raw JSON and returns only a
complete validated result or typed, located Refused. Caller bytes are immutable.

Version 1, ordered version 2 and declared binary64 operations execute in native
Go against the reference fixtures. Integral/fractional token identity, signed
bounds, explicit wrapping, finite conversion/range refusal, lexical collection
indices and lazy first match remain observable. Strict malformed Unicode,
duplicate-key precedence and nested/multiple schema failures are checked.

Schema diagnostic presentation is explicitly target-specific: Go sorts instance
pointers and retains duplicate occurrences. Tests compare the complete reference
schema-finding multiset in that order; non-schema failures remain order-sensitive.
Rust's existing traversal order and artifact bytes are not changed.

The matcher gap is tracked as story:go-normalization-pattern-semantics. Go refuses
all selected pattern obligations before generation with go_schema_pattern and a
bundle-qualified schema pointer. Referenced definitions are checked; unselected
schemas in the retained bundle do not widen refusal. A pattern accepted by RE2
is not evidence that its ECMA-262 meaning is preserved. No matcher parity claim
is made until a bounded compatible implementation is qualified.

Verification on the final local implementation:
- CARGO_NET_OFFLINE=true CARGO_BUILD_JOBS=4 task check: exit 0.
- CARGO_BUILD_JOBS=4 task site-build: exit 0.
- Strict schema-contract Clippy with go-typecheck enabled: exit 0.
- Native go-typecheck lane on Go 1.26.5: four tests pass, including execution of
  three generated modules with go test -count=1 -race -mod=readonly ./..., network
  module lookup disabled and local toolchain selection. Concurrent repeated
  normalization does not mutate input or report races.
- Four generated Rust tests pass, including default and arbitrary_precision
  feature configurations and the unchanged version 1 canonical recipe digest.
- The complete old version 1 Rust normalization report, including every generated
  file digest, is byte-identical before and after source-packaging extraction.
- Go source templates are gofmt-clean; git diff --check exits 0.

This is a library-API checkpoint. The native lane is opt-in locally; default CI
does not thereby gain Go runtime execution. TypeScript normalization, generation
CLI, bounded Go pattern compatibility, concrete runtime dispatch lowering and
complete source-adapter adoption remain required. Keep the original story active.
No new release is cut by this checkpoint.

## Adapter Generation CLI Checkpoint

The generate schema normalize-generate command now exposes the existing Rust and
Go normalization library APIs. It accepts an explicit recipe, repeated bundles,
target, package and output directory. Go requires module identity; Rust refuses
that option. No TypeScript structural projection substitutes for an executable
normalization target.

All branches are admitted and target feasibility checks before the shared output
preflight. The complete output set is checked for incompatible file types, links,
case aliases and containment before writing. Canonical recipe/bundle paths cannot
be overwritten by generated destinations. Output bytes and normalization-target/1
provenance are exactly those returned by the library API.

The read-only --check mode compares every planned file byte-for-byte, deterministically
listing missing and stale paths. It creates no directories and repairs no files.
Neither write nor check claims ownership of unrelated files or deletes obsolete
files. Existing output-parent trust and subsequent I/O rollback limitations remain
explicit; this is not a new transactional directory writer.

Verification:
- CARGO_NET_OFFLINE=true CARGO_BUILD_JOBS=4 task check: exit 0.
- CARGO_BUILD_JOBS=4 task site-build: exit 0 on the final implementation.
- The normalization CLI suite now has 11 passing tests. New coverage compares
  both emitted targets with the API, exercises missing/stale read-only checking,
  preserves unrelated files, rejects invalid unused branches and target identities,
  protects canonical input aliases, preflights later destination failures without
  touching earlier files, and refuses Go pattern semantics before any artifact.
- Command-tree reachability and hidden compatibility spellings remain checked;
  the explicit supported leaf count increases from 50 to 51 for this command.
- git diff --check exits 0. Public guide, command reference and Unreleased
  changelog describe only the implemented target choices and output guarantees.
- The preceding Go checkpoint e9419a790b156d2ac65849e9b660b92525d0742c passed remote
  CI 34001185455, documentation validation 34001185416 and source bundle 34001185412.

The story remains active. TypeScript normalization and complete source-driven
lookup/runtime/declarative-dispatch mapping remain required. Generic pattern
support is tracked separately and must not be conflated with ordinary data fields
whose wire name happens to be pattern. No new release is cut by this checkpoint.

## Model-Owned and Lexical Input Boundaries

Source-grounded adopter record mapping reproduced two further requirements:
model-owned records cannot enter the bundle-only normalization API through the
document importer because their model provenance/type annotations are refused;
and selected embedded JSON token bytes remain observable beyond parsed value
identity. These are now tracked as derived stories
normalize-model-owned-records and raw-json-normalization-provenance.

The model boundary must use checked model authority, not annotation stripping or
a hand-maintained duplicate schema. The lexical boundary must explicitly retain
selected token bytes and distinguish absence from null without broadening every
JSON value into an opaque property bag. Both stories require binding design and
format compatibility decisions before implementation.

The generation CLI commit e113a65a0bac63e77cd17f43fa280a5bf56c93f9 passed remote
CI https://github.com/beyond10x/ess/actions/runs/34001828985. Existing Rust/Go
generation is integrated, not yet a complete multi-target normalization release.
No TypeScript normalization target, lexical capture support or model-owned
normalization support is claimed by this checkpoint.

## Implemented Model-Owned Normalization

The binding design now defines ess-normalization/3 and its model root identity:
system, specification version, resolved source digest, contract digest, projection
digest and exact selected root set. Root::pin_model admits only explicitly
selected roots; Plan::check_with_models accepts sealed ModelTypes selections
beside checked bundles. No imported annotation or parallel authored schema is
accepted as compiler authority. Existing bundle-only recipe bytes are unchanged.

Model wire schemas retain provenance, nominal definitions, requiredness, field
renames, constraints and optional-property semantics. The checked model's finite
string-enum projection is handled without flattening arbitrary intersections.
Selected model invariant statements refuse planning until an evaluator exists.

Rust and Go target reports use ess-normalization-target/2 for version 3 recipes,
retain each complete selected model projection and identify every root schema.
Root schema identifiers include complete selection identity: distinct root sets
that share projection bytes cannot collide in the Go validator registry. Source
projection bytes may still be retained once when they are identical.

The CLI accepts repeated --model inputs with or without bundles, compiles them,
recreates the declared selections and checks every identity coordinate. Model
input files and directory trees are protected from generated or canonical output.
Stale identities, closure-only roots and old recipe versions refuse explicitly.

Verification on the implemented tree:
- Full offline task check exits zero; task site-build exits zero.
- Six model normalization checks cover wire mappings, presence, every identity
  coordinate, old-reader/version refusal, invariant refusal, enum membership,
  and qualified-source-to-model normalization.
- Native Go 1.26.5 runs old, ordered, numeric and model recipes with -race,
  including two distinct selections sharing the same projection. Four tests pass.
- Five Rust target tests pass, including model records and both serde_json feature
  configurations. The old version 1 canonical recipe digest remains unchanged.
- Twelve CLI normalization tests pass, including model acquisition, source-tree
  protection, generation/check and stale model refusals. git diff --check is clean.

An adopter's actual decoded transfer conversion now checks and runs from its
model, and a generated standalone Rust adapter passes the source-grounded mapping
case. Its Go generation correctly remains refused at the model Bytes base64
pattern, tracked by go-normalization-pattern-semantics. Adopter data and evidence
remain outside this public store.

This closes model-owned normalization, not the complete parent objective.
TypeScript normalization, selected lexical JSON capture, bounded Go pattern
qualification and remaining source-driven mappings still require work and a
subsequent integrated release.
