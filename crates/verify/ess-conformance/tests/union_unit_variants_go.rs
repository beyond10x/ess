//! Unit variants through the Go suite runtime (ess/22, beyond10x/ess#418): the Go runtime gives
//! the reference runner's verdict for every scenario over a recorded run, for a correct target and
//! for targets that write a unit variant with a payload member or confuse it with a payload
//! variant — an event payload carrying the union, and a response the union is declared in.

mod support_go;

use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::target::{
    ConformanceTarget, EventObservationRequest, ExternalOutcomeControl, ImplementationIdentity,
    ObservedEvent, RedeliveryRequest, ScenarioContext, SemanticCommandRequest,
    SemanticCommandResult, SemanticViewRequest, SemanticViewResult, TargetError,
};
use ess_conformance::ConformanceSuite;
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

const MODEL: &str =
    include_str!("../../../specify/ess-compiler/tests/fixtures/union-unit-variants.yaml");

/// A response union whose first variant — the one synthesis witnesses — is a unit variant.
const RESPONSE: &str = "format: ess/22
system: example
version: v1
domain: example.api
types:
  - name: example.api.Result
    kind: union
    tag: kind
    variants:
      absent:
      text: String
events:
  - name: example.api.Returned
    fields:
      - {name: item, type: example.api.Result}
commands:
  - name: example.api.Fetch
    response:
      - {name: item, type: example.api.Result}
    outcomes:
      - name: returned
        emits: [example.api.Returned]
        payload:
          example.api.Returned:
            item: {response: item}
";

const OPEN_SCENARIO: &str = "type: ess-scenario/1
domain: demo.work
scenario: open-is-reported
summary: An open status is reported as the unit variant it is.
timeline:
  - at: 2026-01-05T09:00:00Z
    command: demo.work.ReportStatus
    actor: demo.work.Reporter
    input:
      status: {kind: Open}
    outcome: reported
    events:
      - event: demo.work.StatusReported
        payload: {status: {kind: Open}}
";

const OPEN_ID: &str = "demo.work/authored/open-is-reported";

fn ir(text: &str) -> EssIr {
    let spec =
        Specification::assemble([(Source::new("model.yaml"), RawSpecFile::parse(text).unwrap())])
            .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

/// The synthesized suite, with the authored scenario that sends `Open` where the model is `MODEL`.
fn suite(text: &str) -> ConformanceSuite {
    let ir = ir(text);
    let synthesis = ess_conformance::synthesize(&ir);
    assert_eq!(synthesis.refusals.len(), 0, "{:?}", synthesis.refusals);
    let mut suite = synthesis.suite;
    if text == MODEL {
        let authoring = ess_conformance::authored::compile(
            &ir,
            &[ess_conformance::authored::Source::new(
                "open.yaml",
                OPEN_SCENARIO,
            )],
        );
        assert!(authoring.is_complete(), "{:?}", authoring.refusals);
        for (id, scenario) in authoring.scenarios {
            suite.insert(id, scenario).expect("one id");
        }
    }
    suite
}

/// How a faulty target spells the union it answers with.
#[derive(Clone, Copy, Debug)]
enum Fault {
    /// A unit variant written with a payload member.
    UnitWithPayload,
    /// The unit variant answered as the payload variant, and the payload variant as the unit one.
    Swapped,
}

/// A target right about everything except how it writes the union `field` holds.
struct Faulty {
    inner: Interpreted,
    fault: Fault,
    /// The unit variant, the payload variant, and a payload for the latter.
    variants: (&'static str, &'static str, Node),
}

impl Faulty {
    fn mangle(&self, value: &mut Node) {
        let Node::Map(union) = value else { return };
        let kind = union.get("kind").and_then(Node::as_text).map(str::to_owned);
        let (unit, payload, carried) = &self.variants;
        match (self.fault, kind.as_deref()) {
            // Whichever variant the target chose, it answers the unit variant with a payload member:
            // a response union is the implementation's choice, so the fault cannot wait for one.
            (Fault::UnitWithPayload, Some(_)) => {
                union.insert("kind".to_owned(), Node::Text((*unit).to_owned()));
                union.insert("value".to_owned(), carried.clone());
            }
            (Fault::Swapped, Some(label)) if label == *unit => {
                union.insert("kind".to_owned(), Node::Text((*payload).to_owned()));
                union.insert("value".to_owned(), carried.clone());
            }
            (Fault::Swapped, Some(label)) if label == *payload => {
                union.insert("kind".to_owned(), Node::Text((*unit).to_owned()));
                union.remove("value");
            }
            _ => {}
        }
    }

    fn mangle_all(&self, values: &mut BTreeMap<String, Node>) {
        for value in values.values_mut() {
            self.mangle(value);
        }
    }
}

impl ConformanceTarget for Faulty {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("faulty-union", "1"))
    }
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.begin_scenario(scenario)
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let mut result = self.inner.execute_command(request)?;
        for event in &mut result.direct_events {
            self.mangle_all(&mut event.payload);
        }
        if let Some(response) = &mut result.response {
            self.mangle_all(response);
        }
        Ok(result)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        self.inner.query_view(request)
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        let mut events = self.inner.observe_events(request)?;
        for event in &mut events {
            self.mangle_all(&mut event.payload);
        }
        Ok(events)
    }
    fn configure_external_outcome(
        &self,
        control: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        self.inner.configure_external_outcome(control)
    }
    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        self.inner.redeliver_event(request)
    }
    fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.end_scenario(scenario)
    }
}

fn completion() -> Node {
    Node::Map(BTreeMap::from([(
        "outcome".to_owned(),
        Node::Text("shipped".to_owned()),
    )]))
}

#[test]
fn go_gives_the_reference_verdicts_over_an_event_carrying_a_unit_variant() {
    let suite = suite(MODEL);
    let passed = support_go::assert_parity("unit-event", &suite, Interpreted::for_model(ir(MODEL)));
    assert!(passed.contains_key(OPEN_ID), "{passed:?}");
    assert_eq!(support_go::not_passed(&passed).len(), 0, "{passed:?}");
    for (label, fault) in [
        ("unit-event-payload", Fault::UnitWithPayload),
        ("unit-event-swapped", Fault::Swapped),
    ] {
        let verdicts = support_go::assert_parity(
            label,
            &suite,
            Faulty {
                inner: Interpreted::for_model(ir(MODEL)),
                fault,
                variants: ("Open", "Complete", completion()),
            },
        );
        assert_eq!(verdicts[OPEN_ID], "failed", "{label}: {verdicts:?}");
    }
}

#[test]
fn go_gives_the_reference_verdicts_over_a_response_declaring_a_unit_variant() {
    let suite = suite(RESPONSE);
    let passed = support_go::assert_parity(
        "unit-response",
        &suite,
        Interpreted::for_model(ir(RESPONSE)),
    );
    assert_eq!(support_go::not_passed(&passed).len(), 0, "{passed:?}");
    // A response value is the implementation's choice, so answering the other variant consistently
    // is not a fault there; writing the unit variant with a payload member is.
    let verdicts = support_go::assert_parity(
        "unit-response-payload",
        &suite,
        Faulty {
            inner: Interpreted::for_model(ir(RESPONSE)),
            fault: Fault::UnitWithPayload,
            variants: ("absent", "text", Node::Text("ok".to_owned())),
        },
    );
    assert_ne!(
        support_go::not_passed(&verdicts).len(),
        0,
        "a faulty response fails: {verdicts:?}"
    );
}
