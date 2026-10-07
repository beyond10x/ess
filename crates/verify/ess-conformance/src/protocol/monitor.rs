use super::{
    enabled_actions, initial, successors, Action, Trace, TraceOrigin, TraceStep, TRACE_FORMAT,
};
use ess_compiler::protocol::CompiledProtocol;
use serde::Serialize;
use std::collections::VecDeque;

/// Evidence classification; incomplete evidence never passes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    /// All observed steps satisfy the model and no modeled obligations remain.
    Passed,
    /// Observations contradict the model or a declared safety property.
    Failed,
    /// Observation or exploration bounds prevent a conclusion.
    Inconclusive,
}
/// Result of checking one finite trace.
#[derive(Debug, Clone, Serialize)]
pub struct Report {
    /// Observation origin, when a trace was checked; this is an assertion, not attestation.
    pub origin: Option<TraceOrigin>,
    /// Evidence classification.
    pub verdict: Verdict,
    /// Concrete explanation of the classification.
    pub reason: String,
    /// Number of trace steps examined.
    pub checked_steps: usize,
    /// Number of still consistent model configurations.
    pub surviving_states: usize,
}
impl Report {
    pub(super) fn new(
        verdict: Verdict,
        reason: impl Into<String>,
        checked_steps: usize,
        surviving_states: usize,
    ) -> Self {
        Self {
            origin: None,
            verdict,
            reason: reason.into(),
            checked_steps,
            surviving_states,
        }
    }
}
/// Replay exact observations, retaining all consistent nondeterministic alternatives.
pub fn check_trace(model: &CompiledProtocol, trace: &Trace) -> Report {
    let mut report = check(model, trace);
    report.origin = Some(trace.origin.clone());
    report
}

