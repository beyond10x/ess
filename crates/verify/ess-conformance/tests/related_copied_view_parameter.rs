//! Parameterized views must use the value actually copied from the named source (#360).
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::fmt::Write;

use ess_compiler::{
    ir::EssIr,
    refs::{CommandRef, OutcomeRef},
    resolve::compile,
    source::SourceMap,
};
use ess_conformance::{
    report::Status,
    scenario::{ScenarioId, ScenarioStep, ScenarioValue, ViewExpectation},
    synthesize::{synthesize, Synthesis},
    target::*,
    AdmittedSuite, ConformanceScenario, Runner,
};
use ess_domain::{
    command::OutcomeName,
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::{consistency::ConsistencyToken, node::Node};
use sha2::{Digest, Sha256};

const MODEL: &str = include_str!("fixtures/related-copied-view-parameter.yaml");
const PACKED: &str = "shipping.m.Pack/outcome/packed";
const DELIVERED: &str = "shipping.m.Deliver/outcome/delivered";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap();
    let spec = Specification::assemble([(Source::new("shipping.yaml"), raw)]).unwrap();
    compile(&spec, &SourceMap::new()).unwrap()
}

fn scenario<'a>(synthesis: &'a Synthesis, id: &str) -> &'a ConformanceScenario {
    synthesis
        .suite
        .scenarios
        .get(&ScenarioId::parse(id).unwrap())
        .unwrap_or_else(|| panic!("missing {id}: {:?}", synthesis.refusals))
}

fn assert_reads(scenario: &ConformanceScenario) {
    for (view, param) in [
        ("shipping.m.ByOrder", "order"),
        ("shipping.m.ByDepot", "depot"),
    ] {
        let reads: Vec<_> = scenario
            .steps
            .iter()
            .filter_map(|step| match step {
                ScenarioStep::QueryView { view: name, params } if name.to_string() == view => {
                    Some(params)
                }
                _ => None,
            })
            .collect();
        assert!(!reads.is_empty(), "{view}: {scenario:#?}");
        assert!(
            reads.iter().all(|params| params.contains_key(param)),
            "{view}: {reads:?}"
        );
        if param == "order" {
            assert!(
                reads.iter().all(|params| matches!(
                    params.get(param),
                    Some(ScenarioValue::Instance { .. })
                )),
                "{reads:?}"
            );
        }
    }
}

#[test]
fn copied_related_identity_binds_view_parameter_on_creation() {
    let synthesis = synthesize(&ir(MODEL));
    assert_reads(scenario(&synthesis, PACKED));
}

#[test]
fn copied_related_identity_binds_view_parameter_on_later_arrangement() {
    let synthesis = synthesize(&ir(MODEL));
    assert!(synthesis.refusals.is_empty(), "{:?}", synthesis.refusals);
    assert_reads(scenario(&synthesis, DELIVERED));
}

#[test]
fn copied_related_value_filters_matching_and_nonmatching_rows() {
    let synthesis = synthesize(&ir(MODEL));
    let packed = scenario(&synthesis, PACKED);
    for view in ["shipping.m.ByOrder", "shipping.m.ByDepot"] {
        assert!(
            packed.steps.iter().any(|step| matches!(step,
            ScenarioStep::ExpectView { view: name, expectation: ViewExpectation::Excludes { .. } }
            if name.to_string() == view)),
            "missing exclusion for {view}: {packed:#?}"
        );
    }
}

