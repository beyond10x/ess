# Complete ESS expression typechecking — binding design

Accepted for implementation on 2026-09-06 under the standing ESS review remediation authorization. Source implementation and all proposed verification remain pending. Fresh main source pin: `e113a65a0bac63e77cd17f43fa280a5bf56c93f9`; story `review-expression-typechecking`, revision 7. Initial reading used coordinator `1b1c4a3a424e3304e503747422a0c3d10d86928a`; the complete ess-domain, ess-compiler, ess-primitives and ess-conformance directory tree objects are identical at both commits (paired IDs in `source-correspondence.md`). This is a non-interactive, read-only design dispatch. No builds or behavioral probes ran; the matrix below is proposed work, not measured results. The existing Scope is retained verbatim in `scope-as-read.md`; its refresh is additive in `scope-delta.md`.

## Admission boundary

Specification assembly and direct compilation must reject a predicate whose complete path or operand pairing is ill-typed. They must not reject a type-correct predicate because current conformance cannot project its facts, cannot order its text without a scale, or cannot find a satisfying witness. No predicate grammar, primitive Number behavior, assignment/conversion rule, infrastructure predicate policy, or persisted Predicate/IR/suite shape changes.

The semantic owners remain command outcome guards; entity invariants; struct/newtype invariants; and view filters. Authored view `satisfies` is an additional admission boundary against a compiled row surface. The predicates themselves remain the existing `ess_primitives::Predicate` tree. Always/Never, both sides of comparisons, every All/Any/Not child, every membership element, collection targets and quantified bodies are checked even when runtime short-circuiting could hide them. No dead-branch exemption.

## Environments and path rules

| Owner | Observable roots |
|---|---|
| Command guard | Declared command input field names. Preserve the outcome-specific prohibition on reading its command-input subject identity, including free reads inside quantifiers and a quantifier's collection expression. |
| Entity invariant | Identity, declared fields, and synthetic `state` typed by that entity's lifecycle enum. |
| Struct invariant | The struct's fields. |
| Newtype invariant | Synthetic `value` with the wrapped representation's type. `value` is an invariant environment root, never a general-purpose newtype selector. |
| View filter | All source observables, including fields not projected by the view, plus reserved namespace `param.<declared-name>`. |
| Authored view satisfies | Only projected row fields. No filter parameter namespace or implicit access to unprojected source fields. |

Use declared field names, not `naming.wire` aliases. A newtype and Optional unwrap transparently without consuming a segment. Struct traversal consumes exactly one declared member. Primitive/enum leaves reject every remaining segment: `amount.nonexistent` on Decimal and `state.name` are errors. A whole struct/list/map/union is a resolvable aggregate, not a scalar fact. Union tags/members and narrowing remain unavailable as ESS expression selectors.

Optionality is recorded on the resolution but is not a static presence obligation. A finite read through `Optional<Money>` resolves as Money; absent/null values bind no scalar and keep existing runtime semantics. `defined(note)` does not refine later operands and no flow-sensitive narrowing is introduced.

A free `param.p.member` resolves `p` through its declared type and counts as using `p`. Count uses at any depth and within quantified bodies/targets. An undeclared parameter remains an undeclared reference; `param` alone is not a scalar root. Check a parameter's named type before inferring a terminal type; do not suppress an unresolved-type error on the assumption another pass has already reported it. Binder shadowing applies to `param` as it does to every root: a binder called param does not make its body a use of a view parameter.

### Collections and lexical binding

The current primitive FactPath grammar already permits numeric segments, and its evaluator consumes `collection.count` and `collection.<ordinal>` (`facts.rs:308–334,618–631`; `predicate.rs:395–459`). The conformance comment saying indexes cannot be spelled describes its projection limitation, not that grammar.

Bind these ESS-specific rules:

