//! Set effects over filtered instances (source format `ess/16`, beyond10x/ess#167 and #175,
//! `docs/design/set-effects-over-filtered-instances.md`).
//!
//! Two constructs, both about rows an outcome changes that no input names:
//!
//! * **`instances: {where: <predicate>}`** on a `moves:` or `updates:` outcome, instead of
//!   `instance:`: every stored row of the entity the predicate selects. The predicate is the
//!   stored-field grammar of `when_subject:` over that entity's fields, with `input.<field>`
//!   operands. A `moves:` skips a selected row resting outside the transition's `from` states; no
//!   selected row at all is an accepted outcome. `{count: changed}` in the outcome's `payload:` is
//!   the number of rows it changed.
//! * **`affects:`** on an outcome with one existing subject: a list of `{entity, where, sets}`,
//!   each changing every row of `entity` its `where` selects. `where` reads the entity's fields,
//!   `input.<field>` and `subject.<field>` — the subject as it was before the outcome. Where
//!   `entity` is the subject's own, the subject itself is not among the rows.
//!
//! `sets:` of either takes a literal, `input.<field>`, `{input: …, else: …}`, `{generated: true}`
//! or `{cleared: true}`; a source that reads one row (`{subject: …}`, `{related: …}`,
//! `{increment: …}`) or the caller is refused by name in this first cut, and so is a move inside
//! `affects:`.

use std::collections::BTreeMap;
use std::fmt;

use ess_primitives::error::{ConstructRef, ValidationCode, ValidationError, ValidationErrors};
use ess_primitives::predicate::Predicate;

use super::{CommandSpec, Effect, Outcome, OutcomeName, PayloadSource, PayloadTable, Subject};
use crate::entity::EntitySpec;
use crate::expression::{CurrentTimeAdmission, DomainEnvironment, Shape, TypeEnvironment};
use crate::name::QualifiedName;
use crate::system::FormatVersion;
use crate::types::{Field, Primitive, TypeRef, TypeRegistry};
use crate::Specification;

/// The root an `affects:` filter reads the subject under: `subject.team`.
pub const SUBJECT_NAMESPACE: &str = "subject";

/// What `instances:` holds as written.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RawInstances {
    /// The rows selected: a predicate over the entity's stored fields, with `input.` operands.
    #[serde(rename = "where")]
    pub filter: Predicate,
}

/// One `affects:` entry as written.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RawAffect {
    /// The entity whose rows change.
    pub entity: QualifiedName,
    /// The rows selected, over the entity's fields, `input.` and `subject.`.
    #[serde(rename = "where")]
    pub filter: Predicate,
    /// What every selected row comes to hold.
    #[serde(default, skip_serializing_if = "PayloadTable::is_empty")]
    pub sets: PayloadTable,
    /// Read so it can be refused by name: a move inside `affects:` is not in this cut.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub moves: Option<QualifiedName>,
}

/// The rows a `moves:` or `updates:` outcome changes, selected by a filter rather than named.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct SetSubject {
    /// The entity whose rows change.
    pub entity: QualifiedName,
    /// [`Effect::Moves`] or [`Effect::Updates`], and nothing else.
    #[serde(flatten)]
    pub effect: Effect,
    /// The rows selected.
    pub filter: Predicate,
}

/// A secondary effect of an outcome with one subject, on every row a filter selects.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Affect {
    /// The entity whose rows change.
    pub entity: QualifiedName,
    /// The rows selected, over the entity's fields, `input.` and `subject.`.
    pub filter: Predicate,
    /// What every selected row comes to hold.
    pub sets: BTreeMap<String, PayloadSource>,
}

/// Both constructs of one outcome; empty on every outcome that declares neither.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize)]
pub struct SetEffects {
    /// `instances:`, with the verb it was written beside.
    pub instances: Option<SetSubject>,
    /// `affects:`, in the order written.
    pub affects: Vec<Affect>,
}

