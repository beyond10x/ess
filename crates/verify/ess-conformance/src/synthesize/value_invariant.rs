//! Inventory and arrangement of wrapped value-invariant positions
//! (`docs/design/wrapped-value-invariant-observations.md`, "Finite graph and stable obligation
//! identities", "Arrangement is a separate proof").
//!
//! Every view field is walked as a typed graph: `Optional` adds `present`, `List` adds `element`,
//! `Map` adds `map_value`, a struct adds `field`, a union adds `variant` in label order, a newtype
//! adds `unwrap`. Each entry to the value type is a site. A site reached only through fields and
//! newtypes whose rebased predicates a row already answers keeps the direct
//! [`ScenarioId::ValueInvariant`] scenario; every other site becomes a
//! [`ScenarioId::ValueInvariantWrapped`] scenario observed by
//! [`ViewExpectation::ValueInvariants`], or a refusal under that id.
//!
//! The arrangement is the first declared outcome whose after-state the view shows and whose row
//! the scenario determined: the row's value at the root is a literal the scenario sent, and the
//! observer itself, run over that literal, finds an occurrence and every invariant holding. A row
//! whose value at the root the scenario does not know cannot license a present witness, so such a
//! position is refused by name rather than asserted.

use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::ir::{EssIr, ResolvedBody, ResolvedType, ResolvedTypeRef, ResolvedView};
use ess_domain::entity::Invariant;
use ess_domain::name::QualifiedName;
use ess_domain::view::AssertionStyle;

use super::{
    bound, clipped, identity_of, input_types, insert, reachable_types, run, shown, shows_row,
    Refusal, RefusalCause,
};
use crate::scenario::{
    ActorRef, CommandRef, ConformanceScenario, ConformanceSuite, DeclaredTypeRef, EntityRef,
    EssSemanticRef, OutcomeRef, ScenarioId, ScenarioStep, ScenarioValue, ViewExpectation, ViewRef,
};
use crate::value_invariant::{
    Contract, Declaration, DeclaredInvariant, InvariantPosition, Observed, ObservedField, Segment,
    Selector, ValueInvariants, Witness, DEFAULT_WORK_LIMIT, MAX_EDGES,
};
use crate::witness::WitnessGap;

/// The reason a value-invariant site reached across a recursive back-edge is refused.
pub(super) const RECURSIVE: &str = "is a value-invariant site reached across a recursive \
                                    back-edge, which this build does not synthesize";

/// The reason a view field whose type expands past [`MAX_EDGES`] is refused.
pub(super) const OVERFLOW: &str = "expands past 16384 typed edges, so its value-invariant \
                                   positions are not all known";

/// One root's expansion: its ordinary sites, its recursive (back-edge, site) pairs, and whether it
/// outgrew [`MAX_EDGES`].
#[derive(Default)]
struct Expansion {
    sites: Vec<Vec<Segment>>,
    recursive: Vec<(Vec<Segment>, Vec<Segment>)>,
    edges: usize,
    overflow: bool,
}

/// Walks one view field's type, collecting every site of `wanted`.
fn expand(ir: &EssIr, root: &ResolvedTypeRef, wanted: &QualifiedName) -> Expansion {
    let mut expansion = Expansion::default();
    let mut back_edges: Vec<(Vec<Segment>, Vec<Segment>)> = Vec::new();
    let mut active: Vec<(QualifiedName, Vec<Segment>)> = Vec::new();
    walk(
        ir,
        root,
        wanted,
        &mut Vec::new(),
        &mut active,
        &mut back_edges,
        &mut expansion,
    );
    // A back-edge E to an entry path Q pairs with every site in Q's subtree.
    for (edge, target) in back_edges {
        for site in &expansion.sites {
            if site.starts_with(&target) {
                expansion.recursive.push((edge.clone(), site.clone()));
            }
        }
    }
    expansion.recursive.sort();
    expansion.recursive.dedup();
    expansion
}

