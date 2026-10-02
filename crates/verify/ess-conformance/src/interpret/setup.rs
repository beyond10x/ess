//! Explicit upstream state, checked against the model before the isolated store changes.
use super::{execute::Instance, Interpreted};
use crate::target::{EntitySetupRequest, TargetError};

impl Interpreted {
    pub(super) fn setup_entity(&self, request: EntitySetupRequest) -> Result<(), TargetError> {
        let model = self.model("entity setup")?;
        self.scenario.borrow().facts.check(&request.correlation)?;
        crate::input::validate_entity_setup(
            model,
            &request.entity,
            &request.identity,
            &request.fields,
            &request.state,
        )
        .map_err(|detail| TargetError::unavailable("entity setup", detail))?;
        let mut scenario = self.scenario.borrow_mut();
        if !scenario.store.establish(
            request.entity.name().clone(),
            request.identity,
            Instance {
                state: request.state,
                fields: request.fields,
            },
        ) {
            return Err(TargetError::unavailable(
                "entity setup",
                "the entity identity is already established",
            ));
        }
        // Setup promises read-visible upstream data, including eventual views. Publish a
        // current projection snapshot without simulating commands, events or time passing.
        let visible = scenario.projection_reads;
        let store = scenario.store.clone();
        scenario.projection_versions.push((visible, store));
        Ok(())
    }
}