impl SetEffects {
    /// `true` when the outcome declares neither construct.
    pub fn is_empty(&self) -> bool {
        self.instances.is_none() && self.affects.is_empty()
    }
}

fn refusal(
    name: &OutcomeName,
    key: &str,
    code: ValidationCode,
    message: String,
    hint: &str,
) -> ValidationErrors {
    ValidationErrors::from(
        ValidationError::new(code, format!("outcomes.{name}.{key}"), message)
            .with_hint(hint.to_owned()),
    )
}

/// The verbs an outcome declared beside `instances:`, by key.
pub(super) struct Verbs<'a> {
    /// `creates`, `deletes` or `preserves`, where the outcome wrote one of them.
    pub(super) other: Option<&'static str>,
    pub(super) instance: bool,
    pub(super) moves: &'a mut Option<QualifiedName>,
    pub(super) updates: &'a mut Option<QualifiedName>,
}

/// `instances:` as the set subject it declares, taking the `moves:` or `updates:` it was written
/// beside so [`super::subject_of`] sees no single subject; or the refusal of a combination.
pub(super) fn set_subject(
    name: &OutcomeName,
    instances: Option<RawInstances>,
    verbs: &mut Verbs<'_>,
) -> Result<Option<SetSubject>, ValidationErrors> {
    let Some(instances) = instances else {
        return Ok(None);
    };
    if verbs.instance {
        return Err(refusal(
            name,
            "instances",
            ValidationCode::ConflictingDeclaration,
            format!(
                "outcome `{name}` declares both `instance:` and `instances:`; one names the row an \
                 input carries, the other selects rows by a filter"
            ),
            "keep `instance:` for one named row, or `instances: {where: …}` for every row a \
             filter selects",
        ));
    }
    if verbs.other == Some("creates") {
        return Err(refusal(
            name,
            "instances",
            ValidationCode::ConflictingDeclaration,
            format!(
                "outcome `{name}` creates an instance and declares `instances:`; a creation brings \
                 one row into existence and selects none"
            ),
            "drop `instances:`; a creation names its new identity with `instance:`",
        ));
    }
    if let Some(verb) = verbs.other {
        return Err(refusal(
            name,
            "instances",
            ValidationCode::UnsupportedConstruct,
            format!(
                "outcome `{name}` {verb} an entity and declares `instances:`; a set subject is \
                 admitted beside `moves:` and `updates:` only"
            ),
            "name one row with `instance:`, or change the rows with `moves:` or `updates:`",
        ));
    }
    if let Some(entity) = verbs.updates.take() {
        return Ok(Some(SetSubject {
            entity,
            effect: Effect::Updates,
            filter: instances.filter,
        }));
    }
    let Some(qualified) = verbs.moves.take() else {
        return Err(refusal(
            name,
            "instances",
            ValidationCode::MissingDeclaration,
            format!(
                "outcome `{name}` declares `instances:` and no `moves:` or `updates:`, so the \
                 filter selects rows nothing changes"
            ),
            "say what the branch does to the rows: `moves: <Entity>.<transition>` or \
             `updates: <Entity>`",
        ));
    };
    let Some(entity) = qualified.namespace() else {
        return Err(refusal(
            name,
            "moves",
            ValidationCode::MissingDeclaration,
            format!(
                "outcome `{name}` moves `{qualified}`, which names no entity; a move is written as \
                 the entity followed by the transition"
            ),
            "write it as `billing.invoice.Invoice.settle`",
        ));
    };
    Ok(Some(SetSubject {
        entity,
        effect: Effect::Moves {
            transition: qualified.local().to_owned(),
        },
        filter: instances.filter,
    }))
}

