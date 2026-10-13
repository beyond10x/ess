//! A command the system issues itself is awaited, never driven (beyond10x/ess#542,
//! `docs/design/system-issued-commands.md`, *Which branches a scenario can claim*).
//!
//! One rewrite pass over every built scenario, run once the families and the grant scenarios are
//! written. Each step that sends a system-issued command becomes the step its claimed branch asks
//! for:
//!
//! | the branch the step claims (its `expect_outcome`) | becomes |
//! |---|---|
//! | accepting | [`AwaitOutcome`](ScenarioStep::AwaitOutcome) on the identity input, stating no input; the expectations after it stay |
//! | `external:`, refusal or not | the same: the system issued it, and the provider's answer is the outcome reached |
//! | decided by the addressed record: `wrong_state:`, `unknown_instance:`, a held-state or stored-field guard with no input guard | [`ExpectNotIssued`](ScenarioStep::ExpectNotIssued); the steps that read the result nobody produced are dropped, the subject checks stay |
//! | decided by the input or the rows it names: `when:`, `when_related:`, a refusing default, `input_absent:`, an input guard beside a held state | the scenario is withdrawn, `ESS-SYNTH-024` naming the branch |
//!
//! A command whose outcome addresses no existing record ([`Unaddressed`]) withdraws every scenario
//! that needs a step of it, `ESS-SYNTH-023`. A grant refusal — the send followed directly by
//! [`ExpectNotGranted`](ScenarioStep::ExpectNotGranted) — stays driven: an outside caller must not
//! be able to issue the system's step.
//!
//! # A value the system decides is never compared as synthesis chose it
//!
//! Every input but the identity is the system's, so what synthesis sent for it is a value it
//! invented. Where the reached branch writes such an input into an event field, an error field or a
//! stored field, the expectation reads what the target reported instead:
//!
//! | the field's declared source | the expected value |
//! |---|---|
//! | the identity input, a literal, the subject's own field or state, a related row named by the subject | as synthesis wrote it |
//! | another input, top-level (`input.worker`) | [`ScenarioValue::Awaited`] of that input, in an event payload or a row of the subject read back after the most recent await |
//! | anything else that reads such an input (a path into it, a fallback, a related row it names, a selector over it), or an awaited input in an error field or a row of an earlier await | not compared; the shape still holds it |
//!
//! A binding's invocation expected after the await, carrying the value synthesis sent the
//! system-issued command, reads the awaited input too.

use std::collections::BTreeMap;

use ess_compiler::ir::{
    EssIr, ResolvedCommand, ResolvedCondition, ResolvedOutcome, ResolvedPayloadField,
    ResolvedPayloadValue, ResolvedRelatedVia,
};
use ess_domain::name::QualifiedName;

use super::{
    describe, is_external, mark_presence, ActorRef, CommandRef, ConformanceScenario,
    ConformanceSuite, EssSemanticRef, EventRef, OutcomeRef, PayloadShape, RefusalCause, ScenarioId,
    ScenarioStep, ScenarioValue, ViewExpectation,
};
use crate::system_issued::{identity_input, Unaddressed};

/// Every scenario that needs a step of a system-issued command, rewritten to await it; those that
/// cannot be, withdrawn from `suite` and returned with why. A model with no system-issued command
/// is left untouched.
pub(super) fn await_system_issued(
    ir: &EssIr,
    suite: &mut ConformanceSuite,
) -> Vec<(ScenarioId, RefusalCause)> {
    let issued: BTreeMap<&QualifiedName, &ResolvedCommand> = ir
        .commands()
        .iter()
        .filter(|(_, command)| command.system_issued)
        .collect();
    if issued.is_empty() {
        return Vec::new();
    }
    let mut withdrawn = Vec::new();
    for (id, scenario) in &mut suite.scenarios {
        match rewritten(ir, &issued, scenario) {
            Ok(Some(steps)) => {
                scenario.steps = steps;
                forget_unsent_actors(ir, scenario);
            }
            Ok(None) => {}
            Err(cause) => withdrawn.push((id.clone(), cause)),
        }
    }
    for (id, _) in &withdrawn {
        suite.scenarios.remove(id);
    }
    withdrawn
}

