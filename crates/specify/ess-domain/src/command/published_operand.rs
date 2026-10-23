//! A selector reads a value its own outcome publishes: `payload.<Event>.<field>[.<member>…]` in the
//! `where:` of `{related: {entity, where, field}}` and of `{related: {entity, where, order_by,
//! first}}` (source format `ess/27`, beyond10x/ess#549, `docs/design/published-value-operand.md`).
//!
//! ```yaml
//! left_key:
//!   related:
//!     entity: demo.nested.Membership
//!     where: {all: [slot_id == payload.demo.nested.PairMade.slot_id, side == Left]}
//!     order_by: [element_score desc, membership_id]
//!     first: element_key
//! ```
//!
//! The operand is the value this outcome publishes in `<field>` of `<Event>`, typed at the event
//! field's declared type. The candidate row's bare fields are resolved first: an entity declaring a
//! field named `payload` keeps reading `payload.<member>` as that field, at every format.
//!
//! The read field is one this outcome emits and fills, and its source is a **snapshot value**,
//! decided by the request and the store before the outcome: not a mint, not the response, not
//! `{count: changed}`, and not the identity a creating outcome publishes. A guard decides the branch
//! before anything is published, an effect selecting rows by a published value is not admitted in
//! this cut, and an error outcome publishes no event payload; each refuses the operand by name.
//!
//! The payload fields of one outcome form a graph, each field a node and each operand an edge to
//! the field it reads. It has no cycle, a field reading itself included (`validate`).

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt::Write as _;

use ess_primitives::error::{ConstructRef, ValidationCode, ValidationError, ValidationErrors};
use ess_primitives::facts::FactPath;
use ess_primitives::predicate::Predicate;

use super::{Effect, Outcome, PayloadSource};
use crate::name::QualifiedName;
use crate::system::FormatVersion;
use crate::types::{Field, TypeRef, TypeRegistry};
use crate::Specification;

/// The root a published value is read under.
pub const ROOT: &str = "payload";

/// One value a selector reads from its outcome's payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishedRead {
    /// The event this outcome emits and fills.
    pub event: QualifiedName,
    /// The field of that event read.
    pub field: String,
    /// The field's declared type.
    pub type_ref: TypeRef,
}

/// Where a selector's predicate stands, which decides whether it may read a published value.
#[derive(Clone, Copy)]
pub(super) enum Site<'a> {
    /// A guard: `when_related: {entity, where, …}`, its `where:` and `forall:`.
    Guard,
    /// The filter of `instances:` or of an `affects:` entry.
    Effect,
    /// A value source in an error outcome's payload.
    ErrorPayload,
    /// A value source in `payload:`, `sets:` or `affects[].sets:` of this outcome.
    Value(&'a Outcome),
}

/// The published reads one selector is admitted to make, by event and field, at their declared
/// types: what its typed environment resolves under [`ROOT`].
#[derive(Debug, Default)]
pub(super) struct Admitted {
    events: BTreeMap<QualifiedName, BTreeMap<String, TypeRef>>,
}

/// One step of a path under [`ROOT`] through [`Admitted`].
pub(super) enum Step {
    /// More segments of an admitted event's name are needed.
    Namespace(Vec<String>),
    /// The segments so far name an admitted event.
    Event(QualifiedName),
}

impl Admitted {
    /// Whether no read is admitted.
    pub(super) fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Where `prefix` (the segments after [`ROOT`]) and then `segment` lead.
    pub(super) fn step(&self, prefix: &[String], segment: &str) -> Option<Step> {
        let mut segments = prefix.to_vec();
        segments.push(segment.to_owned());
        if let Some(event) = self
            .events
            .keys()
            .find(|event| event.segments() == segments.as_slice())
        {
            return Some(Step::Event(event.clone()));
        }
        self.events
            .keys()
            .any(|event| {
                event.segments().len() > segments.len()
                    && event.segments()[..segments.len()] == segments[..]
            })
            .then_some(Step::Namespace(segments))
    }

