//! Row sets: a branch guarded by the rows of an entity a `where:` predicate selects, and a value
//! read from the one row such a selector selects (source format `ess/22`,
//! `docs/design/filtered-related-reads.md`; beyond10x/ess#228, beyond10x/ess#299).
//!
//! ```yaml
//! when_related:
//!   entity: demo.jobs.Attempt
//!   where: {all: [worker_id == input.worker_id, batch_id == input.batch_id]}
//!   count: {eq: 1}            # or `exists: true|false`, or `forall: <predicate>`
//! ```
//!
//! The selector names an entity and a typed predicate over a candidate row: its declared fields,
//! its identity and its held lifecycle state as `state`, bare; the command's input under `input.`;
//! and, on a command whose branches address one existing subject through the input, that subject
//! as it was before the outcome under `subject.`. A creation has no prior subject, so a `subject.`
//! read on a command that only creates is refused. `now` reads the decision's one instant (A3).
//!
//! The test is exactly one of `exists` (a Boolean), `count` (one comparison — `eq`, `ne`, `lt`,
//! `lte`, `gt`, `gte` — with a nonnegative Integer) and `forall` (a second predicate in the same
//! environment, tested over the selected rows only). An ordinary `when:` beside it is conjunctive.
//!
//! The rows are the store immediately before the branch is selected: a row the outcome creates or
//! changes is never a candidate of its own reads. No order among the rows decides anything, and a
//! row whose membership cannot be decided is kept as a possible member, never dropped.
//!
//! `{related: {entity, where, field}}` reads `field` of the one row the selector selects, at the
//! field's declared type. Zero or several rows supply no value; an application that has a branch
//! for those cases says so with its own guards.
//!
//! **Precedence.** `existing_instance:` and the input-guarded refusals answer first; then the
//! addressed row's existence and held state (`unknown_instance:`, `wrong_state:`); then the row-set
//! refusals, in declaration order; then the accepting branches in declaration order, a row-set one
//! among them where its test holds; then the default, only where every guard before it is
//! definitely false.
//!
//! This cut reads one kind of related row per command: a row-set guard does not share a command
//! with an identity-addressed `when_related:` or a `when_subject*` guard, and is refused there by
//! name.
use std::collections::BTreeMap;

use ess_primitives::error::{ConstructRef, ValidationCode, ValidationError, ValidationErrors};
use ess_primitives::predicate::Predicate;

use super::{CommandSpec, Outcome, OutcomeCondition, PayloadSource};
use crate::entity::EntitySpec;
use crate::name::QualifiedName;
use crate::spec::Specification;
use crate::system::FormatVersion;
use crate::types::{Field, TypeRegistry};

/// The rows of `entity` that `filter` selects.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct RowSelection {
    /// The entity whose rows are candidates.
    pub entity: QualifiedName,
    /// What a candidate must satisfy to be selected.
    #[serde(rename = "where")]
    pub filter: Predicate,
}

/// The comparison a `count:` test makes with the number of selected rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CountOp {
    /// `eq`
    Eq,
    /// `ne`
    Ne,
    /// `lt`
    Lt,
    /// `lte`
    Lte,
    /// `gt`
    Gt,
    /// `gte`
    Gte,
}

impl CountOp {
    /// Every operator, in its written order.
    pub const ALL: [Self; 6] = [Self::Eq, Self::Ne, Self::Lt, Self::Lte, Self::Gt, Self::Gte];

    /// The key it is written under.
    pub fn keyword(self) -> &'static str {
        match self {
            Self::Eq => "eq",
            Self::Ne => "ne",
            Self::Lt => "lt",
            Self::Lte => "lte",
            Self::Gt => "gt",
            Self::Gte => "gte",
        }
    }

    /// The operator written `keyword`.
    pub fn parse(keyword: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|op| op.keyword() == keyword)
    }

    /// Whether `count` compares with `bound` this way. Exact: both are whole numbers.
    pub fn holds(self, count: u64, bound: u64) -> bool {
        match self {
            Self::Eq => count == bound,
            Self::Ne => count != bound,
            Self::Lt => count < bound,
            Self::Lte => count <= bound,
            Self::Gt => count > bound,
            Self::Gte => count >= bound,
        }
    }
}