/// How the branch a send of a system-issued command claims is witnessed.
enum Claim {
    /// The system takes the step: an accepting branch, or an external one the scenario forces.
    Reached,
    /// The record the step addresses refuses it: nothing is issued.
    NotIssued,
    /// The input decides the refusal, which the system chooses: no scenario.
    Unclaimable(&'static str),
}

/// One await written into the scenario: what the expectations after it are read against.
struct Awaited<'ir> {
    outcome: Option<&'ir ResolvedOutcome>,
    identity: &'ir str,
    instance: ScenarioValue,
    /// What synthesis sent for each input the system decides.
    invented: BTreeMap<String, ScenarioValue>,
}

/// The command result the steps being read refer to.
enum Current {
    /// A command the scenario drives, or none yet.
    Driven,
    /// The most recent await (an index into the scenario's awaits).
    Awaited(usize),
    /// An `expect_not_issued`: no result was produced.
    Nothing,
}

/// The scenario's steps with every system-issued send rewritten; `None` where it sends none.
fn rewritten<'ir>(
    ir: &'ir EssIr,
    issued: &BTreeMap<&QualifiedName, &'ir ResolvedCommand>,
    scenario: &ConformanceScenario,
) -> Result<Option<Vec<ScenarioStep>>, RefusalCause> {
    let steps = &scenario.steps;
    if !steps
        .iter()
        .any(|step| sent(step).is_some_and(|(command, _)| issued.contains_key(command.name())))
    {
        return Ok(None);
    }
    let mut out = Vec::with_capacity(steps.len());
    let mut awaits: Vec<Awaited<'ir>> = Vec::new();
    let mut current = Current::Driven;
    for (at, step) in steps.iter().enumerate() {
        let Some((command_ref, input)) = sent(step) else {
            match current {
                Current::Nothing if reads_result(step) => {}
                Current::Awaited(latest) => {
                    out.push(read_against(ir, step, &awaits, latest, true));
                }
                _ if !awaits.is_empty() => {
                    out.push(read_against(ir, step, &awaits, awaits.len() - 1, false));
                }
                _ => out.push(step.clone()),
            }
            continue;
        };
        let Some(command) = issued.get(command_ref.name()).copied() else {
            current = Current::Driven;
            out.push(step.clone());
            continue;
        };
        if matches!(
            steps.get(at + 1),
            Some(ScenarioStep::ExpectNotGranted { .. })
        ) {
            current = Current::Driven;
            out.push(step.clone());
            continue;
        }
        let identity =
            identity_input(command).map_err(|reason| RefusalCause::SystemIssuedUnaddressed {
                command: command_ref.clone(),
                reason,
            })?;
        let outcome = claimed(command, &steps[at + 1..]);
        let claim = outcome.map_or(Claim::Reached, claim_of);
        let refused = |decided_by| {
            outcome.map_or(
                RefusalCause::SystemIssuedUnaddressed {
                    command: command_ref.clone(),
                    reason: Unaddressed::NoInstance,
                },
                |outcome| RefusalCause::SystemDecidedRefusal {
                    outcome: OutcomeRef::new(command_ref.clone(), outcome.name.clone()),
                    decided_by,
                },
            )
        };
        if let Claim::Unclaimable(decided_by) = claim {
            return Err(refused(decided_by));
        }
        // A send that leaves the identity out names no record: what it is refused for is the
        // input's absence, which the system decides.
        let Some(instance) = input.and_then(|input| input.get(identity)).cloned() else {
            return Err(refused("an input that names no record"));
        };
        let within = command.await_within;
        if matches!(claim, Claim::NotIssued) {
            out.push(ScenarioStep::ExpectNotIssued {
                command: command_ref.clone(),
                instance,
                within,
                unpublished: emitted_by(command),
            });
            current = Current::Nothing;
            continue;
        }
        out.push(ScenarioStep::AwaitOutcome {
            command: command_ref.clone(),
            instance: instance.clone(),
            identity: identity.to_owned(),
            within,
            input: BTreeMap::new(),
            shape: input_shape(ir, command),
        });
        awaits.push(Awaited {
            outcome,
            identity,
            instance,
            invented: input
                .into_iter()
                .flatten()
                .filter(|(field, _)| field.as_str() != identity)
                .map(|(field, value)| (field.clone(), value.clone()))
                .collect(),
        });
        current = Current::Awaited(awaits.len() - 1);
    }
    Ok(Some(out))
}

