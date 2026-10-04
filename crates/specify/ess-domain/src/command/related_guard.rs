//! `when_related:` — a branch guarded by a row of another entity, named by an identity the command's
//! input carries (source format `ess/18`, beyond10x/ess#211).
//!
//! Two shapes, one key:
//!
//! * `{via: input.<field>, exists: false}` is taken when no row of the entity whose identity
//!   `<field>` carries exists;
//! * `{via: input.<field>, predicate: <p>}` is taken when that row exists and `<p>` — over the row's
//!   declared stored fields, from `ess/20` its held lifecycle state as `state` (beyond10x/ess#229),
//!   and, as in a `when_subject` predicate, the input under `input.` — holds.
//!
//! One hop, and only by identity: the row is the one whose identity is the input's value, so the
//! entity is the one `<field>`'s type is the identity of ([`super::related_value::referenced_entity`],
//! the resolution `{related: {via, field}}` uses). A lookup by any other field is a query — rule 2 of
//! `docs/design/cross-record-and-stored-field-guards.md` — and stays out of scope.
//!
//! **Optional reference (ess/22, beyond10x/ess#304).** The input may be `Optional<…>` of the
//! identity; the guard is then checked only when present. An absent reference reads no row and
//! selects no `when_related` branch — it is not a missing row — so the branches that read no
//! related row (a `when:` over the input, or the default) must answer it exactly once. A present
//! reference is read as a required one is. Below ess/22 the Optional form is refused naming ess/22.
//!
//! **Precedence.** A missing row is answered by the `exists: false` branch before any other branch —
//! a predicate branch (its predicate is `Unknown`), an input-guarded one, the default — so an
//! `exists: false` branch carries no `when:`, and an accepting `when:` branch overlapping it is legal.
//! The one answer before it is `existing_instance:`, because the command's own identity is checked
//! before the related row is read. A command whose predicate branches have no `exists: false`
//! sibling leaves a missing row unanswered, and is refused for it.
//!
//! The guard reads a row the command does not address, so it composes with any subject a branch
//! names — a `creates:` included — and with a branch that names none. From ess/22 a predicate
//! refusal composes with `wrong_state:` in one command: addressed-row existence and held state
//! answer first, then the one present-related refusal selected by the partition, then acceptance.
//! It does not compose with the other guards that read the addressed subject (`when_subject`,
//! `when_subject_state`, `when_state_changes`) or whether it exists (`unknown_instance`), nor with
//! `input_absent`. `existing_instance:` sits beside it in one command (never on one branch), with
//! the precedence above.
use super::{related_value, CommandSpec, OutcomeCondition, RelatedVia};
use crate::{
    entity::EntitySpec, expression::DomainEnvironment, spec::Specification, types::TypeRef,
    types::TypeRegistry,
};
use ess_primitives::error::{ConstructRef, ValidationCode, ValidationError, ValidationErrors};
use ess_primitives::predicate::Predicate;
use std::fmt::Write as _;

/// The key an author writes.
pub const KEY: &str = "when_related";

/// What a `when_related:` branch requires of the row the input names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelatedTest {
    /// `exists: false`: no row carries the identity.
    Absent,
    /// `predicate:` — the row exists and this predicate over its stored fields (and `input.`) holds.
    Holds(Predicate),
}

impl RelatedTest {
    /// The predicate over the row, where the branch has one.
    pub fn predicate(&self) -> Option<&Predicate> {
        match self {
            Self::Absent => None,
            Self::Holds(predicate) => Some(predicate),
        }
    }
}

/// `when_related:` as a document writes it.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RawRelatedGuard {
    /// The input field that carries the other entity's identity, written `input.<field>`.
    pub via: String,
    /// `false`: the branch is taken when no row carries that identity. Written instead of
    /// `predicate`, never beside it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exists: Option<bool>,
    /// What must hold of that row's stored fields — and of the input, read under `input.` — for the
    /// branch to be taken. Written instead of `exists`, never beside it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub predicate: Option<Predicate>,
}