fn walk(
    ir: &EssIr,
    type_ref: &ResolvedTypeRef,
    wanted: &QualifiedName,
    path: &mut Vec<Segment>,
    active: &mut Vec<(QualifiedName, Vec<Segment>)>,
    back_edges: &mut Vec<(Vec<Segment>, Vec<Segment>)>,
    out: &mut Expansion,
) {
    let child = |segment: Segment,
                 type_ref: &ResolvedTypeRef,
                 path: &mut Vec<Segment>,
                 active: &mut Vec<(QualifiedName, Vec<Segment>)>,
                 back_edges: &mut Vec<(Vec<Segment>, Vec<Segment>)>,
                 out: &mut Expansion| {
        out.edges += 1;
        if out.edges > MAX_EDGES {
            out.overflow = true;
            return;
        }
        path.push(segment);
        walk(ir, type_ref, wanted, path, active, back_edges, out);
        path.pop();
    };
    match type_ref {
        ResolvedTypeRef::Primitive { .. } => {}
        ResolvedTypeRef::Optional { of } => {
            child(Segment::Present, of, path, active, back_edges, out);
        }
        ResolvedTypeRef::List { of } => child(Segment::Element, of, path, active, back_edges, out),
        ResolvedTypeRef::Map { value, .. } => {
            child(Segment::MapValue, value, path, active, back_edges, out);
        }
        ResolvedTypeRef::Declared { name } => {
            let qualified = name.name();
            if let Some((_, entry)) = active.iter().find(|(active, _)| active == qualified) {
                back_edges.push((path.clone(), entry.clone()));
                return;
            }
            if qualified == wanted {
                out.sites.push(path.clone());
            }
            active.push((qualified.clone(), path.clone()));
            match &ir.named_type(name).body {
                ResolvedBody::Newtype { of, .. } => {
                    child(Segment::Unwrap, of, path, active, back_edges, out);
                }
                ResolvedBody::Struct { fields, .. } => {
                    for field in fields {
                        child(
                            Segment::Field {
                                name: field.name.clone(),
                            },
                            &field.type_ref,
                            path,
                            active,
                            back_edges,
                            out,
                        );
                    }
                }
                ResolvedBody::Union { variants, .. } => {
                    for (label, payload) in variants {
                        if let Some(payload) = payload {
                            child(
                                Segment::Variant {
                                    label: label.clone(),
                                },
                                payload,
                                path,
                                active,
                                back_edges,
                                out,
                            );
                        }
                    }
                }
                ResolvedBody::Enum { .. } => {}
            }
            active.pop();
        }
    }
}

/// The dotted field path a direct position is filed under, when the route holds only fields and
/// newtypes.
fn direct(root: &str, path: &[Segment]) -> Option<String> {
    let mut dotted = vec![root.to_owned()];
    for segment in path {
        match segment {
            Segment::Field { name } => dotted.push(name.clone()),
            Segment::Unwrap => {}
            _ => return None,
        }
    }
    Some(dotted.join("."))
}

/// The route as a person reads it: `tags[]`, `byname{}`, `badge?.serial`, `payee<person>`.
fn readable(root: &str, path: &[Segment]) -> String {
    let mut text = root.to_owned();
    for segment in path {
        match segment {
            Segment::Field { name } => {
                text.push('.');
                text.push_str(name);
            }
            Segment::Present => text.push('?'),
            Segment::Element => text.push_str("[]"),
            Segment::MapValue => text.push_str("{}"),
            Segment::Variant { label } => {
                text.push('<');
                text.push_str(label);
                text.push('>');
            }
            Segment::Unwrap => {}
        }
    }
    text
}

fn invariants_of(invariants: &[Invariant]) -> Vec<DeclaredInvariant> {
    invariants
        .iter()
        .map(|invariant| DeclaredInvariant {
            statement: invariant.statement.clone(),
            predicate: invariant.predicate.clone(),
        })
        .collect()
}

/// The declarations the root and the identity reach, or why one cannot be carried.
fn declarations(
    ir: &EssIr,
    reached: &[&ResolvedTypeRef],
) -> Result<BTreeMap<String, Declaration>, &'static str> {
    let mut names = BTreeSet::new();
    for type_ref in reached {
        reachable_types(ir, type_ref, &mut names);
    }
    let mut out = BTreeMap::new();
    for name in names {
        let Some(declared) = ir.types().get(name.name()) else {
            return Err("reaches a type the model does not declare");
        };
        let declaration = match &declared.body {
            ResolvedBody::Newtype {
                of,
                alphabet,
                prefix,
                invariants,
            } => Declaration::Newtype {
                of: of.to_string(),
                invariants: invariants_of(invariants),
                alphabet: alphabet.clone(),
                prefix: prefix.clone(),
            },
            ResolvedBody::Struct { fields, invariants } => {
                if ir.undeclared_fields(name.name()) == ess_domain::types::UndeclaredFields::Ignored
                {
                    return Err(
                        "reaches a struct that ignores undeclared fields, which the observer's \
                         closed declarations cannot carry",
                    );
                }
                Declaration::Struct {
                    fields: fields
                        .iter()
                        .map(|field| ObservedField {
                            name: field.name.clone(),
                            type_ref: field.type_ref.to_string(),
                            presence: field.naming.presence,
                        })
                        .collect(),
                    invariants: invariants_of(invariants),
                }
            }
            ResolvedBody::Enum { variants } => Declaration::Enum {
                variants: variants
                    .iter()
                    .map(|variant| variant.name.clone())
                    .collect(),
            },
            ResolvedBody::Union { tag, variants } => Declaration::Union {
                tag: tag.clone(),
                variants: variants
                    .iter()
                    .map(|(label, payload)| {
                        payload
                            .as_ref()
                            .map(|payload| (label.clone(), payload.to_string()))
                            .ok_or(
                                "reaches a union with a unit variant, which the observer's \
                                 declarations cannot carry",
                            )
                    })
                    .collect::<Result<_, _>>()?,
            },
        };
        out.insert(name.name().to_string(), declaration);
    }
    Ok(out)
}