- `List<T>.count` has Integer/Number representation. A canonical nonnegative decimal ordinal segment (`0` or digits without leading zero) consumes one List layer and yields T. An enormous or out-of-range ordinal remains type-correct; cardinality/value absence is not a type fact. Reject other List selectors and continuation through count.
- `Map<K,V>.count` has Integer/Number representation. Forall/Exists bind **V, the map's values**, not K and not an invented key/value entry record. This is the explicit policy decision where the existing declaration names K/V but no ESS projector establishes an element surface. No key/name selector or stable Map positional order is introduced; direct Map ordinal selectors are refused. A future projector may enumerate values internally for the existing quantifier evaluator without exposing that enumeration as ordered Map semantics.
- Forall/Exists require a List or Map after transparent unwrapping. Resolve the target in the outer environment, then push the binder for the body. List binds T; Map binds V. Scalar, struct and union targets are type errors.
- Nested binders can refer to outer binders and free roots; the nearest binder shadows a repeated name. Pop it after its body. Check nested collection targets and bound paths directly from the AST, not `fact_paths()` or `quantified_collections()`, which intentionally omit bound paths.
- `.count` is special only on a collection. A struct's declared count field has its declared type. Optional collections retain optionality; an absent count keeps quantifier Unknown, not vacuous success.

Actual collection projection and map enumeration are excluded. The typed command-input projector currently publishes neither collection counts nor elements (`input.rs:474–485`). Those facts remain an explicit conformance capability gap after successful semantic validation. The untyped runner's Map walk follows supplied object keys (`runner.rs:2116–2157`); it is not authority for ESS Map selectors and is unchanged.

### Recursion

Resolve a path iteratively, tracking the current type identity and remaining-segment position while following transparent references. A repeated pair without consuming a segment is a non-progress cycle, reported for that read with the involved type names. Consuming a struct member or List index advances the position, so finite paths through recursive optional structs remain legal. Do not eagerly expand an aggregate's complete graph.

Do not turn conformance's MAX_TYPE_DEPTH or witness depth into a specification path limit. Existing TypeRef/Predicate parser limits remain unchanged. Existing inhabitation validation already distinguishes productive recursion (`system.rs:519–614`); leave it intact. In particular, an inhabited `Loop = Optional<Loop>` may still lack a resolvable scalar leaf for a particular predicate: reject that read, not the declaration globally.

## Operand contract

Compare evaluator representations, independently of assignment. Preserve nominal declarations for diagnostics and enum literal checks, but do not call `is_assignable`, apply conversions, or require identical nominal names for fact/fact comparison.

| Operation | Admitted operands |
|---|---|
| Eq / Ne | Two scalars of the same evaluator kind: Bool/Bool, Number/Number, or Text/Text. Literal/literal follows the same rule. Different kinds are a type error even though the untyped primitive evaluator can answer unequal. |
| Lt / Le / Gt / Ge | Number/Number or Text/Text. Boolean ordering, aggregates and mixed kinds are type errors. Text ordering is type-correct; no ESS scale exists, so evaluation may remain Unknown. |
| AnyOf / NoneOf | A scalar path; every listed literal has its evaluator kind. Check all members, not the first. Empty sets remain legal and retain existing runtime behavior. |
| Truthy | A scalar path of Bool, Number or Text. Preserve Boolean value, nonzero Number, and nonempty/non-`false` Text semantics. |
| Defined | A scalar path, including optional scalars. It asks whether a scalar fact was observed. Whole aggregates are rejected here; it is not a new aggregate-presence predicate. |

Representation mapping is the existing `input.rs:561–584`: Boolean→Bool; Integer/Decimal→Number; String/Timestamp/Duration/Uuid/Bytes→Text; enums→Text; newtypes transparently inherit their representation. Thus Email/String, differently named numeric wrappers, and same-representation fact operands can compare while remaining nonassignable. Do not add UUID/date/duration/byte literal parsing to predicate validation.

When a comparison pairs an enum-typed fact with a literal, the literal must be Text and one of that enum's variants; apply the same rule to membership literals, nested enum fields, parameters, wrappers and binders. Preserve the current view-filter enum-name refusal, extending its resolution rather than maintaining a second enum walker. A numeric/Boolean literal whose display happens to equal an enum variant is not Text: quote that variant. Fact/fact enum comparisons require Text/Text only; disjoint enum value sets are not a new satisfiability check.