/// What a row-set branch requires of the rows its selector selects.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RowSetTest {
    /// `exists: true` — some row is selected; `exists: false` — none is.
    Exists(bool),
    /// `count: {<op>: <bound>}` — the number selected compares with `bound`.
    Count {
        /// The comparison.
        op: CountOp,
        /// The nonnegative bound.
        bound: u64,
    },
    /// `forall: <predicate>` — every selected row satisfies it; true of no rows.
    Forall(Predicate),
}

impl RowSetTest {
    /// Whether exactly `count` selected rows satisfy this test, where it is decided by the count
    /// alone; `None` for `forall`, which reads the rows.
    pub fn admits_count(&self, count: u64) -> Option<bool> {
        match self {
            Self::Exists(exists) => Some((count > 0) == *exists),
            Self::Count { op, bound } => Some(op.holds(count, *bound)),
            Self::Forall(_) => None,
        }
    }

    /// The largest number this test names, below which and one past which every count-decided
    /// test is told apart.
    pub fn bound(&self) -> u64 {
        match self {
            Self::Count { bound, .. } => *bound,
            Self::Exists(_) | Self::Forall(_) => 0,
        }
    }
}

/// `count:` as written: one entry, the operator and the bound.
pub type RawCount = BTreeMap<String, i64>;

/// The selector and test of a row-set `when_related:`, read from what was written, or why not.
pub(super) fn read(
    name: &super::OutcomeName,
    entity: Option<QualifiedName>,
    filter: Option<Predicate>,
    tests: (Option<bool>, Option<RawCount>, Option<Predicate>),
) -> Result<(RowSelection, RowSetTest), ValidationErrors> {
    let at = |code: ValidationCode, message: String, hint: &str| {
        ValidationErrors::from(
            ValidationError::new(
                code,
                format!("outcomes.{name}.{}", super::related_guard::KEY),
                message,
            )
            .with_hint(hint.to_owned()),
        )
    };
    let Some(entity) = entity else {
        return Err(at(
            ValidationCode::MissingDeclaration,
            format!(
                "outcome `{name}` selects rows with `where` and names no `entity` they are rows of"
            ),
            "write `entity: <Entity>` beside `where`",
        ));
    };
    let Some(filter) = filter else {
        return Err(at(
            ValidationCode::MissingDeclaration,
            format!(
                "outcome `{name}` names the entity `{entity}` and no `where` selecting its rows"
            ),
            "write `where: <predicate over the row, input. and subject.>`",
        ));
    };
    if filter == Predicate::Always {
        return Err(at(
            ValidationCode::EmptyDeclaration,
            format!(
                "outcome `{name}` selects every row of `{entity}`: its `where` selects every row, \
                 which reads rows other users of the target made"
            ),
            "select by a field the input scopes, as `where: tenant_id == input.tenant_id`",
        ));
    }
    let (exists, count, forall) = tests;
    let written = usize::from(exists.is_some())
        + usize::from(count.is_some())
        + usize::from(forall.is_some());
    if written == 0 {
        return Err(at(
            ValidationCode::EmptyDeclaration,
            format!(
                "outcome `{name}` selects rows of `{entity}` and tests nothing of them; write one \
                 of `exists`, `count` or `forall`"
            ),
            "write `exists: true`, `count: {eq: 1}` or `forall: <predicate>`",
        ));
    }
    if written > 1 {
        return Err(at(
            ValidationCode::ConflictingDeclaration,
            format!(
                "outcome `{name}` writes more than one test of the rows it selects; a row set \
                 takes exactly one of `exists`, `count` or `forall`"
            ),
            "keep one test per branch, and split the others into branches of their own",
        ));
    }
    let test = match (exists, count, forall) {
        (Some(exists), None, None) => RowSetTest::Exists(exists),
        (None, None, Some(forall)) => RowSetTest::Forall(forall),
        (None, Some(count), None) => count_test(name, count)
            .map_err(|(message, hint)| at(ValidationCode::TypeMismatch, message, hint))?,
        _ => unreachable!("exactly one test is written"),
    };
    Ok((RowSelection { entity, filter }, test))
}

