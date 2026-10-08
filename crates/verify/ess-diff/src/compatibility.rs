//! Whether a change breaks someone, and on which side of the system it does (beyond10x/ess#290).
//!
//! A [`SemanticRelation`] says which way a set moved. It does not say who is hurt by the move,
//! because that depends on who *reads* the construct: a variant removed from a type a caller sends
//! refuses a request that used to succeed, and the same variant removed from a type only the system
//! writes is one value a reader will never see again. Design §29 asks for compatibility as a
//! directional answer for that reason, and this module gives it in three dimensions:
//!
//! | dimension | the question |
//! |---|---|
//! | [`Dimension::Callers`] | can a caller written against the `before` revision invoke the `after` one? |
//! | [`Dimension::Readers`] | can a reader written against `before` read what `after` produces? |
//! | [`Dimension::History`] | can `after` read what `before` already wrote — stored entity state and emitted events? |
//!
//! Each answers [`Compatibility::Compatible`], [`Compatibility::Breaking`] or
//! [`Compatibility::Unknown`], and the change's verdict is the worst of the three.
//!
//! # Where a type is used
//!
//! A type change is answered from its [`TypeUse`]s: every role the type reaches in either revision,
//! through any number of structs, newtypes, unions, lists, maps and optionals. Nesting is read from
//! the compiler's [`SemanticDependencyGraph`] — the walk impact analysis uses — and the one fact
//! that graph does not separate, whether a command field is an input or a response, is read from
//! the command itself. A component setting, an external channel's delivery context and a periodic
//! host's context and read fields are inputs too: a party outside the system supplies them.
//!
//! The walk fails closed. Every other mention of the type anywhere in the serialized model is a
//! [`TypeUse::Unmodelled`] use, and makes every dimension `unknown` for any change to the type
//! other than documentation or adding or removing it; only a type referenced nowhere answers `compatible` by having no use. The uses are written into
//! the document beside the answer, so a reader can re-derive the answer from the document alone
//! and refuses one that does not match.
//!
//! # `Unknown` is an answer
//!
//! Design §30: whether an added field or a renamed wire spelling breaks a consumer depends on
//! assumptions about the consumer that the model does not hold. Without a profile stating them, the
//! answer is `Unknown`, never a guess. Only these are decided:
//!
//! | change | decided as |
//! |---|---|
//! | display name, summary or example moved | compatible everywhere: documentation |
//! | a type widened (`expanded`) | breaking for readers of an output use; compatible for callers and history |
//! | a type narrowed (`narrowed`) | breaking for callers of an input use and for stored history; compatible for readers |
//! | a union variant gains or loses a required payload (ess/22) | breaking for every use: neither revision reads what the other writes |
//! | a unit variant gains, or loses, an `Optional<…>` payload (ess/22) | decided as `expanded`, or `narrowed`: the tag alone is a value of both |
//! | a type added or removed | compatible: every use of it is its own change |
//! | an actor gains a grant, or any construct is added | compatible |
//! | a domain is removed (`ess-diff/15`) | breaking for callers and readers; compatible for history, which each removed entity and event answers for |
//! | an actor loses a grant, an actor or a command is removed | breaking for callers |
//! | a refusal gains or loses its compensating change (`compensates: true`, ess/22) | breaking for callers and readers; compatible for history |
//! | a view is removed | breaking for readers |
//! | an event or an entity is removed | breaking for history |
//!
//! Everything else is `Unknown` in the dimensions its family touches and compatible in the rest.
//!
//! # The gate
//!
//! [`Gate`] turns a classified delta into a pass or a failure: it fails on any change at or above
//! its [`FailOn`] threshold in the dimensions it considers, unless an [`Acknowledgements`] document
//! bound to the same two revisions names that change's id.

use std::collections::BTreeSet;
use std::fmt;

use ess_compiler::ir::EssIr;
use ess_conformance::scenario::{DeclaredTypeRef, EssSemanticRef};
use ess_primitives::error::{ValidationCode, ValidationError, ValidationErrors};
use ess_primitives::evidence::SpecDigest;

use crate::change::{
    ActorChange, BindingChange, ChangeId, CommandChange, ComponentChange, DomainChange,
    EntityChange, ErrorChange, EventChange, SemanticChange, SemanticRelation, SystemChange,
    TypeChange, ViewChange,
};
use crate::delta::EssDelta;
use crate::graph::{DependencyRelation, SemanticDependencyGraph};

/// The first delta format whose changes carry a [`ChangeCompatibility`].
pub const CLASSIFIED_DELTA_FORMAT: u32 = 14;

