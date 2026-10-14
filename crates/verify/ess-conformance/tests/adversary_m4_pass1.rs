//! Adversary pass 1 for one event per changed row (ess/25, beyond10x/ess#531,
//! `docs/design/set-effects-over-filtered-instances.md`, "One event per changed row").
//!
//! Each case drives the interpreter or the synthesized suite from the design page: the event order
//! by identity, rows a move does not change, two per-row events with different state filters, a
//! per-row event over an `updates:` that rewrites the field it reads, an `Optional` field absent
//! before the outcome, and targets that publish one event in all, read the row after the outcome
//! or read another row's values.

use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::execute::{execute, Externals, Step, Store};
use ess_conformance::report::Status;
use ess_conformance::target::*;
use ess_conformance::{AdmittedSuite, ConformanceSuite, Runner};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

const MODEL: &str =
    include_str!("../../../specify/ess-compiler/tests/fixtures/set-effects-per-row.yaml");
const CLOSED: &str = "demo.desk.CloseAll/outcome/closed-all";
const REASSIGNED: &str = "demo.desk.Reassign/outcome/reassigned";
const LEFT: &str = "demo.desk.Left";
const ENTRY: &str = "{event: demo.desk.Left, each: changed, when_state: Busy}";

fn ir_of(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("the model parses");
    let spec = Specification::assemble([(Source::new("desk.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}\n{text}"));
    compile(&spec, &SourceMap::new()).expect("the model compiles")
}

fn edited(from: &str, to: &str) -> String {
    assert!(MODEL.contains(from), "the fixture holds:\n{from}");
    MODEL.replacen(from, to, 1)
}

fn values<const N: usize>(entries: [(&str, Node); N]) -> BTreeMap<String, Node> {
    entries
        .into_iter()
        .map(|(name, value)| (name.into(), value))
        .collect()
}

fn text(value: &str) -> Node {
    Node::Text(value.into())
}

fn invoke(ir: &EssIr, store: &mut Store, command: &str, input: &BTreeMap<String, Node>) -> Step {
    let mut steps = execute(
        ir,
        store,
        &format!("demo.desk.{command}").parse().unwrap(),
        input,
        &Externals::Withheld,
    )
    .unwrap_or_else(|error| panic!("{command}: {error:?}"));
    assert_eq!(steps.len(), 1);
    let step = steps.remove(0);
    *store = step.next.clone();
    step
}

fn published(step: &Step, event: &str) -> Vec<BTreeMap<String, Node>> {
    step.events
        .iter()
        .filter(|published| published.event.to_string() == event)
        .map(|published| published.payload.clone())
        .collect()
}

fn run<T: ConformanceTarget>(suite: &ConformanceSuite, target: &T) -> BTreeMap<String, Status> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result.status))
        .collect()
}

fn not_passed(statuses: &BTreeMap<String, Status>) -> Vec<&str> {
    statuses
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .map(|(id, _)| id.as_str())
        .collect()
}

/// What a broken target does to the events one command published, given the command's input.
type Fault = fn(&str, &BTreeMap<String, Node>, &mut Vec<ObservedEvent>);

/// The interpreter, with the events of one command rewritten by a fault.
struct Target {
    inner: ess_conformance::interpret::Interpreted,
    fault: Fault,
}

impl Target {
    fn new(ir: &EssIr, fault: Fault) -> Self {
        Self {
            inner: ess_conformance::interpret::Interpreted::for_model(ir.clone()),
            fault,
        }
    }
}

impl ConformanceTarget for Target {
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
        let command = request.command.to_string();
        let input = request.input.clone();
        let mut result = self.inner.execute_command(request)?;
        (self.fault)(&command, &input, &mut result.direct_events);
        Ok(result)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        self.inner.query_view(request)
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
}

/// The design page: "one occurrence per row, in ascending order of the rows' identities". An
/// `Integer` identity orders as a number, so `9` comes before `10` and `10` before `100`.
#[test]
fn per_row_events_follow_numeric_identity_order() {
    let ir = ir_of(&edited(
        "{name: demo.desk.UserId, kind: newtype, of: String}",
        "{name: demo.desk.UserId, kind: newtype, of: Integer}",
    ));
    let mut store = Store::default();
    let number = |n: i64| Node::Number(n.into());
    for seat in [100, 9, 10] {
        invoke(
            &ir,
            &mut store,
            "SitDown",
            &values([("user_id", number(seat)), ("team", text("red"))]),
        );
        invoke(
            &ir,
            &mut store,
            "Take",
            &values([("user_id", number(seat)), ("call_ref", text("c"))]),
        );
    }
    let step = invoke(
        &ir,
        &mut store,
        "CloseAll",
        &values([("team", text("red"))]),
    );
    let order: Vec<Node> = published(&step, LEFT)
        .into_iter()
        .map(|payload| payload["user_id"].clone())
        .collect();
    assert_eq!(order, vec![number(9), number(10), number(100)]);
}

