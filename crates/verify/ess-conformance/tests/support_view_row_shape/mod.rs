//! The suite/66 fixture of story:a-view-read-checks-every-rows-shape (beyond10x/ess#680): a view
//! whose rows each carry `order_id`, `title` and a `placed_at` timestamp, its row shape written by
//! hand on every view step, and a target that answers the rows with one field dropped or of
//! another type. The Rust runner, the Go runtime and the TypeScript runtime read it alike.
#![allow(dead_code)]

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::scenario::{Holds, LeafShape, PayloadShape, SuiteFormat};
use ess_conformance::target::*;
use ess_conformance::{ConformanceSuite, ScenarioStep};
use ess_domain::types::Primitive;
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

/// The model the fixture is synthesized from.
pub const ORDERS: &str = "format: ess/37
system: example
version: v1
domain: example.order
summary: Orders placed with a title and an instant, read back through a view.
types:
  - {name: example.order.OrderId, kind: newtype, of: String}
entities:
  - name: example.order.Order
    identity: {name: order_id, type: example.order.OrderId}
    fields:
      - {name: title, type: String}
      - {name: placed_at, type: Timestamp}
    lifecycle: {initial: Placed, states: [Placed], terminal: [Placed]}
actors:
  - {name: example.order.Buyer, may: [example.order.Place]}
events:
  - name: example.order.Placed
    fields: [{name: order_id, type: example.order.OrderId}]
commands:
  - name: example.order.Place
    input:
      - {name: title, type: String}
      - {name: placed_at, type: Timestamp}
    outcomes:
      - name: placed
        creates: example.order.Order
        instance: order_id
        sets: {title: input.title, placed_at: input.placed_at}
        emits: [example.order.Placed]
        payload: {example.order.Placed: {order_id: {generated: true}}}
views:
  - name: example.order.Orders
    source: example.order.Order
    consistency: read_your_writes
    fields:
      - {name: order_id, type: example.order.OrderId}
      - {name: title, type: String}
      - {name: placed_at, type: Timestamp}
";

/// [`ORDERS`], compiled.
pub fn orders_ir() -> EssIr {
    let raw = RawSpecFile::parse(ORDERS).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("orders.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

/// The shape every row of `example.order.Orders` is held to.
pub fn orders_shape() -> PayloadShape {
    let mut shape = PayloadShape::new();
    for (path, kind) in [
        ("order_id", Primitive::String),
        ("placed_at", Primitive::Timestamp),
        ("title", Primitive::String),
    ] {
        shape.insert(path, LeafShape::required(Holds::Primitive { kind }));
    }
    shape
}

/// The synthesized suite of [`ORDERS`], written by hand at `major` with the row shape on every
/// view step.
pub fn orders_suite_at(major: u32) -> ConformanceSuite {
    let synthesis = ess_conformance::synthesize::synthesize(&orders_ir());
    assert!(synthesis.refusals.is_empty(), "{:#?}", synthesis.refusals);
    let mut suite = synthesis.suite;
    suite.provenance.suite_version =
        SuiteFormat::parse(&format!("ess-conformance/{major}")).expect("a suite format");
    let mut carried = 0;
    for scenario in suite.scenarios.values_mut() {
        for step in &mut scenario.steps {
            if let ScenarioStep::ExpectView { shape, .. }
            | ScenarioStep::EventuallyView { shape, .. } = step
            {
                *shape = orders_shape();
                carried += 1;
            }
        }
    }
    assert!(carried > 0, "the suite reads the view: {suite:#?}");
    suite
}

/// How [`RowFaults`] answers a read of the view.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RowFault {
    None,
    NoPlacedAt,
    PlacedAtSeven,
}

/// The interpreted model, answering every view row with `fault`.
pub struct RowFaults {
    inner: Interpreted,
    fault: RowFault,
}

impl RowFaults {
    pub fn new(fault: RowFault) -> Self {
        Self {
            inner: Interpreted::for_model(orders_ir()),
            fault,
        }
    }

    fn answer(&self, mut result: SemanticViewResult) -> SemanticViewResult {
        for row in &mut result.rows {
            match self.fault {
                RowFault::None => {}
                RowFault::NoPlacedAt => {
                    row.remove("placed_at");
                }
                RowFault::PlacedAtSeven => {
                    row.insert("placed_at".into(), serde_json::from_str("7").unwrap());
                }
            }
        }
        result
    }
}

impl ConformanceTarget for RowFaults {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.inner.identity()
    }
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.begin_scenario(scenario)
    }
    fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.end_scenario(scenario)
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        self.inner.execute_command(request)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        self.inner
            .query_view(request)
            .map(|result| self.answer(result))
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        self.inner.observe_events(request)
    }
    fn configure_external_outcome(
        &self,
        request: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        self.inner.configure_external_outcome(request)
    }
    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        self.inner.redeliver_event(request)
    }
    fn observe_invocations(
        &self,
        request: InvocationObservationRequest,
    ) -> Result<Vec<ObservedInvocation>, TargetError> {
        self.inner.observe_invocations(request)
    }
}