impl RawRelatedGuard {
    /// The document form of a condition that reads a related row.
    pub(super) fn written(condition: &OutcomeCondition) -> Option<Self> {
        let OutcomeCondition::Related { via, test, .. } = condition else {
            return None;
        };
        let via = via.to_string();
        Some(match test {
            RelatedTest::Absent => Self {
                via,
                exists: Some(false),
                predicate: None,
            },
            RelatedTest::Holds(predicate) => Self {
                via,
                exists: None,
                predicate: Some(predicate.clone()),
            },
        })
    }

    /// The input field `via` names and the test, or why the key says neither.
    pub(super) fn read(
        self,
        name: &super::OutcomeName,
    ) -> Result<(RelatedVia, RelatedTest), ValidationErrors> {
        let at = |code: ValidationCode, message: String, hint: &str| {
            ValidationErrors::from(
                ValidationError::new(code, format!("outcomes.{name}.{KEY}"), message)
                    .with_hint(hint.to_owned()),
            )
        };
        let Some(field) = self
            .via
            .strip_prefix(super::PayloadSource::INPUT_PREFIX)
            .filter(|field| !field.is_empty())
        else {
            return Err(at(
                ValidationCode::TypeMismatch,
                format!(
                    "outcome `{name}` reads the related row through `{}`; `when_related` reads the \
                     row an input field names, one hop",
                    self.via
                ),
                "write `via: input.<field>`, naming the input field typed as the other entity's \
                 identity",
            ));
        };
        let test = match (self.exists, self.predicate) {
            (Some(false), None) => RelatedTest::Absent,
            (None, Some(predicate)) => RelatedTest::Holds(predicate),
            (Some(true), None) => {
                return Err(at(
                    ValidationCode::ConflictingDeclaration,
                    format!(
                        "outcome `{name}` declares `exists: true`; the branch taken while the row \
                         exists is the default beside `exists: false`, or a `predicate:` over it"
                    ),
                    "write `exists: false`, or a `predicate:` over the related row's fields",
                ))
            }
            (Some(_), Some(_)) => {
                return Err(at(
                    ValidationCode::ConflictingDeclaration,
                    format!(
                    "outcome `{name}` declares both `exists` and `predicate` in `when_related`; \
                         a missing row selects only `exists: false`, and a predicate reads a row \
                         that exists"
                ),
                    "split them into two branches",
                ))
            }
            (None, None) => {
                return Err(at(
                    ValidationCode::EmptyDeclaration,
                    format!("outcome `{name}` declares `when_related` with no test"),
                    "write `exists: false` or a `predicate:` over the related row's fields",
                ))
            }
        };
        Ok((RelatedVia::Input(field.to_owned()), test))
    }
}

/// Refuses every condition key written beside `when_related:` but `when:`: on one branch each is a
/// second selection authority, and a replay takes the answer its origin gave.
pub(super) fn alone(raw: &super::RawOutcome) -> Result<(), ValidationErrors> {
    let beside = [
        (raw.when_subject.is_some(), "when_subject"),
        (raw.when_subject_state.is_some(), "when_subject_state"),
        (raw.when_state_changes.is_some(), "when_state_changes"),
        (raw.external.is_some(), "external"),
        (raw.wrong_state, "wrong_state"),
        (raw.unknown_instance, "unknown_instance"),
        (raw.input_absent, "input_absent"),
        (raw.existing_instance, "existing_instance"),
        (raw.replays.is_some(), "replays"),
    ];
    let Some((_, other)) = beside.iter().find(|(written, _)| *written) else {
        return Ok(());
    };
    Err(super::outcome_conflict(
        &raw.name,
        KEY,
        format!(
            "outcome `{}` declares `{KEY}` beside `{other}`; a branch has one selection authority",
            raw.name
        ),
        "keep the related guard and an optional `when:` over the input, or the other key",
    ))
}

