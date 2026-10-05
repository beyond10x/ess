//! A binding's failure policy selected per refusal (ess/22, beyond10x/ess#269;
//! `docs/design/conditional-binding-failure-policies.md`, "Refusal-selected failure policy").
//!
//! Synthesis witnesses every declared selected refusal on its own scenario,
//! `<binding>/binding/refusal/<outcome>` (suite/36 and /37): the refusal forced by injection, and
//! the attempt count, escalation and absence the selected policy owes, observed with the existing
//! count and event instructions and `expect_no_publication` (an escalation is the binding's, so
//! `expect_no_event`, which reads the last command's direct events, cannot see one). One in-memory
//! sender implements the binding healthily and in each faulty way the design names; each faulty
//! one is caught by the scenario it breaks.
//!
//! The named runtime controls (`selected_drop_once`, `selected_escalation_once`,
//! `selected_retry_to_success`, `selected_retry_exhausts_total_budget`,
//! `selected_retry_final_stops`, `outcome_aliases_expand_to_same_policy`,
//! `unknown_failure_uses_explicit_fallback`, `pre_input_failure_is_an_obligation`,
//! `untyped_port_failure_consumes_attempt_budget`, `fallback_escalation_uses_actual_complete_input`)
//! drive a scripted command port through the binding-running native fixture (the interpreter) and
//! through the same in-memory sender, so a sequence of different refusals in one retry is forced
//! where a suite cannot force one.

use std::cell::RefCell;
use std::collections::{BTreeMap, VecDeque};
use std::num::NonZeroU32;

use ess_compiler::refs::{BindingRef, CommandRef, OutcomeRef};
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::{Interpreted, PortAnswer};
use ess_conformance::report::Status;
use ess_conformance::target::*;
use ess_conformance::{AdmittedSuite, ConformanceScenario, ConformanceSuite, Runner, ScenarioStep};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::{ids::CorrelationId, node::Node, time::Timestamp};

const MODEL: &str = include_str!("fixtures/refusal-policy.yaml");
const POLICY: &str = "    on_failure:
      drop: [wrong-state]
      retry: {outcomes: [demo.ledger.Unavailable, rejected], attempts: 3, final: [rejected]}
      escalate:
        emits: demo.ledger.RecordEscalated
        except: [wrong-state, demo.ledger.Unavailable, rejected]
";

const BINDING: &str = "notify-ledger";
const PLACE: &str = "demo.ledger.Place";
const RECORD: &str = "demo.ledger.Record";
const RECORDED: &str = "demo.ledger.Recorded";
const ESCALATED: &str = "demo.ledger.RecordEscalated";
const REFUSALS: [&str; 5] = ["unavailable", "busy", "rejected", "at-limit", "wrong-state"];

fn id(outcome: &str) -> String {
    format!("{BINDING}/binding/refusal/{outcome}")
}

fn with_policy(policy: &str) -> String {
    let model = MODEL.replace(POLICY, &format!("    on_failure:\n{policy}"));
    assert_ne!(
        model, MODEL,
        "the fixture carries the policy this test rewrites"
    );
    model
}

fn ir_of(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("refusal-policy.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}"));
    compile(&spec, &SourceMap::new()).expect("the model compiles")
}

fn synthesis_of(text: &str) -> ess_conformance::synthesize::Synthesis {
    ess_conformance::synthesize::synthesize(&ir_of(text))
}

fn scenario<'a>(suite: &'a ConformanceSuite, id: &str) -> &'a ConformanceScenario {
    suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map_or_else(
            || {
                panic!(
                    "no scenario {id}; the suite holds:\n{}",
                    suite
                        .scenarios
                        .keys()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            },
            |(_, scenario)| scenario,
        )
}

/// What a scenario forces, counts and requires of the escalation event.
fn shape(scenario: &ConformanceScenario) -> (Vec<String>, Vec<u32>, Vec<String>) {
    let mut forced = Vec::new();
    let mut counted = Vec::new();
    let mut events = Vec::new();
    for step in &scenario.steps {
        match step {
            ScenarioStep::ConfigureExternalOutcome { force, times } => forced.push(format!(
                "{} x{}",
                force.outcome,
                times.map_or(1, NonZeroU32::get)
            )),
            ScenarioStep::ExpectInvocation { count, .. } => {
                counted.push(count.map_or(0, NonZeroU32::get));
            }
            ScenarioStep::EventuallyEvent { event, .. } if event.to_string() == ESCALATED => {
                events.push("escalated".to_owned());
            }
            ScenarioStep::EventuallyEvent { event, .. } if event.to_string() == RECORDED => {
                events.push("recorded".to_owned());
            }
            ScenarioStep::ExpectNoPublication { event } if event.to_string() == ESCALATED => {
                events.push("not escalated".to_owned());
            }
            ScenarioStep::ExpectPublicationCount { event, count }
                if event.to_string() == ESCALATED =>
            {
                events.push(format!("escalated x{count}"));
            }
            _ => {}
        }
    }
    (forced, counted, events)
}

// ---- synthesis -----------------------------------------------------------------------------------