/// One answer to one compatibility question.
///
/// Ordered by severity, so the verdict of several answers is their maximum.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "kebab-case")]
pub enum Compatibility {
    /// Nobody on this side is broken by the change.
    Compatible,
    /// The model does not hold the assumption that would decide it.
    Unknown,
    /// Somebody on this side is broken by the change.
    Breaking,
}

impl Compatibility {
    /// How it is written.
    pub const fn written(self) -> &'static str {
        match self {
            Self::Compatible => "compatible",
            Self::Unknown => "unknown",
            Self::Breaking => "breaking",
        }
    }
}

impl fmt::Display for Compatibility {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.written())
    }
}

/// Which side of the system a compatibility answer is about.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "kebab-case")]
pub enum Dimension {
    /// Callers written against `before` invoking `after`: command inputs, view parameters, grants.
    Callers,
    /// Readers written against `before` reading what `after` produces: responses, events, errors,
    /// views.
    Readers,
    /// `after` reading what `before` already wrote: entity state and emitted events.
    History,
}

impl Dimension {
    /// All three, in order.
    pub const ALL: [Self; 3] = [Self::Callers, Self::Readers, Self::History];

    /// How it is written.
    pub const fn written(self) -> &'static str {
        match self {
            Self::Callers => "callers",
            Self::Readers => "readers",
            Self::History => "history",
        }
    }
}

impl fmt::Display for Dimension {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.written())
    }
}

/// A role a declared type plays, directly or nested inside another type.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "kebab-case")]
pub enum TypeUse {
    /// A caller supplies it: a command input or a view parameter.
    Input,
    /// The system produces it: a command response, an event, an error or a view row.
    Output,
    /// The system keeps it: an entity's identity, fields or state, or an event.
    Stored,
    /// The model references it somewhere this classifier does not read a role from — an actor
    /// attribute, a conversion, an aggregate, a selection plan, a binding's event accessor, or
    /// anything a later format adds. Fails closed: every dimension of a non-documentation change
    /// to it is `unknown`.
    Unmodelled,
}

/// How one change bears on callers, readers and history, and the verdict over all three.
///
/// Derived, never declared: [`ChangeCompatibility::derive`] is the only constructor, and a delta
/// read back is refused when its written classification differs from the one its content derives.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ChangeCompatibility {
    verdict: Compatibility,
    callers: Compatibility,
    readers: Compatibility,
    history: Compatibility,
    /// Where the changed type is used. Present on a type change only.
    #[serde(skip_serializing_if = "Option::is_none")]
    uses: Option<BTreeSet<TypeUse>>,
}

/// One classification as a document writes it, before anything has checked it.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawChangeCompatibility {
    /// The verdict the document claims.
    pub verdict: Compatibility,
    /// The callers answer it claims.
    pub callers: Compatibility,
    /// The readers answer it claims.
    pub readers: Compatibility,
    /// The history answer it claims.
    pub history: Compatibility,
    /// The uses it claims for a type change.
    #[serde(default)]
    pub uses: Option<BTreeSet<TypeUse>>,
}

impl ChangeCompatibility {
    /// The classification `change` derives, given where its type is used.
    ///
    /// `uses` is read for a type change and recorded with it; every other family ignores it.
    pub fn derive(change: &SemanticChange, uses: &BTreeSet<TypeUse>) -> Self {
        let [callers, readers, history] = dimensions(change, uses);
        Self {
            verdict: callers.max(readers).max(history),
            callers,
            readers,
            history,
            uses: matches!(change, SemanticChange::Type { .. }).then(|| uses.clone()),
        }
    }

    /// The worst of the three answers.
    pub fn verdict(&self) -> Compatibility {
        self.verdict
    }

    /// Whether a caller written against `before` can invoke `after`.
    pub fn callers(&self) -> Compatibility {
        self.callers
    }

    /// Whether a reader written against `before` can read what `after` produces.
    pub fn readers(&self) -> Compatibility {
        self.readers
    }

    /// Whether `after` can read what `before` wrote.
    pub fn history(&self) -> Compatibility {
        self.history
    }

    /// One dimension's answer.
    pub fn dimension(&self, dimension: Dimension) -> Compatibility {
        match dimension {
            Dimension::Callers => self.callers,
            Dimension::Readers => self.readers,
            Dimension::History => self.history,
        }
    }

    /// Where the changed type is used, for a type change; `None` for every other family.
    pub fn uses(&self) -> Option<&BTreeSet<TypeUse>> {
        self.uses.as_ref()
    }

    /// The worst answer among `dimensions`.
    pub fn worst_of(&self, dimensions: &BTreeSet<Dimension>) -> Compatibility {
        dimensions
            .iter()
            .map(|dimension| self.dimension(*dimension))
            .max()
            .unwrap_or(Compatibility::Compatible)
    }