/// The command a step sends, and its input where it carries one.
fn sent(step: &ScenarioStep) -> Option<(&CommandRef, Option<&BTreeMap<String, ScenarioValue>>)> {
    match step {
        ScenarioStep::ExecuteCommand { command, input, .. } => Some((command, Some(input))),
        ScenarioStep::ExecuteCommandWithoutInput { command, .. } => Some((command, None)),
        _ => None,
    }
}

/// Whether `step` reads the preceding command's result, which an `expect_not_issued` leaves
/// unproduced: kept, it would read the result of whatever command ran before.
fn reads_result(step: &ScenarioStep) -> bool {
    matches!(
        step,
        ScenarioStep::ExpectOutcome { .. }
            | ScenarioStep::ExpectError { .. }
            | ScenarioStep::ExpectNoError
            | ScenarioStep::ExpectEvent { .. }
            | ScenarioStep::ExpectEventValues { .. }
            | ScenarioStep::ExpectNoEvent { .. }
            | ScenarioStep::ExpectNoEvents
            | ScenarioStep::ExpectDirectResponse { .. }
            | ScenarioStep::ExpectResponsePayload { .. }
            | ScenarioStep::CaptureCommandResult { .. }
    )
}

/// The branch the send claims: the first `expect_outcome` of the command before the next send.
fn claimed<'ir>(
    command: &'ir ResolvedCommand,
    after: &[ScenarioStep],
) -> Option<&'ir ResolvedOutcome> {
    after
        .iter()
        .take_while(|step| sent(step).is_none())
        .find_map(|step| match step {
            ScenarioStep::ExpectOutcome { outcome } if outcome.command.name() == &command.name => {
                command
                    .outcomes
                    .iter()
                    .find(|declared| declared.name == outcome.outcome)
            }
            _ => None,
        })
}

/// How a branch of a system-issued command is witnessed, by what selects it.
fn claim_of(outcome: &ResolvedOutcome) -> Claim {
    if is_external(outcome) || outcome.error.is_none() {
        return Claim::Reached;
    }
    match &outcome.condition {
        ResolvedCondition::WrongState
        | ResolvedCondition::UnknownInstance
        | ResolvedCondition::ExistingInstance => Claim::NotIssued,
        ResolvedCondition::SubjectState { predicate, .. }
        | ResolvedCondition::SubjectField { predicate, .. }
        | ResolvedCondition::StateChange { predicate, .. } => match predicate {
            None => Claim::NotIssued,
            Some(_) => Claim::Unclaimable("an input guard beside the held state"),
        },
        ResolvedCondition::SubjectPredicate { predicate, input } => {
            if input.is_some() || mentions_input(&predicate.to_string()) {
                Claim::Unclaimable("an input guard beside the stored-field guard")
            } else {
                Claim::NotIssued
            }
        }
        ResolvedCondition::Related { .. } | ResolvedCondition::RelatedSet { .. } => {
            Claim::Unclaimable("a `when_related:` guard over the rows its input names")
        }
        ResolvedCondition::When { .. } => Claim::Unclaimable("a `when:` guard over its input"),
        ResolvedCondition::Otherwise => {
            Claim::Unclaimable("a refusing default, taken for the inputs no other branch takes,")
        }
        ResolvedCondition::InputAbsent => Claim::Unclaimable("`input_absent:`"),
        ResolvedCondition::External { .. } | ResolvedCondition::ExternalWhen { .. } => {
            Claim::Reached
        }
    }
}

/// Whether a rendered predicate or selector reads the command's input. Read conservatively: a
/// false reading only leaves a value uncompared.
fn mentions_input(rendered: &str) -> bool {
    rendered.contains("input")
}

/// Every event any outcome of the command emits, in name order.
fn emitted_by(command: &ResolvedCommand) -> Vec<EventRef> {
    command
        .outcomes
        .iter()
        .flat_map(|outcome| outcome.emits.iter().map(EventRef::from))
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect()
}

/// The command's declared input, flattened as an event payload's is.
fn input_shape(ir: &EssIr, command: &ResolvedCommand) -> PayloadShape {
    let mut shape = PayloadShape::new();
    for field in &command.input {
        describe(ir, &field.type_ref, &field.name, false, 0, &mut shape);
        mark_presence(field, &field.name, false, &mut shape);
    }
    shape
}

