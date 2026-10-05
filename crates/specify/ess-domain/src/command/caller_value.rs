//! The authenticated caller as a value source and a guard operand (source format `ess/16`,
//! beyond10x/ess#168, `docs/design/caller-values.md`).
//!
//! An actor declares `attributes:` — typed fields its credential carries, such as the account it
//! acts for. A command reads one as `{caller: <attribute>}` in `payload:` and `sets:`, and as
//! `caller.<attribute>` in a guard: compared with an input field in `when:`, or with a stored field
//! (or an input) in `when_subject:`.
//!
//! Which actor runs a command is not known when the document is read, so an attribute is readable
//! only where **every** actor that may invoke the command declares it, at one type. A command
//! whose actors disagree is refused where it reads the attribute, naming the actor that lacks it.
//!
//! A guard compares a caller attribute with `==` or `!=` against a field and nothing else: the
//! comparison the issue asks for — "the caller is the record's agent" — and the only one a
//! conformance suite can witness from both sides by choosing which of two callers sends the
//! command.

use std::collections::BTreeMap;

use ess_primitives::error::{ConstructRef, ValidationCode, ValidationError, ValidationErrors};
use ess_primitives::facts::FactPath;
use ess_primitives::predicate::{CompareOp, Operand, Predicate};

use super::{CommandSpec, OutcomeCondition, PayloadField, PayloadSource, RawPayloadSource};
use crate::actor::ActorSpec;
use crate::expression::DomainEnvironment;
use crate::name::QualifiedName;
use crate::system::FormatVersion;
use crate::types::Field;
use crate::Specification;

/// The root a guard reads a caller attribute under: `caller.account_id`.
pub const CALLER_NAMESPACE: &str = "caller";

/// `{caller: <attribute>}` (ess/16, #168), written alone.
///
/// Recognised by its exact shape — one key, `caller`, holding a text — so `caller` is no source
/// keyword: any other mapping under a `caller` key is a nested mapping, as it was before `ess/16`.
/// A document below `ess/16` whose struct happens to have this shape is read back as that nested
/// mapping by [`read_below_ess_16`].
#[derive(serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct RawCallerSource {
    /// An attribute every actor that may invoke the command declares.
    caller: String,
}

impl RawCallerSource {
    /// The mapping `{caller: <text>}`, and nothing else.
    pub(super) fn recognise(entries: &[(String, RawPayloadSource)]) -> Option<Self> {
        match entries {
            [(key, RawPayloadSource::Text(attribute))] if key == CALLER_NAMESPACE => Some(Self {
                caller: attribute.clone(),
            }),
            _ => None,
        }
    }

    /// The written form of a caller source.
    pub(super) fn of(attribute: &str) -> Self {
        Self {
            caller: attribute.to_owned(),
        }
    }

    /// The source it is.
    pub(super) fn into_source(self) -> PayloadSource {
        PayloadSource::CallerAttribute {
            attribute: self.caller,
        }
    }
}