/// "Which rows": under a `moves:` the rows it changed are the selected rows resting in the
/// transition's `from` states. A team closed twice changes nothing the second time, so it
/// publishes no `Left` and counts 0.
#[test]
fn a_selected_row_the_move_skips_publishes_nothing() {
    let ir = ir_of(MODEL);
    let mut store = Store::default();
    invoke(
        &ir,
        &mut store,
        "SitDown",
        &values([("user_id", text("s1")), ("team", text("red"))]),
    );
    invoke(
        &ir,
        &mut store,
        "Take",
        &values([("user_id", text("s1")), ("call_ref", text("c1"))]),
    );
    let first = invoke(
        &ir,
        &mut store,
        "CloseAll",
        &values([("team", text("red"))]),
    );
    assert_eq!(published(&first, LEFT).len(), 1, "{first:?}");
    let second = invoke(
        &ir,
        &mut store,
        "CloseAll",
        &values([("team", text("red"))]),
    );
    assert!(published(&second, LEFT).is_empty(), "{second:?}");
    assert_eq!(
        published(&second, "demo.desk.TeamClosed")[0]["closed"],
        Node::Number(0_i64.into())
    );
}

/// Without `when_state:` every changed row publishes `Left`, an `Open` seat among them, whose
/// `Optional` `call_ref` was absent before the outcome. The synthesized suite synthesizes the
/// scenario and the interpreter passes all of it.
#[test]
fn an_unfiltered_per_row_event_over_an_absent_optional_is_synthesized_and_passed() {
    let ir = ir_of(&edited(ENTRY, "{event: demo.desk.Left, each: changed}"));
    let synthesis = ess_conformance::synthesize::synthesize(&ir);
    assert!(
        synthesis
            .refusals
            .iter()
            .all(|refusal| !format!("{refusal:?}").contains("CloseAll")),
        "{:#?}",
        synthesis.refusals
    );
    let statuses = run(
        &synthesis.suite,
        &ess_conformance::interpret::Interpreted::for_model(ir.clone()),
    );
    assert_eq!(statuses.get(CLOSED), Some(&Status::Passed), "{statuses:#?}");
    assert!(not_passed(&statuses).is_empty(), "{statuses:#?}");
}

/// Two per-row events whose state filters differ: `Left` for each seat that was `Busy`, `Released`
/// for each that was `Open`. A changed row can rest where each filter admits it, so the page's
/// one refusal ("where no changed row can rest in a state the filter admits") does not apply and
/// the scenario is synthesized; the interpreter passes it.
#[test]
fn two_per_row_events_with_different_state_filters_are_witnessed() {
    let model = edited(
        "errors:\n",
        "  - name: demo.desk.Released\n    fields:\n      - {name: user_id, type: demo.desk.UserId}\nerrors:\n",
    )
    .replacen(
        &format!("          - {ENTRY}\n"),
        &format!(
            "          - {ENTRY}\n          - {{event: demo.desk.Released, each: changed, when_state: Open}}\n"
        ),
        1,
    )
    .replacen(
        "          demo.desk.Left: {user_id: {changed: user_id}",
        "          demo.desk.Released: {user_id: {changed: user_id}}\n          demo.desk.Left: {user_id: {changed: user_id}",
        1,
    );
    let ir = ir_of(&model);
    let synthesis = ess_conformance::synthesize::synthesize(&ir);
    assert!(
        synthesis
            .refusals
            .iter()
            .all(|refusal| !format!("{refusal:?}").contains("CloseAll")),
        "{:#?}",
        synthesis.refusals
    );
    let statuses = run(
        &synthesis.suite,
        &ess_conformance::interpret::Interpreted::for_model(ir.clone()),
    );
    assert_eq!(statuses.get(CLOSED), Some(&Status::Passed), "{statuses:#?}");
}

/// Publishes only the first `Left` of the command, whatever rows it changed.
fn one_left_in_all(command: &str, _: &BTreeMap<String, Node>, events: &mut Vec<ObservedEvent>) {
    if command != "demo.desk.CloseAll" {
        return;
    }
    let mut seen = false;
    events.retain(|event| {
        if event.event.to_string() != LEFT {
            return true;
        }
        let keep = !seen;
        seen = true;
        keep
    });
}

/// Reads `{changed: state}` after the move: every `Left` says `Closed`.
#[allow(clippy::ptr_arg)] // The `Fault` signature.
fn state_after_the_move(
    command: &str,
    _: &BTreeMap<String, Node>,
    events: &mut Vec<ObservedEvent>,
) {
    if command != "demo.desk.CloseAll" {
        return;
    }
    for event in events.iter_mut() {
        if event.event.to_string() == LEFT {
            event.payload.insert("was".into(), text("Closed"));
        }
    }
}

