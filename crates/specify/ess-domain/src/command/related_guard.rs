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
//! **Stored reference (ess/22, beyond10x/ess#304).** `via` may instead be a bare `<field>`: a stored
//! field of the subject the command addresses through its input, as it was just before the branch,
//! typed as the other entity's identity or `Optional<…>` of it ("complete a task only once the task
//! its stored `blocked_by` names is `Done`"). It is read once the addressed row's existence and
//! held state have answered — `unknown_instance`, `wrong_state` — and before every accepting branch,
//! a present-related refusal included; an absent stored reference reads no row and selects no
//! `when_related` branch, as an absent Optional input does. It is refused on a command that creates
//! its subject (no row holds the field before the branch) and on one that addresses no existing
//! subject through its input; beside `wrong_state` it is refused, as an input `via` is, where an
//! accepting `when_related` branch moves the subject. Below ess/22 a stored field typed as an
//! identity is refused naming ess/22.
//!
//! **Precedence.** A missing row is answered by the `exists: false` branch before any other branch —
//! a predicate branch (its predicate is `Unknown`), an input-guarded one, the default — so an
//! `exists: false` branch carries no `when:`, and an accepting `when:` branch overlapping it is legal.
//! The one answer before it is `existing_instance:`, because the command's own identity is checked
//! before the related row is read. A command whose predicate branches have no `exists: false`
//! sibling leaves a missing row unanswered, and is refused for it.
//!
//! **Several rows (ess/22, beyond10x/ess#283).** A command may read more than one row, each named
//! by an input `via` of its own, with one `exists: false` branch at most per row. Missing rows are
//! read in the declaration order of their `exists: false` branches and the first answers, before
//! any present row's predicate; then, after the addressed row's existence and held state, the first
//! declared present-related predicate refusal whose predicate holds answers before every accepting
//! branch. Two refusals over one row stay ambiguous. A stored-field `via` stays the only row of its
//! command. Below ess/22 a second row is refused naming ess/22.
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
use ess_primitives::predicate::{Predicate, WrittenPredicate};
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

/// `when_related:` as a document writes it: one row named by an input identity (`via`), or, from
/// `ess/22`, the rows a selector selects (`entity`, `where`) with one test of them.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RawRelatedGuard {
    /// The input field that carries the other entity's identity, written `input.<field>`. Never
    /// beside `entity`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub via: Option<String>,
    /// The entity whose rows a selector selects (ess/22). Written with `where`, never beside
    /// `via`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entity: Option<crate::name::QualifiedName>,
    /// What a row of `entity` must satisfy to be selected (ess/22): its fields bare, the input
    /// under `input.`, the addressed subject under `subject.`.
    #[serde(default, rename = "where", skip_serializing_if = "Option::is_none")]
    pub filter: Option<WrittenPredicate>,
    /// `false`: the branch is taken when no row carries that identity. Written instead of
    /// `predicate`, never beside it. Over a selector (ess/22): whether any row is selected.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exists: Option<bool>,
    /// What must hold of that row's stored fields — and of the input, read under `input.` — for the
    /// branch to be taken. Written instead of `exists`, never beside it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub predicate: Option<WrittenPredicate>,
    /// Over a selector (ess/22): one comparison of the number of rows selected with a nonnegative
    /// whole number, `{eq: 1}`; the operator is `eq`, `ne`, `lt`, `lte`, `gt` or `gte`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub count: Option<super::row_set::RawCount>,
    /// Over a selector (ess/22): what every selected row satisfies; true of no rows.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub forall: Option<WrittenPredicate>,
}

/// What a `when_related:` reads, as [`RawRelatedGuard::read`] decides from its keys.
#[derive(Debug)]
pub(super) enum ReadGuard {
    /// One row, named by an identity.
    Identity(RelatedVia, RelatedTest),
    /// The rows a selector selects (ess/22).
    RowSet(super::row_set::RowSelection, super::row_set::RowSetTest),
}

impl RawRelatedGuard {
    /// The document form of a condition that reads a related row.
    pub(super) fn written(condition: &OutcomeCondition) -> Option<Self> {
        let empty = Self {
            via: None,
            entity: None,
            filter: None,
            exists: None,
            predicate: None,
            count: None,
            forall: None,
        };
        match condition {
            OutcomeCondition::Related { via, test, .. } => {
                let via = Some(via.to_string());
                Some(match test {
                    RelatedTest::Absent => Self {
                        via,
                        exists: Some(false),
                        ..empty
                    },
                    RelatedTest::Holds(predicate) => Self {
                        via,
                        predicate: Some(predicate.clone().into()),
                        ..empty
                    },
                })
            }
            OutcomeCondition::RelatedSet {
                selection, test, ..
            } => {
                let selected = Self {
                    entity: Some(selection.entity.clone()),
                    filter: Some(selection.filter.clone().into()),
                    ..empty
                };
                Some(match test {
                    super::row_set::RowSetTest::Exists(exists) => Self {
                        exists: Some(*exists),
                        ..selected
                    },
                    super::row_set::RowSetTest::Count { op, bound } => Self {
                        count: Some(
                            [(
                                op.keyword().to_owned(),
                                i64::try_from(*bound).unwrap_or(i64::MAX),
                            )]
                            .into(),
                        ),
                        ..selected
                    },
                    super::row_set::RowSetTest::Forall(predicate) => Self {
                        forall: Some(predicate.clone().into()),
                        ..selected
                    },
                })
            }
            _ => None,
        }
    }