    /// The declared type of `field` of the admitted `event`.
    pub(super) fn field(&self, event: &QualifiedName, field: &str) -> Option<&TypeRef> {
        self.events.get(event).and_then(|fields| fields.get(field))
    }
}

/// What [`classify`] decided of a predicate's paths under [`ROOT`].
pub(super) struct Classified<'p> {
    /// The reads the typed check resolves.
    pub(super) admitted: Admitted,
    /// The refusals, at the site given.
    pub(super) errors: ValidationErrors,
    /// The paths refused, whose typed-check refusals would only repeat these.
    pub(super) refused: Vec<&'p FactPath>,
}

/// Whether `fields` leave [`ROOT`] free for the operand: a candidate's field named `payload` keeps
/// being read as itself.
pub(super) fn root_is_free(fields: &[Field]) -> bool {
    !fields.iter().any(|field| field.name == ROOT)
}

/// `path` split into the declared event its segments after [`ROOT`] name, the field after it and
/// any members after that; `None` where no prefix names a declared event.
fn split<'p>(
    spec: &Specification,
    path: &'p FactPath,
) -> Option<(QualifiedName, Option<&'p str>, &'p [String])> {
    let segments = path.segments();
    (2..=segments.len()).find_map(|end| {
        let event = segments[1..end].join(".").parse::<QualifiedName>().ok()?;
        spec.events().contains_key(&event).then(|| {
            (
                event,
                segments.get(end).map(String::as_str),
                segments.get(end + 1..).unwrap_or(&[]),
            )
        })
    })
}

/// The events this outcome emits and fills under `payload:`.
fn filled(outcome: &Outcome) -> impl Iterator<Item = &QualifiedName> {
    outcome
        .payload
        .keys()
        .filter(|event| outcome.emits.contains(event))
}

fn filled_hint(outcome: &Outcome) -> String {
    let named: Vec<String> = filled(outcome).map(ToString::to_string).collect();
    if named.is_empty() {
        "this outcome fills no event under `payload:`".to_owned()
    } else {
        format!("this outcome fills: {}", named.join(", "))
    }
}

fn at_least_27(format: FormatVersion) -> bool {
    format.major() >= FormatVersion::V27.major()
}

/// The refusal of the operand in a guard, at `at`.
fn in_a_guard(at: &ConstructRef, path: &FactPath) -> ValidationError {
    ValidationError::at(
        at.clone(),
        ValidationCode::UnsupportedConstruct,
        format!(
            "`{path}` reads a value this outcome publishes, and a guard decides the branch before \
             anything is published"
        ),
    )
    .with_hint("guard on the input or on the rows, and read the published value in a value source")
}

/// The refusal of `path` under [`ROOT`] in a guard's predicate whose environment does not resolve
/// it, from `ess/27`: `None` where `path` is another root, or the format has no operand.
pub(super) fn guard_refusal(
    types: &TypeRegistry,
    path: &FactPath,
    segment: Option<&str>,
    at: &ConstructRef,
) -> Option<ValidationError> {
    let admits = types.format().is_none_or(at_least_27);
    (admits && path.namespace() == ROOT && segment == Some(ROOT) && path.segments().len() > 2)
        .then(|| in_a_guard(at, path))
}

