# ESS service contract to Entity Runtime lowering

Status: admitted implementation interface for ESS evolution revision1 M5. Source ESS
`be604d874ee9e567ae104e565e7cbddf953885a9`; coordinated Entity Runtime dependency
`28d06303fc081c1d06b5c2da847e908d7937a5f3`, tree
`5c1077260d79c09bfbc0162008b3f6476f6121ff`. The required operation-field amendment is
implemented, verified and locally integrated. Both source reviews of that operation-field
amendment and both lowerer design reviews are closed. The admission receipt is the root handoff's
`waves/0009-service-convergence/operation-field-fulfillment-integration/integration.md`.

The current target also corrects a nested union definition refusal: a variant object may contain
its own `value` or `content` field under the adjacent payload. The outer wire keys, nested
validation and replay bytes are unchanged. Existing Connectors Parameter/BodyParameter declarations
require this correction; no reference relation is removed to avoid it.

This adopts the capability already specified by corrected design SHA256
`22d0eb4bb0c0847855b75e2458c3ce57d573c3d12e068ddcf360a31001a536ce`; it adds no semantic
requirement or review round. The earlier target-gap analysis below is historical evidence only.
At this accepted pin, every omitted non-identity field on an accepting update/move MUST lower to
its typed Required/Optional fulfillment requirement. Complete billing/gatepass lowering must
succeed; `OperationFieldFulfillmentUnsupported` is not a permitted substitute at this target.
Definitions with fulfillment use service/3 and record/request4; conditional-only definitions use
service/2, and unaffected definitions retain service/1. The accepted entity-core types govern
exact API signatures and ownership: PreparedOutcome owns its loaded instance; its public lifetime
is the definition lifetime. Diagnostic inventory YAML remains unchanged and is not serialization.

## Boundary

The new pure crate is `crates/generate/ess-entity-runtime`. It depends on
`ess-service-contract`, `ess-compiler`, `ess-domain`, `ess-primitives`, `ess-synth`, and the exact accepted
`entity-core` revision, plus `serde_json` for ER's typed template values and `sha2` for the exact
synthesis-plan digest. It has no dependency on an executor, store, Eventlog, Service SDK, async
runtime, filesystem, network, or database. It projects one already admitted `ServiceIr` into
validated `service/1`, `service/2`, or `service/3` definitions and a typed plan for the facts and policies a host
still owns. It never executes, persists, deploys, or selects a component.

`ServiceIr` remains the selection authority. In particular, the projector uses
`ServiceIr::{component,operations,owned_entities,source,capabilities}` and the entity closure already
proved by `ess-service-contract::extract`; it does not infer a component from an SDK service name,
walk all commands in an owned domain, or treat contextual relation entities as newly owned. The
input retains the exact `EssIr` and `SynthesisPlan` admission performed by
`ess-service-contract::extract` (`crates/specify/ess-service-contract/src/lib.rs`). No `EssIr`
field, canonical byte, digest, or format changes.

## Public Rust interface

The Rust interface below is normative. The companion ESS model is a diagnostic inventory only:
its `*Inventory` and `*Kind` types are explicitly lossy summaries, not a serialization, alternative
definition, or executable replacement of any public Rust type. Definition bodies and source
capabilities retain their existing typed homes. Implementors must not generate the Rust interface
from the inventory; new output has no persisted envelope until the separately versioned `/4` design.

The crate exposes owned output and no unchecked output constructor:

```rust
pub struct LoweringOptions {
    pub definition_versions: BTreeMap<QualifiedName, NonZeroU32>,
    pub scales: BTreeMap<QualifiedName, BTreeMap<String, Vec<String>>>,
}

pub fn lower(
    service: &ServiceIr<'_>,
    options: &LoweringOptions,
) -> Result<LoweredService, LoweringDiagnostics>;

pub struct LoweredService {
    component: ComponentName,
    source_digest: String,
    synthesis_digest: String,
    target_revision: String,
    definitions: BTreeMap<QualifiedName, ValidatedDefinition>,
    bindings: BindingPlan,
}

pub const ENTITY_RUNTIME_REVISION: &str =
    "9ee145e888368c668cdaaaa9fa22b8334149616f";

pub struct BindingPlan {
    commands: BTreeMap<QualifiedName, CommandBinding>,
    requirements: Vec<BindingRequirement>,
    source_capabilities: Vec<PlannedCapability>,
}

pub struct CommandBinding {
    pub target: DefinitionCoordinate,
    pub entrypoint: RuntimeEntrypoint,
    pub instance: InstanceBinding,
    pub slots: BTreeMap<BindingSlot, BoundValue>,
    pub operation_fields: BTreeMap<OperationFieldCoordinate, OperationFieldFulfillment>,
}

pub struct DefinitionCoordinate {
    pub entity: QualifiedName,
    pub version: NonZeroU32,
}

pub enum RuntimeEntrypoint {
    Create,
    Operation { name: QualifiedName },
}

pub enum InstanceBinding {
    Created {
        logical_identity: IdentityValue,
        observed_at: EventFieldCoordinate,
    },
    SelectedOutcome {
        identities: BTreeMap<OutcomeName, SelectedCreationIdentity>,
    },
    Supplied { input_field: String },
}

pub struct SelectedCreationIdentity {
    pub logical_identity: IdentityValue,
    pub observed_at: EventFieldCoordinate,
}

pub enum IdentityValue {
    InputField { field: String },
    Literal { value: serde_json::Value },
    Bound { slot: BindingSlot },
}

pub struct EventFieldCoordinate {
    pub outcome: OutcomeName,
    pub occurrence: usize,
    pub event: QualifiedName,
    pub field: String,
}

pub struct BindingSlot(u32);

pub struct BoundValue {
    pub target: BoundTarget,
    pub type_ref: ResolvedTypeRef,
    pub source: BoundSource,
    pub presence: BoundPresence,
}

pub enum BoundPresence {
    Required,
    Optional,
}

pub struct OperationFieldCoordinate {
    pub outcome: OutcomeName,
    pub field: String,
}

pub struct OperationFieldFulfillment {
    pub type_ref: ResolvedTypeRef,
    pub actions: OperationFieldActions,
}

pub enum OperationFieldActions {
    Required,
    Optional,
}

pub enum BoundTarget {
    ExternalEvidence { outcome: OutcomeName },
    LogicalIdentity { at: EventFieldCoordinate },
    EntityField { outcome: OutcomeName, field: String },
    EventField { outcome: OutcomeName, occurrence: usize, event: QualifiedName, field: String },
    ResponseField { outcome: OutcomeName, field: String },
}

pub enum BoundSource {
    Identity,
    ResponseField { field: String },
    Generated,
    Undetermined,
    Conversion { from: ResolvedTypeRef, because: String },
    External { cause: String },
}

pub enum SemanticLocation {
    EntityIdentity { entity: QualifiedName, field: String },
    EntityField { entity: QualifiedName, field: String },
    CommandInput { command: QualifiedName, field: String },
    CommandResponse { command: QualifiedName, field: String },
    EventField { event: QualifiedName, field: String },
    BindingTarget { command: QualifiedName, target: BoundTarget },
}

pub struct RelationCoordinate {
    pub source: QualifiedName,
    pub relation: String,
    pub target: QualifiedName,
    pub via: String,
}

pub enum BindingRequirement {
    IdentitySupplied {
        command: QualifiedName,
        value: IdentityValue,
        observed_at: EventFieldCoordinate,
    },
    ResponseFieldSupplied { command: QualifiedName, outcome: OutcomeName, field: String, slot: BindingSlot },
    GeneratedFieldSupplied { command: QualifiedName, target: BoundTarget, slot: BindingSlot },
    UndeterminedFieldSupplied { command: QualifiedName, target: BoundTarget, slot: BindingSlot },
    ConversionSupplied { command: QualifiedName, target: BoundTarget, slot: BindingSlot, from: ResolvedTypeRef, to: ResolvedTypeRef, because: String },
    ExternalEvidenceSupplied { command: QualifiedName, outcome: OutcomeName, cause: String, slot: BindingSlot },
    OperationFieldPolicySupplied { command: QualifiedName, target: OperationFieldCoordinate, fulfillment: OperationFieldFulfillment },
    TimestampSpelling { primitive: Primitive, at: SemanticLocation },
    UuidSpelling { at: SemanticLocation },
    BytesSpelling { at: SemanticLocation },
    DecimalSpelling { at: SemanticLocation },
    Binary64Spelling { at: SemanticLocation },
    ScaleDeclaration { entity: QualifiedName, name: String, values: Vec<String> },
    RelationExistence { relation: RelationCoordinate },
    RelationCardinality { relation: RelationCoordinate },
    RelationOwnership { relation: RelationCoordinate },
    ErrorPayload { command: QualifiedName, outcome: OutcomeName, error: QualifiedName, fields: Vec<ResolvedField> },
    RevisionExpectation { command: QualifiedName, entity: QualifiedName },
}

pub struct LoweringDiagnostics(Vec<LoweringDiagnostic>);

pub struct LoweringDiagnostic {
    pub code: LoweringCode,
    pub path: String,
    pub message: String,
}

pub enum LoweringCode {
    MissingDefinitionVersion,
    UnknownDefinitionVersion,
    UnknownScaleEntity,
    StatelessCommandUnsupported,
    CommandSpansEntities,
    MixedEntrypointUnsupported,
    MultipleCreationCommands,
    AmbiguousInstanceBinding,
    AcceptingOutcomeWithoutSubject,
    NullableElementUnsupported,
    RecursiveTypeUnsupported,
    OptionalBoundOutputUnsupported,
    OperationFieldFulfillmentUnsupported,
    OperationIdentityMutationUnsupported,
    LiteralShapeUnsupported,
    ClearedValueUnsupported,
    SilentPreserveUnsupported,
    TargetDefinitionRefused,
}
```

