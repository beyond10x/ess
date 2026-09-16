//! Conformance lookup over original, pinned component models.
//!
//! A composition exports names; those names do not contain payload contracts.
//! This resolver retains each original IR and returns a declaration together
//! with its owning IR. Transitive handles are always followed in that owner.
//! No merged model or second system header is constructed.
use ess_compiler::{
    ir::{
        ResolvedCommand, ResolvedEntity, ResolvedEvent, ResolvedField, ResolvedType,
        ResolvedTypeRef, ResolvedView,
    },
    refs::{CommandRef, DeclaredTypeRef, EntityRef, EventRef, ViewRef},
    EssIr,
};
use ess_composition::{
    CompiledService, CompositionDiagnostics, CompositionSpec, EssCompositionIr, ServiceKey,
};
use std::collections::{BTreeMap, BTreeSet};

/// Checked component bindings retaining the actual models used for admission.
#[derive(Debug)]
pub struct Models<'a> {
    composition: EssCompositionIr,
    originals: BTreeMap<ServiceKey, &'a EssIr>,
}

/// A declaration and its owning IR, which resolves every transitive type handle.
#[derive(Debug, Clone, Copy)]
pub struct Owned<'a, T> {
    /// Original, pinned compiler authority.
    model: &'a EssIr,
    /// Borrowed declaration; never copied beneath a different system header.
    declaration: &'a T,
}

impl<'a> Models<'a> {
    /// Recheck all keys, component identities and source digests before lookup.
    /// Duplicate, extra, missing or drifted inputs produce the composition's
    /// original diagnostics, rather than partial conformance authority.
    pub fn compile(
        specification: &CompositionSpec,
        originals: &[(ServiceKey, &'a EssIr)],
    ) -> Result<Self, CompositionDiagnostics> {
        let composition = ess_composition::compile(
            specification,
            originals
                .iter()
                .map(|(key, ir)| CompiledService::new(key, ir)),
        )?;
        Ok(Self {
            composition,
            originals: originals
                .iter()
                .map(|(key, ir)| (key.clone(), *ir))
                .collect(),
        })
    }

    /// Exact checked composition, including each component's semantic digest.
    pub fn composition(&self) -> &EssCompositionIr {
        &self.composition
    }

    /// Exact canonical compiler models retained under their original service keys.
    pub fn canonical_originals(&self) -> BTreeMap<ServiceKey, String> {
        self.originals
            .iter()
            .map(|(key, model)| (key.clone(), model.to_canonical_json()))
            .collect()
    }

    /// Resolve only a command admitted on the selected component surface.
    pub fn command(
        &self,
        service: &ServiceKey,
        name: &CommandRef,
    ) -> Option<Owned<'a, ResolvedCommand>> {
        let selected = self.composition.services().get(service)?;
        if !selected.commands().contains(name) {
            return None;
        }
        let model = *self.originals.get(service)?;
        Some(Owned {
            model,
            declaration: model.commands().get(name.name())?,
        })
    }

    /// Resolve only an event admitted on the selected component surface.
    pub fn event(&self, service: &ServiceKey, name: &EventRef) -> Option<Owned<'a, ResolvedEvent>> {
        let selected = self.composition.services().get(service)?;
        if !selected.events().contains(name) {
            return None;
        }
        let model = *self.originals.get(service)?;
        Some(Owned {
            model,
            declaration: model.events().get(name.name())?,
        })
    }

    /// Resolve an owned view, retaining its parameter and row declarations.
    pub fn view(&self, service: &ServiceKey, name: &ViewRef) -> Option<Owned<'a, ResolvedView>> {
        let selected = self.composition.services().get(service)?;
        if !selected.queries().contains(name) {
            return None;
        }
        let model = *self.originals.get(service)?;
        Some(Owned {
            model,
            declaration: model.views().get(name.name())?,
        })
    }

    /// Resolve setup state only within the selected component's owned domains.
    /// Being present elsewhere in a larger imported model does not grant access.
    pub fn entity(
        &self,
        service: &ServiceKey,
        name: &EntityRef,
    ) -> Option<Owned<'a, ResolvedEntity>> {
        let selected = self.composition.services().get(service)?;
        let model = *self.originals.get(service)?;
        let component = model.components().get(selected.component().name())?;
        if !component.owns.iter().any(|domain| {
            model
                .domain(domain)
                .entities
                .iter()
                .any(|entity| entity.name() == name.name())
        }) {
            return None;
        }
        Some(Owned {
            model,
            declaration: model.entities().get(name.name())?,
        })
    }
}

impl<'a, T> Owned<'a, T> {
    /// Original compiler authority for the returned declaration.
    pub fn model(&self) -> &'a EssIr {
        self.model
    }
    /// The complete borrowed declaration.
    pub fn declaration(&self) -> &'a T {
        self.declaration
    }

    fn field_types(
        &self,
        fields: impl IntoIterator<Item = &'a ResolvedField>,
        extra: impl IntoIterator<Item = ResolvedTypeRef>,
    ) -> BTreeMap<DeclaredTypeRef, &'a ResolvedType> {
        let mut types = BTreeSet::new();
        for field in fields {
            crate::synthesize::reachable_types(self.model, &field.type_ref, &mut types);
        }
        for kind in extra {
            crate::synthesize::reachable_types(self.model, &kind, &mut types);
        }
        types
            .into_iter()
            .map(|name| {
                let declaration = &self.model.types()[name.name()];
                (name, declaration)
            })
            .collect()
    }
}

impl<'a> Owned<'a, ResolvedCommand> {
    /// Original definitions reachable from both inputs and declared responses.
    pub fn types(&self) -> BTreeMap<DeclaredTypeRef, &'a ResolvedType> {
        self.field_types(
            self.declaration
                .input
                .iter()
                .chain(&self.declaration.response),
            [],
        )
    }
}
impl<'a> Owned<'a, ResolvedEvent> {
    /// Original definitions reachable from the event payload.
    pub fn types(&self) -> BTreeMap<DeclaredTypeRef, &'a ResolvedType> {
        self.field_types(&self.declaration.fields, [])
    }
}
impl<'a> Owned<'a, ResolvedView> {
    /// Original definitions for query parameters, projected fields and row shape.
    pub fn types(&self) -> BTreeMap<DeclaredTypeRef, &'a ResolvedType> {
        self.field_types(
            self.declaration
                .params
                .iter()
                .chain(&self.declaration.fields),
            self.declaration
                .shape
                .iter()
                .map(|name| ResolvedTypeRef::Declared { name: name.clone() }),
        )
    }
}
impl<'a> Owned<'a, ResolvedEntity> {
    /// Original identity, field and lifecycle-state type definitions for setup.
    pub fn types(&self) -> BTreeMap<DeclaredTypeRef, &'a ResolvedType> {
        self.field_types(
            std::iter::once(&self.declaration.identity).chain(&self.declaration.fields),
            [ResolvedTypeRef::Declared {
                name: self.declaration.state_type.clone(),
            }],
        )
    }
}