/// Every path of `predicate` under [`ROOT`], over a candidate reading `fields`, checked for `site`:
/// the reads admitted, and the refusals at `at`.
pub(super) fn classify<'p>(
    spec: &Specification,
    fields: &[Field],
    predicate: &'p Predicate,
    site: Site<'_>,
    at: &ConstructRef,
) -> Classified<'p> {
    let mut classified = Classified {
        admitted: Admitted::default(),
        errors: ValidationErrors::new(),
        refused: Vec::new(),
    };
    if !root_is_free(fields) {
        return classified;
    }
    let format = spec.system().format;
    for path in predicate.fact_paths() {
        if path.namespace() != ROOT || path.segments().len() < 2 {
            continue;
        }
        let refusal = if at_least_27(format) {
            match site {
                Site::Guard => (path.segments().len() > 2).then(|| in_a_guard(at, path)),
                Site::Effect => (path.segments().len() > 2).then(|| {
                    ValidationError::at(
                        at.clone(),
                        ValidationCode::UnsupportedConstruct,
                        format!(
                            "`{path}` reads a value this outcome publishes; an effect selecting \
                             its rows by a published value is not admitted in this cut"
                        ),
                    )
                    .with_hint("select the rows by `subject.` or `input.`")
                }),
                Site::ErrorPayload => (path.segments().len() > 2).then(|| {
                    ValidationError::at(
                        at.clone(),
                        ValidationCode::UnsupportedConstruct,
                        format!(
                            "`{path}` reads an event payload, and an error outcome fills no event \
                             payload to read"
                        ),
                    )
                    .with_hint("select by the input or by the subject")
                }),
                Site::Value(outcome) => match value_read(spec, outcome, at, path) {
                    Ok(read) => {
                        classified
                            .admitted
                            .events
                            .entry(read.event)
                            .or_default()
                            .insert(read.field, read.type_ref);
                        None
                    }
                    Err(refusal) => Some(refusal),
                },
            }
        } else {
            below_ess_27(spec, site, at, path)
        };
        if let Some(refusal) = refusal {
            classified.errors.push(refusal);
            classified.refused.push(path);
        }
    }
    classified
}

/// Below `ess/27`, the refusal of a value read's `path` naming an event the outcome fills and one
/// of its fields; every other path keeps the refusal the typed check gives it.
fn below_ess_27(
    spec: &Specification,
    site: Site<'_>,
    at: &ConstructRef,
    path: &FactPath,
) -> Option<ValidationError> {
    let Site::Value(outcome) = site else {
        return None;
    };
    let (event, Some(field), _) = split(spec, path)? else {
        return None;
    };
    let names_a_field = spec
        .events()
        .get(&event)
        .is_some_and(|declared| declared.field(field).is_some());
    (filled(outcome).any(|filled| *filled == event) && names_a_field).then(|| {
        ValidationError::at(
            at.clone(),
            ValidationCode::UnsupportedFormatVersion,
            format!(
                "`{path}` reads the value this outcome publishes in `{event}.{field}`, a selector \
                 operand from specification format ess/27"
            ),
        )
        .with_hint("declare `format: ess/27`")
    })
}

