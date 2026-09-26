//! A guard over the addressed entity's stored fields is witnessed against an arranged row (ess#75).
//!
//! `docs/design/cross-record-and-stored-field-guards.md`, "Conformance: arranging a row to a goal".
//! The refusal is witnessed with an Express parcel of 21 kg the suite created on purpose, the
//! success at the boundary, and an identity no row carries is sent once and asserted to create
//! nothing. Each mutant below is one of the implementations the page names, and each must fail the
//! scenario the page says catches it.
use ess_compiler::{
    ir::EssIr,
    refs::{CommandRef, OutcomeRef},
    resolve::compile,
    source::SourceMap,
};
use ess_conformance::{
    report::{ConformanceStatus, Status},
    scenario::ViewExpectation,
    synthesize::Synthesis,
    target::*,
    AdmittedSuite, ConformanceScenario, ConformanceSuite, Runner, ScenarioStep, ScenarioValue,
};
use ess_domain::{
    command::OutcomeName,
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::node::Node;
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
};

const PARCELS: &str = include_str!("fixtures/stored-field-guards.yaml");
const REFUSAL: &str = "shipping.parcel.Dispatch/outcome/refused-overweight";
const SUCCESS: &str = "shipping.parcel.Dispatch/outcome/dispatched";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("parcels.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn synthesis(text: &str) -> Synthesis {
    ess_conformance::synthesize::synthesize(&ir(text))
}

fn scenario<'a>(suite: &'a ConformanceSuite, id: &str) -> &'a ConformanceScenario {
    suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map_or_else(|| panic!("no scenario {id}"), |(_, scenario)| scenario)
}

fn literal(text: &str) -> ScenarioValue {
    ScenarioValue::literal(Node::Text(text.into()))
}

fn number(value: f64) -> ScenarioValue {
    ScenarioValue::literal(Node::Number(
        ess_primitives::facts::Number::new(value).unwrap(),
    ))
}

/// The input every `ExecuteCommand` of `command` sent, in step order.
fn inputs(scenario: &ConformanceScenario, command: &str) -> Vec<BTreeMap<String, ScenarioValue>> {
    scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand {
                command: sent,
                input,
                ..
            } if sent.to_string() == command => Some(input.clone()),
            _ => None,
        })
        .collect()
}

