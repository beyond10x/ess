//! Awaiting a command the system issues itself (beyond10x/ess#542,
//! `docs/design/system-issued-commands.md`, *Meaning*).
//!
//! The runner never invokes a system-issued command. [`AwaitOutcome`](ScenarioStep::AwaitOutcome)
//! asks the target for the outcome its system reached on an instance, and
//! [`ExpectNotIssued`](ScenarioStep::ExpectNotIssued) requires that none was reached. Both are
//! checked under `ESS-CF-AWAIT`:
//!
//! | answer | `await_outcome` | `expect_not_issued` |
//! |---|---|---|
//! | reached, reading within the bound | passes; the answer becomes the current command result | fails |
//! | reached, reading above the bound, or no reading | fails, and the answer is still the current result | fails |
//! | nothing, reading at least the bound | fails | passes |
//! | nothing, reading short of the bound, or no reading | fails | fails |
//! | nothing, no declared bound | asked again until the runner's deadline, then fails | asked again until the runner's deadline, then passes |
//!
//! A reached answer is also held to the scenario: its identity input is the awaited instance, each
//! stated input value is the one carried, and its input holds each declared field at its declared
//! type (`ESS-CF-PAYLOAD`).

use std::collections::BTreeMap;

use ess_domain::binding::periodic::Period;
use ess_primitives::node::Node;

use super::{
    nothing_published_after, quote, reach_into, target_failure, CheckCode, CheckResult, Clock,
    CommandRef, Deadline, Diagnostic, Elapsed, EventRef, Executed, Flow, PayloadShape, Reached,
    Run, Runner, ScenarioValue,
};
use crate::target::{AwaitAnswer, AwaitRequest, ConformanceTarget};

/// One [`AwaitOutcome`](crate::scenario::ScenarioStep::AwaitOutcome), as the runner reads it.
pub(super) struct Awaited<'a> {
    pub command: &'a CommandRef,
    pub instance: &'a ScenarioValue,
    pub identity: &'a str,
    pub within: Option<Period>,
    pub input: &'a BTreeMap<String, ScenarioValue>,
    pub shape: &'a PayloadShape,
}

/// One [`ExpectNotIssued`](crate::scenario::ScenarioStep::ExpectNotIssued), as the runner reads it.
pub(super) struct NotIssued<'a> {
    pub command: &'a CommandRef,
    pub instance: &'a ScenarioValue,
    pub within: Option<Period>,
    pub unpublished: &'a [EventRef],
}

fn bound(within: Option<Period>) -> Option<Elapsed> {
    within.map(|period| Elapsed::seconds(period.seconds()))
}

/// Resolves a value the step names, or records the suite defect and stops.
fn resolved(
    run: &mut Run,
    command: &CommandRef,
    what: &str,
    value: &ScenarioValue,
) -> Option<Node> {
    match run.resolve(value) {
        Ok(node) => Some(node),
        Err(reason) => {
            run.record(CheckResult::errored(
                format!("the {what} awaited of `{command}`"),
                Diagnostic::new(CheckCode::Suite, run.id.clone())
                    .declared_by(command.clone())
                    .expected(format!(
                        "the {what} refers to something an earlier step established"
                    ))
                    .observed(reason),
            ));
            None
        }
    }
}