/// `affects:` as written, with a move inside refused by name and a field set twice refused.
pub(super) fn affects(
    name: &OutcomeName,
    written: Vec<RawAffect>,
) -> Result<Vec<Affect>, ValidationErrors> {
    let mut errors = ValidationErrors::new();
    let mut affects = Vec::with_capacity(written.len());
    for (index, raw) in written.into_iter().enumerate() {
        if let Some(moved) = &raw.moves {
            errors.extend(refusal(
                name,
                &format!("affects[{index}].moves"),
                ValidationCode::UnsupportedConstruct,
                format!(
                    "outcome `{name}` moves `{moved}` inside `affects:`; a secondary effect sets \
                     fields in this cut and takes no transition"
                ),
                "declare `sets:` on the affected rows, or move them with a command of their own",
            ));
            continue;
        }
        let mut sets = BTreeMap::new();
        for entry in raw.sets.0 {
            if sets.contains_key(&entry.target) {
                errors.extend(refusal(
                    name,
                    &format!("affects[{index}].sets.{}", entry.target),
                    ValidationCode::DuplicateDeclaration,
                    format!("`{}` is set more than once", entry.target),
                    "keep the one that is true and delete the other",
                ));
                continue;
            }
            sets.insert(entry.target, entry.source);
        }
        affects.push(Affect {
            entity: raw.entity,
            filter: raw.filter,
            sets,
        });
    }
    errors.into_result(affects)
}

/// `affects:` is admitted beside one existing subject that `moves:` or `updates:`.
pub(super) fn affects_beside(
    name: &OutcomeName,
    affects: &[Affect],
    subject: Option<&Subject>,
    instances: bool,
) -> Result<(), ValidationErrors> {
    if affects.is_empty() {
        return Ok(());
    }
    if instances {
        return Err(refusal(
            name,
            "affects",
            ValidationCode::UnsupportedConstruct,
            format!(
                "outcome `{name}` declares `affects:` beside `instances:`; a secondary effect is \
                 admitted beside one subject in this cut"
            ),
            "give the set outcome's own `sets:` what it needs, or split the branch",
        ));
    }
    match subject.map(|subject| &subject.effect) {
        Some(Effect::Moves { .. } | Effect::Updates) => Ok(()),
        Some(effect) => Err(refusal(
            name,
            "affects",
            ValidationCode::UnsupportedConstruct,
            format!(
                "outcome `{name}` {} its subject and declares `affects:`; a secondary effect is \
                 admitted beside `moves:` and `updates:`, whose subject exists before the outcome",
                effect.verb()
            ),
            "move `affects:` to the branch that moves or updates an existing row",
        )),
        None => Err(refusal(
            name,
            "affects",
            ValidationCode::MissingDeclaration,
            format!(
                "outcome `{name}` declares `affects:` and no subject; `subject.` in its filter \
                 reads nothing"
            ),
            "name the branch's subject with `moves:` or `updates:` and `instance:`",
        )),
    }
}

/// The keys `effects` is written back with: the verb the set subject takes, `instances:` and
/// `affects:`. `moves` and `updates` are the single subject's, and kept where there is no set one.
pub(super) fn written(
    effects: SetEffects,
    moves: Option<QualifiedName>,
    updates: Option<QualifiedName>,
) -> (
    Option<QualifiedName>,
    Option<QualifiedName>,
    Option<RawInstances>,
    Vec<RawAffect>,
) {
    let affects = effects
        .affects
        .into_iter()
        .map(|affect| RawAffect {
            entity: affect.entity,
            filter: affect.filter,
            sets: PayloadTable(
                affect
                    .sets
                    .into_iter()
                    .map(|(target, source)| super::PayloadField { target, source })
                    .collect(),
            ),
            moves: None,
        })
        .collect();
    let Some(set) = effects.instances else {
        return (moves, updates, None, affects);
    };
    let instances = Some(RawInstances { filter: set.filter });
    match set.effect {
        Effect::Moves { transition } => (
            Some(set.entity.child(&transition)),
            updates,
            instances,
            affects,
        ),
        _ => (moves, Some(set.entity), instances, affects),
    }
}

