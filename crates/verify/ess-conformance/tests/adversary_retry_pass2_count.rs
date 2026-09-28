//! Adversary pass 2 against the bounded retry (beyond10x/ess#165): the pass-1 correction that
//! keeps the arrangement from setting off the counted binding (`clean_trigger`).
//!
//! The correction reads "sets off" as "a step of the arrangement executes a command with a branch
//! that emits the binding's event". A binding chain is the other way an arrangement publishes that
//! event: the arrangement's command emits `Opened`, a second binding on `Opened` invokes
//! `Announce`, and `Announce` publishes `OrderPlaced`. The counted binding is then invoked once
//! while arranging, and a correct sender shows `attempts + 1`.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::{ConformanceScenario, ConformanceSuite, ScenarioStep};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const MODEL: &str = include_str!("../../../specify/ess-compiler/tests/fixtures/bounded-retry.yaml");

const ON_FAILURE: &str = "notify-ledger/binding/on-failure";
const FINAL: &str = "notify-ledger/binding/final-failure";

fn ir_of(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("the model parses");
    let spec = Specification::assemble([(Source::new("bounded-retry.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}"));
    compile(&spec, &SourceMap::new()).expect("the model compiles")
}

fn find<'a>(suite: &'a ConformanceSuite, id: &str) -> Option<&'a ConformanceScenario> {
    suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map(|(_, scenario)| scenario)
}

/// `Amend` is the first publisher of `OrderPlaced` and needs an order, which `Place` makes.
/// `Place` publishes `Opened`, not `OrderPlaced`, and `announce-on-open` turns `Opened` into
/// `Announce`, which publishes `OrderPlaced`.
fn chained_model() -> String {
    let model = MODEL
        .replace(
            "events:\n",
            r"entities:
  - name: demo.ledger.Order
    identity: {name: order_id, type: demo.ledger.OrderId}
    fields:
      - {name: note, type: String}
    lifecycle:
      initial: Open
      states: [Open]
      terminal: [Open]
      transitions: []
events:
  - name: demo.ledger.Opened
    fields:
      - {name: order_id, type: demo.ledger.OrderId}
",
        )
        .replace(
            r"commands:
  - name: demo.ledger.Place
    input:
      - {name: order_id, type: demo.ledger.OrderId}
    outcomes:
      - name: placed
        emits: [demo.ledger.OrderPlaced]
        payload:
          demo.ledger.OrderPlaced: {order_id: input.order_id}
",
            r"commands:
  - name: demo.ledger.Amend
    input:
      - {name: order_id, type: demo.ledger.OrderId}
      - {name: note, type: String}
    outcomes:
      - name: amended
        updates: demo.ledger.Order
        instance: order_id
        sets: {note: input.note}
        emits: [demo.ledger.OrderPlaced]
        payload:
          demo.ledger.OrderPlaced: {order_id: input.order_id}
  - name: demo.ledger.Announce
    input:
      - {name: order_id, type: demo.ledger.OrderId}
    outcomes:
      - name: announced
        emits: [demo.ledger.OrderPlaced]
        payload:
          demo.ledger.OrderPlaced: {order_id: input.order_id}
  - name: demo.ledger.Place
    input:
      - {name: order_id, type: demo.ledger.OrderId}
      - {name: note, type: String}
    outcomes:
      - name: placed
        creates: demo.ledger.Order
        instance: order_id
        sets: {note: input.note}
        emits: [demo.ledger.Opened]
        payload:
          demo.ledger.Opened: {order_id: input.order_id}
",
        )
        .replace(
            "may: [demo.ledger.Place, demo.ledger.Record]",
            "may: [demo.ledger.Amend, demo.ledger.Announce, demo.ledger.Place, demo.ledger.Record]",
        )
        .replace(
            "bindings:\n",
            r"bindings:
  - id: announce-on-open
    when:
      event: demo.ledger.Opened
    invoke:
      command: demo.ledger.Announce
    mapping:
      order_id: event.order_id
    delivery: at_least_once
    on_failure: retry
",
        );
    assert!(model.contains("announce-on-open"), "{model}");
    assert!(model.contains("demo.ledger.Amend\n"), "{model}");
    assert!(model.contains("emits: [demo.ledger.Opened]"), "{model}");
    model
}

/// The commands a scenario executes before it arms the forced failure.
fn arranged(scenario: &ConformanceScenario) -> Vec<String> {
    let arming = scenario
        .steps
        .iter()
        .position(|step| matches!(step, ScenarioStep::ConfigureExternalOutcome { .. }))
        .expect("the failure is forced");
    scenario.steps[..arming]
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, .. }
            | ScenarioStep::ExecuteCommandWithoutInput { command, .. } => Some(command.to_string()),
            _ => None,
        })
        .collect()
}

/// `Place` publishes `OrderPlaced` through `announce-on-open`, so an arrangement running it
/// invokes `notify-ledger` before the counted attempts begin. The scenario must either pick a
/// trigger whose arrangement runs none of `Place`, `Amend`, `Announce`, or be refused by name.
#[test]
fn adversary_retry_pass2_an_arrangement_does_not_set_off_the_binding_through_a_chain() {
    let ir = ir_of(&chained_model());
    let synthesis = ess_conformance::synthesize::synthesize(&ir);
    let publishers = [
        "demo.ledger.Place",
        "demo.ledger.Amend",
        "demo.ledger.Announce",
    ];
    let mut offending = Vec::new();
    for id in [ON_FAILURE, FINAL] {
        let Some(scenario) = find(&synthesis.suite, id) else {
            continue; // refused: the count is not claimed
        };
        let before: Vec<String> = arranged(scenario)
            .into_iter()
            .filter(|command| publishers.contains(&command.as_str()))
            .collect();
        if !before.is_empty() {
            offending.push(format!("{id}: arranges {before:?}"));
        }
    }
    assert!(
        offending.is_empty(),
        "the arrangement publishes `OrderPlaced` through `announce-on-open` and so invokes \
         `notify-ledger` before the counted attempts: {offending:#?}"
    );
}