/// Refuses `when:` beside `exists: false`: a missing related row is answered by that branch before
/// any other (beyond10x/ess#211, adversary pass 1), so an input guard on it would leave the missing
/// rows it refutes with no answer at all.
pub(super) fn absent_alone(
    raw: &super::RawOutcome,
    test: &RelatedTest,
) -> Result<(), ValidationErrors> {
    if *test != RelatedTest::Absent || raw.when.is_none() {
        return Ok(());
    }
    Err(super::outcome_conflict(
        &raw.name,
        KEY,
        format!(
            "outcome `{}` declares `when:` beside `when_related: {{exists: false}}`; a missing \
             related row is answered by the `exists: false` branch before any other, whatever the \
             input",
            raw.name
        ),
        "drop the `when:`: guard the input on the branches taken while the row exists",
    ))
}

/// Whether any branch of this command reads a related row.
pub fn uses(command: &CommandSpec) -> bool {
    command
        .outcomes
        .iter()
        .any(|outcome| matches!(outcome.condition, OutcomeCondition::Related { .. }))
}

/// The input field the command's related guards read, where it has any: the first declared.
pub fn via(command: &CommandSpec) -> Option<&str> {
    command
        .outcomes
        .iter()
        .find_map(|outcome| match &outcome.condition {
            OutcomeCondition::Related { via, .. } => Some(via.field()),
            _ => None,
        })
}

/// The entity whose row a command's related guards read: the one the input `via`'s type is the
/// identity of, or — where several entities share that identity type — the one a `references`
/// relation on the field a creating branch stores the input in names.
pub fn related_entity<'a>(
    spec: &'a Specification,
    command: &CommandSpec,
    via: &str,
) -> related_value::Referenced<'a> {
    let Some(read) = command.input_field(via) else {
        return related_value::Referenced::NoEntity;
    };
    let carrier = command.outcomes.iter().find_map(|outcome| {
        let subject = outcome
            .subject
            .as_ref()
            .and_then(|subject| spec.entities().get(&subject.entity));
        related_value::input_carrier(outcome, subject, via)
    });
    related_value::referenced_entity(
        spec,
        identity_type(&read.type_ref),
        carrier
            .as_ref()
            .map(|(entity, field)| (*entity, field.as_str())),
    )
}

/// The identity type an input `via` carries: its declared type, with one `Optional` wrapper removed
/// (ess/22, beyond10x/ess#304). An absent Optional reference names no row and is read as no lookup;
/// a present one names the row of the entity whose identity is the wrapped type.
fn identity_type(declared: &TypeRef) -> &TypeRef {
    match declared {
        TypeRef::Optional(inner) => inner,
        other => other,
    }
}

/// Whether the input `via` names is Optional, so that an absent reference reads no row and selects
/// no `when_related` branch (ess/22, beyond10x/ess#304).
fn via_is_optional(command: &CommandSpec, via: &str) -> bool {
    command
        .input_field(via)
        .is_some_and(|read| read.type_ref.is_optional())
}

/// The conditions that read the addressed subject or its existence, which a related guard does not
/// share a command with.
fn other_authority(condition: &OutcomeCondition) -> Option<&'static str> {
    match condition {
        OutcomeCondition::SubjectField { .. } | OutcomeCondition::SubjectPredicate { .. } => {
            Some("a `when_subject` guard")
        }
        OutcomeCondition::SubjectState { .. } => Some("a `when_subject_state` guard"),
        OutcomeCondition::StateChange { .. } => Some("a `when_state_changes` guard"),
        OutcomeCondition::UnknownInstance => Some("an `unknown_instance` branch"),
        OutcomeCondition::InputAbsent => Some("an `input_absent` branch"),
        // The command's own identity is checked before the related row is read (beyond10x/ess#211,
        // adversary pass 1): an `existing_instance:` refusal answers first, and reads no related row.
        OutcomeCondition::When(_)
        | OutcomeCondition::ExistingInstance
        | OutcomeCondition::WrongState
        | OutcomeCondition::Related { .. }
        | OutcomeCondition::Otherwise
        | OutcomeCondition::External { .. }
        | OutcomeCondition::ExternalWhen { .. } => None,
    }
}

