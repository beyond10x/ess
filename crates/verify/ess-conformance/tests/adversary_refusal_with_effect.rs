//! Adversary cases for a refusal that declares its compensating change (`compensates: true`,
//! ess/22, beyond10x/ess#197, `docs/design/refusal-with-effect.md`).
//!
//! Each faulty mode below differs from a correct `JoinOrder` service in one way a caller or a
//! reader of the row can see. The design promises that a target which skips, misplaces, or adds to
//! the compensating change fails a synthesized scenario; each test asserts that at least one
//! scenario of the synthesized suite does not pass against that mode.

mod support_go;

use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::target::{
    ConformanceTarget, DeclaredErrorValue, EventObservationRequest, ExternalOutcomeControl,
    ImplementationIdentity, ObservedEvent, RedeliveryRequest, ScenarioContext,
    SemanticCommandRequest, SemanticCommandResult, SemanticViewRequest, SemanticViewResult,
    TargetError,
};
use ess_conformance::ConformanceSuite;
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

const MODEL: &str =
    include_str!("../../../specify/ess-compiler/tests/fixtures/refusal-with-effect.yaml");

fn ir() -> EssIr {
    let raw = RawSpecFile::parse(MODEL).expect("the model parses");
    let spec = Specification::assemble([(Source::new("order.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}"));
    compile(&spec, &SourceMap::new()).expect("the model compiles")
}

fn suite() -> ConformanceSuite {
    let synthesis = ess_conformance::synthesize::synthesize(&ir());
    assert_eq!(synthesis.refusals.len(), 0, "{:#?}", synthesis.refusals);
    synthesis.suite
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mode {
    Correct,
    /// From `Joined` only: answers `Refused` and leaves the row in `Joined`.
    RefusesWithoutResetFromJoined,
    /// From `Joined` only: resets to `Offline` and does not write `failure`.
    SkipsSetsFromJoined,
    /// Everywhere: resets and writes a constant `failure` instead of `input.reason`.
    WritesAConstantFailure,
    /// Resets the addressed row correctly and, as well, every other order not yet `Offline`.
    ResetsTheAddressedRowAndAnother,
    /// From `Joined` only: resets correctly and publishes `OrderJoined` as well.
    ExtraEventFromJoined,
    /// Correct on `failed`; on the `closed` refusal (a row resting in `Offline`) it applies a
    /// change too: the row goes back to `Idle` with `failure` overwritten.
    ChangesTheRowOnTheClosedRefusal,
    /// Correct on `failed`; the accepting `joined` branch also writes `failure` from the input.
    AcceptingBranchWritesFailure,
}

#[derive(Clone)]
struct Row {
    state: &'static str,
    failure: Option<String>,
}

struct Shop {
    mode: Mode,
    rows: std::cell::RefCell<BTreeMap<String, Row>>,
    forced: std::cell::RefCell<Option<String>>,
    published: std::cell::RefCell<Vec<ObservedEvent>>,
    sequence: std::cell::RefCell<u64>,
}

const JOIN: &str = "shop.order.JoinOrder";

impl Shop {
    fn new(mode: Mode) -> Self {
        Self {
            mode,
            rows: std::cell::RefCell::new(BTreeMap::new()),
            forced: std::cell::RefCell::new(None),
            published: std::cell::RefCell::new(Vec::new()),
            sequence: std::cell::RefCell::new(0),
        }
    }

    fn took(command: &str, outcome: &str) -> SemanticCommandResult {
        let mut result = SemanticCommandResult::took(ess_compiler::refs::OutcomeRef::new(
            ess_compiler::refs::CommandRef::new(command.parse().unwrap()),
            outcome.parse().unwrap(),
        ));
        result.consistency =
            Some(ess_primitives::consistency::ConsistencyToken::new("write").unwrap());
        result
    }

    fn refused(outcome: &str, error: &str) -> SemanticCommandResult {
        let mut result = Self::took(JOIN, outcome);
        result.error = Some(DeclaredErrorValue::new(error.parse().unwrap()));
        result
    }

    fn event(&self, name: &str, order: &str) -> ObservedEvent {
        let mut event = ObservedEvent::new(name.parse().unwrap());
        event
            .payload
            .insert("order_id".to_owned(), Node::Text(order.to_owned()));
        self.published.borrow_mut().push(event.clone());
        event
    }

    fn set(&self, order: &str, state: &'static str, failure: Option<String>) {
        let mut rows = self.rows.borrow_mut();
        let row = rows.get_mut(order).expect("a held order");
        row.state = state;
        if failure.is_some() {
            row.failure = failure;
        }
    }

    fn failed(&self, order: &str, reason: &str, held: &Row) -> SemanticCommandResult {
        let from_joined = held.state == "Joined";
        let refused = Self::refused("failed", "shop.order.Refused");
        match self.mode {
            Mode::RefusesWithoutResetFromJoined if from_joined => refused,
            Mode::SkipsSetsFromJoined if from_joined => {
                self.set(order, "Offline", None);
                refused
            }
            Mode::WritesAConstantFailure => {
                self.set(order, "Offline", Some("constant".to_owned()));
                refused
            }
            Mode::ResetsTheAddressedRowAndAnother => {
                self.set(order, "Offline", Some(reason.to_owned()));
                let others: Vec<String> = self
                    .rows
                    .borrow()
                    .iter()
                    .filter(|(id, row)| id.as_str() != order && row.state != "Offline")
                    .map(|(id, _)| id.clone())
                    .collect();
                for other in others {
                    self.set(&other, "Offline", Some(reason.to_owned()));
                }
                refused
            }
            Mode::ExtraEventFromJoined if from_joined => {
                self.set(order, "Offline", Some(reason.to_owned()));
                let mut refused = refused;
                refused
                    .direct_events
                    .push(self.event("shop.order.OrderJoined", order));
                refused
            }
            _ => {
                self.set(order, "Offline", Some(reason.to_owned()));
                refused
            }
        }
    }

    fn join(&self, order: &str, reason: &str) -> SemanticCommandResult {
        let forced = self.forced.borrow_mut().take();
        let Some(held) = self.rows.borrow().get(order).cloned() else {
            return Self::refused("closed", "shop.order.Closed");
        };
        if forced.as_deref() == Some("failed") && held.state != "Offline" {
            return self.failed(order, reason, &held);
        }
        if held.state != "Idle" {
            if self.mode == Mode::ChangesTheRowOnTheClosedRefusal {
                self.set(order, "Idle", Some(format!("closed: {reason}")));
            }
            return Self::refused("closed", "shop.order.Closed");
        }
        let failure = (self.mode == Mode::AcceptingBranchWritesFailure).then(|| reason.to_owned());
        self.set(order, "Joined", failure);
        let mut result = Self::took(JOIN, "joined");
        result
            .direct_events
            .push(self.event("shop.order.OrderJoined", order));
        result
    }
}

impl ConformanceTarget for Shop {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new(
            "adversary-refusal-with-effect",
            "1",
        ))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.rows.borrow_mut().clear();
        self.published.borrow_mut().clear();
        *self.forced.borrow_mut() = None;
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let text = |field: &str| {
            request
                .input
                .get(field)
                .and_then(Node::as_text)
                .unwrap_or_default()
                .to_owned()
        };
        match request.command.to_string().as_str() {
            "shop.order.PlaceOrder" => {
                let mut sequence = self.sequence.borrow_mut();
                *sequence += 1;
                let id = format!("00000000-0000-4000-8000-{:012}", *sequence);
                self.rows.borrow_mut().insert(
                    id.clone(),
                    Row {
                        state: "Idle",
                        failure: None,
                    },
                );
                let mut result = Self::took("shop.order.PlaceOrder", "placed");
                result
                    .direct_events
                    .push(self.event("shop.order.OrderPlaced", &id));
                Ok(result)
            }
            JOIN => Ok(self.join(&text("order_id"), &text("reason"))),
            other => panic!("unexpected command {other}"),
        }
    }
    fn query_view(&self, _: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        Ok(SemanticViewResult {
            rows: self
                .rows
                .borrow()
                .iter()
                .map(|(id, row)| {
                    let mut fields = BTreeMap::from([
                        ("order_id".to_owned(), Node::Text(id.clone())),
                        ("state".to_owned(), Node::Text(row.state.to_owned())),
                    ]);
                    if let Some(failure) = &row.failure {
                        fields.insert("failure".to_owned(), Node::Text(failure.clone()));
                    }
                    fields
                })
                .collect(),
            total: None,
        })
    }
    fn configure_external_outcome(
        &self,
        control: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        *self.forced.borrow_mut() = Some(control.force.outcome.to_string());
        Ok(())
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        Err(TargetError::unsupported("redelivery", "unused"))
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Ok(self
            .published
            .borrow()
            .iter()
            .filter(|event| event.event == request.event)
            .cloned()
            .collect())
    }
}

/// The Rust reference runner's verdicts for `mode`, after checking the correct target passes, so a
/// red here is the mode's fault and not the harness's.
fn rust_verdicts(mode: Mode) -> BTreeMap<String, String> {
    let suite = suite();
    let healthy = support_go::rust_outcomes(&suite, &Shop::new(Mode::Correct));
    assert_eq!(
        support_go::not_passed(&healthy),
        Vec::<&str>::new(),
        "the correct target passes: {healthy:#?}"
    );
    support_go::rust_outcomes(&suite, &Shop::new(mode))
}

fn assert_some_scenario_fails(mode: Mode) {
    let verdicts = rust_verdicts(mode);
    assert_ne!(
        support_go::not_passed(&verdicts),
        Vec::<&str>::new(),
        "{mode:?} passes every synthesized scenario: {verdicts:#?}"
    );
}

#[test]
fn adversary_refusing_without_the_reset_from_joined_fails_a_scenario() {
    assert_some_scenario_fails(Mode::RefusesWithoutResetFromJoined);
}

#[test]
fn adversary_skipping_the_sets_from_joined_fails_a_scenario() {
    assert_some_scenario_fails(Mode::SkipsSetsFromJoined);
}

#[test]
fn adversary_writing_a_constant_failure_fails_a_scenario() {
    assert_some_scenario_fails(Mode::WritesAConstantFailure);
}

#[test]
#[ignore = "follow-up: synthesis reads back only the rows a branch names, so an extra change to another row passes any branch (F2)"]
fn adversary_resetting_another_row_as_well_fails_a_scenario() {
    assert_some_scenario_fails(Mode::ResetsTheAddressedRowAndAnother);
}

#[test]
fn adversary_an_extra_event_from_joined_fails_a_scenario() {
    assert_some_scenario_fails(Mode::ExtraEventFromJoined);
}

#[test]
fn adversary_changing_the_row_on_the_closed_refusal_fails_a_scenario() {
    assert_some_scenario_fails(Mode::ChangesTheRowOnTheClosedRefusal);
}

#[test]
#[ignore = "follow-up: synthesis asserts no field a branch does not declare, so an accepting branch writing an undeclared field passes (F3)"]
fn adversary_the_accepting_branch_writing_failure_fails_a_scenario() {
    assert_some_scenario_fails(Mode::AcceptingBranchWritesFailure);
}

// ---- a compensating update that clears a field -----------------------------------------------

/// The fixture with `failed` as a compensating update clearing `failure` rather than a move (the
/// `reset` transition and `Offline` dropped, as nothing else takes them).
fn update_model(sets: &str) -> String {
    let lifecycle = "      states: [Idle, Joined, Offline]\n      terminal: [Offline]\n      transitions:\n        - {name: join, from: [Idle], to: Joined}\n        - {name: reset, from: [Idle, Joined], to: Offline}\n";
    let effect = "        moves: shop.order.Order.reset\n        instance: order_id\n        sets: {failure: input.reason}\n";
    let text = MODEL.replace(
        lifecycle,
        "      states: [Idle, Joined]\n      terminal: [Joined]\n      transitions:\n        - {name: join, from: [Idle], to: Joined}\n",
    );
    let text = text.replace(
        effect,
        &format!("        updates: shop.order.Order\n        instance: order_id\n        sets: {{failure: {sets}}}\n"),
    );
    assert!(
        text.contains("updates: shop.order.Order") && !text.contains("reset"),
        "{text}"
    );
    text
}

struct Clearing {
    skips: bool,
    writes: Option<String>,
    shop: Shop,
}

impl ConformanceTarget for Clearing {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.shop.identity()
    }
    fn begin_scenario(&self, context: &ScenarioContext) -> Result<(), TargetError> {
        self.shop.begin_scenario(context)
    }
    fn end_scenario(&self, context: &ScenarioContext) -> Result<(), TargetError> {
        self.shop.end_scenario(context)
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        if request.command.to_string() == JOIN
            && self.shop.forced.borrow().as_deref() == Some("failed")
        {
            *self.shop.forced.borrow_mut() = None;
            let order = request
                .input
                .get("order_id")
                .and_then(Node::as_text)
                .unwrap_or_default()
                .to_owned();
            if let Some(row) = self.shop.rows.borrow_mut().get_mut(&order) {
                if !self.skips {
                    row.failure.clone_from(&self.writes);
                }
                return Ok(Shop::refused("failed", "shop.order.Refused"));
            }
            return Ok(Shop::refused("closed", "shop.order.Closed"));
        }
        self.shop.execute_command(request)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        // An absent `failure` is read as `null`, as the clearing branch's read-back spells it.
        let mut result = self.shop.query_view(request)?;
        for row in &mut result.rows {
            row.entry("failure".to_owned()).or_insert(Node::Null);
        }
        Ok(result)
    }
    fn configure_external_outcome(
        &self,
        control: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        self.shop.configure_external_outcome(control)
    }
    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        self.shop.redeliver_event(request)
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        self.shop.observe_events(request)
    }
}

