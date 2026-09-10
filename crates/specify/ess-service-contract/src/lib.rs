//! Complete selected component semantics, borrowed from one validated compiler model.
//!
//! Selection does not implement authorization, persistence, transport or execution. External
//! outcomes remain binding obligations. The optional `synthesis` module resolves target
//! capabilities separately from this language-neutral semantic contract.

use std::collections::BTreeMap;
use std::fmt;

use ess_compiler::ir::{
    ErrorHandle, EventHandle, ResolvedCommand, ResolvedComponent, ResolvedCondition,
    ResolvedEntity, ResolvedError, ResolvedEvent, ResolvedOutcome, ResolvedView,
};
use ess_compiler::refs::ComponentRef;
use ess_compiler::EssIr;

#[cfg(feature = "synthesis")]
pub mod synthesis;

/// A selected service surface tied to the lifetime of its original compiler model.
#[derive(Debug)]
pub struct ServiceIr<'a> {
    source: &'a EssIr,
    component: &'a ResolvedComponent,
    commands: Vec<&'a ResolvedCommand>,
    events: BTreeMap<EventHandle, &'a ResolvedEvent>,
    errors: BTreeMap<ErrorHandle, &'a ResolvedError>,
}

/// Selects a declared component without weakening or copying its semantic definitions.
pub fn compile<'a>(
    source: &'a EssIr,
    component: &ComponentRef,
) -> Result<ServiceIr<'a>, MissingComponent> {
    let selected = source
        .components()
        .get(component.name())
        .ok_or_else(|| MissingComponent(component.clone()))?;
    let commands: Vec<_> = selected
        .accepts
        .iter()
        .map(|handle| source.command(handle))
        .collect();
    let events = selected
        .publishes
        .iter()
        .chain(commands.iter().flat_map(|command| command.emits()))
        .map(|handle| (handle.clone(), source.event(handle)))
        .collect();
    let errors = commands
        .iter()
        .flat_map(|command| command.errors())
        .map(|handle| (handle.clone(), source.error(handle)))
        .collect();
    Ok(ServiceIr {
        source,
        component: selected,
        commands,
        events,
        errors,
    })
}

impl<'a> ServiceIr<'a> {
    /// Original model for total lookup of the selected definitions' handles and references.
    pub const fn source(&self) -> &'a EssIr {
        self.source
    }

    /// The exact component declaration, including domain ownership and published surface.
    pub const fn component(&self) -> &'a ResolvedComponent {
        self.component
    }

    /// Accepted commands in stable name order, retaining every outcome and its event order.
    pub fn commands(&self) -> impl Iterator<Item = &'a ResolvedCommand> + '_ {
        self.commands.iter().copied()
    }

    /// Published or emitted event definitions in stable name order.
    pub fn events(&self) -> impl Iterator<Item = &'a ResolvedEvent> + '_ {
        self.events.values().copied()
    }

    /// Errors named by selected command outcomes, in stable name order.
    pub fn errors(&self) -> impl Iterator<Item = &'a ResolvedError> + '_ {
        self.errors.values().copied()
    }

    /// State owned through the selected component's declared domains.
    pub fn entities(&self) -> impl Iterator<Item = &'a ResolvedEntity> + '_ {
        self.source
            .entities()
            .values()
            .filter(|entity| self.component.owns.contains(&entity.domain))
    }

    /// Complete views belonging to the selected component's owned domains.
    pub fn views(&self) -> impl Iterator<Item = &'a ResolvedView> + '_ {
        self.source
            .views()
            .values()
            .filter(|view| self.component.owns.contains(&view.domain))
    }

    /// Outcomes whose choice is external to the model, with their original causes and subjects.
    pub fn external_outcomes(
        &self,
    ) -> impl Iterator<Item = (&'a ResolvedCommand, &'a ResolvedOutcome)> + '_ {
        self.commands().flat_map(|command| {
            command
                .outcomes
                .iter()
                .filter(|outcome| matches!(outcome.condition, ResolvedCondition::External { .. }))
                .map(move |outcome| (command, outcome))
        })
    }
}

/// A requested service component does not exist in the supplied model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissingComponent(pub ComponentRef);

impl fmt::Display for MissingComponent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ESS component {} does not exist in the supplied EssIr",
            self.0
        )
    }
}

impl std::error::Error for MissingComponent {}

/// Command input type spellings in stable field-name order, for legacy binding envelopes.
///
/// This is a summary, not the complete command contract exposed by [`ServiceIr::commands`].
pub fn command_fields(command: &ResolvedCommand) -> BTreeMap<String, String> {
    command
        .input
        .iter()
        .map(|field| (field.name.clone(), field.type_ref.to_string()))
        .collect()
}

/// View row type spellings in stable field-name order, for legacy binding envelopes.
///
/// Query parameters, filters and ordering remain on the complete [`ResolvedView`].
pub fn view_fields(view: &ResolvedView) -> BTreeMap<String, String> {
    view.fields
        .iter()
        .map(|field| (field.name.clone(), field.type_ref.to_string()))
        .collect()
}
