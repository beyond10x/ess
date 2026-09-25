//! A borrowed, component-scoped contract over an admitted ESS synthesis plan.
//!
//! Extraction keeps the compiler-owned definition graph intact. Selected declarations and plan
//! entries are references into the caller's [`EssIr`] and [`SynthesisPlan`]; this crate defines no
//! persisted service IR and performs no IO or execution.

use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::ir::{
    EssIr, ResolvedBinding, ResolvedBody, ResolvedCommand, ResolvedComponent, ResolvedDomain,
    ResolvedEntity, ResolvedEvent, ResolvedMappingValue, ResolvedPayloadField,
    ResolvedPayloadValue, ResolvedTypeRef, ResolvedView,
};
use ess_domain::component::ComponentName;
use ess_domain::entity::RelationKind;
use ess_domain::name::QualifiedName;
use ess_synth::{
    plan::conversion_source, Capability, CapabilityKind, ImplementationObligation,
    PlannedCapability, SynthesisDisposition, SynthesisPlan, SynthesisRefusal,
};

/// A deterministic refusal to extract a service contract from the supplied inputs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceDiagnostic {
    /// The requested component is absent from the supplied compiler IR.
    UnknownComponent {
        /// The exact requested component name.
        component: ComponentName,
    },
    /// The supplied plan is not exactly [`SynthesisPlan::of`] the supplied IR.
    PlanMismatch,
}

/// A read-only service selection borrowing the complete compiler model and synthesis plan.
pub struct ServiceIr<'a> {
    source: &'a EssIr,
    source_plan: &'a SynthesisPlan,
    component: &'a ResolvedComponent,
    owned_domains: BTreeMap<QualifiedName, &'a ResolvedDomain>,
    operations: BTreeMap<QualifiedName, &'a ResolvedCommand>,
    published_events: BTreeMap<QualifiedName, &'a ResolvedEvent>,
    views: BTreeMap<QualifiedName, &'a ResolvedView>,
    owned_entities: BTreeMap<QualifiedName, &'a ResolvedEntity>,
    capabilities: Vec<&'a PlannedCapability>,
}

impl<'a> ServiceIr<'a> {
    /// The selected component, including its complete resolved declaration.
    pub fn component(&self) -> &'a ResolvedComponent {
        self.component
    }

    /// Exactly the selected component's owned domains, in qualified-name order.
    pub fn owned_domains(&self) -> &BTreeMap<QualifiedName, &'a ResolvedDomain> {
        &self.owned_domains
    }

    /// Exactly the commands accepted by the selected component, in qualified-name order.
    pub fn operations(&self) -> &BTreeMap<QualifiedName, &'a ResolvedCommand> {
        &self.operations
    }

    /// Exactly the events published by the selected component, in qualified-name order.
    pub fn published_events(&self) -> &BTreeMap<QualifiedName, &'a ResolvedEvent> {
        &self.published_events
    }

    /// Every view declared by a domain the selected component owns.
    pub fn views(&self) -> &BTreeMap<QualifiedName, &'a ResolvedView> {
        &self.views
    }

    /// Every entity declared by a domain the selected component owns.
    pub fn owned_entities(&self) -> &BTreeMap<QualifiedName, &'a ResolvedEntity> {
        &self.owned_entities
    }

    /// The complete immutable compiler IR that minted every selected handle.
    pub fn source(&self) -> &'a EssIr {
        self.source
    }

    /// The exact complete synthesis plan admitted during extraction.
    pub fn source_plan(&self) -> &'a SynthesisPlan {
        self.source_plan
    }

    /// Selected capabilities in their original synthesis-plan order.
    pub fn capabilities(
        &self,
    ) -> impl DoubleEndedIterator<Item = &'a PlannedCapability> + ExactSizeIterator + '_ {
        self.capabilities.iter().copied()
    }

    /// Selected implementation obligations in their original synthesis-plan order.
    pub fn obligations(
        &self,
    ) -> impl Iterator<Item = (&'a Capability, &'a ImplementationObligation)> + '_ {
        self.capabilities.iter().copied().filter_map(|planned| {
            if let SynthesisDisposition::Obligation(obligation) = &planned.disposition {
                Some((&planned.capability, obligation))
            } else {
                None
            }
        })
    }

    /// Selected synthesis refusals in their original synthesis-plan order.
    pub fn refusals(&self) -> impl Iterator<Item = (&'a Capability, &'a SynthesisRefusal)> + '_ {
        self.capabilities.iter().copied().filter_map(|planned| {
            if let SynthesisDisposition::Refused(refusal) = &planned.disposition {
                Some((&planned.capability, refusal))
            } else {
                None
            }
        })
    }
}