    /// The answers in one clause: `breaking for callers, history` or `compatible`.
    pub fn describe(&self) -> String {
        let at = |level: Compatibility| {
            Dimension::ALL
                .into_iter()
                .filter(|dimension| self.dimension(*dimension) == level)
                .map(Dimension::written)
                .collect::<Vec<_>>()
                .join(", ")
        };
        match self.verdict {
            Compatibility::Compatible => "compatible".to_owned(),
            Compatibility::Breaking if self.unknowns() => format!(
                "breaking for {}; unknown for {}",
                at(Compatibility::Breaking),
                at(Compatibility::Unknown)
            ),
            level => format!("{level} for {}", at(level)),
        }
    }

    fn unknowns(&self) -> bool {
        Dimension::ALL
            .into_iter()
            .any(|dimension| self.dimension(dimension) == Compatibility::Unknown)
    }
}

impl RawChangeCompatibility {
    /// Whether this is what `change` derives from the uses the document claims.
    ///
    /// # Errors
    /// Every way it is not, accumulated, located under `location`.
    pub fn check(&self, change: &SemanticChange, location: &str) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();
        let is_type = matches!(change, SemanticChange::Type { .. });
        if is_type != self.uses.is_some() {
            errors.push(ValidationError::new(
                ValidationCode::ConflictingDeclaration,
                format!("{location}.uses"),
                if is_type {
                    "a type change records where the type is used".to_owned()
                } else {
                    "only a type change records where a type is used".to_owned()
                },
            ));
            return Err(errors);
        }
        let derived = ChangeCompatibility::derive(change, &self.uses.clone().unwrap_or_default());
        let written = [
            ("verdict", self.verdict, derived.verdict),
            ("callers", self.callers, derived.callers),
            ("readers", self.readers, derived.readers),
            ("history", self.history, derived.history),
        ];
        for (field, claimed, derives) in written {
            if claimed != derives {
                errors.push(
                    ValidationError::new(
                        ValidationCode::ConflictingDeclaration,
                        format!("{location}.{field}"),
                        format!(
                            "the document calls `{}` {claimed} for {field} and its own content \
                             derives {derives}",
                            change.id()
                        ),
                    )
                    .with_hint(
                        "a classification is derived, not declared; regenerate the delta rather \
                         than editing it",
                    ),
                );
            }
        }
        errors.into_result(())
    }

    /// The validated classification, once [`Self::check`] has passed.
    pub(crate) fn admitted(self) -> ChangeCompatibility {
        ChangeCompatibility {
            verdict: self.verdict,
            callers: self.callers,
            readers: self.readers,
            history: self.history,
            uses: self.uses,
        }
    }
}

/// Callers, readers, history — in that order.
type Dimensions = [Compatibility; 3];

use Compatibility::{Breaking as B, Compatible as C, Unknown as U};

/// The three answers for one change.
fn dimensions(change: &SemanticChange, uses: &BTreeSet<TypeUse>) -> Dimensions {
    if documentation_only(change) {
        return [C, C, C];
    }
    match change {
        SemanticChange::System { .. } | SemanticChange::Binding { .. } => [U, U, U],
        // A domain arriving adds a namespace and breaks nobody. One going away takes its wire name
        // from every surface a caller or a reader addresses through it; what it stored is named by
        // each removed entity and event, each breaking history on its own (beyond10x/ess#469).
        SemanticChange::Domain { changed, .. } => match changed {
            DomainChange::Added => [C, C, C],
            DomainChange::Removed => [B, B, C],
        },
        SemanticChange::Type { changed, .. } => type_dimensions(changed, change.relation(), uses),
        SemanticChange::Actor { changed, .. } => match changed {
            ActorChange::Added | ActorChange::GrantAdded { .. } => [C, C, C],
            ActorChange::Removed | ActorChange::GrantRemoved { .. } => [B, C, C],
            _ => [U, C, C],
        },
        SemanticChange::Command { changed, .. } => match changed {
            CommandChange::Added => [C, C, C],
            CommandChange::Removed => [B, C, C],
            // A refusal that now changes its row, or no longer does (ess/22, beyond10x/ess#197):
            // a caller retrying after it and a reader of the row meet a different state; nothing
            // stored changes shape.
            CommandChange::OutcomeCompensatesChanged { .. } => [B, B, C],
            _ => [U, U, C],
        },
        SemanticChange::Event { changed, .. } => match changed {
            EventChange::Added => [C, C, C],
            EventChange::Removed => [C, C, B],
            _ => [C, U, U],
        },
        SemanticChange::Error { changed, .. } => match changed {
            ErrorChange::Added | ErrorChange::Removed => [C, C, C],
            _ => [C, U, C],
        },
        SemanticChange::View { changed, .. } => match changed {
            ViewChange::Added => [C, C, C],
            ViewChange::Removed => [C, B, C],
            _ => [U, U, C],
        },
        SemanticChange::Entity { changed, .. } => match changed {
            EntityChange::Added => [C, C, C],
            EntityChange::Removed => [C, C, B],
            _ => [U, U, U],
        },
        SemanticChange::Component { changed, .. } => match changed {
            ComponentChange::Added => [C, C, C],
            _ => [U, U, C],
        },
    }
}

