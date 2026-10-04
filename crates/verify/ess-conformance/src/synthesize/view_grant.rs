//! Who may read what, on a served surface (beyond10x/ess#286).
//!
//! From source `ess/22` an actor's `may:` may name a view. A view some actor names is
//! read-granted: a served component's surface answers it only to the actors naming it, and the
//! standard refusal to anyone else, before anything is read. A view no actor names stays open to
//! every caller. Three things follow for the suite, and this module does all three, for views a
//! served component (`reached_by: network`) answers and for no other.
//!
//! - **Who reads.** Every read of a read-granted view the suite already makes is sent as an actor
//!   naming it, the lowest-named: a [`ReadAs`](ScenarioStep::ReadAs) step goes before the read
//!   wherever the reader in force does not name the view. Reads of open views are left as they
//!   are.
//! - **Who is refused.** One `<view>/grant/read/denied` per read-granted view: the view read as
//!   the lowest-named declared actor its grant does not name, where there is one, and then as no
//!   actor at all, each requiring the standard refusal of the read naming whom it was read as.
//!   Where every declared actor names the view only the no-actor read is made, and a
//!   [`Note::ReadGrantedToEveryActor`] says so.
//! - **Who is admitted.** One `<view>/grant/read/admitted/<actor>` per actor naming the view, read
//!   as that actor and requiring it served, so a surface whose read grants drop one fails.
//!
//! A view without parameters is read on its own. A view with parameters borrows the arrangement
//! of the shortest scenario that reads it, so its parameters are bound; where none reads it, a
//! [`Note::ReadGrantUnwitnessed`] names it. A model that names a view in a grant and serves nothing
//! gets one [`Note::ReadGrantEnforcedByCaller`] instead. A model that names no view gets none of
//! this, and its suite keeps its bytes.

use super::{
    clipped, insert, ActorRef, BTreeSet, ConformanceScenario, ConformanceSuite, EssIr,
    EssSemanticRef, Note, QualifiedName, Refusal, ScenarioId, ScenarioStep, Synthesis,
    ViewExpectation, ViewRef,
};

/// Every read-grant scenario and note the model's served surfaces owe, after the suite is
/// otherwise finished: the reader of every read of a read-granted view, then the refusals and the
/// admissions.
pub(super) fn read_grants(ir: &EssIr, synthesis: &mut Synthesis) {
    if !ir.grants_reads() {
        return;
    }
    let granted: Vec<QualifiedName> = served_views(ir)
        .into_iter()
        .filter(|view| ir.read_granted(view))
        .collect();
    if granted.is_empty() {
        if ess_gen::http::checks_grants(ir) {
            return;
        }
        synthesis.notes.push(Note::ReadGrantEnforcedByCaller);
        return;
    }
    for scenario in synthesis.suite.scenarios.values_mut() {
        send_reads_as_granted(ir, &granted, scenario);
    }
    for view in &granted {
        witness(ir, view, synthesis);
    }
}

/// The views a served component answers.
fn served_views(ir: &EssIr) -> BTreeSet<QualifiedName> {
    ir.components()
        .values()
        .filter(|component| ess_gen::http::grants_checked_on(ir, component))
        .flat_map(|component| ess_gen::http::routes(ir, component))
        .filter_map(|route| match route.serves {
            ess_gen::http::Served::View(handle) => Some(handle.name().clone()),
            ess_gen::http::Served::Command(_) => None,
        })
        .collect()
}

/// The view a step reads from the target, if it reads one.
fn read_of(step: &ScenarioStep) -> Option<&ViewRef> {
    match step {
        ScenarioStep::QueryView { view, .. }
        | ScenarioStep::EventuallyView { view, .. }
        | ScenarioStep::ExpectHalt { view, .. }
        | ScenarioStep::EventuallyHalt { view, .. } => Some(view),
        _ => None,
    }
}

/// The lowest-named actor whose `may:` names `view`.
fn first_reader(ir: &EssIr, view: &QualifiedName) -> Option<ActorRef> {
    ir.readers(view)
        .next()
        .map(|actor| ActorRef::new(actor.name.clone()))
}

/// Puts a [`ReadAs`](ScenarioStep::ReadAs) before every read of a read-granted view whose reader in
/// force does not name it.
fn send_reads_as_granted(
    ir: &EssIr,
    granted: &[QualifiedName],
    scenario: &mut ConformanceScenario,
) {
    let mut reader: Option<ActorRef> = None;
    let mut steps = Vec::with_capacity(scenario.steps.len());
    let mut readers = BTreeSet::new();
    for step in std::mem::take(&mut scenario.steps) {
        if let ScenarioStep::ReadAs { actor } = &step {
            reader.clone_from(actor);
        }
        if let Some(view) = read_of(&step).filter(|view| granted.contains(view.name())) {
            let holds = reader.as_ref().is_some_and(|actor| {
                ir.actors()
                    .get(actor.name())
                    .is_some_and(|declared| declared.may_read(view.name()))
            });
            if !holds {
                if let Some(actor) = first_reader(ir, view.name()) {
                    steps.push(ScenarioStep::ReadAs {
                        actor: Some(actor.clone()),
                    });
                    readers.insert(actor.clone());
                    reader = Some(actor);
                }
            }
        }
        steps.push(step);
    }
    scenario.steps = steps;
    for actor in readers {
        scenario.source.insert(actor.into());
    }
}