/// Extracts one exact component-scoped contract from an immutable compiler IR and complete plan.
pub fn extract<'a>(
    ir: &'a EssIr,
    plan: &'a SynthesisPlan,
    component: &ComponentName,
) -> Result<ServiceIr<'a>, Vec<ServiceDiagnostic>> {
    let selected = ir.components().get(component);
    let mut diagnostics = Vec::new();
    if selected.is_none() {
        diagnostics.push(ServiceDiagnostic::UnknownComponent {
            component: component.clone(),
        });
    }
    if plan != &SynthesisPlan::of(ir) {
        diagnostics.push(ServiceDiagnostic::PlanMismatch);
    }
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    let component = selected.expect("the unknown-component diagnostic returned above");

    let owned_domains = component
        .owns
        .iter()
        .map(|handle| {
            let domain = ir.domain(handle);
            (domain.name.clone(), domain)
        })
        .collect();
    let operations = component
        .accepts
        .iter()
        .map(|handle| {
            let command = ir.command(handle);
            (command.name.clone(), command)
        })
        .collect();
    let published_events = component
        .publishes
        .iter()
        .map(|handle| {
            let event = ir.event(handle);
            (event.name.clone(), event)
        })
        .collect();

    let mut views = BTreeMap::new();
    let mut owned_entities = BTreeMap::new();
    for domain_handle in &component.owns {
        let domain = ir.domain(domain_handle);
        for handle in &domain.views {
            let view = ir.view(handle);
            views.insert(view.name.clone(), view);
        }
        for handle in &domain.entities {
            let entity = ir.entity(handle);
            owned_entities.insert(entity.name.clone(), entity);
        }
    }

    let selected_capabilities = selected_capabilities(
        ir,
        component,
        &operations,
        &published_events,
        &views,
        &owned_entities,
    );
    let capabilities = plan
        .capabilities
        .iter()
        .filter(|planned| selected_capabilities.contains(&planned.capability))
        .collect();

    Ok(ServiceIr {
        source: ir,
        source_plan: plan,
        component,
        owned_domains,
        operations,
        published_events,
        views,
        owned_entities,
        capabilities,
    })
}

fn selected_capabilities(
    ir: &EssIr,
    component: &ResolvedComponent,
    operations: &BTreeMap<QualifiedName, &ResolvedCommand>,
    published_events: &BTreeMap<QualifiedName, &ResolvedEvent>,
    views: &BTreeMap<QualifiedName, &ResolvedView>,
    owned_entities: &BTreeMap<QualifiedName, &ResolvedEntity>,
) -> BTreeSet<Capability> {
    let operation_names = operations.keys().cloned().collect::<BTreeSet<_>>();
    let published_names = published_events.keys().cloned().collect::<BTreeSet<_>>();
    let mut selection = CapabilitySelection::new(component, owned_entities, &published_names);
    selection.include_operations(ir, operations);
    selection.include_views(views);
    selection.include_actors(ir, &operation_names);
    selection.include_bindings(ir, &operation_names, &published_names);
    selection.finish(ir)
}

struct CapabilitySelection {
    selected: BTreeSet<Capability>,
    entities: BTreeSet<QualifiedName>,
    events: BTreeSet<QualifiedName>,
    errors: BTreeSet<QualifiedName>,
    types: BTreeSet<QualifiedName>,
    conversions: BTreeSet<usize>,
}

impl CapabilitySelection {
    fn new(
        component: &ResolvedComponent,
        owned_entities: &BTreeMap<QualifiedName, &ResolvedEntity>,
        published_names: &BTreeSet<QualifiedName>,
    ) -> Self {
        let mut selection = Self {
            selected: BTreeSet::new(),
            entities: owned_entities.keys().cloned().collect(),
            events: published_names.clone(),
            errors: BTreeSet::new(),
            types: BTreeSet::new(),
            conversions: BTreeSet::new(),
        };
        for kind in [
            CapabilityKind::ComponentPort,
            CapabilityKind::ComponentTransport,
            CapabilityKind::Workload,
        ] {
            include(&mut selection.selected, kind, component.name.to_string());
        }
        for setting in &component.settings {
            include_type_ref(&mut selection.types, &setting.type_ref);
        }
        selection
    }

