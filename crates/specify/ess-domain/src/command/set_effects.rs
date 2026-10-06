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
//!   From ess/22 (beyond10x/ess#229, `story:related-record-effects`) an entry may also declare
//!   `moves: <Entity>.<transition>`: every selected row resting in the transition's `from` states
//!   takes it, and a selected row resting elsewhere is skipped, as under `instances:`.
//!
//! From ess/23 (beyond10x/ess#452) `instances:` is also admitted beside `deletes:`, removing every
//! row the filter selects; an `affects:` entry may declare `deletes: <Entity>` naming its own
//! entity, removing every row it selects; and `affects:` sits beside a `deletes:` subject. Neither
//! takes `sets:`, and a deleting entry stands alone over its entity among the entries. Below ess/23
//! each keeps its refusal, naming ess/23.
//!
//! From ess/23 (beyond10x/ess#459) an `affects:` entry may also write one row per element of an
//! input list: `each: {in: input.<list>, as: <name>}` with `instance: <name>.<member>` in place of
//! `where:`. The row each element names is updated if held and created in `initial` if not, and a
//! `sets:` source may read `<name>.<member>`; the member is held distinct across the list by a
//! declared `distinct:`. Below ess/23 `each:` is refused, naming ess/23.
//!
//! `sets:` of either takes a literal, `input.<field>`, `{input: …, else: …}`, `{generated: true}`
//! or `{cleared: true}`; a source that reads one row (`{subject: …}`, `{related: …}`,
//! `{increment: …}`) or the caller is refused by name in this first cut. Below ess/22 a move inside
//! `affects:` is refused naming ess/22.

use std::collections::BTreeMap;
use std::fmt;

use ess_primitives::error::{ConstructRef, ValidationCode, ValidationError, ValidationErrors};
use ess_primitives::predicate::{Predicate, WrittenPredicate};

use super::{CommandSpec, Effect, Outcome, OutcomeName, PayloadSource, PayloadTable, Subject};
use crate::entity::EntitySpec;
use crate::expression::{CurrentTimeAdmission, DomainEnvironment, Shape, TypeEnvironment};
use crate::name::QualifiedName;
use crate::system::FormatVersion;
use crate::types::{Field, Primitive, TypeRef, TypeRegistry};
use crate::Specification;

/// The root an `affects:` filter reads the subject under: `subject.team`.
pub const SUBJECT_NAMESPACE: &str = "subject";

std::thread_local! {
    /// Whether the commands being converted belong to an `ess/23` source, where `deletes:` takes a
    /// set subject and `affects:` (beyond10x/ess#452). Set only while
    /// [`crate::spec::Specification::assemble`] converts a source whose one header it has read; it
    /// decides only what a refusal of another verb names as admitted.
    static ADMITS_DELETIONS: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Runs `convert` with `deletes:` named among the verbs a set subject and `affects:` sit beside
/// when `admitted`, restoring what was set before, so a nested assembly cannot leak its format
/// into its caller's.
pub(crate) fn converting_deletions<T>(admitted: bool, convert: impl FnOnce() -> T) -> T {
    let before = ADMITS_DELETIONS.with(|cell| cell.replace(admitted));
    let converted = convert();
    ADMITS_DELETIONS.with(|cell| cell.set(before));
    converted
}

fn admits_deletions() -> bool {
    ADMITS_DELETIONS.with(std::cell::Cell::get)
}

/// What `instances:` holds as written.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RawInstances {
    /// The rows selected: a predicate over the entity's stored fields, with `input.` operands.
    #[serde(rename = "where")]
    pub filter: WrittenPredicate,
}

/// One `affects:` entry as written.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RawAffect {
    /// The entity whose rows change.
    pub entity: QualifiedName,
    /// The rows selected, over the entity's fields, `input.` and `subject.`. Absent only on an
    /// entry that writes one row per element instead (`each:`, ess/23, beyond10x/ess#459).
    #[serde(rename = "where", default, skip_serializing_if = "Option::is_none")]
    pub filter: Option<WrittenPredicate>,
    /// One row per element of an input list (ess/23, beyond10x/ess#459): the row `instance:`
    /// names is updated if held and created in `initial` if not.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub each: Option<RawEach>,
    /// The row an element names, written `<as>.<member>`; beside `each:` only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instance: Option<String>,
    /// What every selected row comes to hold. Beside `each:` a source may read the element as
    /// `<as>.<member>`.
    #[serde(default, skip_serializing_if = "PayloadTable::is_empty")]
    pub sets: PayloadTable,
    /// The move every selected row resting in its `from` states takes, written
    /// `<Entity>.<transition>` over `entity` (ess/22, beyond10x/ess#229); a selected row resting
    /// elsewhere is skipped.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub moves: Option<QualifiedName>,
    /// Every selected row is removed, written `deletes: <Entity>` naming `entity` (ess/23,
    /// beyond10x/ess#452).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deletes: Option<QualifiedName>,
}

/// `each: {in: input.<list>, as: <name>}` as written (ess/23, beyond10x/ess#459).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RawEach {
    /// The list, `input.<field>`.
    #[serde(rename = "in")]
    pub list: String,
    /// The name each element is read under.
    #[serde(rename = "as")]
    pub binder: String,
}

/// One row per element of an input list (ess/23, beyond10x/ess#459): for each element, the row
/// whose identity `member` holds is updated if held and created in `initial` if not.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Each {
    /// The input field holding the list.
    pub list: String,
    /// The name each element is read under.
    pub binder: String,
    /// The member of the element holding the identity of the row it writes.
    pub member: String,
    /// The entity fields written from a member of the element: field, then member.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub reads: BTreeMap<String, String>,
}

/// The rows a `moves:`, `updates:` or — from ess/23 — `deletes:` outcome changes, selected by a
/// filter rather than named.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct SetSubject {
    /// The entity whose rows change.
    pub entity: QualifiedName,
    /// [`Effect::Moves`], [`Effect::Updates`] or [`Effect::Deletes`] (ess/23, beyond10x/ess#452),
    /// and nothing else.
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
    /// The rows selected, over the entity's fields, `input.` and `subject.`; `None` exactly where
    /// [`Self::each`] names the rows instead.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<Predicate>,
    /// One row per element of an input list (ess/23, beyond10x/ess#459).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub each: Option<Each>,
    /// What every selected row comes to hold; beside [`Self::each`], every source but the
    /// element's members, which [`Each::reads`] holds.
    pub sets: BTreeMap<String, PayloadSource>,
    /// The transition of `entity` every selected row resting in its `from` states takes (ess/22,
    /// beyond10x/ess#229); `None` where the entry only sets fields.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub moves: Option<String>,
    /// Every selected row is removed (ess/23, beyond10x/ess#452); `false` where the entry sets
    /// fields or moves its rows.
    #[serde(skip_serializing_if = "is_false")]
    pub deletes: bool,
}

#[allow(clippy::trivially_copy_pass_by_ref)] // serde passes the field by reference.
fn is_false(value: &bool) -> bool {
    !*value
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
    /// `creates` or `preserves`, where the outcome wrote one of them.
    pub(super) other: Option<&'static str>,
    pub(super) instance: bool,
    pub(super) moves: &'a mut Option<QualifiedName>,
    pub(super) updates: &'a mut Option<QualifiedName>,
    /// `deletes:`, which takes a set subject from ess/23 (beyond10x/ess#452); below it the header
    /// refuses the pair before conversion ([`refuse_deletions`]).
    pub(super) deletes: &'a mut Option<QualifiedName>,
}