    /// The row the guard reads and its test, or why the keys say neither.
    pub(super) fn read(self, name: &super::OutcomeName) -> Result<ReadGuard, ValidationErrors> {
        let at = |code: ValidationCode, message: String, hint: &str| {
            ValidationErrors::from(
                ValidationError::new(code, format!("outcomes.{name}.{KEY}"), message)
                    .with_hint(hint.to_owned()),
            )
        };
        let selects = self.entity.is_some()
            || self.filter.is_some()
            || self.count.is_some()
            || self.forall.is_some();
        // Which keys were written is decided above, from what the document wrote; what they say
        // is read here, before any of it is used (beyond10x/ess#448).
        let written_predicate = self.predicate.is_some();
        let (filter, predicate, forall) =
            read_predicates(name, (self.filter, self.predicate, self.forall))?;
        let Some(via) = self.via else {
            if !selects {
                return Err(at(
                    ValidationCode::MissingDeclaration,
                    format!(
                        "outcome `{name}` declares `when_related` with neither `via` nor `entity`; \
                         it reads the row an input identity names, or (ess/22) the rows a \
                         selector selects"
                    ),
                    INPUT_HINT,
                ));
            }
            if written_predicate {
                return Err(at(
                    ValidationCode::ConflictingDeclaration,
                    format!(
                        "outcome `{name}` writes `predicate` over the rows a selector selects; \
                         over a row set every selected row is tested with `forall`"
                    ),
                    "write `forall: <predicate>`",
                ));
            }
            let (selection, test) =
                super::row_set::read(name, self.entity, filter, (self.exists, self.count, forall))?;
            return Ok(ReadGuard::RowSet(selection, test));
        };
        if selects {
            return Err(at(
                ValidationCode::ConflictingDeclaration,
                format!(
                    "outcome `{name}` writes `via` beside a selector's keys; `via` names one row \
                     by an input identity, and `entity` with `where` selects rows by a predicate"
                ),
                "keep `via` with `exists` or `predicate`, or `entity` and `where` with one of \
                 `exists`, `count` or `forall`",
            ));
        }
        // `input.<field>`, or from ess/22 a bare `<field>` of the addressed subject, which
        // [`validate`] gates by format (beyond10x/ess#304). Anything else names no field.
        let via = match via.strip_prefix(super::PayloadSource::INPUT_PREFIX) {
            Some(field) if !field.is_empty() => RelatedVia::Input(field.to_owned()),
            None if is_bare_field(&via) => RelatedVia::Subject(via.clone()),
            _ => {
                return Err(at(
                    ValidationCode::TypeMismatch,
                    not_an_input(name, &via),
                    INPUT_HINT,
                ))
            }
        };
        let test = match (self.exists, predicate) {
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
        Ok(ReadGuard::Identity(via, test))
    }
}

/// The three predicates a `when_related:` may write — `where`, `predicate`, `forall` — or the
/// refusal of each one that does not parse, at its own key, together (beyond10x/ess#448).
type ReadPredicates = (Option<Predicate>, Option<Predicate>, Option<Predicate>);

fn read_predicates(
    name: &super::OutcomeName,
    (filter, predicate, forall): (
        Option<WrittenPredicate>,
        Option<WrittenPredicate>,
        Option<WrittenPredicate>,
    ),
) -> Result<ReadPredicates, ValidationErrors> {
    let mut unparsed = ValidationErrors::new();
    let mut read = |written: Option<WrittenPredicate>, key: &str| match written
        .map(|written| written.read(format!("outcomes.{name}.{KEY}.{key}")))
    {
        Some(Ok(predicate)) => Some(predicate),
        Some(Err(error)) => {
            unparsed.push(error);
            None
        }
        None => None,
    };
    let read = (
        read(filter, "where"),
        read(predicate, "predicate"),
        read(forall, "forall"),
    );
    unparsed.into_result(read)
}

/// The hint of the refusal of a `via` that names no input field, through ess/21.
const INPUT_HINT: &str = "write `via: input.<field>`, naming the input field typed as the other \
                          entity's identity";

/// The refusal message of a `via` that names no input field: the one every format through ess/21
/// gives, kept for those formats (beyond10x/ess#304).
fn not_an_input(name: &super::OutcomeName, via: &str) -> String {
    format!(
        "outcome `{name}` reads the related row through `{via}`; `when_related` reads the row an \
         input field names, one hop"
    )
}

/// Whether `written` can name a stored field of the addressed subject: one plain segment.
fn is_bare_field(written: &str) -> bool {
    !written.is_empty()
        && !written
            .chars()
            .any(|character| character == '.' || character.is_whitespace())
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

/// The field the command's related guards read, where it has any: the first declared.
pub fn via(command: &CommandSpec) -> Option<&str> {
    read_via(command).map(RelatedVia::field)
}

/// Where the command's related guards read the related row's identity, where it has any: the first
/// declared — an input field, or from ess/22 a stored field of the addressed subject.
pub fn read_via(command: &CommandSpec) -> Option<&RelatedVia> {
    command
        .outcomes
        .iter()
        .find_map(|outcome| match &outcome.condition {
            OutcomeCondition::Related { via, .. } => Some(via),
            _ => None,
        })
}

/// Every distinct `via` the command's related guards read, in the order first declared: one per
/// related row. From ess/22 a command may read more than one row through its input
/// (beyond10x/ess#283); below it, and for a stored-field `via`, a command reads one.
pub fn read_vias(command: &CommandSpec) -> Vec<&RelatedVia> {
    let mut vias: Vec<&RelatedVia> = Vec::new();
    for outcome in &command.outcomes {
        if let OutcomeCondition::Related { via, .. } = &outcome.condition {
            if !vias.contains(&via) {
                vias.push(via);
            }
        }
    }
    vias
}

/// Whether the command reads more than one related row, each named by an input field (ess/22,
/// beyond10x/ess#283). Missing rows are then answered in the declaration order of their
/// `exists: false` branches, and the present-related predicate refusals in declaration order,
/// before every accepting branch (`docs/design/cross-record-and-stored-field-guards.md`, "The
/// precedence order").
pub fn reads_several_rows(command: &CommandSpec) -> bool {
    let vias = read_vias(command);
    vias.len() > 1 && vias.iter().all(|via| matches!(via, RelatedVia::Input(_)))
}

/// The existing subject a stored-field `via` is read from (ess/22, beyond10x/ess#304): the one the
/// command's branches address through its input, as it was just before the branch. `None` where no
/// branch addresses one.
pub fn addressed_subject<'a>(
    spec: &'a Specification,
    command: &CommandSpec,
) -> Option<&'a EntitySpec> {
    super::subject_fact::common_subject(command)
        .and_then(|subject| spec.entities().get(&subject.entity))
}