fn site(command: &CommandSpec, outcome: &super::Outcome) -> ConstructRef {
    command
        .site()
        .key("outcomes")
        .named(outcome.name.as_str())
        .key(KEY)
}

/// Local declaration checks, which need no registry and no entity: one default at most, one
/// `exists: false` branch at most, one related row per command, a branch a scenario can reach, and
/// no second selection strategy beside this one.
pub fn validate_shape(command: &CommandSpec) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    if let Some(other) = command
        .outcomes
        .iter()
        .find_map(|outcome| other_authority(&outcome.condition))
    {
        errors.push(
            ValidationError::at(
                command.site().key("outcomes"),
                ValidationCode::ConflictingDeclaration,
                format!(
                    "`{}` selects on a related row (`when_related`) and on {other}; which of the two \
                     answers first is not stated",
                    command.name
                ),
            )
            .with_hint(
                "guard the command on the related row alone, or split the other guard into a \
                 command of its own",
            ),
        );
        return errors;
    }
    let unconditional: Vec<_> = command
        .outcomes
        .iter()
        .filter(|outcome| outcome.is_unconditional())
        .map(|outcome| &outcome.name)
        .collect();
    if unconditional.len() > 1 {
        errors.push(
            ValidationError::at(
                command.site().key("outcomes"),
                ValidationCode::ConflictingDeclaration,
                format!(
                    "outcomes {} are all unconditional, so the result of `{}` is not determined \
                     by its input or the related row",
                    super::join(unconditional.iter()),
                    command.name
                ),
            )
            .with_hint("give all but one of them a `when` or a `when_related`"),
        );
    }
    errors.extend(one_related_row(command));
    let reachable = command
        .outcomes
        .iter()
        .filter(|outcome| {
            outcome.is_testable_from_input()
                || outcome.test_strategy() == super::TestStrategy::ArrangeRelatedRow
        })
        .count();
    errors.extend(command.validate_reachable_and_wrong_state(reachable));
    errors
}

/// One related row per command, one `exists: false` branch at most, and one wherever a predicate
/// branch leaves a missing row unanswered.
fn one_related_row(command: &CommandSpec) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    let first = via(command).unwrap_or_default();
    let mut absent = Vec::new();
    for outcome in &command.outcomes {
        let OutcomeCondition::Related { via, test, .. } = &outcome.condition else {
            continue;
        };
        if via.field() != first {
            errors.push(
                ValidationError::at(
                    site(command, outcome),
                    ValidationCode::ConflictingDeclaration,
                    format!(
                        "outcome `{}` reads the row `{via}` names, and a sibling reads the one \
                         `input.{first}` names; a command reads one related row",
                        outcome.name
                    ),
                )
                .with_hint(format!("read the related row through `input.{first}`")),
            );
        }
        if *test == RelatedTest::Absent {
            absent.push(&outcome.name);
        }
    }
    if absent.len() > 1 {
        errors.push(
            ValidationError::at(
                command.site().key("outcomes"),
                ValidationCode::ConflictingDeclaration,
                format!(
                    "outcomes {} are all `exists: false`, so `{}` declares more than one answer for \
                     a missing row",
                    super::join(absent.iter()),
                    command.name
                ),
            )
            .with_hint("keep one `exists: false` branch"),
        );
    }
    let holds = command.outcomes.iter().any(|outcome| {
        matches!(
            &outcome.condition,
            OutcomeCondition::Related {
                test: RelatedTest::Holds(_),
                ..
            }
        )
    });
    if holds && absent.is_empty() {
        errors.push(
            ValidationError::at(
                command.site().key("outcomes"),
                ValidationCode::NonExhaustiveBranches,
                format!(
                    "`{}` reads the related row's fields, and a missing row makes every predicate \
                     over it unknown: no branch answers when the row does not exist",
                    command.name
                ),
            )
            .with_hint(format!(
                "declare the branch taken when it does not: `when_related: {{via: input.{first}, \
                 exists: false}}`"
            )),
        );
    }
    errors
}