/// `instances:` as the set subject it declares, taking the `moves:`, `updates:` or `deletes:` it
/// was written beside so [`super::subject_of`] sees no single subject; or the refusal of a
/// combination.
pub(super) fn set_subject(
    name: &OutcomeName,
    instances: Option<RawInstances>,
    verbs: &mut Verbs<'_>,
) -> Result<Option<SetSubject>, ValidationErrors> {
    let Some(instances) = instances else {
        return Ok(None);
    };
    // A filter that does not parse is refused here, at the outcome's `instances.where`
    // (beyond10x/ess#448).
    let filter = instances
        .filter
        .read(format!("outcomes.{name}.instances.where"))
        .map_err(ValidationErrors::from)?;
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
        return Err(other_verb_beside_instances(name, verb));
    }
    if let Some(entity) = verbs.deletes.take() {
        return Ok(Some(SetSubject {
            entity,
            effect: Effect::Deletes,
            filter,
        }));
    }
    if let Some(entity) = verbs.updates.take() {
        return Ok(Some(SetSubject {
            entity,
            effect: Effect::Updates,
            filter,
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
        filter,
    }))
}

/// The refusal of `instances:` beside `verb`, `preserves` (`creates` is refused before it). Below
/// ess/23 the wording is the one it always was; `deletes:` is named only where the source's format
/// admits it (beyond10x/ess#452).
fn other_verb_beside_instances(name: &OutcomeName, verb: &str) -> ValidationErrors {
    let (message, hint) = if admits_deletions() {
        (
            format!(
                "outcome `{name}` {verb} an entity and declares `instances:`; a set subject is \
                 admitted beside `moves:` and `updates:` only, and, from ess/23, `deletes:`"
            ),
            "name one row with `instance:`, or change the rows with `moves:` or `updates:`, or, \
             from ess/23, remove them with `deletes:`",
        )
    } else {
        (
            format!(
                "outcome `{name}` {verb} an entity and declares `instances:`; a set subject is \
                 admitted beside `moves:` and `updates:` only"
            ),
            "name one row with `instance:`, or change the rows with `moves:` or `updates:`",
        )
    };
    refusal(
        name,
        "instances",
        ValidationCode::UnsupportedConstruct,
        message,
        hint,
    )
}

/// Why `moved`, written inside an `affects:` entry over `entity`, is not a transition of that
/// entity as written — another entity's, or one naming no entity — as the refusal's code, message
/// and hint; `None` where it names one of `entity`'s.
fn misnamed_move(
    name: &str,
    entity: &QualifiedName,
    moved: &QualifiedName,
) -> Option<(ValidationCode, String, &'static str)> {
    match moved.namespace() {
        Some(named) if named == *entity => None,
        Some(named) => Some((
            ValidationCode::ConflictingDeclaration,
            format!(
                "outcome `{name}` moves `{moved}` inside an `affects:` entry over `{entity}`; the \
                 move is a transition of `{named}`, and the entry changes rows of `{entity}` only"
            ),
            "name a transition of the entry's own entity, written `<Entity>.<transition>`",
        )),
        None => Some((
            ValidationCode::MissingDeclaration,
            format!(
                "outcome `{name}` moves `{moved}` inside `affects:`, which names no entity; a \
                 move is written as the entity followed by the transition"
            ),
            "write it as `billing.invoice.Invoice.settle`",
        )),
    }
}

/// Why `deleted`, written as `deletes:` inside an `affects:` entry over `entity`, is not that
/// entity (beyond10x/ess#452), as the refusal's message and hint; `None` where it is.
fn misnamed_deletion(
    name: &str,
    entity: &QualifiedName,
    deleted: &QualifiedName,
) -> Option<(String, String)> {
    (deleted != entity).then(|| {
        (
            format!(
                "outcome `{name}` deletes `{deleted}` inside an `affects:` entry over `{entity}`; \
                 the entry removes rows of `{entity}` only"
            ),
            format!("name the entry's own entity: `deletes: {entity}`"),
        )
    })
}

/// `affects:` as written, with a move naming another entity than the entry's refused and a field
/// set twice refused. An assembled specification reaches this with such a move already taken off
/// the entry and refused alone by [`refuse_affect_moves`], which also says whether the format
/// admits a move at all; the refusal here is the backstop for an outcome converted on its own. A
/// `deletes:` naming another entity than the entry's is refused at the entry the same way, with
/// [`refuse_deletions`] as the pass before conversion.
pub(super) fn affects(
    name: &OutcomeName,
    written: Vec<RawAffect>,
) -> Result<Vec<Affect>, ValidationErrors> {
    let mut errors = ValidationErrors::new();
    let mut affects = Vec::with_capacity(written.len());
    for (index, mut raw) in written.into_iter().enumerate() {
        // An entry writing one row per element (ess/23, beyond10x/ess#459) names its rows by the
        // element, and takes its element reads out of `sets:`.
        let each = match each_entry(name, index, &mut raw) {
            Ok(each) => each,
            Err(refused) => {
                errors.extend(refused);
                continue;
            }
        };
        // A filter that does not parse is refused at its entry's `where` (beyond10x/ess#448).
        let filter = match raw
            .filter
            .take()
            .map(|filter| filter.read(format!("outcomes.{name}.affects[{index}].where")))
        {
            None => None,
            Some(Ok(filter)) => Some(filter),
            Some(Err(error)) => {
                errors.push(error);
                continue;
            }
        };
        let moves = match &raw.moves {
            None => None,
            Some(moved) => match misnamed_move(name.as_str(), &raw.entity, moved) {
                None => Some(moved.local().to_owned()),
                Some((code, message, hint)) => {
                    errors.extend(refusal(
                        name,
                        &format!("affects[{index}].moves"),
                        code,
                        message,
                        hint,
                    ));
                    continue;
                }
            },
        };
        // A misnamed deletion is refused and the rest of the entry still read, so its other
        // refusals keep their place (beyond10x/ess#452).
        let deletes = match &raw.deletes {
            None => false,
            Some(deleted) => match misnamed_deletion(name.as_str(), &raw.entity, deleted) {
                None => true,
                Some((message, hint)) => {
                    errors.extend(refusal(
                        name,
                        &format!("affects[{index}]"),
                        ValidationCode::ConflictingDeclaration,
                        message,
                        &hint,
                    ));
                    false
                }
            },
        };
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
            filter,
            each,
            sets,
            moves,
            deletes,
        });
    }
    errors.into_result(affects)
}

/// An `affects:` entry without `each:` selects its rows by `where:`: `instance:` beside it, or no
/// `where:` at all, is refused (ess/23, beyond10x/ess#459). An assembled specification of one
/// header reaches this with both already refused and taken off by [`refuse_each`]; the refusal here
/// is the backstop for an outcome converted on its own.
fn selected_by_filter(
    name: &OutcomeName,
    index: usize,
    raw: &RawAffect,
) -> Result<(), ValidationErrors> {
    let at = format!("outcomes.{name}.affects[{index}]");
    if raw.instance.is_some() {
        return Err(
            instance_without_each(name.as_str(), index, format!("{at}.instance"), false).into(),
        );
    }
    if raw.filter.is_none() {
        return Err(no_rows_named(name.as_str(), index, at, false).into());
    }
    Ok(())
}