#[test]
fn synthesis_witnesses_every_selected_refusal_on_its_own_scenario_in_suite_36() {
    let synthesis = synthesis_of(MODEL);
    let about_binding: Vec<String> = synthesis
        .refusals
        .iter()
        .filter(|refusal| refusal.to_string().contains(BINDING))
        .map(ToString::to_string)
        .collect();
    assert_eq!(
        about_binding.len(),
        0,
        "nothing is refused: {about_binding:#?}"
    );
    assert_eq!(
        synthesis.suite.provenance.suite_version.to_string(),
        "ess-conformance/36"
    );
    let ids: Vec<String> = synthesis
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect();
    assert!(
        !ids.contains(&format!("{BINDING}/binding/on-failure")),
        "the universal claim is split per refusal: {ids:#?}"
    );
    for (outcome, forced, counted, events) in [
        ("unavailable", "unavailable x3", 3, vec!["not escalated"]),
        ("busy", "busy x3", 3, vec!["not escalated"]),
        ("rejected", "rejected x1", 1, vec!["not escalated"]),
        (
            "at-limit",
            "at-limit x1",
            1,
            vec!["escalated", "escalated x1"],
        ),
        ("wrong-state", "wrong-state x1", 1, vec!["not escalated"]),
    ] {
        let scenario = scenario(&synthesis.suite, &id(outcome));
        assert_eq!(
            shape(scenario),
            (
                vec![forced.to_owned()],
                vec![counted],
                events.into_iter().map(str::to_owned).collect()
            ),
            "{outcome}: {:#?}",
            scenario.steps
        );
    }
    for aspect in ["flow", "mapping", "delivery"] {
        assert!(
            ids.contains(&format!("{BINDING}/binding/{aspect}")),
            "{aspect}"
        );
    }
}

#[test]
fn an_unbounded_selected_retry_is_witnessed_retrying_to_success() {
    let model = with_policy(
        "      drop: [wrong-state]\n      retry: [unavailable, busy, rejected]\n      escalate: {emits: demo.ledger.RecordEscalated, except: [wrong-state, unavailable, busy, rejected]}\n",
    );
    let synthesis = synthesis_of(&model);
    let scenario = scenario(&synthesis.suite, &id("busy"));
    assert_eq!(
        shape(scenario),
        (
            vec!["busy x1".to_owned()],
            // No count: as the universal `retry`, an unbounded retry promises the consequence.
            vec![],
            vec!["recorded".to_owned(), "not escalated".to_owned()]
        ),
        "{:#?}",
        scenario.steps
    );
    let statuses = run(&synthesis.suite, &Interpreted::for_model(ir_of(&model)));
    assert_eq!(not_passed(&statuses).len(), 0, "{statuses:#?}");
}

#[test]
fn a_refusal_no_scenario_can_force_is_a_named_coverage_limitation() {
    let model = MODEL.replace(
        "        external: the order is already recorded\n",
        "        when: order_id == done\n",
    );
    assert_ne!(model, MODEL);
    let synthesis = synthesis_of(&model);
    let refused: Vec<String> = synthesis
        .refusals
        .iter()
        .filter(|refusal| {
            refusal
                .scenario
                .as_ref()
                .is_some_and(|scenario| scenario.to_string() == id("wrong-state"))
        })
        .map(ToString::to_string)
        .collect();
    assert_eq!(refused.len(), 1, "{:#?}", synthesis.refusals);
    assert!(refused[0].contains("wrong-state"), "{refused:#?}");
    assert!(!synthesis
        .suite
        .scenarios
        .keys()
        .any(|key| key.to_string() == id("wrong-state")));
    for outcome in ["unavailable", "busy", "rejected", "at-limit"] {
        scenario(&synthesis.suite, &id(outcome));
    }
}

#[test]
fn the_refusal_scenario_round_trips_and_an_older_reader_refuses_it() {
    let suite = synthesis_of(MODEL).suite;
    let json = suite.to_canonical_json().expect("the suite serialises");
    let admitted = AdmittedSuite::from_json(&json).unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(admitted.suite(), &suite);
    // /34: the newest earlier ordinary major, so that only the vocabulary is out of date. The
    // `expect_no_publication` and `expect_publication_count` steps are refused first, as any step
    // a major cannot carry is.
    let older = json.replace("\"ess-conformance/36\"", "\"ess-conformance/34\"");
    let error = AdmittedSuite::from_json(&older).expect_err("suite/34 has no refusal scenario");
    assert!(
        error.to_string().contains("UnsupportedVocabulary"),
        "{error}"
    );
    // Without those steps, the scenario id alone is still refused, naming its format.
    let mut document: serde_json::Value = serde_json::from_str(&older).unwrap();
    for scenario in document["scenarios"].as_object_mut().unwrap().values_mut() {
        scenario["steps"].as_array_mut().unwrap().retain(|step| {
            step["step"] != "expect_no_publication" && step["step"] != "expect_publication_count"
        });
    }
    let error = AdmittedSuite::from_json(&document.to_string())
        .expect_err("suite/34 has no refusal scenario id");
    assert!(error.to_string().contains("suite/36"), "{error}");
    let mut pinned = suite;
    pinned.provenance.suite_version =
        ess_conformance::scenario::SuiteFormat::parse("ess-conformance/34").unwrap();
    assert!(AdmittedSuite::from_suite(&pinned).is_err());
}