/// A type change, read through the roles the type plays.
///
/// A widening hands readers a value they have never seen and costs callers and history nothing; a
/// narrowing refuses a value callers may still send and history may still hold, and costs readers
/// nothing. A change with no direction is unknown wherever the type is used.
fn type_dimensions(
    changed: &TypeChange,
    relation: SemanticRelation,
    uses: &BTreeSet<TypeUse>,
) -> Dimensions {
    if matches!(changed, TypeChange::Added | TypeChange::Removed) {
        return [C, C, C];
    }
    let [input, output, stored] = unit_variant_dimensions(changed).unwrap_or(match relation {
        SemanticRelation::Expanded => [C, B, C],
        SemanticRelation::Narrowed => [B, C, B],
        SemanticRelation::Changed => [U, U, U],
    });
    let at = |used: TypeUse, answer: Compatibility| if uses.contains(&used) { answer } else { C };
    // A use the classifier cannot place is a question the model does not answer: never better
    // than unknown, in any dimension.
    let floor = if uses.contains(&TypeUse::Unmodelled) {
        U
    } else {
        C
    };
    [
        at(TypeUse::Input, input).max(floor),
        at(TypeUse::Output, output).max(floor),
        at(TypeUse::Stored, stored).max(floor),
    ]
}

/// Callers, readers and history for a union variant that gained or lost its payload (ess/22,
/// beyond10x/ess#418), or `None` for any other change.
///
/// A required payload changes every value of the variant in both directions — `{"kind": "Open"}`
/// against `{"kind": "Open", "value": …}` — so neither revision reads what the other writes:
/// breaking wherever the union is used. An `Optional<…>` payload is read without its content
/// member, so the tag alone stays a value of it: gaining one widens the variant, as `expanded`
/// does, and losing one narrows it.
fn unit_variant_dimensions(changed: &TypeChange) -> Option<Dimensions> {
    let TypeChange::VariantTypeChanged { before, after, .. } = changed else {
        return None;
    };
    let unit = |side: &str| side == crate::change::UNIT_PAYLOAD;
    let optional = |side: &str| side.starts_with("Optional<");
    match (before.as_str(), after.as_str()) {
        (was, is) if unit(was) && optional(is) => Some([C, B, C]),
        (was, is) if optional(was) && unit(is) => Some([B, C, B]),
        (was, is) if unit(was) || unit(is) => Some([B, B, B]),
        _ => None,
    }
}

