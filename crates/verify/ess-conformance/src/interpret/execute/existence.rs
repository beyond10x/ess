//! Existence is read from actual typed rows, and creation follows its declared identity source.
use super::{
    input, refusal, select, value, Context, EssIr, Externals, Node, ResolvedCommand,
    ResolvedCondition, ResolvedEffect, ResolvedInstance, ResolvedOutcome, ResolvedPayloadField,
    ResolvedPayloadValue, State, Transition, Undetermined, Work,
};

pub(super) fn identity_source(outcome: &ResolvedOutcome) -> Option<&ResolvedPayloadField> {
    let subject = outcome
        .subject
        .as_ref()
        .filter(|subject| subject.effect == ResolvedEffect::Creates)?;
    let ResolvedInstance::Observed { event, field } = &subject.instance else {
        return None;
    };
    outcome
        .payload
        .iter()
        .filter(|payload| &payload.event == event)
        .flat_map(|payload| &payload.fields)
        .find(|source| source.target == field.name)
}

/// Generated fallback identities use the same bounded collision search as ordinary generation.
pub(super) fn generated(source: Option<&ResolvedPayloadField>, input: &Context<'_>) -> bool {
    match source.map(|source| &source.value) {
        None | Some(ResolvedPayloadValue::Generated) => true,
        Some(ResolvedPayloadValue::InputOrGenerated {
            field,
            otherwise: None,
            ..
        }) => input.get(field).is_none_or(|value| *value == Node::Null),
        _ => false,
    }
}

pub(super) fn identity(
    ir: &EssIr,
    source: &ResolvedPayloadField,
    input: &Context<'_>,
    work: &mut Work<'_>,
) -> Result<Node, Undetermined> {
    let identity = value(ir, source, input, work)?
        .ok_or_else(|| Undetermined::NoValue {
            what: "the declared creation identity".into(),
        })?
        .require("the declared creation identity")?;
    input::validate_typed_value(ir, &source.target_type, &identity)
        .map_err(Undetermined::Request)?;
    Ok(identity)
}

pub(super) fn existing(
    ir: &EssIr,
    command: &ResolvedCommand,
    store: &State,
    input: &Context<'_>,
    externals: &Externals,
) -> Result<Option<Transition>, Undetermined> {
    let Some(refused) = command
        .outcomes
        .iter()
        .find(|outcome| outcome.condition == ResolvedCondition::ExistingInstance)
    else {
        return Ok(None);
    };
    let creations: Vec<_> = command
        .outcomes
        .iter()
        .filter(|outcome| {
            outcome
                .subject
                .as_ref()
                .is_some_and(|subject| subject.effect == ResolvedEffect::Creates)
        })
        .collect();
    let addresses: std::collections::BTreeSet<_> = creations
        .iter()
        .map(|outcome| {
            let field = identity_source(outcome).and_then(|source| match &source.value {
                ResolvedPayloadValue::InputField { field, .. }
                | ResolvedPayloadValue::InputOrGenerated { field, .. } => Some(field.as_str()),
                _ => None,
            });
            (&outcome.subject.as_ref().expect("creation").entity, field)
        })
        .collect();
    // A shared address is known before guards, as required for related-row precedence.
    // Different addresses require the accepting branch's actual input/external selection.
    let selected = if addresses.len() <= 1 {
        creations
    } else {
        if command
            .outcomes
            .iter()
            .any(|outcome| matches!(outcome.condition, ResolvedCondition::Related { .. }))
        {
            return Err(Undetermined::NotInterpreted {
                construct:
                    "different creation addresses whose existence precedes related-row selection"
                        .into(),
            });
        }
        let facts = input::flatten(ir, command, input)
            .map_err(|why| Undetermined::Request(why.to_string()))?;
        let selected = select(
            command,
            &facts,
            &command.name,
            externals,
            &std::collections::BTreeMap::new(),
            input.caller,
            input,
            true,
            false,
        )?;
        if selected.len() > 1 {
            return Err(Undetermined::NotInterpreted {
                construct: "different creation addresses with unresolved external selection".into(),
            });
        }
        selected
    };
    for creation in selected {
        let Some(source) = identity_source(creation) else {
            continue;
        };
        let (ResolvedPayloadValue::InputField { field, .. }
        | ResolvedPayloadValue::InputOrGenerated { field, .. }) = &source.value
        else {
            continue;
        };
        let Some(identity) = input.get(field).filter(|value| **value != Node::Null) else {
            continue;
        };
        input::validate_typed_value(ir, &source.target_type, identity)
            .map_err(Undetermined::Request)?;
        let entity = ir.entity(&creation.subject.as_ref().expect("creation source").entity);
        if store.instance_typed(&entity.name, identity).is_some() {
            return refusal(ir, command, refused, store, input, None).map(Some);
        }
    }
    Ok(None)
}