/// `path` as a read of this outcome's payload, or why it is none.
fn value_read(
    spec: &Specification,
    outcome: &Outcome,
    at: &ConstructRef,
    path: &FactPath,
) -> Result<PublishedRead, ValidationError> {
    let undeclared = |message: String| {
        ValidationError::at(at.clone(), ValidationCode::UndeclaredReference, message)
    };
    let Some((event, field, members)) = split(spec, path) else {
        return Err(undeclared(format!(
            "`{path}` names no event this outcome emits and fills under `payload:`"
        ))
        .with_hint(filled_hint(outcome)));
    };
    let Some(sources) = outcome
        .payload
        .get(&event)
        .filter(|_| outcome.emits.contains(&event))
    else {
        return Err(undeclared(format!(
            "`{event}` is not an event this outcome emits and fills under `payload:`, so `{path}` \
             reads no published value"
        ))
        .with_hint(filled_hint(outcome)));
    };
    let Some(field) = field else {
        return Err(undeclared(format!(
            "`{path}` names the event `{event}` and none of its fields"
        )));
    };
    let Some(declared) = spec
        .events()
        .get(&event)
        .and_then(|declared| declared.field(field))
    else {
        return Err(undeclared(format!(
            "`{path}`: `{field}` is not a field `{event}` carries"
        )));
    };
    let Some(source) = sources.get(field) else {
        return Err(undeclared(format!(
            "this outcome's `payload:` gives `{event}.{field}` no source, so `{path}` reads no \
             published value"
        ))
        .with_hint(format!(
            "give `{field}` a source under `payload: {{{event}: …}}`"
        )));
    };
    if let Some(why) = not_a_snapshot(source, members) {
        return Err(ValidationError::at(
            at.clone(),
            ValidationCode::UnsupportedConstruct,
            format!(
                "`{path}` reads `{event}.{field}`, whose source is {why}; a selector reads only a \
                 value the request and the store decide before the outcome"
            ),
        )
        .with_hint("give the read field a source the request or the store decides"));
    }
    if let Some(subject) = outcome
        .subject
        .as_ref()
        .filter(|subject| subject.effect == Effect::Creates && subject.instance == field)
    {
        let hint = match source {
            PayloadSource::InputField { field } => {
                format!("read the input it is published from, `input.{field}`")
            }
            _ => "read the input it is published from".to_owned(),
        };
        return Err(ValidationError::at(
            at.clone(),
            ValidationCode::UnsupportedConstruct,
            format!(
                "`{path}` reads `{event}.{field}`, the identity of the `{}` this outcome creates; \
                 a selector reads what the identity is published from",
                subject.entity
            ),
        )
        .with_hint(hint));
    }
    Ok(PublishedRead {
        event,
        field: field.to_owned(),
        type_ref: declared.type_ref.clone(),
    })
}

/// Why `source`, reached through `members` where it is a nested mapping, is not decided by the
/// request and the store before the outcome; `None` where it is.
fn not_a_snapshot(source: &PayloadSource, members: &[String]) -> Option<&'static str> {
    const MINT: &str = "a value the implementation mints, which no evaluator knows before the \
                        outcome and no scenario can arrange rows against";
    match source {
        PayloadSource::Generated { .. }
        | PayloadSource::InputOrGenerated {
            otherwise: None, ..
        } => Some(MINT),
        PayloadSource::InputOrGenerated {
            otherwise: Some(otherwise),
            ..
        } => not_a_snapshot(otherwise, &[]),
        PayloadSource::ResponseField { .. } => Some("the response, known only after the call"),
        PayloadSource::ChangedCount => {
            Some("the number of rows the outcome changed, known only after its effects")
        }
        PayloadSource::ChangedField { .. } => {
            Some("a field of one changed row, known only once per changed row after its effects")
        }
        PayloadSource::Increment { .. } | PayloadSource::Append { .. } => {
            Some("a value the outcome's own change computes")
        }
        PayloadSource::Expression { .. } => {
            Some("a value an expression computes (`{expr: …}`), which a selector does not read yet")
        }
        PayloadSource::Struct { fields } => match members.split_first() {
            Some((member, rest)) => fields
                .iter()
                .find(|leaf| leaf.target == *member)
                .and_then(|leaf| not_a_snapshot(&leaf.source, rest)),
            None => fields
                .iter()
                .find_map(|leaf| not_a_snapshot(&leaf.source, &[])),
        },
        PayloadSource::InputField { .. }
        | PayloadSource::Literal { .. }
        | PayloadSource::Scalar { .. }
        | PayloadSource::Cleared
        | PayloadSource::EmptyList
        | PayloadSource::SubjectField { .. }
        | PayloadSource::RelatedField { .. }
        | PayloadSource::RelatedSelection { .. }
        | PayloadSource::RelatedHead { .. }
        | PayloadSource::CallerAttribute { .. }
        | PayloadSource::MissingElement { .. }
        | PayloadSource::PayloadRead { .. } => None,
    }
}

/// A payload field of one outcome: its event and its name.
type Node = (QualifiedName, String);

