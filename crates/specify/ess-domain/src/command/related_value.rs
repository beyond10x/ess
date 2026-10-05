//! `{related: {via: <field>, field: <field>}}`: a value read from a field of the row another row
//! references (source format `ess/16`, beyond10x/ess#166, `docs/design/value-expressions.md` E8).
//!
//! The subject (or the command's input) holds the other row's identity; the value lives on that
//! row. From `ess/22` (beyond10x/ess#285) the reference may be `Optional<…>` — the value is then
//! absent where it is — and `via` may name a second reference on that row. What this module owns
//! is the one question the validator and the compiler must answer alike: **which entity a
//! reference names**.

use std::fmt;

use crate::entity::{Cardinality, EntitySpec, RelationKind, RelationSpec};
use crate::types::TypeRef;
use crate::Specification;

/// Where a related source reads the other row's identity.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RelatedVia {
    /// A field of the addressed entity, as it was immediately before the outcome: `via: customer_id`.
    Subject(String),
    /// A field of the command's input: `via: input.customer_id`.
    Input(String),
}

impl RelatedVia {
    /// Reads `input.<field>` as the input's field and anything else as the subject's.
    pub fn parse(written: &str) -> Self {
        match written.strip_prefix(super::PayloadSource::INPUT_PREFIX) {
            Some(field) => Self::Input(field.to_owned()),
            None => Self::Subject(written.to_owned()),
        }
    }

    /// The field read, without its prefix.
    pub fn field(&self) -> &str {
        match self {
            Self::Subject(field) | Self::Input(field) => field,
        }
    }
}

impl PartialEq<str> for RelatedVia {
    /// Compares the field read, without its prefix.
    fn eq(&self, field: &str) -> bool {
        self.field() == field
    }
}

impl fmt::Display for RelatedVia {
    /// As the document wrote it.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Subject(field) => f.write_str(field),
            Self::Input(field) => write!(f, "{}{field}", super::PayloadSource::INPUT_PREFIX),
        }
    }
}

/// Which entity a `via` field names.
#[derive(Debug)]
pub enum Referenced<'a> {
    /// The one entity it names.
    Entity(&'a EntitySpec),
    /// Its type is no entity's identity.
    NoEntity,
    /// Its type is the identity of several entities, and no `references` relation says which.
    Ambiguous(Vec<&'a EntitySpec>),
}

/// The entity a `via` field of type `via_type` names.
///
/// The relation `carrier` (the subject entity and its field) carries decides it: a `references`
/// relation of cardinality `one` the subject declares on the field, or an `owns` relation another
/// entity declares over the subject through the field — its owner. Without one, the entity whose identity is exactly `via_type` does, where
/// there is one. The identity must be the type itself — not `Optional<…>`, not `List<…>` — because
/// the source reads one row; callers refuse a list, and pass an `Optional<…>` reference (ess/22,
/// beyond10x/ess#285) unwrapped, before asking.
pub fn referenced_entity<'a>(
    spec: &'a Specification,
    via_type: &TypeRef,
    carrier: Option<(&EntitySpec, &str)>,
) -> Referenced<'a> {
    let declared = carrier.and_then(|(entity, field)| {
        entity.relations.iter().find(|relation| {
            relation.kind == RelationKind::References
                && relation.cardinality == Cardinality::One
                && relation.via == field
        })
    });
    let referenced = declared.and_then(|relation| spec.entities().get(&relation.target));
    // A field carries one relation, so an owned subject names its owner through the `owns` the
    // owner declares, and cannot declare a `references` on the same field as well.
    let owner = carrier.and_then(|(entity, field)| {
        spec.entities().values().find(|owner| {
            owner.relations.iter().any(|relation| {
                relation.kind == RelationKind::Owns
                    && relation.target == entity.name
                    && relation.via == field
            })
        })
    });
    if let Some(target) = referenced.or(owner) {
        if target.identity.type_ref == *via_type {
            return Referenced::Entity(target);
        }
    }
    let mut named: Vec<&EntitySpec> = spec
        .entities()
        .values()
        .filter(|entity| entity.identity.type_ref == *via_type)
        .collect();
    match named.len() {
        0 => Referenced::NoEntity,
        1 => Referenced::Entity(named.remove(0)),
        _ => Referenced::Ambiguous(named),
    }
}