/// The format gate, the entity `via` names, the predicates against that entity's fields, and the
/// joint related-field × input partition.
pub fn validate(spec: &Specification, types: &TypeRegistry) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    for command in spec.commands().values().filter(|command| uses(command)) {
        if spec.system().format.major() < crate::system::FormatVersion::V18.major() {
            for outcome in &command.outcomes {
                if matches!(outcome.condition, OutcomeCondition::Related { .. }) {
                    errors.push(
                        ValidationError::at(
                            site(command, outcome),
                            ValidationCode::UnsupportedFormatVersion,
                            "a guard over a related row (`when_related`) requires specification \
                             format ess/18",
                        )
                        .with_hint("declare `format: ess/18`"),
                    );
                }
            }
            continue;
        }
        // The Optional form's own format gate answers before every other refusal of the command,
        // the `wrong_state` composition below included, so a document below ess/22 is told the
        // format that admits it (beyond10x/ess#304 adversary pass 1).
        if let Some(refusal) = optional_below_ess_22(spec, command) {
            errors.push(refusal);
            continue;
        }
        let has_wrong_state = command
            .outcomes
            .iter()
            .any(|outcome| matches!(outcome.condition, OutcomeCondition::WrongState));
        let (present_related_count, all_present_related_refuse) = command
            .outcomes
            .iter()
            .filter(|outcome| {
                matches!(
                    outcome.condition,
                    OutcomeCondition::Related {
                        test: RelatedTest::Holds(_),
                        ..
                    }
                )
            })
            .fold((0_usize, true), |(count, all_refuse), outcome| {
                (count + 1, all_refuse && outcome.is_refusal())
            });
        let orders_wrong_state = has_wrong_state
            && spec.system().format.major() >= crate::system::FormatVersion::V22.major()
            && present_related_count > 0
            && all_present_related_refuse;
        if has_wrong_state && !orders_wrong_state {
            errors.push(
                ValidationError::at(
                    command.site().key("outcomes"),
                    ValidationCode::ConflictingDeclaration,
                    format!(
                        "`{}` selects on a related row (`when_related`) and on a `wrong_state` branch; which of the two answers first is not stated",
                        command.name
                    ),
                )
                .with_hint(
                    "guard the command on the related row alone, or split the other guard into a command of its own",
                ),
            );
            continue;
        }
        let Some(via) = via(command) else {
            continue;
        };
        let Some(entity) = entity_or_refusal(spec, command, via, &mut errors) else {
            continue;
        };
        let mut checked = ValidationErrors::new();
        for outcome in &command.outcomes {
            if let OutcomeCondition::Related {
                test: RelatedTest::Holds(predicate),
                ..
            } = &outcome.condition
            {
                checked.extend(check(
                    command,
                    entity,
                    types,
                    predicate,
                    &site(command, outcome),
                ));
            }
        }
        // A predicate the checker refused is not partitioned: its refusal is the repair.
        if checked.is_empty() {
            checked.extend(validate_partition(
                command,
                entity,
                types,
                orders_wrong_state,
            ));
            if via_is_optional(command, via) {
                checked.extend(validate_absent(command, via, types));
            }
        }
        errors.extend(checked);
    }
    errors
}

/// The refusal of an Optional reference below ess/22 (beyond10x/ess#304), at the first related
/// branch, where the command reads its related row through an Optional input; `None` otherwise.
fn optional_below_ess_22(spec: &Specification, command: &CommandSpec) -> Option<ValidationError> {
    if spec.system().format.major() >= crate::system::FormatVersion::V22.major() {
        return None;
    }
    let via = via(command)?;
    let read = command
        .input_field(via)
        .filter(|read| read.type_ref.is_optional())?;
    let outcome = command
        .outcomes
        .iter()
        .find(|outcome| matches!(outcome.condition, OutcomeCondition::Related { .. }))?;
    Some(
        ValidationError::at(
            site(command, outcome),
            ValidationCode::UnsupportedFormatVersion,
            format!(
                "`input.{via}` is `{}`; a `when_related` guard that reads through an Optional \
                 input, checked only when present, requires specification format ess/22",
                read.type_ref
            ),
        )
        .with_hint("declare `format: ess/22`, or make the input required"),
    )
}