/// The transitions set moves take, for the lifecycle-cause check.
pub(crate) fn performed(
    commands: &BTreeMap<QualifiedName, CommandSpec>,
) -> impl Iterator<Item = (&QualifiedName, &str)> {
    commands
        .values()
        .flat_map(|command| &command.outcomes)
        .filter(|outcome| !outcome.is_refusal())
        .filter_map(|outcome| outcome.set_effects.instances.as_ref())
        .filter_map(|set| set.effect.transition().map(|move_| (&set.entity, move_)))
}

/// The assignments one outcome declares, with the entity each is over and the key it is written
/// under: its own `sets:` (over the subject, or over the set subject), then each `affects:` entry.
pub(crate) fn assignments(
    outcome: &Outcome,
) -> Vec<(&QualifiedName, &BTreeMap<String, PayloadSource>, String)> {
    let own = outcome
        .subject
        .as_ref()
        .map(|subject| &subject.entity)
        .or(outcome
            .set_effects
            .instances
            .as_ref()
            .map(|set| &set.entity));
    own.map(|entity| (entity, &outcome.sets, "sets".to_owned()))
        .into_iter()
        .chain(
            outcome
                .set_effects
                .affects
                .iter()
                .enumerate()
                .map(|(index, affect)| {
                    (
                        &affect.entity,
                        &affect.sets,
                        format!("affects[{index}].sets"),
                    )
                }),
        )
        .collect()
}

/// The keys a set outcome and an `affects:` entry are written with, for the format refusal.
fn used_keys(outcome: &Outcome) -> Vec<&'static str> {
    let mut keys = Vec::new();
    if outcome.set_effects.instances.is_some() {
        keys.push("instances");
    }
    if !outcome.set_effects.affects.is_empty() {
        keys.push("affects");
    }
    keys
}

/// Every rule that needs the whole specification: the format, the entity and transition, the
/// filter's reads, the sources admitted, and `{count: changed}`.
pub(crate) fn validate(spec: &Specification) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    let format = spec.system().format;
    let types = &spec.system().types;
    for command in spec.commands().values() {
        for outcome in &command.outcomes {
            let site = command.site().key("outcomes").named(outcome.name.as_str());
            if format.major() < FormatVersion::V16.major() {
                for key in used_keys(outcome) {
                    errors.push(
                        ValidationError::at(
                            site.clone().key(key),
                            ValidationCode::UnsupportedFormatVersion,
                            format!("`{key}:` requires specification format ess/16"),
                        )
                        .with_hint("declare `format: ess/16`"),
                    );
                }
                continue;
            }
            if let Some(set) = &outcome.set_effects.instances {
                let at = site.clone().key("instances");
                errors.extend(set_branch(outcome, &at));
                if let Some(entity) = declared(spec, &set.entity, &at, &mut errors) {
                    if let Some(transition) = set.effect.transition() {
                        if entity.states.transition(transition).is_none() {
                            errors.push(ValidationError::at(
                                site.clone().key("moves"),
                                ValidationCode::UndeclaredReference,
                                format!(
                                    "outcome `{}` of `{}` takes `{transition}`, which `{}` does \
                                     not declare as a transition",
                                    outcome.name, command.name, set.entity
                                ),
                            ));
                        }
                    }
                    errors.extend(check_filter(command, entity, None, types, &set.filter, &at));
                    errors.extend(identity_set(
                        entity,
                        &site.clone().key("sets"),
                        &outcome.sets,
                    ));
                }
                errors.extend(one_row_sources(&site.clone().key("sets"), &outcome.sets));
            }
            let subject = outcome
                .subject
                .as_ref()
                .and_then(|subject| spec.entities().get(&subject.entity));
            for (index, affect) in outcome.set_effects.affects.iter().enumerate() {
                let at = site.clone().key("affects").index(index);
                if let Some(entity) =
                    declared(spec, &affect.entity, &at.clone().key("entity"), &mut errors)
                {
                    errors.extend(check_filter(
                        command,
                        entity,
                        subject,
                        types,
                        &affect.filter,
                        &at.clone().key("where"),
                    ));
                    errors.extend(super::value_expression::validate_affect(
                        spec,
                        command,
                        outcome,
                        entity,
                        &affect.sets,
                        &at,
                    ));
                    errors.extend(identity_set(entity, &at.clone().key("sets"), &affect.sets));
                }
                errors.extend(one_row_sources(&at.clone().key("sets"), &affect.sets));
            }
        }
    }
    errors
}

