//! Rows the deployment provides (`provided: true`, ess/33): the interpreted target is that
//! deployment, so it answers an entity fixture by creating a row of the entity in the state the
//! fixture names, before the scenario begins, and answers its identity.
//!
//! The row's identity is minted as a command's generated values are; its stored fields are the
//! first values the witness search offers under the entity's invariants that the entity admits,
//! checked against its declared types and invariants as an explicit setup is. A type-only fixture
//! stays unsupported here: the target holds no value of a type that no row it holds answers.
use std::collections::BTreeMap;

use ess_compiler::ir::{EssIr, ResolvedCommand, ResolvedEntity};
use ess_domain::entity::StateName;
use ess_domain::types::UndeclaredFields;
use ess_primitives::node::Node;
use ess_primitives::predicate::Predicate;

use super::execute::{Instance, Store};
use super::{refusal, Interpreted};
use crate::fixtures::Contract;
use crate::target::{ScenarioContext, TargetError};
use crate::witness::Distinction;

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
            let fields = row_fields(model, declared, (&identity, state), values.len())
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

/// The stored fields of the `nth` provided row of `declared`, whose identity is `identity` and
/// which rests in `state`: the first of the values the witness search offers for the entity's
/// fields, read as one command's input under the entity's invariants, that the entity admits as a
/// row (story:a-fixture-may-be-a-row-the-deployment-already-holds). A guard over a command's input
/// is searched the same way, so an invariant such as `seats > 1000` is met by the literal's
/// neighbour. Refused naming the entity's invariants where no value offered meets them.
fn row_fields(
    model: &EssIr,
    declared: &ResolvedEntity,
    (identity, state): (&Node, &StateName),
    nth: usize,
) -> Result<BTreeMap<String, Node>, String> {
    let entity = ess_compiler::refs::EntityRef::new(declared.name.clone());
    let admitted = |fields: &BTreeMap<String, Node>| {
        crate::input::validate_entity_setup(model, &entity, identity, fields, state)
    };
    let row = ResolvedCommand {
        name: declared.name.child("provided-row"),
        domain: declared.domain.clone(),
        input: declared.fields.clone(),
        examples: BTreeMap::new(),
        fixture_inputs: BTreeMap::new(),
        response: Vec::new(),
        undeclared_fields: UndeclaredFields::default(),
        outcomes: Vec::new(),
        naming: declared.naming.clone(),
        refs: Vec::new(),
        system_issued: false,
        await_within: None,
    };
    let invariants: Vec<&Predicate> = declared
        .invariants
        .iter()
        .map(|invariant| &invariant.predicate)
        .collect();
    let offered =
        crate::witness::candidates(model, &row, &invariants, Distinction::further(nth))
            .map_err(|gap| format!("a provided row of `{}` has no value: {gap}", declared.name))?;
    let mut refused = None;
    for fields in offered {
        match admitted(&fields) {
            Ok(()) => return Ok(fields),
            Err(why) => {
                refused.get_or_insert(why);
            }
        }
    }
    let statements: Vec<&str> = declared
        .invariants
        .iter()
        .map(|invariant| invariant.statement.as_str())
        .collect();
    Err(format!(
        "no row of `{}` the interpreted target mints in `{state}` meets its invariants `{}`{}",
        declared.name,
        statements.join("`, `"),
        refused.map_or_else(String::new, |why| format!(": {why}"))
    ))
}