#[test]
fn the_scenario_player_refuses_the_selected_policy_by_name() {
    // Its closed model names one failure word per binding; the fallback's would be a claim about
    // every refusal, so the page is not emitted.
    let ir = ir_of(MODEL);
    let suite = synthesis_of(MODEL).suite;
    let error = ess_conformance::web::emit(&ir, &suite).expect_err("refused");
    assert!(
        error
            .to_string()
            .contains("$model.bindings.notify-ledger.on_failure"),
        "{error}"
    );
}

#[test]
fn coverage_for_the_fixture_is_37() {
    let input = ess_conformance::coverage_build::build(
        &ir_of(MODEL),
        &[],
        ess_conformance::coverage::Scope::System,
        ess_conformance::coverage::Origins::Generated,
    )
    .unwrap_or_else(|error| panic!("{error}"));
    let original = input.selected().original_json().to_owned();
    assert!(original.contains("\"ess-conformance/37\""), "{original}");
    assert!(original.contains(&id("at-limit")), "{original}");
}

#[test]
fn a_universal_policy_keeps_its_scenarios_and_suite_format() {
    let synthesis = synthesis_of(&MODEL.replace(POLICY, "    on_failure: retry\n"));
    let ids: Vec<String> = synthesis
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect();
    assert!(
        ids.contains(&format!("{BINDING}/binding/on-failure")),
        "{ids:#?}"
    );
    assert!(
        !ids.iter().any(|id| id.contains("/binding/refusal/")),
        "{ids:#?}"
    );
    assert_eq!(
        synthesis.suite.provenance.suite_version.to_string(),
        "ess-conformance/34"
    );
}

// ---- the suite held against healthy and faulty senders -------------------------------------------

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

#[test]
fn the_binding_running_native_fixture_passes_every_scenario() {
    let synthesis = synthesis_of(MODEL);
    let statuses = run(&synthesis.suite, &Interpreted::for_model(ir_of(MODEL)));
    for outcome in REFUSALS {
        assert_eq!(
            statuses[&id(outcome)],
            Status::Passed,
            "{outcome}: {statuses:#?}"
        );
    }
    assert_eq!(not_passed(&statuses).len(), 0, "{statuses:#?}");
}

#[test]
fn the_healthy_sender_passes_and_each_faulty_one_fails_the_scenario_it_breaks() {
    let suite = synthesis_of(MODEL).suite;
    for (mode, caught) in [
        (Mode::Correct, vec![]),
        (
            Mode::FallbackEverywhere,
            vec!["busy", "rejected", "unavailable", "wrong-state"],
        ),
        (Mode::SwappedPolicy, vec!["at-limit", "wrong-state"]),
        (Mode::ExtraRetry, vec!["busy", "unavailable"]),
        (Mode::DuplicateEscalation, vec!["at-limit"]),
        (Mode::OmitsAttempt, vec!["busy", "unavailable"]),
        (Mode::RetriesFinal, vec!["rejected"]),
        (Mode::RetriesDrop, vec!["wrong-state"]),
        // Only a sequence of different refusals tells it apart; see the runtime control below.
        (Mode::ResetsBudget, vec![]),
    ] {
        let statuses = run(&suite, &Ledger::new(mode));
        let expected: Vec<String> = caught.iter().map(|outcome| id(outcome)).collect();
        assert_eq!(not_passed(&statuses), expected, "{mode:?}: {statuses:#?}");
        for id in &expected {
            assert_eq!(statuses[id], Status::Failed, "{mode:?}: {id}");
        }
    }
}

#[test]
fn the_native_fixture_running_a_faulty_policy_is_caught_too() {
    let suite = synthesis_of(MODEL).suite;
    for (faulty, caught) in [
        // Omitted policy: the fallback applied to every refusal.
        (
            "      escalate: {emits: demo.ledger.RecordEscalated}\n",
            vec!["busy", "rejected", "unavailable", "wrong-state"],
        ),
        // Wrong policy for two refusals.
        (
            "      drop: [at-limit]\n      retry: {outcomes: [demo.ledger.Unavailable, rejected], attempts: 3, final: [rejected]}\n      escalate: {emits: demo.ledger.RecordEscalated, except: [at-limit, demo.ledger.Unavailable, rejected]}\n",
            vec!["at-limit", "wrong-state"],
        ),
        // An extra retry.
        (
            "      drop: [wrong-state]\n      retry: {outcomes: [demo.ledger.Unavailable, rejected], attempts: 4, final: [rejected]}\n      escalate: {emits: demo.ledger.RecordEscalated, except: [wrong-state, demo.ledger.Unavailable, rejected]}\n",
            vec!["busy", "unavailable"],
        ),
        // A final refusal retried.
        (
            "      drop: [wrong-state]\n      retry: {outcomes: [demo.ledger.Unavailable, rejected], attempts: 3}\n      escalate: {emits: demo.ledger.RecordEscalated, except: [wrong-state, demo.ledger.Unavailable, rejected]}\n",
            vec!["rejected"],
        ),
    ] {
        let model = if faulty.starts_with("      escalate: {emits: demo.ledger.RecordEscalated}") {
            MODEL.replace(
                POLICY,
                "    on_failure:\n      escalate:\n        emits: demo.ledger.RecordEscalated\n",
            )
        } else {
            with_policy(faulty)
        };
        let statuses = run(&suite, &Interpreted::for_model(ir_of(&model)));
        let expected: Vec<String> = caught.iter().map(|outcome| id(outcome)).collect();
        assert_eq!(not_passed(&statuses), expected, "{faulty}: {statuses:#?}");
    }
}

