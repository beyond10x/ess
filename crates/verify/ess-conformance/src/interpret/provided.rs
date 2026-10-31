//! Rows the deployment provides (`provided: true`, ess/33): the interpreted target is that
//! deployment, so it answers an entity fixture by creating a row of the entity in the state the
//! fixture names, before the scenario begins, and answers its identity.
//!
//! The row's identity and every required field are minted as a command's generated values are,
//! and the row is checked against the entity's declared types and invariants as an explicit setup
//! is. A type-only fixture stays unsupported here: the target holds no value of a type that no row
//! it holds answers.
use std::collections::BTreeMap;

use ess_primitives::node::Node;

use super::execute::{Instance, Store};
use super::{refusal, Interpreted};
use crate::fixtures::Contract;
use crate::target::{ScenarioContext, TargetError};

impl Interpreted {
    /// Each provided row the contract names, created in a store the scenario begins with.
    pub(super) fn provide_rows(
        &self,
        scenario: &ScenarioContext,
        contract: &Contract,
    ) -> Result<BTreeMap<String, Node>, TargetError> {
        let observation = format!("fixture values for `{}`", scenario.scenario);
        let model = self.model(observation.clone())?;
        let mut store = Store::default();
        let mut values = BTreeMap::new();
        for field in &contract.fields {
            let (Some(entity), Some(state)) = (&field.entity, &field.state) else {
                return Err(TargetError::unsupported(
                    observation,
                    format!(
                        "fixture `{}` is type-only, and the interpreted target provides only rows \
                         of an entity the deployment provides",
                        field.name
                    ),
                ));
            };
            let declared = model.entities().get(entity).ok_or_else(|| {
                TargetError::unavailable(&observation, format!("undeclared entity `{entity}`"))
            })?;
            if !declared.provided {
                return Err(TargetError::unavailable(
                    &observation,
                    format!("`{entity}` does not declare `provided: true`"),
                ));
            }
            let identity = store
                .mint(model, &declared.identity.type_ref)
                .map_err(|why| refusal(observation.clone(), &why))?;
            let mut fields = BTreeMap::new();
            for held in declared
                .fields
                .iter()
                .filter(|held| !held.type_ref.is_optional())
            {
                let value = store
                    .mint(model, &held.type_ref)
                    .map_err(|why| refusal(observation.clone(), &why))?;
                fields.insert(held.name.clone(), value);
            }
            crate::input::validate_entity_setup(
                model,
                &ess_compiler::refs::EntityRef::new(entity.clone()),
                &identity,
                &fields,
                state,
            )
            .map_err(|detail| TargetError::unsupported(&observation, detail))?;
            if !store.establish(
                entity.clone(),
                identity.clone(),
                Instance {
                    state: state.clone(),
                    fields,
                },
            ) {
                return Err(TargetError::unavailable(
                    &observation,
                    "a minted identity is already held",
                ));
            }
            values.insert(field.name.to_string(), identity);
        }
        *self.provided.borrow_mut() = Some((scenario.scenario.clone(), store));
        Ok(values)
    }
}
