//! An `updates:` whose `sets:` writes the entity's identity re-keys the record (ess/23,
//! beyond10x/ess#429, `docs/design/identity-changing-updates.md`).
//!
//! The row is read under the identity `instance:` names and comes to rest under the identity
//! `sets:` writes; every field `sets:` does not name is carried over, and the old identity names
//! nothing afterwards. No keyword is added: the construct already parsed, and before `ess/23` its
//! synthesized suite required the opposite of a rename, so below `ess/23` it is refused naming the
//! format that gives it a meaning.
//!
//! | rule | code |
//! |---|---|
//! | the identity write under a header older than `ess/23` | [`UnsupportedFormatVersion`](ValidationCode::UnsupportedFormatVersion) |
//! | the identity write beside `compensates: true` | [`UnsupportedConstruct`](ValidationCode::UnsupportedConstruct) |
//! | the identity write on the updating branch of a create-or-update pair | [`UnsupportedConstruct`](ValidationCode::UnsupportedConstruct) |
//! | the identity write on an entity whose identity is a struct | [`UnsupportedConstruct`](ValidationCode::UnsupportedConstruct) |
//! | the identity write on an entity a declared relation carries | [`UnsupportedConstruct`](ValidationCode::UnsupportedConstruct) |
//! | the identity write with no declared collision answer | [`MissingDeclaration`](ValidationCode::MissingDeclaration) |
//!
//! The collision answer is a sibling refusal guarded by exactly
//! `when_related: {entity: <the entity>, where: <identity> == input.<field>, exists: true}`, where
//! `input.<field>` is the input the identity is written from. Its rows are the store before the
//! branch, so the record's own row is one of them: a rename to the identity it already carries is
//! the collision. A narrower selector or an input guard beside it would leave a collision
//! unanswered, and the store would have to hold two rows under one identity.
//!
//! `affects:` and `instances:` keep refusing an identity write (`set_effects::identity_set`): rows
//! that all come to hold one identity are one row.

use ess_primitives::error::{ConstructRef, ValidationCode, ValidationError, ValidationErrors};
use ess_primitives::predicate::{CompareKind, CompareOp, Operand, Predicate};

use super::row_set::RowSetTest;
use super::{CommandSpec, Effect, Outcome, OutcomeCondition, PayloadSource};
use crate::entity::{EntitySpec, RelationKind};
use crate::spec::Specification;
use crate::system::FormatVersion;

/// The identity `outcome` writes on the row it updates, where it writes one: the entity and the
/// source of the new identity.
pub fn written<'a>(
    spec: &'a Specification,
    outcome: &'a Outcome,
) -> Option<(&'a EntitySpec, &'a PayloadSource)> {
    let subject = outcome
        .subject
        .as_ref()
        .filter(|subject| subject.effect == Effect::Updates)?;
    let entity = spec.entities().get(&subject.entity)?;
    outcome
        .sets
        .get(&entity.identity.name)
        .map(|source| (entity, source))
}

/// Every identity write in `spec`, held to the rules in the [module documentation](self).
pub(crate) fn validate(spec: &Specification) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    let format = spec.system().format;
    for command in spec.commands().values() {
        for outcome in &command.outcomes {
            let Some((entity, source)) = written(spec, outcome) else {
                continue;
            };
            let at = command.site().key("outcomes").named(outcome.name.as_str());
            if let Some(refused) = refusal(spec, (command, outcome), entity, &at, format) {
                errors.push(refused);
                continue;
            }
            if !collision_answered(command, entity, source) {
                errors.push(unanswered(outcome, entity, source, &at));
            }
        }
    }
    errors
}

