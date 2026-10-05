//! Security review of beyond10x/ess#286: what the read-grant scenarios measure, checked against
//! the contract `docs/design/view-grants.md` states.
//!
//! The design says a read-granted view is refused to an actor the grant does not name *and* to a
//! request authenticated as no actor ("No caller is refused too"), and that the Rust, Go and
//! TypeScript runners give the same verdict. These cases drive the synthesized suite with targets
//! that break one half of that contract each and assert the suite notices, the same way.

mod support_go;

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::scenario::ActorRef;
use ess_conformance::target::{
    ConformanceTarget, EventObservationRequest, ExternalOutcomeControl, ImplementationIdentity,
    ObservedEvent, RedeliveryRequest, ScenarioContext, SemanticCommandRequest,
    SemanticCommandResult, SemanticViewRequest, SemanticViewResult, TargetError,
};
use ess_conformance::{AdmittedSuite, Runner};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const MODEL: &str = include_str!("../../../../docs/design/view-grants.example.yaml");
const BOARD: &str = "desk.tickets.Board";
const ADMITTED: &str = "desk.tickets.Board/grant/read/admitted/desk.tickets.Clerk";

fn ir() -> EssIr {
    let raw = RawSpecFile::parse(MODEL).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("view-grants.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

/// How a probe target answers a read of the read-granted `desk.tickets.Board`.
#[derive(Clone, Copy)]
enum Board {
    /// Refuses a named actor the grant does not name, as the interpreter does, and serves a read
    /// sent as no actor: a surface whose `admit_read` maps `None` to `Ok(())`.
    ServesNoActor,
    /// Refuses every read sent as an actor, granted or not: a surface whose read grants dropped
    /// the Clerk.
    RefusesEveryReader,
}

/// The interpreter, with one half of the Board's read grant broken as `Board` says.
struct Probe(Interpreted, Board);

impl ConformanceTarget for Probe {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("view-grant-probe", "1"))
    }
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.0.begin_scenario(scenario)
    }
    fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.0.end_scenario(scenario)
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        self.0.execute_command(request)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        // A read sent as no actor. The contract refuses it on the Board; `ServesNoActor` serves it.
        self.0.query_view(request)
    }
    fn query_view_as(
        &self,
        request: SemanticViewRequest,
        reader: &ActorRef,
    ) -> Result<SemanticViewResult, TargetError> {
        match self.1 {
            Board::RefusesEveryReader if request.view.to_string() == BOARD => {
                Err(TargetError::not_granted(Some(reader.to_string())))
            }
            _ => self.0.query_view_as(request, reader),
        }
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        self.0.observe_events(request)
    }
    fn configure_external_outcome(
        &self,
        request: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        self.0.configure_external_outcome(request)
    }
    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        self.0.redeliver_event(request)
    }
}

/// Invariant 2, over conformance: a target that serves the read-granted Board to a request sent as
/// no actor breaks the contract the design states ("No caller is refused too"), so some scenario
/// of the synthesized suite has to fail it.
#[test]
fn a_target_serving_a_read_granted_view_to_no_actor_fails_some_scenario() {
    let ir = ir();
    let suite = ess_conformance::synthesize(&ir).suite;
    let admitted = AdmittedSuite::from_suite(&suite).expect("admits");
    let report = Runner::for_suite(&suite).run_admitted(
        &admitted,
        &Probe(Interpreted::for_model(ir), Board::ServesNoActor),
    );
    let not_passed: Vec<String> = report
        .scenarios
        .iter()
        .filter(|scenario| scenario.status != Status::Passed)
        .map(|scenario| scenario.scenario.to_string())
        .collect();
    assert_ne!(
        not_passed.len(),
        0,
        "a target serving `{BOARD}` to a request authenticated as no actor passes all {} \
         synthesized scenarios: nothing in the suite reads a read-granted view as no actor",
        report.scenarios.len()
    );
}

/// Invariant 5: the admitted scenario fails a target that refuses the granted reader, and the Go
/// runtime gives the Rust reference's verdict for it.
#[test]
fn a_target_refusing_the_granted_reader_fails_the_admitted_read_alike_in_rust_and_go() {
    let suite = ess_conformance::synthesize(&ir()).suite;
    // Go (left) against Rust (right), per scenario; panics naming both on a divergence.
    let verdicts = support_go::assert_parity(
        "view-grant-refuses-reader",
        &suite,
        Probe(Interpreted::for_model(ir()), Board::RefusesEveryReader),
    );
    assert_eq!(
        verdicts.get(ADMITTED).map(String::as_str),
        Some("failed"),
        "the verdict on `{ADMITTED}` against a target refusing the Clerk: {verdicts:?}"
    );
}