The concrete implementation may make fields private and expose read-only accessors, as
`ServiceIr` does. It must keep these distinctions as closed enums. `BindingSlot` values are assigned
after requirements are collected and sorted by `(command, outcome source index, target kind,
occurrence, field)`, then numbered from zero. Definitions refer to them as
`$args.bound.b00000000`, `$args.bound.b00000001`, and so on. They are opaque coordinates whose
meaning lives in `BoundValue`; no user string is parsed as a slot and no arbitrary metadata map is
introduced. `Required` produces a required argument leaf and an ordinary template. `Optional`
produces an optional leaf with `DeclaredDefault::Absent` and a `PresentArgument` in exactly one of
`set_if_present`, `payload_if_present`, or `responds_if_present`. Missing means absent and is never
rewritten to JSON `null`.

Every bound output whose ESS semantics require one value is `Required`. Every host-bound top-level
creation, event-payload, or response member whose ESS semantics permit a per-invocation absence is
`Optional`. The destination's outer optionality alone does not merge values or create a default;
the source mapping and semantic reuse decide which slot owns the choice.

Slots represent semantic values rather than output positions. Reuse of one ESS response value by
an entity field, event member, and response member points every template or `PresentArgument` at the
same slot, so the normalized presence and value are identical in every output. Independently
omitted source values remain distinct slots even when their types match.

`operation_fields` is separate from value slots because one selected operation branch needs an
action, not merely a value. For each non-identity entity field omitted from an accepting operation
outcome's `sets`, the projector emits one `OperationFieldCoordinate` and one
`OperationFieldPolicySupplied` requirement. A required field admits `Set(value)` or `Preserve`; an
outer-optional field also admits `Remove`. The closed shape states that type-level difference and
leaves no action implicit. A refusing branch has no
operation-field requirement. The identity is never a fulfillment target: an operation that names
the identity in `sets` is `OperationIdentityMutationUnsupported`, while an omitted identity retains
ER's checked identity mirror.

The lowerer describes these requirements but does not choose an action. SDK `/4` must bind every
coordinate exactly once to the typed fulfillment policy described below, reject missing and extra
bindings, and preserve the action returned for that invocation. Until the target capability exists,
the lowerer returns `OperationFieldFulfillmentUnsupported` at every such coordinate and no partial
`LoweredService`; it does not silently compile the coordinate as `Preserve`.

The spelling, relation, error, revision, scale, and operation-field variants have no ordinary value
slot when they state a policy rather than provide an argument. `SemanticLocation` prevents these requirements from losing
which ESS surface owns a field while avoiding a free-form property bag. This is the concrete form
of the closed obligation list in Entity
Runtime's `docs/design/service-semantics-v0.1.md` sections 8 and 9. An omitted ESS payload or creation
field is `Undetermined`, while an explicit `ResolvedPayloadValue::Generated` is `Generated`; the
two are not collapsed. `TimestampSpelling` carries a primitive discriminator and covers both
`Timestamp` and `Duration`, matching the accepted ER obligation name. `UndeterminedFieldSupplied`
and `ConversionSupplied` are the two source-specific additions needed to make ESS omission and a
declared conversion executable; neither adds a kernel capability.

