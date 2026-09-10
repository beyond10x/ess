# ESS service semantics and Entity Runtime

Owner: `story:ess-er-semantic-crosswalk`. This is the source comparison required by
[migration step 3](migration.md), after [service contract extraction](../service-contract.md).
It records requirements for implementation. It does not declare an ER target, a migrated service,
or a new persisted format implemented.

## Source vector and reading rules

The compared sources are published feature commits, not claims of integration into main:

| Source | Exact revision | Authority inspected |
| --- | --- | --- |
| ESS | `f300965f409f0370f843b45793541cf66aaee17a` | `ess-compiler/src/ir.rs`, `ess-domain/src/{command,entity,expression,types}.rs`, `ess-primitives/src/{facts,predicate}.rs`, `ess-conformance/src/{input,decision,scenario,runner}.rs` |
| ER | `074818fd119d0bc74210b3fb27b58088f3d4f80c` | [`entity-core`](https://github.com/beyond10x/entity-runtime/tree/074818fd119d0bc74210b3fb27b58088f3d4f80c/crates/entity-core/src): `definition.rs`, `validation.rs`, `runtime.rs`, `replay.rs`, `number.rs`; `entity-store` recorded ports |
| SDK | `03026ec181b6d343672a3863dc1c9874398c6105` | [`service-engine`](https://github.com/beyond10x/service-sdk/blob/03026ec181b6d343672a3863dc1c9874398c6105/crates/service-engine/src/lib.rs): `ServicePlan`, `intent`, `replay_intent`, `decide_intent`, `fold`, `reduce`, `produce_events`; `service-runtime-ir` persisted reader |

ESS crate paths below are relative to `crates/specify/` for compiler/domain/primitives and
`crates/verify/` for conformance. ER paths are relative to its `crates/entity-core/src/` unless
qualified. These paths and named symbols are the citations for each row, not similarly named
features inferred from a package description.

The compared ER branch retains the definition and execution model from main; its replay source
adds `VerifiedReplay`. Inspect the pinned branch for that symbol, rather than an older primary
checkout. The SDK branch also contains intent-claim replay and authorization changes absent from
older SDK primary checkouts.

“Available” below means a kernel primitive exists with the stated qualification. It is not proof
that a lowerer uses it correctly. “Kernel gap” must be implemented in pure ER. “Binding” denotes
external input or IO that remains outside that kernel. Neither category permits dropping a rule.

## Commands, state and recorded results

| ESS construct and source | Current ER behavior and source | Required mapping or gap |
| --- | --- | --- |
| `ResolvedEffect::Creates`; declared initial state in `ResolvedEntity.lifecycle` | `create` validates fields, selects `lifecycle.initial`, checks invariants and creates revision 1 | Initial-state selection is available. ESS command arguments, selected named outcome and supplied effect observations must remain in the recorded command; precomputing only final fields loses that decision evidence. |
| `ResolvedInstance::Observed` identifies the emitted event and field carrying a new identity | `create` takes a caller-supplied nonempty string ID; the kernel generates none | Binding allocates identity once and records it as input; kernel verifies its typed correspondence to the outcome's observed event field. Never generate a second identity during replay or substitute the aggregate stream ID for an entity ID. |
| `ResolvedEffect::Moves` carries an exact named transition | `OperationDefinition.transitions` chooses by `from`; validation refuses two transitions from one state | State membership and transition application are available. An ESS command with several conditional moves from the same state cannot be flattened into this structure. Named outcome selection is a kernel gap. |
| `ResolvedEffect::Updates` changes fields without moving lifecycle state | An ER operation may use self-transitions, but every success increments revision | A selected updating outcome can preserve lifecycle via self-transitions over the permitted states. This does not implement an observation with no mutation or authorize a generic state setter. |
| `ResolvedOutcome.sets` contains typed input references/literals and optional named conversions | `OperationDefinition.set` resolves against pre-operation fields; fields are validated before invariants, event templates see post-operation fields | Available assignments need typed lowering. Preserve input versus prior/post-state scope, literal interpretation and conversion identity. Undetermined ESS fields require explicit bound values; do not fill them with guessed defaults. |
| Entity invariants plus newtype/struct invariants (`ResolvedEntity`, `ResolvedBody`) | `EntityDefinition.invariants` runs after create and update; rule reference scope excludes command arguments | Preserve every nested invariant and its evaluation location. A top-level entity invariant alone does not validate every nested/newtype/list/map value. Kernel validation and quantified predicates need extension for the unrepresented cases. |
| Declared lifecycle initial, states, terminal states and named transitions | ER lifecycle stores initial/states; operation transitions carry edges, without a terminal-state declaration | Preserve ESS terminal metadata and edge identity in the lowering/binding account. Do not infer that terminal means an `updates` outcome is forbidden: enforce only rules declared by ESS. |
| One command, multiple named `ResolvedOutcome`s | ER has one `OperationDefinition` with one set/event vector for all its transitions | Kernel gap: data-defined outcome selection, distinct per-outcome sets/events/errors, and a recorded selected outcome that replay recomputes. Splitting each branch into a caller-selected operation would let the binding choose business semantics. |
| `When`, `Otherwise`, `External`, `WrongState` | ER has preconditions on an already selected operation/transition | Preconditions are not a substitute for branch selection. `Otherwise` depends on other branches, external causes require observations, and wrong states come from `EssIr::wrong_states` across the command's declared moves. |
| `WrongState` with `refuses: false` | Ordinary ER self-transition execution increments revision even with no field changes/events | Kernel gap: record an accepted observation without changing entity state or revision. `entity-store` already separates ordered observations from mutating decision history; the service decision vocabulary still must produce and replay the correct observation. |
| Refusing outcomes carry a declared error and no subject mutation/events (`command.rs::validate_outcome`) | ER returns `CoreError`; no accepted `DecisionRecord` is produced on refusal | Preserve the ESS error identity, payload contract and outcome identity. Store refusal observations through explicit recorded ports; never turn them into mutating decisions or flatten all errors into an unnamed precondition failure. |
| Ordinary outcomes cannot be empty; accepting wrong-state is the explicit exception | ER can create/execute with zero domain events and still emit a complete decision record | Preserve both facts. Do not weaken ESS `EmptyChange` admission to fit ER. A zero-event mutating ER decision and an accepted non-mutating ESS wrong-state response are different histories. |
| Ordered zero/one/multiple domain events on a selected outcome | ER operations carry `emits: Vec<EventDefinition>`; `CreateDefinition` carries only `emit: Option<EventDefinition>` | Ordered multi-event operations are available. Multi-event creation is a kernel gap; splitting it into create plus an operation introduces an extra revision/decision and may expose an intermediate state. |
| Payload mappings can leave event fields undetermined (`ResolvedPayload`) | ER materializes a complete templated payload but declares no event payload schema | Bind undetermined inputs explicitly, then check event field types and declared conversions in the pure decision. A missing payload mapping is not permission to invent a value or claim ER's arbitrary JSON payload is an ESS schema check. |
| An outcome may have no entity subject (for example an external effect command) | `create`/`execute` require one entity and produce an entity instance | A separate recorded non-entity service outcome is needed at the execution seam. Do not invent a synthetic domain entity merely to call `execute`, or fake an entity mutation for event-only work. |

### Selection must be specified, not guessed

`CommandSpec` validation checks one unconditional branch, valid input predicate references,
wrong-state consistency and refusal purity. That is not a proof that all `when` predicates are
mutually exclusive. The inspected model carries no branch-priority field. Declaration order alone
is not evidence that overlapping `when` branches mean first-match-wins.

The target must account for overlap, unevaluable predicates and external/wrong-state precedence
explicitly. An ambiguous source cannot acquire silent priority during lowering. Record the
ambiguity and settle it in the semantic contract before enabling that case. Any selected deterministic
execution profile must refuse unresolved selection before mutation. The pending ER definition design
must name how these refusals differ from a declared business error; an ambiguity diagnostic must not
masquerade as the command's fallback outcome.

`ess-conformance/src/decision.rs` already distinguishes `Satisfied`, `Refuted` and `Unevaluable`.
`Unknown` does not mean “take otherwise.” Reusing a Boolean predicate API would erase a distinction
the independent observer can already see.

## Predicate and value crosswalk

| ESS source | ER source | Required behavior |
| --- | --- | --- |
| `Predicate::Always`, `Never`, `All`, `Any`, `Not`; Kleene `Truth` | `Condition::Literal`, `All`, `Any`, `Not`; Kleene `Truth` | Logical results are available. ESS can represent empty conjunction/disjunction; ER definition validation refuses empty lists. Lower constants instead of producing invalid empty ER operators. Keep Unknown reasons and source attribution. |
| `Compare` with `Eq`, `Ne`, `Lt`, `Le`, `Gt`, `Ge` | Same numeric/equality operator family | Map operator spelling and typed operands explicitly. ER returns false for ordering present nonnumbers; ESS primitive predicates return Unknown for invalid/unavailable ordering. Domain typechecking may refuse such a source earlier; do not claim that primitive probe is a compiled service counterexample. |
| Exact ESS `Number`, `FactValue` and optional protocol scales | `number.rs::compare` is exact decimal ordering with arbitrary exponents; ER has no scale registry | Preserve the admitted ESS numeric value before comparison. Do not assume every JSON number is a finite Binary64 or every text order is lexical. Scale-dependent ordering must have a kernel representation or a visible unsupported account. |
| `Truthy(path)` and `Defined(path)` | `Exists` only asks presence of a non-null operand; equality compares values | Defined can map only when the fact projection matches. Truthy is not Exists: false, zero, empty text and the text `"false"` can be present but falsey. Preserve ESS `FactValue::is_truthy`, rather than asking only whether the key exists. |
| `AnyOf` and `NoneOf` | `In`, with `Not` for negative membership | Available for equivalently typed scalar facts/literals, with identical numeric meaning. Missing observations remain Unknown under negative membership. |
| `Forall` and `Exists` quantify List/Map elements with binders (`expression.rs::quantified`) | No quantifier or bound-variable operator; ER `Exists` means value presence | Kernel gap: scoped quantification, nested binder resolution and actual collection traversal. Empty forall is true, empty exists false; unobserved collection is Unknown. Do not unroll only the current fixture's length or confuse the two meanings of Exists. |
| `InputFacts` at the inspected ESS revision projects newtypes and named scalar fields but only checks collection containers; view `row_facts` has a separate projection | ER references walk nested object properties; schema validation checks references at registration | The bare FactStore quantifier probes below do not establish compiled input support. The subsequent [typed collection facts](quantified-input-facts.md) change supplies input counts and ordinal values with compiled-input tests. `runner.rs::bind` still omits sequences and nulls; keep that view capability distinct. Map keys/list ordinals/quantifier scopes need explicit typed access. |
| `Optional<T>`; `LeafShape::admits` accepts omission/null for optional fields | `required: false` permits omission; present null still undergoes kind validation | Kernel gap for nullable typed values at all depths. Simply marking a field optional refuses valid ESS nulls. Using `FieldKind::Json` would discard its type, nested constraints and invariants. |
| `Integer` has signed i64 admission | ER `Integer` accepts signed and unsigned JSON integers | Available with explicit `min: i64::MIN`, `max: i64::MAX`. A probe confirms i64::MAX accepted and 2^63 refused only after the bound is added. Keep exact integers above 2^53 distinct. |
| `Decimal` is semantically exact; established wire/primitive stages are described in `review-primitive-semantics.md` | ER numbers are exact JSON-number values under `arbitrary_precision`; strings are text | String-encoded Decimal is not an ER numeric operand. Preserve existing ESS admission/canonicalization limits and wire representation; do not silently switch old artifacts to arbitrary decimal token semantics. New representation/codec work requires an explicit version boundary and corpus. |
| `Binary64` in ess/2 is finite and preserves signed zero | ER `Number` accepts arbitrary JSON decimals; numeric equality treats signed zeros equally | Kernel gap for field admission and tagged semantic representation/codec. Exact decimal support does not establish Binary64 support or signed-zero wire preservation. Do not identify the two types. |
| `String`, `Boolean`, enums, closed structs, lists | ER String/Boolean/Enum/Object/Array schemas | Base shapes are available; preserve naming, required/nullable elements, nested invariants and declaration order where observable. ER maps being ordered is not proof of ESS field-order preservation. |
| `Map<primitive, T>` and tagged unions (`ResolvedTypeRef`, `ResolvedBody`) | ER Object permits declared properties or untyped additional properties; no typed additional-value schema or discriminated union | Kernel gap for typed map keys/values and tag-selected shapes. `additional_properties: true` does not validate T, and flattening a union makes invalid combinations admissible. |
| Newtypes are nominally distinct even when wire representations match | ER schema has no nominal newtype identity | Lower only after ESS assignability is settled; retain type and conversion identity in the binding account and validate nested constraints. Wire representation equality must never authorize an undeclared conversion. |
| Timestamp, Duration, Uuid, Bytes have their own admitted spellings | ER has String and limited ISO instant operators, but no corresponding field kinds | Kernel/codec gap wherever plain String omits validation. A valid ER `before` operand is not proof of ESS Timestamp admission; never use string ordering for instants or durations. Keep codecs outside the kernel only if pure kernel validation still enforces the admitted value contract. |
| Payload/set literals are typed string spellings, distinct from input references | ER template strings beginning `$` are references; `$$` escapes one dollar | Available with type-directed literal parsing and dollar escaping. A literal `$id` must not read entity identity. Preserve nested literal values and source distinctions through serialization and replay. |

The abstract Decimal claim and existing ESS primitive/canonical-serialization limitations must both
remain visible. This migration cannot repair those limits by changing old readers as a side effect.
The independent primitive corpus, structural codecs and exact token tests remain upstream evidence;
comparing ER's numeric implementation alone proves none of their target behavior.

## Identity, relations, concurrency and IO

| Requirement | Ownership and required evidence |
| --- | --- |
| Typed entity identity and field/wire names | ESS `ResolvedEntity.identity`, `ResolvedInstance` and `Naming` stay authoritative. ER string IDs need an injective typed encoding and validation; no inferred equality with stream keys, request IDs or foreign-key names. |
| Owns/references, cardinality and `via` | ESS `ResolvedRelation` and `EssIr::relations_carried_by` determine the carrying side. ER `FieldKind::Ref` checks nonempty string and target type declarations; `acyclic` is metadata, not cross-instance enforcement. Relation existence/cardinality/ownership needs explicit transaction-bound validation where commands require it. Do not label kernel registration a database integrity check. |
| Entity revision versus physical store position | ER revision belongs to mutations; one mutation may emit many events. Recorded observations and group records have their own physical ordering. SDK `through_version` currently refers to the service stream. Keep all three identities explicit; deriving revision by counting emitted events is invalid. |
| Multiple entities in one aggregate/service action | ER pure decision evaluates one instance; `entity-store` atomic recorded ports and Eventlog group commits compose decisions outside the kernel. Preserve one atomic boundary with record-ID equality/conflict and request-order semantics. Do not loop independent commits or treat a flat SDK event batch as proof of multi-entity atomicity. |
| View parameters, filtering, order, consistency | ESS `ResolvedView` remains the semantic contract; ER `ProjectionDefinition` is only key/group/state metadata | Query/delivery plans and providers implement the full selected view semantics. ER's projection shape is not an implementation of arbitrary ESS views. Preserve authorized hidden source metadata and partition boundaries. |
| External outcomes and effects | ESS external cause is an obligation, not a supplied caller branch name | Binding performs or observes external work, records bounded typed evidence, and passes it into the pure decision. Replay consumes recorded observations without rerunning effects. Uncertain external outcomes remain uncertain; do not retry a mutation as if no effect occurred. |
| Authentication and admission | SDK `intent` enforces realm before decoding; command claim hashes bind operation/input/auth partition, then replay rechecks current authorization | Preserve this actual ordering and current-authorization check on idempotent replay. Tenant/realm/authority/user/executor stay injected, and absent realm differs from `Some("default")`. ER receives admitted inputs; it never chooses authentication authority. |
| Content, time, IDs and projection failure | SDK `decide_intent` stages content, produces events, appends, projects, then accepts staged content | Keep external policy and compensation/recovery at bindings. Once persistence commits, projection/content retry must use the durable receipt rather than repeat semantic mutation or delete committed referenced content. Capture time/IDs once so replay never consults a clock or generator. |

## Opt-in migration and replay

The current `EntityDefinition.version` identifies a domain definition; it is not a new format or an
evaluator-semantics selector. Changing it alone cannot make an old reader understand branch,
nullable, quantifier or creation-event-vector semantics. New definitions and decision envelopes
need explicit discrimination. Keep the current closed reader and kernel behavior intact for their
existing documents. The required owner-local ER and SDK designs, plus the organization migration
ADR for changed consumer-verified bytes, must name relying parties and move order before that work
is enabled. This page assigns no unimplemented format number.

For the new profile, persist the validated definition, typed command, selected outcome, bounded
external observations, result, exact ordered events and conversion/binding identities necessary to
recompute the decision. Replay must evaluate selection, assignments, invariants and event payloads
again and compare the result. A recorded outcome name is evidence to verify, never authority to
skip selection. Ordinary repeated observations need their own record identity and order without
advancing entity revision.

The current `entity-store::RecordedObservation` requires a nonempty entity/ID and revision at
least 1. Its existence therefore proves storage for observations about existing entities, not
for a refused creation or a subjectless service command. The new service observation envelope
must represent those cases truthfully; a fabricated instance at revision 1 is not a solution.

Keep `service-runtime-ir/3` and existing `ServicePlan` readers and entrypoints. Service contract
extraction already preserved canonical bytes for the existing runtime fixtures; it did not switch
execution engines. Add the ER execution profile explicitly, then opt in generated services after
source-based equivalence checks. Old event-only SDK history lacks the command/definition/observation
inputs needed for full decision replay. Preserve it through a truthful legacy boundary; do not
manufacture earlier decisions. ER `replay`/`VerifiedReplay` already rejects `LegacyImport` as genesis.

## Independent evidence and implementation order

The bounded local harness `semantic-crosswalk-probe` pins both revisions above. Its retained
manifest, lockfile, Rust source and output live under
`local-evidence:ess-evolution-20260910/semantic-crosswalk-probe/` and
`local-evidence:ess-evolution-20260910/semantic-crosswalk-probe-final.log`.
All eight probes passed with the retained lockfile, both source-pinned dependencies and ER's
required `serde_json/arbitrary_precision` feature enabled. The final invocation was `cargo run
--locked --manifest-path <retained-probe>/Cargo.toml`; its compile/run finished successfully.
An initial offline attempt lacked the new ER revision in Cargo's cache, and the first compilation
found a count-construction error in the probe (floating point instead of the supported `usize`).
Both earlier failures are retained separately and are not test successes.
The harness checks the creation-event limit, absent named-outcome shape, optional-null admission,
zero-event revision/replay, ordered operation events and dollar escaping, signed integer bounds,
ill-typed primitive ordering, and empty/unobserved quantification. These are characterization probes,
not a service acceptance suite or a full gate.

Implementation proceeds from the semantic gaps, not from the easiest sample that current ER accepts:

1. Specify the opt-in ER definition/decision model and compatibility dispatch, including outcome
   selection, observation revisions, multi-event creation and pure typed validation. Resolve the
   selection ambiguities above in source semantics before claiming their behavior implemented.
2. Implement missing rules inside the IO-free kernel. Pin exact success, refusal and replay vectors,
   including null/absence, nested quantifiers, typed maps/unions, numeric/codec edges and literal
   escaping. Verify old-reader and old-definition behavior independently.
3. Implement `ess-entity-runtime` with exhaustive accounting of the resolved types above. Emit
   validated definitions plus explicit binding obligations; no generic JSON escape hatch and no
   SDK fallback semantic evaluator for unrepresented rules.
4. Delegate the opted-in SDK decision and replay paths to ER while keeping auth, queries, effects,
   content policy and Eventlog assembly at their established boundaries. Verify billing/gatepass
   startup and restart, then the actual application behaviors from the acceptance register.

Independent tests must derive expected branch, fields, state, errors, event order and revision from
authored ESS scenarios and hand-audited vectors, before lowering. Drop a predicate, choose the wrong
branch, remove a creation event, coerce an exact value, increment an observation revision, or trust a
recorded outcome during replay: each must fail a named vector. Tests that only compare the lowerer's
output with the executor built from that output cannot establish semantic equivalence.