/// A set outcome accepts, and is selected by its input alone or as the default: a refusal changes
/// no row, and a condition reading one subject has none to read.
fn set_branch(outcome: &Outcome, at: &ConstructRef) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    if let Some(error) = &outcome.error {
        errors.push(
            ValidationError::at(
                at.clone(),
                ValidationCode::RefusalMutatedState,
                format!(
                    "outcome `{}` reports `{error}` and changes every row `instances:` selects; a \
                     refused command changes nothing",
                    outcome.name
                ),
            )
            .with_hint(
                "drop `instances:` from the refusal, and declare it on the branch that succeeds",
            ),
        );
    }
    if !matches!(
        outcome.condition,
        super::OutcomeCondition::When(_) | super::OutcomeCondition::Otherwise
    ) {
        errors.push(
            ValidationError::at(
                at.clone(),
                ValidationCode::UnsupportedConstruct,
                format!(
                    "outcome `{}` declares `instances:` under a condition other than `when:` or the \
                     default; a set outcome is selected by its input in this cut",
                    outcome.name
                ),
            )
            .with_hint("select the branch with `when:`, or make it the default"),
        );
    }
    errors
}

/// The entity `name` declares, or an `undeclared_reference` at `at`.
fn declared<'s>(
    spec: &'s Specification,
    name: &QualifiedName,
    at: &ConstructRef,
    errors: &mut ValidationErrors,
) -> Option<&'s EntitySpec> {
    let found = spec.entities().get(name);
    if found.is_none() {
        errors.push(
            ValidationError::at(
                at.clone(),
                ValidationCode::UndeclaredReference,
                format!("`{name}` is not a declared entity"),
            )
            .with_hint(format!(
                "declared entities: {}",
                super::join(spec.entities().keys())
            )),
        );
    }
    found
}

/// A source that reads one row, or the caller, refused by name under a set effect in this cut.
fn one_row_sources(at: &ConstructRef, sets: &BTreeMap<String, PayloadSource>) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    for (target, source) in sets {
        visit(source, &mut |leaf| {
            let written = match leaf {
                PayloadSource::SubjectField { .. } => "{subject: …}",
                PayloadSource::RelatedField { .. } => "{related: …}",
                PayloadSource::Increment { .. } => "{increment: …}",
                PayloadSource::CallerAttribute { .. } => "{caller: …}",
                _ => return,
            };
            errors.push(
                ValidationError::at(
                    at.clone().named(target),
                    ValidationCode::UnsupportedConstruct,
                    format!(
                        "`{target}` is set from `{written}` on a set effect; a source reading one \
                         row or the caller is not admitted over a set of rows in this cut"
                    ),
                )
                .with_hint("set a literal, `input.<field>` or `{input: …, else: …}`"),
            );
        });
    }
    errors
}

/// `source` and every leaf of a nested mapping under it.
fn visit(source: &PayloadSource, each: &mut dyn FnMut(&PayloadSource)) {
    each(source);
    if let PayloadSource::Struct { fields } = source {
        for field in fields {
            visit(&field.source, each);
        }
    }
}

