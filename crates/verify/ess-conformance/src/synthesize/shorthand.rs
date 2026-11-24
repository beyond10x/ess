//! No scenario observes a struct with a shorthand (`ess/27`, beyond10x/ess#594).
//!
//! A target may answer a value of such a struct in its short form, the field's scalar alone, and
//! every runner's observer reads a struct as its object only. A scenario that would observe one — in
//! a command response, an event payload, an error's fields or a view row — would fail a target that
//! honours the specification, so [`withdraw_observed`] takes it out of the finished suite and
//! refuses it naming `shorthand` and the type. A model with no shorthand is left as it was.

use std::collections::BTreeSet;

use ess_compiler::ir::{EssIr, ResolvedBody, ResolvedField};
use ess_domain::QualifiedName;

use crate::scenario::ScenarioStep;
use crate::witness::WitnessGap;

use super::{RefusalCause, Synthesis};

/// Why a scenario observing a struct with a shorthand has no witness.
pub(super) const OBSERVED: &str = "may be answered in its short form, the shorthand field's value \
                                   alone, which no observer decodes yet (ess/27)";

/// Every scenario of `synthesis` that observes a value reaching a struct with a shorthand,
/// withdrawn and refused with that struct.
pub(super) fn withdraw_observed(ir: &EssIr, synthesis: &mut Synthesis) {
    if ir.shorthands().is_empty() {
        return;
    }
    let reaching = reaching_shorthand(ir);
    let withdrawn: Vec<_> = synthesis
        .suite
        .scenarios
        .iter()
        .filter_map(|(id, scenario)| {
            let mut observed = BTreeSet::new();
            for step in &scenario.steps {
                observed_by(ir, step, &mut observed);
            }
            if let Some(trace) = &scenario.one_time_response {
                for origin in &trace.origins {
                    observed.extend(response_of(ir, origin.command.name()));
                }
            }
            observed.retain(|name| reaching.contains(name));
            if observed.is_empty() {
                return None;
            }
            first_shorthand(ir, observed).map(|(name, field)| {
                (
                    id.clone(),
                    RefusalCause::NoWitness(WitnessGap {
                        path: format!("{name}.shorthand"),
                        type_ref: format!("{name}, `shorthand: {field}`"),
                        reason: OBSERVED,
                    }),
                )
            })
        })
        .collect();
    for (id, cause) in withdrawn {
        super::withdraw(synthesis, &id, cause);
    }
}

/// The declared types `step` reads off a target's answer, added to `observed`.
fn observed_by(ir: &EssIr, step: &ScenarioStep, observed: &mut BTreeSet<QualifiedName>) {
    let fields = |declared: Option<&Vec<ResolvedField>>| {
        declared
            .into_iter()
            .flatten()
            .flat_map(|field| named(&field.type_ref))
            .collect::<Vec<_>>()
    };
    match step {
        ScenarioStep::ExpectEvent { event, .. }
        | ScenarioStep::EventuallyEvent { event, .. }
        | ScenarioStep::ExpectEventValues { event, .. } => {
            observed.extend(fields(ir.events().get(event.name()).map(|e| &e.fields)));
        }
        ScenarioStep::ExpectError { error, .. } => {
            observed.extend(fields(ir.errors().get(error.name()).map(|e| &e.fields)));
        }
        ScenarioStep::ExpectDirectResponse { response } => {
            observed.extend(response.declarations.keys().cloned());
        }
        ScenarioStep::ExpectResponsePayload { response } => {
            observed.extend(response.declarations.keys().cloned());
        }
        ScenarioStep::CaptureCommandResult { capture }
        | ScenarioStep::ExpectReplayResult { capture } => {
            observed.extend(capture.declarations.keys().cloned());
        }
        ScenarioStep::SnapshotCompleteSubject { view, shape, .. } => {
            observed.extend(shape.declarations.keys().cloned());
            observed.extend(fields(ir.views().get(view.name()).map(|v| &v.fields)));
        }
        ScenarioStep::ExpectView { view, .. }
        | ScenarioStep::EventuallyView { view, .. }
        | ScenarioStep::SnapshotSubject { view, .. }
        | ScenarioStep::SnapshotView { view }
        | ScenarioStep::ExpectSubjectUnchanged { view }
        | ScenarioStep::ExpectViewUnchanged { view }
        | ScenarioStep::ExpectCompleteSubjectUnchanged { view }
        | ScenarioStep::ExpectSubjectAbsent { view, .. } => {
            observed.extend(fields(ir.views().get(view.name()).map(|v| &v.fields)));
        }
        _ => {}
    }
}