/// Every `Contains` row asserted against `Parcels`, in step order.
fn rows(scenario: &ConformanceScenario) -> Vec<BTreeMap<String, ScenarioValue>> {
    scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExpectView {
                expectation: ViewExpectation::Contains { fields },
                ..
            } => Some(fields.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn the_design_example_synthesizes_with_only_the_undeclared_wrong_state_refusal() {
    let result = synthesis(PARCELS);
    let codes: Vec<String> = result
        .refusals
        .iter()
        .map(|refusal| {
            format!(
                "{} {}",
                refusal.cause.code(),
                refusal
                    .scenario
                    .as_ref()
                    .map_or_else(String::new, ToString::to_string)
            )
        })
        .collect();
    assert_eq!(
        codes,
        ["ESS-SYNTH-012 shipping.parcel.Parcel/state/Dispatched/refuses/shipping.parcel.Dispatch"],
        "{:?}",
        result.refusals
    );
}

#[test]
fn the_refusal_is_witnessed_against_an_express_parcel_of_21_kg() {
    let suite = synthesis(PARCELS).suite;
    let refusal = scenario(&suite, REFUSAL);
    let created = inputs(refusal, "shipping.parcel.Create");
    assert_eq!(
        created,
        [BTreeMap::from([
            ("service".to_owned(), literal("Express")),
            ("weight_kg".to_owned(), number(21.0)),
        ])],
        "the goal is chosen through the create's sets: mapping"
    );
    let observed = BTreeMap::from([
        (
            "parcel_id".to_owned(),
            ScenarioValue::instance("parcel".parse().unwrap()),
        ),
        ("state".to_owned(), literal("Created")),
        ("service".to_owned(), literal("Express")),
        ("weight_kg".to_owned(), number(21.0)),
    ]);
    assert_eq!(
        rows(refusal),
        [observed.clone(), observed],
        "the arranged row is observed before the command and unchanged after it"
    );
    let steps = &refusal.steps;
    let dispatch = steps
        .iter()
        .rposition(|step| matches!(step, ScenarioStep::ExecuteCommand { command, .. } if command.to_string() == "shipping.parcel.Dispatch"))
        .unwrap();
    let ScenarioStep::ExecuteCommand { input, .. } = &steps[dispatch] else {
        unreachable!()
    };
    assert_eq!(
        input.get("parcel_id"),
        Some(&ScenarioValue::instance("parcel".parse().unwrap())),
        "the refusal is sent for the arranged parcel, never an identity no row carries"
    );
    let after: Vec<String> = steps[dispatch + 1..]
        .iter()
        .map(|step| serde_json::to_string(step).unwrap())
        .collect();
    let after = after.join("\n");
    for expected in [
        r#""outcome":{"command":"shipping.parcel.Dispatch","outcome":"refused-overweight"}"#,
        r#""error":"shipping.parcel.ExpressOverweight""#,
        r#"{"step":"expect_no_event","event":"shipping.parcel.Dispatched"}"#,
        r#"{"step":"expect_no_event","event":"shipping.parcel.Created"}"#,
    ] {
        assert!(after.contains(expected), "missing {expected} in:\n{after}");
    }
}

#[test]
fn the_success_is_witnessed_at_the_boundary_the_guard_does_not_refuse() {
    let suite = synthesis(PARCELS).suite;
    let success = scenario(&suite, SUCCESS);
    assert_eq!(
        inputs(success, "shipping.parcel.Create"),
        [
            BTreeMap::from([
                ("service".to_owned(), literal("Express")),
                ("weight_kg".to_owned(), number(20.0)),
            ]),
            BTreeMap::from([
                ("service".to_owned(), literal("Standard")),
                ("weight_kg".to_owned(), number(21.0)),
            ]),
        ],
        "Express at 20 kg, the row nearest the guard, then one row per conjunct refuted alone: \
         Standard at 21 kg refutes only the service (Express at 20 is the first row already)"
    );
    let dispatches = inputs(success, "shipping.parcel.Dispatch");
    assert_eq!(
        dispatches
            .iter()
            .map(|input| input.get("parcel_id").cloned())
            .collect::<Vec<_>>(),
        [
            Some(ScenarioValue::instance("parcel".parse().unwrap())),
            Some(ScenarioValue::instance("parcel-2".parse().unwrap())),
        ],
        "each row is its own instance"
    );
    let text = serde_json::to_string(&success.steps).unwrap();
    for expected in [
        r#""step":"expect_no_error""#,
        r#""event":"shipping.parcel.Dispatched""#,
        r#""state":{"kind":"literal","value":"Dispatched"}"#,
    ] {
        assert!(text.contains(expected), "missing {expected} in {text}");
    }
    let transition = scenario(
        &suite,
        "shipping.parcel.Parcel/transition/dispatch/by/shipping.parcel.Dispatch/dispatched",
    );
    assert_eq!(
        inputs(transition, "shipping.parcel.Create"),
        inputs(success, "shipping.parcel.Create"),
        "the transition scenario reuses the arrangement of the branch that drives it"
    );
}

#[test]
fn an_absent_subject_is_sent_once_and_asserted_to_create_nothing() {
    let suite = synthesis(PARCELS).suite;
    let refusal = scenario(&suite, REFUSAL);
    let steps = &refusal.steps;
    let absent = steps
        .iter()
        .position(|step| {
            matches!(step, ScenarioStep::ExecuteCommand { command, input, .. }
            if command.to_string() == "shipping.parcel.Dispatch"
                && matches!(input.get("parcel_id"), Some(ScenarioValue::Literal { .. })))
        })
        .expect("one invocation with an identity no row carries");
    let ScenarioStep::ExecuteCommand { input, .. } = &steps[absent] else {
        unreachable!()
    };
    let identity = input["parcel_id"].clone();
    let mut rest = steps[absent + 1..].iter();
    let mut forbidden = Vec::new();
    let mut excluded = false;
    for step in rest.by_ref() {
        match step {
            ScenarioStep::ExpectNoEvent { event } => forbidden.push(event.to_string()),
            ScenarioStep::QueryView { view, .. } => {
                assert_eq!(view.to_string(), "shipping.parcel.Parcels");
            }
            ScenarioStep::ExpectView {
                expectation: ViewExpectation::Excludes { fields },
                ..
            } => {
                assert_eq!(
                    fields,
                    &BTreeMap::from([("parcel_id".to_owned(), identity.clone())])
                );
                excluded = true;
                break;
            }
            ScenarioStep::ExpectError { .. } | ScenarioStep::ExpectOutcome { .. } => {
                panic!("the specification declares no absent-subject outcome: {step:?}")
            }
            other => panic!("unexpected step after the absent-subject invocation: {other:?}"),
        }
    }
    forbidden.sort();
    assert_eq!(
        forbidden,
        ["shipping.parcel.Created", "shipping.parcel.Dispatched"]
    );
    assert!(excluded, "the view is required to exclude the identity");
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Behavior {
    /// The rule as specified.
    Good,
    /// Dispatches every parcel: the stored weight and service are never read.
    IgnoresStoredFields,
    /// Refuses every Express parcel: the stored weight is never read.
    IgnoresStoredWeight,
    /// Reads the weight the caller could have sent instead of the stored one; the dispatch input
    /// carries none, so this is "never overweight".
    UpsertsAbsentSubject,
}

struct Parcels {
    behavior: Behavior,
    rows: RefCell<BTreeMap<String, BTreeMap<String, Node>>>,
    minted: Cell<u64>,
}

impl Parcels {
    fn new(behavior: Behavior) -> Self {
        Self {
            behavior,
            rows: RefCell::default(),
            minted: Cell::new(0),
        }
    }
}

fn outcome(command: &CommandRef, name: &str) -> OutcomeRef {
    OutcomeRef::new(command.clone(), OutcomeName::new(name).unwrap())
}

impl ConformanceTarget for Parcels {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("parcels-fixture", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.rows.replace(BTreeMap::new());
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        self.minted.set(self.minted.get() + 1);
        let token = ess_primitives::consistency::ConsistencyToken::new(format!(
            "seq:{}",
            self.minted.get()
        ))
        .unwrap();
        self.answer(&request)
            .map(|result| result.with_consistency(token))
    }
    fn query_view(&self, _: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        Ok(SemanticViewResult::of(self.rows.borrow().values().cloned()))
    }
    fn observe_events(
        &self,
        _: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Ok(Vec::new())
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        panic!("nothing here is externally decided")
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        panic!("no bindings")
    }
}

impl Parcels {
    fn answer(
        &self,
        request: &SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let mut rows = self.rows.borrow_mut();
        let command = request.command.clone();
        match command.to_string().as_str() {
            "shipping.parcel.Create" => {
                self.minted.set(self.minted.get() + 1);
                let id = format!("00000000-0000-4000-8000-{:012}", self.minted.get());
                rows.insert(
                    id.clone(),
                    BTreeMap::from([
                        ("parcel_id".to_owned(), Node::Text(id.clone())),
                        ("state".to_owned(), Node::Text("Created".into())),
                        ("service".to_owned(), request.input["service"].clone()),
                        ("weight_kg".to_owned(), request.input["weight_kg"].clone()),
                    ]),
                );
                Ok(
                    SemanticCommandResult::took(outcome(&command, "created")).emitting(
                        ObservedEvent::new("shipping.parcel.Created".parse().unwrap())
                            .with("parcel_id", Node::Text(id)),
                    ),
                )
            }
            "shipping.parcel.Dispatch" => {
                let Node::Text(id) = &request.input["parcel_id"] else {
                    return Ok(SemanticCommandResult::undeclared());
                };
                let Some(row) = rows.get_mut(id) else {
                    if self.behavior == Behavior::UpsertsAbsentSubject {
                        rows.insert(
                            id.clone(),
                            BTreeMap::from([
                                ("parcel_id".to_owned(), Node::Text(id.clone())),
                                ("state".to_owned(), Node::Text("Dispatched".into())),
                            ]),
                        );
                        return Ok(SemanticCommandResult::took(outcome(&command, "dispatched"))
                            .emitting(ObservedEvent::new(
                                "shipping.parcel.Dispatched".parse().unwrap(),
                            )));
                    }
                    return Ok(SemanticCommandResult::undeclared());
                };
                if row["state"] != Node::Text("Created".into()) {
                    return Ok(SemanticCommandResult::undeclared());
                }
                let express = row["service"] == Node::Text("Express".into());
                let heavy = matches!(&row["weight_kg"], Node::Number(n) if n.get() > 20.0);
                let refuse = match self.behavior {
                    Behavior::Good | Behavior::UpsertsAbsentSubject => express && heavy,
                    Behavior::IgnoresStoredFields => false,
                    Behavior::IgnoresStoredWeight => express,
                };
                if refuse {
                    return Ok(SemanticCommandResult::took(outcome(
                        &command,
                        "refused-overweight",
                    ))
                    .with_error(DeclaredErrorValue::new(
                        "shipping.parcel.ExpressOverweight".parse().unwrap(),
                    )));
                }
                row.insert("state".into(), Node::Text("Dispatched".into()));
                Ok(
                    SemanticCommandResult::took(outcome(&command, "dispatched")).emitting(
                        ObservedEvent::new("shipping.parcel.Dispatched".parse().unwrap()),
                    ),
                )
            }
            other => Err(TargetError::unsupported("command", other)),
        }
    }
}

fn failing(behavior: Behavior) -> Vec<String> {
    let suite = synthesis(PARCELS).suite;
    let admitted = AdmittedSuite::from_suite(&suite).unwrap();
    let report = Runner::for_suite(&suite)
        .run_admitted(&admitted, &Parcels::new(behavior))
        .into_report();
    let failed: Vec<String> = report
        .scenarios
        .iter()
        .filter(|scenario| scenario.status != Status::Passed)
        .map(|scenario| scenario.scenario.to_string())
        .collect();
    assert_eq!(
        report.status == ConformanceStatus::Passed,
        failed.is_empty(),
        "{report:?}"
    );
    failed
}

#[test]
fn the_rule_as_specified_passes_its_own_suite() {
    assert_eq!(failing(Behavior::Good), Vec::<String>::new());
}

#[test]
fn an_implementation_that_never_reads_the_stored_fields_fails_the_refusal() {
    assert_eq!(failing(Behavior::IgnoresStoredFields), [REFUSAL]);
}

#[test]
fn an_implementation_that_ignores_the_stored_weight_fails_the_boundary_success() {
    let failed = failing(Behavior::IgnoresStoredWeight);
    assert!(failed.contains(&SUCCESS.to_owned()), "{failed:?}");
    assert!(!failed.contains(&REFUSAL.to_owned()), "{failed:?}");
}

#[test]
fn an_implementation_that_answers_an_absent_subject_fails_the_absent_subject_witness() {
    assert_eq!(failing(Behavior::UpsertsAbsentSubject), [REFUSAL]);
}

/// The refusals synthesis recorded about one scenario, rendered with their codes.
fn refusals_about(result: &Synthesis, id: &str) -> Vec<String> {
    result
        .refusals
        .iter()
        .filter(|refusal| {
            refusal
                .scenario
                .as_ref()
                .is_some_and(|scenario| scenario.to_string() == id)
        })
        .map(ToString::to_string)
        .collect()
}

#[test]
fn a_guarded_field_is_arranged_through_an_update_whose_sets_maps_it() {
    let relabelled = PARCELS
        .replace(
            "        sets: {service: input.service, weight_kg: input.weight_kg}\n",
            "        sets: {service: Standard, weight_kg: input.weight_kg}\n",
        )
        .replace(
            "  - name: shipping.parcel.Dispatched\n",
            "  - name: shipping.parcel.Relabelled\n    fields: []\n  - name: shipping.parcel.Dispatched\n",
        )
        .replace(
            "  - name: shipping.parcel.Dispatch\n",
            "  - name: shipping.parcel.Relabel\n    input:\n      - {name: parcel_id, type: Uuid}\n      - {name: service, type: shipping.parcel.Service}\n    outcomes:\n      - name: relabelled\n        updates: shipping.parcel.Parcel\n        instance: parcel_id\n        sets: {service: input.service}\n        emits: [shipping.parcel.Relabelled]\n  - name: shipping.parcel.Dispatch\n",
        );
    let suite = synthesis(&relabelled).suite;
    let refusal = scenario(&suite, REFUSAL);
    let relabel = inputs(refusal, "shipping.parcel.Relabel");
    assert_eq!(relabel.len(), 1, "one update reaches the goal");
    assert_eq!(relabel[0].get("service"), Some(&literal("Express")));
    assert_eq!(
        inputs(refusal, "shipping.parcel.Create")[0].get("weight_kg"),
        Some(&number(21.0)),
        "the literal `sets:` is a fixed point and the mapped weight is still chosen to the goal"
    );
}

#[test]
fn a_guarded_field_no_arranging_branch_can_set_is_refused_by_name() {
    let unset = PARCELS.replace(
        "        sets: {service: input.service, weight_kg: input.weight_kg}\n",
        "        sets: {service: input.service}\n",
    );
    let result = synthesis(&unset);
    let about = refusals_about(&result, REFUSAL);
    assert!(
        about
            .iter()
            .any(|refusal| refusal.contains("ESS-SYNTH-001") && refusal.contains("weight_kg")),
        "{about:?}"
    );
    assert!(
        !result
            .suite
            .scenarios
            .keys()
            .any(|id| id.to_string() == REFUSAL),
        "no unarranged refusal scenario is emitted in its place"
    );
}

#[test]
fn without_a_view_observing_every_guarded_field_the_arrangement_is_refused() {
    let unobserved = PARCELS
        .strip_suffix("      - {name: weight_kg, type: Integer}\n")
        .expect("the view projects the weight last");
    let about = refusals_about(&synthesis(unobserved), REFUSAL);
    assert!(
        about
            .iter()
            .any(|refusal| refusal.contains("immediate unfiltered identity/state/fact view")),
        "{about:?}"
    );
}

#[test]
fn an_optional_stored_field_is_arranged_absent_for_not_defined() {
    let noted = PARCELS
        .replace(
            "      - {name: weight_kg, type: Integer}\n    lifecycle:",
            "      - {name: weight_kg, type: Integer}\n      - {name: note, type: Optional<String>}\n    lifecycle:",
        )
        .replace(
            "      - {name: weight_kg, type: Integer}\n    outcomes:\n      - name: created",
            "      - {name: weight_kg, type: Integer}\n      - {name: note, type: Optional<String>}\n    outcomes:\n      - name: created",
        )
        .replace(
            "        sets: {service: input.service, weight_kg: input.weight_kg}\n",
            "        sets: {service: input.service, weight_kg: input.weight_kg, note: input.note}\n",
        )
        .replace(
            "        when_subject:\n          predicate:\n            all:\n              - service == Express\n              - weight_kg > 20\n",
            "        when_subject: {predicate: not defined(note)}\n",
        )
        + "      - {name: note, type: Optional<String>}\n";
    let result = synthesis(&noted);
    assert!(
        refusals_about(&result, REFUSAL).is_empty(),
        "{:?}",
        result.refusals
    );
    let refusal = scenario(&result.suite, REFUSAL);
    let created = inputs(refusal, "shipping.parcel.Create");
    assert!(
        !created[0].contains_key("note"),
        "omitted, never null: {created:?}"
    );
    assert!(
        rows(refusal)
            .iter()
            .any(|row| row.get("note") == Some(&ScenarioValue::literal(Node::Null))),
        "{:?}",
        rows(refusal)
    );
    let success = scenario(&result.suite, SUCCESS);
    assert!(inputs(success, "shipping.parcel.Create")[0].contains_key("note"));
}

#[test]
fn the_suite_needs_no_new_vocabulary_and_every_lane_admits_it() {
    let suite = synthesis(PARCELS).suite;
    assert_eq!(
        suite.provenance.suite_version.to_string(),
        "ess-conformance/10",
        "the format `expect_no_error` and the subject-fact observations already select; this \
         construct adds no step"
    );
    AdmittedSuite::from_suite(&suite).unwrap();
    ess_conformance::go::emit(&suite).unwrap();
    ess_conformance::ts::emit(&suite).unwrap();
}