/// [`related_entity`] for either kind of `via`: through a stored field of the addressed subject
/// (ess/22), the entity that field's type is the identity of — or the one a `references` relation
/// the subject declares on the field names.
pub fn related_entity_via<'a>(
    spec: &'a Specification,
    command: &CommandSpec,
    via: &RelatedVia,
) -> related_value::Referenced<'a> {
    match via {
        RelatedVia::Input(field) => related_entity(spec, command, field),
        RelatedVia::Subject(field) => {
            let Some(subject) = addressed_subject(spec, command) else {
                return related_value::Referenced::NoEntity;
            };
            let Some(stored) = subject.fields.iter().find(|held| held.name == *field) else {
                return related_value::Referenced::NoEntity;
            };
            related_value::referenced_entity(
                spec,
                identity_type(&stored.type_ref),
                Some((subject, field.as_str())),
            )
        }
    }
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

/// Whether the field `via` names — an input, or a stored field of the addressed subject — is
/// Optional, so that an absent reference reads no row and selects no `when_related` branch (ess/22,
/// beyond10x/ess#304).
fn via_is_optional(spec: &Specification, command: &CommandSpec, via: &RelatedVia) -> bool {
    match via {
        RelatedVia::Input(field) => command
            .input_field(field)
            .is_some_and(|read| read.type_ref.is_optional()),
        RelatedVia::Subject(field) => addressed_subject(spec, command).is_some_and(|subject| {
            subject
                .fields
                .iter()
                .any(|held| held.name == *field && held.type_ref.is_optional())
        }),
    }
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
        | OutcomeCondition::RelatedSet { .. }
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
/// `exists: false` branch at most per related row, one stored-field row per command, a branch a
/// scenario can reach, and no second selection strategy beside this one.
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

/// Whether two `via`s read from the same place: both the input, or both the addressed subject.
fn same_kind(one: &RelatedVia, other: &RelatedVia) -> bool {
    matches!(
        (one, other),
        (RelatedVia::Input(_), RelatedVia::Input(_))
            | (RelatedVia::Subject(_), RelatedVia::Subject(_))
    )
}

/// The refusal of a branch reading a related row other than the one `first` names.
fn second_row(
    command: &CommandSpec,
    outcome: &super::Outcome,
    via: &RelatedVia,
    first: &RelatedVia,
) -> ValidationError {
    ValidationError::at(
        site(command, outcome),
        ValidationCode::ConflictingDeclaration,
        format!(
            "outcome `{}` reads the row `{via}` names, and a sibling reads the one `{first}` \
             names; a command reads one related row",
            outcome.name
        ),
    )
    .with_hint(format!("read the related row through `{first}`"))
}

/// One `exists: false` branch at most per related row, and one wherever a predicate branch over
/// that row leaves a missing row unanswered. A second stored-field row is refused here; a second
/// row named by the input is admitted from ess/22 (beyond10x/ess#283), which [`validate`] decides
/// once the format is known, and an input `via` beside a stored-field one is refused there too.
fn one_related_row(command: &CommandSpec) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    let vias = read_vias(command);
    let Some(first) = vias.first().copied() else {
        return errors;
    };
    for outcome in &command.outcomes {
        let OutcomeCondition::Related { via, .. } = &outcome.condition else {
            continue;
        };
        if via != first
            && matches!(
                (via, first),
                (RelatedVia::Subject(_), RelatedVia::Subject(_))
            )
        {
            errors.push(second_row(command, outcome, via, first));
        }
    }
    // Several rows named by the input are answered row by row (ess/22, beyond10x/ess#283); any
    // other command is answered as the one row it reads, whatever else [`validate`] refuses of it.
    let several = reads_several_rows(command);
    let groups: Vec<Option<&RelatedVia>> = if several {
        vias.iter().copied().map(Some).collect()
    } else {
        vec![None]
    };
    for group in groups {
        errors.extend(missing_row_answers(
            command,
            group.unwrap_or(first),
            group,
            several,
        ));
    }
    errors
}

/// One `exists: false` branch at most over the row `via` names — over every related row where
/// `group` is `None` — and one wherever a predicate branch reads it.
fn missing_row_answers(
    command: &CommandSpec,
    via: &RelatedVia,
    group: Option<&RelatedVia>,
    several: bool,
) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    let reads = |outcome: &&super::Outcome| {
        matches!(&outcome.condition, OutcomeCondition::Related { via: read, .. }
                if group.is_none_or(|group| read == group))
    };
    let absent: Vec<_> = command
        .outcomes
        .iter()
        .filter(reads)
        .filter(|outcome| {
            matches!(
                &outcome.condition,
                OutcomeCondition::Related {
                    test: RelatedTest::Absent,
                    ..
                }
            )
        })
        .map(|outcome| &outcome.name)
        .collect();
    if absent.len() > 1 {
        errors.push(
            ValidationError::at(
                command.site().key("outcomes"),
                ValidationCode::ConflictingDeclaration,
                format!(
                    "outcomes {} are all `exists: false`, so `{}` declares more than one answer \
                         for a missing row{}",
                    super::join(absent.iter()),
                    command.name,
                    if several {
                        format!(" of the row `{via}` names")
                    } else {
                        String::new()
                    }
                ),
            )
            .with_hint("keep one `exists: false` branch"),
        );
    }
    let holds = command.outcomes.iter().filter(reads).any(|outcome| {
        matches!(
            &outcome.condition,
            OutcomeCondition::Related {
                test: RelatedTest::Holds(_),
                ..
            }
        )
    });
    if holds && absent.is_empty() {
        let message = if several {
            format!(
                "`{}` reads the fields of the row `{via}` names, and a missing row makes every \
                     predicate over it unknown: no branch answers when that row does not exist",
                command.name
            )
        } else {
            format!(
                "`{}` reads the related row's fields, and a missing row makes every predicate \
                     over it unknown: no branch answers when the row does not exist",
                command.name
            )
        };
        errors.push(
            ValidationError::at(
                command.site().key("outcomes"),
                ValidationCode::NonExhaustiveBranches,
                message,
            )
            .with_hint(format!(
                "declare the branch taken when it does not: `when_related: {{via: {via}, \
                     exists: false}}`"
            )),
        );
    }
    errors
}