/// The wrapped positions of one value type, each a scenario or a refusal under its id.
///
/// `direct` holds the `(view, dotted field)` positions [`super::value_object_invariants`] already
/// files a direct scenario under, which no wrapped duplicate repeats.
pub(super) fn wrapped(
    ir: &EssIr,
    declared: &ResolvedType,
    invariants: &[Invariant],
    direct_positions: &BTreeSet<(QualifiedName, String)>,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    suite: &mut ConformanceSuite,
    refusals: &mut Vec<Refusal>,
) -> usize {
    let mut found = 0;
    for view in ir.views().values().filter(|view| !view.is_aggregate()) {
        for field in &view.fields {
            let expansion = expand(ir, &field.type_ref, &declared.name);
            let gap = |path: String, reason: &'static str| {
                RefusalCause::NoWitness(WitnessGap {
                    path,
                    type_ref: field.type_ref.to_string(),
                    reason,
                })
            };
            for (edge, site) in &expansion.recursive {
                found += 1;
                let position = InvariantPosition::Recursive {
                    edge: edge.clone(),
                    site: site.clone(),
                };
                let id = ScenarioId::ValueInvariantWrapped {
                    value: DeclaredTypeRef::new(declared.name.clone()),
                    at: ViewRef::new(view.name.clone()),
                    root: field.name.clone(),
                    digest: crate::value_invariant::position_digest(&position),
                };
                refusals.push(Refusal::about(
                    &id,
                    gap(
                        format!("{}.{}", view.name, readable(&field.name, site)),
                        RECURSIVE,
                    ),
                ));
            }
            if expansion.overflow {
                found += 1;
                refusals.push(Refusal {
                    subject: DeclaredTypeRef::new(declared.name.clone()).into(),
                    scenario: None,
                    stands: false,
                    cause: gap(format!("{}.{}", view.name, field.name), OVERFLOW),
                    caused_by: None,
                });
            }
            for path in expansion.sites {
                if let Some(dotted) = direct(&field.name, &path) {
                    if direct_positions.contains(&(view.name.clone(), dotted)) {
                        continue;
                    }
                }
                found += 1;
                let position = InvariantPosition::Site { path: path.clone() };
                let id = ScenarioId::ValueInvariantWrapped {
                    value: DeclaredTypeRef::new(declared.name.clone()),
                    at: ViewRef::new(view.name.clone()),
                    root: field.name.clone(),
                    digest: crate::value_invariant::position_digest(&position),
                };
                let at = format!("{}.{}", view.name, readable(&field.name, &path));
                match observed_at(ir, declared, view, field, &position, actors) {
                    Ok(scenario) => insert(suite, id, scenario, refusals),
                    Err(Unarranged::Carried(reason)) => {
                        refusals.push(Refusal::about(&id, gap(at, reason)));
                    }
                    Err(Unarranged::NoRow) => refusals.push(Refusal::about(
                        &id,
                        RefusalCause::ValueInvariantUnwitnessed {
                            value: DeclaredTypeRef::new(declared.name.clone()),
                            invariants: invariants
                                .iter()
                                .map(|invariant| invariant.statement.clone())
                                .collect(),
                            at: Some((
                                ViewRef::new(view.name.clone()),
                                readable(&field.name, &path),
                            )),
                        },
                    )),
                }
            }
        }
    }
    found
}