fn check(model: &CompiledProtocol, trace: &Trace) -> Report {
    if matches!(&trace.origin, TraceOrigin::Target { implementation } if implementation.trim().is_empty())
    {
        return Report::new(
            Verdict::Failed,
            "target trace has no implementation identity",
            0,
            0,
        );
    }
    if trace.format != TRACE_FORMAT || trace.model_digest != model.digest() {
        return Report::new(
            Verdict::Failed,
            "trace format or model digest mismatch",
            0,
            0,
        );
    }
    if trace.steps.is_empty() {
        return Report::new(
            Verdict::Inconclusive,
            "empty trace supplies no behavioral evidence",
            0,
            1,
        );
    }
    let mut states = vec![initial(model)];
    for (index, recorded) in trace.steps.iter().enumerate() {
        let mut next = vec![];
        let mut bounded = false;
        for state in &states {
            match successors(model, state, &recorded.action) {
                Ok(steps) => {
                    for step in steps {
                        if step.observations == recorded.observations
                            && step.configuration.violations.is_empty()
                            && !next.contains(&step.configuration)
                        {
                            next.push(step.configuration);
                            if next.len() > model.model().bounds.max_states {
                                bounded = true;
                                break;
                            }
                        }
                    }
                }
                Err(error) => bounded |= error.bound,
            }
            if bounded {
                break;
            }
        }
        if bounded {
            return Report::new(
                Verdict::Inconclusive,
                "execution or state bound exhausted",
                index + 1,
                next.len(),
            );
        }
        if next.is_empty() {
            return Report::new(
                Verdict::Failed,
                "observations contradict every permitted transition or property",
                index + 1,
                0,
            );
        }
        states = next;
    }
    if !trace.complete || states.iter().any(|state| !state.settled()) {
        Report::new(
            Verdict::Inconclusive,
            "capture is incomplete or modeled obligations remain",
            trace.steps.len(),
            states.len(),
        )
    } else {
        Report::new(
            Verdict::Passed,
            "finite trace satisfies the model at the declared observation boundary",
            trace.steps.len(),
            states.len(),
        )
    }
}
/// Bounded model exploration, never an implementation conformance claim.
#[derive(Debug, Clone, Serialize)]
pub struct ExplorationReport {
    /// Finite scheduling profile; arbitrary intermediate-time inputs are not enumerated.
    pub time_policy: &'static str,
    /// Verdict for the finite explored model and supplied input witnesses.
    pub verdict: Verdict,
    /// Reason and scope of the result.
    pub reason: String,
    /// Unique configurations examined.
    pub explored_states: usize,
    /// Enabled transition results examined.
    pub explored_transitions: usize,
    /// Exact limits under which exploration ran.
    pub bounds: ess_domain::protocol::Bounds,
    /// Finite input witnesses supplied by the caller.
    pub inputs: Vec<Action>,
    /// Replayable property violation, when found.
    pub counterexample: Option<Trace>,
}
/// Breadth-first exploration in stable scheduling order, including only finite input witnesses.
pub fn explore(model: &CompiledProtocol, inputs: &[Action]) -> ExplorationReport {
    let mut report = ExplorationReport {
        time_policy: "inputs_at_initial_time_and_timer_deadlines",
        verdict: Verdict::Passed,
        reason: "bounded model search completed for the supplied finite input witnesses; not implementation evidence".into(),
        explored_states: 0,
        explored_transitions: 0,
        bounds: model.model().bounds.clone(),
        inputs: inputs.to_vec(),
        counterexample: None,
    };
    if !inputs.iter().all(|action| valid_input(model, action)) {
        report.verdict = Verdict::Inconclusive;
        report.reason = "exploration witnesses must match declared participant input shapes".into();
        return report;
    }
    let start = initial(model);
    let mut queue = VecDeque::from([(start.clone(), Vec::new())]);
    let mut seen = vec![start];
    while let Some((state, trace)) = queue.pop_front() {
        report.explored_states += 1;
        if !state.violations.is_empty() {
            report.verdict = Verdict::Failed;
            report.reason = format!("property violated: {:?}", state.violations);
            report.counterexample = Some(Trace {
                format: TRACE_FORMAT.into(),
                model_digest: model.digest().into(),
                origin: TraceOrigin::Model,
                steps: trace,
                complete: state.settled(),
            });
            return report;
        }
        let actions = enabled_actions(model, &state, inputs);
        if actions.is_empty() && !state.settled() {
            report.verdict = Verdict::Inconclusive;
            report.reason = "dead end with outstanding queue, timer or flush obligations".into();
        }
        for action in actions {
            let steps = match successors(model, &state, &action) {
                Ok(steps) => steps,
                Err(error) => {
                    report.verdict = Verdict::Inconclusive;
                    report.reason = error.message;
                    continue;
                }
            };
            for step in steps {
                report.explored_transitions += 1;
                let mut path = trace.clone();
                path.push(TraceStep {
                    action: action.clone(),
                    observations: step.observations,
                });
                if !step.configuration.violations.is_empty() {
                    report.verdict = Verdict::Failed;
                    report.reason =
                        format!("property violated: {:?}", step.configuration.violations);
                    report.counterexample = Some(Trace {
                        format: TRACE_FORMAT.into(),
                        model_digest: model.digest().into(),
                        origin: TraceOrigin::Model,
                        steps: path,
                        complete: step.configuration.settled(),
                    });
                    return report;
                }
                if !seen.contains(&step.configuration) {
                    if seen.len() >= model.model().bounds.max_states {
                        report.verdict = Verdict::Inconclusive;
                        report.reason = "state bound exhausted".into();
                        return report;
                    }
                    seen.push(step.configuration.clone());
                    queue.push_back((step.configuration, path));
                }
            }
        }
    }
    if report.explored_transitions == 0 {
        report.verdict = Verdict::Inconclusive;
        report.reason = "no transitions exercised".into();
    }
    report
}

fn valid_input(model: &CompiledProtocol, action: &Action) -> bool {
    let Action::Input {
        participant,
        name,
        payload,
    } = action
    else {
        return false;
    };
    let declaration = model
        .model()
        .participants
        .iter()
        .find(|peer| peer.name == *participant)
        .and_then(|peer| peer.inputs.iter().find(|input| input.name == *name));
    declaration.is_some_and(|input| {
        input.fields.len() == payload.len()
            && input.fields.iter().all(|field| {
                payload
                    .get(&field.name)
                    .is_some_and(|value| ess_domain::protocol::admits(&field.type_ref, value))
            })
    })
}