/// The selections a source reads, nested mappings and fallbacks included.
fn selections(source: &PayloadSource) -> Vec<&super::row_set::RowSelection> {
    match source {
        PayloadSource::RelatedSelection { selection, .. }
        | PayloadSource::RelatedHead { selection, .. } => vec![selection],
        PayloadSource::Struct { fields } => fields
            .iter()
            .flat_map(|leaf| selections(&leaf.source))
            .collect(),
        PayloadSource::InputOrGenerated {
            otherwise: Some(otherwise),
            ..
        } => selections(otherwise),
        _ => Vec::new(),
    }
}

/// The payload fields of `outcome` that `selection` reads, as written, first appearance first;
/// below `ess/27`, over an entity declaring a field `payload`, or naming no filled field, none.
fn read_nodes(
    spec: &Specification,
    outcome: &Outcome,
    selection: &super::row_set::RowSelection,
) -> Vec<Node> {
    if !at_least_27(spec.system().format) {
        return Vec::new();
    }
    let Some(entity) = spec.entities().get(&selection.entity) else {
        return Vec::new();
    };
    if !root_is_free(&super::row_set::candidate_fields(entity)) {
        return Vec::new();
    }
    let mut nodes: Vec<Node> = Vec::new();
    for path in selection.filter.fact_paths() {
        if path.namespace() != ROOT {
            continue;
        }
        let Some((event, Some(field), _)) = split(spec, path) else {
            continue;
        };
        let is_filled = outcome
            .payload
            .get(&event)
            .is_some_and(|fields| fields.contains_key(field))
            && outcome.emits.contains(&event);
        let node = (event, field.to_owned());
        if is_filled && !nodes.contains(&node) {
            nodes.push(node);
        }
    }
    nodes
}

/// The payload fields of `outcome` in its map's order, each with the fields its selectors read.
fn graph(spec: &Specification, outcome: &Outcome) -> Vec<(Node, Vec<Node>)> {
    filled(outcome)
        .flat_map(|event| {
            outcome.payload[event].iter().map(move |(field, source)| {
                let reads = selections(source)
                    .into_iter()
                    .flat_map(|selection| read_nodes(spec, outcome, selection))
                    .fold(Vec::new(), |mut reads, node| {
                        if !reads.contains(&node) {
                            reads.push(node);
                        }
                        reads
                    });
                ((event.clone(), field.clone()), reads)
            })
        })
        .collect()
}

/// The published reads of `selection`, a value source of `outcome`, in dependency order: a read
/// whose field's selectors read another read of the list comes after it, and reads that depend on
/// none of the others keep their first appearance. Empty below `ess/27`, over an entity declaring
/// a field `payload`, and for a guard's selection, which reads none.
pub fn reads(
    spec: &Specification,
    outcome: &Outcome,
    selection: &super::row_set::RowSelection,
) -> Vec<PublishedRead> {
    let nodes = read_nodes(spec, outcome, selection);
    if nodes.is_empty() {
        return Vec::new();
    }
    let edges: BTreeMap<Node, Vec<Node>> = graph(spec, outcome).into_iter().collect();
    let mut ordered: Vec<(usize, PublishedRead)> = nodes
        .into_iter()
        .filter_map(|node| {
            let type_ref = spec
                .events()
                .get(&node.0)
                .and_then(|event| event.field(&node.1))?
                .type_ref
                .clone();
            let depth = depth(&edges, &node, &mut BTreeSet::new());
            Some((
                depth,
                PublishedRead {
                    event: node.0,
                    field: node.1,
                    type_ref,
                },
            ))
        })
        .collect();
    // Stable: equal depths keep their first appearance.
    ordered.sort_by_key(|(depth, _)| *depth);
    ordered.into_iter().map(|(_, read)| read).collect()
}