#[test]
fn related_read_through_a_copied_link_keeps_its_named_refusal() {
    let model = MODEL.replace("        moves: shipping.m.Shipment.deliver", "        sets: {depot: {related: {via: source_order, field: depot}}}\n        moves: shipping.m.Shipment.deliver");
    let synthesis = synthesize(&ir(&model));
    // Chaining another related read through a copied link is outside this view-selection repair.
    // Keep the existing explicit limitation; do not treat its target scenario as passed.
    let refusals: Vec<_> = synthesis.refusals.iter().map(ToString::to_string).collect();
    assert!(
        refusals
            .iter()
            .any(|refusal| refusal.contains("ESS-SYNTH-005")
                && refusal.contains("shipping.m.ByDepot")),
        "{refusals:?}"
    );
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mutant {
    None,
    FirstOrder,
    LastOrder,
    IgnoreOrder,
    IgnoreDepot,
}
type Row = BTreeMap<String, Node>;

struct Store {
    mutant: Mutant,
    orders: RefCell<Vec<Row>>,
    shipments: RefCell<Vec<Row>>,
    serial: Cell<u64>,
}

impl Store {
    fn new(mutant: Mutant) -> Self {
        Self {
            mutant,
            orders: RefCell::default(),
            shipments: RefCell::default(),
            serial: Cell::new(0),
        }
    }
    fn mint(&self) -> Node {
        let next = self.serial.get() + 1;
        self.serial.set(next);
        Node::Text(format!("00000000-0000-4000-8000-{next:012}"))
    }
}

fn answer(command: &CommandRef, outcome: &str) -> SemanticCommandResult {
    SemanticCommandResult::took(OutcomeRef::new(
        command.clone(),
        OutcomeName::new(outcome).unwrap(),
    ))
}

fn event(name: &str, field: &str, id: Node) -> ObservedEvent {
    ObservedEvent::new(name.parse().unwrap()).with(field, id)
}

impl ConformanceTarget for Store {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("shipping-360", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.orders.replace(Vec::new());
        self.shipments.replace(Vec::new());
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let command = &request.command;
        let input = &request.input;
        let result = match command.to_string().as_str() {
            "shipping.m.OpenOrder" => {
                let id = self.mint();
                self.orders.borrow_mut().push(Row::from([
                    ("order_id".into(), id.clone()),
                    ("depot".into(), input["depot"].clone()),
                ]));
                answer(command, "opened").emitting(event("shipping.m.OrderOpened", "order_id", id))
            }
            "shipping.m.Pack" => {
                let orders = self.orders.borrow();
                let order = match self.mutant {
                    Mutant::FirstOrder => orders.first(),
                    Mutant::LastOrder => orders.last(),
                    _ => orders
                        .iter()
                        .find(|row| row["order_id"] == input["order_id"]),
                }
                .ok_or_else(|| {
                    TargetError::unsupported("order", "the named source was not created")
                })?;
                let id = self.mint();
                self.shipments.borrow_mut().push(Row::from([
                    ("shipment_id".into(), id.clone()),
                    ("source_order".into(), order["order_id"].clone()),
                    ("depot".into(), order["depot"].clone()),
                    ("state".into(), Node::Text("Packed".into())),
                ]));
                answer(command, "packed").emitting(event("shipping.m.Packed", "shipment_id", id))
            }
            "shipping.m.Deliver" => {
                let mut rows = self.shipments.borrow_mut();
                match rows
                    .iter_mut()
                    .find(|row| row["shipment_id"] == input["shipment_id"])
                    .filter(|row| row["state"] == Node::Text("Packed".into()))
                {
                    Some(row) => {
                        row.insert("state".into(), Node::Text("Delivered".into()));
                        answer(command, "delivered").emitting(event(
                            "shipping.m.Delivered",
                            "shipment_id",
                            row["shipment_id"].clone(),
                        ))
                    }
                    None => answer(command, "conflict").with_error(DeclaredErrorValue::new(
                        "shipping.m.Conflict".parse().unwrap(),
                    )),
                }
            }
            other => return Err(TargetError::unsupported("command", other)),
        };
        Ok(result
            .with_consistency(ConsistencyToken::new(format!("seq:{}", self.serial.get())).unwrap()))
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let rows: Vec<_> = self
            .shipments
            .borrow()
            .iter()
            .filter(|row| match request.view.to_string().as_str() {
                "shipping.m.ByOrder" => {
                    self.mutant == Mutant::IgnoreOrder
                        || request.params.get("order") == row.get("source_order")
                }
                "shipping.m.ByDepot" => {
                    self.mutant == Mutant::IgnoreDepot
                        || request.params.get("depot") == row.get("depot")
                }
                "shipping.m.Shipments" => true,
                _ => false,
            })
            .cloned()
            .collect();
        Ok(SemanticViewResult::of(rows))
    }
    fn observe_events(
        &self,
        _: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Ok(Vec::new())
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        panic!("no external outcomes")
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        panic!("no bindings")
    }
}

fn statuses(mutant: Mutant) -> BTreeMap<String, Status> {
    model_statuses(MODEL, mutant)
}

fn model_statuses(model: &str, mutant: Mutant) -> BTreeMap<String, Status> {
    let synthesis = synthesize(&ir(model));
    let admitted = AdmittedSuite::from_suite(&synthesis.suite).unwrap();
    let target = Store::new(mutant);
    Runner::for_suite(&synthesis.suite)
        .run_admitted(&admitted, &target)
        .into_report()
        .scenarios
        .into_iter()
        .map(|run| (run.scenario.to_string(), run.status))
        .collect()
}

#[test]
fn honest_target_passes_every_copied_related_view_scenario() {
    let statuses = statuses(Mutant::None);
    for id in [
        PACKED,
        DELIVERED,
        "shipping.m.Shipment/transition/deliver/by/shipping.m.Deliver/delivered",
    ] {
        assert_eq!(statuses.get(id), Some(&Status::Passed), "{statuses:?}");
    }
    assert!(
        statuses.values().all(|status| *status == Status::Passed),
        "{statuses:?}"
    );
}

#[test]
fn wrong_related_row_and_ignored_param_mutants_fail() {
    let mut survivors = Vec::new();
    for mutant in [
        Mutant::FirstOrder,
        Mutant::LastOrder,
        Mutant::IgnoreOrder,
        Mutant::IgnoreDepot,
    ] {
        let statuses = statuses(mutant);
        if statuses.get(PACKED) != Some(&Status::Failed) {
            survivors.push(mutant);
        }
    }
    assert!(survivors.is_empty(), "view mutants survived: {survivors:?}");
}

#[test]
fn reversed_bare_field_comparison_keeps_its_existing_type_refusal() {
    let model = MODEL
        .replace("source_order == param.order", "param.order == source_order")
        .replace("depot == param.depot", "param.depot == depot");
    let raw = RawSpecFile::parse(&model).unwrap();
    let refusal = Specification::assemble([(Source::new("shipping.yaml"), raw)]).unwrap_err();
    assert!(
        refusal
            .to_string()
            .contains("a right-hand side without a dot is a literal"),
        "{refusal}"
    );
}

#[test]
fn cyclic_and_uncreatable_related_sources_keep_named_refusals() {
    let cyclic = MODEL
        .replace(
            "input: [{name: depot, type: String}]",
            "input: [{name: shipment_id, type: shipping.m.ShipmentId}]",
        )
        .replace(
            "sets: {depot: input.depot}",
            "sets: {depot: {related: {via: input.shipment_id, field: depot}}}",
        );
    let start = MODEL.find("  - name: shipping.m.OpenOrder\n").unwrap();
    let end = MODEL.find("  - name: shipping.m.Pack\n").unwrap();
    let uncreatable = format!("{}{}", &MODEL[..start], &MODEL[end..]);
    for model in [cyclic, uncreatable] {
        let synthesis = synthesize(&ir(&model));
        let refusals: Vec<_> = synthesis.refusals.iter().map(ToString::to_string).collect();
        assert!(
            refusals
                .iter()
                .any(|refusal| refusal.contains("ESS-SYNTH-005")),
            "{refusals:?}"
        );
        assert!(synthesis.suite.scenarios.len() < 32);
    }
}

/// The coordinator compares these canonical digests on identical tests against base and treatment.
/// They hold the established scalar-parameter/paging and direct identity arrangements to their bytes.
#[test]
fn existing_direct_field_parameter_canonical_bytes() {
    let direct = MODEL
        .replace(
            "input: [{name: order_id, type: shipping.m.OrderId}]",
            "input: [{name: order_id, type: shipping.m.OrderId}, {name: depot, type: String}]",
        )
        .replace(
            "{related: {via: input.order_id, field: order_id}}",
            "input.order_id",
        )
        .replace(
            "{related: {via: input.order_id, field: depot}}",
            "input.depot",
        )
        .replace(
            "params: [{name: order, type: shipping.m.OrderId}]",
            "params: [{name: source_order, type: shipping.m.OrderId}]",
        )
        .replace("param.order", "param.source_order");
    for (name, model) in [
        ("direct", direct.as_str()),
        ("paging", include_str!("fixtures/view-paging.yaml")),
    ] {
        let synthesis = synthesize(&ir(model));
        assert!(
            synthesis.refusals.is_empty(),
            "{name}: {:?}",
            synthesis.refusals
        );
        let canonical = synthesis.suite.to_canonical_json().unwrap();
        let mut digest = String::new();
        for byte in Sha256::digest(canonical.as_bytes()) {
            write!(digest, "{byte:02x}").unwrap();
        }
        println!("canonical {name} {digest}");
    }
}