`source_capabilities` clones the selected `PlannedCapability` values in their original synthesis
plan order. Existing `ImplementationObligation` and `SynthesisRefusal` values therefore remain
visible to `/4` composition. `source_digest` is `ServiceIr::source().source_digest()`;
`synthesis_digest` is lowercase SHA-256 of `ServiceIr::source_plan().to_canonical_json()` bytes,
the same rule SDK `/3` already uses; and `target_revision` is the constant above. A successful pure
projection does not upgrade any source disposition to generated.

`LoweringDiagnostic` has a closed `LoweringCode`, a stable semantic path, and a message. Independent
diagnostics accumulate in entity-name, command-name, source-outcome-index, then field order. There
is no partial `LoweredService` on error. Important codes are named in the refusal section below.

## Definition identity, naming, and closure

An ESS entity has no ER `u32` definition version. The projector therefore requires exactly one
nonzero version for every entity in the selected closure and refuses missing or extra entries with
`MissingDefinitionVersion` or `UnknownDefinitionVersion`. It must not parse the ESS system version,
truncate a digest into 32 bits, or silently choose `1`. The chosen `(qualified entity name, version)`
is copied into every command binding and later `/4` source binding. A different definition under the
same coordinate is a caller error which the `/4` digest comparison and durable definition admission
must refuse.

`scales` may omit an entity, which means the exact empty scale context. A scale entry for an entity
outside the selected closure is `UnknownScaleEntity`; blank names, empty scales, or other invalid
declarations are returned through `TargetDefinitionRefused`. No scale is inferred from primitive
names or chronological ordering.

The ER entity, operation, event, error, relation target, and relation source spellings are the exact
ESS semantic names (`QualifiedName::to_string()`), never display or wire aliases. Entity field,
identity, input, response, event field, relation, outcome, and state spellings are their declared
semantic names. The SDK owns wire decoding and naming aliases.

The definition set is exactly the `EntityLifecycle` closure selected by `ServiceIr`: owned entities,
their outgoing relation targets, and the declaring source of each incoming `Owns` relation, to a
fixpoint. All closure definitions are required so `Registry::validate_all` can decide target,
carrier, owner, and duplicate-carrier rules. A contextual entity receives its schema, lifecycle,
invariants, identity, and relations, but only accepted service commands become callable entries in
`BindingPlan`. Direct use of a contextual definition outside the binding plan is outside the target
contract.

## Entity and value lowering

Every emitted `EntityDefinition` has `number_observation: NumberObservation::SourceNumber1`, an
empty `projections` map, and the caller's exact scales for that entity (empty is a valid declaration
and makes text ordering `Unknown`). Its semantics are `Semantics::Service2` exactly when any outcome
uses `set_if_present`, `payload_if_present`, or `responds_if_present`; otherwise it remains
`Semantics::Service1`. Conditional keys are never written under `service/1`. ESS views remain SDK
query/projection bindings; ER's existing grouped `ProjectionDefinition` is not a general ESS view
and is not used as an approximation.

The instance schema contains the identity first in source reasoning, but is stored in ER's ordered
map under its semantic name alongside the declared entity fields. The identity is always
`required: true`, including `Optional<T>` identities: ESS admits that declaration while its own
conformance input refuses an absent instantiated identity, and ER's identity mirror needs a value.
Other fields are required exactly when their outer type is not `Optional`.

The structural mapping is:

| ESS `ResolvedTypeRef` / `ResolvedBody` | ER `FieldDefinition` |
| --- | --- |
| `String` | `FieldKind::String` |
| `Boolean` | `FieldKind::Boolean` |
| `Integer` | `FieldKind::Integer` |
| `Decimal` | `FieldKind::Number` |
| `Binary64` | `FieldKind::Binary64` |
| `Timestamp`, `Duration`, `Uuid`, `Bytes` | `FieldKind::String` plus the matching spelling requirement |
| outer `Optional<T>` in an object field or union payload | lower `T`, with `required: false` |
| `List<T>` | `FieldKind::Array`, with `items` recursively lowered |
| `Map<K,V>` | `FieldKind::Map`, exact `MapKey`, and recursively lowered `items` |
| newtype | its recursively lowered representation; its invariant is retained separately |
| struct | closed `FieldKind::Object` with declared fields as `properties` |
| enum | `FieldKind::Enum` with variants in declaration order |
| union | `FieldKind::Union` with the exact tag and variant map; ER derives the adjacent content key |

No field lowers to `Json`. Recursive named bodies are detected with an active-handle stack rather
than expanded forever. An `Optional` below a list or map value means a JSON `null` in ESS's existing
wire projection (`ess-gen/src/types.rs::type_ref`), while ER `FieldDefinition.required` can express
only object-key absence and `validate_value` refuses that null. These shapes return
`NullableElementUnsupported`, citing the complete type path. Recursive bodies return
`RecursiveTypeUnsupported`. Neither is rewritten to `Json` or silently made required.

ESS `ResolvedPayloadValue::Literal` is decoded against its already resolved target type. Boolean and
numeric literals become their typed JSON scalar without an `f64` conversion; text-shaped primitives,
enums, and their newtypes remain JSON strings. A non-scalar literal target returns
`LiteralShapeUnsupported`. The projector never guesses a structured value from text.

## Predicates and invariants

The mapping is one for one with the accepted `service/1` condition types in
`entity-core/src/definition.rs`: `Always`/`Never` become boolean conditions; `All`, `Any`, and `Not`
recurse; `Defined` becomes `Exists`; `Compare` becomes `Condition::Compare`; `Truthy` becomes
`Condition::Truthy`; `AnyOf` becomes `In` with the exact literal vector and `NoneOf` its `Not`; ESS
`Forall` and `Exists` become ER `ForAll` and `ForAny`. Empty `All`, `Any`, `AnyOf`, and `NoneOf` retain
their ESS truth tables through the literal or membership forms and are not discarded. Comparison
operators map `Eq/Ne/Lt/Le/Gt/Ge` to `Eq/Ne/Lt/Lte/Gt/Gte`.

Path rewriting is context typed:

- an outcome predicate root is the command input and becomes `$args.input.<path>`;
- an entity invariant field becomes `$fields.<path>`, its identity uses that same field path, and
  its `state` pseudo-field becomes `$state`;
- a newtype's `value` root is replaced by the containing field/element path;
- a struct invariant root is appended to the containing object path;
- a quantifier binder becomes `$<binder>` inside its body, shadows an equal outer or fixed-root
  spelling exactly as ESS does, and every other path continues through the enclosing rewrite.

Entity invariants become `RuleDefinition`s in source order. Every reachable newtype and struct
invariant is expanded at each use site. An optional field wraps the rule as
`not exists(path) OR rule`; list and map-value positions wrap it in `for_all`; union variants guard
the invariant by the exact tag and rewrite through the derived content path. Visited use-site and
active-type stacks make the walk deterministic and expose recursion as the diagnostic above. No
nominal invariant is reduced to a schema constraint or dropped.