/// The refusal of `instance:` in an `affects:` entry without `each:`, at `at`; below ess/23 it
/// names the format `each:` needs.
fn instance_without_each(name: &str, index: usize, at: String, below_23: bool) -> ValidationError {
    let (message, hint) = if below_23 {
        (
            format!(
                "outcome `{name}` declares `instance:` in `affects[{index}]`, which selects its \
                 rows by `where:`; `instance:` names the row an element of `each:` writes, which \
                 requires specification format ess/23"
            ),
            "drop `instance:`, or declare `format: ess/23` and write one row per element with \
             `each:` and `instance:`",
        )
    } else {
        (
            format!(
                "outcome `{name}` declares `instance:` in `affects[{index}]`, which selects its \
                 rows by `where:`; `instance:` names the row an element of `each:` writes"
            ),
            "drop `instance:`, or write one row per element with `each: {in: input.<list>, as: \
             <name>}` and `instance: <name>.<member>` in place of `where:`",
        )
    };
    ValidationError::new(ValidationCode::ConflictingDeclaration, at, message).with_hint(hint)
}

/// The refusal of an `affects:` entry naming no rows, with neither `where:` nor `each:`, at `at`;
/// below ess/23 it names the format `each:` needs.
fn no_rows_named(name: &str, index: usize, at: String, below_23: bool) -> ValidationError {
    let (message, hint) = if below_23 {
        (
            format!(
                "outcome `{name}` declares `affects[{index}]` with no `where:`, so the entry names \
                 no rows; an entry naming its rows by `each:` requires specification format ess/23"
            ),
            "select the rows with `where: <predicate>`, or declare `format: ess/23` and write one \
             row per element with `each:` and `instance:`",
        )
    } else {
        (
            format!(
                "outcome `{name}` declares `affects[{index}]` with neither `where:` nor `each:`, \
                 so the entry names no rows"
            ),
            "select the rows with `where: <predicate>`, or write one row per element of an input \
             list with `each:` and `instance:`",
        )
    };
    ValidationError::new(ValidationCode::MissingDeclaration, at, message).with_hint(hint)
}

/// The refusal of `target` set more than once in `affects[index]`.
fn set_twice(name: &OutcomeName, index: usize, target: &str) -> ValidationErrors {
    refusal(
        name,
        &format!("affects[{index}].sets.{target}"),
        ValidationCode::DuplicateDeclaration,
        format!("`{target}` is set more than once"),
        "keep the one that is true and delete the other",
    )
}

/// The `each:` of one `affects:` entry as written (ess/23, beyond10x/ess#459): the list it reads,
/// its binder, the member naming each row, and every `sets:` source reading `<as>.<member>`, taken
/// out of `raw.sets`; `None` where the entry selects by `where:`. Refused: `each:` beside `where:`,
/// `moves:` or `deletes:`, neither `each:` nor `where:`, `instance:` without `each:` or `each:`
/// without it, a list that is not `input.<field>`, and a binder or a member read that is not one
/// name.
fn each_entry(
    name: &OutcomeName,
    index: usize,
    raw: &mut RawAffect,
) -> Result<Option<Each>, ValidationErrors> {
    let at = |key: &str| format!("affects[{index}]{key}");
    let refused = |key: &str, code, message: String, hint: &str| {
        Err(refusal(name, &at(key), code, message, hint))
    };
    let Some(each) = raw.each.take() else {
        return selected_by_filter(name, index, raw).map(|()| None);
    };
    for (beside, present) in [
        ("where:", raw.filter.is_some()),
        ("moves:", raw.moves.is_some()),
        ("deletes:", raw.deletes.is_some()),
    ] {
        if present {
            return refused(
                ".each",
                ValidationCode::ConflictingDeclaration,
                format!(
                    "outcome `{name}` declares `each:` and `{beside}` in `affects[{index}]`; an \
                     `each:` entry creates or updates the row each element names, and selects, \
                     moves or removes no other"
                ),
                "keep `each:` with `instance:` and `sets:`, or write the other effect as an entry \
                 of its own",
            );
        }
    }
    let list = each
        .list
        .strip_prefix("input.")
        .filter(|field| crate::binding::is_field_name(field));
    let Some(list) = list else {
        return refused(
            ".each",
            ValidationCode::UnsupportedConstruct,
            format!(
                "outcome `{name}` reads `{}` in `affects[{index}].each`; `each:` walks a list of \
                 the command's input, written `input.<field>`",
                each.list
            ),
            "write `each: {in: input.<field>, as: <name>}`",
        );
    };
    let binder = each.binder;
    if !crate::binding::is_field_name(&binder)
        || binder == super::subject_fact::INPUT_NAMESPACE
        || binder == SUBJECT_NAMESPACE
    {
        return refused(
            ".each",
            ValidationCode::UnsupportedConstruct,
            format!(
                "outcome `{name}` reads each element of `input.{list}` as `{binder}`; the name is \
                 one field name other than `input` and `subject`"
            ),
            "write `as: <name>`, such as `as: item`",
        );
    }
    let member = |read: &str| {
        read.strip_prefix(&format!("{binder}."))
            .filter(|member| crate::binding::is_field_name(member))
            .map(str::to_owned)
    };
    let Some(instance) = raw.instance.take() else {
        return refused(
            ".each",
            ValidationCode::MissingDeclaration,
            format!(
                "outcome `{name}` declares `each:` in `affects[{index}]` and no `instance:`, so no \
                 element says which row it writes"
            ),
            &format!("name each row by a member of the element: `instance: {binder}.<member>`"),
        );
    };
    let Some(identity) = member(&instance) else {
        return refused(
            ".instance",
            ValidationCode::UndeclaredReference,
            format!(
                "outcome `{name}` names each row of `affects[{index}]` by `{instance}`, which is \
                 not one member of the element `{binder}`"
            ),
            &format!("write `instance: {binder}.<member>`"),
        );
    };
    let reads = element_reads(name, index, &binder, &mut raw.sets)?;
    Ok(Some(Each {
        list: list.to_owned(),
        binder,
        member: identity,
        reads,
    }))
}

/// The `sets:` entries of `affects[index]` reading `<binder>.<member>`, as field and member, taken
/// out of `sets` (ess/23, beyond10x/ess#459); or the refusal of one reading deeper than a member,
/// or of a field set twice.
fn element_reads(
    name: &OutcomeName,
    index: usize,
    binder: &str,
    sets: &mut PayloadTable,
) -> Result<BTreeMap<String, String>, ValidationErrors> {
    let prefix = format!("{binder}.");
    let mut reads = BTreeMap::new();
    let mut errors = ValidationErrors::new();
    sets.0.retain(|entry| {
        let PayloadSource::Literal { value } = &entry.source else {
            return true;
        };
        let Some(read) = value.strip_prefix(&prefix) else {
            return true;
        };
        if crate::binding::is_field_name(read) {
            if reads
                .insert(entry.target.clone(), read.to_owned())
                .is_some()
            {
                errors.extend(set_twice(name, index, &entry.target));
            }
        } else {
            errors.extend(refusal(
                name,
                &format!("affects[{index}].sets.{}", entry.target),
                ValidationCode::UnsupportedConstruct,
                format!(
                    "`{}` is set from `{value}`; an element is read one member deep, as \
                     `{binder}.<member>`",
                    entry.target
                ),
                "read one member of the element, or a literal or `input.<field>`",
            ));
        }
        false
    });
    for entry in &sets.0 {
        if reads.contains_key(&entry.target) {
            errors.extend(set_twice(name, index, &entry.target));
        }
    }
    errors.into_result(reads)
}