#[test]
fn a_conditioned_selected_binding_keeps_zero_invocations_under_every_policy() {
    let model = MODEL
        .replace(
            "      - {name: order_id, type: demo.ledger.OrderId}\n  - name: demo.ledger.Recorded\n",
            "      - {name: order_id, type: demo.ledger.OrderId}\n      - {name: note, type: Optional<String>}\n  - name: demo.ledger.Recorded\n",
        )
        .replace(
            "  - name: demo.ledger.Place\n    input:\n      - {name: order_id, type: demo.ledger.OrderId}\n",
            "  - name: demo.ledger.Place\n    input:\n      - {name: order_id, type: demo.ledger.OrderId}\n      - {name: note, type: Optional<String>}\n",
        )
        .replace(
            "          demo.ledger.OrderPlaced: {order_id: input.order_id}\n",
            "          demo.ledger.OrderPlaced: {order_id: input.order_id, note: input.note}\n",
        )
        .replace(
            "      event: demo.ledger.OrderPlaced\n",
            "      event: demo.ledger.OrderPlaced\n      where: defined(event.note)\n",
        );
    let synthesis = synthesis_of(&model);
    let ids: Vec<String> = synthesis
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect();
    assert!(
        ids.contains(&format!("{BINDING}/binding/condition-absent")),
        "{ids:#?}"
    );
    for outcome in REFUSALS {
        assert!(ids.contains(&id(outcome)), "{outcome}: {ids:#?}");
    }
    let statuses = run(&synthesis.suite, &Interpreted::for_model(ir_of(&model)));
    assert_eq!(not_passed(&statuses).len(), 0, "{statuses:#?}");
    // A scripted refusal on every port answer changes nothing when the condition is false: the
    // binding is never invoked, under every policy the table holds.
    let control = Control::new(Interpreted::for_model(ir_of(&model)));
    for outcome in REFUSALS {
        let run = control.place_with("o-1", None, &[outcome.into()]);
        assert_eq!(run.invocations, 0, "{outcome}: {run:?}");
        assert_eq!(run.escalated.len(), 0, "{outcome}: {run:?}");
    }
}

// ---- runtime controls: a scripted port, the interpreter and the in-memory sender ---------------

/// One answer of the scripted port, written as an outcome name or `untyped`.
#[derive(Clone, Debug)]
struct Scripted(String);

impl From<&str> for Scripted {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

/// What one placed order came to.
#[derive(Debug)]
struct Placed {
    result: Result<(), String>,
    invocations: usize,
    inputs: Vec<Node>,
    recorded: usize,
    escalated: Vec<Node>,
}

/// A dispatcher the controls drive: the interpreter, or the in-memory sender in some mode.
trait Scriptable: ConformanceTarget {
    fn script(&self, answers: &[Scripted]);
}

impl Scriptable for Interpreted {
    fn script(&self, answers: &[Scripted]) {
        self.script_binding_port(
            &CommandRef::new(RECORD.parse().unwrap()),
            answers.iter().map(|Scripted(answer)| {
                if answer == "untyped" {
                    PortAnswer::Untyped
                } else {
                    PortAnswer::Outcome(answer.parse().unwrap())
                }
            }),
        );
    }
}

struct Control<T: Scriptable> {
    target: T,
}

impl<T: Scriptable> Control<T> {
    fn new(target: T) -> Self {
        Self { target }
    }

    fn context() -> ScenarioContext {
        ScenarioContext::new(
            "demo.ledger/authored/refusal-policy".parse().unwrap(),
            CorrelationId::new("refusal-policy").unwrap(),
        )
    }

    fn place(&self, answers: &[&str]) -> Placed {
        self.place_with(
            "o-1",
            None,
            &answers
                .iter()
                .map(|answer| (*answer).into())
                .collect::<Vec<Scripted>>(),
        )
    }