/// `{count: changed}` filling `filled`: admitted in a set outcome's payload, into an `Integer`.
pub(super) fn check_count(
    at: &ConstructRef,
    outcome: &Outcome,
    filled: &Field,
) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    if outcome.set_effects.instances.is_none() {
        errors.push(
            ValidationError::at(
                at.clone(),
                ValidationCode::UnsupportedConstruct,
                format!(
                    "`{{count: changed}}` fills `{}` on outcome `{}`, which declares no \
                     `instances:`; the count is of the rows a set outcome changed",
                    filled.name, outcome.name
                ),
            )
            .with_hint(
                "declare `instances: {where: …}` on the branch, or fill the field another way",
            ),
        );
    }
    if filled.type_ref != TypeRef::Primitive(Primitive::Integer) {
        errors.push(
            ValidationError::at(
                at.clone(),
                ValidationCode::TypeMismatch,
                format!(
                    "`{{count: changed}}` is an `Integer`, and `{}` holds `{}`",
                    filled.name, filled.type_ref
                ),
            )
            .with_hint("declare the field `Integer`"),
        );
    }
    errors
}

/// `{count: changed}` anywhere but a top-level payload field, refused by name.
pub(super) fn count_elsewhere(at: &ConstructRef) -> ValidationError {
    ValidationError::at(
        at.clone(),
        ValidationCode::UnsupportedConstruct,
        "`{count: changed}` is admitted as a whole `payload:` field of a set outcome, and nowhere \
         else",
    )
    .with_hint("fill this field another way")
}

/// The filter, checked over `entity`'s fields, the command's input under `input.`, and — for an
/// `affects:` entry — the subject's fields under `subject.`.
fn check_filter(
    command: &CommandSpec,
    entity: &EntitySpec,
    subject: Option<&EntitySpec>,
    types: &TypeRegistry,
    filter: &Predicate,
    at: &ConstructRef,
) -> ValidationErrors {
    let owner = at.render();
    let mut inner = DomainEnvironment::new(types, &entity.fields);
    if !entity
        .fields
        .iter()
        .any(|field| field.name == super::subject_fact::INPUT_NAMESPACE)
    {
        inner = inner.with_input(&command.input);
    }
    let environment = WithSubject {
        inner,
        subject: subject.map(|subject| {
            (
                &subject.fields,
                entity
                    .fields
                    .iter()
                    .any(|field| field.name == SUBJECT_NAMESPACE),
            )
        }),
        identity: subject.map(|subject| &subject.identity),
    };
    let checked = crate::expression::check_predicate(&environment, filter, &owner);
    let mut errors = ValidationErrors::new();
    for error in &checked.errors {
        errors.push(ValidationError::at(
            at.clone(),
            error.code,
            error.message.clone(),
        ));
    }
    errors
}

/// A type the filter environment reads: one of the specification's, or the subject row.
#[derive(Clone)]
enum Read {
    Declared(TypeRef),
    Subject,
}

impl fmt::Display for Read {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Declared(type_ref) => type_ref.fmt(f),
            Self::Subject => f.write_str("the subject"),
        }
    }
}

/// [`DomainEnvironment`], with `subject.<field>` reading the subject's declared fields.
struct WithSubject<'a> {
    inner: DomainEnvironment<'a>,
    /// The subject's fields, and whether the filtered entity declares a field named `subject`
    /// (which then keeps being read as itself).
    subject: Option<(&'a Vec<Field>, bool)>,
    identity: Option<&'a Field>,
}

impl WithSubject<'_> {
    fn declared(read: &Read) -> Option<&TypeRef> {
        match read {
            Read::Declared(type_ref) => Some(type_ref),
            Read::Subject => None,
        }
    }
}

fn lifted(shape: Shape<TypeRef>) -> Shape<Read> {
    match shape {
        Shape::Scalar(kind) => Shape::Scalar(kind),
        Shape::Enum(variants) => Shape::Enum(variants),
        Shape::Alias(of) => Shape::Alias(Read::Declared(of)),
        Shape::Optional(of) => Shape::Optional(Read::Declared(of)),
        Shape::Struct => Shape::Struct,
        Shape::List(of) => Shape::List(Read::Declared(of)),
        Shape::Map(of) => Shape::Map(Read::Declared(of)),
        Shape::Json => Shape::Json,
        Shape::Union => Shape::Union,
    }
}