    fn include_operations(
        &mut self,
        ir: &EssIr,
        operations: &BTreeMap<QualifiedName, &ResolvedCommand>,
    ) {
        for command in operations.values() {
            self.include_operation(ir, command);
        }
    }

    fn include_operation(&mut self, ir: &EssIr, command: &ResolvedCommand) {
        include(
            &mut self.selected,
            CapabilityKind::CommandContract,
            command.name.to_string(),
        );
        include(
            &mut self.selected,
            CapabilityKind::CommandBehavior,
            command.name.to_string(),
        );
        include_fields(&mut self.types, &command.input);
        include_fields(&mut self.types, &command.response);
        for outcome in &command.outcomes {
            if let Some(subject) = &outcome.subject {
                self.entities.insert(subject.entity.name().clone());
                include_type_ref(&mut self.types, &subject.instance.field().type_ref);
                if let Some(event) = subject.instance.event() {
                    self.events.insert(event.name().clone());
                }
            }
            self.events
                .extend(outcome.emits.iter().map(|event| event.name().clone()));
            if let Some(error) = &outcome.error {
                self.errors.insert(error.name().clone());
            }
            for payload in &outcome.payload {
                self.events.insert(payload.event.name().clone());
                self.include_payload_fields(ir, &payload.fields);
            }
            self.include_payload_fields(ir, &outcome.sets);
        }
    }

    fn include_payload_fields(&mut self, ir: &EssIr, fields: &[ResolvedPayloadField]) {
        for field in fields {
            include_payload_field(ir, &mut self.types, &mut self.conversions, field);
        }
    }

    fn include_views(&mut self, views: &BTreeMap<QualifiedName, &ResolvedView>) {
        for view in views.values() {
            include(
                &mut self.selected,
                CapabilityKind::ViewType,
                view.name.to_string(),
            );
            include(
                &mut self.selected,
                CapabilityKind::ViewQuery,
                view.name.to_string(),
            );
            self.entities.insert(view.source.name().clone());
            include_fields(&mut self.types, &view.fields);
            include_fields(&mut self.types, &view.params);
            if let Some(shape) = &view.shape {
                self.types.insert(shape.name().clone());
            }
        }
    }

    fn include_actors(&mut self, ir: &EssIr, operation_names: &BTreeSet<QualifiedName>) {
        for actor in ir.actors().values() {
            if actor
                .may
                .iter()
                .any(|command| operation_names.contains(command.name()))
            {
                include(
                    &mut self.selected,
                    CapabilityKind::ActorGrants,
                    actor.name.to_string(),
                );
            }
        }
    }

    fn include_bindings(
        &mut self,
        ir: &EssIr,
        operation_names: &BTreeSet<QualifiedName>,
        published_names: &BTreeSet<QualifiedName>,
    ) {
        for binding in ir.bindings().values() {
            let invokes_selected = operation_names.contains(binding.command.name());
            let reacts_to_publication = binding
                .cause
                .event()
                .is_some_and(|event| published_names.contains(event.name()));
            if invokes_selected || reacts_to_publication {
                include_binding(
                    ir,
                    &mut self.selected,
                    &mut self.events,
                    &mut self.types,
                    &mut self.conversions,
                    binding,
                );
            }
        }
    }

    fn finish(mut self, ir: &EssIr) -> BTreeSet<Capability> {
        include_entity_closure(ir, &mut self.selected, &mut self.entities, &mut self.types);
        self.include_events(ir);
        self.include_errors(ir);
        self.include_conversions(ir);
        include_type_closure(ir, &mut self.selected, &mut self.types);
        self.selected
    }

    fn include_events(&mut self, ir: &EssIr) {
        for event_name in &self.events {
            let event = ir
                .events()
                .get(event_name)
                .expect("an event name came from a compiler-minted handle");
            include(
                &mut self.selected,
                CapabilityKind::EventType,
                event.name.to_string(),
            );
            include_fields(&mut self.types, &event.fields);
        }
    }