/// `affects:` is admitted beside one existing subject that `moves:`, `updates:` or — from ess/23,
/// which [`refuse_deletions`] and [`validate`] hold — `deletes:`.
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
        Some(Effect::Moves { .. } | Effect::Updates | Effect::Deletes) => Ok(()),
        Some(effect) => {
            // Below ess/23 the wording is the one it always was; `deletes:` is named only where
            // the source's format admits it (beyond10x/ess#452).
            let (beside, hint) = if admits_deletions() {
                (
                    "`moves:` and `updates:`, and, from ess/23, `deletes:`",
                    "move `affects:` to the branch that moves, updates or deletes an existing row",
                )
            } else {
                (
                    "`moves:` and `updates:`",
                    "move `affects:` to the branch that moves or updates an existing row",
                )
            };
            Err(refusal(
                name,
                "affects",
                ValidationCode::UnsupportedConstruct,
                format!(
                    "outcome `{name}` {} its subject and declares `affects:`; a secondary effect \
                     is admitted beside {beside}, whose subject exists before the outcome",
                    effect.verb()
                ),
                hint,
            ))
        }
        // An entry writing one row per element (ess/23, beyond10x/ess#459) sits beside one
        // subject too; an outcome whose own subject is the list is not part of this cut.
        None if affects.iter().any(|affect| affect.each.is_some()) => Err(refusal(
            name,
            "affects",
            ValidationCode::MissingDeclaration,
            format!(
                "outcome `{name}` declares an `each:` entry in `affects:` and no subject; an entry \
                 writing one row per element of an input list is admitted beside one subject \
                 that the branch moves or updates"
            ),
            "name the branch's subject with `moves:` or `updates:` and `instance:`; a command \
             with no subject of its own cannot write one row per element in this cut",
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

/// The single subject's verbs, as [`written`] takes and gives them back: `moves`, `updates` and
/// `deletes`.
pub(super) type WrittenVerbs = (
    Option<QualifiedName>,
    Option<QualifiedName>,
    Option<QualifiedName>,
);

/// The keys `effects` is written back with: the verb the set subject takes, `instances:` and
/// `affects:`. `moves`, `updates` and `deletes` are the single subject's, and kept where there is
/// no set one.
pub(super) fn written(
    effects: SetEffects,
    (moves, updates, deletes): WrittenVerbs,
) -> (WrittenVerbs, Option<RawInstances>, Vec<RawAffect>) {
    let affects = effects
        .affects
        .into_iter()
        .map(|affect| {
            // An element read is written back as the `<as>.<member>` it was read from (ess/23,
            // beyond10x/ess#459).
            let reads = affect.each.iter().flat_map(|each| {
                each.reads
                    .iter()
                    .map(|(target, member)| super::PayloadField {
                        target: target.clone(),
                        source: PayloadSource::Literal {
                            value: format!("{}.{member}", each.binder),
                        },
                    })
            });
            let sets = affect
                .sets
                .iter()
                .map(|(target, source)| super::PayloadField {
                    target: target.clone(),
                    source: source.clone(),
                })
                .chain(reads)
                .collect();
            RawAffect {
                moves: affect
                    .moves
                    .as_deref()
                    .map(|transition| affect.entity.child(transition)),
                deletes: affect.deletes.then(|| affect.entity.clone()),
                filter: affect.filter.map(Into::into),
                instance: affect
                    .each
                    .as_ref()
                    .map(|each| format!("{}.{}", each.binder, each.member)),
                each: affect.each.map(|each| RawEach {
                    list: format!("input.{}", each.list),
                    binder: each.binder,
                }),
                entity: affect.entity,
                sets: PayloadTable(sets),
            }
        })
        .collect();
    let Some(set) = effects.instances else {
        return ((moves, updates, deletes), None, affects);
    };
    let instances = Some(RawInstances {
        filter: set.filter.into(),
    });
    let verbs = match set.effect {
        Effect::Moves { transition } => (Some(set.entity.child(&transition)), updates, deletes),
        Effect::Deletes => (moves, updates, Some(set.entity)),
        _ => (moves, Some(set.entity), deletes),
    };
    (verbs, instances, affects)
}

/// The transitions set moves take, for the lifecycle-cause check: an `instances:` move and, from
/// ess/22, a move inside an `affects:` entry.
pub(crate) fn performed(
    commands: &BTreeMap<QualifiedName, CommandSpec>,
) -> impl Iterator<Item = (&QualifiedName, &str)> {
    let accepting = || {
        commands
            .values()
            .flat_map(|command| &command.outcomes)
            .filter(|outcome| !outcome.is_refusal())
    };
    let instances = accepting()
        .filter_map(|outcome| outcome.set_effects.instances.as_ref())
        .filter_map(|set| set.effect.transition().map(|move_| (&set.entity, move_)));
    let affects = accepting()
        .flat_map(|outcome| &outcome.set_effects.affects)
        .filter_map(|affect| affect.moves.as_deref().map(|move_| (&affect.entity, move_)));
    instances.chain(affects)
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
#[allow(clippy::too_many_lines)] // One pass over every set effect, construct by construct.
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
            if format.major() < FormatVersion::V23.major() {
                // A backstop for a specification assembled without the one header
                // [`refuse_deletions`] reads.
                errors.extend(deletions_below_ess_23(outcome, &site));
            }
            errors.extend(deletion_shapes(spec, command, outcome, &site));
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
            errors.extend(one_move_per_entity(command, outcome, &site));
            for (index, affect) in outcome.set_effects.affects.iter().enumerate() {
                let at = site.clone().key("affects").index(index);
                if affect.moves.is_some() && format.major() < FormatVersion::V22.major() {
                    // A backstop for a specification assembled without the one header
                    // [`refuse_affect_moves`] reads.
                    errors.push(move_below_ess_22(at.clone().key("moves").render()));
                }
                if affect.each.is_some() && format.major() < FormatVersion::V23.major() {
                    // A backstop for a specification assembled without the one header
                    // [`refuse_each`] reads.
                    errors.push(each_below_ess_23(at.clone().key("each").render()));
                }
                if let Some(entity) =
                    declared(spec, &affect.entity, &at.clone().key("entity"), &mut errors)
                {
                    if let Some(transition) = &affect.moves {
                        if entity.states.transition(transition).is_none() {
                            errors.push(ValidationError::at(
                                at.clone().key("moves"),
                                ValidationCode::UndeclaredReference,
                                format!(
                                    "outcome `{}` of `{}` moves the rows of `affects[{index}]` \
                                     along `{transition}`, which `{}` does not declare as a \
                                     transition",
                                    outcome.name, command.name, affect.entity
                                ),
                            ));
                        }
                    }
                    if let Some(filter) = &affect.filter {
                        errors.extend(check_filter(
                            command,
                            entity,
                            subject,
                            types,
                            filter,
                            &at.clone().key("where"),
                        ));
                    }
                    if let Some(each) = &affect.each {
                        errors.extend(each_rules(
                            spec, command, outcome, entity, affect, each, &at,
                        ));
                    }
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
    check_row_predicate(
        command,
        entity,
        &entity.fields,
        subject,
        types,
        filter,
        at,
        false,
    )
}

/// `predicate` checked over `fields` of a row of `entity`, the command's input under `input.`
/// (unless the entity declares a field named `input`), and — where `subject` is given — the
/// subject's fields under `subject.` (unless the entity declares a field named `subject`). With
/// `current_time`, `now` reads the decision's one instant, as over a stored row (ess/22, A3): the
/// environment a row set's selector and `forall` are checked in (`super::row_set`).
#[allow(clippy::too_many_arguments)]
pub(super) fn check_row_predicate(
    command: &CommandSpec,
    entity: &EntitySpec,
    fields: &[Field],
    subject: Option<&EntitySpec>,
    types: &TypeRegistry,
    filter: &Predicate,
    at: &ConstructRef,
    current_time: bool,
) -> ValidationErrors {
    let owner = at.render();
    let mut inner = DomainEnvironment::new(types, fields);
    if current_time {
        inner = inner.with_stored_current_time();
    }
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
        current_time,
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
    /// Whether `now` is admitted, as over a stored row: a row set's selector (ess/22).
    current_time: bool,
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
    fn is_integer(&self, reference: &Read) -> bool {
        Self::declared(reference).is_some_and(|type_ref| self.inner.is_integer(type_ref))
    }
    fn admits_text_length(&self) -> bool {
        self.inner.admits_text_length()
    }
    fn admits_aggregate_presence(&self) -> bool {
        self.inner.admits_aggregate_presence()
    }
    fn current_time(&self) -> CurrentTimeAdmission {
        let inner = self.inner.current_time();
        CurrentTimeAdmission {
            site: self.current_time && inner.site,
            format: inner.format,
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
                    // A `deletes:` beside it (ess/23, beyond10x/ess#452) goes with it, so no
                    // `missing_declaration` for the `instance:` it never had follows.
                    outcome.deletes = None;
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

/// The refusal of a move inside `affects:` below ess/22, at `at`. The code is the one every format
/// from ess/16 gave it, `unsupported_construct`; the message names the format that admits it.
fn move_below_ess_22(at: String) -> ValidationError {
    ValidationError::new(
        ValidationCode::UnsupportedConstruct,
        at,
        "a move inside `affects:` — `moves: <Entity>.<transition>` on the rows an entry selects — \
         requires specification format ess/22",
    )
    .with_hint(
        "declare `format: ess/22`, or declare `sets:` on the affected rows and move them with a \
         command of their own",
    )
}

/// Refuses, before any outcome is converted, each move inside an `affects:` entry that cannot be
/// read (beyond10x/ess#229), at the key written: under a header from `ess/16` below `ess/22` every
/// such move, naming `ess/22`; from `ess/22` a move naming another entity's transition, or naming
/// no entity.
///
/// The move is taken off the entry and the rest of the entry kept, so the branch converts and the
/// refusal comes alone: no `empty_declaration` for a command whose only branch it refused, and none
/// of the reachability checks that follow from that (`non_exhaustive_branches`,
/// `unreachable_branch`). The transition it named is recorded in `refused_moves` — for a move naming
/// no entity, the entry's entity's transition of that name — so a transition only it took is not
/// also reported as one nothing takes. Below `ess/16` the whole `affects:` is
/// [`refuse_below_ess_16`]'s. A specification of several headers keeps the conversion's refusal of
/// a misnamed move, and [`validate`]'s of the format.
pub(crate) fn refuse_affect_moves(
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
    let major = format.unwrap_or(FormatVersion::V1).major();
    if major < FormatVersion::V16.major() {
        return;
    }
    let below_22 = major < FormatVersion::V22.major();
    for (_, file) in files.iter_mut() {
        for command in &mut file.commands {
            for outcome in &mut command.outcomes {
                for (index, affect) in outcome.affects.iter_mut().enumerate() {
                    let Some(moved) = &affect.moves else {
                        continue;
                    };
                    let at = format!(
                        "command.{}.outcomes.{}.affects[{index}].moves",
                        command.name, outcome.name
                    );
                    let refusal = if below_22 {
                        move_below_ess_22(at)
                    } else {
                        let Some((code, message, hint)) =
                            misnamed_move(outcome.name.as_str(), &affect.entity, moved)
                        else {
                            continue;
                        };
                        ValidationError::new(code, at, message).with_hint(hint)
                    };
                    errors.push(refusal);
                    let named = if moved.namespace().is_some() {
                        moved.clone()
                    } else {
                        affect.entity.child(moved.local())
                    };
                    refused_moves.insert(named);
                    affect.moves = None;
                }
            }
        }
    }
}

/// The refusal of `instances:` beside `deletes:` below ess/23, at `at`: the code and message it
/// always had, naming the format that admits it (beyond10x/ess#452).
fn bulk_deletion_below_ess_23(name: &str, at: String) -> ValidationError {
    ValidationError::new(
        ValidationCode::UnsupportedConstruct,
        at,
        format!(
            "outcome `{name}` deletes an entity and declares `instances:`; a set subject is \
             admitted beside `moves:` and `updates:` only, and beside `deletes:` from \
             specification format ess/23"
        ),
    )
    .with_hint("declare `format: ess/23`, or name one row with `instance:`")
}

/// The refusal of `affects:` beside a `deletes:` subject below ess/23, at `at`: the code and
/// message it always had, naming the format that admits it (beyond10x/ess#452).
fn affects_beside_deletion_below_ess_23(name: &str, at: String) -> ValidationError {
    ValidationError::new(
        ValidationCode::UnsupportedConstruct,
        at,
        format!(
            "outcome `{name}` deletes its subject and declares `affects:`; a secondary effect is \
             admitted beside `moves:` and `updates:`, whose subject exists before the outcome, \
             and beside `deletes:` from specification format ess/23"
        ),
    )
    .with_hint(
        "declare `format: ess/23`, or move `affects:` to the branch that moves or updates an \
         existing row",
    )
}

/// The refusal of `deletes:` inside an `affects:` entry below ess/23, at `at` (beyond10x/ess#452).
fn affect_deletion_below_ess_23(at: String) -> ValidationError {
    ValidationError::new(
        ValidationCode::UnsupportedFormatVersion,
        at,
        "a deletion inside `affects:` — `deletes: <Entity>` on the rows an entry selects — \
         requires specification format ess/23",
    )
    .with_hint("declare `format: ess/23`")
}

/// Every deletion a set effect declares under a format below ess/23, refused where an assembled
/// specification did not already take it off before conversion ([`refuse_deletions`]).
fn deletions_below_ess_23(outcome: &Outcome, site: &ConstructRef) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    let name = outcome.name.as_str();
    if outcome
        .set_effects
        .instances
        .as_ref()
        .is_some_and(|set| set.effect == Effect::Deletes)
    {
        errors.push(bulk_deletion_below_ess_23(
            name,
            site.clone().key("instances").render(),
        ));
    }
    let deletes_subject = outcome
        .subject
        .as_ref()
        .is_some_and(|subject| subject.effect == Effect::Deletes);
    if deletes_subject && !outcome.set_effects.affects.is_empty() {
        errors.push(affects_beside_deletion_below_ess_23(
            name,
            site.clone().key("affects").render(),
        ));
    }
    for (index, affect) in outcome.set_effects.affects.iter().enumerate() {
        if affect.deletes {
            errors.push(affect_deletion_below_ess_23(
                site.clone()
                    .key("affects")
                    .index(index)
                    .key("deletes")
                    .render(),
            ));
        }
    }
    errors
}

/// A deleting `affects:` entry over an entity another domain than the command's owns, refused at
/// the entry (ess/23, beyond10x/ess#452): removal in another domain is the cascade the story
/// declines, answered with one event and one binding per receiving domain.
fn deletion_across_domains(
    spec: &Specification,
    command: &CommandSpec,
    outcome: &Outcome,
    affect: &Affect,
    at: &ConstructRef,
) -> Option<ValidationError> {
    let system = spec.system();
    let own = system.owner_of(&command.name)?;
    let other = system.owner_of(&affect.entity)?;
    (own.name != other.name).then(|| {
        ValidationError::at(
            at.clone(),
            ValidationCode::UnsupportedConstruct,
            format!(
                "outcome `{}` of `{}` deletes rows of `{}`, which domain `{}` owns, from domain \
                 `{}`; `affects:` stays inside its outcome's domain, and removal in another domain \
                 is one event and one binding per receiving domain",
                outcome.name, command.name, affect.entity, other.name, own.name
            ),
        )
        .with_hint(format!(
            "emit an event from this branch and bind it to a command of `{}` that deletes its own \
             rows with `instances:`, with that binding's own `delivery:` and `on_failure:`",
            other.name
        ))
    })
}

/// The shape of every deletion a set effect declares (ess/23, beyond10x/ess#452): no `sets:` beside
/// a bulk `deletes:` or a deleting entry, no `moves:` in a deleting entry, a deleting entry alone
/// over its entity among the entries that change rows, and a deleting entry over an entity of the
/// command's own domain only.
fn deletion_shapes(
    spec: &Specification,
    command: &CommandSpec,
    outcome: &Outcome,
    site: &ConstructRef,
) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    if let Some(set) = outcome
        .set_effects
        .instances
        .as_ref()
        .filter(|set| set.effect == Effect::Deletes && !outcome.sets.is_empty())
    {
        errors.push(
            ValidationError::at(
                site.clone().key("sets"),
                ValidationCode::ConflictingDeclaration,
                format!(
                    "outcome `{}` of `{}` deletes every `{}` its filter selects and declares \
                     `sets:`; a removed row holds nothing to set, so `sets:` beside `deletes:` \
                     writes nothing",
                    outcome.name, command.name, set.entity
                ),
            )
            .with_hint("drop `sets:`, or split the branch into a set update and a set deletion"),
        );
    }
    let affects = &outcome.set_effects.affects;
    for (index, affect) in affects.iter().enumerate() {
        let at = site.clone().key("affects").index(index);
        if affect.deletes && !affect.sets.is_empty() {
            errors.push(
                ValidationError::at(
                    at.clone().key("sets"),
                    ValidationCode::ConflictingDeclaration,
                    format!(
                        "outcome `{}` of `{}` deletes the rows `affects[{index}]` selects and \
                         declares `sets:` on them; a removed row holds nothing to set",
                        outcome.name, command.name
                    ),
                )
                .with_hint("drop `sets:` from the deleting entry"),
            );
        }
        if affect.deletes && affect.moves.is_some() {
            errors.push(
                ValidationError::at(
                    at.clone().key("moves"),
                    ValidationCode::ConflictingDeclaration,
                    format!(
                        "outcome `{}` of `{}` both moves and deletes the rows `affects[{index}]` \
                         selects; one entry does one thing to its rows",
                        outcome.name, command.name
                    ),
                )
                .with_hint("keep `deletes:`, or keep `moves:` and remove the rows with a command of their own"),
            );
        }
        if let Some(across) = affect
            .deletes
            .then(|| deletion_across_domains(spec, command, outcome, affect, &at))
            .flatten()
        {
            errors.push(across);
        }
        // A deleting entry beside any other entry over the same entity, refused at the second of
        // the two: a row both select would be changed and removed, in no defined order. An entry
        // that changes no row — one whose `deletes:` was refused and taken off — is beside nothing.
        let changes =
            |entry: &Affect| entry.deletes || entry.moves.is_some() || !entry.sets.is_empty();
        if !changes(affect) {
            continue;
        }
        let Some(first) = affects[..index].iter().position(|earlier| {
            changes(earlier)
                && earlier.entity == affect.entity
                && (earlier.deletes || affect.deletes)
        }) else {
            continue;
        };
        errors.push(
            ValidationError::at(
                at,
                ValidationCode::ConflictingDeclaration,
                format!(
                    "outcome `{}` of `{}` declares `affects[{first}]` and `affects[{index}]` over \
                     `{}`, and one of them deletes the rows it selects; a row both select would \
                     be changed and removed, and no order between them is defined",
                    outcome.name, command.name, affect.entity
                ),
            )
            .with_hint(
                "keep the deleting entry alone over its entity, or change the other rows with a \
                 command of their own",
            ),
        );
    }
    errors
}

/// Refuses, before any outcome is converted, each deletion a set effect cannot declare under the
/// one header (beyond10x/ess#452), at the key written: from `ess/16` below `ess/23` `instances:`
/// beside `deletes:`, `affects:` beside a `deletes:` subject, and `deletes:` inside an `affects:`
/// entry, each naming `ess/23`; from `ess/23` an entry's `deletes:` naming another entity than the
/// entry's.
///
/// What is refused is taken off — `instances:` with its `deletes:` (and with them a `{count:
/// changed}` and an `affects:` the branch could only hold beside a set subject), `affects:`, or the
/// entry's `deletes:` key, the entry itself kept so every later entry keeps its position and the
/// entry's own entity is still checked — so the branch converts and the refusal comes alone: no
/// `empty_declaration` for a command whose only branch it refused. A bulk deletion the conversion
/// refuses first under every format — beside `instance:`, or with a filter that does not parse —
/// is left to it, with the refusal it always had. Below `ess/16` the whole construct is
/// [`refuse_below_ess_16`]'s; a specification of several headers keeps the conversion's refusal
/// and [`validate`]'s.
pub(crate) fn refuse_deletions(
    files: &mut [(crate::system::Source, crate::spec::RawSpecFile)],
    errors: &mut ValidationErrors,
) {
    let headers: Vec<Option<FormatVersion>> = files
        .iter()
        .filter(|(_, file)| file.system.is_some())
        .map(|(_, file)| file.format)
        .collect();
    let [format] = headers.as_slice() else {
        return;
    };
    let major = format.unwrap_or(FormatVersion::V1).major();
    if major < FormatVersion::V16.major() {
        return;
    }
    let below_23 = major < FormatVersion::V23.major();
    for (_, file) in files.iter_mut() {
        for command in &mut file.commands {
            for outcome in &mut command.outcomes {
                let at = format!("command.{}.outcomes.{}", command.name, outcome.name);
                let name = outcome.name.as_str().to_owned();
                if below_23 && outcome.deletes.is_some() {
                    if let Some(instances) = &outcome.instances {
                        if outcome.instance.is_some() || instances.filter.predicate().is_none() {
                            // The conversion refuses `instance:` beside `instances:`, or the
                            // filter, before it reads the verb, as it always did.
                            continue;
                        }
                        errors.push(bulk_deletion_below_ess_23(&name, format!("{at}.instances")));
                        outcome.instances = None;
                        outcome.deletes = None;
                        outcome.affects.clear();
                        for (_, table) in &mut outcome.payload.0 {
                            for field in &mut table.0 {
                                if field.source == PayloadSource::ChangedCount {
                                    field.source = PayloadSource::Generated;
                                }
                            }
                        }
                        continue;
                    }
                    if !outcome.affects.is_empty() {
                        errors.push(affects_beside_deletion_below_ess_23(
                            &name,
                            format!("{at}.affects"),
                        ));
                        outcome.affects.clear();
                        continue;
                    }
                }
                for (index, affect) in outcome.affects.iter_mut().enumerate() {
                    let Some(deleted) = &affect.deletes else {
                        continue;
                    };
                    let at = format!("{at}.affects[{index}]");
                    let refusal = if below_23 {
                        affect_deletion_below_ess_23(format!("{at}.deletes"))
                    } else {
                        let Some((message, hint)) =
                            misnamed_deletion(&name, &affect.entity, deleted)
                        else {
                            continue;
                        };
                        ValidationError::new(ValidationCode::ConflictingDeclaration, at, message)
                            .with_hint(hint)
                    };
                    errors.push(refusal);
                    affect.deletes = None;
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

/// The refusal of `each:` inside an `affects:` entry below ess/23, at `at` (beyond10x/ess#459).
fn each_below_ess_23(at: String) -> ValidationError {
    ValidationError::new(
        ValidationCode::UnsupportedFormatVersion,
        at,
        "an entry writing one row per element of an input list — `each: {in: input.<list>, as: \
         <name>}` inside `affects:` — requires specification format ess/23",
    )
    .with_hint("declare `format: ess/23`")
}

/// Refuses, before any outcome is converted, under one header from `ess/16` (beyond10x/ess#459):
/// below `ess/23` each `each:` inside an `affects:` entry, naming `ess/23`; and in every such format
/// an entry without `each:` that declares `instance:` or no `where:`, naming `ess/23` below it. The
/// refused `instance:` is taken off, and an entry naming no rows is given a `where:` selecting
/// every row, so the refusal comes alone.
///
/// The entry keeps its place, so every later entry's refusal names the position written, and is
/// left changing nothing: its `each:`, `instance:` and `sets:` — whose element reads would be read
/// as literals — are taken off, and an entry with no `where:` is given one selecting every row, so
/// the refusal comes alone: no `missing_declaration` for the `where:` it never had, no
/// `type_mismatch` for an element read, and no `empty_declaration`. Below `ess/16` the whole
/// `affects:` is [`refuse_below_ess_16`]'s; a specification of several headers keeps
/// [`validate`]'s refusal.
pub(crate) fn refuse_each(
    files: &mut [(crate::system::Source, crate::spec::RawSpecFile)],
    errors: &mut ValidationErrors,
) {
    let headers: Vec<Option<FormatVersion>> = files
        .iter()
        .filter(|(_, file)| file.system.is_some())
        .map(|(_, file)| file.format)
        .collect();
    let [format] = headers.as_slice() else {
        return;
    };
    let major = format.unwrap_or(FormatVersion::V1).major();
    if major < FormatVersion::V16.major() {
        return;
    }
    let below_23 = major < FormatVersion::V23.major();
    for (_, file) in files.iter_mut() {
        for command in &mut file.commands {
            for outcome in &mut command.outcomes {
                let name = outcome.name.as_str().to_owned();
                for (index, affect) in outcome.affects.iter_mut().enumerate() {
                    let at = format!("command.{}.outcomes.{name}.affects[{index}]", command.name);
                    if below_23 && affect.each.take().is_some() {
                        errors.push(each_below_ess_23(format!("{at}.each")));
                        affect.instance = None;
                        affect.sets.0.clear();
                        affect
                            .filter
                            .get_or_insert_with(|| Predicate::Always.into());
                        continue;
                    }
                    if affect.each.is_some() {
                        continue;
                    }
                    if affect.instance.take().is_some() {
                        errors.push(instance_without_each(
                            &name,
                            index,
                            format!("{at}.instance"),
                            below_23,
                        ));
                    }
                    if affect.filter.is_none() {
                        errors.push(no_rows_named(&name, index, at, below_23));
                        affect.filter = Some(Predicate::Always.into());
                    }
                }
            }
        }
    }
}

/// The rules of an `each:` entry that need the whole specification (ess/23, beyond10x/ess#459):
/// the list is a declared input holding a `List` of a struct; `instance:` names a member of the
/// entity's identity type; that member is held distinct across the list by a declared `distinct:`;
/// each element read names a member of the type of the field it fills, and not the identity; no
/// element read sits inside a nested mapping; and every required field an entity invariant reads is
/// written, as by any creation (`ESS-COMMAND-018`).
#[allow(clippy::too_many_lines)] // One rule per paragraph of the design note's section.
fn each_rules(
    spec: &Specification,
    command: &CommandSpec,
    outcome: &Outcome,
    entity: &EntitySpec,
    affect: &Affect,
    each: &Each,
    at: &ConstructRef,
) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    let site = at.clone().key("each");
    let binder = &each.binder;
    let Some(list) = command.input.iter().find(|field| field.name == each.list) else {
        errors.push(
            ValidationError::at(
                site,
                ValidationCode::UndeclaredReference,
                format!(
                    "`each:` reads `input.{}`, which `{}` does not declare as an input",
                    each.list, command.name
                ),
            )
            .with_hint("declare the list among the command's `input:`"),
        );
        return errors;
    };
    let element = match &list.type_ref {
        TypeRef::List(of) => match of.as_ref() {
            TypeRef::Named(named) => spec
                .system()
                .types
                .get(named)
                .filter(|named| matches!(named.body, crate::types::TypeBody::Struct { .. })),
            _ => None,
        },
        _ => None,
    };
    let Some(element) = element else {
        errors.push(
            ValidationError::at(
                site,
                ValidationCode::TypeMismatch,
                format!(
                    "`each:` reads `input.{}`, which is `{}`; it walks a `List` of a struct whose \
                     members name and fill each row",
                    each.list, list.type_ref
                ),
            )
            .with_hint(format!(
                "declare `{}` as `List<…>` of a struct with a member of `{}`",
                each.list, entity.identity.type_ref
            )),
        );
        return errors;
    };
    let instance = at.clone().key("instance");
    match element.field(&each.member) {
        None => errors.push(
            ValidationError::at(
                instance,
                ValidationCode::UndeclaredReference,
                format!(
                    "`instance: {binder}.{}` reads a member `{}` does not declare",
                    each.member, element.name
                ),
            )
            .with_hint(format!(
                "name each row by a member of `{}` holding a `{}`",
                element.name, entity.identity.type_ref
            )),
        ),
        Some(field) if field.type_ref != entity.identity.type_ref => errors.push(
            ValidationError::at(
                instance,
                ValidationCode::TypeMismatch,
                format!(
                    "`instance: {binder}.{}` is `{}`, and the identity `{}` of `{}` is `{}`",
                    each.member,
                    field.type_ref,
                    entity.identity.name,
                    entity.name,
                    entity.identity.type_ref
                ),
            )
            .with_hint("name each row by a member holding the entity's identity type"),
        ),
        Some(_) => {
            if !held_distinct(command, outcome, each) {
                errors.push(
                    ValidationError::at(
                        site,
                        ValidationCode::MissingDeclaration,
                        format!(
                            "outcome `{}` of `{}` writes one `{}` per element of `input.{}`, \
                             named by `{binder}.{}`, and no `distinct:` holds that member distinct \
                             across the list; two elements naming one row would leave a result \
                             that depends on their order",
                            outcome.name, command.name, entity.name, each.list, each.member
                        ),
                    )
                    .with_hint(format!(
                        "refuse a repeated member with a branch `when: {{not: {{distinct: {{in: \
                         {list}, as: item, by: item.{member}}}}}}}`, or require that `distinct:` \
                         in this branch's own `when:`",
                        list = each.list,
                        member = each.member
                    )),
                );
            }
        }
    }
    for (target, member) in &each.reads {
        let read_at = at.clone().key("sets").named(target);
        if *target == entity.identity.name {
            errors.push(
                ValidationError::at(
                    read_at,
                    ValidationCode::ConflictingDeclaration,
                    format!(
                        "`{target}` is the identity of `{}`, which an `each:` entry takes from \
                         `instance:`",
                        entity.name
                    ),
                )
                .with_hint("drop the identity from `sets:`"),
            );
            continue;
        }
        let Some(filled) = entity.field(target) else {
            errors.push(ValidationError::at(
                read_at,
                ValidationCode::UndeclaredReference,
                format!("`{target}` is not a field of `{}`", entity.name),
            ));
            continue;
        };
        let Some(read) = element.field(member) else {
            errors.push(
                ValidationError::at(
                    read_at,
                    ValidationCode::UndeclaredReference,
                    format!(
                        "`{target}` reads `{binder}.{member}`, a member `{}` does not declare",
                        element.name
                    ),
                )
                .with_hint(format!(
                    "read a declared member of `{}`, written `{binder}.<member>`",
                    element.name
                )),
            );
            continue;
        };
        let admitted = read.type_ref == filled.type_ref
            || matches!(&filled.type_ref, TypeRef::Optional(of) if **of == read.type_ref);
        if !admitted {
            errors.push(
                ValidationError::at(
                    read_at,
                    ValidationCode::TypeMismatch,
                    format!(
                        "`{target}` holds `{}`, and `{binder}.{member}` is `{}`",
                        filled.type_ref, read.type_ref
                    ),
                )
                .with_hint("read a member of the field's own type"),
            );
        }
    }
    let prefix = format!("{binder}.");
    for (target, source) in &affect.sets {
        visit(source, &mut |leaf| {
            if matches!(leaf, PayloadSource::Literal { value } if value.starts_with(&prefix)) {
                errors.push(
                    ValidationError::at(
                        at.clone().key("sets").named(target),
                        ValidationCode::UnsupportedConstruct,
                        format!(
                            "`{target}` reads the element `{binder}` inside a nested mapping; an \
                             element member fills a whole field in this cut"
                        ),
                    )
                    .with_hint("fill the whole field from one member of the element"),
                );
            }
        });
    }
    for field in &entity.fields {
        if field.type_ref.is_optional()
            || affect.sets.contains_key(&field.name)
            || each.reads.contains_key(&field.name)
        {
            continue;
        }
        let read = entity.invariants.iter().any(|invariant| {
            invariant
                .predicate
                .fact_paths()
                .iter()
                .any(|path| path.namespace() == field.name)
        });
        if read {
            errors.push(
                ValidationError::at(
                    at.clone(),
                    ValidationCode::InvariantReadsUnsetField,
                    format!(
                        "outcome `{}` of `{}` may create a `{}` per element without setting `{}`, \
                         which an invariant of `{}` reads; a row it creates would hold no \
                         specified value there",
                        outcome.name, command.name, entity.name, field.name, entity.name
                    ),
                )
                .with_hint(format!(
                    "set `{}` in the entry's `sets:`, from the element or the input",
                    field.name
                )),
            );
        }
    }
    errors
}

/// Whether a declared `distinct:` holds `each`'s identity member distinct across its list: conjoined
/// in the outcome's own guard, or refused by a refusal of the command whose guard holds where it
/// does not (`not distinct`, alone or among the alternatives of an `any:`), a refusal whose
/// condition is a plain `when:` over the input.
fn held_distinct(command: &CommandSpec, outcome: &Outcome, each: &Each) -> bool {
    let names = |distinct: &ess_primitives::predicate::Distinct| {
        let list = match distinct.over.segments() {
            [only] => *only == each.list,
            [root, only] => root == super::subject_fact::INPUT_NAMESPACE && *only == each.list,
            _ => false,
        };
        list && distinct.key.as_ref().is_some_and(|key| {
            matches!(key.segments(), [bind, member]
                if *bind == distinct.bind && *member == each.member)
        })
    };
    input_guard(&outcome.condition).is_some_and(|predicate| conjoined(predicate, &names))
        || command
            .outcomes
            .iter()
            .filter(|other| other.error.is_some())
            // Only a refusal guarded by its input alone answers every repeated list: one scoped by
            // the stored subject or a related row refuses only where that guard holds too.
            .filter_map(|other| match &other.condition {
                super::OutcomeCondition::When(predicate) => Some(predicate),
                _ => None,
            })
            .any(|predicate| refuses(predicate, &names))
}

/// Whether `predicate` holds only where a `distinct:` `names` admits holds: that `distinct:`, or
/// a conjunction with it among its conjuncts.
fn conjoined(
    predicate: &Predicate,
    names: &dyn Fn(&ess_primitives::predicate::Distinct) -> bool,
) -> bool {
    match predicate {
        Predicate::Distinct(distinct) => names(distinct),
        Predicate::All(children) => children.iter().any(|child| conjoined(child, names)),
        _ => false,
    }
}

/// Whether `predicate` holds wherever a `distinct:` `names` admits does not: its negation, or a
/// disjunction with it among its alternatives.
fn refuses(
    predicate: &Predicate,
    names: &dyn Fn(&ess_primitives::predicate::Distinct) -> bool,
) -> bool {
    match predicate {
        Predicate::Not(inner) => conjoined(inner, names),
        Predicate::Any(children) => children.iter().any(|child| refuses(child, names)),
        _ => false,
    }
}

/// The predicate over the input a branch's condition carries, where it carries one.
fn input_guard(condition: &super::OutcomeCondition) -> Option<&Predicate> {
    match condition {
        super::OutcomeCondition::When(predicate) => Some(predicate),
        super::OutcomeCondition::SubjectField { predicate, .. } => predicate.as_ref(),
        super::OutcomeCondition::SubjectPredicate { input, .. }
        | super::OutcomeCondition::Related { input, .. } => input.as_ref(),
        _ => None,
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

/// One transition per entity per outcome (beyond10x/ess#229): a second `affects:` entry declaring
/// `moves:` over an entity an earlier entry already moves is refused at its `moves`, since a row
/// both select would have to take two, and no order between their moves is defined.
fn one_move_per_entity(
    command: &CommandSpec,
    outcome: &Outcome,
    site: &ConstructRef,
) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    let mut moving: BTreeMap<&QualifiedName, usize> = BTreeMap::new();
    for (index, affect) in outcome.set_effects.affects.iter().enumerate() {
        if affect.moves.is_none() {
            continue;
        }
        let Some(first) = moving.get(&affect.entity) else {
            moving.insert(&affect.entity, index);
            continue;
        };
        errors.push(
            ValidationError::at(
                site.clone().key("affects").index(index).key("moves"),
                ValidationCode::ConflictingDeclaration,
                format!(
                    "outcome `{}` of `{}` moves rows of `{}` in `affects[{first}]` and again in \
                     `affects[{index}]`; an outcome takes one transition per entity, since one \
                     row both entries select would have to take two",
                    outcome.name, command.name, affect.entity
                ),
            )
            .with_hint(
                "keep one moving entry per entity, widen its filter to the rows both select, or \
                 move the others with a command of their own",
            ),
        );
    }
    errors
}