/// The refusals answered before any other of a command reading a related row, from ess/18: the
/// Optional form's format gate, before every other refusal of the command — the `wrong_state`
/// composition included — so a document below ess/22 is told the format that admits it
/// (beyond10x/ess#304 adversary pass 1); the stored-field form's gate, at the same place for the
/// same reason; then an input `via` beside a stored-field one, which reads two rows.
fn gates(spec: &Specification, command: &CommandSpec) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    if let Some(refusal) = optional_below_ess_22(spec, command) {
        errors.push(refusal);
        return errors;
    }
    let gated = subject_via_below_ess_22(spec, command);
    if !gated.is_empty() {
        return gated;
    }
    let Some(first) = read_via(command) else {
        return errors;
    };
    for outcome in &command.outcomes {
        if let OutcomeCondition::Related { via, .. } = &outcome.condition {
            if !same_kind(via, first) {
                errors.push(second_row(command, outcome, via, first));
            }
        }
    }
    if !errors.is_empty() {
        return errors;
    }
    errors.extend(several_rows_below_ess_22(spec, command));
    errors
}

/// The refusals of a second related row named by the input below ess/22 (beyond10x/ess#283), one
/// per branch reading a row other than the first declared: the format that admits it is named.
/// Empty from ess/22, and for a command reading one row.
fn several_rows_below_ess_22(spec: &Specification, command: &CommandSpec) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    if spec.system().format.major() >= crate::system::FormatVersion::V22.major()
        || !reads_several_rows(command)
    {
        return errors;
    }
    let Some(first) = read_via(command) else {
        return errors;
    };
    for outcome in &command.outcomes {
        let OutcomeCondition::Related { via, .. } = &outcome.condition else {
            continue;
        };
        if via == first {
            continue;
        }
        errors.push(
            ValidationError::at(
                site(command, outcome),
                ValidationCode::UnsupportedFormatVersion,
                format!(
                    "outcome `{}` reads the row `{via}` names, and a sibling reads the one `{first}` \
                     names; a command guarding on more than one related row requires \
                     specification format ess/22",
                    outcome.name
                ),
            )
            .with_hint(format!(
                "declare `format: ess/22`, or read the related row through `{first}`"
            )),
        );
    }
    errors
}

/// Whether a present-related predicate refusal answers before every accepting branch: from ess/22
/// where the command declares `wrong_state` and every present-related branch refuses
/// (beyond10x/ess#282), and for every stored-field `via`, which is read from the addressed row once
/// its existence and held state have answered (ess/22, beyond10x/ess#304). The refusal of a
/// `wrong_state` branch beside a related guard where neither order is stated.
fn orders_present_refusals(
    spec: &Specification,
    command: &CommandSpec,
    via: &RelatedVia,
) -> Result<bool, ValidationError> {
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
    // An accepting related branch that moves the subject beside `wrong_state`: which answers a
    // subject the move does not start from is not designed for a stored `via` either, so it is
    // refused as for an input one (beyond10x/ess#304, correction round 1, decision F1).
    let accepting_related_move = command.outcomes.iter().any(|outcome| {
        matches!(outcome.condition, OutcomeCondition::Related { .. })
            && !outcome.is_refusal()
            && outcome
                .subject
                .as_ref()
                .is_some_and(|subject| matches!(subject.effect, super::Effect::Moves { .. }))
    });
    let orders = (matches!(via, RelatedVia::Subject(_)) && !accepting_related_move)
        || (has_wrong_state
            && spec.system().format.major() >= crate::system::FormatVersion::V22.major()
            && present_related_count > 0
            && all_present_related_refuse);
    if has_wrong_state && !orders {
        return Err(ValidationError::at(
            command.site().key("outcomes"),
            ValidationCode::ConflictingDeclaration,
            format!(
                "`{}` selects on a related row (`when_related`) and on a `wrong_state` branch; which of the two answers first is not stated",
                command.name
            ),
        )
        .with_hint(
            "guard the command on the related row alone, or split the other guard into a command of its own",
        ));
    }
    Ok(orders)
}

/// Every related row the identity type alone settles where a relation could say which entity it
/// is (beyond10x/ess#437): advisories, never refusals.
///
/// A `via` whose value a field carries — the subject field a stored `via` reads, or the field a
/// branch fills from an input `via` ([`related_value::input_carrier`]) — names its entity through
/// a relation on that field when one is declared, and otherwise through the one entity its type
/// identifies. The second is legal, and it is a relation the specification relies on and never
/// declares, so it is reported once per row, at the first branch reading it. An input `via` no
/// field carries has nowhere a relation could be declared, and is not reported; nor is a `via`
/// typed as a bare primitive, which [`crate::entity::implied_relations`] does not lint either. A
/// row-set selector (`entity`, `where`) is a query, not a relation, and is not read here.
/// Nor is a row of an entity whose identity an `updates:` rewrites (ess/23, beyond10x/ess#429):
/// a relation carrying that identity is refused, so the warning could not be silenced.
pub fn implied_relations(spec: &Specification) -> ValidationErrors {
    let mut advisories = ValidationErrors::new();
    if spec.system().format.major() < crate::system::FormatVersion::V18.major() {
        return advisories;
    }
    let rekeyed = crate::entity::rekeyed(spec);
    for command in spec.commands().values().filter(|command| uses(command)) {
        for via in read_vias(command) {
            let Some((carrier, field, via_type)) = carried(spec, command, via) else {
                continue;
            };
            // Named identity types only, as for a stored field: a bare primitive such as `Uuid`
            // is shared by many entities, and the rule does not ask about it.
            if !matches!(via_type, TypeRef::Named(_)) {
                continue;
            }
            let Some(target) =
                related_value::implied_entity(spec, via_type, (carrier, field.as_str()))
            else {
                continue;
            };
            // A re-keyed entity's identity carries no relation (beyond10x/ess#429).
            if rekeyed.contains(&target.name) {
                continue;
            }
            let Some(outcome) = command.outcomes.iter().find(|outcome| {
                matches!(&outcome.condition, OutcomeCondition::Related { via: read, .. } if read == via)
            }) else {
                continue;
            };
            advisories.push(
                ValidationError::at(
                    site(command, outcome),
                    ValidationCode::ImpliedRelation,
                    format!(
                        "`{}` reads the row `{via}` names by its type alone: `{via_type}` \
                         identifies `{}`, and no relation on `{}`'s `{field}` declares that it \
                         names a `{}`",
                        command.name, target.name, carrier.name, target.name
                    ),
                )
                .with_hint(crate::entity::implied_relation_hint(
                    &carrier.name,
                    &field,
                    &target.name,
                    crate::entity::Cardinality::One,
                    carrier.fields.iter().any(|held| {
                        held.name == field && held.type_ref == target.identity.type_ref
                    }),
                )),
            );
        }
    }
    advisories
}