/// What a value source of the reached branch leaves an expectation.
enum Source {
    /// The arrangement decided it: as synthesis wrote it.
    Kept,
    /// The input of this name, as the target reported it.
    Awaited(String),
    /// A value the system decides through its input, which no expectation states.
    Uncompared,
}

/// What `value` leaves an expectation, where `identity` is the awaited instance's input.
fn source(value: &ResolvedPayloadValue, identity: &str) -> Source {
    match value {
        ResolvedPayloadValue::InputField { field, .. } if field == identity => Source::Kept,
        ResolvedPayloadValue::InputField { field, .. }
            if !ess_domain::command::input_path::is_path(field) =>
        {
            Source::Awaited(field.clone())
        }
        ResolvedPayloadValue::InputField { .. } | ResolvedPayloadValue::InputOrGenerated { .. } => {
            Source::Uncompared
        }
        ResolvedPayloadValue::Struct { fields } => {
            if fields
                .iter()
                .all(|field| matches!(source(&field.value, identity), Source::Kept))
            {
                Source::Kept
            } else {
                Source::Uncompared
            }
        }
        ResolvedPayloadValue::RelatedField {
            via: ResolvedRelatedVia::Input { field, .. },
            ..
        } if field != identity => Source::Uncompared,
        ResolvedPayloadValue::RelatedSelection { selection, .. }
        | ResolvedPayloadValue::RelatedHead { selection, .. }
            if mentions_input(&selection.filter.to_string()) =>
        {
            Source::Uncompared
        }
        _ => Source::Kept,
    }
}

/// `step`, its expectations read against the awaits written before it. `current` says whether
/// `step` reads the result of the most recent await (`latest`) directly.
fn read_against(
    ir: &EssIr,
    step: &ScenarioStep,
    awaits: &[Awaited<'_>],
    latest: usize,
    current: bool,
) -> ScenarioStep {
    let mut step = step.clone();
    let last = &awaits[latest];
    match &mut step {
        ScenarioStep::ExpectEvent {
            event,
            payload,
            shape,
        } if current => {
            let mut values: BTreeMap<String, ScenarioValue> = payload
                .iter()
                .map(|(field, value)| (field.clone(), ScenarioValue::literal(value.clone())))
                .collect();
            let fields = published(last.outcome, event);
            if rewrite_fields(&mut values, fields, last.identity, true) {
                return ScenarioStep::ExpectEventValues {
                    event: event.clone(),
                    payload: values,
                    shape: std::mem::take(shape),
                };
            }
            payload.retain(|field, _| values.contains_key(field));
        }
        ScenarioStep::ExpectEventValues { event, payload, .. } if current => {
            let fields = published(last.outcome, event);
            rewrite_fields(payload, fields, last.identity, true);
        }
        ScenarioStep::ExpectError { fields, .. } if current => {
            let mut values: BTreeMap<String, ScenarioValue> = fields
                .iter()
                .map(|(field, value)| (field.clone(), ScenarioValue::literal(value.clone())))
                .collect();
            let declared = last
                .outcome
                .map_or(&[][..], |outcome| &outcome.error_payload[..]);
            rewrite_fields(&mut values, declared, last.identity, false);
            fields.retain(|field, _| values.contains_key(field));
        }
        ScenarioStep::ExpectView { view, expectation }
        | ScenarioStep::EventuallyView {
            view, expectation, ..
        } => {
            let source = ir.views().get(view.name()).map(|view| view.source.name());
            if let Some(rows) = rows(expectation) {
                for (index, awaited) in awaits.iter().enumerate() {
                    read_rows(ir, rows, source, awaited, index == latest);
                }
            }
        }
        ScenarioStep::ExpectInvocation { input, .. } => {
            relay(input, last);
        }
        ScenarioStep::ExpectEveryInvocation {
            selecting, input, ..
        } => {
            relay(selecting, last);
            relay(input, last);
        }
        _ => {}
    }
    step
}

/// The fields the reached branch declares for `event`.
fn published<'ir>(
    outcome: Option<&'ir ResolvedOutcome>,
    event: &EventRef,
) -> &'ir [ResolvedPayloadField] {
    outcome
        .and_then(|outcome| {
            outcome
                .payload
                .iter()
                .find(|payload| EventRef::from(&payload.event) == *event)
        })
        .map_or(&[][..], |payload| &payload.fields[..])
}