    fn include_errors(&mut self, ir: &EssIr) {
        for error_name in &self.errors {
            let error = ir
                .errors()
                .get(error_name)
                .expect("an error name came from a compiler-minted handle");
            include(
                &mut self.selected,
                CapabilityKind::ErrorType,
                error.name.to_string(),
            );
            include_fields(&mut self.types, &error.fields);
        }
    }

    fn include_conversions(&mut self, ir: &EssIr) {
        for conversion_index in &self.conversions {
            let conversion = &ir.conversions()[*conversion_index];
            include(
                &mut self.selected,
                CapabilityKind::Conversion,
                conversion_source(conversion),
            );
            include_type_ref(&mut self.types, &conversion.from);
            include_type_ref(&mut self.types, &conversion.to);
        }
    }
}

fn include(selected: &mut BTreeSet<Capability>, kind: CapabilityKind, source: String) {
    selected.insert(Capability { kind, source });
}

fn include_fields(types: &mut BTreeSet<QualifiedName>, fields: &[ess_compiler::ir::ResolvedField]) {
    for field in fields {
        include_type_ref(types, &field.type_ref);
    }
}

fn include_type_ref(types: &mut BTreeSet<QualifiedName>, type_ref: &ResolvedTypeRef) {
    types.extend(
        type_ref
            .named_leaves()
            .into_iter()
            .map(|handle| handle.name().clone()),
    );
}

fn include_payload_field(
    ir: &EssIr,
    types: &mut BTreeSet<QualifiedName>,
    conversions: &mut BTreeSet<usize>,
    field: &ResolvedPayloadField,
) {
    include_type_ref(types, &field.target_type);
    let source = match &field.value {
        ResolvedPayloadValue::ResponseField { type_ref, .. }
        | ResolvedPayloadValue::InputField { type_ref, .. } => Some(type_ref),
        ResolvedPayloadValue::Generated
        | ResolvedPayloadValue::Literal { .. }
        | ResolvedPayloadValue::Cleared => None,
    };
    if let Some(source) = source {
        include_type_ref(types, source);
    }
    if let Some(because) = &field.conversion {
        include_conversion(ir, conversions, source, &field.target_type, because);
    }
}

fn include_binding(
    ir: &EssIr,
    selected: &mut BTreeSet<Capability>,
    events: &mut BTreeSet<QualifiedName>,
    types: &mut BTreeSet<QualifiedName>,
    conversions: &mut BTreeSet<usize>,
    binding: &ResolvedBinding,
) {
    for kind in [
        CapabilityKind::BindingTransformation,
        CapabilityKind::BindingDelivery,
        CapabilityKind::BindingEscalation,
    ] {
        include(selected, kind, binding.name.to_string());
    }
    if let Some(event) = binding.cause.event() {
        events.insert(event.name().clone());
    }
    if let Some(event) = &binding.escalation {
        events.insert(event.name().clone());
    }
    include_fields(types, &ir.command(&binding.command).input);
    if let Some(periodic) = binding.cause.periodic() {
        include_fields(types, &periodic.context);
        include_fields(types, &periodic.read);
    }
    if let Some(selection) = &binding.selection {
        types.extend(selection.types.values().map(|handle| handle.name().clone()));
        for input in &selection.plan.inputs {
            let Some(because) = &input.conversion else {
                continue;
            };
            let source = input.source.type_ref();
            for (index, conversion) in ir.conversions().iter().enumerate() {
                if conversion.because == *because
                    && selection_type_matches(&conversion.from, &source, &selection.types)
                    && selection_type_matches(&conversion.to, &input.list_type, &selection.types)
                {
                    conversions.insert(index);
                }
            }
        }
    }
    for mapping in &binding.mapping {
        include_type_ref(types, &mapping.target_type);
        let source = match &mapping.value {
            ResolvedMappingValue::HostContext { type_ref, .. }
            | ResolvedMappingValue::HostRead { type_ref, .. }
            | ResolvedMappingValue::Selection { type_ref, .. }
            | ResolvedMappingValue::EventField { type_ref, .. } => Some(type_ref),
            ResolvedMappingValue::EventAccessor {
                type_ref,
                types: used,
                ..
            } => {
                types.extend(used.values().map(|handle| handle.name().clone()));
                Some(type_ref)
            }
            ResolvedMappingValue::Literal { .. } => None,
        };
        if let Some(source) = source {
            include_type_ref(types, source);
        }
        if let Some(because) = &mapping.conversion {
            include_conversion(ir, conversions, source, &mapping.target_type, because);
        }
    }
}