/// A change to a display name, a summary or an example, which no caller, reader or stored value
/// depends on.
fn documentation_only(change: &SemanticChange) -> bool {
    match change {
        SemanticChange::Domain { .. } => false,
        SemanticChange::System { changed, .. } => {
            matches!(changed, SystemChange::SummaryChanged { .. })
        }
        SemanticChange::Type { changed, .. } => matches!(
            changed,
            TypeChange::DisplayNameChanged { .. }
                | TypeChange::SummaryChanged { .. }
                | TypeChange::FieldDisplayNameChanged { .. }
                | TypeChange::FieldSummaryChanged { .. }
                | TypeChange::VariantDisplayNameChanged { .. }
                | TypeChange::VariantSummaryChanged { .. }
        ),
        SemanticChange::Entity { changed, .. } => matches!(
            changed,
            EntityChange::IdentityDisplayNameChanged { .. }
                | EntityChange::IdentitySummaryChanged { .. }
                | EntityChange::FieldDisplayNameChanged { .. }
                | EntityChange::FieldSummaryChanged { .. }
                | EntityChange::DisplayNameChanged { .. }
                | EntityChange::SummaryChanged { .. }
        ),
        SemanticChange::Command { changed, .. } => matches!(
            changed,
            CommandChange::InputExampleChanged { .. }
                | CommandChange::InputDisplayNameChanged { .. }
                | CommandChange::InputSummaryChanged { .. }
                | CommandChange::OutcomeSummaryChanged { .. }
                | CommandChange::DisplayNameChanged { .. }
                | CommandChange::SummaryChanged { .. }
        ),
        SemanticChange::Event { changed, .. } => matches!(
            changed,
            EventChange::FieldDisplayNameChanged { .. }
                | EventChange::FieldSummaryChanged { .. }
                | EventChange::DisplayNameChanged { .. }
                | EventChange::SummaryChanged { .. }
        ),
        SemanticChange::Error { changed, .. } => matches!(
            changed,
            ErrorChange::DisplayNameChanged { .. }
                | ErrorChange::NamingSummaryChanged { .. }
                | ErrorChange::FieldDisplayNameChanged { .. }
                | ErrorChange::FieldSummaryChanged { .. }
                | ErrorChange::SummaryChanged { .. }
        ),
        SemanticChange::View { changed, .. } => matches!(
            changed,
            ViewChange::FieldDisplayNameChanged { .. }
                | ViewChange::FieldSummaryChanged { .. }
                | ViewChange::DisplayNameChanged { .. }
                | ViewChange::SummaryChanged { .. }
        ),
        SemanticChange::Actor { changed, .. } => matches!(
            changed,
            ActorChange::DisplayNameChanged { .. } | ActorChange::SummaryChanged { .. }
        ),
        SemanticChange::Component { changed, .. } => matches!(
            changed,
            ComponentChange::DisplayNameChanged { .. } | ComponentChange::SummaryChanged { .. }
        ),
        SemanticChange::Binding { changed, .. } => matches!(
            changed,
            BindingChange::DisplayNameChanged { .. }
                | BindingChange::SummaryChanged { .. }
                | BindingChange::ContextFieldDisplayChanged { .. }
                | BindingChange::ContextFieldSummaryChanged { .. }
        ),
    }
}