/// Whether any branch of this command is guarded by a row set.
pub fn uses(command: &CommandSpec) -> bool {
    command
        .outcomes
        .iter()
        .any(|outcome| matches!(outcome.condition, OutcomeCondition::RelatedSet { .. }))
}

/// The candidate row's readable fields: the entity's declared fields, its identity, and its held
/// lifecycle state as `state`.
pub fn candidate_fields(entity: &EntitySpec) -> Vec<Field> {
    let mut fields = super::subject_fact::readable_fields(entity, true);
    if !fields
        .iter()
        .any(|field| field.name == entity.identity.name)
    {
        fields.push(entity.identity.clone());
    }
    fields
}

/// The existing subject a selector's `subject.` reads: the one the command's branches address
/// through the input, as it was before the outcome; `None` where they address none.
pub fn subject_entity<'a>(
    spec: &'a Specification,
    command: &CommandSpec,
) -> Option<&'a EntitySpec> {
    super::subject_fact::common_subject(command)
        .filter(|subject| !matches!(subject.effect, super::Effect::Creates))
        .and_then(|subject| spec.entities().get(&subject.entity))
}

/// `predicate` checked over a candidate row of `entity`, the input under `input.` and, where the
/// command addresses one, the subject under `subject.`; `now` reads the decision's instant.
pub(super) fn check(
    spec: &Specification,
    command: &CommandSpec,
    entity: &EntitySpec,
    types: &TypeRegistry,
    predicate: &Predicate,
    at: &ConstructRef,
) -> ValidationErrors {
    let fields = candidate_fields(entity);
    super::set_effects::check_row_predicate(
        command,
        entity,
        &fields,
        subject_entity(spec, command),
        types,
        predicate,
        at,
        true,
    )
}

fn site(command: &CommandSpec, outcome: &Outcome) -> ConstructRef {
    command
        .site()
        .key("outcomes")
        .named(outcome.name.as_str())
        .key(super::related_guard::KEY)
}

/// The guards a row-set command does not share a command with in this cut, by key.
fn beside(condition: &OutcomeCondition) -> Option<&'static str> {
    match condition {
        OutcomeCondition::Related { .. } => Some("an identity-addressed `when_related`"),
        OutcomeCondition::SubjectField { .. } | OutcomeCondition::SubjectPredicate { .. } => {
            Some("a `when_subject` guard")
        }
        OutcomeCondition::SubjectState { .. } => Some("a `when_subject_state` guard"),
        OutcomeCondition::StateChange { .. } => Some("a `when_state_changes` guard"),
        OutcomeCondition::InputAbsent => Some("an `input_absent` branch"),
        _ => None,
    }
}

/// Local declaration checks, which need no registry: one kind of related read per command, one
/// unconditional branch at most, every cardinality answered where no default is declared, no branch
/// a branch before it always answers first, and a branch a scenario can reach.
pub fn validate_shape(command: &CommandSpec) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    if let Some(other) = command
        .outcomes
        .iter()
        .find_map(|outcome| beside(&outcome.condition))
    {
        errors.push(
            ValidationError::at(
                command.site().key("outcomes"),
                ValidationCode::UnsupportedConstruct,
                format!(
                    "`{}` selects on a row set (`when_related: {{entity, where, …}}`) and on \
                     {other}; one command reads one kind of related row in this cut",
                    command.name
                ),
            )
            .with_hint(
                "guard the command on the row set alone, or move the other guard to a command of \
                 its own",
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
                     by its input or the rows it selects",
                    super::join(unconditional.iter()),
                    command.name
                ),
            )
            .with_hint("give all but one of them a `when` or a `when_related`"),
        );
    }
    errors.extend(partition(command, unconditional.is_empty()));
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