/// Reads `call_ref` from another changed row: the `call_ref`s of the `Left`s rotate by one.
#[allow(clippy::ptr_arg)] // The `Fault` signature.
fn call_ref_of_another_row(
    command: &str,
    _: &BTreeMap<String, Node>,
    events: &mut Vec<ObservedEvent>,
) {
    if command != "demo.desk.CloseAll" {
        return;
    }
    let lefts: Vec<usize> = (0..events.len())
        .filter(|index| events[*index].event.to_string() == LEFT)
        .collect();
    if lefts.len() < 2 {
        return;
    }
    let refs: Vec<Option<Node>> = lefts
        .iter()
        .map(|index| events[*index].payload.get("call_ref").cloned())
        .collect();
    for (n, index) in lefts.iter().enumerate() {
        match &refs[(n + 1) % refs.len()] {
            Some(value) => {
                events[*index]
                    .payload
                    .insert("call_ref".into(), value.clone());
            }
            None => {
                events[*index].payload.remove("call_ref");
            }
        }
    }
}

/// The page: a target omitting a row fails a claim, one reading another row's values fails
/// `ESS-CF-PAYLOAD`, and every value is read as the row was before the outcome. The broken target
/// fails the `closed-all` scenario that the interpreter passes.
fn closed_all_fails_under(fault: Fault) {
    let ir = ir_of(MODEL);
    let synthesis = ess_conformance::synthesize::synthesize(&ir);
    let correct = run(
        &synthesis.suite,
        &ess_conformance::interpret::Interpreted::for_model(ir.clone()),
    );
    assert_eq!(correct.get(CLOSED), Some(&Status::Passed), "{correct:#?}");
    assert_eq!(
        run(&synthesis.suite, &Target::new(&ir, fault)).get(CLOSED),
        Some(&Status::Failed)
    );
}

#[test]
fn the_suite_fails_a_target_publishing_one_event_in_all() {
    closed_all_fails_under(one_left_in_all);
}

#[test]
fn the_suite_fails_a_target_reading_the_state_after_the_move() {
    closed_all_fails_under(state_after_the_move);
}

#[test]
fn the_suite_fails_a_target_reading_another_rows_field() {
    closed_all_fails_under(call_ref_of_another_row);
}

/// A per-row event over an `updates:` that rewrites the very field its payload reads.
fn reassign_model() -> String {
    edited(
        "errors:\n",
        "  - name: demo.desk.Moved\n    fields:\n      - {name: user_id, type: demo.desk.UserId}\n      - {name: from, type: demo.desk.Team}\nerrors:\n",
    )
    .replacen(
        "may: [demo.desk.SitDown, demo.desk.Take, demo.desk.CloseAll]",
        "may: [demo.desk.SitDown, demo.desk.Take, demo.desk.CloseAll, demo.desk.Reassign]",
        1,
    )
    .replacen(
        "views:\n",
        "  - name: demo.desk.Reassign
    input:
      - {name: team, type: demo.desk.Team}
      - {name: to, type: demo.desk.Team}
    outcomes:
      - name: reassigned
        updates: demo.desk.Seat
        instances: {where: team == input.team}
        emits:
          - {event: demo.desk.Moved, each: changed}
        payload:
          demo.desk.Moved: {user_id: {changed: user_id}, from: {changed: team}}
        sets:
          team: input.to
views:\n",
        1,
    )
}

/// Reads `{changed: team}` after the update: every `Moved` says the team it moved to.
#[allow(clippy::ptr_arg)] // The `Fault` signature.
fn team_after_the_update(
    command: &str,
    input: &BTreeMap<String, Node>,
    events: &mut Vec<ObservedEvent>,
) {
    if command != "demo.desk.Reassign" {
        return;
    }
    for event in events.iter_mut() {
        if event.event.to_string() == "demo.desk.Moved" {
            event.payload.insert(
                "from".into(),
                input.get("to").cloned().unwrap_or(Node::Null),
            );
        }
    }
}

/// The interpreter reads the team each row had before the update, and the synthesized suite fails
/// a target that reads it after.
#[test]
fn an_update_rewriting_the_field_read_is_read_before_the_outcome() {
    let ir = ir_of(&reassign_model());
    let mut store = Store::default();
    for seat in ["s2", "s1"] {
        invoke(
            &ir,
            &mut store,
            "SitDown",
            &values([("user_id", text(seat)), ("team", text("red"))]),
        );
    }
    let step = invoke(
        &ir,
        &mut store,
        "Reassign",
        &values([("team", text("red")), ("to", text("blue"))]),
    );
    assert_eq!(
        published(&step, "demo.desk.Moved"),
        vec![
            values([("user_id", text("s1")), ("from", text("red"))]),
            values([("user_id", text("s2")), ("from", text("red"))]),
        ]
    );

    let synthesis = ess_conformance::synthesize::synthesize(&ir);
    let correct = run(
        &synthesis.suite,
        &ess_conformance::interpret::Interpreted::for_model(ir.clone()),
    );
    assert_eq!(
        correct.get(REASSIGNED),
        Some(&Status::Passed),
        "{correct:#?}\n{:#?}",
        synthesis.refusals
    );
    assert_eq!(
        run(&synthesis.suite, &Target::new(&ir, team_after_the_update)).get(REASSIGNED),
        Some(&Status::Failed)
    );
}