/// Where each declared type is used, across both revisions.
pub(crate) struct UseIndex<'a> {
    revisions: [(&'a EssIr, SemanticDependencyGraph, Residual); 2],
}

impl<'a> UseIndex<'a> {
    /// The index over two revisions.
    pub(crate) fn new(before: &'a EssIr, after: &'a EssIr) -> Self {
        Self {
            revisions: [
                (
                    before,
                    SemanticDependencyGraph::of(before),
                    Residual::of(before),
                ),
                (
                    after,
                    SemanticDependencyGraph::of(after),
                    Residual::of(after),
                ),
            ],
        }
    }

    /// Every role `subject` plays in either revision, directly or nested in another type.
    pub(crate) fn uses(&self, subject: &DeclaredTypeRef) -> BTreeSet<TypeUse> {
        let mut uses = BTreeSet::new();
        for (ir, graph, residual) in &self.revisions {
            uses.extend(uses_in(ir, graph, residual, subject));
        }
        uses
    }
}

/// The roles `subject` plays in one revision.
///
/// The closure backwards from a type reaches every type that holds it — a type is only ever the
/// dependent of another type's `wraps`, `has a field of type` or `has a variant carrying` edge,
/// or of its own domain, which nothing reaches — so the declared types it collects are exactly the
/// containers. Each container's direct dependents then say the role.
fn uses_in(
    ir: &EssIr,
    graph: &SemanticDependencyGraph,
    residual: &Residual,
    subject: &DeclaredTypeRef,
) -> BTreeSet<TypeUse> {
    let reach = graph.closure(&subject.clone().into());
    let containers: BTreeSet<&EssSemanticRef> = reach
        .constructs()
        .filter(|construct| matches!(construct, EssSemanticRef::Type { .. }))
        .collect();
    let names: BTreeSet<_> = containers
        .iter()
        .filter_map(|construct| match construct {
            EssSemanticRef::Type { name } => Some(name.name().clone()),
            _ => None,
        })
        .collect();

    let mut uses = BTreeSet::new();
    for container in &containers {
        for edge in graph.dependents_of(container) {
            match (&edge.dependent, edge.relation) {
                (EssSemanticRef::Command { name }, DependencyRelation::FieldType) => {
                    let Some(command) = ir.commands().get(name.name()) else {
                        continue;
                    };
                    let reaches = |fields: &[ess_compiler::ir::ResolvedField]| {
                        fields.iter().any(|field| {
                            field
                                .type_ref
                                .named_leaves()
                                .into_iter()
                                .any(|leaf| names.contains(leaf.name()))
                        })
                    };
                    if reaches(&command.input) {
                        uses.insert(TypeUse::Input);
                    }
                    if reaches(&command.response) {
                        uses.insert(TypeUse::Output);
                    }
                }
                (EssSemanticRef::Event { .. }, DependencyRelation::FieldType) => {
                    uses.insert(TypeUse::Output);
                    uses.insert(TypeUse::Stored);
                }
                (EssSemanticRef::Error { .. }, DependencyRelation::FieldType)
                | (
                    EssSemanticRef::View { .. },
                    DependencyRelation::FieldType | DependencyRelation::RowShape,
                ) => {
                    uses.insert(TypeUse::Output);
                }
                (EssSemanticRef::View { .. }, DependencyRelation::ParameterType) => {
                    uses.insert(TypeUse::Input);
                }
                (
                    EssSemanticRef::Entity { .. },
                    DependencyRelation::FieldType | DependencyRelation::StateType,
                ) => {
                    uses.insert(TypeUse::Stored);
                }
                // Not dropped: every edge is minted from IR content, and `Residual` reads all of
                // that content except the places this match reads. A binding's `maps`, for one, is
                // a host or channel input there, or unmodelled.
                _ => {}
            }
        }
    }
    uses.extend(residual.uses(&names));
    uses
}

/// The IR content [`uses_in`]'s graph match does not read, for the fail-closed half of the walk.
///
/// The graph says where a type sits in commands, events, errors, views and entities. Everything
/// else in the serialized model is split two ways: values a party outside the system supplies — a
/// component's `settings:`, an external channel's delivery context, a periodic host's context and
/// read fields — which are an [`TypeUse::Input`], and the rest, where any mention of the type is a
/// [`TypeUse::Unmodelled`] use. The same serialize-and-strip shape the residual comparison in
/// `diff.rs` uses, for the same reason: a field the model gains later is visible by default.
pub(crate) struct Residual {
    supplied: Vec<serde_json::Value>,
    rest: serde_json::Value,
}

/// Declaration families, and the members of each whose type uses the graph match reads.
const MODELLED: [(&str, &[&str]); 9] = [
    // A type body is containment, which the closure follows.
    ("types", &["body"]),
    ("entities", &["identity", "fields", "state_type"]),
    ("commands", &["input", "response"]),
    ("events", &["fields"]),
    ("errors", &["fields"]),
    ("views", &["fields", "params", "shape"]),
    ("actors", &[]),
    ("bindings", &[]),
    ("components", &[]),
];

impl Residual {
    /// Splits one compiled model.
    pub(crate) fn of(ir: &EssIr) -> Self {
        let mut rest = serde_json::to_value(ir).expect("the canonical IR serializes");
        let mut supplied = Vec::new();
        for (family, modelled) in MODELLED {
            let Some(declarations) = rest
                .get_mut(family)
                .and_then(serde_json::Value::as_object_mut)
            else {
                continue;
            };
            for declaration in declarations.values_mut() {
                let Some(object) = declaration.as_object_mut() else {
                    continue;
                };
                object.remove("name");
                for key in modelled {
                    object.remove(*key);
                }
                match family {
                    // An outcome reads and writes fields; each `type_ref` in it restates the type
                    // of an input, event, error, response or entity field read above.
                    "commands" => {
                        if let Some(outcomes) = object.get_mut("outcomes") {
                            remove_everywhere(outcomes, "type_ref");
                        }
                    }
                    "components" => supplied.extend(object.remove("settings")),
                    "bindings" => {
                        supplied.extend(object.remove("context"));
                        if let Some(periodic) = object
                            .get_mut("periodic")
                            .and_then(serde_json::Value::as_object_mut)
                        {
                            supplied.extend(periodic.remove("context"));
                            supplied.extend(periodic.remove("read"));
                            if let Some(host) = periodic
                                .get_mut("host")
                                .and_then(serde_json::Value::as_object_mut)
                            {
                                supplied.extend(host.remove("context_fields"));
                                supplied.extend(host.remove("read_fields"));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        // Domain membership declares a type; it does not use one.
        if let Some(domains) = rest
            .get_mut("domains")
            .and_then(serde_json::Value::as_object_mut)
        {
            for domain in domains.values_mut() {
                if let Some(domain) = domain.as_object_mut() {
                    domain.remove("types");
                }
            }
        }
        // A payload or mapping target type restates the event, error or command-input field it
        // fills, each of which is read above.
        remove_everywhere(&mut rest, "target_type");
        Self { supplied, rest }
    }

    /// The uses outside the graph match of any of `names` — a type and every type holding it.
    fn uses(&self, names: &BTreeSet<ess_domain::name::QualifiedName>) -> BTreeSet<TypeUse> {
        let names: Vec<String> = names.iter().map(ToString::to_string).collect();
        let mut uses = BTreeSet::new();
        if self.supplied.iter().any(|value| mentions(value, &names)) {
            uses.insert(TypeUse::Input);
        }
        // Declaration keys are the declarations' own names, not uses: read below the key.
        let unmodelled = match &self.rest {
            serde_json::Value::Object(model) => model.iter().any(|(family, value)| {
                let declarations = MODELLED.iter().any(|(name, _)| name == family);
                match value {
                    serde_json::Value::Object(map) if declarations => {
                        map.values().any(|value| mentions(value, &names))
                    }
                    other => mentions(other, &names),
                }
            }),
            other => mentions(other, &names),
        };
        if unmodelled {
            uses.insert(TypeUse::Unmodelled);
        }
        uses
    }
}

fn remove_everywhere(value: &mut serde_json::Value, key: &str) {
    match value {
        serde_json::Value::Object(map) => {
            map.remove(key);
            map.values_mut()
                .for_each(|value| remove_everywhere(value, key));
        }
        serde_json::Value::Array(items) => items
            .iter_mut()
            .for_each(|value| remove_everywhere(value, key)),
        _ => {}
    }
}

/// Whether any key or string in `value` names one of `names` — whole, or inside a written type
/// such as `List<demo.calls.Channel>`, never as the prefix of a longer name.
fn mentions(value: &serde_json::Value, names: &[String]) -> bool {
    match value {
        serde_json::Value::String(text) => names.iter().any(|name| names_in(text, name)),
        serde_json::Value::Array(items) => items.iter().any(|item| mentions(item, names)),
        serde_json::Value::Object(map) => map.iter().any(|(key, value)| {
            names.iter().any(|name| names_in(key, name)) || mentions(value, names)
        }),
        _ => false,
    }
}

fn names_in(text: &str, name: &str) -> bool {
    let part = |c: char| c.is_alphanumeric() || c == '.' || c == '_' || c == '-';
    text.match_indices(name).any(|(at, _)| {
        let before = text[..at].chars().next_back();
        let after = text[at + name.len()..].chars().next();
        !before.is_some_and(part) && !after.is_some_and(part)
    })
}

/// What a [`Gate`] fails on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FailOn {
    /// A change that is breaking in a considered dimension.
    Breaking,
    /// A change that is breaking or unknown in a considered dimension.
    BreakingOrUnknown,
}

impl FailOn {
    /// The least severe answer that fails.
    const fn threshold(self) -> Compatibility {
        match self {
            Self::Breaking => Compatibility::Breaking,
            Self::BreakingOrUnknown => Compatibility::Unknown,
        }
    }
}

/// A compatibility gate over a classified delta.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gate {
    fail_on: FailOn,
    dimensions: BTreeSet<Dimension>,
}

impl Gate {
    /// A gate failing on `fail_on` in every dimension.
    pub fn new(fail_on: FailOn) -> Self {
        Self {
            fail_on,
            dimensions: Dimension::ALL.into_iter().collect(),
        }
    }

    /// The same gate, considering only `dimensions`.
    #[must_use]
    pub fn dimensions(mut self, dimensions: impl IntoIterator<Item = Dimension>) -> Self {
        self.dimensions = dimensions.into_iter().collect();
        self
    }

    /// Judges `delta`, excusing every change `acknowledged` names.
    ///
    /// # Errors
    /// [`GateRefusal::Unclassified`] when `delta` carries no classification.
    pub fn judge(
        &self,
        delta: &EssDelta,
        acknowledged: Option<&Acknowledgements>,
    ) -> Result<GateOutcome, GateRefusal> {
        let classifications = delta.compatibility().ok_or(GateRefusal::Unclassified)?;
        let mut outcome = GateOutcome {
            failing: Vec::new(),
            acknowledged: Vec::new(),
        };
        for (change, compatibility) in delta.changes().iter().zip(classifications) {
            if compatibility.worst_of(&self.dimensions) < self.fail_on.threshold() {
                continue;
            }
            let id = change.id();
            if acknowledged.is_some_and(|acknowledged| acknowledged.ids.contains(&id)) {
                outcome.acknowledged.push(id);
            } else {
                outcome.failing.push(id);
            }
        }
        Ok(outcome)
    }
}

/// What a [`Gate`] found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GateOutcome {
    failing: Vec<ChangeId>,
    acknowledged: Vec<ChangeId>,
}

impl GateOutcome {
    /// `true` when no change fails.
    pub fn passed(&self) -> bool {
        self.failing.is_empty()
    }

    /// The changes that fail, in canonical order.
    pub fn failing(&self) -> &[ChangeId] {
        &self.failing
    }

    /// The changes that would fail and are acknowledged, in canonical order.
    pub fn acknowledged(&self) -> &[ChangeId] {
        &self.acknowledged
    }
}

/// Why a [`Gate`] cannot judge a delta.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GateRefusal {
    /// The delta carries no classification; produce it with [`crate::classified`].
    Unclassified,
}

impl fmt::Display for GateRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unclassified => f.write_str(
                "the delta carries no compatibility classification, so there is nothing to gate on",
            ),
        }
    }
}