/// The declared types a command's response names.
fn response_of(ir: &EssIr, command: &QualifiedName) -> Vec<QualifiedName> {
    ir.commands()
        .get(command)
        .into_iter()
        .flat_map(|command| &command.response)
        .flat_map(|field| named(&field.type_ref))
        .collect()
}

fn named(ty: &ess_compiler::ir::ResolvedTypeRef) -> Vec<QualifiedName> {
    crate::accessor::unresolve(ty)
        .named_dependencies()
        .into_iter()
        .cloned()
        .collect()
}

thread_local! {
    /// How many traversals of the declared types ran on this thread.
    static TRAVERSALS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

/// What `run` returns, and how many traversals of the declared types ran on this thread while it
/// ran: the work a test bounds.
pub(super) fn traversals<R>(run: impl FnOnce() -> R) -> (R, u64) {
    let before = TRAVERSALS.with(std::cell::Cell::get);
    let out = run();
    (out, TRAVERSALS.with(std::cell::Cell::get) - before)
}

/// The count of [`TRAVERSALS`] on this thread.
pub(super) fn taken_traversals() -> u64 {
    TRAVERSALS.with(std::cell::Cell::get)
}

/// Adds `more` to the count of [`TRAVERSALS`] on this thread.
pub(super) fn add_traversals(more: u64) {
    TRAVERSALS.with(|it| it.set(it.get() + more));
}

/// Every declared type from which a value reaches a struct with a shorthand, itself included: one
/// traversal of the declared types against the reverse of their references, made once per run.
pub(super) fn reaching_shorthand(ir: &EssIr) -> BTreeSet<QualifiedName> {
    TRAVERSALS.with(|count| count.set(count.get() + 1));
    let mut referrers: std::collections::BTreeMap<QualifiedName, Vec<QualifiedName>> =
        std::collections::BTreeMap::new();
    for (name, declared) in ir.types() {
        let targets: Vec<QualifiedName> = match &declared.body {
            ResolvedBody::Newtype { of, .. } => named(of),
            ResolvedBody::Struct { fields, .. } => fields
                .iter()
                .flat_map(|field| named(&field.type_ref))
                .collect(),
            ResolvedBody::Union { variants, .. } => {
                variants.values().flatten().flat_map(named).collect()
            }
            ResolvedBody::Enum { .. } => Vec::new(),
        };
        for target in targets {
            referrers.entry(target).or_default().push(name.clone());
        }
    }
    let mut reaching = BTreeSet::new();
    let mut pending: Vec<QualifiedName> = ir
        .types()
        .keys()
        .filter(|name| ir.shorthand(name).is_some())
        .cloned()
        .collect();
    while let Some(name) = pending.pop() {
        if reaching.insert(name.clone()) {
            pending.extend(referrers.get(&name).into_iter().flatten().cloned());
        }
    }
    reaching
}

/// The first struct with a shorthand, in name order, that a value of one of `roots` reaches, with
/// the field it names.
fn first_shorthand(ir: &EssIr, roots: BTreeSet<QualifiedName>) -> Option<(QualifiedName, String)> {
    TRAVERSALS.with(|count| count.set(count.get() + 1));
    let mut seen = BTreeSet::new();
    let mut pending: Vec<_> = roots.into_iter().collect();
    let mut found = BTreeSet::new();
    while let Some(name) = pending.pop() {
        if !seen.insert(name.clone()) {
            continue;
        }
        if let Some(field) = ir.shorthand(&name) {
            found.insert((name.clone(), field.to_owned()));
        }
        let Some(declared) = ir.types().get(&name) else {
            continue;
        };
        match &declared.body {
            ResolvedBody::Newtype { of, .. } => pending.extend(named(of)),
            ResolvedBody::Struct { fields, .. } => {
                pending.extend(fields.iter().flat_map(|field| named(&field.type_ref)));
            }
            ResolvedBody::Union { variants, .. } => {
                pending.extend(variants.values().flatten().flat_map(named));
            }
            ResolvedBody::Enum { .. } => {}
        }
    }
    found.into_iter().next()
}

/// Whether a fresh traversal from `name` reaches a struct with a shorthand.
pub(super) fn reaches_fresh(ir: &EssIr, name: &QualifiedName) -> bool {
    first_shorthand(ir, BTreeSet::from([name.clone()])).is_some()
}