## Commands, outcomes, and events

Each selected command must have exactly one runtime entity target across all subject-bearing
outcomes. A command with no subject-bearing outcome is `StatelessCommandUnsupported`; a command
whose outcomes name more than one entity is `CommandSpansEntities`. This is a current ER type
boundary: `decide` receives one `EntityInstance`, and `OutcomeDefinition` carries no subject entity.

A command is a creation command when every subject-bearing outcome uses
`ResolvedEffect::Creates`; it becomes that entity's single `CreateDefinition`. Every other command
must contain no `Creates` outcome and becomes an `OperationDefinition` keyed by the command's exact
qualified name. Mixed creation/existing-instance commands return `MixedEntrypointUnsupported`.
Two selected creation commands for one entity return `MultipleCreationCommands`; ER has one unnamed
creation entrypoint. All subject-bearing existing-instance outcomes must use the same
`ResolvedInstance::Supplied` command field, recorded in `InstanceBinding::Supplied`; disagreement is
`AmbiguousInstanceBinding`. Every accepting creation outcome must use
`ResolvedInstance::Observed`. The projector locates each outcome's first matching emitted event
occurrence in source order, exactly as the compiler resolved that branch's instance surface, and
records its outcome, occurrence, event, and field in `EventFieldCoordinate`.

The ER argument schema always has exactly two required closed object fields: `input`, containing the
ESS command input field for field, and `bound`, containing only the typed slots this command needs.
This is the accepted ER contract. The host may fill slots, but only ER selects an outcome.

Outcomes preserve source order except for the semantic normalization ER requires: the single ESS
`Otherwise` branch is placed last among non-`WrongState` branches, and `WrongState` is the separate
fallback. `When` lowers to `when`; `SubjectState` lowers to `in_state` plus its optional `when`;
`Otherwise` has no selector; `WrongState` sets `wrong_state`; and `External { cause }` gets one
boolean `ExternalEvidenceSupplied` slot and a `Truthy` guard over that slot. The host supplies facts,
not an outcome name. ER still performs ordered selection and records the selected outcome.

`Creates`, `Updates`, and `Moves` lower to the same-named `OutcomeEffect`; a move copies the exact
transition `from` list and `to` state. No update becomes a self-transition. An accepting
subjectless branch mixed into an entity command is `AcceptingOutcomeWithoutSubject`, except the
source-admitted non-refusing `WrongState` branch: running any other subjectless success through ER
would create or revision an instance ESS did not say it changed. Refusing branches lower to
`RefusalDefinition` with the qualified error name and source summary as the optional message; error
field values remain `ErrorPayload` requirements.

Each `sets` entry becomes a branch `set` template. `InputField` with no conversion reads
`$args.input.<field>`; `Literal` uses the typed scalar; `Generated`, `ResponseField`, and every
converted input read a dedicated bound slot. On creation, the observed event field determines the
logical identity source: an unconverted `InputField` or `Literal` becomes the corresponding
`IdentityValue`; every generated, response-derived, converted, or omitted identity uses one typed
bound slot. Before allocating a slot, the projector compares every accepting branch's complete
typed identity mapping: input or response coordinate, exact literal spelling, generated or omitted
ownership, target type, conversion source and reason. Structurally equal mappings retain the
existing `InstanceBinding::Created` value, representative observation, one slot and one
`IdentitySupplied` requirement. Its value template populates every branch's entity identity and
exact observed event field; the host derives the address with `entity_core::identity::address` and
calls `decide_create` as before.

Unequal mappings produce `InstanceBinding::SelectedOutcome`, a closed ordered map containing each
accepting outcome's authored `IdentityValue` and exact observation coordinate. Each branch's value
template populates only that branch's entity identity and observed event field. The host supplies
the declared input and bound values but selects no source and interprets no predicate; it calls
`decide_create_derived`, and ER selects the outcome once before deriving the address from the
selected validated fields. The projector also supplies bound slots for every other required entity field
not determined by `sets`. An optional undetermined creation field remains a per-invocation host
choice of absence or a present typed value. It gets one `BoundPresence::Optional` slot whose
argument leaf is optional and has no default, and the creation branch maps it with
`set_if_present`. Absence inserts no member; presence copies the exact validated value. This is the
rule for billing `CreateInvoice.note`, `CreateInvoice.issued_at`, and gatepass
`RegisterVisit.badge`; none has a lowerer-chosen default. Gatepass's deliberately incomplete
creation mapping therefore remains an explicit binding decision rather than a guessed same-name
copy.

An operation omission has different meaning. ESS deliberately does not check a field with no
source: a branch naming two fields says what it determines, not what happens to every other field
(`ess-compiler/src/resolve.rs::Resolver::sets`; the conformance table in
`ess-conformance/src/synthesize.rs` asserts only row identity when `sets` is absent). Consequently,
the lowerer must visit every declared non-identity entity field on every accepting `Updates` or
`Moves` outcome. A field named by `sets` uses that ordinary mapping. Every omitted field becomes the
typed operation-field requirement above; no field is specialized to unchanged merely because ER
starts its operation evaluation by cloning `old_fields`.

The complete action algebra is:

| entity field | admitted host action | result |
| --- | --- | --- |
| required | `Set(value)` | validate the exact field type, then replace the member |
| required | `Preserve` | retain the loaded member byte-for-byte |
| required | `Remove` | refuse before mutation as `required_field_removal` |
| outer-optional | `Set(value)` | validate the exact inner type, then insert or replace the member |
| outer-optional | `Preserve` | retain presence and value byte-for-byte |
| outer-optional | `Remove` | remove the member; absence is not JSON `null` |
| identity | any host action | never requested or accepted; the identity mirror remains immutable |

The action is per command, selected outcome, field, and invocation. An SDK policy may deliberately
return a different admitted action on a later invocation. Required and optional refer to the entity
field's outer presence only; nested optional values remain governed by the existing
`NullableElementUnsupported` rule. An action applies identically to `Updates` and `Moves`; the
selected effect alone decides whether the lifecycle state stays put or changes.

This is the full affected class. At the pinned target, its first required real failures are
`billing.invoice.IssueInvoice.issued.issued_at` and
`gatepass.visit.AdmitVisitor.admitted.badge`. The accepted billing realization fills the first from
its host clock (`examples/billing-realization/src/invoice.rs::issue_invoice`), and the accepted
gatepass realization fills the second from `AdmitVisitor.badge`
(`examples/gatepass-realization/src/visit.rs::admit_visitor`). The other omitted operation fields
are requirements too. SDK `/4` must enumerate their source-backed policies, including every
deliberate `Preserve`; neither those two examples nor the remaining class license a lowerer default.