/// The entity and field carrying a related guard's `via`, and the identity type it holds: the
/// addressed subject's stored field, or the field a branch fills from the input. `None` where no
/// field carries it.
fn carried<'a>(
    spec: &'a Specification,
    command: &'a CommandSpec,
    via: &RelatedVia,
) -> Option<(&'a EntitySpec, String, &'a TypeRef)> {
    match via {
        RelatedVia::Input(field) => {
            let read = command.input_field(field)?;
            let (holder, holder_field) = command.outcomes.iter().find_map(|outcome| {
                let subject = outcome
                    .subject
                    .as_ref()
                    .and_then(|subject| spec.entities().get(&subject.entity));
                related_value::input_carrier(outcome, subject, field)
            })?;
            Some((holder, holder_field, identity_type(&read.type_ref)))
        }
        RelatedVia::Subject(field) => {
            let subject = addressed_subject(spec, command)?;
            let stored = subject.fields.iter().find(|held| held.name == *field)?;
            Some((subject, field.clone(), identity_type(&stored.type_ref)))
        }
    }
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
        let gated = gates(spec, command);
        if !gated.is_empty() {
            errors.extend(gated);
            continue;
        }
        let Some(first) = read_via(command) else {
            continue;
        };
        let orders_wrong_state = match orders_present_refusals(spec, command, first) {
            Ok(orders) => orders,
            Err(refusal) => {
                errors.push(refusal);
                continue;
            }
        };
        if reads_several_rows(command) {
            errors.extend(validate_several(spec, command, types));
            continue;
        }
        let via = first;
        let entity = match via {
            RelatedVia::Input(field) => entity_or_refusal(spec, command, field, &mut errors),
            RelatedVia::Subject(field) => {
                subject_entity_or_refusal(spec, command, field, &mut errors)
            }
        };
        let Some(entity) = entity else {
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
            if via_is_optional(spec, command, via) {
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
    let RelatedVia::Input(via) = read_via(command)? else {
        return None;
    };
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

/// The refusals of a stored-field `via` below ess/22 (beyond10x/ess#304), one per branch writing
/// one: where the command could read it from ess/22 — it addresses an existing subject through its
/// input, creates none, and the subject stores the field typed as one entity's identity — the
/// format that admits it is named;
/// otherwise the refusal every earlier format gave a `via` that is not `input.<field>`, unchanged.
/// Empty from ess/22, and for a command reading its related row through its input.
fn subject_via_below_ess_22(spec: &Specification, command: &CommandSpec) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    if spec.system().format.major() >= crate::system::FormatVersion::V22.major() {
        return errors;
    }
    for outcome in &command.outcomes {
        let OutcomeCondition::Related {
            via: RelatedVia::Subject(field),
            ..
        } = &outcome.condition
        else {
            continue;
        };
        // Readable from ess/22 only where the field is typed as one entity's identity, or
        // `Optional<…>` of it: a field ess/22 refuses too keeps the refusal it always had
        // (correction round 1, decision F3).
        let readable = !creates(command)
            && addressed_subject(spec, command)
                .is_some_and(|subject| subject.fields.iter().any(|held| held.name == *field))
            && matches!(
                related_entity_via(spec, command, &RelatedVia::Subject(field.clone())),
                related_value::Referenced::Entity(_)
            );
        errors.push(if readable {
            ValidationError::at(
                site(command, outcome),
                ValidationCode::UnsupportedFormatVersion,
                format!(
                    "outcome `{}` reads the related row through `{field}`, a stored field of the \
                     subject `{}` addresses; a `when_related` guard reading a stored field of the \
                     addressed subject requires specification format ess/22",
                    outcome.name, command.name
                ),
            )
            .with_hint("declare `format: ess/22`, or read the identity from the input: `via: input.<field>`")
        } else {
            ValidationError::at(
                site(command, outcome),
                ValidationCode::TypeMismatch,
                not_an_input(&outcome.name, field),
            )
            .with_hint(INPUT_HINT)
        });
    }
    errors
}

/// The first branch of the command that creates its subject, where one does.
fn creating(command: &CommandSpec) -> Option<&super::Outcome> {
    command.outcomes.iter().find(|outcome| {
        outcome
            .subject
            .as_ref()
            .is_some_and(|subject| matches!(subject.effect, super::Effect::Creates))
    })
}

fn creates(command: &CommandSpec) -> bool {
    creating(command).is_some()
}

/// The hint's tail for a stored-field `via` the command cannot read.
const FROM_INPUT: &str = "or read the identity from the input: `via: input.<field>`";

/// The subject a stored-field `via` is read from, or the code, message and hint of the refusal
/// that says why the command has none: it creates its subject, addresses no existing subject
/// through its input, or addresses more than one.
fn addressed_or_refusal<'a>(
    spec: &'a Specification,
    command: &CommandSpec,
    field: &str,
) -> Result<&'a EntitySpec, (ValidationCode, String, String)> {
    if let Some(created) = creating(command) {
        return Err((
            ValidationCode::ConflictingDeclaration,
            format!(
                "`{}` reads the related row through `{field}`, a stored field of the subject it \
                 addresses as it was before the branch, and outcome `{}` creates its subject: no \
                 row holds that field before it",
                command.name, created.name
            ),
            format!("guard a command that moves or updates an existing subject, {FROM_INPUT}"),
        ));
    }
    let (Some(subject), Some(common)) = (
        addressed_subject(spec, command),
        super::subject_fact::common_subject(command),
    ) else {
        return Err((
            ValidationCode::UndeclaredReference,
            format!(
                "`{}` reads the related row through `{field}`, a stored field of the subject it \
                 addresses, and it addresses no existing subject through its input",
                command.name
            ),
            format!(
                "address the subject on the branch that moves or updates it (`instance: <input \
                 field>`), {FROM_INPUT}"
            ),
        ));
    };
    let several = command
        .outcomes
        .iter()
        .filter_map(|outcome| outcome.subject.as_ref())
        .filter(|other| other.surface() == super::InstanceSurface::CommandInput)
        .any(|other| other.entity != common.entity || other.instance != common.instance);
    if several {
        return Err((
            ValidationCode::ConflictingDeclaration,
            format!(
                "`{}` reads the related row through `{field}`, a stored field of the subject it \
                 addresses, and its branches address more than one subject",
                command.name
            ),
            format!("address one subject through one input field, {FROM_INPUT}"),
        ));
    }
    Ok(subject)
}