impl TypeEnvironment for WithSubject<'_> {
    type Type = Read;

    fn root(&self, name: &str) -> Option<Read> {
        match (self.inner.root(name), self.subject) {
            (Some(found), _) => Some(Read::Declared(found)),
            (None, Some((_, false))) if name == SUBJECT_NAMESPACE => Some(Read::Subject),
            _ => None,
        }
    }
    fn cardinality_type(&self) -> Read {
        Read::Declared(self.inner.cardinality_type())
    }
    fn shape(&self, reference: &Read) -> Result<Shape<Read>, String> {
        match reference {
            Read::Declared(type_ref) => self.inner.shape(type_ref).map(lifted),
            Read::Subject => Ok(Shape::Struct),
        }
    }
    fn is_clock_reading(&self, reference: &Read) -> bool {
        Self::declared(reference).is_some_and(|type_ref| self.inner.is_clock_reading(type_ref))
    }
    fn is_instant(&self, reference: &Read) -> bool {
        Self::declared(reference).is_some_and(|type_ref| self.inner.is_instant(type_ref))
    }
    fn is_duration(&self, reference: &Read) -> bool {
        Self::declared(reference).is_some_and(|type_ref| self.inner.is_duration(type_ref))
    }
    fn is_string(&self, reference: &Read) -> bool {
        Self::declared(reference).is_some_and(|type_ref| self.inner.is_string(type_ref))
    }
    fn admits_text_length(&self) -> bool {
        self.inner.admits_text_length()
    }
    fn admits_aggregate_presence(&self) -> bool {
        self.inner.admits_aggregate_presence()
    }
    fn current_time(&self) -> CurrentTimeAdmission {
        CurrentTimeAdmission {
            site: false,
            format: self.inner.current_time().format,
        }
    }
    fn member(&self, reference: &Read, name: &str) -> Option<Read> {
        match reference {
            Read::Declared(type_ref) => self.inner.member(type_ref, name).map(Read::Declared),
            Read::Subject => {
                let (fields, _) = self.subject?;
                fields
                    .iter()
                    .chain(self.identity)
                    .find(|field| field.name == name)
                    .map(|field| Read::Declared(field.type_ref.clone()))
            }
        }
    }
    fn has_parameters(&self) -> bool {
        self.inner.has_parameters()
    }
    fn parameter(&self, name: &str) -> Option<Read> {
        self.inner.parameter(name).map(Read::Declared)
    }
    fn parameter_namespace(&self) -> &'static str {
        self.inner.parameter_namespace()
    }
}

/// Refuses `instances:`, `affects:` and — beside `instances:` — `{count: changed}` under a header
/// below `ess/16`, at the key written, before any outcome is converted: the format is what is wrong,
/// and the shape rules of a construct the format does not have would only bury it.
///
/// Each refused key is taken off the outcome, and the verb beside `instances:` with it, so the
/// conversion that follows reports no cascade (`missing_declaration` for the `instance:` a set
/// subject never had, `empty_declaration` for a command whose only branch it refused); a transition
/// only such a verb named is recorded in `refused_moves`, as a refused command's is. A
/// `{count: changed}` on any other outcome is a nested mapping below `ess/16`
/// ([`read_below_ess_16`]), as it always was.
pub(crate) fn refuse_below_ess_16(
    files: &mut [(crate::system::Source, crate::spec::RawSpecFile)],
    errors: &mut ValidationErrors,
    refused_moves: &mut std::collections::BTreeSet<QualifiedName>,
) {
    let headers: Vec<Option<FormatVersion>> = files
        .iter()
        .filter(|(_, file)| file.system.is_some())
        .map(|(_, file)| file.format)
        .collect();
    let [format] = headers.as_slice() else {
        return;
    };
    if format.unwrap_or(FormatVersion::V1).major() >= FormatVersion::V16.major() {
        return;
    }
    let refuse = |at: String, key: &str| {
        ValidationError::new(
            ValidationCode::UnsupportedFormatVersion,
            at,
            format!("`{key}` requires specification format ess/16"),
        )
        .with_hint("declare `format: ess/16`")
    };
    for (_, file) in files.iter_mut() {
        for command in &mut file.commands {
            for outcome in &mut command.outcomes {
                let at = format!("command.{}.outcomes.{}", command.name, outcome.name);
                if outcome.instances.take().is_some() {
                    errors.push(refuse(format!("{at}.instances"), "instances:"));
                    refused_moves.extend(outcome.moves.take());
                    outcome.updates = None;
                    for (event, table) in &mut outcome.payload.0 {
                        for field in &mut table.0 {
                            if field.source == PayloadSource::ChangedCount {
                                errors.push(refuse(
                                    format!("{at}.payload.{event}.{}", field.target),
                                    "{count: changed}",
                                ));
                                field.source = PayloadSource::Generated;
                            }
                        }
                    }
                }
                if !outcome.affects.is_empty() {
                    outcome.affects.clear();
                    errors.push(refuse(format!("{at}.affects"), "affects:"));
                }
            }
        }
    }
}