    fn place_with(&self, order_id: &str, note: Option<Node>, answers: &[Scripted]) -> Placed {
        self.target.begin_scenario(&Self::context()).unwrap();
        self.target.script(answers);
        let mut input = BTreeMap::from([("order_id".to_owned(), Node::Text(order_id.into()))]);
        if let Some(note) = note {
            input.insert("note".to_owned(), note);
        }
        let result = self
            .target
            .execute_command(SemanticCommandRequest {
                command: CommandRef::new(PLACE.parse().unwrap()),
                actor: None,
                caller: None,
                input,
                correlation: Self::context().correlation,
            })
            .map(|_| ())
            .map_err(|error| error.to_string());
        let invocations = self
            .target
            .observe_invocations(InvocationObservationRequest {
                binding: BindingRef::new(ess_domain::binding::BindingName::new(BINDING).unwrap()),
                command: CommandRef::new(RECORD.parse().unwrap()),
                correlation: Self::context().correlation,
                deadline: Deadline::at(Timestamp::from_epoch_millis(0)),
            })
            .unwrap();
        let events = |name: &str| {
            self.target
                .observe_events(EventObservationRequest {
                    event: name.parse().unwrap(),
                    correlation: Self::context().correlation,
                    deadline: Deadline::at(Timestamp::from_epoch_millis(0)),
                })
                .unwrap()
        };
        Placed {
            result,
            invocations: invocations.len(),
            inputs: invocations
                .iter()
                .map(|invocation| invocation.input["order_id"].clone())
                .collect(),
            recorded: events(RECORDED).len(),
            escalated: events(ESCALATED)
                .into_iter()
                .map(|event| event.payload["order_id"].clone())
                .collect(),
        }
    }
}

/// Every named control the design lists that one dispatcher can be held to, with whether it held.
fn controls<T: Scriptable>(target: T) -> BTreeMap<&'static str, Result<(), String>> {
    let control = Control::new(target);
    let mut held = BTreeMap::new();
    let mut check = |name: &'static str, placed: &Placed, ok: bool| {
        held.insert(
            name,
            if ok && placed.result.is_ok() {
                Ok(())
            } else {
                Err(format!("{placed:?}"))
            },
        );
    };
    let placed = control.place(&["wrong-state"]);
    check(
        "selected_drop_once",
        &placed,
        placed.invocations == 1 && placed.recorded == 0 && placed.escalated.is_empty(),
    );
    let placed = control.place(&["at-limit"]);
    check(
        "selected_escalation_once",
        &placed,
        placed.invocations == 1 && placed.recorded == 0 && placed.escalated.len() == 1,
    );
    let placed = control.place(&["unavailable"]);
    check(
        "selected_retry_to_success",
        &placed,
        placed.invocations == 2 && placed.recorded == 1 && placed.escalated.is_empty(),
    );
    // Three different refusals in one retry: the total, not the run of one refusal, is the budget.
    let placed = control.place(&["unavailable", "busy", "unavailable", "busy"]);
    check(
        "selected_retry_exhausts_total_budget",
        &placed,
        placed.invocations == 3 && placed.recorded == 0 && placed.escalated.is_empty(),
    );
    let placed = control.place(&["busy", "rejected"]);
    check(
        "selected_retry_final_stops",
        &placed,
        placed.invocations == 2 && placed.recorded == 0 && placed.escalated.is_empty(),
    );
    let placed = control.place(&["busy", "busy", "busy"]);
    check(
        "outcome_aliases_expand_to_same_policy",
        &placed,
        placed.invocations == 3 && placed.recorded == 0 && placed.escalated.is_empty(),
    );
    let placed = control.place(&["untyped"]);
    check(
        "unknown_failure_uses_explicit_fallback",
        &placed,
        placed.invocations == 1 && placed.recorded == 0 && placed.escalated.len() == 1,
    );
    let placed = control.place(&["unavailable", "untyped"]);
    check(
        "fallback_escalation_uses_actual_complete_input",
        &placed,
        placed.invocations == 2
            && placed.escalated.len() == 1
            && placed.inputs.last() == placed.escalated.first(),
    );
    held
}

#[test]
fn the_native_fixture_holds_every_named_control() {
    let held = controls(Interpreted::for_model(ir_of(MODEL)));
    for (name, result) in &held {
        assert!(result.is_ok(), "{name}: {result:?}");
    }
    assert_eq!(held.len(), 8, "{held:#?}");
}

#[test]
fn the_healthy_sender_holds_every_named_control_and_each_fault_breaks_one() {
    for (mode, broken) in [
        (Mode::Correct, vec![]),
        (
            Mode::FallbackEverywhere,
            vec![
                "fallback_escalation_uses_actual_complete_input",
                "outcome_aliases_expand_to_same_policy",
                "selected_drop_once",
                "selected_retry_exhausts_total_budget",
                "selected_retry_final_stops",
                "selected_retry_to_success",
            ],
        ),
        (
            Mode::ResetsBudget,
            vec!["selected_retry_exhausts_total_budget"],
        ),
        (
            Mode::OmitsAttempt,
            vec![
                "fallback_escalation_uses_actual_complete_input",
                "outcome_aliases_expand_to_same_policy",
                "selected_retry_exhausts_total_budget",
                "selected_retry_final_stops",
                "selected_retry_to_success",
            ],
        ),
        (
            Mode::DuplicateEscalation,
            vec![
                "fallback_escalation_uses_actual_complete_input",
                "selected_escalation_once",
                "unknown_failure_uses_explicit_fallback",
            ],
        ),
    ] {
        let held = controls(Ledger::new(mode));
        let failed: Vec<&str> = held
            .iter()
            .filter(|(_, result)| result.is_err())
            .map(|(name, _)| *name)
            .collect();
        assert_eq!(failed, broken, "{mode:?}: {held:#?}");
    }
}