/// The steps that bind a read of `view`: none for a view without parameters, and otherwise the
/// shortest scenario's steps up to the first read of it, that read excluded. `None` where nothing
/// binds them.
fn arrangement(ir: &EssIr, view: &QualifiedName, suite: &ConformanceSuite) -> Option<Arranged> {
    let declared = ir.views().get(view)?;
    if declared.params.is_empty() {
        return Some(Arranged {
            steps: Vec::new(),
            read: ScenarioStep::QueryView {
                view: ViewRef::new(view.clone()),
                params: std::collections::BTreeMap::new(),
            },
            source: BTreeSet::new(),
        });
    }
    suite
        .scenarios
        .iter()
        .filter(|(id, _)| !is_read_grant(id))
        .filter_map(|(_, scenario)| {
            let at = scenario.steps.iter().position(|step| {
                matches!(step, ScenarioStep::QueryView { view: read, .. } if read.name() == view)
            })?;
            Some((scenario, at))
        })
        .min_by_key(|(scenario, at)| (*at, scenario.steps.len()))
        .map(|(scenario, at)| Arranged {
            steps: scenario.steps[..at].to_vec(),
            read: scenario.steps[at].clone(),
            source: scenario.source.clone(),
        })
}

/// A read of a view, and what has to happen before it.
struct Arranged {
    steps: Vec<ScenarioStep>,
    read: ScenarioStep,
    source: BTreeSet<EssSemanticRef>,
}

/// Whether `id` is one this module files, which nothing it does rewrites.
fn is_read_grant(id: &ScenarioId) -> bool {
    matches!(
        id,
        ScenarioId::ReadGrant { .. } | ScenarioId::ReadGrantAdmitted { .. }
    )
}

/// One read-granted view's refusal and admissions.
fn witness(ir: &EssIr, view: &QualifiedName, synthesis: &mut Synthesis) {
    let view_ref = ViewRef::new(view.clone());
    let Some(arranged) = arrangement(ir, view, &synthesis.suite) else {
        synthesis
            .notes
            .push(Note::ReadGrantUnwitnessed { view: view_ref });
        return;
    };
    let denied = ir
        .actors()
        .values()
        .find(|actor| !actor.may_read(view))
        .map(|actor| ActorRef::new(actor.name.clone()));
    let mut filed: Vec<(ScenarioId, ConformanceScenario)> = vec![(
        ScenarioId::ReadGrant {
            view: view_ref.clone(),
        },
        denied_reads(&arranged, view, denied.as_ref()),
    )];
    if denied.is_none() {
        synthesis.notes.push(Note::ReadGrantedToEveryActor {
            view: view_ref.clone(),
        });
    }
    for actor in ir.readers(view) {
        let reader = ActorRef::new(actor.name.clone());
        filed.push((
            ScenarioId::ReadGrantAdmitted {
                view: view_ref.clone(),
                actor: reader.clone(),
            },
            read_as(
                &arranged,
                Some(&reader),
                ScenarioStep::ExpectView {
                    view: view_ref.clone(),
                    expectation: ViewExpectation::Counts {
                        at_least: Some(0),
                        at_most: None,
                    },
                },
                &format!("`{view}` read as `{reader}`, whose grant names it, is served"),
            ),
        ));
    }
    let mut refusals: Vec<Refusal> = Vec::new();
    for (id, scenario) in filed {
        insert(&mut synthesis.suite, id, scenario, &mut refusals);
    }
    synthesis.refusals.extend(refusals);
}

/// The denied reads of a read-granted view: as `sender`, an actor its grant does not name, where
/// one is declared, and always as no actor, each requiring the standard refusal of the read
/// naming whom it was read as.
fn denied_reads(
    arranged: &Arranged,
    view: &QualifiedName,
    sender: Option<&ActorRef>,
) -> ConformanceScenario {
    let mut steps = arranged.steps.clone();
    let mut source = arranged.source.clone();
    if let Some(read) = read_of(&arranged.read) {
        source.insert(read.clone().into());
    }
    for actor in sender.into_iter().map(Some).chain([None]) {
        steps.push(ScenarioStep::ReadAs {
            actor: actor.cloned(),
        });
        steps.push(arranged.read.clone());
        steps.push(ScenarioStep::ExpectNotGranted {
            actor: actor.cloned(),
            unpublished: Vec::new(),
        });
        if let Some(actor) = actor {
            source.insert(actor.clone().into());
        }
    }
    let purpose = match sender {
        Some(sender) => format!(
            "`{view}` read as `{sender}`, whose grant does not name it, and as no actor is \
             refused before anything is read"
        ),
        None => format!("`{view}` read as no actor is refused before anything is read"),
    };
    ConformanceScenario::new(clipped(&purpose), steps, source)
}

/// The arranged read, sent as `reader` or as no actor, followed by `then`.
fn read_as(
    arranged: &Arranged,
    reader: Option<&ActorRef>,
    then: ScenarioStep,
    purpose: &str,
) -> ConformanceScenario {
    let mut steps = arranged.steps.clone();
    steps.push(ScenarioStep::ReadAs {
        actor: reader.cloned(),
    });
    steps.push(arranged.read.clone());
    steps.push(then);
    let mut source = arranged.source.clone();
    if let Some(view) = read_of(&arranged.read) {
        source.insert(view.clone().into());
    }
    if let Some(reader) = reader {
        source.insert(reader.clone().into());
    }
    ConformanceScenario::new(clipped(purpose), steps, source)
}