impl<C: Clock> Runner<C> {
    /// Awaits an outcome of a system-issued command on its instance.
    pub(super) fn await_outcome<T: ConformanceTarget>(
        &mut self,
        step: &Awaited<'_>,
        run: &mut Run,
        target: &T,
    ) -> Flow {
        let command = step.command;
        let Some(instance) = resolved(run, command, "instance", step.instance) else {
            return Flow::Stop;
        };
        let mut stated = BTreeMap::new();
        for (field, value) in step.input {
            let Some(node) = resolved(run, command, &format!("input `{field}`"), value) else {
                return Flow::Stop;
            };
            stated.insert(field.clone(), node);
        }
        let within = bound(step.within);
        let deadline = self.deadline();
        let about = format!("{command} awaited on {}", quote(&instance));
        let mut asks = 0_u32;
        loop {
            asks += 1;
            let request = AwaitRequest {
                command: command.clone(),
                instance: instance.clone(),
                within,
                deadline,
                compared: stated.keys().cloned().collect(),
                correlation: run.context.correlation.clone(),
            };
            match target.await_outcome(request) {
                Err(error) => {
                    run.record(target_failure(
                        &run.id,
                        &format!("awaiting `{command}`"),
                        &error,
                    ));
                    return Flow::Stop;
                }
                Ok(AwaitAnswer::NotIssued { .. })
                    if within.is_none() && !deadline.has_passed(self.clock.now()) =>
                {
                    // Not believed before the deadline: a target may answer before it has held
                    // anything, and the step is about what the system does, not what it said first.
                }
                Ok(AwaitAnswer::NotIssued { elapsed_ms }) => {
                    let observed = match (within, elapsed_ms) {
                        (Some(within), Some(reading)) if reading < within.millis() => format!(
                            "nothing reached, answered after {reading} ms, short of the {within} \
                             bound"
                        ),
                        (Some(within), Some(reading)) => format!(
                            "nothing reached within {within}; the target's clock read {reading} ms"
                        ),
                        (Some(within), None) => format!(
                            "nothing reached, with no reading of the target's clock against the \
                             {within} bound"
                        ),
                        (None, _) => format!(
                            "nothing reached by the runner's deadline, after {asks} await(s)"
                        ),
                    };
                    run.record(CheckResult::failed(
                        about,
                        Diagnostic::new(CheckCode::Await, run.id.clone())
                            .declared_by(command.clone())
                            .expected(expectation(command, &instance, within))
                            .observed(observed),
                    ));
                    return Flow::Stop;
                }
                Ok(AwaitAnswer::Reached {
                    result,
                    input,
                    elapsed_ms,
                }) => {
                    let reading = (within, elapsed_ms);
                    reached(step, (&instance, &stated), reading, (result, input), run);
                    return Flow::Continue;
                }
            }
        }
    }

    /// Requires that a system-issued command reaches no outcome on its instance within the bound,
    /// and publishes none of its events.
    pub(super) fn expect_not_issued<T: ConformanceTarget>(
        &mut self,
        step: &NotIssued<'_>,
        run: &mut Run,
        target: &T,
    ) -> Flow {
        let command = step.command;
        let Some(instance) = resolved(run, command, "instance", step.instance) else {
            return Flow::Stop;
        };
        let now = self.clock.now();
        if super::count_before(step.unpublished, Deadline::at(now), run, target) == Flow::Stop {
            return Flow::Stop;
        }
        let within = bound(step.within);
        let deadline = self.deadline();
        let about = format!("{command} not issued on {}", quote(&instance));
        let expected = format!(
            "no outcome of {command} on {} {}",
            quote(&instance),
            within.map_or_else(
                || "by the runner's deadline".to_owned(),
                |within| format!("within {within}")
            )
        );
        loop {
            let request = AwaitRequest {
                command: command.clone(),
                instance: instance.clone(),
                within,
                deadline,
                compared: std::collections::BTreeSet::new(),
                correlation: run.context.correlation.clone(),
            };
            let failure = match target.await_outcome(request) {
                Err(error) => {
                    run.record(target_failure(
                        &run.id,
                        &format!("awaiting `{command}`"),
                        &error,
                    ));
                    return Flow::Stop;
                }
                Ok(AwaitAnswer::Reached { result, input, .. }) => {
                    run.remember(&result.direct_events);
                    Some((
                        result.outcome.as_ref().map_or_else(
                            || "the system issued it and reached no declared outcome".to_owned(),
                            |reached| {
                                format!("the system issued it: outcome = {}", reached.outcome)
                            },
                        ),
                        Some(super::quote_input(&command.to_string(), &input)),
                    ))
                }
                Ok(AwaitAnswer::NotIssued { .. }) if within.is_none() => {
                    if deadline.has_passed(self.clock.now()) {
                        None
                    } else {
                        continue;
                    }
                }
                Ok(AwaitAnswer::NotIssued { elapsed_ms }) => {
                    let within = within.unwrap_or_else(|| Elapsed::seconds(0));
                    match elapsed_ms {
                        Some(reading) if reading >= within.millis() => None,
                        Some(reading) => Some((
                            format!(
                                "nothing reached, answered after {reading} ms, before the \
                                 {within} window closed"
                            ),
                            None,
                        )),
                        None => Some((
                            format!(
                                "nothing reached, with no reading of the target's clock against \
                                 the {within} bound"
                            ),
                            None,
                        )),
                    }
                }
            };
            match failure {
                None => run.record(CheckResult::passed(CheckCode::Await, about)),
                Some((observed, executing)) => {
                    let mut diagnostic = Diagnostic::new(CheckCode::Await, run.id.clone())
                        .declared_by(command.clone())
                        .expected(expected)
                        .observed(observed);
                    if let Some(executing) = executing {
                        diagnostic = diagnostic.executing(executing);
                    }
                    run.record(CheckResult::failed(about, diagnostic));
                }
            }
            let now = self.clock.now();
            return nothing_published_after(step.unpublished, Deadline::at(now), run, target);
        }
    }
}