#[test]
fn untyped_port_failure_consumes_attempt_budget() {
    // A bounded fallback: three untyped failures, or untyped failures mixed with a refusal the
    // fallback also selects, are three attempts and then the event's effect is lost.
    let model = with_policy(
        "      drop: [wrong-state, at-limit]\n      retry: {except: [wrong-state, at-limit], attempts: 3, final: [rejected]}\n",
    );
    let control = Control::new(Interpreted::for_model(ir_of(&model)));
    for answers in [
        vec!["untyped", "untyped", "untyped", "untyped"],
        vec!["untyped", "busy", "untyped", "untyped"],
    ] {
        let placed = control.place(&answers);
        assert!(placed.result.is_ok(), "{answers:?}: {placed:?}");
        assert_eq!(placed.invocations, 3, "{answers:?}: {placed:?}");
        assert_eq!(placed.recorded, 0, "{answers:?}: {placed:?}");
    }
    let placed = control.place(&["untyped", "rejected"]);
    assert_eq!(
        placed.invocations, 2,
        "a final refusal stops the fallback retry: {placed:?}"
    );
    let placed = control.place(&["untyped", "at-limit"]);
    assert_eq!(
        placed.invocations, 2,
        "a dropped refusal ends it: {placed:?}"
    );
}

#[test]
fn pre_input_failure_is_an_obligation() {
    // The ledger keys a record by digits only, converted from the order's id: an order id with a
    // letter cannot become the command's input, so the binding never reaches its port.
    let model = MODEL
        .replace(
            "  - {name: demo.ledger.OrderId, kind: newtype, of: String}\n",
            "  - {name: demo.ledger.OrderId, kind: newtype, of: String}\n  - {name: demo.ledger.LedgerKey, kind: newtype, of: String, alphabet: \"0123456789\"}\nconversions:\n  - from: demo.ledger.OrderId\n    to: demo.ledger.LedgerKey\n    because: the ledger keys a record by the order's digits\n",
        )
        .replace(
            "  - name: demo.ledger.Record\n    input:\n      - {name: order_id, type: demo.ledger.OrderId}\n",
            "  - name: demo.ledger.Record\n    input:\n      - {name: order_id, type: demo.ledger.LedgerKey}\n",
        )
        .replace(
            "  - name: demo.ledger.Recorded\n    fields:\n      - {name: order_id, type: demo.ledger.OrderId}\n  - name: demo.ledger.RecordEscalated\n    fields:\n      - {name: order_id, type: demo.ledger.OrderId}\n",
            "  - name: demo.ledger.Recorded\n    fields:\n      - {name: order_id, type: demo.ledger.LedgerKey}\n  - name: demo.ledger.RecordEscalated\n    fields:\n      - {name: order_id, type: demo.ledger.LedgerKey}\n",
        );
    let control = Control::new(Interpreted::for_model(ir_of(&model)));
    let placed = control.place_with("o-1", None, &["untyped".into()]);
    assert!(
        placed
            .result
            .as_ref()
            .is_err_and(|error| error.contains("input")),
        "an unmet binding-input obligation, never a skip: {placed:?}"
    );
    assert_eq!(placed.invocations, 0, "zero attempts: {placed:?}");
    assert_eq!(placed.escalated.len(), 0, "no policy runs: {placed:?}");
    let placed = control.place_with("17", None, &["untyped".into()]);
    assert!(placed.result.is_ok(), "{placed:?}");
    assert_eq!(placed.invocations, 1, "{placed:?}");
    assert_eq!(
        placed.escalated,
        vec![Node::Text("17".into())],
        "{placed:?}"
    );
}

#[test]
fn an_escalation_builder_failure_publishes_nothing_and_does_not_retry() {
    // The escalation event's field takes digits only, and the actual input carries a letter: the
    // typed builder cannot construct the payload from the complete input it was handed.
    let model = MODEL
        .replace(
            "  - {name: demo.ledger.OrderId, kind: newtype, of: String}\n",
            "  - {name: demo.ledger.OrderId, kind: newtype, of: String}\n  - {name: demo.ledger.LedgerKey, kind: newtype, of: String, alphabet: \"0123456789\"}\n",
        )
        .replace(
            "  - name: demo.ledger.RecordEscalated\n    fields:\n      - {name: order_id, type: demo.ledger.OrderId}\n",
            "  - name: demo.ledger.RecordEscalated\n    fields:\n      - {name: order_id, type: demo.ledger.LedgerKey}\n",
        );
    let control = Control::new(Interpreted::for_model(ir_of(&model)));
    let placed = control.place(&["at-limit"]);
    assert!(placed.result.is_err(), "a reported obligation: {placed:?}");
    assert_eq!(placed.invocations, 1, "no reentry into retry: {placed:?}");
    assert_eq!(placed.escalated.len(), 0, "no partial event: {placed:?}");
    assert_eq!(placed.recorded, 0, "{placed:?}");
}

