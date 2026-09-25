//! Pure lowering from an admitted ESS service contract to Entity Runtime data.
//!
//! This crate performs no IO and chooses no host-owned value or policy. A successful projection
//! contains validated runtime definitions plus the complete typed binding obligations that remain
//! for a host. Unsupported source shapes are returned together in deterministic semantic order.

#![allow(missing_docs)]

use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroU32;

use entity_core::{
    Cardinality as RuntimeCardinality, CompareOp as RuntimeCompareOp, Comparison, Condition,
    CreateDefinition, EntityDefinition, EventDefinition, FieldDefinition, FieldKind,
    IdentityDefinition, LifecycleDefinition, MapKey, NumberObservation, ObjectSchema, OneOrMany,
    OperationDefinition, OperationFieldActions as RuntimeOperationFieldActions,
    OperationFieldRequirement, OutcomeDefinition, OutcomeEffect, PresentArgument, Quantifier,
    RefusalDefinition, Registry, RelationDefinition, RelationKind as RuntimeRelationKind,
    RuleDefinition, Semantics, ValidatedDefinition,
};
use ess_compiler::ir::{
    EssIr, ResolvedBody, ResolvedCommand, ResolvedCondition, ResolvedEffect, ResolvedEntity,
    ResolvedField, ResolvedInstance, ResolvedOutcome, ResolvedPayloadField, ResolvedPayloadValue,
    ResolvedTypeRef,
};
use ess_domain::command::OutcomeName;
use ess_domain::component::ComponentName;
use ess_domain::entity::{Cardinality, RelationKind};
use ess_domain::name::QualifiedName;
use ess_domain::types::Primitive;
use ess_primitives::facts::{FactPath, FactValue};
use ess_primitives::predicate::{CompareOp, Operand, Predicate, Quantified};
use ess_service_contract::ServiceIr;
use ess_synth::PlannedCapability;
use serde_json::{Map, Number, Value};
use sha2::{Digest, Sha256};

/// The exact Entity Runtime source revision this projector targets.
pub const ENTITY_RUNTIME_REVISION: &str = "9ee145e888368c668cdaaaa9fa22b8334149616f";

/// Caller-owned coordinates which ESS does not encode itself.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LoweringOptions {
    pub definition_versions: BTreeMap<QualifiedName, NonZeroU32>,
    pub scales: BTreeMap<QualifiedName, BTreeMap<String, Vec<String>>>,
}

/// One complete, validated service projection.
#[derive(Debug, Clone, PartialEq)]
pub struct LoweredService {
    component: ComponentName,
    source_digest: String,
    synthesis_digest: String,
    target_revision: String,
    definitions: BTreeMap<QualifiedName, ValidatedDefinition>,
    bindings: BindingPlan,
}