/// Rewrites each expected value `declared` gives a source the system decides: to
/// [`ScenarioValue::Awaited`] where `awaitable` and the source is a top-level input, and out of
/// the comparison otherwise. Only a value the expectation states is touched. Whether one now reads
/// an awaited input.
fn rewrite_fields(
    values: &mut BTreeMap<String, ScenarioValue>,
    declared: &[ResolvedPayloadField],
    identity: &str,
    awaitable: bool,
) -> bool {
    let mut reads = false;
    for field in declared {
        match source(&field.value, identity) {
            Source::Kept => {}
            Source::Awaited(input) if awaitable && values.contains_key(&field.target) => {
                drop_under(values, &field.target);
                values.insert(
                    field.target.clone(),
                    ScenarioValue::Awaited { field: input },
                );
                reads = true;
            }
            Source::Awaited(_) | Source::Uncompared => {
                values.remove(&field.target);
                drop_under(values, &field.target);
            }
        }
    }
    reads
}

/// Removes every dotted leaf under `field`.
fn drop_under<V>(values: &mut BTreeMap<String, V>, field: &str) {
    let prefix = format!("{field}.");
    values.retain(|key, _| !key.starts_with(&prefix));
}

/// The rows a view expectation states field by field.
fn rows(expectation: &mut ViewExpectation) -> Option<&mut BTreeMap<String, ScenarioValue>> {
    match expectation {
        ViewExpectation::Contains { fields }
        | ViewExpectation::Excludes { fields }
        | ViewExpectation::At { fields, .. } => Some(fields),
        _ => None,
    }
}

/// A row of the awaited instance, read back after its await: each stored field the reached branch
/// writes from an input the system decides reads the awaited input, where it is the most recent
/// await's, and is left uncompared otherwise.
fn read_rows(
    ir: &EssIr,
    row: &mut BTreeMap<String, ScenarioValue>,
    source: Option<&QualifiedName>,
    awaited: &Awaited<'_>,
    latest: bool,
) {
    let Some(outcome) = awaited.outcome else {
        return;
    };
    let Some(subject) = &outcome.subject else {
        return;
    };
    if source != Some(subject.entity.name()) {
        return;
    }
    let identity = &ir.entity(&subject.entity).identity.name;
    if row.get(identity) != Some(&awaited.instance) {
        return;
    }
    rewrite_fields(row, &outcome.sets, awaited.identity, latest);
}

/// A value an expected invocation carries that is exactly what synthesis sent the most recent
/// awaited command for an input the system decides: relayed from that input, so it reads the
/// awaited one.
fn relay(values: &mut BTreeMap<String, ScenarioValue>, last: &Awaited<'_>) {
    for value in values.values_mut() {
        if let Some((field, _)) = last
            .invented
            .iter()
            .find(|(_, invented)| *invented == value)
        {
            *value = ScenarioValue::Awaited {
                field: field.clone(),
            };
        }
    }
}

/// Takes out of the scenario's sources every system actor no step sends as any longer: it awaits
/// them instead.
fn forget_unsent_actors(ir: &EssIr, scenario: &mut ConformanceScenario) {
    let still_sent: std::collections::BTreeSet<ActorRef> = scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand { actor, .. }
            | ScenarioStep::ExecuteCommandWithoutInput { actor, .. } => actor.clone(),
            _ => None,
        })
        .collect();
    for actor in ir.actors().values().filter(|actor| actor.system) {
        let actor = ActorRef::new(actor.name.clone());
        if !still_sent.contains(&actor) {
            scenario.source.remove(&EssSemanticRef::from(actor));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ess_primitives::node::Node;

    #[test]
    fn a_rendered_guard_reads_the_input_by_its_namespace() {
        assert!(mentions_input("input.worker == row.worker_id"));
        assert!(!mentions_input("free == true"));
    }

    #[test]
    fn a_leaf_under_a_dropped_field_goes_with_it() {
        let mut values: BTreeMap<String, Node> = BTreeMap::from([
            ("worker".to_owned(), Node::Null),
            ("worker.name".to_owned(), Node::Null),
            ("workers".to_owned(), Node::Null),
        ]);
        drop_under(&mut values, "worker");
        assert_eq!(
            values.keys().map(String::as_str).collect::<Vec<_>>(),
            vec!["worker", "workers"]
        );
    }
}