Integer compared with decimal literal `0.5` is **type-correct and unsatisfiable**, not a malformed Integer value assignment. Integer input projection still rejects a fractional supplied value. Also preserve the separate satisfiable search-gap control: `all: [amount > 0.1, amount < 0.2]` on Decimal is valid; explicit 0.15 evaluates True, while the current literal/±1/0/−1/base-1 candidate ladder has no satisfying candidate. These are source-grounded expectations to measure during implementation (`witness.rs:295–336`; `tests/synthesis.rs:2456–2483`).

## One checker and two adapters

Proposed modules: `ess-domain/src/expression.rs` owns `resolve_path` and `check_predicate`; `ess-compiler/src/expression.rs` adapts resolved types. Export them from their crate roots; dependencies already point in the required direction.

The read-only type-environment interface supplies root lookup, type shape, struct-member lookup, named-body lookup/identity, and enum variants. Domain adapts TypeRegistry plus owner fields; compiler adapts EssIr plus ResolvedField. The common checker owns traversal, lexical scope, representation compatibility and enum-literal policy. Do not copy a Specification from the IR, build a second registry of serialized types, or persist a mirrored predicate tree.

Resolution returns terminal declared/representation information, optional traversal and structural access requirements (ordinary scalar leaf, collection cardinality/element access). Those requirements are facts about the path, not a claim that a producer implements it. Predicate checking returns ordered semantic errors and the checked reads/free parameter uses. Results are transient and tied to the environment checked, not reusable admission tokens after mutation.

Registry-aware checks run in CommandSpec/EntitySpec/ViewSpec validation and a named-type invariant pass within Specification::validate after lifecycle types have joined the registry. Replace the partial semantic walkers, preserving owner-specific restrictions and avoiding duplicate root/enum/parameter errors. Construction without a registry may retain local shape checks; it is not the final typed-path boundary. Direct compile_locating keeps revalidation before minting IR.

Conformance consumes the compiler adapter. Its public resolve_path facade may preserve Target for callers, but its semantic resolution must delegate. Keep projection support/depth classification separately, based on the returned traversal requirements; a semantically legal collection scalar must not become projectable merely because its terminal kind is Number. Authored satisfies checks semantics first, then row/projection availability. Synthesis reuses those checked accesses for published-field checks. Witness candidates, projection values, untyped evaluator/runner, and Unknown-does-not-pass behavior remain unchanged.

## Diagnostics and compatibility limits

Each semantic error identifies its owner location; full original path or expression; first failing segment and resolved prefix type for path errors; operator and both declared/representation types for operand errors; and binder/parameter context where relevant. Visit all AST children in stable source order, resolving left before right and membership items in order. Suppress operand cascades when an operand path failed; continue unrelated leaves. Report a missing named type without a fabricated terminal type.

Use existing domain codes: UnobservableFact for missing/forbidden selectors; TypeMismatch for operator/aggregate/quantifier mismatches; UndeclaredReference for missing parameter/type/enum names; SelfReference for non-progress transparent cycles. Compiler bridge retains owner family and existing class mapping.

No structured field extension to primitives::ValidationError is required here. Shared internal errors may be typed, but conversion to current domain/compiler diagnostics preserves the existing message/hint/span envelope. The bridge cannot manufacture typed Detail entries from prose (`resolve.rs:513–550`). Locator uses unique textual needles: promise owner/declaration attribution only when found, and otherwise document path with no line. Never promise a precise predicate-leaf line/column or fabricate a source for a programmatically assembled object.

For authored admission, retain ESS-AUTHOR-026 for existing unreadable-path/projection failures. Add one distinct InvalidPredicate cause for newly rejected operand/quantifier-type errors, carrying the shared diagnostic and row owner; reserve ESS-AUTHOR-035, currently unused after 034 (`authored.rs:1041–1087`). Do not claim those are unpublished fields. Authoring/Refusal/Cause are in-memory results, not serialized Predicate or suite payloads (`authored.rs:709–779`). Existing valid output bytes and codes remain fixed; new malformed-input refusals are deliberate.

## Required implementation matrix

Every row is proposed; capture actual initial red runs and counts before changing production, then final runner counts and exits.