/// One count from every run of counts no test tells apart: 0 and 1, each bound and the count one
/// past it. A count test changes its answer only at its bound and one past it, and `exists` only
/// at 1, so what holds at these holds everywhere — at any bound, without counting up to it.
fn breakpoints<'a>(tests: impl Iterator<Item = &'a RowSetTest>) -> Vec<u64> {
    let mut counts: Vec<u64> = tests
        .flat_map(|test| [test.bound(), test.bound().saturating_add(1)])
        .chain([0, 1])
        .collect();
    counts.sort_unstable();
    counts.dedup();
    counts
}

/// The count-decided branches over one selector, with no input guard beside them, in the order
/// they are answered: refusals in declaration order, then accepting branches in declaration order.
/// A branch every count of which an earlier one answers is unreachable; without a default, a
/// count no branch answers leaves a request the specification says nothing about, whatever guard
/// on the input or `forall:` stands beside it.
fn partition(command: &CommandSpec, no_default: bool) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    let guarded: Vec<(&Outcome, &RowSelection, &RowSetTest)> = command
        .outcomes
        .iter()
        .filter_map(|outcome| match &outcome.condition {
            OutcomeCondition::RelatedSet {
                selection,
                test,
                input: None,
            } if test.admits_count(0).is_some() => Some((outcome, selection, test)),
            _ => None,
        })
        .collect();
    let ordered: Vec<_> = guarded
        .iter()
        .filter(|(outcome, ..)| outcome.error.is_some())
        .chain(
            guarded
                .iter()
                .filter(|(outcome, ..)| outcome.error.is_none()),
        )
        .collect();
    let counts = breakpoints(guarded.iter().map(|(.., test)| *test));
    for (index, (outcome, selection, test)) in ordered.iter().enumerate() {
        let earlier: Vec<_> = ordered[..index]
            .iter()
            .filter(|(_, other, _)| other == selection)
            .collect();
        if earlier.is_empty() {
            continue;
        }
        let answered = counts.iter().all(|&count| {
            test.admits_count(count) != Some(true)
                || earlier
                    .iter()
                    .any(|(.., other)| other.admits_count(count) == Some(true))
        });
        if answered {
            errors.push(
                ValidationError::at(
                    command.site().key("outcomes").named(outcome.name.as_str()),
                    ValidationCode::UnreachableBranch,
                    format!(
                        "outcome `{}` of `{}` is never taken: every number of rows its test admits \
                         is answered by a branch over the same rows before it",
                        outcome.name, command.name
                    ),
                )
                .with_hint("widen the test, or delete the branch"),
            );
        }
    }
    if no_default {
        let selections: Vec<&RowSelection> =
            guarded.iter().map(|(_, selection, _)| *selection).collect();
        let covered = selections.iter().any(|selection| {
            counts.iter().all(|&count| {
                guarded.iter().any(|(_, other, test)| {
                    other == selection && test.admits_count(count) == Some(true)
                })
            })
        });
        // An input guard, a `forall:` or a row set with a `when:` beside it is credited with no
        // count: a request it does not take, at a count no test over one selector answers, is
        // answered by nothing.
        if !covered {
            errors.push(
                ValidationError::at(
                    command.site().key("outcomes"),
                    ValidationCode::NonExhaustiveBranches,
                    format!(
                        "every outcome of `{}` is conditional, and no one row set's tests answer \
                         every number of rows it can select",
                        command.name
                    ),
                )
                .with_hint("declare a default branch, or a test for every count"),
            );
        }
    }
    errors
}

/// Every rule that needs the whole specification: the format, the entity, and the types of each
/// selector and `forall` over the candidate row, the input and the subject.
pub fn validate(spec: &Specification, types: &TypeRegistry) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    let format = spec.system().format;
    for command in spec.commands().values().filter(|command| uses(command)) {
        for outcome in &command.outcomes {
            let OutcomeCondition::RelatedSet {
                selection, test, ..
            } = &outcome.condition
            else {
                continue;
            };
            let at = site(command, outcome);
            if format.major() < FormatVersion::V22.major() {
                errors.push(below_ess_22(at));
                continue;
            }
            let Some(entity) = spec.entities().get(&selection.entity) else {
                errors.push(undeclared(spec, &at, &selection.entity));
                continue;
            };
            errors.extend(check(spec, command, entity, types, &selection.filter, &at));
            if let RowSetTest::Forall(predicate) = test {
                errors.extend(check(spec, command, entity, types, predicate, &at));
            }
        }
    }
    errors
}