impl std::error::Error for GateRefusal {}

/// An acknowledgements document as it is written.
///
/// ```json
/// {
///   "format": "ess-diff-acknowledgements/1",
///   "before": "<spec digest of --from>",
///   "after": "<spec digest of --to>",
///   "acknowledged": ["type/demo.calls.Channel/variant-removed/Chat"]
/// }
/// ```
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawAcknowledgements {
    /// `ess-diff-acknowledgements/1`.
    pub format: String,
    /// The `before` digest of the delta it acknowledges.
    pub before: SpecDigest,
    /// The `after` digest of the delta it acknowledges.
    pub after: SpecDigest,
    /// Change ids that are allowed to fail the gate.
    pub acknowledged: Vec<String>,
}

/// Change ids allowed past a [`Gate`], bound to the one pair of revisions they were written for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Acknowledgements {
    ids: BTreeSet<ChangeId>,
}

impl Acknowledgements {
    /// The one format this build reads.
    pub const FORMAT: &'static str = "ess-diff-acknowledgements/1";

    /// Admits `raw` for `delta`.
    ///
    /// # Errors
    /// Every way `raw` does not acknowledge changes of `delta`, accumulated: another format,
    /// another pair of revisions, an id the delta does not hold, an id twice.
    pub fn admit(raw: &RawAcknowledgements, delta: &EssDelta) -> Result<Self, ValidationErrors> {
        let mut errors = ValidationErrors::new();
        if raw.format != Self::FORMAT {
            errors.push(ValidationError::new(
                ValidationCode::UnsupportedFormatVersion,
                "acknowledgements.format",
                format!(
                    "this build reads `{}`, and the document is written in `{}`",
                    Self::FORMAT,
                    raw.format
                ),
            ));
        }
        for (side, written, compared) in [
            ("before", &raw.before, &delta.before.spec_digest),
            ("after", &raw.after, &delta.after.spec_digest),
        ] {
            if written != compared {
                errors.push(
                    ValidationError::new(
                        ValidationCode::ConflictingDeclaration,
                        format!("acknowledgements.{side}"),
                        format!(
                            "the acknowledgements are for `{side}` {written}, and this comparison's \
                             `{side}` is {compared}"
                        ),
                    )
                    .with_hint(
                        "an acknowledgement holds for the exact pair it was reviewed against; \
                         review the new delta and write its digests",
                    ),
                );
            }
        }
        let mut ids = BTreeSet::new();
        for (index, written) in raw.acknowledged.iter().enumerate() {
            let Some(change) = delta
                .changes()
                .iter()
                .find(|change| change.id().to_string() == *written)
            else {
                errors.push(ValidationError::new(
                    ValidationCode::UndeclaredReference,
                    format!("acknowledgements.acknowledged[{index}]"),
                    format!("`{written}` is not a change of this delta"),
                ));
                continue;
            };
            if !ids.insert(change.id()) {
                errors.push(ValidationError::new(
                    ValidationCode::DuplicateDeclaration,
                    format!("acknowledgements.acknowledged[{index}]"),
                    format!("`{written}` is acknowledged twice"),
                ));
            }
        }
        errors.into_result(Self { ids })
    }