/// A compensating update whose only write clears a field: either synthesis refuses the branch by
/// name, or its scenario fails a target that answers the error and never clears.
#[test]
fn adversary_a_compensating_clear_is_refused_or_fails_a_target_that_skips_it() {
    update_variant("{cleared: true}", None);
}

/// The same for a compensating update writing a generated value.
#[test]
fn adversary_a_compensating_generated_write_is_refused_or_fails_a_target_that_skips_it() {
    update_variant("{generated: true}", Some("generated-1"));
}

fn update_variant(sets: &str, writes: Option<&str>) {
    let text = update_model(sets);
    let raw = RawSpecFile::parse(&text).expect("the variant parses");
    let spec = Specification::assemble([(Source::new("order.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the variant is admitted: {errors}\n{text}"));
    let ir = compile(&spec, &SourceMap::new()).expect("the variant compiles");
    let synthesis = ess_conformance::synthesize::synthesize(&ir);
    let failed = "shop.order.JoinOrder/outcome/failed";
    if !synthesis
        .suite
        .scenarios
        .keys()
        .any(|id| id.to_string() == failed)
    {
        // Refused by name: the design's `no_witness` answer.
        return;
    }
    let healthy = support_go::rust_outcomes(
        &synthesis.suite,
        &Clearing {
            skips: false,
            writes: writes.map(str::to_owned),
            shop: Shop::new(Mode::Correct),
        },
    );
    let detail = || {
        let admitted = ess_conformance::AdmittedSuite::from_suite(&synthesis.suite).unwrap();
        let report = ess_conformance::Runner::for_suite(admitted.suite())
            .run_admitted(
                &admitted,
                &Clearing {
                    skips: false,
                    writes: writes.map(str::to_owned),
                    shop: Shop::new(Mode::Correct),
                },
            )
            .into_report();
        format!(
            "{:#?}",
            report
                .scenarios
                .iter()
                .find(|result| result.scenario.to_string() == failed)
        )
    };
    assert_eq!(
        healthy.get(failed).map(String::as_str),
        Some("passed"),
        "the clearing target passes the branch: {healthy:#?}\n{}",
        detail()
    );
    let skipping = support_go::rust_outcomes(
        &synthesis.suite,
        &Clearing {
            skips: true,
            writes: writes.map(str::to_owned),
            shop: Shop::new(Mode::Correct),
        },
    );
    let steps = &synthesis
        .suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == failed)
        .unwrap()
        .1
        .steps;
    assert_ne!(
        skipping.get(failed).map(String::as_str),
        Some("passed"),
        "a target that never clears passes the compensating scenario: {steps:#?}"
    );
}

/// Every mutant of the fixture that touches the compensating branch or its `reset` move is killed,
/// stillborn or reported equivalent: none survives the interpreter.
#[test]
#[ignore = "follow-up: from-drop survivor on any external move (docs/design/refusal-with-effect.md §Every lane, mutation)"]
fn adversary_no_mutant_of_the_compensating_branch_survives() {
    use ess_conformance::mutate::{self, MutantClass, Verdict};
    let mut texts = SourceMap::new();
    texts.insert("order.yaml".to_owned(), MODEL.to_owned());
    let documents = vec![(
        Source::new("order.yaml"),
        RawSpecFile::parse(MODEL).unwrap_or_else(|error| panic!("{error}")),
    )];
    let original = mutate::compile(documents.clone(), &texts).unwrap_or_else(|error| {
        panic!("{error:?}");
    });
    let mutants = mutate::mutants(&documents, MutantClass::ALL);
    let mut seen = Vec::new();
    let mut survivors = Vec::new();
    for mutant in &mutants {
        let entry = mutate::evaluate(&documents, &texts, mutant, || {
            Interpreted::for_model(original.clone())
        })
        .unwrap_or_else(|refusal| panic!("{}: {refusal:?}", mutant.id));
        seen.push(format!(
            "{} [{}] => {:?}",
            mutant.id, mutant.change, entry.verdict
        ));
        if matches!(entry.verdict, Verdict::Survived | Verdict::Inconclusive) {
            survivors.push(format!(
                "{} [{}] => {:?}",
                mutant.id, mutant.change, entry.verdict
            ));
        }
    }
    eprintln!("every mutant:\n{}", seen.join("\n"));
    let touching: Vec<&String> = survivors
        .iter()
        .filter(|line| {
            line.contains("failed")
                || line.contains("reset")
                || line.contains("failure")
                || line.contains("Offline")
        })
        .collect();
    assert_eq!(
        touching,
        Vec::<&String>::new(),
        "surviving mutants of the compensating branch"
    );
}