Each emitted occurrence becomes one `EventDefinition`, in source order, including duplicates. Its
type is the qualified event name and its payload is a closed object over every declared event field.
Mapped input/literal fields use the same rules as `sets`; generated, response, converted, and omitted
fields use occurrence-specific bound slots. An optional host-owned event member uses an optional,
no-default slot and `payload_if_present`; absence omits the member and presence copies the validated
value. An empty event has `{}`. Zero, one, and many emissions therefore remain distinct for both
creation and operations.

Every declared command response becomes the entrypoint response schema. On each accepting outcome,
every required response field reads a `ResponseFieldSupplied` bound slot. An optional response field
uses an optional, no-default slot and `responds_if_present`. Response values subsequently reused by
`sets` or event payloads point to that same outcome-and-field slot, including its single presence
choice. A missing slot omits every conditional destination; a present slot supplies the same value
to each. `null` is a present value and is refused during argument validation when the slot type does
not admit it.

## Relations and binding ownership

Every `ResolvedRelation` becomes an ER `RelationDefinition` under its declared name, copying
`Owns`/`References`, qualified target, `One`/`Many`, and `via`. Carrier fields keep the related
entity's logical identity shape rather than becoming a generic `Ref`. For `Owns`, the target field
carries the source identity once and is required for either cardinality. For `References/One`, the
source field carries the target identity and preserves outer optionality. For `References/Many`, it
is one required array around the target identity shape. These are the exact rows enforced jointly by
`EntityDefinition::validate` and `Registry::validate_all` at the accepted ER revision.

The kernel validates declaration shape, registered targets, carrier types, unique ownership, and
unique carrier claims. The binding plan retains `RelationExistence`, `RelationCardinality`, and
`RelationOwnership` because resolving another instance, enforcing membership, and applying owner
deletion behavior require graph/storage access. No projector performs those operations.

The host also owns identity generation, value conversion, generated and undetermined fields,
external evidence, error payloads, primitive wire spelling, scale declarations, and expected
revision. For a shared creation binding it derives the storage address through
`entity_core::identity::address`; for a selected-outcome binding ER derives it after selection. It
may not publish that address as the logical identity. Authentication, authorization, realm, transport,
hosting, idempotency, queries, content staging, external effects, and projection delivery remain SDK
bindings and are not added to ER definitions.

## Mandatory target validation

Lowering succeeds only after all four checks complete:

1. every raw definition is converted through `ValidatedDefinition::new` and retained for output;
2. a raw clone of every retained definition is passed to `Registry::register`, whose accepted API
   validates again and registers it under the definition's exact coordinate;
3. `Registry::validate_all` passes over the complete selected closure;
4. rebuilding from the same `ServiceIr` and `LoweringOptions` produces equal definitions, binding
   plan, diagnostic order, and canonical serialized ER definition bytes.

ER validation failures are returned as `TargetDefinitionRefused` diagnostics carrying the entity
coordinate and every underlying `DefinitionError` in ER's order. The projector does not catch a
validation error and weaken or rewrite the definition around it.

## Finite source forms refused by this target

| Code | Exact source form | Current limiting target type |
| --- | --- | --- |
| `StatelessCommandUnsupported` | accepted command with no subject-bearing outcome, for example `billing.email.SendEmail` | ER `decide` acts on one `EntityInstance`; there is no stateless definition/decision type |
| `CommandSpansEntities` | one command has outcomes whose `ResolvedSubject.entity` values differ | `OperationDefinition` is nested in one `EntityDefinition` and an outcome has no entity field |
| `MixedEntrypointUnsupported` | one command mixes `Creates` with `Updates`/`Moves` | ER has separate `decide_create` and `decide` entrypoints |
| `MultipleCreationCommands` | two selected commands create the same entity | `EntityDefinition` has one `CreateDefinition`, with no command name |
| `AmbiguousInstanceBinding` | existing-instance outcomes use different supplied identity fields | the host must load one instance before ER selects a branch |
| `AcceptingOutcomeWithoutSubject` | a non-refusing ordinary outcome in an entity command has no subject | ER would create or revision the target instance despite no ESS effect |
| `NullableElementUnsupported` | `Optional<T>` below `List` items or `Map` values, for example `List<Optional<String>>` | ER `required` represents member absence only and value validation refuses JSON `null` |
| `RecursiveTypeUnsupported` | a recursively referenced named type reaches a lowered field | ER embeds `FieldDefinition` recursively and has no typed definition reference |
| `OptionalBoundOutputUnsupported` | conditional presence is required below a nested creation, event-payload, or response member | ER `PresentArgument` conditionally inserts only top-level creation, event-payload, and response members |
| `OperationFieldFulfillmentUnsupported` (historical only) | formerly missing operation-field capability at da5d3687 | resolved at the accepted target; emit typed fulfillment requirements instead |
| `OperationIdentityMutationUnsupported` | an operation outcome names the entity identity in `sets` | ER's identity mirror is immutable; host fulfillment never targets it |
| `LiteralShapeUnsupported` | a string-form ESS literal targets a non-scalar representation | no exact structured value can be recovered without guessing |
| `ClearedValueUnsupported` | an operation outcome clears an `Optional` field (ESS `Cleared`) | ER admits `Remove` only as a host-supplied action; a definition cannot state a removal, and handing the field to the host would let it choose `Set` |
| `SilentPreserveUnsupported` | a `Preserves` outcome that returns no response | ER refuses a branch with no effect, write, event or response (`UnobservableOutcome`) |

These refusals are bounded and path-bearing. They do not make billing or gatepass acceptance pass by
subtraction. At target `da5d3687`, `billing.invoice`'s `invoice-service` is blocked at
`billing.invoice.IssueInvoice.issued.issued_at` and `gatepass.visit`'s `pass-service` is blocked at
`gatepass.visit.AdmitVisitor.admitted.badge`; the diagnostic also names every other omitted
non-identity operation field rather than stopping at those first fixture witnesses. Complete fixture
acceptance resumes only after the target amendment below exists, is pinned, and passes its source
and compatibility checks. The separate
`billing.email` component currently demonstrates `StatelessCommandUnsupported`; whole-billing
service acceptance retains `SendEmail` as the existing SDK/provider external-effect obligation,
reachable through the selected publication binding and exact source capabilities. It does not
require directly projecting the separate email component into ER. The end-to-end effect test remains
required; preserving a binding without executing it is not whole-billing acceptance. This projector
does not synthesize a singleton entity to hide a direct stateless refusal.

## Historical target capabilities and gap at da5d3687

