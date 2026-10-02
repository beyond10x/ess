//! What a generated entry point can reach, independent of its target language.
use std::collections::BTreeSet;

use ess_compiler::ir::{EssIr, ResolvedBody, ResolvedComponent, ResolvedTypeRef};
use ess_domain::{name::QualifiedName, types::Primitive};
use ess_gen::http::{routes, Served};

use crate::plan::{CapabilityKind, SynthesisPlan};

/// Commands, queries and binding capabilities reachable from one served surface.
pub(crate) struct Reachable {
    pub commands: BTreeSet<QualifiedName>,
    pub views: BTreeSet<QualifiedName>,
    pub bindings: BTreeSet<String>,
}

impl Reachable {
    pub fn of(ir: &EssIr, component: &ResolvedComponent) -> Self {
        let mut this = Self {
            commands: BTreeSet::new(),
            views: BTreeSet::new(),
            bindings: BTreeSet::new(),
        };
        for route in routes(ir, component) {
            match route.serves {
                Served::Command(command) => {
                    this.commands.insert(command.name().clone());
                }
                Served::View(view) => {
                    this.views.insert(view.name().clone());
                }
            }
        }
        let mut events = BTreeSet::new();
        loop {
            let before = (this.commands.len(), this.bindings.len(), events.len());
            for command in &this.commands {
                for outcome in &ir.commands()[command].outcomes {
                    events.extend(outcome.emits.iter().map(|event| event.name().clone()));
                }
            }
            for binding in ir.bindings().values() {
                if binding
                    .cause
                    .event()
                    .is_some_and(|event| events.contains(event.name()))
                {
                    this.bindings.insert(binding.name.to_string());
                    this.commands.insert(binding.command.name().clone());
                    if let Some(event) = &binding.escalation {
                        events.insert(event.name().clone());
                    }
                }
            }
            if before == (this.commands.len(), this.bindings.len(), events.len()) {
                break;
            }
        }
        this
    }

    pub fn obligations(&self, plan: &SynthesisPlan) -> BTreeSet<String> {
        plan.obligations()
            .filter_map(|(capability, _)| {
                let reached = match capability.kind {
                    CapabilityKind::CommandBehavior => self
                        .commands
                        .iter()
                        .any(|name| name.to_string() == capability.source),
                    CapabilityKind::ViewQuery => self
                        .views
                        .iter()
                        .any(|name| name.to_string() == capability.source),
                    CapabilityKind::BindingTransformation
                    | CapabilityKind::BindingDelivery
                    | CapabilityKind::BindingEscalation => {
                        self.bindings.contains(&capability.source)
                    }
                    _ => false,
                };
                reached.then(|| format!("{}: {}", capability.kind.describes(), capability.source))
            })
            .collect()
    }
}

/// The primitive beneath transparent nominal wrappers, if there is one.
pub(crate) fn primitive(ir: &EssIr, reference: &ResolvedTypeRef) -> Option<Primitive> {
    match reference {
        ResolvedTypeRef::Primitive { name } => Some(*name),
        ResolvedTypeRef::Declared { name } => match &ir.named_type(name).body {
            ResolvedBody::Newtype { of, .. } => primitive(ir, of),
            _ => None,
        },
        _ => None,
    }
}

/// A generated context supplies only these two explicitly accepted kinds.
pub(crate) fn supported(ir: &EssIr, reference: &ResolvedTypeRef) -> bool {
    matches!(
        primitive(ir, reference),
        Some(Primitive::Uuid | Primitive::Timestamp)
    )
}