/// The entity a stored field of the addressed subject names (ess/22, beyond10x/ess#304), or the
/// refusal that says why it names none: the command creates its subject, so no row holds the field
/// before the branch; it addresses no existing subject through its input, or more than one; the
/// subject stores no such field; or the field's type is no entity's identity, nor `Optional<…>` of
/// one.
fn subject_entity_or_refusal<'a>(
    spec: &'a Specification,
    command: &CommandSpec,
    field: &str,
    errors: &mut ValidationErrors,
) -> Option<&'a EntitySpec> {
    let outcome = command
        .outcomes
        .iter()
        .find(|outcome| matches!(outcome.condition, OutcomeCondition::Related { .. }))?;
    let at = site(command, outcome);
    let refuse = |code: ValidationCode, message: String, hint: String| {
        ValidationError::at(at.clone(), code, message).with_hint(hint)
    };
    let subject = match addressed_or_refusal(spec, command, field) {
        Ok(subject) => subject,
        Err((code, message, hint)) => {
            errors.push(refuse(code, message, hint));
            return None;
        }
    };
    let from_input = FROM_INPUT;
    let Some(stored) = subject.fields.iter().find(|held| held.name == field) else {
        errors.push(refuse(
            ValidationCode::UndeclaredReference,
            format!(
                "`{field}` is not a stored field of `{}`, the subject `{}` addresses",
                subject.name, command.name
            ),
            format!(
                "stored fields: {}; {from_input}",
                super::join(subject.fields.iter().map(|held| &held.name))
            ),
        ));
        return None;
    };
    if !matches!(
        identity_type(&stored.type_ref),
        TypeRef::Primitive(_) | TypeRef::Named(_)
    ) {
        errors.push(refuse(
            ValidationCode::TypeMismatch,
            format!(
                "`{field}` of `{}` is `{}`, and `when_related` reads the one row an identity names",
                subject.name, stored.type_ref
            ),
            "store the other entity's identity, or `Optional<…>` of it".to_owned(),
        ));
        return None;
    }
    match related_entity_via(spec, command, &RelatedVia::Subject(field.to_owned())) {
        related_value::Referenced::Entity(entity) => Some(entity),
        related_value::Referenced::NoEntity => {
            errors.push(refuse(
                ValidationCode::TypeMismatch,
                format!(
                    "`{field}` of `{}` is `{}`, which is no entity's identity; `when_related` reads \
                     the row an identity names, and a lookup by any other field is out of scope",
                    subject.name, stored.type_ref
                ),
                "store the other entity's identity".to_owned(),
            ));
            None
        }
        related_value::Referenced::Ambiguous(entities) => {
            errors.push(refuse(
                ValidationCode::ConflictingDeclaration,
                format!(
                    "`{field}` of `{}` is `{}`, the identity of {}",
                    subject.name,
                    stored.type_ref,
                    entities
                        .iter()
                        .map(|entity| format!("`{}`", entity.name))
                        .collect::<Vec<_>>()
                        .join(" and ")
                ),
                format!(
                    "give the entities distinct identity types, or declare a `references` relation \
                     on `{field}` naming one"
                ),
            ));
            None
        }
    }
}