/// The refusal of a row set below `ess/22`, at `at`.
fn below_ess_22(at: ConstructRef) -> ValidationError {
    ValidationError::at(
        at,
        ValidationCode::UnsupportedFormatVersion,
        "a guard over the rows a selector selects — `when_related: {entity, where, exists | count \
         | forall}` — requires specification format ess/22",
    )
    .with_hint(
        "declare `format: ess/22`, or guard on one row named by an input identity with \
         `when_related: {via: input.<field>, …}`",
    )
}

/// `name` is no declared entity.
pub(super) fn undeclared(
    spec: &Specification,
    at: &ConstructRef,
    name: &QualifiedName,
) -> ValidationError {
    ValidationError::at(
        at.clone(),
        ValidationCode::UndeclaredReference,
        format!("`{name}` is not a declared entity"),
    )
    .with_hint(format!(
        "declared entities: {}",
        super::join(spec.entities().keys())
    ))
}

std::thread_local! {
    /// Whether a `related:` mapping's `where:` is kept for a filtered read to recognise. A parser
    /// that knows its source is below `ess/22` turns it off with [`reading_filtered_reads`], and
    /// the `where:` is then read by the nested-mapping reader at parse, refusing what it always
    /// refused (`docs/design/filtered-related-reads.md`, "Parsing precedes source-version
    /// admission"). With no format known it is kept, and [`read_below_ess_22`] restores the
    /// nested mapping at assembly.
    static FILTERED_READS: std::cell::Cell<bool> = const { std::cell::Cell::new(true) };
}

/// Runs `read` with filtered reads recognised or not, restoring what was set before.
pub(crate) fn reading_filtered_reads<T>(admitted: bool, read: impl FnOnce() -> T) -> T {
    let before = FILTERED_READS.with(|cell| cell.replace(admitted));
    let value = read();
    FILTERED_READS.with(|cell| cell.set(before));
    value
}

/// Whether filtered reads are recognised here; see [`reading_filtered_reads`].
pub(super) fn filtered_reads() -> bool {
    FILTERED_READS.with(std::cell::Cell::get)
}

/// A filtered read, `{related: {entity, where, field}}`, as the source format reads it: from
/// `ess/22` the read, below it the nested mapping the shape was before (a struct field `related`
/// whose fields `entity`, `where` and `field` hold what was written). `where` read the way the
/// nested reader read it there; a value that reader refused is refused again, naming why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Legacy {
    /// The `where:` value as a nested-mapping leaf read it before `ess/22`, or that reader's
    /// refusal.
    pub(super) filter: LegacyFilter,
    /// Why the `where:` value is not a predicate, where it is not; `None` where it is.
    pub(super) predicate: Option<String>,
}

/// The `where:` value as the nested reader below `ess/22` read it, or that reader's refusal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LegacyFilter {
    /// The value it read.
    Read(Box<PayloadSource>),
    /// Why it refused the value.
    Refused(String),
}

impl From<Result<Box<PayloadSource>, String>> for LegacyFilter {
    fn from(reading: Result<Box<PayloadSource>, String>) -> Self {
        match reading {
            Ok(read) => Self::Read(read),
            Err(refusal) => Self::Refused(refusal),
        }
    }
}

impl Legacy {
    /// A read assembled in code, which has no earlier reading.
    pub fn none() -> Self {
        Self {
            filter: LegacyFilter::Refused(
                "a filtered read assembled in code has no earlier reading".to_owned(),
            ),
            predicate: None,
        }
    }
}