/// The entity `via` names, or the refusal that says why it names none.
fn entity_or_refusal<'a>(
    spec: &'a Specification,
    command: &CommandSpec,
    via: &str,
    errors: &mut ValidationErrors,
) -> Option<&'a EntitySpec> {
    let outcome = command
        .outcomes
        .iter()
        .find(|outcome| matches!(outcome.condition, OutcomeCondition::Related { .. }))?;
    let at = site(command, outcome);
    let Some(read) = command.input_field(via) else {
        errors.push(
            ValidationError::at(
                at,
                ValidationCode::UndeclaredReference,
                format!("`{via}` is not an input of `{}`", command.name),
            )
            .with_hint(format!(
                "declared input: {}",
                super::join(command.input.iter().map(|field| &field.name))
            )),
        );
        return None;
    };
    if !matches!(
        identity_type(&read.type_ref),
        TypeRef::Primitive(_) | TypeRef::Named(_)
    ) {
        errors.push(
            ValidationError::at(
                at,
                ValidationCode::TypeMismatch,
                format!(
                    "`input.{via}` is `{}`, and `when_related` reads the one row an identity names",
                    read.type_ref
                ),
            )
            .with_hint(
                "read an input typed as the other entity's identity, or as `Optional<…>` of it \
                 (ess/22)",
            ),
        );
        return None;
    }
    match related_entity(spec, command, via) {
        related_value::Referenced::Entity(entity) => Some(entity),
        related_value::Referenced::NoEntity => {
            errors.push(
                ValidationError::at(
                    at,
                    ValidationCode::TypeMismatch,
                    format!(
                        "`input.{via}` is `{}`, which is no entity's identity; `when_related` reads \
                         the row an identity names, and a lookup by any other field is out of scope",
                        read.type_ref
                    ),
                )
                .with_hint("read an input typed as the other entity's identity"),
            );
            None
        }
        related_value::Referenced::Ambiguous(entities) => {
            errors.push(
                ValidationError::at(
                    at,
                    ValidationCode::ConflictingDeclaration,
                    format!(
                        "`input.{via}` is `{}`, the identity of {}",
                        read.type_ref,
                        entities
                            .iter()
                            .map(|entity| format!("`{}`", entity.name))
                            .collect::<Vec<_>>()
                            .join(" and ")
                    ),
                )
                .with_hint(
                    "give the entities distinct identity types, or store the input in a field of \
                     the created subject that a `references` relation carries",
                ),
            );
            None
        }
    }
}

/// Whether the document's format reads the related row's held state as `state` (ess/20,
/// beyond10x/ess#229). Below it the path is refused with the format it needs.
fn admits_state(types: &TypeRegistry) -> bool {
    super::subject_fact::admits_state_from(types, crate::system::FormatVersion::V20)
}

/// The fields a `when_related` predicate reads: the related entity's stored fields and, from
/// `ess/20`, its held state as `state`, typed by its lifecycle — the list a `when_subject`
/// predicate reads from `ess/18` (beyond10x/ess#204), for another row.
fn readable_fields(entity: &EntitySpec, types: &TypeRegistry) -> Vec<crate::types::Field> {
    super::subject_fact::readable_fields(entity, admits_state(types))
}