/// The refusal of an identity write that is not admitted where it is written, if it is not.
fn refusal(
    spec: &Specification,
    (command, outcome): (&CommandSpec, &Outcome),
    entity: &EntitySpec,
    at: &ConstructRef,
    format: FormatVersion,
) -> Option<ValidationError> {
    let identity = &entity.identity.name;
    let write = at.clone().key("sets").named(identity);
    if format.major() < FormatVersion::V23.major() {
        return Some(
            ValidationError::at(
                write,
                ValidationCode::UnsupportedFormatVersion,
                format!(
                    "`sets:` writes `{identity}`, the identity of `{}`, on an `updates:`; a write \
                     that re-keys the record requires specification format ess/23",
                    entity.name
                ),
            )
            .with_hint(format!(
                "declare `format: ess/23` to rename the record, or drop `{identity}` from `sets:`"
            )),
        );
    }
    let unsupported = |message: String, hint: &str| {
        Some(
            ValidationError::at(write.clone(), ValidationCode::UnsupportedConstruct, message)
                .with_hint(hint.to_owned()),
        )
    };
    // A struct identity is out of this cut: its collision answer would compare an aggregate, which
    // `==` refuses, so the rename is refused here, by name.
    let structured = match &entity.identity.type_ref {
        crate::types::TypeRef::Named(name) => spec
            .system()
            .types
            .get(name)
            .is_some_and(|named| matches!(named.body, crate::types::TypeBody::Struct { .. })),
        _ => false,
    };
    if structured {
        return unsupported(
            format!(
                "outcome `{}` writes `{identity}`, the identity of `{}`, which is a struct; a \
                 re-key of a struct identity is not supported",
                outcome.name, entity.name
            ),
            "rename only an entity whose identity is a scalar or a newtype over one, or drop the \
             identity from `sets:`",
        );
    }
    if outcome.compensates {
        return unsupported(
            format!(
                "outcome `{}` declares `compensates: true` and writes `{identity}`, the identity \
                 of `{}`; a compensating refusal changes the row its `instance:` names, and a \
                 re-key leaves that identity naming nothing",
                outcome.name, entity.name
            ),
            "rename the record on a branch that succeeds, or drop the identity from `sets:`",
        );
    }
    if let Some(creation) = command.outcomes.iter().find(|other| {
        other.condition == OutcomeCondition::UnknownInstance
            && other
                .subject
                .as_ref()
                .is_some_and(|subject| subject.effect == Effect::Creates)
    }) {
        return unsupported(
            format!(
                "outcome `{}` writes `{identity}`, the identity of `{}`, beside `{}`, the \
                 `unknown_instance:` creation of a create-or-update pair; the pair reads \
                 `instance:` as the identity that persists, and a re-key leaves it naming nothing",
                outcome.name, entity.name, creation.name
            ),
            "rename the record with a command of its own, or drop the identity from `sets:`",
        );
    }
    if let Some((owner, relation)) = carried(spec, entity) {
        // An `owns` carrier is on the target; a `references` carrier on the declaring entity.
        let carrier = match relation.kind {
            RelationKind::Owns => &relation.target,
            RelationKind::References => owner,
        };
        return unsupported(
            format!(
                "`{}` is carried by `{carrier}.{}`, through the relation `{}` declared on \
                 `{owner}` (`{}`); a re-key of its identity would leave every carrier naming a \
                 record that is gone",
                entity.name,
                relation.via,
                relation.name,
                relation.kind.as_str(),
            ),
            "rename only an entity no `owns` or `references` relation carries, or drop the \
             identity from `sets:`",
        );
    }
    None
}

/// The first declared relation whose carrying field holds `entity`'s identity: an `owns` relation
/// `entity` declares, whose carrier is on its target, or a `references` relation targeting it.
fn carried<'a>(
    spec: &'a Specification,
    entity: &EntitySpec,
) -> Option<(
    &'a crate::name::QualifiedName,
    &'a crate::entity::RelationSpec,
)> {
    spec.entities().values().find_map(|declaring| {
        declaring
            .relations
            .iter()
            .find(|relation| match relation.kind {
                RelationKind::Owns => declaring.name == entity.name,
                RelationKind::References => relation.target == entity.name,
            })
            .map(|relation| (&declaring.name, relation))
    })
}

/// Whether `command` declares the refusal a collision on the written identity takes.
fn collision_answered(command: &CommandSpec, entity: &EntitySpec, source: &PayloadSource) -> bool {
    let PayloadSource::InputField { field } = source else {
        return false;
    };
    command.outcomes.iter().any(|outcome| {
        outcome.is_refusal()
            && matches!(
                &outcome.condition,
                OutcomeCondition::RelatedSet {
                    selection,
                    test: RowSetTest::Exists(true),
                    input: None,
                } if selection.entity == entity.name
                    && selects_identity(&selection.filter, &entity.identity.name, field)
            )
    })
}

/// Whether `filter` is exactly `<identity> == input.<field>`, either way round.
fn selects_identity(filter: &Predicate, identity: &str, field: &str) -> bool {
    let Predicate::Compare {
        left,
        op: CompareOp::Eq,
        right,
        kind: CompareKind::Value,
    } = filter
    else {
        return false;
    };
    let fact = |operand: &Operand| match operand {
        Operand::Fact(path) => Some(path.to_string()),
        _ => None,
    };
    let input = format!("input.{field}");
    match (fact(left), fact(right)) {
        (Some(left), Some(right)) => {
            (left == identity && right == input) || (left == input && right == identity)
        }
        _ => false,
    }
}

/// The refusal of an identity write whose collision answer is not declared.
fn unanswered(
    outcome: &Outcome,
    entity: &EntitySpec,
    source: &PayloadSource,
    at: &ConstructRef,
) -> ValidationError {
    let identity = &entity.identity.name;
    let hint = match source {
        PayloadSource::InputField { field } => format!(
            "declare a refusal beside it: `when_related: {{entity: {}, where: {identity} == \
             input.{field}, exists: true}}` with an `error:`",
            entity.name
        ),
        _ => format!(
            "write `{identity}` from an input field, `{identity}: input.<field>`, and declare a \
             refusal guarded by `when_related: {{entity: {}, where: {identity} == input.<field>, \
             exists: true}}` with an `error:`",
            entity.name
        ),
    };
    ValidationError::at(
        at.clone(),
        ValidationCode::MissingDeclaration,
        format!(
            "outcome `{}` writes `{identity}`, the identity of `{}`, and re-keys the record; the \
             command declares no answer for a new identity another record already carries",
            outcome.name, entity.name
        ),
    )
    .with_hint(hint)
}