fn selection_type_matches(
    resolved: &ResolvedTypeRef,
    declared: &ess_domain::types::TypeRef,
    types: &BTreeMap<QualifiedName, ess_compiler::ir::TypeHandle>,
) -> bool {
    match (resolved, declared) {
        (
            ResolvedTypeRef::Primitive { name: resolved },
            ess_domain::types::TypeRef::Primitive(declared),
        ) => resolved == declared,
        (
            ResolvedTypeRef::Declared { name: resolved },
            ess_domain::types::TypeRef::Named(declared),
        ) => types.get(declared) == Some(resolved),
        (
            ResolvedTypeRef::Optional { of: resolved },
            ess_domain::types::TypeRef::Optional(declared),
        )
        | (ResolvedTypeRef::List { of: resolved }, ess_domain::types::TypeRef::List(declared)) => {
            selection_type_matches(resolved, declared, types)
        }
        (
            ResolvedTypeRef::Map {
                key: resolved_key,
                value: resolved_value,
            },
            ess_domain::types::TypeRef::Map(declared_key, declared_value),
        ) => {
            resolved_key == declared_key
                && selection_type_matches(resolved_value, declared_value, types)
        }
        _ => false,
    }
}

fn include_conversion(
    ir: &EssIr,
    conversions: &mut BTreeSet<usize>,
    source: Option<&ResolvedTypeRef>,
    target: &ResolvedTypeRef,
    because: &str,
) {
    for (index, conversion) in ir.conversions().iter().enumerate() {
        if conversion.to == *target
            && conversion.because == because
            && source.is_none_or(|source| &conversion.from == source)
        {
            conversions.insert(index);
        }
    }
}

fn include_entity_closure(
    ir: &EssIr,
    selected: &mut BTreeSet<Capability>,
    entities: &mut BTreeSet<QualifiedName>,
    types: &mut BTreeSet<QualifiedName>,
) {
    let mut pending = entities.iter().cloned().collect::<Vec<_>>();
    while let Some(name) = pending.pop() {
        let entity = ir
            .entities()
            .get(&name)
            .expect("an entity name came from a compiler-minted handle");
        include(
            selected,
            CapabilityKind::EntityLifecycle,
            entity.name.to_string(),
        );
        include_type_ref(types, &entity.identity.type_ref);
        include_fields(types, &entity.fields);
        types.insert(entity.state_type.name().clone());

        for relation in &entity.relations {
            let target = relation.target.name().clone();
            if entities.insert(target.clone()) {
                pending.push(target);
            }
        }
        for carried in ir.relations_carried_by(&entity.name).values() {
            if carried.relation.kind == RelationKind::Owns {
                let source = (*carried.source).clone();
                if entities.insert(source.clone()) {
                    pending.push(source);
                }
            }
        }
    }
}

fn include_type_closure(
    ir: &EssIr,
    selected: &mut BTreeSet<Capability>,
    types: &mut BTreeSet<QualifiedName>,
) {
    let mut pending = types.iter().cloned().collect::<Vec<_>>();
    while let Some(name) = pending.pop() {
        let declared = ir
            .types()
            .get(&name)
            .expect("a type name came from a compiler-minted handle");
        include(
            selected,
            CapabilityKind::DomainType,
            declared.name.to_string(),
        );
        let mut reached = BTreeSet::new();
        match &declared.body {
            ResolvedBody::Newtype { of, .. } => include_type_ref(&mut reached, of),
            ResolvedBody::Struct { fields, .. } => include_fields(&mut reached, fields),
            ResolvedBody::Enum { .. } => {}
            ResolvedBody::Union { variants, .. } => {
                for variant in variants.values() {
                    include_type_ref(&mut reached, variant);
                }
            }
        }
        for reached in reached {
            if types.insert(reached.clone()) {
                pending.push(reached);
            }
        }
    }
}