/// Reads every `{caller: <text>}` of a document below `ess/16` back as the nested mapping it was
/// before that format existed: a struct field `caller` holding the text as a bare text is read.
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
        PayloadSource::CallerAttribute { attribute } => {
            *source = PayloadSource::Struct {
                fields: vec![PayloadField {
                    target: CALLER_NAMESPACE.to_owned(),
                    source: PayloadSource::parse(attribute),
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

/// The actors that may invoke `command`, in name order.
pub fn callers<'a>(spec: &'a Specification, command: &QualifiedName) -> Vec<&'a ActorSpec> {
    spec.actors()
        .values()
        .filter(|actor| actor.may_invoke(command))
        .collect()
}

/// The attributes every actor that may invoke `command` declares, at one type: what a command
/// can read of its caller.
pub fn common_attributes(spec: &Specification, command: &QualifiedName) -> Vec<Field> {
    let actors = callers(spec, command);
    let Some((first, rest)) = actors.split_first() else {
        return Vec::new();
    };
    first
        .attributes
        .iter()
        .filter(|attribute| {
            rest.iter().all(|actor| {
                actor
                    .attribute(&attribute.name)
                    .is_some_and(|other| other.type_ref == attribute.type_ref)
            })
        })
        .cloned()
        .collect()
}

/// Whether `path` reads the caller: `caller.<attribute>`, where no root field the predicate is
/// read over is itself named `caller`.
pub fn is_caller_path(path: &FactPath, roots: &[Field]) -> bool {
    path.namespace() == CALLER_NAMESPACE
        && path.segments().len() > 1
        && !roots.iter().any(|field| field.name == CALLER_NAMESPACE)
}

/// Whether a predicate over `roots` reads the caller anywhere.
pub fn reads_caller(predicate: &Predicate, roots: &[Field]) -> bool {
    predicate
        .fact_paths()
        .into_iter()
        .any(|path| is_caller_path(path, roots))
}

/// Whether who sent the command decides `outcome`: its own guard reads the caller (its `when:`
/// over the input, or its `when_subject:` over the stored fields of the subject it reads), or it
/// is an unguarded refusal reached only because an earlier accepting branch guarded on the caller
/// did not hold — "the edit is permitted only to the note's agent", with the refusal as the rest.
pub fn decides(spec: &Specification, command: &CommandSpec, outcome: &super::Outcome) -> bool {
    if guard_reads_caller(spec, command, outcome) {
        return true;
    }
    outcome.error.is_some()
        && outcome.condition == OutcomeCondition::Otherwise
        && command
            .outcomes
            .iter()
            .take_while(|earlier| earlier.name != outcome.name)
            .any(|earlier| earlier.error.is_none() && guard_reads_caller(spec, command, earlier))
}

/// Whether `outcome`'s own guard reads the caller.
fn guard_reads_caller(
    spec: &Specification,
    command: &CommandSpec,
    outcome: &super::Outcome,
) -> bool {
    if outcome
        .condition
        .predicate()
        .is_some_and(|predicate| reads_caller(predicate, &command.input))
    {
        return true;
    }
    let OutcomeCondition::SubjectPredicate { predicate, .. } = &outcome.condition else {
        return false;
    };
    let fields = super::subject_fact::reading_subject(command, outcome)
        .and_then(|subject| spec.entities().get(&subject.entity))
        .map_or(&[][..], |entity| entity.fields.as_slice());
    reads_caller(predicate, fields)
}

fn paths_read(
    predicate: &Predicate,
    roots: &[Field],
    read: &mut std::collections::BTreeSet<String>,
) {
    for path in predicate.fact_paths() {
        if is_caller_path(path, roots) {
            read.insert(path.segments()[1].clone());
        }
    }
}

/// Why `attribute` cannot be read by `command`, where it cannot: no actor may invoke it, or one
/// that may does not declare the attribute, or two declare it at different types.
fn unreadable(spec: &Specification, command: &QualifiedName, attribute: &str) -> Option<String> {
    let actors = callers(spec, command);
    if actors.is_empty() {
        return Some(format!(
            "no actor may invoke `{command}`, so there is no caller to read `{attribute}` from"
        ));
    }
    let lacking: Vec<String> = actors
        .iter()
        .filter(|actor| actor.attribute(attribute).is_none())
        .map(|actor| format!("`{}`", actor.name))
        .collect();
    if !lacking.is_empty() {
        let declaring: Vec<String> = actors
            .iter()
            .filter(|actor| actor.attribute(attribute).is_some())
            .map(|actor| format!("`{}`", actor.name))
            .collect();
        return Some(if declaring.is_empty() {
            format!(
                "`{command}` reads the caller's `{attribute}`, and {} may invoke it without \
                 declaring that attribute",
                lacking.join(" and ")
            )
        } else {
            format!(
                "`{command}` reads the caller's `{attribute}`, which {} declare{}, and {} may \
                 invoke it without declaring it",
                declaring.join(" and "),
                if declaring.len() == 1 { "s" } else { "" },
                lacking.join(" and ")
            )
        });
    }
    let types = actors
        .iter()
        .filter_map(|actor| actor.attribute(attribute))
        .map(|field| field.type_ref.to_string())
        .collect::<std::collections::BTreeSet<_>>();
    (types.len() > 1).then(|| {
        format!(
            "the actors that may invoke `{command}` declare `{attribute}` at different types: {}",
            types.into_iter().collect::<Vec<_>>().join(", ")
        )
    })
}

/// The code a refusal from [`unreadable`] carries: nobody to read from is a reference with nothing
/// behind it; actors that disagree are a conflict between declarations.
fn unreadable_code(
    spec: &Specification,
    command: &QualifiedName,
    attribute: &str,
) -> ValidationCode {
    let actors = callers(spec, command);
    if actors.is_empty()
        || actors
            .iter()
            .all(|actor| actor.attribute(attribute).is_none())
    {
        ValidationCode::UndeclaredReference
    } else {
        ValidationCode::ConflictingDeclaration
    }
}

/// `{caller: <attribute>}` in `payload:` or `sets:`, filling `target`: an attribute the command's
/// every actor declares, of a type the target accepts.
pub(super) fn check_source(
    spec: &Specification,
    command: &CommandSpec,
    at: &ConstructRef,
    target: &Field,
    attribute: &str,
    errors: &mut ValidationErrors,
) {
    if spec.system().format.major() < FormatVersion::V16.major() {
        errors.push(
            ValidationError::at(
                at.clone(),
                ValidationCode::UnsupportedFormatVersion,
                "a `{caller: …}` source requires specification format ess/16",
            )
            .with_hint("write `format: ess/16` on the source that declares the system"),
        );
        return;
    }
    if let Some(reason) = unreadable(spec, &command.name, attribute) {
        errors.push(
            ValidationError::at(
                at.clone(),
                unreadable_code(spec, &command.name, attribute),
                reason,
            )
            .with_hint(
                "declare the attribute under `attributes:` on every actor whose `may` names the \
                 command, at one type",
            ),
        );
        return;
    }
    let Some(declared) = callers(spec, &command.name)
        .into_iter()
        .find_map(|actor| actor.attribute(attribute))
    else {
        return;
    };
    if !spec
        .conversions()
        .permits(&declared.type_ref, &target.type_ref)
    {
        errors.push(
            ValidationError::at(
                at.clone(),
                ValidationCode::TypeMismatch,
                format!(
                    "the caller's `{attribute}` has type `{}`, and `{}` requires `{}`; no \
                     conversion is declared",
                    declared.type_ref, target.name, target.type_ref
                ),
            )
            .with_hint(format!(
                "declare the crossing — `conversions: [{{from: {}, to: {}, because: …}}]` — or \
                 make the two types agree",
                declared.type_ref, target.type_ref
            )),
        );
    }
}

/// Every guard that reads the caller: the format, the attributes, the shape of the comparison and
/// the types on both sides.
///
/// The command's own guard checks leave a `caller.` read to this pass, because only here are the
/// actors in hand.
pub(crate) fn validate(spec: &Specification) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    let types = &spec.system().types;
    for actor in spec.actors().values() {
        errors.extend(actor.validate_attributes(spec.system().format, types));
    }
    for command in spec.commands().values() {
        let attributes = common_attributes(spec, &command.name);
        for outcome in &command.outcomes {
            let site = command.site().key("outcomes").named(outcome.name.as_str());
            if let Some(predicate) = outcome.condition.predicate() {
                if reads_caller(predicate, &command.input) {
                    let at = site.clone().key("when");
                    let environment =
                        DomainEnvironment::new(types, &command.input).with_caller(&attributes);
                    check_guard(
                        spec,
                        command,
                        (&at, false),
                        predicate,
                        &command.input,
                        &environment,
                        &mut errors,
                    );
                }
            }
            if let OutcomeCondition::SubjectPredicate { predicate, .. } = &outcome.condition {
                let Some(entity) = super::subject_fact::reading_subject(command, outcome)
                    .and_then(|subject| spec.entities().get(&subject.entity))
                else {
                    continue;
                };
                if reads_caller(predicate, &entity.fields) {
                    let at = site.clone().key("when_subject");
                    let mut environment =
                        DomainEnvironment::new(types, &entity.fields).with_stored_current_time();
                    if !entity
                        .fields
                        .iter()
                        .any(|field| field.name == super::subject_fact::INPUT_NAMESPACE)
                    {
                        environment = environment.with_input(&command.input);
                    }
                    let environment = environment.with_caller(&attributes);
                    check_guard(
                        spec,
                        command,
                        (&at, true),
                        predicate,
                        &entity.fields,
                        &environment,
                        &mut errors,
                    );
                }
            }
        }
    }
    errors
}

/// One guard reading the caller. `whole` reports every finding of the typed check: a
/// `when_subject:` predicate reading the caller is checked here and nowhere else, where a `when:`
/// guard's other reads are checked by the command itself.
fn check_guard(
    spec: &Specification,
    command: &CommandSpec,
    (at, whole): (&ConstructRef, bool),
    predicate: &Predicate,
    roots: &[Field],
    environment: &DomainEnvironment<'_>,
    errors: &mut ValidationErrors,
) {
    if spec.system().format.major() < FormatVersion::V16.major() {
        errors.push(
            ValidationError::at(
                at.clone(),
                ValidationCode::UnsupportedFormatVersion,
                "a `caller.` operand requires specification format ess/16",
            )
            .with_hint("write `format: ess/16` on the source that declares the system"),
        );
        return;
    }
    let mut attributes = std::collections::BTreeSet::new();
    paths_read(predicate, roots, &mut attributes);
    let mut readable = true;
    for attribute in &attributes {
        if let Some(reason) = unreadable(spec, &command.name, attribute) {
            readable = false;
            errors.push(
                ValidationError::at(
                    at.clone(),
                    unreadable_code(spec, &command.name, attribute),
                    reason,
                )
                .with_hint(
                    "declare the attribute under `attributes:` on every actor whose `may` names \
                     the command, at one type",
                ),
            );
        }
    }
    if let Some(misplaced) = misplaced(predicate, roots) {
        errors.push(
            ValidationError::at(
                at.clone(),
                ValidationCode::UnobservableFact,
                format!(
                    "`{misplaced}` reads the caller, and a caller attribute is compared with `==` \
                     or `!=` against a field and nothing else"
                ),
            )
            .with_hint(
                "compare it with an input field (`when: account_id == caller.account_id`) or a \
                 stored field (`when_subject: agent_id != caller.agent_id`)",
            ),
        );
        return;
    }
    if !readable {
        return;
    }
    for mismatch in nominal_mismatches(predicate, roots, environment, &at.render()) {
        errors.push(ValidationError::at(
            at.clone(),
            ValidationCode::TypeMismatch,
            mismatch,
        ));
    }
    let checked = crate::expression::check_predicate(environment, predicate, &at.render());
    for error in &checked.errors {
        // A read of anything but the caller is reported by the guard's own check, which has no
        // caller in hand and leaves every caller read, and the comparison it sits in, to this one.
        let other_read = error
            .path
            .as_ref()
            .is_some_and(|path| !is_caller_path(path, roots));
        if !whole && other_read && error.code != ValidationCode::TypeMismatch {
            continue;
        }
        errors.push(ValidationError::at(
            at.clone(),
            error.code,
            error.message.clone(),
        ));
    }
}

/// Every comparison of a caller attribute with a field declared at another type.
///
/// The expression checker compares representations — an account id and a note's text are both
/// text — and a caller is compared with the field that holds the same kind of identity, so the
/// declared types must agree, `Optional` aside.
fn nominal_mismatches(
    predicate: &Predicate,
    roots: &[Field],
    environment: &DomainEnvironment<'_>,
    owner: &str,
) -> Vec<String> {
    let mut found = Vec::new();
    let mut pending = vec![predicate];
    while let Some(predicate) = pending.pop() {
        match predicate {
            Predicate::All(children) | Predicate::Any(children) => pending.extend(children),
            Predicate::Not(child) => pending.push(child),
            Predicate::Compare {
                left: Operand::Fact(left),
                right: Operand::Fact(right),
                ..
            } if is_caller_path(left, roots) != is_caller_path(right, roots) => {
                let declared = |path: &FactPath| {
                    crate::expression::resolve_path(environment, path, owner)
                        .ok()
                        .map(|resolved| {
                            let declared = resolved.declared;
                            declared
                                .strip_prefix("Optional<")
                                .and_then(|rest| rest.strip_suffix('>'))
                                .map_or(declared.clone(), str::to_owned)
                        })
                };
                if let (Some(left_type), Some(right_type)) = (declared(left), declared(right)) {
                    if left_type != right_type {
                        found.push(format!(
                            "`{predicate}` compares `{left}` (`{left_type}`) with `{right}` \
                             (`{right_type}`); a caller attribute is compared with a field of \
                             its own type"
                        ));
                    }
                }
            }
            _ => {}
        }
    }
    found
}

/// The first caller read that is not one side of an `==` or `!=` whose other side is a field
/// other than the caller.
fn misplaced(predicate: &Predicate, roots: &[Field]) -> Option<FactPath> {
    let caller = |operand: &Operand| match operand {
        Operand::Fact(path) if is_caller_path(path, roots) => Some(path.clone()),
        _ => None,
    };
    match predicate {
        Predicate::Always | Predicate::Never => None,
        Predicate::All(children) | Predicate::Any(children) => {
            children.iter().find_map(|child| misplaced(child, roots))
        }
        Predicate::Not(child) => misplaced(child, roots),
        // A caller moved by a constant (A2) is no caller equality, whatever the operator.
        Predicate::Compare {
            right: Operand::Offset(offset),
            ..
        } if is_caller_path(&offset.base, roots) => Some(offset.base.clone()),
        Predicate::Compare {
            left, op, right, ..
        } => match (caller(left), caller(right)) {
            (None, None) => None,
            (Some(path), None) | (None, Some(path)) => {
                let other = if caller(left).is_some() { right } else { left };
                (!matches!(op, CompareOp::Eq | CompareOp::Ne) || !matches!(other, Operand::Fact(_)))
                    .then_some(path)
            }
            (Some(path), Some(_)) => Some(path),
        },
        other => other
            .fact_paths()
            .into_iter()
            .find(|path| is_caller_path(path, roots))
            .cloned(),
    }
}