Entity Runtime `da5d368756f5a63e4b2efd5589f7bc3441cd7aff` implements the two previously
reviewed capabilities at their required boundaries. Actual `entity-core` types are normative for their behavior; the
lowerer's Rust contract above is normative for projection and binding data.

1. **Pre-load selection.** `entity_core::Runtime::decide_before_load` and the free
   `decide_before_load` return `PreloadDecision::{Refused,Load}`. A refusal carries the selected ER
   `Refusal` and requires no subject. `Load` carries an opaque `PreparedOperation`; its
   `PreparedSubject` exposes the exact entity, version, and storage id, and
   `PreparedOperation::continue_with` accepts only that loaded instance. The continuation retains
   the validated definition, operation, normalized arguments, and expected identity. Entity,
   subject, and state checks occur in that order. Ordered partial evaluation stops when subject
   facts are needed; it never skips to a later default.
2. **Conditional output presence.** `Semantics::Service2`, `PresentArgument`,
   `OutcomeDefinition::{set_if_present,responds_if_present}`, and
   `EventDefinition::payload_if_present` copy an optional no-default argument leaf only when it is
   present. Registration requires a closed path with required parents, an optional leaf, complete
   source/target field-definition equality for entity and response destinations, no ordinary-map
   conflict, and an allowed top-level destination. An event destination must be a nonblank new key
   in an object payload and is typed by the source leaf. Missing is absence; present `null` is a
   value and must pass the declared type. Reusing one argument path shares both presence and value
   across outputs.

`service/2` persists through `er.record/3` and reconstructs through `er.request/3`; `er.batch/1`
continues to carry each member's own domain. Existing `kernel/1`, `service/1`, `er.record/1`,
`er.record/2`, `er.request/1`, and `er.request/2` bytes and behavior remain unchanged. Older readers
refuse `/3`, and conditional keys under earlier semantics are registration errors. The lowerer does
not invent a fallback or downgrade.

That target does not implement operation-field fulfillment. Its runtime clones `old_fields` and then
inserts each ordinary `set` result (`entity-core/src/runtime.rs::decide`); it has no remove operation.
An absent ordinary argument is a template error, not a remove instruction. Validation also returns
`ConditionalSetOnOperation` for every operation `set_if_present`
(`entity-core/src/validation.rs::validate_conditional_outcome`). Therefore the following candidates
were tested against the accepted types and rejected:

- an ordinary required bound slot expresses `Set`, but it cannot express `Remove`;
- `$old_fields.<field>` expresses an authored `Preserve`, but making it the default would invent the
  source policy this correction is required to retain;
- an optional no-default slot plus ordinary `set` fails when absent, while `set_if_present` is
  rejected on an operation and would still conflate preserve with remove;
- a fabricated value used to let ER select a branch records a decision on a fact the host never
  supplied;
- selecting an outcome in the SDK, or patching the entity after ER records its decision, creates a
  second decision authority and makes record/replay false.

## Required Entity Runtime amendment — now accepted at 250f699

The required amendment is one operation-only fulfillment phase in `entity-core`; it is not a
stateless engine, registry, host outcome selector, or general property bag. `OutcomeDefinition`
gains one closed, ordered map available only under new `Semantics::Service3`:

```rust
pub struct OutcomeDefinition {
    // existing fields unchanged
    pub fulfills: BTreeMap<String, OperationFieldRequirement>,
}

pub struct OperationFieldRequirement {
    pub actions: OperationFieldActions,
}

pub enum OperationFieldActions {
    Required,
    Optional,
}

pub enum OperationFieldAction {
    Preserve,
    Set { value: serde_json::Value },
    Remove,
}
```

Registration permits `fulfills` only on accepting operation outcomes, rejects a key also present in
`set`, rejects unknown fields and the identity field, and requires `Required` versus `Optional` to
equal the target field's outer presence. `Required` admits `Set` and `Preserve`; `Optional` admits
all three actions. `Set` validates against the exact target `FieldDefinition`; `Remove` on a required
field is a typed refusal. There is no user-supplied field name or schema in the invocation.

`PreparedOperation` retains its existing input-only partial evaluation. Add the following post-load
surface without changing the existing `continue_with` signature or behavior for `kernel/1`,
`service/1`, and `service/2` definitions:

```rust
pub enum LoadedDecision<'a> {
    Complete(Evaluation),
    NeedsFulfillment(PreparedOutcome<'a>),
}

impl PreparedOperation<'_> {
    pub fn select_with<'a>(self, instance: &'a EntityInstance)
        -> Result<LoadedDecision<'a>, CoreError>;
}

impl PreparedOutcome<'_> {
    pub fn outcome(&self) -> &str;
    pub fn requirements(&self) -> &BTreeMap<String, OperationFieldRequirement>;
    pub fn complete(
        self,
        actions: BTreeMap<String, OperationFieldAction>,
    ) -> Result<Evaluation, CoreError>;
}
```

`select_with` performs the existing entity, exact subject, state, ordered outcome selection,
refusal, state admission, and precondition steps. A refusal returns `Complete(Refused)` and never
requests fulfillment. An accepted branch with an empty map follows the existing path. Otherwise it
returns the opaque selected continuation. `complete` requires exactly the advertised field keys,
applies actions to the loaded field map, validates the resulting schema, checks the identity mirror
and invariants, materializes events and response from that one resulting field map, and builds one
decision. The continuation owns the selected branch; the SDK cannot substitute an outcome. The
existing `continue_with` delegates to this flow and returns a typed `FulfillmentRequired` error only
when called on a `service/3` branch that actually needs actions.

This placement handles facts unavailable before load. The SDK invokes no fulfillment policy while
`decide_before_load` can still answer an input-only refusal. After the exact subject is loaded,
`select_with` chooses the branch before the SDK sees `requirements()`. Only then does the SDK call
the bound policies with the normalized command input, verified context, read-only loaded instance,
selected outcome coordinate, and named field requirement. No placeholder action is used to probe a
branch.

The SDK `/4` policy is a closed binding per exact command/outcome/field:

```rust
pub enum OperationFieldPolicy {
    Preserve,
    Remove,
    CommandField { field: String },
    Obligation { name: DefinitionId },
}

pub trait OperationFieldObligation {
    fn fulfill(
        &self,
        context: OperationFieldContext<'_>,
    ) -> Result<OperationFieldAction, UnmetObligation>;
}
```