/// The expression checker, over the related entity's declared stored fields — from `ess/20` with
/// its held state as `state` — and the command's input under `input.`: the environment a
/// `when_subject` predicate is checked in, for another row.
fn check(
    command: &CommandSpec,
    entity: &EntitySpec,
    types: &TypeRegistry,
    predicate: &Predicate,
    site: &ConstructRef,
) -> ValidationErrors {
    let owner = site.render();
    let mut errors = ValidationErrors::new();
    if super::subject_fact::reads_state(predicate) && !admits_state(types) {
        errors.push(
            ValidationError::at(
                site.clone(),
                ValidationCode::UnsupportedFormatVersion,
                "reading the related row's held lifecycle state as `state` in a `when_related` \
                 predicate requires specification format ess/20",
            )
            .with_hint("declare `format: ess/20`"),
        );
        return errors;
    }
    let readable = readable_fields(entity, types);
    let mut environment = DomainEnvironment::new(types, &readable);
    let reads_input = !entity
        .fields
        .iter()
        .any(|field| field.name == super::subject_fact::INPUT_NAMESPACE);
    if reads_input {
        environment = environment.with_input(&command.input);
    }
    let checked = crate::expression::check_predicate(&environment, predicate, &owner);
    for error in &checked.errors {
        let mut diagnostic = error.validation_error();
        if let Some(path) = &error.path {
            if error.segment.as_deref() == Some(path.namespace())
                && !entity
                    .fields
                    .iter()
                    .any(|field| field.name == path.namespace())
            {
                write!(
                    diagnostic.message,
                    "; `when_related` reads the stored fields of `{}`, and `{}` is not one",
                    entity.name,
                    path.namespace()
                )
                .expect("writing to a String");
                diagnostic.hint = Some(format!(
                    "stored fields: {}; the input is read as `input.<field>`, and the held \
                     lifecycle state as `state` (ess/20)",
                    super::join(entity.fields.iter().map(|field| &field.name))
                ));
            }
        }
        errors.push(diagnostic);
    }
    errors
}