// ---- the in-memory sender, healthy and in each faulty way ---------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mode {
    /// The binding as the table says.
    Correct,
    /// The fallback applied to every refusal: the selected policy omitted.
    FallbackEverywhere,
    /// `wrong-state` escalated and `at-limit` dropped: the wrong policy chosen.
    SwappedPolicy,
    /// Four attempts where the bound is three.
    ExtraRetry,
    /// Escalates, then tries again and escalates again.
    DuplicateEscalation,
    /// Records only the first attempt it makes.
    OmitsAttempt,
    /// Retries the final refusal.
    RetriesFinal,
    /// Retries the dropped refusal.
    RetriesDrop,
    /// Restarts the count whenever the refusal changes.
    ResetsBudget,
}

#[derive(Default)]
struct State {
    forced: Option<(String, u32)>,
    script: VecDeque<String>,
    log: Vec<ObservedEvent>,
    invocations: Vec<ObservedInvocation>,
}

struct Ledger {
    mode: Mode,
    state: RefCell<State>,
}

#[derive(PartialEq, Eq)]
enum Policy {
    Drop,
    Retry,
    Escalate,
}

impl Ledger {
    fn new(mode: Mode) -> Self {
        Self {
            mode,
            state: RefCell::new(State::default()),
        }
    }

    fn event(name: &str, order_id: &Node) -> ObservedEvent {
        ObservedEvent::new(name.parse().unwrap()).with("order_id", order_id.clone())
    }

    fn outcome(command: &str, outcome: &str) -> OutcomeRef {
        OutcomeRef::new(
            CommandRef::new(command.parse().unwrap()),
            outcome.parse().unwrap(),
        )
    }

    /// The next answer of `Record`: the scripted one, a forced one, or `recorded`.
    fn answer(state: &mut State) -> String {
        if let Some(answer) = state.script.pop_front() {
            return answer;
        }
        match state.forced.as_mut() {
            Some((outcome, remaining)) if *remaining > 0 => {
                *remaining -= 1;
                outcome.clone()
            }
            _ => "recorded".to_owned(),
        }
    }

    fn policy(&self, answer: &str) -> Policy {
        if self.mode == Mode::FallbackEverywhere {
            return Policy::Escalate;
        }
        match answer {
            "wrong-state" if self.mode == Mode::SwappedPolicy => Policy::Escalate,
            "wrong-state" if self.mode == Mode::RetriesDrop => Policy::Retry,
            "wrong-state" => Policy::Drop,
            "unavailable" | "busy" | "rejected" => Policy::Retry,
            "at-limit" if self.mode == Mode::SwappedPolicy => Policy::Drop,
            _ => Policy::Escalate,
        }
    }

    /// The binding: invoke `Record`, and answer each failure with the policy its refusal selects.
    fn notify(&self, state: &mut State, placed: &ObservedEvent) {
        let order_id = placed.payload["order_id"].clone();
        let bound = if self.mode == Mode::ExtraRetry { 4 } else { 3 };
        let mut attempts = 0_u32;
        let mut run = 0_u32;
        let mut last = String::new();
        while attempts < 64 {
            attempts += 1;
            if self.mode != Mode::OmitsAttempt || attempts == 1 {
                state.invocations.push(
                    ObservedInvocation::new(
                        BindingRef::new(ess_domain::binding::BindingName::new(BINDING).unwrap()),
                        CommandRef::new(RECORD.parse().unwrap()),
                    )
                    .with("order_id", order_id.clone()),
                );
            }
            let answer = Self::answer(state);
            if answer == "recorded" {
                state.log.push(Self::event(RECORDED, &order_id));
                return;
            }
            run = if answer == last { run + 1 } else { 1 };
            last.clone_from(&answer);
            match self.policy(&answer) {
                Policy::Drop => return,
                Policy::Escalate => {
                    state.log.push(Self::event(ESCALATED, &order_id));
                    if self.mode != Mode::DuplicateEscalation {
                        return;
                    }
                }
                Policy::Retry => {
                    if answer == "rejected" && self.mode != Mode::RetriesFinal {
                        return;
                    }
                    let used = if self.mode == Mode::ResetsBudget {
                        run
                    } else {
                        attempts
                    };
                    if used >= bound {
                        return;
                    }
                }
            }
        }
    }
}

impl Scriptable for Ledger {
    fn script(&self, answers: &[Scripted]) {
        self.state.borrow_mut().script = answers.iter().map(|Scripted(a)| a.clone()).collect();
    }
}