/// Reads every filtered read of a document below `ess/22` back as the nested mapping it was, and
/// refuses one whose `where:` the nested reader refused. A document that names its format below
/// `ess/22` is read by the nested reader at parse (`reading_filtered_reads`); one read without
/// its format — a file naming none, or deserialised directly — recognises the exact shape, and the
/// assembled specification undoes that here, in every place a payload source stands.
pub fn read_below_ess_22(
    format: FormatVersion,
    commands: &mut BTreeMap<QualifiedName, CommandSpec>,
    errors: &mut ValidationErrors,
) {
    if format.major() >= FormatVersion::V22.major() {
        return;
    }
    for command in commands.values_mut() {
        let name = command.name.clone();
        for outcome in &mut command.outcomes {
            let at = format!("command.{name}.outcomes.{}", outcome.name);
            for (event, fields) in &mut outcome.payload {
                for (target, source) in fields.iter_mut() {
                    nested(source, &format!("{at}.payload.{event}.{target}"), errors);
                }
            }
            for (target, source) in &mut outcome.sets {
                nested(source, &format!("{at}.sets.{target}"), errors);
            }
            for (target, source) in &mut outcome.error_payload {
                nested(source, &format!("{at}.payload.{target}"), errors);
            }
            for (index, affect) in outcome.set_effects.affects.iter_mut().enumerate() {
                for (target, source) in &mut affect.sets {
                    nested(
                        source,
                        &format!("{at}.affects[{index}].sets.{target}"),
                        errors,
                    );
                }
            }
        }
    }
}

fn nested(source: &mut PayloadSource, at: &str, errors: &mut ValidationErrors) {
    use super::PayloadField;
    match source {
        PayloadSource::RelatedSelection {
            selection,
            field,
            legacy,
        } => {
            let filter = match &legacy.filter {
                LegacyFilter::Read(filter) => (**filter).clone(),
                LegacyFilter::Refused(refusal) => {
                    errors.push(ValidationError::new(
                        ValidationCode::TypeMismatch,
                        at.to_owned(),
                        refusal.clone(),
                    ));
                    PayloadSource::Generated
                }
            };
            let leaf = |target: &str, source: PayloadSource| PayloadField {
                target: target.to_owned(),
                source,
            };
            *source = PayloadSource::Struct {
                fields: vec![leaf(
                    "related",
                    PayloadSource::Struct {
                        fields: vec![
                            leaf(
                                "entity",
                                PayloadSource::parse(&selection.entity.to_string()),
                            ),
                            leaf("where", filter),
                            leaf("field", PayloadSource::parse(field)),
                        ],
                    },
                )],
            };
        }
        PayloadSource::Struct { fields } => {
            for leaf in fields {
                nested(&mut leaf.source, at, errors);
            }
        }
        PayloadSource::InputOrGenerated {
            otherwise: Some(otherwise),
            ..
        } => nested(otherwise, at, errors),
        _ => {}
    }
}

/// `count:` as the comparison it writes, or the refusal's message and hint.
fn count_test(
    name: &super::OutcomeName,
    count: RawCount,
) -> Result<RowSetTest, (String, &'static str)> {
    let [(keyword, bound)] = <[(String, i64); 1]>::try_from(count.into_iter().collect::<Vec<_>>())
        .map_err(|_| {
            (
                format!(
                    "outcome `{name}` writes `count:` with other than one comparison; it takes \
                     one, as `count: {{eq: 1}}`"
                ),
                "write `count: {<op>: <n>}` with one operator",
            )
        })?;
    let Some(op) = CountOp::parse(&keyword) else {
        return Err((
            format!(
                "outcome `{name}` compares the count with `{keyword}`; `count:` takes `eq`, `ne`, \
                 `lt`, `lte`, `gt` or `gte`"
            ),
            "write `count: {eq: 1}`",
        ));
    };
    let Ok(bound) = u64::try_from(bound) else {
        return Err((
            format!(
                "outcome `{name}` compares the count with `{bound}`; a count is compared with a \
                 nonnegative whole number"
            ),
            "write a bound of 0 or more",
        ));
    };
    Ok(RowSetTest::Count { op, bound })
}