/// The joint related-field × input partition over the rows that exist, under the finite prover's
/// caps; the `exists: false` branch answers the rows that do not, and takes no part in it.
///
/// Where the prover declines — an open domain, a comparison with the input — the command needs a
/// genuine default, as an open input guard does.
fn validate_partition(
    command: &CommandSpec,
    entity: &EntitySpec,
    types: &TypeRegistry,
    orders_present_refusals: bool,
) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    let guarded: Vec<&super::Outcome> = command
        .outcomes
        .iter()
        .filter(|outcome| {
            !outcome.is_unconditional()
                && outcome.condition.cause().is_none()
                // Answered before every row-reading branch: by the command's own identity.
                && outcome.condition != OutcomeCondition::ExistingInstance
                // From ess/22 this is selected later, against the addressed row's held state.
                && outcome.condition != OutcomeCondition::WrongState
                && !matches!(
                    outcome.condition,
                    OutcomeCondition::Related {
                        test: RelatedTest::Absent,
                        ..
                    }
                )
        })
        .collect();
    let guards: Vec<_> = guarded
        .iter()
        .map(|outcome| super::finite::FieldGuard {
            fields: match &outcome.condition {
                OutcomeCondition::Related { test, .. } => test.predicate(),
                _ => None,
            },
            input: outcome.condition.predicate(),
        })
        .collect();
    let default = command.default_outcome();
    let readable = readable_fields(entity, types);
    let analyze = if default.is_some() {
        super::finite::analyze_enum_fields
    } else {
        super::finite::analyze_with_fields
    };
    let Some(cases) = analyze(
        &DomainEnvironment::new(types, &readable),
        &DomainEnvironment::new(types, &command.input),
        &guards,
    ) else {
        if default.is_none() {
            errors.push(
                ValidationError::at(
                    command.site().key("outcomes"),
                    ValidationCode::NonExhaustiveBranches,
                    "related-row/input coverage is open, unsupported, or exceeds 64 joint \
                     assignments; declare a genuine default",
                )
                .with_hint(
                    "drop the guard from the branch that answers every other related row — usually \
                     the success beside the refusal",
                ),
            );
        }
        return errors;
    };
    for case in cases {
        let selected = selected_count(
            &case.selected,
            &guarded,
            default.is_some(),
            orders_present_refusals,
        );
        if selected == 1 {
            continue;
        }
        let assignment =
            |values: &std::collections::BTreeMap<_, ess_primitives::facts::FactValue>| {
                values
                    .iter()
                    .map(|(path, value)| format!("{path} = {value}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            };
        errors.push(ValidationError::at(
            command.site().key("outcomes"),
            if selected == 0 {
                ValidationCode::NonExhaustiveBranches
            } else {
                ValidationCode::ConflictingDeclaration
            },
            format!(
                "related [{}] and input [{}] select {selected} branches: {}",
                assignment(&case.fields),
                assignment(&case.input),
                case.selected
                    .iter()
                    .map(|index| guarded[*index].name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        ));
    }
    errors
}

/// The case an absent Optional `via` leaves (ess/22, beyond10x/ess#304): no row is read and no
/// `when_related` branch is selected, so exactly one of the branches that read no related row —
/// an input-guarded `when:` branch or the default — must answer every admitted input. Held state
/// (`wrong_state:`) and the command's own identity (`existing_instance:`) answer before it, as they
/// do for a present reference.
fn validate_absent(command: &CommandSpec, via: &str, types: &TypeRegistry) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    let guarded: Vec<&super::Outcome> = command
        .outcomes
        .iter()
        .filter(|outcome| matches!(outcome.condition, OutcomeCondition::When(_)))
        .collect();
    let default = command.default_outcome();
    let uncovered = |detail: String| {
        ValidationError::at(
            command.site().key("outcomes"),
            ValidationCode::NonExhaustiveBranches,
            format!(
                "`{}` reads the related row through the Optional `input.{via}`; when it is absent \
                 no row is read and no `when_related` branch is selected, and {detail}",
                command.name
            ),
        )
        .with_hint(format!(
            "declare the branch taken when `input.{via}` is absent: a default, or `when:` \
             branches over the input that cover it"
        ))
    };
    if guarded.is_empty() {
        if default.is_none() {
            errors.push(uncovered("no other branch answers".to_owned()));
        }
        return errors;
    }
    let guards: Vec<_> = guarded
        .iter()
        .map(|outcome| super::finite::FieldGuard {
            fields: None,
            input: outcome.condition.predicate(),
        })
        .collect();
    let analyze = if default.is_some() {
        super::finite::analyze_enum_fields
    } else {
        super::finite::analyze_with_fields
    };
    let Some(cases) = analyze(
        &DomainEnvironment::new(types, &[]),
        &DomainEnvironment::new(types, &command.input),
        &guards,
    ) else {
        if default.is_none() {
            errors.push(uncovered(
                "input coverage is open, unsupported, or exceeds 64 assignments without a default"
                    .to_owned(),
            ));
        }
        return errors;
    };
    for case in cases {
        let selected = if case.selected.is_empty() {
            usize::from(default.is_some())
        } else {
            case.selected.len()
        };
        if selected == 1 {
            continue;
        }
        let input = case
            .input
            .iter()
            .map(|(path, value)| format!("{path} = {value}"))
            .collect::<Vec<_>>()
            .join(", ");
        if selected == 0 {
            errors.push(uncovered(format!("input [{input}] selects no branch")));
        } else {
            errors.push(ValidationError::at(
                command.site().key("outcomes"),
                ValidationCode::ConflictingDeclaration,
                format!(
                    "with `input.{via}` absent, input [{input}] selects {selected} branches: {}",
                    case.selected
                        .iter()
                        .map(|index| guarded[*index].name.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            ));
        }
    }
    errors
}

fn selected_count(
    selected: &[usize],
    guarded: &[&super::Outcome],
    has_default: bool,
    orders_present_refusals: bool,
) -> usize {
    if selected.is_empty() {
        return usize::from(has_default);
    }
    let related_refusals = selected
        .iter()
        .filter(|index| {
            let outcome = guarded[**index];
            outcome.is_refusal()
                && matches!(
                    outcome.condition,
                    OutcomeCondition::Related {
                        test: RelatedTest::Holds(_),
                        ..
                    }
                )
        })
        .count();
    // From ess/22 one selected present-related predicate refusal answers before every accepting
    // branch. Two such refusals remain ambiguous, as in earlier formats: the new order introduces
    // no author-declared tie-break between them.
    if orders_present_refusals && related_refusals == 1 {
        1
    } else {
        selected.len()
    }
}
