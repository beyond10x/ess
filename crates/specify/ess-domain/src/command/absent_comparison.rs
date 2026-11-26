//! A comparison over an `Optional` stored field that a row may hold absent, with no presence test
//! beside it (`docs/design/cross-record-and-stored-field-guards.md`, "A comparison over an absent
//! field").
//!
//! Against a row that holds the field absent the comparison is Unknown, which selects no branch and
//! leaves the row's membership open, so the interpreter cannot decide the guard. Four positions
//! read a row's stored fields: a row-set selector (`when_related: {entity, where}` and its
//! `forall`), a related predicate (`when_related: {via, predicate}` and its `all:` rows), a subject
//! predicate (`when_subject:`) and an `affects:` filter. In each, a comparison that reads such a
//! field is named unless a presence test of the same field stands beside it: `defined(f)` in a
//! conjunction around it, or `not defined(f)` in a disjunction around it. `defined(f) and f == x`
//! is False where `f` is absent, by the Kleene rule the evaluator applies, so the guard decides.
//!
//! A field may be absent where a branch that creates the entity leaves it unset, a branch sets it
//! `{cleared: true}` or from an `Optional` input, or nothing in the model creates the entity (its
//! rows come from outside). A field every writer sets is not named.
//!
//! From `ess/38` each is refused as `missing_declaration` ([`validate`]); below it the same finding
//! is a warning ([`advisories`]), and the answers and IR bytes do not move.

use std::collections::BTreeSet;

use ess_primitives::error::{ConstructRef, ValidationCode, ValidationError, ValidationErrors};
use ess_primitives::predicate::Predicate;

use super::{CommandSpec, Effect, OutcomeCondition, PayloadSource};
use crate::entity::EntitySpec;
use crate::spec::Specification;
use crate::system::FormatVersion;

/// The refusals from `ess/38`; nothing below it.
pub fn validate(spec: &Specification) -> ValidationErrors {
    if spec.system().format.major() < FormatVersion::V38.major() {
        return ValidationErrors::new();
    }
    findings(spec)
}

/// The same findings below `ess/38`, as advisories `ess specify validate` reports as warnings.
pub fn advisories(spec: &Specification) -> ValidationErrors {
    if spec.system().format.major() >= FormatVersion::V38.major() {
        return ValidationErrors::new();
    }
    findings(spec)
}

fn findings(spec: &Specification) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    for command in spec.commands().values() {
        for outcome in &command.outcomes {
            let at = command.site().key("outcomes").named(outcome.name.as_str());
            let mut read = |entity: Option<&EntitySpec>,
                            predicate: &Predicate,
                            position: &str,
                            site: ConstructRef| {
                if let Some(entity) = entity {
                    for (field, compare) in unguarded(spec, entity, predicate) {
                        errors.push(finding(
                            command,
                            &outcome.name,
                            entity,
                            &field,
                            compare,
                            position,
                            site.clone(),
                        ));
                    }
                }
            };
            match &outcome.condition {
                OutcomeCondition::RelatedSet {
                    selection, test, ..
                } => {
                    let entity = spec.entities().get(&selection.entity);
                    let site = at.clone().key(super::related_guard::KEY);
                    read(
                        entity,
                        &selection.filter,
                        "row-set selector `where:`",
                        site.clone(),
                    );
                    if let super::row_set::RowSetTest::Forall(predicate) = test {
                        read(entity, predicate, "row-set test `forall:`", site);
                    }
                }
                OutcomeCondition::Related {
                    via, test, also, ..
                } => {
                    let site = at.clone().key(super::related_guard::KEY);
                    if let Some(predicate) = test.predicate() {
                        let entity = super::related_guard::row_entity(spec, command, via);
                        read(entity, predicate, "related predicate", site.clone());
                    }
                    for row in also {
                        let entity = super::related_guard::row_entity(spec, command, &row.via);
                        read(entity, &row.predicate, "related predicate", site.clone());
                    }
                }
                OutcomeCondition::SubjectPredicate { predicate, .. } => {
                    let entity = super::subject_fact::reading_subject(command, outcome)
                        .and_then(|subject| spec.entities().get(&subject.entity));
                    read(
                        entity,
                        predicate,
                        "subject predicate `when_subject:`",
                        at.clone().key("when_subject"),
                    );
                }
                _ => {}
            }
            for (index, affect) in outcome.set_effects.affects.iter().enumerate() {
                if let Some(filter) = &affect.filter {
                    read(
                        spec.entities().get(&affect.entity),
                        filter,
                        "`affects:` filter",
                        at.clone().key("affects").index(index).key("filter"),
                    );
                }
            }
        }
    }
    errors
}

