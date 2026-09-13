//! The specification itself, selected as the implementation under test.
//!
//! `examples/billing` and `examples/oracle-fixture` each have a committed suite and a hand-written
//! target, and both of those targets decided their command behaviour by hand. An interpreter runs
//! the same suite against the document the suite was synthesized from, which makes a specification
//! falsifiable before anybody has implemented it — see
//! `docs/design/ess-model-driven-interpretation-design-v0.1.md`.
//!
//! # What is here now: the seam, and nothing behind it
//!
//! [`Interpreted`] is the target an operator can select. It derives **no** behaviour: it executes no
//! command, projects no view, publishes no event and holds no state. Every scenario run against it
//! comes back [`Status::Unsupported`](crate::report::Status::Unsupported) — §28's fourth word, and
//! the only honest answer for a target that has been selected and has decided nothing.
//!
//! **`Unsupported` rather than `Error`, and that distinction is the whole of what this module
//! claims.** `report.rs` writes it down: `Failed` says the implementation contradicted the
//! specification, `Error` says nobody found out. Nothing here went wrong — no run failed, no adapter
//! broke, no observation was lost — so an `Error` would misreport a capability this target does not
//! yet have as an execution that did not happen, and an implementer reading the report would go
//! looking for a fault that is not there. `Unsupported` says exactly what is true: the obligation
//! stands, unsatisfied, and the target cannot expose what would satisfy it.
//!
//! It is equally not a skip. §28 makes an unsupported scenario fail conformance, so selecting this
//! target cannot turn a scenario green: a run over a suite holding **at least one** scenario comes
//! back `failed` and exits non-zero, and will keep doing so until interpretation actually decides
//! something.
//!
//! A suite holding **no** scenarios is the one exception, and it is not this target's.
//! [`ConformanceReport::verdict`](crate::report::ConformanceReport::verdict) reads a run's verdict
//! off its scenarios, and the verdict of none of them is
//! [`Passed`](crate::report::ConformanceStatus::Passed) — so an empty admitted suite reports
//! `passed` and exits 0 against `billing` and `oracle-fixture` exactly as it does here. That is a
//! property of the runner and of what a vacuous suite means, not a capability this target has, and
//! `ess verify conform select --ids` documents `[]` as a selection somebody can actually ask for.
//!
//! # Why the refusal is at [`begin_scenario`](ConformanceTarget::begin_scenario)
//!
//! Because that is where the claim is true rather than convenient. An interpreter's isolated context
//! (§8) is the model state it would interpret against; with no interpretation there is no such
//! state, and there is nothing to open. Refusing there also makes the report's shape a property of
//! the target rather than of the suite: the runner records one unsupported check and runs no step,
//! so **every scenario of every suite** — not only the two committed ones — comes back as an
//! unsatisfied obligation and never as an error, whatever steps it happens to hold. It says nothing
//! about a suite holding no scenarios, which reaches no method of this target at all.
//!
//! The six methods after it refuse in the same terms anyway. They are unreachable while
//! `begin_scenario` refuses, and they are written out rather than left to a default body so that no
//! method of this target can be read as agreeing with a claim it never checked — which is the rule
//! the whole of [`target`](crate::target) is built on. [`identity`](ConformanceTarget::identity) is
//! the one that answers: §30 requires a report to name the implementation that answered.

use crate::target::{
    ConformanceTarget, EventObservationRequest, ExternalOutcomeControl, ImplementationIdentity,
    ObservedEvent, RedeliveryRequest, ScenarioContext, SemanticCommandRequest,
    SemanticCommandResult, SemanticViewRequest, SemanticViewResult, TargetError,
};

/// How a report names this target, and the value `--target` selects it by.
///
/// One string for both, so a reader holding a report can get back to the command that produced it.
/// That is this target's choice and not a convention of the family: `billing` reports
/// `billing-reference` and `oracle-fixture` reports `oracle-reference`, because those two name a
/// hand-written implementation while this one names nothing but the selection itself.
const IDENTITY: &str = "interpreted";

/// Why every observation this target is asked for is refused, in one sentence.
const NOTHING_DERIVED: &str =
    "the interpreted target derives no behaviour yet: it executes no part of the model";

/// The model, run as the implementation under test — a seam, with nothing behind it yet.
///
/// See the [module documentation](self) for what it refuses and why that is `unsupported` rather
/// than `error`.
#[derive(Debug, Default)]
pub struct Interpreted;

impl Interpreted {
    /// The target an operator selects with `--target interpreted`.
    pub fn new() -> Self {
        Self
    }
}

impl ConformanceTarget for Interpreted {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        // Answered rather than refused: §30 requires a report to name the implementation that
        // answered, and this one did answer — with nothing, which the scenarios then say.
        Ok(ImplementationIdentity::new(
            IDENTITY,
            env!("CARGO_PKG_VERSION"),
        ))
    }

    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        Err(TargetError::unsupported(
            format!("an isolated execution context for `{}`", scenario.scenario),
            NOTHING_DERIVED,
        ))
    }

    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        Err(TargetError::unsupported(
            format!("invoking `{}`", request.command),
            NOTHING_DERIVED,
        ))
    }

    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        Err(TargetError::unsupported(
            format!("reading `{}`", request.view),
            NOTHING_DERIVED,
        ))
    }

    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Err(TargetError::unsupported(
            format!("the occurrences of `{}`", request.event),
            NOTHING_DERIVED,
        ))
    }

    fn configure_external_outcome(
        &self,
        request: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        Err(TargetError::unsupported(
            format!("forcing `{}`", request.force),
            NOTHING_DERIVED,
        ))
    }

    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        Err(TargetError::unsupported(
            format!("delivering `{}` again", request.event),
            NOTHING_DERIVED,
        ))
    }

    fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        Err(TargetError::unsupported(
            format!("closing the execution context of `{}`", scenario.scenario),
            NOTHING_DERIVED,
        ))
    }
}