/// The entity `via` names, or the refusal that says why it names none, at the first branch reading
/// the row it names.
fn entity_or_refusal<'a>(
    spec: &'a Specification,
    command: &CommandSpec,
    via: &str,
    errors: &mut ValidationErrors,
) -> Option<&'a EntitySpec> {
    let outcome = command.outcomes.iter().find(|outcome| {
        matches!(&outcome.condition, OutcomeCondition::Related { via: read, .. } if read == via)
    })?;
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
    // The decision reads the related row with the one instant it reads its input with (ess/22,
    // A3).
    let mut environment = DomainEnvironment::new(types, &readable).with_stored_current_time();
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
fn validate_absent(
    command: &CommandSpec,
    via: &RelatedVia,
    types: &TypeRegistry,
) -> ValidationErrors {
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
                "`{}` reads the related row through the Optional `{via}`; when it is absent \
                 no row is read and no `when_related` branch is selected, and {detail}",
                command.name
            ),
        )
        .with_hint(format!(
            "declare the branch taken when `{via}` is absent: a default, or `when:` \
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
                    "with `{via}` absent, input [{input}] selects {selected} branches: {}",
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

/// A command reading more than one related row through its input (ess/22, beyond10x/ess#283): the
/// entity each `via` names, every predicate checked against its own row's entity, and the joint
/// partition of every row's fields — or its absence, for an Optional reference — with the input.
fn validate_several(
    spec: &Specification,
    command: &CommandSpec,
    types: &TypeRegistry,
) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    let mut rows: Vec<(&RelatedVia, &EntitySpec)> = Vec::new();
    for via in read_vias(command) {
        if let Some(entity) = entity_or_refusal(spec, command, via.field(), &mut errors) {
            rows.push((via, entity));
        }
    }
    if !errors.is_empty() {
        return errors;
    }
    for outcome in &command.outcomes {
        if let OutcomeCondition::Related {
            via,
            test: RelatedTest::Holds(predicate),
            ..
        } = &outcome.condition
        {
            let Some((_, entity)) = rows.iter().find(|(read, _)| *read == via) else {
                continue;
            };
            errors.extend(check(
                command,
                entity,
                types,
                predicate,
                &site(command, outcome),
            ));
        }
    }
    // A predicate the checker refused is not partitioned: its refusal is the repair.
    if errors.is_empty() {
        errors.extend(validate_joint_partition(spec, command, &rows, types));
    }
    errors
}

/// One related row's side of [`validate_joint_partition`]: the values its fields take, or `None`
/// for an absent Optional reference, and the guarded branches over that row its predicates select.
struct RowCase {
    values: Option<
        std::collections::BTreeMap<
            ess_primitives::facts::FactPath,
            ess_primitives::facts::FactValue,
        >,
    >,
    selected: Vec<usize>,
}

/// The joint partition of a command reading several related rows (ess/22, beyond10x/ess#283): every
/// row's fields — and, for an Optional reference, its absence, which selects none of that row's
/// branches — crossed with the input, under the finite prover's cap. Missing rows are answered by
/// their `exists: false` branches before any of this, and take no part in it.
///
/// Each combination is answered by exactly one branch where: none is selected and a default
/// exists; one is; or a present-related predicate refusal is, the first declared answering before
/// every accepting branch and before the refusals over other rows — but two selected refusals over
/// the row the first one reads stay ambiguous, as they do on a command reading one row.
fn validate_joint_partition(
    spec: &Specification,
    command: &CommandSpec,
    rows: &[(&RelatedVia, &EntitySpec)],
    types: &TypeRegistry,
) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    let guarded = partitioned(command);
    let default = command.default_outcome();
    let input_guards: Vec<_> = guarded
        .iter()
        .map(|outcome| super::finite::FieldGuard {
            fields: None,
            input: outcome.condition.predicate(),
        })
        .collect();
    let inputs = analyze_fields(
        default.is_some(),
        &DomainEnvironment::new(types, &[]),
        &DomainEnvironment::new(types, &command.input),
        &input_guards,
    );
    let sides: Vec<Option<Vec<RowCase>>> = rows
        .iter()
        .map(|(via, entity)| row_side(spec, command, &guarded, (via, entity), types))
        .collect();
    let total = inputs.as_ref().map(Vec::len).and_then(|cases| {
        sides
            .iter()
            .try_fold(cases, |total, side| total.checked_mul(side.as_ref()?.len()))
    });
    let (Some(inputs), Some(_)) = (
        inputs.as_ref(),
        total.filter(|total| *total <= super::finite::MAX_ASSIGNMENTS),
    ) else {
        if default.is_none() {
            errors.push(open_coverage(command));
        } else {
            errors.extend(accepting_overlaps(
                command,
                rows,
                &guarded,
                (inputs.as_deref(), &sides),
            ));
        }
        return errors;
    };
    let sides: Vec<&Vec<RowCase>> = sides.iter().flatten().collect();
    let related_of = |index: usize| match &guarded[index].condition {
        OutcomeCondition::Related { via, .. } => rows.iter().position(|(read, _)| read == &via),
        _ => None,
    };
    let combinations: usize = sides.iter().map(|side| side.len()).product();
    for input in inputs {
        for mut number in 0..combinations {
            // One case of every row, the last row's moving fastest.
            let mut combination = vec![0_usize; sides.len()];
            for (at, side) in sides.iter().enumerate().rev() {
                combination[at] = number % side.len();
                number /= side.len();
            }
            let selected: Vec<usize> = (0..guarded.len())
                .filter(|index| input.selected.contains(index))
                .filter(|index| match related_of(*index) {
                    None => !matches!(guarded[*index].condition, OutcomeCondition::Related { .. }),
                    Some(at) => sides[at][combination[at]].selected.contains(index),
                })
                .collect();
            let answering = several_selected(&selected, &guarded);
            let count = if selected.is_empty() {
                usize::from(default.is_some())
            } else {
                answering.len()
            };
            if count != 1 {
                let case: Vec<(&RelatedVia, &RowCase)> = rows
                    .iter()
                    .zip(sides.iter().zip(&combination))
                    .map(|((via, _), (side, at))| (*via, &side[*at]))
                    .collect();
                let names: Vec<&str> = answering
                    .iter()
                    .map(|index| guarded[*index].name.as_str())
                    .collect();
                errors.push(unanswered_case(command, &case, input, (count, &names)));
            }
        }
    }
    errors
}

/// The branches the joint partition decides between: every guarded one but those answered before any
/// row is read or by the addressed row's held state, and the `exists: false` branches, which answer
/// missing rows.
fn partitioned(command: &CommandSpec) -> Vec<&super::Outcome> {
    command
        .outcomes
        .iter()
        .filter(|outcome| {
            !outcome.is_unconditional()
                && outcome.condition.cause().is_none()
                && outcome.condition != OutcomeCondition::ExistingInstance
                && outcome.condition != OutcomeCondition::WrongState
                && !matches!(
                    outcome.condition,
                    OutcomeCondition::Related {
                        test: RelatedTest::Absent,
                        ..
                    }
                )
        })
        .collect()
}

/// The refusal of a command without a default whose coverage the finite prover declines.
fn open_coverage(command: &CommandSpec) -> ValidationError {
    ValidationError::at(
        command.site().key("outcomes"),
        ValidationCode::NonExhaustiveBranches,
        "related-row/input coverage is open, unsupported, or exceeds 64 joint assignments; \
         declare a genuine default",
    )
    .with_hint(
        "drop the guard from the branch that answers every other related row — usually the \
         success beside the refusal",
    )
}