/// The refusal (or warning) of one comparison.
fn finding(
    command: &CommandSpec,
    outcome: &super::OutcomeName,
    entity: &EntitySpec,
    field: &str,
    compare: &Predicate,
    position: &str,
    site: ConstructRef,
) -> ValidationError {
    ValidationError::at(
        site,
        ValidationCode::MissingDeclaration,
        format!(
            "outcome `{outcome}` of `{}` compares `{field}`, an Optional field of `{}` a row may \
             hold absent, in its {position} (`{compare}`) with no `defined({field})` beside it: \
             against such a row the comparison is unknown, so the guard is never decided",
            command.name, entity.name
        ),
    )
    .with_hint(format!(
        "test presence first: `defined({field}) and {compare}`, false where `{field}` is absent"
    ))
}

/// Every comparison of `predicate` that reads a field of `entity` a row may hold absent with no
/// presence test of that field beside it, once per field and comparison.
fn unguarded<'p>(
    spec: &Specification,
    entity: &EntitySpec,
    predicate: &'p Predicate,
) -> Vec<(String, &'p Predicate)> {
    let optional = |name: &str| {
        entity
            .fields
            .iter()
            .any(|field| field.name == name && field.type_ref.is_optional())
            && may_be_absent(spec, entity, name)
    };
    let mut found = Vec::new();
    walk(predicate, &optional, &BTreeSet::new(), &mut found);
    found
}

fn walk<'p>(
    predicate: &'p Predicate,
    optional: &dyn Fn(&str) -> bool,
    guarded: &BTreeSet<String>,
    found: &mut Vec<(String, &'p Predicate)>,
) {
    match predicate {
        Predicate::All(items) => {
            let mut inner = guarded.clone();
            inner.extend(items.iter().filter_map(|item| match item {
                Predicate::Defined(path) => Some(path.namespace().to_owned()),
                _ => None,
            }));
            for item in items {
                walk(item, optional, &inner, found);
            }
        }
        Predicate::Any(items) => {
            let mut inner = guarded.clone();
            inner.extend(items.iter().filter_map(|item| match item {
                Predicate::Not(negated) => match negated.as_ref() {
                    Predicate::Defined(path) => Some(path.namespace().to_owned()),
                    _ => None,
                },
                _ => None,
            }));
            for item in items {
                walk(item, optional, &inner, found);
            }
        }
        Predicate::Not(negated) => walk(negated, optional, guarded, found),
        Predicate::Compare { .. } | Predicate::AnyOf { .. } | Predicate::NoneOf { .. } => {
            let mut fields: Vec<String> = predicate
                .fact_paths()
                .iter()
                .map(|path| path.namespace().to_owned())
                .filter(|field| optional(field) && !guarded.contains(field))
                .collect();
            fields.dedup();
            found.extend(fields.into_iter().map(|field| (field, predicate)));
        }
        _ => {}
    }
}

/// Whether a stored row of `entity` may hold `field` absent: a creating branch leaves it unset, a
/// branch clears it or sets it from an `Optional` input, or nothing in the model creates the
/// entity.
fn may_be_absent(spec: &Specification, entity: &EntitySpec, field: &str) -> bool {
    let absent_source = |command: &CommandSpec, source: &PayloadSource| match source {
        PayloadSource::Cleared => true,
        PayloadSource::InputField { field: input } => command
            .input_field(input)
            .is_some_and(|declared| declared.type_ref.is_optional()),
        _ => false,
    };
    let mut created = false;
    for command in spec.commands().values() {
        for outcome in &command.outcomes {
            if let Some(subject) = outcome.subject.as_ref().filter(|s| s.entity == entity.name) {
                if subject.effect == Effect::Creates {
                    created = true;
                    if !outcome.sets.contains_key(field) {
                        return true;
                    }
                }
                if outcome
                    .sets
                    .get(field)
                    .is_some_and(|source| absent_source(command, source))
                {
                    return true;
                }
            }
            for affect in &outcome.set_effects.affects {
                if affect.entity == entity.name
                    && affect
                        .sets
                        .get(field)
                        .is_some_and(|source| absent_source(command, source))
                {
                    return true;
                }
            }
        }
    }
    !created
}