    /// Parses and admits an acknowledgements document for `delta`.
    ///
    /// # Errors
    /// [`AcknowledgementRefusal::Malformed`] when it is not the document's shape, and
    /// [`AcknowledgementRefusal::Invalid`] when [`Self::admit`] refuses it.
    pub fn from_json(text: &str, delta: &EssDelta) -> Result<Self, AcknowledgementRefusal> {
        let raw: RawAcknowledgements = serde_json::from_str(text)
            .map_err(|error| AcknowledgementRefusal::Malformed(error.to_string()))?;
        Self::admit(&raw, delta).map_err(AcknowledgementRefusal::Invalid)
    }

    /// The acknowledged ids, in canonical order.
    pub fn ids(&self) -> &BTreeSet<ChangeId> {
        &self.ids
    }
}

/// Why an acknowledgements document was not admitted.
#[derive(Debug, Clone)]
pub enum AcknowledgementRefusal {
    /// It is not JSON of the acknowledgements shape.
    Malformed(String),
    /// It is, and it does not acknowledge changes of this delta.
    Invalid(ValidationErrors),
}

impl fmt::Display for AcknowledgementRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Malformed(error) => write!(f, "acknowledgements: {error}"),
            Self::Invalid(errors) => write!(f, "{errors}"),
        }
    }
}

impl std::error::Error for AcknowledgementRefusal {}