/// Holds a reached answer to the step, and makes it the scenario's current command result.
fn reached(
    step: &Awaited<'_>,
    (instance, stated): (&Node, &BTreeMap<String, Node>),
    (within, elapsed_ms): (Option<Elapsed>, Option<u64>),
    (result, input): (crate::target::SemanticCommandResult, BTreeMap<String, Node>),
    run: &mut Run,
) {
    let command = step.command;
    let about = format!("{command} awaited on {}", quote(instance));
    let mut wrong = Vec::new();
    if let Some(within) = within {
        match elapsed_ms {
            None => wrong.push(format!(
                "no reading of the target's clock against the {within} bound"
            )),
            Some(reading) if reading > within.millis() => {
                wrong.push(format!("reached after {reading} ms, later than {within}"));
            }
            Some(_) => {}
        }
    }
    match input.get(step.identity) {
        Some(carried) if carried == instance => {}
        Some(carried) => wrong.push(format!(
            "the answer is about {} = {}, not the awaited instance",
            step.identity,
            quote(carried)
        )),
        None => wrong.push(format!(
            "the answer carries no `{}`, the input naming the instance",
            step.identity
        )),
    }
    for (field, value) in stated {
        if input.get(field) != Some(value) {
            wrong.push(match input.get(field) {
                Some(carried) => format!(
                    "{field} = {}, where the scenario states {}",
                    quote(carried),
                    quote(value)
                ),
                None => format!(
                    "{field} was not carried, where the scenario states {}",
                    quote(value)
                ),
            });
        }
    }
    let quoted = super::quote_input(&command.to_string(), &input);
    if wrong.is_empty() {
        run.record(CheckResult::passed(CheckCode::Await, about));
    } else {
        run.record(CheckResult::failed(
            about,
            Diagnostic::new(CheckCode::Await, run.id.clone())
                .declared_by(command.clone())
                .executing(quoted.clone())
                .expected(expectation(command, instance, within))
                .observed(wrong.join("; ")),
        ));
    }
    answer_shape(command, &input, step.shape, &quoted, run);
    run.remember(&result.direct_events);
    run.awaited = Some(input.clone());
    run.executed(Executed {
        command: command.to_string(),
        actor: None,
        input,
        result,
        not_granted: None,
    });
}

fn expectation(command: &CommandRef, instance: &Node, within: Option<Elapsed>) -> String {
    match within {
        Some(within) => format!(
            "an outcome of {command} on {} within {within}",
            quote(instance)
        ),
        None => format!(
            "an outcome of {command} on {} by the runner's deadline",
            quote(instance)
        ),
    }
}

/// Holds the input an answer reports to the command's declared input (`ESS-CF-PAYLOAD`).
fn answer_shape(
    command: &CommandRef,
    input: &BTreeMap<String, Node>,
    shape: &PayloadShape,
    quoted: &str,
    run: &mut Run,
) {
    if shape.is_empty() {
        return;
    }
    let about = format!("input of {command}, as the system issued it");
    let mut diagnostic = Diagnostic::new(CheckCode::Payload, run.id.clone())
        .declared_by(command.clone())
        .executing(quoted.to_owned());
    let mut wrong = Vec::new();
    for (path, leaf) in shape.leaves() {
        let reached = reach_into(input, path);
        let admitted = match &reached {
            Reached::Value(value) => leaf.admits(Some(value)),
            Reached::Absent => leaf.admits(None),
            Reached::Blocked { .. } => false,
        };
        if admitted {
            continue;
        }
        diagnostic = diagnostic.expected(format!("{command}.{path} holds {}", leaf.holds));
        wrong.push(match reached {
            Reached::Value(value) => format!("{path} = {}", quote(value)),
            Reached::Absent => format!("{path} was not carried"),
            Reached::Blocked { at, found } => {
                format!("{at} holds {found}, so {path} is not there to read")
            }
        });
    }
    if wrong.is_empty() {
        run.record(CheckResult::passed(CheckCode::Payload, about));
    } else {
        run.record(CheckResult::failed(
            about,
            diagnostic.observed(wrong.join("; ")),
        ));
    }
}
