use super::{
    check_trace, Action, Observation, Report, Trace, TraceOrigin, TraceStep, Verdict, TRACE_FORMAT,
};
use ess_compiler::protocol::CompiledProtocol;
use serde::Serialize;

/// Adapter capabilities, checked before any drive operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Capabilities {
    /// Independent implementation identity, reported with captured evidence.
    pub implementation: String,
    /// Adapter can drive every protocol action, including the network and logical timers.
    pub all_actions: bool,
    /// Adapter observes the complete ordered facts at every requested boundary.
    pub complete_observations: bool,
}
/// A native implementation adapter; it supplies facts and never reference-model output.
pub trait ProtocolTarget {
    /// Read-only capability admission before opening a session.
    fn capabilities(&self) -> Capabilities;
    /// Open a new isolated protocol lifetime, binding the exact model digest.
    fn open(&mut self, model_digest: &str) -> Result<(), String>;
    /// Apply an action and observe actual ordered facts through its causal boundary.
    fn drive(&mut self, action: &Action) -> Result<Vec<Observation>, String>;
    /// Close the observation lifetime; true means capture completed without losing facts.
    /// This is adapter cleanup, not an injected protocol close action.
    fn close(&mut self) -> Result<bool, String>;
}
/// Actual implementation capture, distinct from a simulated model trace.
#[derive(Debug, Clone, Serialize)]
pub struct Capture {
    /// Independent implementation identity.
    pub implementation: String,
    /// Exact observation record.
    pub trace: Trace,
    /// Capture errors, if the observation boundary could not be completed.
    pub error: Option<String>,
}
/// Capture a bounded target run, always closing an opened observation lifetime.
pub fn capture_target(
    model: &CompiledProtocol,
    actions: &[Action],
    target: &mut impl ProtocolTarget,
) -> Capture {
    let capabilities = target.capabilities();
    let mut capture = Capture {
        implementation: capabilities.implementation.clone(),
        trace: Trace {
            format: TRACE_FORMAT.into(),
            model_digest: model.digest().into(),
            origin: TraceOrigin::Target {
                implementation: capabilities.implementation.clone(),
            },
            steps: vec![],
            complete: false,
        },
        error: None,
    };
    if capabilities.implementation.trim().is_empty()
        || !capabilities.all_actions
        || !capabilities.complete_observations
    {
        capture.error =
            Some("target lacks declared identity, action support or complete observations".into());
        return capture;
    }
    if actions.len() > model.model().bounds.max_steps {
        capture.error = Some("action sequence exceeds model step bound".into());
        return capture;
    }
    if let Err(error) = target.open(model.digest()) {
        capture.error = Some(error);
        return capture;
    }
    for action in actions {
        match target.drive(action) {
            Ok(observations) => capture.trace.steps.push(TraceStep {
                action: action.clone(),
                observations,
            }),
            Err(error) => {
                capture.error = Some(error);
                break;
            }
        }
    }
    match target.close() {
        Ok(complete) => capture.trace.complete = complete && capture.error.is_none(),
        Err(error) => capture.error = Some(error),
    }
    capture
}
/// Check actual captured behavior against the protocol, preserving observation errors.
pub fn run_target(
    model: &CompiledProtocol,
    actions: &[Action],
    target: &mut impl ProtocolTarget,
) -> Report {
    let capture = capture_target(model, actions, target);
    let mut report = check_trace(model, &capture.trace);
    if let Some(error) = capture.error {
        // A failed admission supplies no behavior to judge. An observed contradiction still
        // fails even if a later capture operation also failed. Preserve origin in both cases.
        if report.verdict != Verdict::Failed || capture.trace.steps.is_empty() {
            report.verdict = Verdict::Inconclusive;
            report.reason = error;
        }
    }
    report
}