/// Why a wrapped position has no scenario.
enum Unarranged {
    /// No declared outcome leaves a row the view shows with a known occurrence there.
    NoRow,
    /// The declarations it reaches cannot be carried by the closed contract.
    Carried(&'static str),
}

/// The scenario that puts a row where the view shows it, with an occurrence of the type at the
/// position, and observes it.
#[allow(clippy::too_many_lines)]
fn observed_at(
    ir: &EssIr,
    declared: &ResolvedType,
    view: &ResolvedView,
    field: &ess_compiler::ir::ResolvedField,
    position: &InvariantPosition,
    actors: &BTreeMap<QualifiedName, ActorRef>,
) -> Result<ConformanceScenario, Unarranged> {
    let entity = ir.entity(&view.source);
    let projected = view
        .fields
        .iter()
        .find(|projected| {
            projected.name == entity.identity.name && projected.type_ref == entity.identity.type_ref
        })
        .filter(|identity| identity.name != field.name);
    let mut reached = vec![&field.type_ref];
    if let Some(identity) = projected {
        reached.push(&identity.type_ref);
    }
    for command in ir.commands().values() {
        for outcome in &command.outcomes {
            let Some(subject) = &outcome.subject else {
                continue;
            };
            if subject.entity != view.source {
                continue;
            }
            // The scenario observes the value this branch copies, so an `Optional` input it copies
            // is sent present here (beyond10x/ess#546).
            let _under_test = crate::witness::under_test(ir, command, outcome);
            let Ok(arranged) = run(ir, command, outcome, actors) else {
                continue;
            };
            let Some(state) = arranged.after.clone() else {
                continue;
            };
            let known = identity_of(subject, arranged.instance.as_ref());
            let params = bound(ir, view, &arranged.settled, known.as_ref());
            if !matches!(
                shows_row(ir, view, &state, &arranged.settled, known.as_ref(), &params),
                Ok(true)
            ) {
                continue;
            }
            // The row the scenario determined: its value at the root must be a literal it sent.
            let row = shown(view, BTreeMap::new(), &arranged.settled);
            let Some(ScenarioValue::Literal { value }) = row.get(&field.name) else {
                continue;
            };
            let witness = match (projected, &known) {
                (Some(identity), Some(expected)) => Witness::ArrangedRow {
                    selector: Selector::ProjectedIdentity {
                        entity: EntityRef::from(&view.source),
                        identity: ObservedField {
                            name: identity.name.clone(),
                            type_ref: identity.type_ref.to_string(),
                            presence: None,
                        },
                        expected: expected.clone(),
                    },
                },
                _ => Witness::AnyObservedOccurrence,
            };
            let reached: Vec<&ResolvedTypeRef> = match &witness {
                Witness::AnyObservedOccurrence => vec![&field.type_ref],
                Witness::ArrangedRow { .. } => reached.clone(),
            };
            let observed = ValueInvariants {
                root: ObservedField {
                    name: field.name.clone(),
                    type_ref: field.type_ref.to_string(),
                    presence: field.naming.presence,
                },
                declarations: declarations(ir, &reached).map_err(Unarranged::Carried)?,
                value: DeclaredTypeRef::new(declared.name.clone()),
                position: position.clone(),
                witness,
                work_limit: DEFAULT_WORK_LIMIT,
            };
            // The observer over the row the scenario sent: a healthy target answers this row, so
            // it must hold an occurrence and satisfy every invariant.
            let contract = Contract::admit(&observed).map_err(|_| {
                Unarranged::Carried("reaches declarations the observer's contract cannot admit")
            })?;
            let literal = BTreeMap::from([(field.name.clone(), value.clone())]);
            if contract.observe(&[literal], &|_| true) != Observed::Held {
                continue;
            }

            let name = ViewRef::new(view.name.clone());
            let expectation = ViewExpectation::ValueInvariants(Box::new(observed));
            let mut steps = arranged.steps();
            match view.assertion_style {
                AssertionStyle::Expect => {
                    steps.push(ScenarioStep::QueryView {
                        view: name.clone(),
                        params,
                    });
                    steps.push(ScenarioStep::ExpectView {
                        view: name,
                        expectation,
                    });
                }
                AssertionStyle::Eventually => steps.push(ScenarioStep::EventuallyView {
                    view: name,
                    params,
                    expectation,
                }),
            }

            let mut source: BTreeSet<EssSemanticRef> = arranged.source.clone();
            let command_ref = CommandRef::new(command.name.clone());
            source.insert(command_ref.clone().into());
            source.insert(OutcomeRef::new(command_ref, outcome.name.clone()).into());
            source.insert(EntityRef::from(&view.source).into());
            source.insert(ViewRef::new(view.name.clone()).into());
            if let Some(actor) = arranged.actor.clone() {
                source.insert(actor.into());
            }
            source.extend(
                input_types(ir, command)
                    .into_iter()
                    .map(EssSemanticRef::from),
            );
            let mut of_value: BTreeSet<DeclaredTypeRef> = BTreeSet::new();
            for type_ref in reached.iter().copied() {
                reachable_types(ir, type_ref, &mut of_value);
            }
            source.extend(of_value.into_iter().map(EssSemanticRef::from));
            let text = format!(
                "every `{}` that `{}` publishes at `{}` satisfies what the type declares",
                declared.name,
                view.name,
                readable(&field.name, position_path(position))
            );
            return Ok(ConformanceScenario::new(clipped(&text), steps, source));
        }
    }
    Err(Unarranged::NoRow)
}

fn position_path(position: &InvariantPosition) -> &[Segment] {
    match position {
        InvariantPosition::Site { path } => path,
        InvariantPosition::Recursive { site, .. } => site,
    }
}