`Preserve`, `Remove`, and `CommandField` are interpreted directly and type-checked at SDK `/4`
compile time; `Obligation` reuses the SDK's existing versioned `ObligationProviderId` admission and
returns one action per invocation. There is no lookup by an arbitrary runtime string: the compiled
realization plan carries the resolved provider and exact coordinate. Missing, duplicate, extra,
wrong-type, identity-targeting, required-field `Remove`, and undeclared-action bindings are compile
diagnostics. For the accepted fixtures, `IssueInvoice.issued.issued_at` binds a clock obligation that
returns `Set` once after selection, and `AdmitVisitor.admitted.badge` binds
`CommandField { field: "badge" }`. Every other omitted operation field has an explicit
source-backed policy, commonly `Preserve`; the compiler does not synthesize one.

A chosen `Set` value is inserted before event and response materialization. An event or response
explicitly bound to the same semantic entity value reads the post-action `$fields.<field>`; a
command-field binding such as gatepass badge reads the same normalized command value for both the
action and its already declared event payload. `Preserve` exposes the loaded value and `Remove`
exposes absence. Independent event/response obligations remain independent. Thus reuse is explicit
and one invocation never calls a value-producing obligation twice.

Record and replay must preserve the action as well as the result. Under new `service/3`,
`DecisionCommand::Execute` carries an ordered `fulfillments` map, `DecisionRecord` and every
`DomainEvent` carry an ordered `removed` field-name set beside `changed`, and replay supplies the
recorded actions to the same continuation before byte-comparing the complete decision. Folding
removes `removed` members before inserting `changed`; the two sets must be disjoint. This requires
new `er.record/4` and `er.request/4` domains. Existing `kernel/1`, `service/1`, `service/2`,
`er.record/1` through `/3`, `er.request/1` through `/3`, and `er.batch/1` bytes and behavior remain
unchanged; new readers accept all versions, older readers refuse `/4`, and a `fulfills` key under an
earlier service semantics is a registration error. The amendment is now accepted at `250f699`; the old target remains cited only for historical gap analysis.

## Ten-dimension crosswalk

| Dimension | ESS authority at `be604d87` | ER target at `28d06303` | Lowering rule |
| --- | --- | --- | --- |
| Creation | `ResolvedEffect::Creates`, `ResolvedInstance::Observed`, `ResolvedOutcome::{sets,emits}` in `ess-compiler/src/ir.rs` | `CreateDefinition`, `OutcomeEffect::Creates`, `decide_create`, `decide_create_derived`, `set_if_present` | one create command per entity; equal identity mappings retain one shared binding, unequal mappings retain each selected branch's authored identity; optional fields use no-default slots and conditional insertion |
| Updates | `ResolvedEffect::Updates`, partial `ResolvedOutcome::sets` | `OutcomeEffect::Updates`, `DecisionEffect::Updated`; accepted `fulfills` | exact state-preserving effect; explicit `Set`/`Preserve`/optional `Remove` for every omitted non-identity field; never a self-transition |
| Transitions | `ResolvedEffect::Moves { transition }`, partial `ResolvedOutcome::sets`, `StateMachine`, `EssIr::wrong_states` | `OutcomeEffect::Moves { from,to }`, service selection/refusal order; accepted `fulfills` | copy declared sources/destination and fulfill omitted fields after branch selection; ER owns the accepted `UnspecifiedMoveSource` behavior |
| Predicates | `Predicate`, `Operand`, `Quantified`, `FactValue` in `ess-primitives` | `Condition`, `Comparison`, `Quantifier`, `source-number/1` | typed path rewrite and one-for-one operator mapping, including empty membership and collection addressing |
| Invariants | `ResolvedEntity::invariants`, `ResolvedBody::{Newtype,Struct}` invariants | `EntityDefinition::invariants` and `RuleDefinition` | entity rules plus every nominal use-site rule, guarded/quantified for optional/collection/union positions |
| Identities | `ResolvedEntity::identity`, `ResolvedInstance::{Observed,Supplied}` | `IdentityDefinition`, `identity::address`, `decide_create_derived`, identity mirror | logical field stays typed; shared creation identities keep host address derivation, while branch-specific identities are derived by ER after selection |
| Relations | `ResolvedRelation`, `EssIr::relations_carried_by` | `RelationDefinition`, `Registry::validate_all` | exact kind/target/cardinality/via and carrier shape; external graph effects remain named obligations |
| Exact values | `ResolvedTypeRef`, `ResolvedBody`, `ResolvedPayloadValue`, exact `FactValue` | closed field kinds, exact JSON templates, `PresentArgument`, `Observed`; accepted `OperationFieldAction` | structural type mapping, no `f64` conversion, typed slots carry required or optional presence; operation actions preserve set/preserve/remove; shared semantics reuse one value |
| Outcomes | ordered `ResolvedCommand::outcomes`, `ResolvedCondition`, error/refusal and response | ordered `OutcomeDefinition`, `Evaluation`, `Refusal`, `PreloadDecision`; future opaque `PreparedOutcome` | ER selects and records before or after the exact load boundary; only a selected accepting branch requests host actions; default normalized last as required by ER semantics |
| Event order/multiplicity | ordered `ResolvedOutcome::emits: Vec<EventHandle>` and payload mappings | ordered `OutcomeDefinition::emits: Vec<EventDefinition>`, `payload_if_present` | emit zero/one/many, preserve duplicates and source order, use occurrence-specific slots and conditional members |

## SDK `/4` handoff and acceptance boundary

The SDK adds opt-in `service-definition/4`, `service-runtime-ir/4`, and
`service-realization-plan/4`; `/3` constants, strict readers, canonical fixtures, generated paths,
and behavior remain byte-for-byte unchanged. A `/4` runtime compilation takes the same exact `EssIr`
and `SynthesisPlan`, explicitly selects a component, calls `ess-service-contract::extract`, then calls
this lowerer with authored definition versions and scale declarations, then requires typed SDK
fulfillment policies for every operation-field coordinate. Its
persisted document binds at least the ESS source digest, synthesis digest, selected component, exact
ER revision, every `(entity, version)`, the complete definitions, and the complete binding plan.
Reload recompiles and compares; it never trusts persisted definitions independently of those inputs.

SDK `/4` retains authentication, authorization, realm, HTTP/Connector delivery, idempotency,
expected-version source, content staging, views/queries, external effects, and projection delivery.
For an existing-instance mutation it constructs `{input,bound}`, derives the ER storage address
from the supplied logical identity with `entity_core::identity::address`, and calls
`entity_core::Runtime::decide_before_load` with the binding's entity, version, address, and operation. A
`PreloadDecision::Refused` maps to the selected ESS error plus SDK-bound error payload with no state
lookup, append, or event. `PreloadDecision::Load` supplies the exact `PreparedSubject`; the SDK loads
that subject and passes the result only to `PreparedOperation::select_with`. It does not rebuild,
clone, or substitute the continuation. `Complete` is returned directly. On `NeedsFulfillment`, the
SDK resolves exactly the selected coordinate's compiled policies and passes their actions once to
`PreparedOutcome::complete`. An absent subject returns the existing unknown-subject result and
performs no append. A shared creation binding uses `decide_create`; a `SelectedOutcome` binding uses
`decide_create_derived`. Neither creation mode enters the load path or lets the SDK select an outcome.