/// Beside a default, where the joint cases are not enumerated — past the cap, or over a domain the
/// prover declines — the refusal of every two accepting branches over different rows that can both
/// hold: each on some case of its own row, with an input both guards admit, any side the prover
/// declines counting as one that can (beyond10x/ess#283). One request would select both, and only
/// refusals are ordered across rows, so the partition fails closed rather than admitting it.
fn accepting_overlaps(
    command: &CommandSpec,
    rows: &[(&RelatedVia, &EntitySpec)],
    guarded: &[&super::Outcome],
    (inputs, sides): (Option<&[super::finite::FieldCase]>, &[Option<Vec<RowCase>>]),
) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    let accepting: Vec<(usize, usize)> = guarded
        .iter()
        .enumerate()
        .filter_map(|(index, outcome)| match &outcome.condition {
            OutcomeCondition::Related {
                via,
                test: RelatedTest::Holds(_),
                ..
            } if !outcome.is_refusal() => rows
                .iter()
                .position(|(read, _)| read == &via)
                .map(|row| (index, row)),
            _ => None,
        })
        .collect();
    let holds = |index: usize, row: usize| {
        sides[row]
            .as_ref()
            .is_none_or(|side| side.iter().any(|case| case.selected.contains(&index)))
    };
    for (at, (first, first_row)) in accepting.iter().enumerate() {
        for (second, second_row) in &accepting[at + 1..] {
            let together = inputs.is_none_or(|inputs| {
                inputs
                    .iter()
                    .any(|case| case.selected.contains(first) && case.selected.contains(second))
            });
            if first_row != second_row
                && together
                && holds(*first, *first_row)
                && holds(*second, *second_row)
            {
                errors.push(
                    ValidationError::at(
                        command.site().key("outcomes"),
                        ValidationCode::ConflictingDeclaration,
                        format!(
                            "outcomes `{}` and `{}` accept on the rows `{}` and `{}` name, and \
                             one request may select both: past the {} joint cases validation \
                             enumerates, two accepting branches over different related rows \
                             that can both hold are refused",
                            guarded[*first].name,
                            guarded[*second].name,
                            rows[*first_row].0,
                            rows[*second_row].0,
                            super::finite::MAX_ASSIGNMENTS
                        ),
                    )
                    .with_hint(
                        "make the two acceptances exclusive through the input, or merge them \
                         into one branch",
                    ),
                );
            }
        }
    }
    errors
}

/// The finite prover over `fields` crossed with `input`: enum domains only beside a default, as the
/// one-row partition proves them.
fn analyze_fields(
    enum_only: bool,
    fields: &DomainEnvironment<'_>,
    input: &DomainEnvironment<'_>,
    guards: &[super::finite::FieldGuard<'_>],
) -> Option<Vec<super::finite::FieldCase>> {
    if enum_only {
        super::finite::analyze_enum_fields(fields, input, guards)
    } else {
        super::finite::analyze_with_fields(fields, input, guards)
    }
}

/// One row's side of [`validate_joint_partition`]: every assignment of the fields its predicates
/// read, with the guarded branches over it each selects, and its absence where its reference is
/// Optional. `None` where the prover declines.
fn row_side(
    spec: &Specification,
    command: &CommandSpec,
    guarded: &[&super::Outcome],
    (via, entity): (&RelatedVia, &EntitySpec),
    types: &TypeRegistry,
) -> Option<Vec<RowCase>> {
    let readable = readable_fields(entity, types);
    let guards: Vec<_> = guarded
        .iter()
        .map(|outcome| super::finite::FieldGuard {
            fields: match &outcome.condition {
                OutcomeCondition::Related {
                    via: read,
                    test: RelatedTest::Holds(predicate),
                    ..
                } if read == via => Some(predicate),
                _ => None,
            },
            input: None,
        })
        .collect();
    let cases = analyze_fields(
        command.default_outcome().is_some(),
        &DomainEnvironment::new(types, &readable),
        &DomainEnvironment::new(types, &[]),
        &guards,
    )?;
    let mut side: Vec<RowCase> = cases
        .into_iter()
        .map(|case| RowCase {
            values: Some(case.fields),
            selected: case
                .selected
                .into_iter()
                .filter(|index| {
                    matches!(&guarded[*index].condition,
                        OutcomeCondition::Related { via: read, .. } if read == via)
                })
                .collect(),
        })
        .collect();
    if via_is_optional(spec, command, via) {
        side.push(RowCase {
            values: None,
            selected: Vec::new(),
        });
    }
    Some(side)
}

/// The refusal of one case of [`validate_joint_partition`] that `count` branches — `names` —
/// answer rather than one.
fn unanswered_case(
    command: &CommandSpec,
    case: &[(&RelatedVia, &RowCase)],
    input: &super::finite::FieldCase,
    (count, names): (usize, &[&str]),
) -> ValidationError {
    let related = case
        .iter()
        .map(|(via, row)| match &row.values {
            None => format!("{via} absent"),
            Some(values) if values.is_empty() => format!("{via} present"),
            Some(values) => values
                .iter()
                .map(|(path, value)| format!("{via}: {path} = {value}"))
                .collect::<Vec<_>>()
                .join(", "),
        })
        .collect::<Vec<_>>()
        .join("; ");
    let supplied = input
        .input
        .iter()
        .map(|(path, value)| format!("{path} = {value}"))
        .collect::<Vec<_>>()
        .join(", ");
    ValidationError::at(
        command.site().key("outcomes"),
        if count == 0 {
            ValidationCode::NonExhaustiveBranches
        } else {
            ValidationCode::ConflictingDeclaration
        },
        format!(
            "related [{related}] and input [{supplied}] select {count} branches: {}",
            names.join(", ")
        ),
    )
}

/// The branches answering among `selected` (indices into `guarded`, in declaration order) on a
/// command reading several related rows: the first declared present-related predicate refusal,
/// together with every other selected refusal over the row it reads — which leave it ambiguous —
/// where one is selected; otherwise every selected branch.
fn several_selected(selected: &[usize], guarded: &[&super::Outcome]) -> Vec<usize> {
    let refusal_via = |index: usize| {
        let outcome = guarded[index];
        match &outcome.condition {
            OutcomeCondition::Related {
                via,
                test: RelatedTest::Holds(_),
                ..
            } if outcome.is_refusal() => Some(via),
            _ => None,
        }
    };
    let Some(first) = selected.iter().find_map(|index| refusal_via(*index)) else {
        return selected.to_vec();
    };
    selected
        .iter()
        .copied()
        .filter(|index| refusal_via(*index) == Some(first))
        .collect()
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
