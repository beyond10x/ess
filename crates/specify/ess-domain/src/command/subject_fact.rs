//! Typed admission of bounded facts on an existing subject.
use crate::{
    command::{Effect, OutcomeCondition},
    spec::Specification,
    types::{TypeBody, TypeRef, TypeRegistry},
};
use ess_primitives::error::{ValidationCode, ValidationError, ValidationErrors};

/// Check the field and enum value against the entity that owns them.
pub fn validate(spec: &Specification, types: &TypeRegistry) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    for command in spec.commands().values() {
        let mut authority = None;
        for outcome in &command.outcomes {
            let OutcomeCondition::SubjectField { field, equals, .. } = &outcome.condition else {
                continue;
            };
            let site = command
                .site()
                .key("outcomes")
                .named(outcome.name.as_str())
                .key("when_subject");
            let Some(subject) = &outcome.subject else {
                continue;
            };
            let current = (&subject.entity, &subject.instance, field);
            if authority.is_some_and(|previous| previous != current) {
                errors.push(ValidationError::at(
                    site.clone(),
                    ValidationCode::ConflictingDeclaration,
                    "subject fact branches must share one entity, identity and enum field",
                ));
            }
            authority = Some(current);
            let valid = spec
                .entities()
                .get(&subject.entity)
                .and_then(|entity| {
                    entity
                        .fields
                        .iter()
                        .find(|candidate| candidate.name == *field)
                })
                .and_then(|candidate| match &candidate.type_ref {
                    TypeRef::Named(name) => types.get(name),
                    _ => None,
                })
                .is_some_and(|named| match &named.body {
                    TypeBody::Enum { variants } => {
                        variants.iter().any(|variant| variant.name() == equals)
                    }
                    _ => false,
                });
            if !valid {
                errors.push(ValidationError::at(
                    site,
                    ValidationCode::UndeclaredReference,
                    "subject fact must name a declared enum field and one of its variants",
                ));
            }
        }
        if let Some((entity, instance, _)) = authority {
            if command.outcomes.iter().any(|outcome| {
                matches!(
                    outcome.condition,
                    OutcomeCondition::SubjectState { .. } | OutcomeCondition::StateChange { .. }
                )
            }) {
                errors.push(ValidationError::at(
                    command.site().key("outcomes"),
                    ValidationCode::ConflictingDeclaration,
                    "subject fact and lifecycle guards cannot be combined in one command",
                ));
            }
            for subject in command
                .outcomes
                .iter()
                .filter_map(|outcome| outcome.subject.as_ref())
            {
                if subject.effect == Effect::Creates
                    || &subject.entity != entity
                    || &subject.instance != instance
                {
                    errors.push(ValidationError::at(
                        command.site().key("outcomes"),
                        ValidationCode::ConflictingDeclaration,
                        "subject fact selection requires one existing subject identity",
                    ));
                }
            }
        }
    }
    errors
}