This preserves `PayInvoice` ordering explicitly. Zero or negative amounts refuse as `rejected`
before any lookup, including for an unknown invoice identity. A positive amount returns `Load` for
that exact invoice; a known instance continues through ER selection and an unknown instance ends at
the SDK's unknown-subject boundary. Wrong-state selection occurs only after the exact instance is
loaded. The SDK does not run the old `/3` exactly-one-`Otherwise` selector, inspect ER guards, or
use the `ReducerEffect` decision path.

The SDK persists the complete ER decision record through the recorded adapter and uses ER replay.
Definitions using operation fulfillment require the accepted `/4` record/request domains; definitions
using conditional maps but no fulfillment retain the accepted `/3` domains, and definitions without
either may remain `service/1` and retain `/2` bytes. A zero-event accepted decision is still appended
as a decision. Existing public operation names and entrypoints remain the SDK definition's
responsibility. Billing email remains the existing SDK/provider publication binding; the stateless
email command is not moved into the ER decision engine.

Pure projection acceptance, including every required operation-fulfillment shape, ends when this
crate returns a validated, deterministic `LoweredService` and the complete fixture and mutation
controls pass. Omitted operation fields must lower to typed requirements at the accepted target.
This proves no storage behavior. Durable
adapter/service acceptance additionally requires the exact ER Eventlog adapter, complete decision
records, concurrency and idempotency behavior, restart/replay, and generated HTTP behavior at the
same pinned revisions. In-memory execution is not durable acceptance.

## Implementation and checks

The separately routed target amendment is finite: `entity-core` definition, validation, runtime,
record/replay and codec code plus focused tests; the recorded Eventlog adapter only needs the new
domain/version and `removed` folding path. It adds no executor, store selection, network, registry,
or SDK dependency. Its acceptance is:

1. registration covers every `fulfills` placement/type/identity/conflict refusal and proves a
   `service/3` definition round-trips deterministically;
2. required fields exercise `Set` and `Preserve` and refuse `Remove`; optional fields exercise
   `Set`, `Preserve`, and physical `Remove`, including absent and present starting states;
3. refusal branches request no policy, update and move branches apply the same actions while
   retaining their distinct effects, and identity substitution is refused;
4. pre-load input-only refusal performs no load or fulfillment, exact-subject continuation remains
   enforced, and a loaded-fact policy runs only after ER has selected its branch;
5. event/response reuse observes exactly the post-action value or absence, and one generated value
   is invoked once and reused;
6. record/replay and restart cover all three actions, including a removal where `changed` alone
   cannot rebuild the instance; mutating or dropping one recorded action or removed name refuses
   replay;
7. canonical `kernel/1`, `service/1`, `service/2`, `er.record/1` through `/3`, and `er.request/1`
   through `/3` vectors are byte-identical, old readers refuse `/4`, and new readers accept every
   old vector.

The lowerer source assignment remains the new pure crate, ESS workspace entries, focused tests, and
the ordinary changelog line. Against the current pin it must accumulate
`OperationFieldFulfillmentUnsupported` for every omitted non-identity operation field. It must name
at least the exact billing and gatepass paths above, return no `LoweredService`, and make no complete
fixture claim. It also tests `OperationIdentityMutationUnsupported`, while source shapes needing no
operation fulfillment still pass `ValidatedDefinition::new`, `Registry::validate_all`, repeated
typed equality, canonical target-byte equality, and unchanged source `EssIr` bytes/digest.

After an accepted target revision is pinned, the same lowerer fixture tests switch from those exact
diagnostics to complete definitions and `OperationFieldPolicySupplied` requirements. SDK `/4`
compile tests enumerate all operation-field coordinates, refuse missing/extra/wrong-type policies,
and bind the real fixtures as follows: billing issue time comes from the admitted clock obligation,
gatepass badge comes from the normalized command field, and every remaining omission has an
explicit realization-backed action. Complete billing/gatepass acceptance then asserts the resulting
entity fields as well as all definitions, coordinates, ordinary slots, nominal invariants, outcomes,
events, responses, relations, and inherited synthesis dispositions.

Creation and other optional-output tests remain unchanged: billing `CreateInvoice.note` and
`issued_at`, gatepass `RegisterVisit.badge`, optional event members, and optional response members
use no-default slots and the accepted `service/2` conditional maps. Shared slots retain one presence
choice, independent slots remain independent, and a definition with no conditional or fulfillment
map retains `service/1` bytes. Existing negative fixtures still cover every other refusal in the
table, missing/extra versions, invalid scales, accumulated target errors, stateless
`billing.email.SendEmail`, nullable elements, and relation closure.

SDK `/4` integration retains the full `PayInvoice` amount-by-identity matrix: negative and zero with
known or unknown identities refuse before lookup, fulfillment, and append; positive with a known
identity loads the exact subject and continues, including wrong-state selection; positive with an
unknown identity loads and ends at unknown subject. It adds restart/replay for `Set`, `Preserve`, and
`Remove`, `/4` framing for `service/3`, and unchanged `/1` through `/3` vectors. Billing email
continues through the existing publication/provider binding.

Required lowerer/SDK fault controls are applied one at a time and reverted:

1. map one `sets` or event field to a neighboring input/bound slot: the exact billing/gatepass value
   assertion fails (`wrong-field`);
2. remove the second of two emissions or collapse duplicate occurrences: the ordered occurrence
   assertion fails (`dropped-event`);
3. replace an update by a self-move or change one move's `from`/`to`: the exact effect and ER
   decision assertion fails (`wrong-transition`);
4. delete one operation-field coordinate, silently map it to preserve, permit removal of a required
   field, or invoke a policy before selection: completeness, action, and ordering assertions fail;
5. call the issue-time provider twice or bind the badge action to a value other than the event's
   normalized command field: shared-value assertions fail;
6. load before `decide_before_load`, continue with another subject, patch after decision, or append
   a refused payment: the payment matrix and counting lookup/fulfillment/append controls fail;
7. drop `removed` or an action from a record, frame `service/3` as `/3`, or change an existing
   `/1` through `/3` vector: replay or compatibility assertions fail.

The bounded implementors run their focused tests, strict crate Clippy, formatting, and Rust 1.85
dependency checks. Root later owns full gates, consumer accounting, site validation, source reviews,
and integration. No Cargo command belongs to this design correction.