| Family | Deciding negative | Positive / preservation control |
|---|---|---|
| Complete paths | `amount.nonexistent` with amount: Decimal; scalar/enum continuation; missing nested field; missing root on either fact operand | Nested struct, Optional and transparent newtype; only synthetic invariant `value`, real state enum; declared naming.wire does not rename expressions |
| Every admission owner | Guard; entity; struct/newtype invariant; filter; authored satisfies; module-local validation of a specification mutated after assembly | Existing normative fixtures still assemble/compile; safe compiler entry controls and existing revalidation-order fence remain |
| Numeric/Boolean/Text | Boolean ordering; Number/Text compare; mixed membership with the invalid item last; quoted-text versus numeric literal | Both fact operands; all three scalar truthiness kinds; same-kind nominal wrappers; typed literal/literal AST |
| Enums | Invalid nested/parameter/binder variant, numeric or Bool literal masquerading as text; bad entry in NoneOf | Valid enum literals and lifecycle variants; enum/newtype wrapping; same-representation fact/fact operands |
| Presence and aggregates | Aggregate Truthy/Defined/comparison; primitive .count; union selectors | Optional scalar absent keeps Defined False and comparison Unknown; optional nested fields typecheck without narrowing |
| Collections | Scalar quantifier; invalid bound field/kind; nested scalar target; invalid List selector/index spelling; Map key/ordinal selector | List .count/.0.field; Map .count and value binder; nested List/Map targets; count-field shadowing; legal collection access remains a conformance capability refusal |
| Lexical scope / params | Undeclared nested param; bad param member/type; free identity read inside binder; out-of-scope binder; unused declared param | Nested param counts as use; outer/free reads; repeated binder names; param binder shadowing and target-before-push behavior |
| Recursion | Non-progress newtype/optional cycle on a read; misspelled field after recursive traversal | Finite repeated optional-struct path longer than conformance's 32-step limit validates; projection may refuse depth; existing productive declarations remain legal |
| Validation ≠ solving | No validation error for Integer == 0.5; retain synthesis's unwitnessed outcome | Decimal interval validates, explicit 0.15 satisfies, current candidate search fails for its own reason; text order without scale remains unevaluable |
| Honest diagnostics | Several independent leaves; both operands wrong; bad binder; unresolved type; ambiguous/absent source text | Full path/segment/type and stable owner/class; no cascaded mismatch on unknown operands; real owner line or honest unlocated span |
| Compatibility | Malformed TaxOrder/UNDECIDED fixtures now stop at assembly | Replace their shared fixture guard with a valid one; retain runtime Unknown assertions via directly supplied bad predicates against valid IR, never by bypassing validation in the common fixture |
| Bytes | No rewrite/optimization of valid predicates, IR or suite | Capture before/after canonical IR and digests plus suites/artifact maps for billing, gatepass, oracle-fixture and revision-pair before/after; retain conformance-report bytes and unrelated primitive/infra tests |

Implementation gates: baseline and final offline locked package tests, package formatting and strict Clippy for ess-domain, ess-compiler and ess-conformance. Root owns exact full integration `task check` and any required site gate. No package count, test result, timing, byte-equivalence result or approval is asserted by this proposal.

## Coordinator decision

The complete operand, collection, optional, nominal-representation, binder and recursion rules above are bound for story:review-expression-typechecking. Root reviewed the full proposal and exact source correspondence before acceptance. The implementation must preserve the legal Integer-versus-0.5 case and distinguish semantic validation from projection support and witness search. This is an implementation decision, not a test result or a claim of shipped support. Root owns this binding and the shared release/planning records; source implementors must return any necessary design change before widening semantics.

## Sealed specification verification correction

Source inspection during implementation establishes that Specification has private fields, shared getters and Serialize only (spec.rs:115–213); it has no safe public post-assembly mutation or deserialization path. The earlier proposed literal mutated-Specification-to-compile_locating fixture therefore cannot be constructed by a downstream safe caller. Retain unconditional compiler revalidation at resolve.rs:744–755. Measure module-local test-only mutation inside ess-domain against Specification::validate, real public compile/compile_locating controls, compiler diagnostic propagation and the existing sealed_state revalidation-order fence. Report these as separate boundaries; do not claim an impossible public mutation case was executed. No production mutator, public test-access feature, unsafe construction or domain-to-compiler dependency is added for the fixture. This corrects the verification mechanism, not the complete-expression admission requirement.