impl ConformanceTarget for Ledger {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("refusal-policy", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        *self.state.borrow_mut() = State::default();
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let mut state = self.state.borrow_mut();
        let order_id = request.input.get("order_id").cloned().unwrap_or(Node::Null);
        match request.command.to_string().as_str() {
            PLACE => {
                let placed = Self::event("demo.ledger.OrderPlaced", &order_id);
                state.log.push(placed.clone());
                let mut result = SemanticCommandResult::took(Self::outcome(PLACE, "placed"));
                result.direct_events.push(placed.clone());
                self.notify(&mut state, &placed);
                Ok(result)
            }
            RECORD => {
                let answer = Self::answer(&mut state);
                let mut result = SemanticCommandResult::took(Self::outcome(RECORD, &answer));
                let error = match answer.as_str() {
                    "recorded" => {
                        let event = Self::event(RECORDED, &order_id);
                        state.log.push(event.clone());
                        result.direct_events.push(event);
                        None
                    }
                    "unavailable" | "busy" => Some("demo.ledger.Unavailable"),
                    "rejected" => Some("demo.ledger.Unknown"),
                    "at-limit" => Some("demo.ledger.AtLimit"),
                    _ => Some("demo.ledger.WrongState"),
                };
                result.error = error.map(|error| DeclaredErrorValue::new(error.parse().unwrap()));
                Ok(result)
            }
            other => panic!("unexpected command {other}"),
        }
    }
    fn query_view(&self, _: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        Err(TargetError::unsupported("views", "the model declares none"))
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Ok(self
            .state
            .borrow()
            .log
            .iter()
            .filter(|event| event.event == request.event)
            .cloned()
            .collect())
    }
    fn configure_external_outcome(
        &self,
        request: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        self.state.borrow_mut().forced = Some((request.force.outcome.to_string(), 1));
        Ok(())
    }
    fn configure_external_outcome_repeatedly(
        &self,
        request: ExternalOutcomeControl,
        times: NonZeroU32,
    ) -> Result<(), TargetError> {
        self.state.borrow_mut().forced = Some((request.force.outcome.to_string(), times.get()));
        Ok(())
    }
    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        let mut state = self.state.borrow_mut();
        let placed = state
            .log
            .iter()
            .rev()
            .find(|event| event.event == request.event)
            .cloned()
            .expect("the event was published");
        self.notify(&mut state, &placed);
        Ok(())
    }
    fn observe_invocations(
        &self,
        request: InvocationObservationRequest,
    ) -> Result<Vec<ObservedInvocation>, TargetError> {
        Ok(self
            .state
            .borrow()
            .invocations
            .iter()
            .filter(|invocation| {
                invocation.binding == request.binding && invocation.command == request.command
            })
            .cloned()
            .collect())
    }
}

// ---- the generated Go runtime gives the native verdicts, healthy and faulty --------------------

mod support_go;

#[test]
fn go_gives_the_native_verdicts_for_every_sender() {
    let suite = synthesis_of(MODEL).suite;
    let verdicts = support_go::assert_parity(
        "refusal-policy-native",
        &suite,
        Interpreted::for_model(ir_of(MODEL)),
    );
    assert_eq!(support_go::not_passed(&verdicts).len(), 0, "{verdicts:#?}");
    for (mode, caught) in [
        (Mode::Correct, vec![]),
        (
            Mode::FallbackEverywhere,
            vec!["busy", "rejected", "unavailable", "wrong-state"],
        ),
        (Mode::SwappedPolicy, vec!["at-limit", "wrong-state"]),
        (Mode::ExtraRetry, vec!["busy", "unavailable"]),
        (Mode::DuplicateEscalation, vec!["at-limit"]),
        (Mode::OmitsAttempt, vec!["busy", "unavailable"]),
        (Mode::RetriesFinal, vec!["rejected"]),
        (Mode::RetriesDrop, vec!["wrong-state"]),
    ] {
        let verdicts = support_go::assert_parity(
            &format!("refusal-policy-{mode:?}").to_lowercase(),
            &suite,
            Ledger::new(mode),
        );
        let expected: Vec<String> = caught.iter().map(|outcome| id(outcome)).collect();
        assert_eq!(support_go::not_passed(&verdicts), expected, "{mode:?}");
    }
}

#[test]
fn go_refuses_a_refusal_scenario_below_suite_36_before_any_target_is_reached() {
    let suite = synthesis_of(MODEL).suite;
    let directory = support_go::package(
        "refusal-policy-old-reader",
        &suite,
        &[support_go::TRANSCRIPT_TARGET],
    );
    support_go::rewrite_suite(&directory, |document| {
        document["provenance"]["suite_version"] = "ess-conformance/34".into();
    });
    // An empty transcript: admission refuses the document before any target is asked anything.
    let transcript = directory.join("transcript.json");
    std::fs::write(&transcript, "{}").unwrap();
    let go = support_go::go_test(
        &directory,
        "TestTranscriptReplay",
        &[
            ("ESS_TRANSCRIPT", transcript.to_str().unwrap()),
            (
                "ESS_TRANSCRIPT_DIVERGENCE",
                directory.join("divergence.txt").to_str().unwrap(),
            ),
        ],
    );
    assert!(go.log.contains("requires suite/36"), "{}", go.log);
    assert!(go.outcomes.is_empty() && !go.success, "{}", go.log);
    std::fs::remove_dir_all(directory).unwrap();
}