/// Reads `{count: changed}` back as the nested mapping it was below `ess/16`.
pub fn read_below_ess_16(
    format: FormatVersion,
    commands: &mut BTreeMap<QualifiedName, CommandSpec>,
) {
    if format.major() >= FormatVersion::V16.major() {
        return;
    }
    for outcome in commands
        .values_mut()
        .flat_map(|command| command.outcomes.iter_mut())
    {
        for source in outcome
            .payload
            .values_mut()
            .flat_map(|fields| fields.values_mut())
            .chain(outcome.sets.values_mut())
        {
            nested(source);
        }
    }
}

fn nested(source: &mut PayloadSource) {
    match source {
        PayloadSource::ChangedCount => {
            *source = PayloadSource::Struct {
                fields: vec![super::PayloadField {
                    target: COUNT.to_owned(),
                    source: PayloadSource::parse(CHANGED),
                }],
            };
        }
        PayloadSource::Struct { fields } => {
            for leaf in fields {
                nested(&mut leaf.source);
            }
        }
        _ => {}
    }
}

/// The key of `{count: changed}`.
pub(super) const COUNT: &str = "count";
/// The one word it takes.
pub(super) const CHANGED: &str = "changed";

/// `{count: changed}` as written.
#[derive(serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct RawCountSource {
    /// `changed`: the rows the outcome changed.
    count: String,
}

impl RawCountSource {
    /// The mapping `{count: changed}`, and nothing else.
    pub(super) fn recognise(entries: &[(String, super::RawPayloadSource)]) -> Option<Self> {
        match entries {
            [(key, super::RawPayloadSource::Text(word))] if key == COUNT && word == CHANGED => {
                Some(Self {
                    count: CHANGED.to_owned(),
                })
            }
            _ => None,
        }
    }

    /// The written form.
    pub(super) fn changed() -> Self {
        Self {
            count: CHANGED.to_owned(),
        }
    }
}

/// A set effect's `sets:` writing the entity's identity, refused: every selected row would come to
/// hold one identity.
fn identity_set(
    entity: &EntitySpec,
    at: &ConstructRef,
    sets: &BTreeMap<String, PayloadSource>,
) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    let identity = &entity.identity.name;
    if sets.contains_key(identity) {
        errors.push(
            ValidationError::at(
                at.clone().named(identity),
                ValidationCode::ConflictingDeclaration,
                format!(
                    "`{identity}` is the identity of `{}`, and a set effect writes it on every \
                     row it selects; rows that all come to hold one identity are one row",
                    entity.name
                ),
            )
            .with_hint("drop the identity from `sets:`; a set effect changes rows, it names none"),
        );
    }
    errors
}
