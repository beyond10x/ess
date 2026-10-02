//! Keys filled by one input are equal, including when absent (beyond10x/ess#309).
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::{
    interpret::Interpreted, report::Status, scenario::ViewExpectation, synthesize::synthesize,
    target::*, AdmittedSuite, Runner, ScenarioStep, ScenarioValue,
};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::node::Node;
use std::{cell::RefCell, collections::BTreeMap};

const MODEL: &str = r"format: ess/20
system: demo
version: v1
domain: demo.shipping
entities:
  - name: demo.shipping.Order
    identity: {name: order_id, type: Uuid}
    fields:
      - {name: origin, type: String}
      - {name: destination, type: String}
      - {name: carrier, type: String}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
events:
  - name: demo.shipping.Opened
    fields:
      - {name: order_id, type: Uuid}
      - {name: place, type: String}
      - {name: carrier, type: String}
commands:
  - name: demo.shipping.Open
    input:
      - {name: place, type: String}
      - {name: carrier, type: String}
    outcomes:
      - name: opened
        creates: demo.shipping.Order
        instance: order_id
        sets: {origin: input.place, destination: input.place, carrier: input.carrier}
        emits: [demo.shipping.Opened]
        payload:
          demo.shipping.Opened: {order_id: {generated: true}, place: input.place, carrier: input.carrier}
views:
  - name: demo.shipping.Orders
    source: demo.shipping.Order
    fields:
      - {name: order_id, type: Uuid}
      - {name: state, type: demo.shipping.Order.State}
      - {name: origin, type: String}
      - {name: destination, type: String}
      - {name: carrier, type: String}
  - name: demo.shipping.PerRoute
    source: demo.shipping.Order
    group_by: [origin, destination]
    fields:
      - {name: origin, type: String}
      - {name: destination, type: String}
      - {name: orders, type: Integer, aggregate: {count: {}}}
";

fn check(optional: bool, independent: bool, reversed: bool) {
    let mut model = MODEL.to_owned();
    if independent {
        model = model
            .replace(
                "group_by: [origin, destination]",
                "group_by: [origin, destination, carrier]",
            )
            .replace(
                "      - {name: orders,",
                "      - {name: carrier, type: String}\n      - {name: orders,",
            );
    }
    if reversed {
        model = model
            .replace("[origin, destination", "[destination, origin")
            .replace(
                "      - {name: origin, type: String}\n      - {name: destination, type: String}",
                "      - {name: destination, type: String}\n      - {name: origin, type: String}",
            );
    }
    if optional {
        for name in ["origin", "destination", "place"] {
            model = model.replace(
                &format!("name: {name}, type: String"),
                &format!("name: {name}, type: Optional<String>"),
            );
        }
    }
    let raw = RawSpecFile::parse(&model).unwrap();
    let spec = Specification::assemble([(Source::new("shipping.yaml"), raw)]).unwrap();
    let ir = compile(&spec, &SourceMap::new()).unwrap();
    let mut synthesis = synthesize(&ir);
    let id = "demo.shipping.PerRoute/aggregate";
    assert!(
        synthesis
            .suite
            .scenarios
            .keys()
            .any(|key| key.to_string() == id),
        "{:?}",
        synthesis.refusals
    );
    synthesis
        .suite
        .scenarios
        .retain(|key, _| key.to_string() == id);
    let scenario = synthesis.suite.scenarios.values().next().unwrap();
    let rows: Vec<_> = scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExpectView {
                expectation: ViewExpectation::Contains { fields },
                ..
            }
            | ScenarioStep::EventuallyView {
                expectation: ViewExpectation::Contains { fields },
                ..
            } => Some(fields),
            _ => None,
        })
        .collect();
    assert!(!rows.is_empty());
    for row in &rows {
        assert_eq!(row.get("origin"), row.get("destination"), "{row:?}");
    }
    if optional {
        assert!(rows
            .iter()
            .any(|row| row.get("origin") == Some(&ScenarioValue::literal(Node::Null))));
    }
    if independent {
        // Both independent dimensions must vary while the other stays put. A planner that just
        // drops every B-key row would synthesize, but could no longer detect omitted keys.
        for (same, different) in [("origin", "carrier"), ("carrier", "origin")] {
            assert!(
                rows.iter().any(|a| rows
                    .iter()
                    .any(|b| a.get(same) == b.get(same) && a.get(different) != b.get(different))),
                "no distinction for {different}: {rows:?}"
            );
        }
    }
    let admitted = AdmittedSuite::from_suite(&synthesis.suite).unwrap();
    let target = Shipping {
        commands: Interpreted::for_model(ir),
        rows: RefCell::default(),
        independent,
    };
    let report = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &target)
        .into_report();
    assert_eq!(report.scenarios.len(), 1);
    assert_eq!(
        report.scenarios[0].status,
        Status::Passed,
        "{:?}",
        report.scenarios[0].checks
    );
}

/// The interpreter executes commands but does not implement views. Materialize this fixture's
/// count from its interpreted creation events, without reading the generated tuple pattern.
struct Shipping {
    commands: Interpreted,
    rows: RefCell<Vec<(Node, Node)>>,
    independent: bool,
}

impl ConformanceTarget for Shipping {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.commands.identity()
    }
    fn begin_scenario(&self, context: &ScenarioContext) -> Result<(), TargetError> {
        self.rows.borrow_mut().clear();
        self.commands.begin_scenario(context)
    }
    fn end_scenario(&self, context: &ScenarioContext) -> Result<(), TargetError> {
        self.commands.end_scenario(context)
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let result = self.commands.execute_command(request)?;
        for event in &result.direct_events {
            self.rows.borrow_mut().push((
                event.payload.get("place").cloned().unwrap_or(Node::Null),
                event.payload["carrier"].clone(),
            ));
        }
        Ok(result)
    }
    fn query_view(&self, _: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let mut groups: Vec<(Node, Node, usize)> = Vec::new();
        for (place, carrier) in self.rows.borrow().iter() {
            if let Some((_, _, count)) = groups
                .iter_mut()
                .find(|(p, c, _)| p == place && (!self.independent || c == carrier))
            {
                *count += 1;
            } else {
                groups.push((place.clone(), carrier.clone(), 1));
            }
        }
        Ok(SemanticViewResult::of(groups.into_iter().map(
            |(place, carrier, count)| {
                let mut row = BTreeMap::from([
                    ("origin".to_owned(), place.clone()),
                    ("destination".to_owned(), place),
                    ("orders".to_owned(), Node::Number(count.into())),
                ]);
                if self.independent {
                    row.insert("carrier".to_owned(), carrier);
                }
                row
            },
        )))
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        self.commands.observe_events(request)
    }
    fn configure_external_outcome(
        &self,
        request: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        self.commands.configure_external_outcome(request)
    }
    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        self.commands.redeliver_event(request)
    }
}

#[test]
fn shared_input_keys_without_moves() {
    check(false, false, false);
}

#[test]
fn shared_input_keys_preserve_an_independent_key() {
    check(false, true, false);
}

#[test]
fn shared_input_keys_can_be_reordered() {
    check(false, true, true);
}

#[test]
fn optional_shared_input_keys_are_absent_together() {
    check(true, true, false);
}

#[test]
fn optional_shared_input_keys_can_be_reordered() {
    check(true, true, true);
}