/// The length of the longest chain of reads from `node`; a node already on the chain counts
/// nothing, so a cycle (refused by [`validate`]) still terminates.
fn depth(edges: &BTreeMap<Node, Vec<Node>>, node: &Node, on_chain: &mut BTreeSet<Node>) -> usize {
    if !on_chain.insert(node.clone()) {
        return 0;
    }
    let deepest = edges
        .get(node)
        .into_iter()
        .flatten()
        .map(|next| depth(edges, next, on_chain) + 1)
        .max()
        .unwrap_or(0);
    on_chain.remove(node);
    deepest
}

/// The nodes reachable from `node` by one edge or more.
fn reachable(edges: &BTreeMap<Node, Vec<Node>>, node: &Node) -> BTreeSet<Node> {
    let mut seen = BTreeSet::new();
    let mut queue: VecDeque<&Node> = edges.get(node).into_iter().flatten().collect();
    while let Some(next) = queue.pop_front() {
        if seen.insert(next.clone()) {
            queue.extend(edges.get(next).into_iter().flatten());
        }
    }
    seen
}

/// The shortest chain from `node` back to itself through `within`, `node` first and last.
fn chain(edges: &BTreeMap<Node, Vec<Node>>, node: &Node, within: &BTreeSet<Node>) -> Vec<Node> {
    // `before[n]` is the node `n` was first reached from.
    let mut before: BTreeMap<Node, Node> = BTreeMap::new();
    let mut queue: VecDeque<Node> = VecDeque::from([node.clone()]);
    while let Some(current) = queue.pop_front() {
        for next in edges.get(&current).into_iter().flatten() {
            if next == node {
                let mut path = vec![current.clone()];
                let mut at = &current;
                while at != node {
                    at = &before[at];
                    path.push(at.clone());
                }
                path.reverse();
                path.push(node.clone());
                return path;
            }
            if within.contains(next) && !before.contains_key(next) {
                before.insert(next.clone(), current.clone());
                queue.push_back(next.clone());
            }
        }
    }
    vec![node.clone(), node.clone()]
}

/// Every cycle among one outcome's payload fields, a field reading itself included, refused once
/// at its first field in the payload's order as `self_reference`, naming the chain.
pub(crate) fn validate(spec: &Specification) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    if !at_least_27(spec.system().format) {
        return errors;
    }
    for command in spec.commands().values() {
        for outcome in &command.outcomes {
            let ordered = graph(spec, outcome);
            if ordered.iter().all(|(_, reads)| reads.is_empty()) {
                continue;
            }
            let edges: BTreeMap<Node, Vec<Node>> = ordered.iter().cloned().collect();
            let reach: BTreeMap<&Node, BTreeSet<Node>> = ordered
                .iter()
                .map(|(node, _)| (node, reachable(&edges, node)))
                .collect();
            let mut reported: BTreeSet<Node> = BTreeSet::new();
            for (node, _) in &ordered {
                if reported.contains(node) || !reach[node].contains(node) {
                    continue;
                }
                let component: BTreeSet<Node> = reach[node]
                    .iter()
                    .filter(|other| reach.get(other).is_some_and(|back| back.contains(node)))
                    .cloned()
                    .collect();
                let chain = chain(&edges, node, &component);
                let named = |(event, field): &Node| format!("`{event}.{field}`");
                let mut message = format!("{} reads {}", named(&chain[0]), named(&chain[1]));
                for next in &chain[2..] {
                    write!(message, ", which reads {}", named(next)).expect("writing to a String");
                }
                errors.push(
                    ValidationError::at(
                        command
                            .site()
                            .key("outcomes")
                            .named(outcome.name.as_str())
                            .key("payload")
                            .named(node.0.to_string())
                            .named(&node.1),
                        ValidationCode::SelfReference,
                        format!(
                            "{message}; a published value read by a selector is resolved before \
                             the field that reads it, so no field may read itself through others"
                        ),
                    )
                    .with_hint("scope one of the selectors by the input or the subject instead"),
                );
                reported.extend(component);
                reported.insert(node.clone());
            }
        }
    }
    errors
}