impl LoweredService {
    pub fn component(&self) -> &ComponentName {
        &self.component
    }
    pub fn source_digest(&self) -> &str {
        &self.source_digest
    }
    pub fn synthesis_digest(&self) -> &str {
        &self.synthesis_digest
    }
    pub fn target_revision(&self) -> &str {
        &self.target_revision
    }
    pub fn definitions(&self) -> &BTreeMap<QualifiedName, ValidatedDefinition> {
        &self.definitions
    }
    pub fn bindings(&self) -> &BindingPlan {
        &self.bindings
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindingPlan {
    commands: BTreeMap<QualifiedName, CommandBinding>,
    requirements: Vec<BindingRequirement>,
    source_capabilities: Vec<PlannedCapability>,
}

impl BindingPlan {
    pub fn commands(&self) -> &BTreeMap<QualifiedName, CommandBinding> {
        &self.commands
    }
    pub fn requirements(&self) -> &[BindingRequirement] {
        &self.requirements
    }
    pub fn source_capabilities(&self) -> &[PlannedCapability] {
        &self.source_capabilities
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandBinding {
    pub target: DefinitionCoordinate,
    pub entrypoint: RuntimeEntrypoint,
    pub instance: InstanceBinding,
    pub slots: BTreeMap<BindingSlot, BoundValue>,
    pub operation_fields: BTreeMap<OperationFieldCoordinate, OperationFieldFulfillment>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefinitionCoordinate {
    pub entity: QualifiedName,
    pub version: NonZeroU32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeEntrypoint {
    Create,
    Operation { name: QualifiedName },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstanceBinding {
    Created {
        logical_identity: IdentityValue,
        observed_at: EventFieldCoordinate,
    },
    SelectedOutcome {
        identities: BTreeMap<OutcomeName, SelectedCreationIdentity>,
    },
    Supplied {
        input_field: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectedCreationIdentity {
    pub logical_identity: IdentityValue,
    pub observed_at: EventFieldCoordinate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdentityValue {
    InputField { field: String },
    Literal { value: Value },
    Bound { slot: BindingSlot },
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct EventFieldCoordinate {
    pub outcome: OutcomeName,
    pub occurrence: usize,
    pub event: QualifiedName,
    pub field: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BindingSlot(u32);

impl BindingSlot {
    pub const fn index(self) -> u32 {
        self.0
    }
    fn name(self) -> String {
        format!("b{:08}", self.0)
    }
    fn argument(self) -> String {
        format!("bound.{}", self.name())
    }
    fn template(self) -> Value {
        Value::String(format!("$args.{}", self.argument()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundValue {
    pub target: BoundTarget,
    pub type_ref: ResolvedTypeRef,
    pub source: BoundSource,
    pub presence: BoundPresence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoundPresence {
    Required,
    Optional,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct OperationFieldCoordinate {
    pub outcome: OutcomeName,
    pub field: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperationFieldFulfillment {
    pub type_ref: ResolvedTypeRef,
    pub actions: OperationFieldActions,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationFieldActions {
    Required,
    Optional,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoundTarget {
    ExternalEvidence {
        outcome: OutcomeName,
    },
    LogicalIdentity {
        at: EventFieldCoordinate,
    },
    EntityField {
        outcome: OutcomeName,
        field: String,
    },
    EventField {
        outcome: OutcomeName,
        occurrence: usize,
        event: QualifiedName,
        field: String,
    },
    ResponseField {
        outcome: OutcomeName,
        field: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoundSource {
    Identity,
    ResponseField {
        field: String,
    },
    Generated,
    Undetermined,
    Conversion {
        from: ResolvedTypeRef,
        because: String,
    },
    External {
        cause: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemanticLocation {
    EntityIdentity {
        entity: QualifiedName,
        field: String,
    },
    EntityField {
        entity: QualifiedName,
        field: String,
    },
    CommandInput {
        command: QualifiedName,
        field: String,
    },
    CommandResponse {
        command: QualifiedName,
        field: String,
    },
    EventField {
        event: QualifiedName,
        field: String,
    },
    BindingTarget {
        command: QualifiedName,
        target: BoundTarget,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationCoordinate {
    pub source: QualifiedName,
    pub relation: String,
    pub target: QualifiedName,
    pub via: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BindingRequirement {
    IdentitySupplied {
        command: QualifiedName,
        value: IdentityValue,
        observed_at: EventFieldCoordinate,
    },
    ResponseFieldSupplied {
        command: QualifiedName,
        outcome: OutcomeName,
        field: String,
        slot: BindingSlot,
    },
    GeneratedFieldSupplied {
        command: QualifiedName,
        target: BoundTarget,
        slot: BindingSlot,
    },
    UndeterminedFieldSupplied {
        command: QualifiedName,
        target: BoundTarget,
        slot: BindingSlot,
    },
    ConversionSupplied {
        command: QualifiedName,
        target: BoundTarget,
        slot: BindingSlot,
        from: ResolvedTypeRef,
        to: ResolvedTypeRef,
        because: String,
    },
    ExternalEvidenceSupplied {
        command: QualifiedName,
        outcome: OutcomeName,
        cause: String,
        slot: BindingSlot,
    },
    OperationFieldPolicySupplied {
        command: QualifiedName,
        target: OperationFieldCoordinate,
        fulfillment: OperationFieldFulfillment,
    },
    TimestampSpelling {
        primitive: Primitive,
        at: SemanticLocation,
    },
    UuidSpelling {
        at: SemanticLocation,
    },
    BytesSpelling {
        at: SemanticLocation,
    },
    DecimalSpelling {
        at: SemanticLocation,
    },
    Binary64Spelling {
        at: SemanticLocation,
    },
    ScaleDeclaration {
        entity: QualifiedName,
        name: String,
        values: Vec<String>,
    },
    RelationExistence {
        relation: RelationCoordinate,
    },
    RelationCardinality {
        relation: RelationCoordinate,
    },
    RelationOwnership {
        relation: RelationCoordinate,
    },
    ErrorPayload {
        command: QualifiedName,
        outcome: OutcomeName,
        error: QualifiedName,
        fields: Vec<ResolvedField>,
    },
    RevisionExpectation {
        command: QualifiedName,
        entity: QualifiedName,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoweringDiagnostics(Vec<LoweringDiagnostic>);

impl LoweringDiagnostics {
    pub fn as_slice(&self) -> &[LoweringDiagnostic] {
        &self.0
    }
    pub fn into_vec(self) -> Vec<LoweringDiagnostic> {
        self.0
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoweringDiagnostic {
    pub code: LoweringCode,
    pub path: String,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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

/// Projects one admitted component-scoped service contract.
pub fn lower(
    service: &ServiceIr<'_>,
    options: &LoweringOptions,
) -> Result<LoweredService, LoweringDiagnostics> {
    let first = lower_once(service, options);
    let second = lower_once(service, options);
    if first == second {
        if let (Ok(first), Ok(second)) = (&first, &second) {
            let first_bytes = canonical_definition_bytes(first);
            let second_bytes = canonical_definition_bytes(second);
            if first_bytes != second_bytes {
                return Err(LoweringDiagnostics(vec![LoweringDiagnostic {
                    code: LoweringCode::TargetDefinitionRefused,
                    path: "determinism.definitions".to_owned(),
                    message: "repeated lowering produced different canonical definition bytes"
                        .to_owned(),
                }]));
            }
        }
        return first;
    }
    Err(LoweringDiagnostics(vec![LoweringDiagnostic {
        code: LoweringCode::TargetDefinitionRefused,
        path: "determinism".to_owned(),
        message: "repeated lowering produced different typed output or diagnostic order".to_owned(),
    }]))
}

fn lower_once(
    service: &ServiceIr<'_>,
    options: &LoweringOptions,
) -> Result<LoweredService, LoweringDiagnostics> {
    let closure = entity_closure(service);
    let mut projector = Projector {
        service,
        options,
        closure,
        definitions: BTreeMap::new(),
        commands: BTreeMap::new(),
        requirements: Vec::new(),
        diagnostics: Vec::new(),
    };
    projector.check_options();
    projector.build_definitions();
    projector.build_commands();
    projector.finish()
}

fn canonical_definition_bytes(service: &LoweredService) -> Vec<(QualifiedName, Vec<u8>)> {
    service
        .definitions
        .iter()
        .map(|(name, definition)| {
            (
                name.clone(),
                serde_json::to_vec(definition.as_definition())
                    .expect("a validated Entity Runtime definition serializes"),
            )
        })
        .collect()
}

struct Projector<'a> {
    service: &'a ServiceIr<'a>,
    options: &'a LoweringOptions,
    closure: BTreeSet<QualifiedName>,
    definitions: BTreeMap<QualifiedName, EntityDefinition>,
    commands: BTreeMap<QualifiedName, CommandBinding>,
    requirements: Vec<BindingRequirement>,
    diagnostics: Vec<LoweringDiagnostic>,
}

struct CreationObservation {
    binding: EventFieldCoordinate,
    by_outcome: BTreeMap<OutcomeName, CreationOutcomeIdentity>,
}

struct CreationOutcomeIdentity {
    binding: EventFieldCoordinate,
    mapping: Option<ResolvedPayloadField>,
}

impl Projector<'_> {
    fn diagnostic(
        &mut self,
        code: LoweringCode,
        path: impl Into<String>,
        message: impl Into<String>,
    ) {
        self.diagnostics.push(LoweringDiagnostic {
            code,
            path: path.into(),
            message: message.into(),
        });
    }

    fn check_options(&mut self) {
        for entity in &self.closure.clone() {
            if !self.options.definition_versions.contains_key(entity) {
                self.diagnostic(
                    LoweringCode::MissingDefinitionVersion,
                    entity.to_string(),
                    "the selected entity closure requires an explicit nonzero Entity Runtime definition version",
                );
            }
        }
        for entity in self.options.definition_versions.keys() {
            if !self.closure.contains(entity) {
                self.diagnostic(
                    LoweringCode::UnknownDefinitionVersion,
                    entity.to_string(),
                    "a definition version was supplied for an entity outside the selected closure",
                );
            }
        }
        for entity in self.options.scales.keys() {
            if !self.closure.contains(entity) {
                self.diagnostic(
                    LoweringCode::UnknownScaleEntity,
                    entity.to_string(),
                    "scale declarations are accepted only for entities in the selected closure",
                );
            }
        }
    }

    #[allow(clippy::too_many_lines)]
    fn build_definitions(&mut self) {
        let names = self.closure.iter().cloned().collect::<Vec<_>>();
        for name in names {
            let entity = self
                .service
                .source()
                .entities()
                .get(&name)
                .expect("closure names a resolved entity");
            let Some(version) = self.options.definition_versions.get(&name).copied() else {
                continue;
            };
            let mut fields = BTreeMap::new();
            let identity_location = SemanticLocation::EntityIdentity {
                entity: name.clone(),
                field: entity.identity.name.clone(),
            };
            fields.insert(
                entity.identity.name.clone(),
                self.lower_identity_field(
                    &entity.identity.type_ref,
                    &identity_location,
                    &format!("{name}.identity"),
                ),
            );
            for field in &entity.fields {
                let location = SemanticLocation::EntityField {
                    entity: name.clone(),
                    field: field.name.clone(),
                };
                fields.insert(
                    field.name.clone(),
                    self.lower_field(
                        &field.type_ref,
                        !field.type_ref.is_optional(),
                        &location,
                        &format!("{}.fields.{}", name, field.name),
                    ),
                );
            }

            let mut invariants = Vec::new();
            for (index, invariant) in entity.invariants.iter().enumerate() {
                invariants.push(RuleDefinition {
                    name: Some(format!("entity_{index:04}")),
                    condition: lower_predicate(&invariant.predicate, &PathRewrite::Entity),
                    message: Some(invariant.statement.clone()),
                });
            }
            self.lower_nominal_invariants(
                &entity.identity.type_ref,
                &format!("$fields.{}", entity.identity.name),
                false,
                &format!("{}.fields.{}", name, entity.identity.name),
                &mut Vec::new(),
                &mut invariants,
            );
            for field in &entity.fields {
                self.lower_nominal_invariants(
                    &field.type_ref,
                    &format!("$fields.{}", field.name),
                    field.type_ref.is_optional(),
                    &format!("{}.fields.{}", name, field.name),
                    &mut Vec::new(),
                    &mut invariants,
                );
            }

            let mut relations = BTreeMap::new();
            for relation in &entity.relations {
                let target = relation.target.name().clone();
                let coordinate = RelationCoordinate {
                    source: name.clone(),
                    relation: relation.name.clone(),
                    target: target.clone(),
                    via: relation.via.clone(),
                };
                relations.insert(
                    relation.name.clone(),
                    RelationDefinition {
                        kind: match relation.kind {
                            RelationKind::Owns => RuntimeRelationKind::Owns,
                            RelationKind::References => RuntimeRelationKind::References,
                        },
                        target: target.to_string(),
                        cardinality: match relation.cardinality {
                            Cardinality::One => RuntimeCardinality::One,
                            Cardinality::Many => RuntimeCardinality::Many,
                        },
                        via: relation.via.clone(),
                    },
                );
                self.requirements
                    .push(BindingRequirement::RelationExistence {
                        relation: coordinate.clone(),
                    });
                self.requirements
                    .push(BindingRequirement::RelationCardinality {
                        relation: coordinate.clone(),
                    });
                self.requirements
                    .push(BindingRequirement::RelationOwnership {
                        relation: coordinate,
                    });
            }
            let scales = self.options.scales.get(&name).cloned().unwrap_or_default();
            for (scale, values) in &scales {
                self.requirements
                    .push(BindingRequirement::ScaleDeclaration {
                        entity: name.clone(),
                        name: scale.clone(),
                        values: values.clone(),
                    });
            }
            self.definitions.insert(
                name.clone(),
                EntityDefinition {
                    entity: name.to_string(),
                    version: version.get(),
                    schema: ObjectSchema {
                        fields,
                        additional_fields: false,
                    },
                    lifecycle: LifecycleDefinition {
                        initial: entity.lifecycle.initial.to_string(),
                        states: entity
                            .lifecycle
                            .states
                            .iter()
                            .map(ToString::to_string)
                            .collect(),
                    },
                    invariants,
                    create: CreateDefinition::default(),
                    operations: BTreeMap::new(),
                    projections: BTreeMap::new(),
                    semantics: Semantics::Service1,
                    identity: Some(IdentityDefinition {
                        field: entity.identity.name.clone(),
                    }),
                    relations,
                    scales,
                    number_observation: NumberObservation::SourceNumber1,
                },
            );
        }
    }

    fn lower_field(
        &mut self,
        type_ref: &ResolvedTypeRef,
        required: bool,
        location: &SemanticLocation,
        path: &str,
    ) -> FieldDefinition {
        self.lower_field_inner(type_ref, required, location, path, true, &mut Vec::new())
    }

    fn lower_identity_field(
        &mut self,
        type_ref: &ResolvedTypeRef,
        location: &SemanticLocation,
        path: &str,
    ) -> FieldDefinition {
        let representation = match type_ref {
            ResolvedTypeRef::Optional { of } => of.as_ref(),
            other => other,
        };
        self.lower_field_inner(representation, true, location, path, true, &mut Vec::new())
    }

    #[allow(clippy::too_many_lines)]
    fn lower_field_inner(
        &mut self,
        type_ref: &ResolvedTypeRef,
        required: bool,
        location: &SemanticLocation,
        path: &str,
        member_position: bool,
        active: &mut Vec<QualifiedName>,
    ) -> FieldDefinition {
        match type_ref {
            ResolvedTypeRef::Optional { of } => {
                if !member_position {
                    self.diagnostic(
                        LoweringCode::NullableElementUnsupported,
                        path,
                        "Optional below a list or map value denotes JSON null, which Entity Runtime field definitions cannot represent",
                    );
                }
                self.lower_field_inner(of, false, location, path, member_position, active)
            }
            ResolvedTypeRef::Primitive { name } => {
                self.spelling_requirement(*name, location.clone());
                FieldDefinition {
                    kind: primitive_kind(*name),
                    required,
                    ..FieldDefinition::default()
                }
            }
            ResolvedTypeRef::List { of } => FieldDefinition {
                kind: FieldKind::Array,
                required,
                items: Some(Box::new(self.lower_field_inner(
                    of,
                    true,
                    location,
                    &format!("{path}.items"),
                    false,
                    active,
                ))),
                ..FieldDefinition::default()
            },
            ResolvedTypeRef::Map { key, value } => FieldDefinition {
                kind: FieldKind::Map,
                required,
                key: Some(map_key(*key)),
                items: Some(Box::new(self.lower_field_inner(
                    value,
                    true,
                    location,
                    &format!("{path}.values"),
                    false,
                    active,
                ))),
                ..FieldDefinition::default()
            },
            ResolvedTypeRef::Declared { name } => {
                let qualified = name.name().clone();
                if active.contains(&qualified) {
                    self.diagnostic(
                        LoweringCode::RecursiveTypeUnsupported,
                        path,
                        format!("resolved type `{qualified}` recursively reaches itself"),
                    );
                    return FieldDefinition {
                        kind: FieldKind::String,
                        required,
                        ..FieldDefinition::default()
                    };
                }
                active.push(qualified.clone());
                let resolved = self.service.source().named_type(name);
                let field = match &resolved.body {
                    ResolvedBody::Newtype { of, .. } => self.lower_field_inner(
                        of,
                        required,
                        location,
                        path,
                        member_position,
                        active,
                    ),
                    ResolvedBody::Struct { fields, .. } => {
                        let mut properties = BTreeMap::new();
                        for field in fields {
                            properties.insert(
                                field.name.clone(),
                                self.lower_field_inner(
                                    &field.type_ref,
                                    !field.type_ref.is_optional(),
                                    location,
                                    &format!("{path}.{}", field.name),
                                    true,
                                    active,
                                ),
                            );
                        }
                        FieldDefinition {
                            kind: FieldKind::Object,
                            required,
                            properties,
                            additional_properties: false,
                            ..FieldDefinition::default()
                        }
                    }
                    ResolvedBody::Enum { variants } => FieldDefinition {
                        kind: FieldKind::Enum,
                        required,
                        values: variants
                            .iter()
                            .map(|variant| variant.name.clone())
                            .collect(),
                        ..FieldDefinition::default()
                    },
                    ResolvedBody::Union { tag, variants } => {
                        let mut lowered = BTreeMap::new();
                        for (variant, shape) in variants {
                            lowered.insert(
                                variant.clone(),
                                self.lower_field_inner(
                                    shape,
                                    true,
                                    location,
                                    &format!("{path}.{variant}"),
                                    true,
                                    active,
                                ),
                            );
                        }
                        FieldDefinition {
                            kind: FieldKind::Union,
                            required,
                            tag: Some(tag.clone()),
                            variants: lowered,
                            ..FieldDefinition::default()
                        }
                    }
                };
                active.pop();
                field
            }
        }
    }

    fn spelling_requirement(&mut self, primitive: Primitive, at: SemanticLocation) {
        let requirement = match primitive {
            Primitive::Timestamp | Primitive::Duration => {
                BindingRequirement::TimestampSpelling { primitive, at }
            }
            Primitive::Uuid => BindingRequirement::UuidSpelling { at },
            Primitive::Bytes => BindingRequirement::BytesSpelling { at },
            Primitive::Decimal => BindingRequirement::DecimalSpelling { at },
            Primitive::Binary64 => BindingRequirement::Binary64Spelling { at },
            Primitive::String | Primitive::Boolean | Primitive::Integer => return,
        };
        self.requirements.push(requirement);
    }

    #[allow(clippy::too_many_lines)]
    fn lower_nominal_invariants(
        &mut self,
        type_ref: &ResolvedTypeRef,
        base: &str,
        optional: bool,
        semantic_path: &str,
        active: &mut Vec<QualifiedName>,
        out: &mut Vec<RuleDefinition>,
    ) {
        match type_ref {
            ResolvedTypeRef::Optional { of } => {
                self.lower_nominal_invariants(of, base, true, semantic_path, active, out);
            }
            ResolvedTypeRef::List { of } | ResolvedTypeRef::Map { value: of, .. } => {
                let binder = format!("item{:04}", out.len());
                let before = out.len();
                self.lower_nominal_invariants(
                    of,
                    &format!("${binder}"),
                    false,
                    semantic_path,
                    active,
                    out,
                );
                if out.len() > before {
                    let added = out.split_off(before);
                    for mut rule in added {
                        rule.condition = Condition::ForAll {
                            for_all: Box::new(Quantifier {
                                over: Value::String(base.to_owned()),
                                bind: binder.clone(),
                                body: Box::new(rule.condition),
                            }),
                        };
                        if optional {
                            rule.condition = optional_guard(base, rule.condition);
                        }
                        out.push(rule);
                    }
                }
            }
            ResolvedTypeRef::Declared { name } => {
                let qualified = name.name().clone();
                if active.contains(&qualified) {
                    return;
                }
                active.push(qualified);
                let resolved = self.service.source().named_type(name);
                match &resolved.body {
                    ResolvedBody::Newtype { of, invariants } => {
                        for (index, invariant) in invariants.iter().enumerate() {
                            let mut condition = lower_predicate(
                                &invariant.predicate,
                                &PathRewrite::Nominal {
                                    base: base.to_owned(),
                                    value_root: true,
                                },
                            );
                            if optional {
                                condition = optional_guard(base, condition);
                            }
                            out.push(RuleDefinition {
                                name: Some(format!(
                                    "nominal_{}_{index:04}",
                                    sanitize(semantic_path)
                                )),
                                condition,
                                message: Some(invariant.statement.clone()),
                            });
                        }
                        self.lower_nominal_invariants(
                            of,
                            base,
                            optional,
                            semantic_path,
                            active,
                            out,
                        );
                    }
                    ResolvedBody::Struct { fields, invariants } => {
                        for (index, invariant) in invariants.iter().enumerate() {
                            let mut condition = lower_predicate(
                                &invariant.predicate,
                                &PathRewrite::Nominal {
                                    base: base.to_owned(),
                                    value_root: false,
                                },
                            );
                            if optional {
                                condition = optional_guard(base, condition);
                            }
                            out.push(RuleDefinition {
                                name: Some(format!(
                                    "nominal_{}_{index:04}",
                                    sanitize(semantic_path)
                                )),
                                condition,
                                message: Some(invariant.statement.clone()),
                            });
                        }
                        for field in fields {
                            self.lower_nominal_invariants(
                                &field.type_ref,
                                &format!("{base}.{}", field.name),
                                optional || field.type_ref.is_optional(),
                                &format!("{semantic_path}.{}", field.name),
                                active,
                                out,
                            );
                        }
                    }
                    ResolvedBody::Union { tag, variants } => {
                        let content = if tag == "value" { "content" } else { "value" };
                        for (variant, shape) in variants {
                            let before = out.len();
                            self.lower_nominal_invariants(
                                shape,
                                &format!("{base}.{content}"),
                                false,
                                &format!("{semantic_path}.{variant}"),
                                active,
                                out,
                            );
                            if out.len() > before {
                                let added = out.split_off(before);
                                for mut rule in added {
                                    rule.condition = Condition::Any {
                                        any: vec![
                                            Condition::Ne {
                                                ne: [
                                                    Value::String(format!("{base}.{tag}")),
                                                    Value::String(variant.clone()),
                                                ],
                                            },
                                            rule.condition,
                                        ],
                                    };
                                    if optional {
                                        rule.condition = optional_guard(base, rule.condition);
                                    }
                                    out.push(rule);
                                }
                            }
                        }
                    }
                    ResolvedBody::Enum { .. } => {}
                }
                active.pop();
            }
            ResolvedTypeRef::Primitive { .. } => {}
        }
    }

    fn build_commands(&mut self) {
        let commands = self
            .service
            .operations()
            .values()
            .copied()
            .collect::<Vec<_>>();
        for command in commands {
            self.build_command(command);
        }
    }

    #[allow(clippy::too_many_lines)]
    fn build_command(&mut self, command: &ResolvedCommand) {
        let command_path = command.name.to_string();
        let mut targets = BTreeSet::new();
        for outcome in &command.outcomes {
            if let Some(subject) = &outcome.subject {
                targets.insert(subject.entity.name().clone());
            } else if outcome.error.is_none()
                && !matches!(outcome.condition, ResolvedCondition::WrongState)
            {
                self.diagnostic(
                    LoweringCode::AcceptingOutcomeWithoutSubject,
                    format!("{}.{}", command.name, outcome.name.as_str()),
                    "an accepting ordinary outcome has no entity subject",
                );
            }
        }
        if targets.is_empty() {
            self.diagnostic(
                LoweringCode::StatelessCommandUnsupported,
                &command_path,
                "the selected command has no subject-bearing outcome and Entity Runtime has no stateless decision type",
            );
            return;
        }
        if targets.len() != 1 {
            self.diagnostic(
                LoweringCode::CommandSpansEntities,
                &command_path,
                "one Entity Runtime operation cannot select outcomes for more than one entity",
            );
            return;
        }
        let entity_name = targets.into_iter().next().expect("one target");
        if !self.definitions.contains_key(&entity_name) {
            return;
        }
        let entity = self
            .service
            .source()
            .entities()
            .get(&entity_name)
            .expect("target is resolved");

        let effects = command
            .outcomes
            .iter()
            .filter_map(|outcome| outcome.subject.as_ref().map(|subject| &subject.effect))
            .collect::<Vec<_>>();
        let has_create = effects
            .iter()
            .any(|effect| matches!(effect, ResolvedEffect::Creates));
        let has_operation = effects
            .iter()
            .any(|effect| !matches!(effect, ResolvedEffect::Creates));
        if has_create && has_operation {
            self.diagnostic(
                LoweringCode::MixedEntrypointUnsupported,
                &command_path,
                "creation and existing-instance effects require different Entity Runtime entrypoints",
            );
            return;
        }
        let is_create = has_create;

        let observed = if is_create {
            self.creation_coordinate(command)
        } else {
            None
        };
        let supplied = if is_create {
            None
        } else {
            self.supplied_identity(command)
        };
        if (is_create && observed.is_none()) || (!is_create && supplied.is_none()) {
            return;
        }

        let mut slots = SlotBook::default();
        let shared_creation_identity = observed.as_ref().is_some_and(|observation| {
            let mut mappings = observation
                .by_outcome
                .values()
                .map(|identity| &identity.mapping);
            mappings
                .next()
                .is_some_and(|first| mappings.all(|mapping| mapping == first))
        });
        let mut identities = BTreeMap::new();
        if let Some(observation) = &observed {
            if shared_creation_identity {
                let identity =
                    self.creation_identity(command, entity, &observation.binding, &mut slots);
                for outcome in observation.by_outcome.keys() {
                    identities.insert(outcome.clone(), identity.clone());
                }
            } else {
                for (outcome, observation) in &observation.by_outcome {
                    let identity =
                        self.creation_identity(command, entity, &observation.binding, &mut slots);
                    identities.insert(outcome.clone(), identity);
                }
            }
        }

        let input_schema = self.schema_for_fields(
            &command.input,
            |field| SemanticLocation::CommandInput {
                command: command.name.clone(),
                field: field.to_owned(),
            },
            &format!("{command_path}.input"),
        );
        let response_schema = self.schema_for_fields(
            &command.response,
            |field| SemanticLocation::CommandResponse {
                command: command.name.clone(),
                field: field.to_owned(),
            },
            &format!("{command_path}.response"),
        );

        let mut compiled = Vec::new();
        let mut operation_fields = BTreeMap::new();
        for (source_index, outcome) in command.outcomes.iter().enumerate() {
            let lowered = self.lower_outcome(
                command,
                entity,
                outcome,
                source_index,
                is_create,
                observed
                    .as_ref()
                    .and_then(|observation| observation.by_outcome.get(&outcome.name))
                    .map(|identity| &identity.binding),
                identities.get(&outcome.name),
                &mut slots,
                &mut operation_fields,
            );
            compiled.push((source_index, lowered));
        }
        let outcome_indices = command
            .outcomes
            .iter()
            .enumerate()
            .map(|(index, outcome)| (outcome.name.clone(), index))
            .collect::<BTreeMap<_, _>>();
        let remap = slots.canonicalize(&outcome_indices);
        for identity in identities.values_mut() {
            if let IdentityValue::Bound { slot } = identity {
                *slot = remap[slot];
            }
        }
        for (_, outcome) in &mut compiled {
            remap_outcome_slots(outcome, &remap, &slots.references);
        }
        compiled.sort_by_key(|(index, outcome)| {
            let category = if outcome.wrong_state {
                2
            } else {
                usize::from(outcome.is_default_branch())
            };
            (category, *index)
        });
        let outcomes = compiled
            .into_iter()
            .map(|(_, outcome)| outcome)
            .collect::<Vec<_>>();

        let mut bound_fields = BTreeMap::new();
        for (slot, value) in &slots.slots {
            let location = SemanticLocation::BindingTarget {
                command: command.name.clone(),
                target: value.target.clone(),
            };
            let required = value.presence == BoundPresence::Required;
            bound_fields.insert(
                slot.name(),
                self.lower_field(
                    &value.type_ref,
                    required,
                    &location,
                    &format!("{command_path}.bound.{}", slot.name()),
                ),
            );
        }
        let arguments = ObjectSchema {
            fields: BTreeMap::from([
                (
                    "bound".to_owned(),
                    FieldDefinition {
                        kind: FieldKind::Object,
                        required: true,
                        properties: bound_fields,
                        additional_properties: false,
                        ..FieldDefinition::default()
                    },
                ),
                (
                    "input".to_owned(),
                    FieldDefinition {
                        kind: FieldKind::Object,
                        required: true,
                        properties: input_schema.fields,
                        additional_properties: false,
                        ..FieldDefinition::default()
                    },
                ),
            ]),
            additional_fields: false,
        };

        self.requirements.extend(slots.requirements);
        if !is_create {
            self.requirements
                .push(BindingRequirement::RevisionExpectation {
                    command: command.name.clone(),
                    entity: entity_name.clone(),
                });
        }
        let version = self.options.definition_versions[&entity_name];
        let binding = CommandBinding {
            target: DefinitionCoordinate {
                entity: entity_name.clone(),
                version,
            },
            entrypoint: if is_create {
                RuntimeEntrypoint::Create
            } else {
                RuntimeEntrypoint::Operation {
                    name: command.name.clone(),
                }
            },
            instance: if let Some(observation) = observed.as_ref() {
                if shared_creation_identity {
                    InstanceBinding::Created {
                        logical_identity: identities[&observation.binding.outcome].clone(),
                        observed_at: observation.binding.clone(),
                    }
                } else {
                    InstanceBinding::SelectedOutcome {
                        identities: observation
                            .by_outcome
                            .iter()
                            .map(|(outcome, observation)| {
                                (
                                    outcome.clone(),
                                    SelectedCreationIdentity {
                                        logical_identity: identities[outcome].clone(),
                                        observed_at: observation.binding.clone(),
                                    },
                                )
                            })
                            .collect(),
                    }
                }
            } else {
                InstanceBinding::Supplied {
                    input_field: supplied.expect("checked above"),
                }
            },
            slots: slots.slots,
            operation_fields,
        };
        self.commands.insert(command.name.clone(), binding);

        let definition = self
            .definitions
            .get_mut(&entity_name)
            .expect("target definition");
        if is_create {
            if !definition.create.outcomes.is_empty() {
                self.diagnostic(
                    LoweringCode::MultipleCreationCommands,
                    entity_name.to_string(),
                    "Entity Runtime has one unnamed creation entrypoint for an entity",
                );
                return;
            }
            definition.create = CreateDefinition {
                emit: None,
                arguments,
                response: response_schema,
                outcomes,
            };
        } else {
            definition.operations.insert(
                command.name.to_string(),
                OperationDefinition {
                    arguments,
                    transitions: Vec::new(),
                    preconditions: Vec::new(),
                    set: BTreeMap::new(),
                    emits: Vec::new(),
                    response: response_schema,
                    outcomes,
                },
            );
        }
    }

    fn schema_for_fields(
        &mut self,
        fields: &[ResolvedField],
        location: impl Fn(&str) -> SemanticLocation,
        path: &str,
    ) -> ObjectSchema {
        let mut lowered = BTreeMap::new();
        for field in fields {
            lowered.insert(
                field.name.clone(),
                self.lower_field(
                    &field.type_ref,
                    !field.type_ref.is_optional(),
                    &location(&field.name),
                    &format!("{path}.{}", field.name),
                ),
            );
        }
        ObjectSchema {
            fields: lowered,
            additional_fields: false,
        }
    }

    fn creation_coordinate(&mut self, command: &ResolvedCommand) -> Option<CreationObservation> {
        let mut binding = None;
        let mut by_outcome = BTreeMap::new();
        for outcome in &command.outcomes {
            if outcome.error.is_some() {
                continue;
            }
            let Some(subject) = &outcome.subject else {
                continue;
            };
            let ResolvedInstance::Observed { event, field } = &subject.instance else {
                self.diagnostic(
                    LoweringCode::AmbiguousInstanceBinding,
                    command.name.to_string(),
                    "every accepting creation outcome must observe its logical identity in an emitted event",
                );
                return None;
            };
            let occurrence = outcome
                .emits
                .iter()
                .position(|candidate| candidate == event)
                .expect("resolved observed event is emitted");
            let coordinate = EventFieldCoordinate {
                outcome: outcome.name.clone(),
                occurrence,
                event: event.name().clone(),
                field: field.name.clone(),
            };
            if binding.is_none() {
                binding = Some(coordinate.clone());
            }
            by_outcome.insert(
                outcome.name.clone(),
                CreationOutcomeIdentity {
                    mapping: payload_mapping(outcome, &coordinate.event, &coordinate.field)
                        .cloned(),
                    binding: coordinate,
                },
            );
        }
        Some(CreationObservation {
            binding: binding.expect("a creation command has an accepting subject"),
            by_outcome,
        })
    }

    fn supplied_identity(&mut self, command: &ResolvedCommand) -> Option<String> {
        let mut supplied = None::<String>;
        for subject in command
            .outcomes
            .iter()
            .filter_map(|outcome| outcome.subject.as_ref())
        {
            let ResolvedInstance::Supplied { field } = &subject.instance else {
                self.diagnostic(
                    LoweringCode::AmbiguousInstanceBinding,
                    command.name.to_string(),
                    "an existing-instance outcome must name an input identity field",
                );
                return None;
            };
            if supplied
                .as_deref()
                .is_some_and(|existing| existing != field.name)
            {
                self.diagnostic(
                    LoweringCode::AmbiguousInstanceBinding,
                    command.name.to_string(),
                    "existing-instance outcomes name different input identity fields",
                );
                return None;
            }
            supplied = Some(field.name.clone());
        }
        supplied
    }

    #[allow(clippy::too_many_lines)]
    fn creation_identity(
        &mut self,
        command: &ResolvedCommand,
        entity: &ResolvedEntity,
        at: &EventFieldCoordinate,
        slots: &mut SlotBook,
    ) -> IdentityValue {
        let outcome = command
            .outcomes
            .iter()
            .find(|outcome| outcome.name == at.outcome)
            .expect("coordinate outcome");
        let mapping = payload_mapping(outcome, &at.event, &at.field);
        let value = match mapping {
            Some(ResolvedPayloadField {
                value: ResolvedPayloadValue::InputField { field, .. },
                conversion: None,
                ..
            }) => IdentityValue::InputField {
                field: field.clone(),
            },
            Some(ResolvedPayloadField {
                value: ResolvedPayloadValue::Literal { value },
                target_type,
                ..
            }) => {
                match self.literal(
                    target_type,
                    value,
                    &format!("{}.{}.identity", command.name, outcome.name.as_str()),
                ) {
                    Some(value) => IdentityValue::Literal { value },
                    None => IdentityValue::Literal { value: Value::Null },
                }
            }
            Some(ResolvedPayloadField {
                value: ResolvedPayloadValue::Cleared,
                ..
            }) => {
                self.diagnostic(
                    LoweringCode::ClearedValueUnsupported,
                    format!("{}.{}.identity", command.name, outcome.name.as_str()),
                    "a logical identity cannot be cleared",
                );
                IdentityValue::Literal { value: Value::Null }
            }
            mapping => {
                let target = BoundTarget::LogicalIdentity { at: at.clone() };
                let (source, requirement_kind) = match mapping {
                    None => (BoundSource::Identity, SlotRequirement::Identity),
                    Some(mapped) => match &mapped.value {
                        ResolvedPayloadValue::Generated => (
                            BoundSource::Generated,
                            SlotRequirement::Generated {
                                target: target.clone(),
                            },
                        ),
                        ResolvedPayloadValue::ResponseField { field, type_ref: _ }
                            if mapped.conversion.is_none() =>
                        {
                            (
                                BoundSource::ResponseField {
                                    field: field.clone(),
                                },
                                SlotRequirement::Response {
                                    outcome: outcome.name.clone(),
                                    field: field.clone(),
                                },
                            )
                        }
                        ResolvedPayloadValue::InputField { type_ref, .. }
                        | ResolvedPayloadValue::ResponseField { type_ref, .. } => {
                            let because = mapped
                                .conversion
                                .clone()
                                .expect("a non-direct identity mapping declares its conversion");
                            (
                                BoundSource::Conversion {
                                    from: type_ref.clone(),
                                    because: because.clone(),
                                },
                                SlotRequirement::Conversion {
                                    target: target.clone(),
                                    from: type_ref.clone(),
                                    to: entity.identity.type_ref.clone(),
                                    because,
                                },
                            )
                        }
                        ResolvedPayloadValue::Literal { .. } | ResolvedPayloadValue::Cleared => {
                            unreachable!("literals and clears were handled above")
                        }
                    },
                };
                let slot = slots.allocate(
                    None,
                    BoundValue {
                        target,
                        type_ref: entity.identity.type_ref.clone(),
                        source,
                        presence: BoundPresence::Required,
                    },
                    &command.name,
                    requirement_kind,
                );
                IdentityValue::Bound { slot }
            }
        };
        slots
            .requirements
            .push(BindingRequirement::IdentitySupplied {
                command: command.name.clone(),
                value: value.clone(),
                observed_at: at.clone(),
            });
        value
    }

    #[allow(clippy::too_many_arguments, clippy::too_many_lines)]
    fn lower_outcome(
        &mut self,
        command: &ResolvedCommand,
        entity: &ResolvedEntity,
        outcome: &ResolvedOutcome,
        _source_index: usize,
        is_create: bool,
        observed: Option<&EventFieldCoordinate>,
        identity: Option<&IdentityValue>,
        slots: &mut SlotBook,
        operation_fields: &mut BTreeMap<OperationFieldCoordinate, OperationFieldFulfillment>,
    ) -> OutcomeDefinition {
        let path = format!("{}.{}", command.name, outcome.name.as_str());
        let mut when = None;
        let mut in_state = None;
        let mut wrong_state = false;
        match &outcome.condition {
            ResolvedCondition::When { predicate } => {
                when = Some(lower_predicate(predicate, &PathRewrite::Input));
            }
            ResolvedCondition::SubjectState { state, predicate } => {
                in_state = Some(state.to_string());
                when = predicate
                    .as_ref()
                    .map(|predicate| lower_predicate(predicate, &PathRewrite::Input));
            }
            ResolvedCondition::SubjectField {
                field,
                equals,
                predicate,
            } => {
                let subject = Condition::Compare {
                    compare: Box::new(Comparison {
                        left: Value::String(format!("$fields.{field}")),
                        op: RuntimeCompareOp::Eq,
                        right: escape_template_literal(Value::String(equals.clone())),
                    }),
                };
                when = Some(with_input_guard(subject, predicate.as_ref()));
            }
            ResolvedCondition::StateChange {
                states, predicate, ..
            } => {
                let held = Condition::In {
                    values: [
                        Value::String("$from_state".to_owned()),
                        Value::Array(
                            states
                                .iter()
                                .map(|state| Value::String(state.to_string()))
                                .collect(),
                        ),
                    ],
                };
                when = Some(with_input_guard(held, predicate.as_ref()));
            }
            ResolvedCondition::Otherwise => {}
            ResolvedCondition::External { cause } => {
                when = Some(external_evidence(command, outcome, cause, slots));
            }
            ResolvedCondition::ExternalWhen { cause, predicate } => {
                let eligible = lower_predicate(predicate, &PathRewrite::Input);
                let evidence = external_evidence(command, outcome, cause, slots);
                when = Some(Condition::All {
                    all: vec![eligible, evidence],
                });
            }
            ResolvedCondition::WrongState => wrong_state = true,
        }

        let effect = outcome
            .subject
            .as_ref()
            .map_or(OutcomeEffect::None, |subject| match &subject.effect {
                ResolvedEffect::Creates => OutcomeEffect::Creates,
                ResolvedEffect::Updates => OutcomeEffect::Updates,
                ResolvedEffect::Preserves => OutcomeEffect::None,
                ResolvedEffect::Moves { transition } => {
                    let from = transition
                        .from
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>();
                    OutcomeEffect::Moves {
                        from: if let [only] = from.as_slice() {
                            OneOrMany::One(only.clone())
                        } else {
                            OneOrMany::Many(from)
                        },
                        to: transition.to.to_string(),
                    }
                }
            });

        let refusal = outcome.error.as_ref().map(|handle| {
            let error = self.service.source().error(handle);
            self.requirements.push(BindingRequirement::ErrorPayload {
                command: command.name.clone(),
                outcome: outcome.name.clone(),
                error: error.name.clone(),
                fields: error.fields.clone(),
            });
            RefusalDefinition {
                error: error.name.to_string(),
                message: outcome.summary.clone(),
            }
        });

        let accepting = outcome.error.is_none() && outcome.subject.is_some();
        let preserving = outcome
            .subject
            .as_ref()
            .is_some_and(|subject| subject.effect == ResolvedEffect::Preserves);
        if preserving && !wrong_state && command.response.is_empty() {
            self.diagnostic(
                LoweringCode::SilentPreserveUnsupported,
                &path,
                "Entity Runtime refuses an accepting branch with no effect, write, event or response",
            );
        }
        let mut set = BTreeMap::new();
        let mut set_if_present = BTreeMap::new();
        let mapped = outcome
            .sets
            .iter()
            .map(|field| field.target.as_str())
            .collect::<BTreeSet<_>>();

        if accepting {
            if is_create {
                if let Some(identity_value) = identity {
                    set.insert(
                        entity.identity.name.clone(),
                        identity_template(identity_value),
                    );
                    if let IdentityValue::Bound { slot } = identity_value {
                        slots.reference(
                            *slot,
                            &outcome.name,
                            OutcomeSlotLocation::Set(entity.identity.name.clone()),
                        );
                    }
                }
                for mapping in &outcome.sets {
                    if mapping.target == entity.identity.name {
                        continue;
                    }
                    let target = BoundTarget::EntityField {
                        outcome: outcome.name.clone(),
                        field: mapping.target.clone(),
                    };
                    if let Some(value) =
                        self.mapping_value(command, outcome, mapping, target, slots)
                    {
                        let ProducedValue { kind, slot } = value;
                        match kind {
                            ProducedValueKind::Always(produced) => {
                                set.insert(mapping.target.clone(), produced);
                                if let Some(slot) = slot {
                                    slots.reference(
                                        slot,
                                        &outcome.name,
                                        OutcomeSlotLocation::Set(mapping.target.clone()),
                                    );
                                }
                            }
                            ProducedValueKind::IfPresent(argument) => {
                                set_if_present.insert(mapping.target.clone(), argument);
                                if let Some(slot) = slot {
                                    slots.reference(
                                        slot,
                                        &outcome.name,
                                        OutcomeSlotLocation::SetIfPresent(mapping.target.clone()),
                                    );
                                }
                            }
                            // A new instance simply lacks the member; nothing is set.
                            ProducedValueKind::Absent => {}
                        }
                    }
                }
                for field in &entity.fields {
                    if mapped.contains(field.name.as_str()) {
                        continue;
                    }
                    let target = BoundTarget::EntityField {
                        outcome: outcome.name.clone(),
                        field: field.name.clone(),
                    };
                    let optional = field.type_ref.is_optional();
                    let slot = slots.allocate(
                        None,
                        BoundValue {
                            target: target.clone(),
                            type_ref: field.type_ref.clone(),
                            source: BoundSource::Undetermined,
                            presence: if optional {
                                BoundPresence::Optional
                            } else {
                                BoundPresence::Required
                            },
                        },
                        &command.name,
                        SlotRequirement::Undetermined { target },
                    );
                    if optional {
                        set_if_present.insert(
                            field.name.clone(),
                            PresentArgument {
                                argument: slot.argument(),
                            },
                        );
                        slots.reference(
                            slot,
                            &outcome.name,
                            OutcomeSlotLocation::SetIfPresent(field.name.clone()),
                        );
                    } else {
                        set.insert(field.name.clone(), slot.template());
                        slots.reference(
                            slot,
                            &outcome.name,
                            OutcomeSlotLocation::Set(field.name.clone()),
                        );
                    }
                }
            } else if !preserving {
                for mapping in &outcome.sets {
                    if mapping.target == entity.identity.name {
                        self.diagnostic(
                            LoweringCode::OperationIdentityMutationUnsupported,
                            format!("{path}.sets.{}", mapping.target),
                            "an operation cannot mutate the entity's immutable logical identity",
                        );
                        continue;
                    }
                    let target = BoundTarget::EntityField {
                        outcome: outcome.name.clone(),
                        field: mapping.target.clone(),
                    };
                    if let Some(value) =
                        self.mapping_value(command, outcome, mapping, target, slots)
                    {
                        let ProducedValue { kind, slot } = value;
                        match kind {
                            ProducedValueKind::Always(produced) => {
                                set.insert(mapping.target.clone(), produced);
                                if let Some(slot) = slot {
                                    slots.reference(
                                        slot,
                                        &outcome.name,
                                        OutcomeSlotLocation::Set(mapping.target.clone()),
                                    );
                                }
                            }
                            ProducedValueKind::IfPresent(_) => {
                                self.diagnostic(
                                    LoweringCode::OptionalBoundOutputUnsupported,
                                    format!("{path}.sets.{}", mapping.target),
                                    "an operation mapping with invocation-dependent absence requires an explicit operation-field action",
                                );
                            }
                            ProducedValueKind::Absent => {
                                self.diagnostic(
                                    LoweringCode::ClearedValueUnsupported,
                                    format!("{path}.sets.{}", mapping.target),
                                    "Entity Runtime removes an operation field only as a host-selected action, so a source-determined clear has no definition form",
                                );
                            }
                        }
                    }
                }
                for field in &entity.fields {
                    if mapped.contains(field.name.as_str()) {
                        continue;
                    }
                    let coordinate = OperationFieldCoordinate {
                        outcome: outcome.name.clone(),
                        field: field.name.clone(),
                    };
                    let fulfillment = OperationFieldFulfillment {
                        type_ref: field.type_ref.clone(),
                        actions: if field.type_ref.is_optional() {
                            OperationFieldActions::Optional
                        } else {
                            OperationFieldActions::Required
                        },
                    };
                    operation_fields.insert(coordinate.clone(), fulfillment.clone());
                    self.requirements
                        .push(BindingRequirement::OperationFieldPolicySupplied {
                            command: command.name.clone(),
                            target: coordinate.clone(),
                            fulfillment: fulfillment.clone(),
                        });
                }
            }
        }

        let mut fulfills = BTreeMap::new();
        if accepting && !is_create && !preserving {
            for field in &entity.fields {
                if mapped.contains(field.name.as_str()) {
                    continue;
                }
                fulfills.insert(
                    field.name.clone(),
                    OperationFieldRequirement {
                        actions: if field.type_ref.is_optional() {
                            RuntimeOperationFieldActions::Optional
                        } else {
                            RuntimeOperationFieldActions::Required
                        },
                    },
                );
            }
        }

        let mut emits = Vec::new();
        if accepting {
            for (occurrence, handle) in outcome.emits.iter().enumerate() {
                let event = self.service.source().event(handle);
                let mut payload = Map::new();
                let mut payload_if_present = BTreeMap::new();
                for field in &event.fields {
                    let coordinate = EventFieldCoordinate {
                        outcome: outcome.name.clone(),
                        occurrence,
                        event: event.name.clone(),
                        field: field.name.clone(),
                    };
                    if observed == Some(&coordinate) {
                        if let Some(identity_value) = identity {
                            payload.insert(field.name.clone(), identity_template(identity_value));
                            if let IdentityValue::Bound { slot } = identity_value {
                                slots.reference(
                                    *slot,
                                    &outcome.name,
                                    OutcomeSlotLocation::EventPayload {
                                        occurrence,
                                        field: field.name.clone(),
                                    },
                                );
                            }
                            continue;
                        }
                    }
                    let target = BoundTarget::EventField {
                        outcome: outcome.name.clone(),
                        occurrence,
                        event: event.name.clone(),
                        field: field.name.clone(),
                    };
                    if let Some(mapping) = payload_mapping(outcome, &event.name, &field.name) {
                        if let Some(value) =
                            self.mapping_value(command, outcome, mapping, target, slots)
                        {
                            let ProducedValue { kind, slot } = value;
                            match kind {
                                ProducedValueKind::Always(produced) => {
                                    payload.insert(field.name.clone(), produced);
                                    if let Some(slot) = slot {
                                        slots.reference(
                                            slot,
                                            &outcome.name,
                                            OutcomeSlotLocation::EventPayload {
                                                occurrence,
                                                field: field.name.clone(),
                                            },
                                        );
                                    }
                                }
                                ProducedValueKind::IfPresent(argument) => {
                                    payload_if_present.insert(field.name.clone(), argument);
                                    if let Some(slot) = slot {
                                        slots.reference(
                                            slot,
                                            &outcome.name,
                                            OutcomeSlotLocation::EventPayloadIfPresent {
                                                occurrence,
                                                field: field.name.clone(),
                                            },
                                        );
                                    }
                                }
                                ProducedValueKind::Absent => {
                                    self.diagnostic(
                                        LoweringCode::ClearedValueUnsupported,
                                        format!("{path}.payload.{}.{}", event.name, field.name),
                                        "an event payload member cannot be cleared",
                                    );
                                }
                            }
                        }
                    } else {
                        let optional = field.type_ref.is_optional();
                        let slot = slots.allocate(
                            None,
                            BoundValue {
                                target: target.clone(),
                                type_ref: field.type_ref.clone(),
                                source: BoundSource::Undetermined,
                                presence: if optional {
                                    BoundPresence::Optional
                                } else {
                                    BoundPresence::Required
                                },
                            },
                            &command.name,
                            SlotRequirement::Undetermined { target },
                        );
                        if optional {
                            payload_if_present.insert(
                                field.name.clone(),
                                PresentArgument {
                                    argument: slot.argument(),
                                },
                            );
                            slots.reference(
                                slot,
                                &outcome.name,
                                OutcomeSlotLocation::EventPayloadIfPresent {
                                    occurrence,
                                    field: field.name.clone(),
                                },
                            );
                        } else {
                            payload.insert(field.name.clone(), slot.template());
                            slots.reference(
                                slot,
                                &outcome.name,
                                OutcomeSlotLocation::EventPayload {
                                    occurrence,
                                    field: field.name.clone(),
                                },
                            );
                        }
                    }
                }
                emits.push(EventDefinition {
                    event_type: event.name.to_string(),
                    payload: Value::Object(payload),
                    payload_if_present,
                });
            }
        }

        let mut responds = BTreeMap::new();
        let mut responds_if_present = BTreeMap::new();
        if accepting {
            for field in &command.response {
                let target = BoundTarget::ResponseField {
                    outcome: outcome.name.clone(),
                    field: field.name.clone(),
                };
                let optional = field.type_ref.is_optional();
                let key = format!("response:{}:{}", outcome.name.as_str(), field.name);
                let slot = slots.allocate(
                    Some(key),
                    BoundValue {
                        target,
                        type_ref: field.type_ref.clone(),
                        source: BoundSource::ResponseField {
                            field: field.name.clone(),
                        },
                        presence: if optional {
                            BoundPresence::Optional
                        } else {
                            BoundPresence::Required
                        },
                    },
                    &command.name,
                    SlotRequirement::Response {
                        outcome: outcome.name.clone(),
                        field: field.name.clone(),
                    },
                );
                if optional {
                    responds_if_present.insert(
                        field.name.clone(),
                        PresentArgument {
                            argument: slot.argument(),
                        },
                    );
                    slots.reference(
                        slot,
                        &outcome.name,
                        OutcomeSlotLocation::RespondsIfPresent(field.name.clone()),
                    );
                } else {
                    responds.insert(field.name.clone(), slot.template());
                    slots.reference(
                        slot,
                        &outcome.name,
                        OutcomeSlotLocation::Responds(field.name.clone()),
                    );
                }
            }
        }

        OutcomeDefinition {
            name: outcome.name.as_str().to_owned(),
            when,
            in_state,
            wrong_state,
            effect,
            set,
            fulfills,
            set_if_present,
            emits,
            responds,
            responds_if_present,
            refuses: refusal,
        }
    }

    #[allow(clippy::too_many_lines)]
    fn mapping_value(
        &mut self,
        command: &ResolvedCommand,
        outcome: &ResolvedOutcome,
        mapping: &ResolvedPayloadField,
        target: BoundTarget,
        slots: &mut SlotBook,
    ) -> Option<ProducedValue> {
        match &mapping.value {
            ResolvedPayloadValue::Cleared => Some(ProducedValue {
                kind: ProducedValueKind::Absent,
                slot: None,
            }),
            ResolvedPayloadValue::InputField { field, type_ref }
                if mapping.conversion.is_none() =>
            {
                if type_ref.is_optional() {
                    Some(ProducedValue {
                        kind: ProducedValueKind::IfPresent(PresentArgument {
                            argument: format!("input.{field}"),
                        }),
                        slot: None,
                    })
                } else {
                    Some(ProducedValue {
                        kind: ProducedValueKind::Always(Value::String(format!(
                            "$args.input.{field}"
                        ))),
                        slot: None,
                    })
                }
            }
            ResolvedPayloadValue::Literal { value } => self
                .literal(
                    &mapping.target_type,
                    value,
                    &format!(
                        "{}.{}.{}",
                        command.name,
                        outcome.name.as_str(),
                        mapping.target
                    ),
                )
                .map(escape_template_literal)
                .map(|value| ProducedValue {
                    kind: ProducedValueKind::Always(value),
                    slot: None,
                }),
            ResolvedPayloadValue::ResponseField { field, type_ref }
                if mapping.conversion.is_none() =>
            {
                let key = format!("response:{}:{field}", outcome.name.as_str());
                let optional = type_ref.is_optional();
                let slot = slots.allocate(
                    Some(key),
                    BoundValue {
                        target: BoundTarget::ResponseField {
                            outcome: outcome.name.clone(),
                            field: field.clone(),
                        },
                        type_ref: type_ref.clone(),
                        source: BoundSource::ResponseField {
                            field: field.clone(),
                        },
                        presence: if optional {
                            BoundPresence::Optional
                        } else {
                            BoundPresence::Required
                        },
                    },
                    &command.name,
                    SlotRequirement::Response {
                        outcome: outcome.name.clone(),
                        field: field.clone(),
                    },
                );
                if optional {
                    Some(ProducedValue {
                        kind: ProducedValueKind::IfPresent(PresentArgument {
                            argument: slot.argument(),
                        }),
                        slot: Some(slot),
                    })
                } else {
                    Some(ProducedValue {
                        kind: ProducedValueKind::Always(slot.template()),
                        slot: Some(slot),
                    })
                }
            }
            ResolvedPayloadValue::Generated => {
                let slot = slots.allocate(
                    None,
                    BoundValue {
                        target: target.clone(),
                        type_ref: mapping.target_type.clone(),
                        source: BoundSource::Generated,
                        presence: if mapping.target_type.is_optional() {
                            BoundPresence::Optional
                        } else {
                            BoundPresence::Required
                        },
                    },
                    &command.name,
                    SlotRequirement::Generated { target },
                );
                if mapping.target_type.is_optional() {
                    Some(ProducedValue {
                        kind: ProducedValueKind::IfPresent(PresentArgument {
                            argument: slot.argument(),
                        }),
                        slot: Some(slot),
                    })
                } else {
                    Some(ProducedValue {
                        kind: ProducedValueKind::Always(slot.template()),
                        slot: Some(slot),
                    })
                }
            }
            ResolvedPayloadValue::InputField { type_ref, .. }
            | ResolvedPayloadValue::ResponseField { type_ref, .. } => {
                let because = mapping
                    .conversion
                    .clone()
                    .expect("this arm is a declared conversion");
                let slot = slots.allocate(
                    None,
                    BoundValue {
                        target: target.clone(),
                        type_ref: mapping.target_type.clone(),
                        source: BoundSource::Conversion {
                            from: type_ref.clone(),
                            because: because.clone(),
                        },
                        presence: if mapping.target_type.is_optional() {
                            BoundPresence::Optional
                        } else {
                            BoundPresence::Required
                        },
                    },
                    &command.name,
                    SlotRequirement::Conversion {
                        target,
                        from: type_ref.clone(),
                        to: mapping.target_type.clone(),
                        because,
                    },
                );
                if mapping.target_type.is_optional() {
                    Some(ProducedValue {
                        kind: ProducedValueKind::IfPresent(PresentArgument {
                            argument: slot.argument(),
                        }),
                        slot: Some(slot),
                    })
                } else {
                    Some(ProducedValue {
                        kind: ProducedValueKind::Always(slot.template()),
                        slot: Some(slot),
                    })
                }
            }
        }
    }

    fn literal(&mut self, type_ref: &ResolvedTypeRef, text: &str, path: &str) -> Option<Value> {
        let scalar = scalar_primitive(self.service.source(), type_ref, &mut BTreeSet::new());
        match decode_literal(scalar.as_ref(), text) {
            Ok(value) => Some(value),
            Err(code) => {
                self.diagnostic(
                    code,
                    path,
                    "a string-form ESS literal targets a non-scalar representation",
                );
                None
            }
        }
    }

    fn finish(mut self) -> Result<LoweredService, LoweringDiagnostics> {
        for definition in self.definitions.values_mut() {
            let mut conditional = definition
                .create
                .outcomes
                .iter()
                .any(outcome_has_conditional_presence);
            let mut fulfillment = definition
                .create
                .outcomes
                .iter()
                .any(|outcome| !outcome.fulfills.is_empty());
            for operation in definition.operations.values() {
                conditional |= operation
                    .outcomes
                    .iter()
                    .any(outcome_has_conditional_presence);
                fulfillment |= operation
                    .outcomes
                    .iter()
                    .any(|outcome| !outcome.fulfills.is_empty());
            }
            definition.semantics = if fulfillment {
                Semantics::Service3
            } else if conditional {
                Semantics::Service2
            } else {
                Semantics::Service1
            };
        }

        if !self.diagnostics.is_empty() {
            return Err(LoweringDiagnostics(self.diagnostics));
        }

        let mut definitions = BTreeMap::new();
        let mut registry = Registry::new();
        let raw_definitions = self.definitions.clone();
        for (name, definition) in &raw_definitions {
            match ValidatedDefinition::new(definition.clone()) {
                Ok(validated) => {
                    definitions.insert(name.clone(), validated);
                }
                Err(errors) => {
                    for error in errors {
                        self.diagnostic(
                            LoweringCode::TargetDefinitionRefused,
                            format!("{}@{}", name, definition.version),
                            error.to_string(),
                        );
                    }
                }
            }
            if let Err(errors) = registry.register(definition.clone()) {
                for error in errors {
                    self.diagnostic(
                        LoweringCode::TargetDefinitionRefused,
                        format!("{}@{}.register", name, definition.version),
                        error.to_string(),
                    );
                }
            }
        }
        if self.diagnostics.is_empty() {
            if let Err(errors) = registry.validate_all() {
                for error in errors {
                    self.diagnostic(
                        LoweringCode::TargetDefinitionRefused,
                        "registry",
                        error.to_string(),
                    );
                }
            }
        }
        if !self.diagnostics.is_empty() {
            return Err(LoweringDiagnostics(self.diagnostics));
        }

        let synthesis_bytes = self.service.source_plan().to_canonical_json();
        let synthesis_digest = hex_digest(&Sha256::digest(synthesis_bytes.as_bytes()));
        Ok(LoweredService {
            component: self.service.component().name.clone(),
            source_digest: self.service.source().source_digest(),
            synthesis_digest,
            target_revision: ENTITY_RUNTIME_REVISION.to_owned(),
            definitions,
            bindings: BindingPlan {
                commands: self.commands,
                requirements: self.requirements,
                source_capabilities: self.service.capabilities().cloned().collect(),
            },
        })
    }
}

#[derive(Default)]
struct SlotBook {
    slots: BTreeMap<BindingSlot, BoundValue>,
    keys: BTreeMap<String, BindingSlot>,
    requirements: Vec<BindingRequirement>,
    references: Vec<OutcomeSlotReference>,
}

struct OutcomeSlotReference {
    slot: BindingSlot,
    outcome: OutcomeName,
    location: OutcomeSlotLocation,
}

enum OutcomeSlotLocation {
    WhenExternal,
    Set(String),
    SetIfPresent(String),
    EventPayload { occurrence: usize, field: String },
    EventPayloadIfPresent { occurrence: usize, field: String },
    Responds(String),
    RespondsIfPresent(String),
}

impl SlotBook {
    fn reference(
        &mut self,
        slot: BindingSlot,
        outcome: &OutcomeName,
        location: OutcomeSlotLocation,
    ) {
        self.references.push(OutcomeSlotReference {
            slot,
            outcome: outcome.clone(),
            location,
        });
    }

    fn canonicalize(
        &mut self,
        outcome_indices: &BTreeMap<OutcomeName, usize>,
    ) -> BTreeMap<BindingSlot, BindingSlot> {
        let mut entries = std::mem::take(&mut self.slots)
            .into_iter()
            .collect::<Vec<_>>();
        entries.sort_by(|(left_slot, left), (right_slot, right)| {
            slot_sort_key(&left.target, outcome_indices)
                .cmp(&slot_sort_key(&right.target, outcome_indices))
                .then_with(|| left_slot.cmp(right_slot))
        });
        let remap = entries
            .iter()
            .enumerate()
            .map(|(index, (old, _))| {
                (
                    *old,
                    BindingSlot(u32::try_from(index).expect("a service has fewer than 2^32 slots")),
                )
            })
            .collect::<BTreeMap<_, _>>();
        self.slots = entries
            .into_iter()
            .map(|(old, value)| (remap[&old], value))
            .collect();
        for slot in self.keys.values_mut() {
            *slot = remap[slot];
        }
        for requirement in &mut self.requirements {
            remap_requirement_slot(requirement, &remap);
        }
        remap
    }

    fn allocate(
        &mut self,
        reuse: Option<String>,
        value: BoundValue,
        command: &QualifiedName,
        requirement: SlotRequirement,
    ) -> BindingSlot {
        if let Some(key) = reuse.as_ref() {
            if let Some(slot) = self.keys.get(key).copied() {
                assert_eq!(
                    self.slots.get(&slot),
                    Some(&value),
                    "one semantic binding key must retain one source-owned typed value"
                );
                return slot;
            }
        }
        let slot = BindingSlot(
            u32::try_from(self.slots.len()).expect("a service has fewer than 2^32 slots"),
        );
        if let Some(key) = reuse {
            self.keys.insert(key, slot);
        }
        self.slots.insert(slot, value);
        match requirement {
            SlotRequirement::Identity => {}
            SlotRequirement::Response { outcome, field } => {
                self.requirements
                    .push(BindingRequirement::ResponseFieldSupplied {
                        command: command.clone(),
                        outcome,
                        field,
                        slot,
                    });
            }
            SlotRequirement::Generated { target } => {
                self.requirements
                    .push(BindingRequirement::GeneratedFieldSupplied {
                        command: command.clone(),
                        target,
                        slot,
                    });
            }
            SlotRequirement::Undetermined { target } => {
                self.requirements
                    .push(BindingRequirement::UndeterminedFieldSupplied {
                        command: command.clone(),
                        target,
                        slot,
                    });
            }
            SlotRequirement::Conversion {
                target,
                from,
                to,
                because,
            } => self
                .requirements
                .push(BindingRequirement::ConversionSupplied {
                    command: command.clone(),
                    target,
                    slot,
                    from,
                    to,
                    because,
                }),
            SlotRequirement::External { outcome, cause } => {
                self.requirements
                    .push(BindingRequirement::ExternalEvidenceSupplied {
                        command: command.clone(),
                        outcome,
                        cause,
                        slot,
                    });
            }
        }
        slot
    }
}

fn slot_sort_key(
    target: &BoundTarget,
    outcome_indices: &BTreeMap<OutcomeName, usize>,
) -> (usize, u8, usize, String, String) {
    match target {
        BoundTarget::ExternalEvidence { outcome } => {
            (outcome_indices[outcome], 0, 0, String::new(), String::new())
        }
        BoundTarget::LogicalIdentity { at } => (
            outcome_indices[&at.outcome],
            1,
            at.occurrence,
            at.event.to_string(),
            at.field.clone(),
        ),
        BoundTarget::EntityField { outcome, field } => {
            (outcome_indices[outcome], 2, 0, String::new(), field.clone())
        }
        BoundTarget::EventField {
            outcome,
            occurrence,
            event,
            field,
        } => (
            outcome_indices[outcome],
            3,
            *occurrence,
            event.to_string(),
            field.clone(),
        ),
        BoundTarget::ResponseField { outcome, field } => {
            (outcome_indices[outcome], 4, 0, String::new(), field.clone())
        }
    }
}

fn remap_requirement_slot(
    requirement: &mut BindingRequirement,
    remap: &BTreeMap<BindingSlot, BindingSlot>,
) {
    let slot = match requirement {
        BindingRequirement::ResponseFieldSupplied { slot, .. }
        | BindingRequirement::GeneratedFieldSupplied { slot, .. }
        | BindingRequirement::UndeterminedFieldSupplied { slot, .. }
        | BindingRequirement::ConversionSupplied { slot, .. }
        | BindingRequirement::ExternalEvidenceSupplied { slot, .. }
        | BindingRequirement::IdentitySupplied {
            value: IdentityValue::Bound { slot },
            ..
        } => Some(slot),
        BindingRequirement::IdentitySupplied { .. }
        | BindingRequirement::OperationFieldPolicySupplied { .. }
        | BindingRequirement::TimestampSpelling { .. }
        | BindingRequirement::UuidSpelling { .. }
        | BindingRequirement::BytesSpelling { .. }
        | BindingRequirement::DecimalSpelling { .. }
        | BindingRequirement::Binary64Spelling { .. }
        | BindingRequirement::ScaleDeclaration { .. }
        | BindingRequirement::RelationExistence { .. }
        | BindingRequirement::RelationCardinality { .. }
        | BindingRequirement::RelationOwnership { .. }
        | BindingRequirement::ErrorPayload { .. }
        | BindingRequirement::RevisionExpectation { .. } => None,
    };
    if let Some(slot) = slot {
        *slot = remap[slot];
    }
}

fn remap_outcome_slots(
    outcome: &mut OutcomeDefinition,
    remap: &BTreeMap<BindingSlot, BindingSlot>,
    references: &[OutcomeSlotReference],
) {
    for reference in references
        .iter()
        .filter(|reference| reference.outcome.as_str() == outcome.name)
    {
        let replacement = remap[&reference.slot];
        match &reference.location {
            OutcomeSlotLocation::WhenExternal => {
                let truthy = match outcome.when.as_mut() {
                    Some(Condition::Truthy { truthy }) => truthy,
                    Some(Condition::All { all }) => match all.last_mut() {
                        Some(Condition::Truthy { truthy }) => truthy,
                        _ => unreachable!("an input-guarded external slot is the last conjunct"),
                    },
                    _ => unreachable!("an external slot remains a truthy outcome condition"),
                };
                remap_template(truthy, reference.slot, replacement);
            }
            OutcomeSlotLocation::Set(field) => {
                remap_template(
                    outcome.set.get_mut(field).expect("recorded field"),
                    reference.slot,
                    replacement,
                );
            }
            OutcomeSlotLocation::SetIfPresent(field) => {
                remap_argument(
                    &mut outcome
                        .set_if_present
                        .get_mut(field)
                        .expect("recorded field")
                        .argument,
                    reference.slot,
                    replacement,
                );
            }
            OutcomeSlotLocation::EventPayload { occurrence, field } => {
                let payload = outcome.emits[*occurrence]
                    .payload
                    .as_object_mut()
                    .expect("lowered event payloads are objects");
                remap_template(&mut payload[field], reference.slot, replacement);
            }
            OutcomeSlotLocation::EventPayloadIfPresent { occurrence, field } => {
                remap_argument(
                    &mut outcome.emits[*occurrence]
                        .payload_if_present
                        .get_mut(field)
                        .expect("recorded event field")
                        .argument,
                    reference.slot,
                    replacement,
                );
            }
            OutcomeSlotLocation::Responds(field) => {
                remap_template(
                    outcome
                        .responds
                        .get_mut(field)
                        .expect("recorded response field"),
                    reference.slot,
                    replacement,
                );
            }
            OutcomeSlotLocation::RespondsIfPresent(field) => {
                remap_argument(
                    &mut outcome
                        .responds_if_present
                        .get_mut(field)
                        .expect("recorded response field")
                        .argument,
                    reference.slot,
                    replacement,
                );
            }
        }
    }
}

fn remap_template(value: &mut Value, old: BindingSlot, new: BindingSlot) {
    assert_eq!(
        value,
        &old.template(),
        "recorded template reference drifted"
    );
    *value = new.template();
}

fn remap_argument(argument: &mut String, old: BindingSlot, new: BindingSlot) {
    assert_eq!(
        argument,
        &old.argument(),
        "recorded optional argument reference drifted"
    );
    *argument = new.argument();
}

enum SlotRequirement {
    Identity,
    Response {
        outcome: OutcomeName,
        field: String,
    },
    Generated {
        target: BoundTarget,
    },
    Undetermined {
        target: BoundTarget,
    },
    Conversion {
        target: BoundTarget,
        from: ResolvedTypeRef,
        to: ResolvedTypeRef,
        because: String,
    },
    External {
        outcome: OutcomeName,
        cause: String,
    },
}

fn entity_closure(service: &ServiceIr<'_>) -> BTreeSet<QualifiedName> {
    let mut closure = service
        .owned_entities()
        .keys()
        .cloned()
        .collect::<BTreeSet<_>>();
    loop {
        let before = closure.len();
        for entity in service.source().entities().values() {
            if closure.contains(&entity.name) {
                closure.extend(
                    entity
                        .relations
                        .iter()
                        .map(|relation| relation.target.name().clone()),
                );
            }
            if entity.relations.iter().any(|relation| {
                relation.kind == RelationKind::Owns && closure.contains(relation.target.name())
            }) {
                closure.insert(entity.name.clone());
            }
        }
        if closure.len() == before {
            break;
        }
    }
    closure
}

fn outcome_has_conditional_presence(outcome: &OutcomeDefinition) -> bool {
    !outcome.set_if_present.is_empty()
        || !outcome.responds_if_present.is_empty()
        || outcome
            .emits
            .iter()
            .any(|event| !event.payload_if_present.is_empty())
}

fn hex_digest(bytes: &[u8]) -> String {
    use std::fmt::Write as _;

    bytes.iter().fold(
        String::with_capacity(bytes.len() * 2),
        |mut encoded, byte| {
            write!(&mut encoded, "{byte:02x}").expect("writing to a String cannot fail");
            encoded
        },
    )
}

fn primitive_kind(primitive: Primitive) -> FieldKind {
    match primitive {
        Primitive::String
        | Primitive::Timestamp
        | Primitive::Duration
        | Primitive::Uuid
        | Primitive::Bytes => FieldKind::String,
        Primitive::Boolean => FieldKind::Boolean,
        Primitive::Integer => FieldKind::Integer,
        Primitive::Decimal => FieldKind::Number,
        Primitive::Binary64 => FieldKind::Binary64,
    }
}

fn map_key(primitive: Primitive) -> MapKey {
    match primitive {
        Primitive::String => MapKey::String,
        Primitive::Boolean => MapKey::Boolean,
        Primitive::Integer => MapKey::Integer,
        Primitive::Decimal | Primitive::Binary64 => MapKey::Decimal,
        Primitive::Timestamp => MapKey::Timestamp,
        Primitive::Duration => MapKey::Duration,
        Primitive::Uuid => MapKey::Uuid,
        Primitive::Bytes => MapKey::Bytes,
    }
}

fn optional_guard(path: &str, condition: Condition) -> Condition {
    Condition::Any {
        any: vec![
            Condition::Not {
                not: Box::new(Condition::Exists {
                    exists: Value::String(path.to_owned()),
                }),
            },
            condition,
        ],
    }
}

fn sanitize(path: &str) -> String {
    path.chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character
            } else {
                '_'
            }
        })
        .collect()
}

fn identity_template(value: &IdentityValue) -> Value {
    match value {
        IdentityValue::InputField { field } => Value::String(format!("$args.input.{field}")),
        IdentityValue::Literal { value } => escape_template_literal(value.clone()),
        IdentityValue::Bound { slot } => slot.template(),
    }
}

fn escape_template_literal(value: Value) -> Value {
    match value {
        Value::String(text) if text.starts_with('$') => Value::String(format!("${text}")),
        Value::Array(values) => {
            Value::Array(values.into_iter().map(escape_template_literal).collect())
        }
        Value::Object(values) => Value::Object(
            values
                .into_iter()
                .map(|(name, value)| (name, escape_template_literal(value)))
                .collect(),
        ),
        value => value,
    }
}

fn payload_mapping<'a>(
    outcome: &'a ResolvedOutcome,
    event: &QualifiedName,
    field: &str,
) -> Option<&'a ResolvedPayloadField> {
    outcome
        .payload
        .iter()
        .find(|payload| payload.event.name() == event)
        .and_then(|payload| {
            payload
                .fields
                .iter()
                .find(|mapping| mapping.target == field)
        })
}

struct ProducedValue {
    kind: ProducedValueKind,
    slot: Option<BindingSlot>,
}

enum ProducedValueKind {
    Always(Value),
    IfPresent(PresentArgument),
    /// The source leaves the member holding nothing.
    Absent,
}

enum Scalar {
    Boolean,
    Integer,
    Number,
    Binary64,
    String,
}

fn decode_literal(scalar: Option<&Scalar>, text: &str) -> Result<Value, LoweringCode> {
    match scalar {
        Some(Scalar::Boolean) => text
            .parse::<bool>()
            .map(Value::Bool)
            .map_err(|_| LoweringCode::LiteralShapeUnsupported),
        Some(Scalar::Integer) => text
            .parse::<i64>()
            .map(|value| Value::Number(Number::from(value)))
            .map_err(|_| LoweringCode::LiteralShapeUnsupported),
        Some(Scalar::Number | Scalar::Binary64) => serde_json::from_str::<Value>(text)
            .ok()
            .filter(Value::is_number)
            .ok_or(LoweringCode::LiteralShapeUnsupported),
        Some(Scalar::String) => Ok(Value::String(text.to_owned())),
        None => Err(LoweringCode::LiteralShapeUnsupported),
    }
}

fn scalar_primitive(
    ir: &EssIr,
    type_ref: &ResolvedTypeRef,
    active: &mut BTreeSet<QualifiedName>,
) -> Option<Scalar> {
    match type_ref {
        ResolvedTypeRef::Optional { of } => scalar_primitive(ir, of, active),
        ResolvedTypeRef::Primitive { name } => Some(match name {
            Primitive::Boolean => Scalar::Boolean,
            Primitive::Integer => Scalar::Integer,
            Primitive::Decimal => Scalar::Number,
            Primitive::Binary64 => Scalar::Binary64,
            Primitive::String
            | Primitive::Timestamp
            | Primitive::Duration
            | Primitive::Uuid
            | Primitive::Bytes => Scalar::String,
        }),
        ResolvedTypeRef::Declared { name } => {
            if !active.insert(name.name().clone()) {
                return None;
            }
            let result = match &ir.named_type(name).body {
                ResolvedBody::Newtype { of, .. } => scalar_primitive(ir, of, active),
                ResolvedBody::Enum { .. } => Some(Scalar::String),
                ResolvedBody::Struct { .. } | ResolvedBody::Union { .. } => None,
            };
            active.remove(name.name());
            result
        }
        ResolvedTypeRef::List { .. } | ResolvedTypeRef::Map { .. } => None,
    }
}

#[derive(Clone)]
enum PathRewrite {
    Input,
    Entity,
    Nominal {
        base: String,
        value_root: bool,
    },
    Bound {
        outer: Box<PathRewrite>,
        binder: String,
    },
}

impl PathRewrite {
    fn path(&self, path: &FactPath) -> Value {
        let segments = path.segments();
        match self {
            Self::Input => Value::String(format!("$args.input.{}", segments.join("."))),
            Self::Entity => {
                if segments.len() == 1 && segments[0] == "state" {
                    Value::String("$state".to_owned())
                } else {
                    Value::String(format!("$fields.{}", segments.join(".")))
                }
            }
            Self::Nominal { base, value_root } => {
                let tail =
                    if *value_root && segments.first().is_some_and(|segment| segment == "value") {
                        &segments[1..]
                    } else {
                        segments
                    };
                if tail.is_empty() {
                    Value::String(base.clone())
                } else {
                    Value::String(format!("{base}.{}", tail.join(".")))
                }
            }
            Self::Bound { outer, binder } => {
                if segments.first() == Some(binder) {
                    if segments.len() == 1 {
                        Value::String(format!("${binder}"))
                    } else {
                        Value::String(format!("${binder}.{}", segments[1..].join(".")))
                    }
                } else {
                    outer.path(path)
                }
            }
        }
    }
}

fn with_input_guard(selector: Condition, predicate: Option<&Predicate>) -> Condition {
    match predicate {
        None => selector,
        Some(predicate) => Condition::All {
            all: vec![selector, lower_predicate(predicate, &PathRewrite::Input)],
        },
    }
}

/// One required boolean slot for an externally decided verdict, as a truthy selector over it.
fn external_evidence(
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    cause: &str,
    slots: &mut SlotBook,
) -> Condition {
    let slot = slots.allocate(
        None,
        BoundValue {
            target: BoundTarget::ExternalEvidence {
                outcome: outcome.name.clone(),
            },
            type_ref: ResolvedTypeRef::Primitive {
                name: Primitive::Boolean,
            },
            source: BoundSource::External {
                cause: cause.to_owned(),
            },
            presence: BoundPresence::Required,
        },
        &command.name,
        SlotRequirement::External {
            outcome: outcome.name.clone(),
            cause: cause.to_owned(),
        },
    );
    slots.reference(slot, &outcome.name, OutcomeSlotLocation::WhenExternal);
    Condition::Truthy {
        truthy: slot.template(),
    }
}

fn lower_predicate(predicate: &Predicate, rewrite: &PathRewrite) -> Condition {
    match predicate {
        Predicate::Always => Condition::Literal(true),
        Predicate::Never => Condition::Literal(false),
        Predicate::All(children) if children.is_empty() => Condition::Literal(true),
        Predicate::All(children) => Condition::All {
            all: children
                .iter()
                .map(|child| lower_predicate(child, rewrite))
                .collect(),
        },
        Predicate::Any(children) if children.is_empty() => Condition::Literal(false),
        Predicate::Any(children) => Condition::Any {
            any: children
                .iter()
                .map(|child| lower_predicate(child, rewrite))
                .collect(),
        },
        Predicate::Not(child) => Condition::Not {
            not: Box::new(lower_predicate(child, rewrite)),
        },
        Predicate::Compare { left, op, right } => Condition::Compare {
            compare: Box::new(Comparison {
                left: lower_operand(left, rewrite),
                op: match op {
                    CompareOp::Eq => RuntimeCompareOp::Eq,
                    CompareOp::Ne => RuntimeCompareOp::Ne,
                    CompareOp::Lt => RuntimeCompareOp::Lt,
                    CompareOp::Le => RuntimeCompareOp::Lte,
                    CompareOp::Gt => RuntimeCompareOp::Gt,
                    CompareOp::Ge => RuntimeCompareOp::Gte,
                },
                right: lower_operand(right, rewrite),
            }),
        },
        Predicate::Truthy(path) => Condition::Truthy {
            truthy: rewrite.path(path),
        },
        Predicate::Defined(path) => Condition::Exists {
            exists: rewrite.path(path),
        },
        Predicate::AnyOf { path: _, values } if values.is_empty() => Condition::Literal(false),
        Predicate::AnyOf { path, values } => Condition::In {
            values: [
                rewrite.path(path),
                Value::Array(values.iter().map(fact_value).collect()),
            ],
        },
        Predicate::NoneOf { path: _, values } if values.is_empty() => Condition::Literal(true),
        Predicate::NoneOf { path, values } => Condition::Not {
            not: Box::new(Condition::In {
                values: [
                    rewrite.path(path),
                    Value::Array(values.iter().map(fact_value).collect()),
                ],
            }),
        },
        Predicate::Forall(quantified) => lower_quantified(quantified, rewrite, true),
        Predicate::Exists(quantified) => lower_quantified(quantified, rewrite, false),
    }
}

fn lower_quantified(quantified: &Quantified, rewrite: &PathRewrite, universal: bool) -> Condition {
    let nested = PathRewrite::Bound {
        outer: Box::new(rewrite.clone()),
        binder: quantified.bind.clone(),
    };
    let closed = Quantifier {
        over: rewrite.path(&quantified.over),
        bind: quantified.bind.clone(),
        body: Box::new(lower_predicate(&quantified.body, &nested)),
    };
    if universal {
        Condition::ForAll {
            for_all: Box::new(closed),
        }
    } else {
        Condition::ForAny {
            for_any: Box::new(closed),
        }
    }
}

fn lower_operand(operand: &Operand, rewrite: &PathRewrite) -> Value {
    match operand {
        Operand::Fact(path) => rewrite.path(path),
        Operand::Literal(value) => fact_value(value),
    }
}

fn fact_value(value: &FactValue) -> Value {
    escape_template_literal(
        serde_json::to_value(value).expect("fact values have an exact JSON representation"),
    )
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{
        decode_literal, escape_template_literal, fact_value, identity_template,
        remap_outcome_slots, BindingSlot, IdentityValue, LoweringCode, OutcomeSlotLocation,
        OutcomeSlotReference, Scalar,
    };
    use entity_core::{
        Condition, EventDefinition, OutcomeDefinition, PresentArgument, RefusalDefinition,
    };
    use ess_domain::command::OutcomeName;
    use ess_primitives::facts::FactValue;
    use serde_json::{json, Value};

    #[test]
    fn scalar_literal_decoding_preserves_exact_source_values() {
        assert_eq!(
            decode_literal(Some(&Scalar::Boolean), "true"),
            Ok(json!(true))
        );
        assert_eq!(
            decode_literal(Some(&Scalar::Integer), "9007199254740993"),
            Ok(json!(9_007_199_254_740_993_i64))
        );
        assert_eq!(
            decode_literal(Some(&Scalar::String), "001.20"),
            Ok(Value::String("001.20".to_owned()))
        );
        assert_eq!(
            decode_literal(Some(&Scalar::Number), "1234567890.123456789")
                .expect("exact decimal")
                .to_string(),
            "1234567890.123456789"
        );
        assert_eq!(
            decode_literal(Some(&Scalar::Binary64), "-0.0")
                .expect("signed zero")
                .to_string(),
            "-0.0"
        );
        assert_eq!(
            decode_literal(None, "structured"),
            Err(LoweringCode::LiteralShapeUnsupported)
        );
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn slot_remapping_changes_only_recorded_runtime_references() {
        let old = BindingSlot(2);
        let replacement = BindingSlot(9);
        let old_template = old.template();
        let old_argument = old.argument();
        let outcome_name = OutcomeName::new("completed").expect("outcome name");
        let mut outcome = OutcomeDefinition {
            name: outcome_name.as_str().to_owned(),
            when: Some(Condition::Truthy {
                truthy: old_template.clone(),
            }),
            set: BTreeMap::from([
                ("bound".to_owned(), old_template.clone()),
                ("authored".to_owned(), old_template.clone()),
            ]),
            set_if_present: BTreeMap::from([(
                "bound_optional".to_owned(),
                PresentArgument {
                    argument: old_argument.clone(),
                },
            )]),
            emits: vec![EventDefinition {
                event_type: "Observed".to_owned(),
                payload: json!({
                    "bound": old_template,
                    "authored": old.template(),
                    "nested_authored": {"value": old.template()},
                }),
                payload_if_present: BTreeMap::from([(
                    "bound_optional".to_owned(),
                    PresentArgument {
                        argument: old_argument.clone(),
                    },
                )]),
            }],
            responds: BTreeMap::from([
                ("bound".to_owned(), old.template()),
                ("authored".to_owned(), old.template()),
            ]),
            responds_if_present: BTreeMap::from([(
                "bound_optional".to_owned(),
                PresentArgument {
                    argument: old_argument,
                },
            )]),
            refuses: Some(RefusalDefinition {
                error: "Rejected".to_owned(),
                message: old.template().as_str().map(ToOwned::to_owned),
            }),
            ..OutcomeDefinition::default()
        };
        let references = [
            OutcomeSlotLocation::WhenExternal,
            OutcomeSlotLocation::Set("bound".to_owned()),
            OutcomeSlotLocation::SetIfPresent("bound_optional".to_owned()),
            OutcomeSlotLocation::EventPayload {
                occurrence: 0,
                field: "bound".to_owned(),
            },
            OutcomeSlotLocation::EventPayloadIfPresent {
                occurrence: 0,
                field: "bound_optional".to_owned(),
            },
            OutcomeSlotLocation::Responds("bound".to_owned()),
            OutcomeSlotLocation::RespondsIfPresent("bound_optional".to_owned()),
        ]
        .into_iter()
        .map(|location| OutcomeSlotReference {
            slot: old,
            outcome: outcome_name.clone(),
            location,
        })
        .collect::<Vec<_>>();

        remap_outcome_slots(
            &mut outcome,
            &BTreeMap::from([(old, replacement)]),
            &references,
        );

        assert_eq!(
            outcome.when,
            Some(Condition::Truthy {
                truthy: replacement.template()
            })
        );
        assert_eq!(outcome.set["bound"], replacement.template());
        assert_eq!(
            outcome.set_if_present["bound_optional"].argument,
            replacement.argument()
        );
        assert_eq!(outcome.emits[0].payload["bound"], replacement.template());
        assert_eq!(
            outcome.emits[0].payload_if_present["bound_optional"].argument,
            replacement.argument()
        );
        assert_eq!(outcome.responds["bound"], replacement.template());
        assert_eq!(
            outcome.responds_if_present["bound_optional"].argument,
            replacement.argument()
        );

        assert_eq!(outcome.set["authored"], old.template());
        assert_eq!(outcome.emits[0].payload["authored"], old.template());
        assert_eq!(
            outcome.emits[0].payload["nested_authored"]["value"],
            old.template()
        );
        assert_eq!(outcome.responds["authored"], old.template());
        assert_eq!(
            outcome.refuses.expect("refusal").message,
            old.template().as_str().map(ToOwned::to_owned)
        );
    }

    #[test]
    fn authored_dollar_values_are_escaped_only_at_runtime_expression_boundaries() {
        assert_eq!(
            escape_template_literal(json!({
                "plain": "value",
                "single": "$args.bound.b00000002",
                "double": "$$already-authored",
                "nested": ["$fields.note", 7],
            })),
            json!({
                "plain": "value",
                "single": "$$args.bound.b00000002",
                "double": "$$$already-authored",
                "nested": ["$$fields.note", 7],
            })
        );
        let raw_identity = json!("$literal-identity");
        assert_eq!(
            identity_template(&IdentityValue::Literal {
                value: raw_identity.clone(),
            }),
            json!("$$literal-identity")
        );
        assert_eq!(raw_identity, json!("$literal-identity"));
        assert_eq!(
            fact_value(&FactValue::text("$literal-predicate")),
            json!("$$literal-predicate")
        );
    }
}