/// Reads every `{related: {via, field}}` of a document below `ess/16` back as the nested mapping it
/// was before that format existed: a struct field `related` whose own fields `via` and `field` hold
/// the texts written, each read as a bare text is (`input.<field>` or a literal).
///
/// The reader cannot know the format — a document's header may be in another file — so it
/// recognises the exact shape in every format, and the assembled specification undoes that below
/// `ess/16`. Without this, an `ess/14` or `ess/15` document whose struct has a field `related`
/// filled that way would change meaning.
pub fn read_below_ess_16(
    format: crate::system::FormatVersion,
    commands: &mut std::collections::BTreeMap<crate::name::QualifiedName, super::CommandSpec>,
) {
    if format.major() >= crate::system::FormatVersion::V16.major() {
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

/// One source, and every leaf of a nested mapping, with a related source read as its mapping.
fn nested(source: &mut super::PayloadSource) {
    use super::{PayloadField, PayloadSource};
    match source {
        // A chained `via:` was no mapping before `ess/22` (beyond10x/ess#285) — a list of texts
        // was refused when read — so it has no earlier meaning to restore, and stays the source
        // for the format check to refuse.
        PayloadSource::RelatedField {
            via,
            through,
            field,
        } if through.is_empty() => {
            let leaf = |target: &str, text: String| PayloadField {
                target: target.to_owned(),
                source: PayloadSource::parse(&text),
            };
            *source = PayloadSource::Struct {
                fields: vec![PayloadField {
                    target: "related".to_owned(),
                    source: PayloadSource::Struct {
                        fields: vec![leaf("via", via.to_string()), leaf("field", field.clone())],
                    },
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

/// The input a branch's `sets:` writes into the subject field `field` unchanged (`field:
/// input.<name>`), where it does.
///
/// On a `creates:` branch the subject field `via` names has no value before the outcome, but where
/// the branch fills it from its input it holds that input, so `{related: {via: <field>}}` reads the
/// input — and the relation the field carries says which entity it names.
pub fn written_from_input<'a>(outcome: &'a super::Outcome, field: &str) -> Option<&'a str> {
    match outcome.sets.get(field)? {
        super::PayloadSource::InputField { field: input } => Some(input),
        _ => None,
    }
}

/// The relation `subject`'s identity carries, where it carries one that makes the identity a
/// carrier: a `references` of `cardinality: one` to **another** entity — a one-to-one link keyed by
/// the same id (beyond10x/ess#230).
///
/// This is the one gate every place that counts the identity as a relation carrier asks. An
/// identity that carries no such relation carries nothing, so a model that does not use the
/// construct reads exactly as before it existed; and an identity never names the row it is on.
pub fn identity_reference(subject: &EntitySpec) -> Option<&RelationSpec> {
    subject.relations.iter().find(|relation| {
        relation.kind == RelationKind::References
            && relation.cardinality == Cardinality::One
            && relation.via == subject.identity.name
            && relation.target != subject.name
    })
}

/// [`written_from_input`], and for the subject's identity the input the branch names the instance
/// by ([`identity_from_input`]) — only where the identity carries a relation to another entity
/// ([`identity_reference`]).
///
/// On a `creates:` branch that identity holds the input it is filled from exactly as a field
/// `sets:` fills does, so a related read through it reads that input, and the relation says it
/// names another entity's row. Without the relation the identity names only the row being
/// created, which does not exist before the outcome, and it is not admitted.
pub fn subject_field_from_input<'a>(
    outcome: &'a super::Outcome,
    subject: &EntitySpec,
    field: &str,
) -> Option<&'a str> {
    written_from_input(outcome, field).or_else(|| {
        (subject.identity.name == field && identity_reference(subject).is_some())
            .then(|| identity_from_input(outcome))
            .flatten()
    })
}

/// The input a branch names its subject's instance by, unchanged: the input field `instance:`
/// names where the caller supplies the instance, or — on `creates:`, where `instance:` names the
/// event field that publishes the new identity — the input every emitted payload fills that field
/// from, where they agree on one.
pub fn identity_from_input(outcome: &super::Outcome) -> Option<&str> {
    let subject = outcome.subject.as_ref()?;
    if subject.effect != super::Effect::Creates {
        return Some(&subject.instance);
    }
    let mut filled = outcome
        .payload
        .values()
        .filter_map(|fields| fields.get(&subject.instance))
        .map(|source| match source {
            super::PayloadSource::InputField { field } => Some(field.as_str()),
            _ => None,
        });
    let first = filled.next()??;
    filled.all(|other| other == Some(first)).then_some(first)
}

/// The subject field a branch's `sets:` fills from the input `input` unchanged, and the subject:
/// where a relation on that field says which entity the input names.
///
/// The input the branch names its instance by is carried by the subject's identity
/// ([`identity_from_input`]), where no `sets:` field claims it and the identity carries a relation
/// to another entity ([`identity_reference`], beyond10x/ess#230). An identity that carries none is
/// no carrier, so a later branch's field relation still decides.
pub fn input_carrier<'a>(
    outcome: &super::Outcome,
    subject: Option<&'a EntitySpec>,
    input: &str,
) -> Option<(&'a EntitySpec, String)> {
    let subject = subject?;
    outcome
        .sets
        .iter()
        .find_map(|(target, source)| match source {
            super::PayloadSource::InputField { field } if field == input => {
                Some((subject, target.clone()))
            }
            _ => None,
        })
        .or_else(|| {
            (identity_reference(subject).is_some() && identity_from_input(outcome) == Some(input))
                .then(|| (subject, subject.identity.name.clone()))
        })
}
