//! Adversarial cases against the stored-field guard unit (ess#75, ess/9).
//!
//! Each case asserts what `docs/design/cross-record-and-stored-field-guards.md`, the story's
//! acceptance, or the base behaviour of an ess/6 model says, and is expected to fail where the
//! implementation disagrees.
use ess_compiler::{
    ir::EssIr,
    refs::{CommandRef, OutcomeRef},
    resolve::compile,
    source::SourceMap,
};
use ess_conformance::{
    report::Status, synthesize::Synthesis, target::*, AdmittedSuite, ConformanceScenario,
    ConformanceSuite, Runner, ScenarioStep,
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
const HISTORY: &str = include_str!("fixtures/subject-history.yaml");
const REFUSAL: &str = "shipping.parcel.Dispatch/outcome/refused-overweight";
const SUCCESS: &str = "shipping.parcel.Dispatch/outcome/dispatched";
const GUARD: &str = "        when_subject:\n          predicate:\n            all:\n              - service == Express\n              - weight_kg > 20\n";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn synthesis(text: &str) -> Synthesis {
    ess_conformance::synthesize::synthesize(&ir(text))
}

fn scenario<'a>(suite: &'a ConformanceSuite, id: &str) -> Option<&'a ConformanceScenario> {
    suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map(|(_, scenario)| scenario)
}

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

/// Both Dispatch branches of a parcels variant are witnessed, with nothing refused about either.
fn both_witnessed(model: &str) {
    let result = synthesis(model);
    for id in [REFUSAL, SUCCESS] {
        assert!(
            refusals_about(&result, id).is_empty(),
            "{id}: {:?}",
            refusals_about(&result, id)
        );
        assert!(scenario(&result.suite, id).is_some(), "no scenario {id}");
    }
}

/// An in-memory parcels service whose Create copies every input field onto the row and whose
/// Dispatch refuses where `refuse` says so over the stored row.
struct Parcels {
    refuse: fn(&BTreeMap<String, Node>) -> bool,
    rows: RefCell<BTreeMap<String, BTreeMap<String, Node>>>,
    minted: Cell<u64>,
}

impl Parcels {
    fn new(refuse: fn(&BTreeMap<String, Node>) -> bool) -> Self {
        Self {
            refuse,
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
        Ok(ImplementationIdentity::new("parcels-adversary", "1"))
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
        let mut rows = self.rows.borrow_mut();
        let command = request.command.clone();
        let result = match command.to_string().as_str() {
            "shipping.parcel.Create" => {
                let id = format!("00000000-0000-4000-8000-{:012}", self.minted.get());
                let mut row: BTreeMap<String, Node> = request.input.clone();
                row.insert("parcel_id".into(), Node::Text(id.clone()));
                row.insert("state".into(), Node::Text("Created".into()));
                rows.insert(id.clone(), row);
                SemanticCommandResult::took(outcome(&command, "created")).emitting(
                    ObservedEvent::new("shipping.parcel.Created".parse().unwrap())
                        .with("parcel_id", Node::Text(id)),
                )
            }
            "shipping.parcel.Dispatch" => {
                let Node::Text(id) = &request.input["parcel_id"] else {
                    return Ok(SemanticCommandResult::undeclared().with_consistency(token));
                };
                let Some(row) = rows.get_mut(id) else {
                    return Ok(SemanticCommandResult::undeclared().with_consistency(token));
                };
                if row["state"] != Node::Text("Created".into()) {
                    return Ok(SemanticCommandResult::undeclared().with_consistency(token));
                }
                if (self.refuse)(row) {
                    SemanticCommandResult::took(outcome(&command, "refused-overweight")).with_error(
                        DeclaredErrorValue::new(
                            "shipping.parcel.ExpressOverweight".parse().unwrap(),
                        ),
                    )
                } else {
                    row.insert("state".into(), Node::Text("Dispatched".into()));
                    SemanticCommandResult::took(outcome(&command, "dispatched")).emitting(
                        ObservedEvent::new("shipping.parcel.Dispatched".parse().unwrap()),
                    )
                }
            }
            other => return Err(TargetError::unsupported("command", other)),
        };
        Ok(result.with_consistency(token))
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

fn failing(model: &str, refuse: fn(&BTreeMap<String, Node>) -> bool) -> Vec<String> {
    let suite = synthesis(model).suite;
    let admitted = AdmittedSuite::from_suite(&suite).unwrap();
    let report = Runner::for_suite(&suite)
        .run_admitted(&admitted, &Parcels::new(refuse))
        .into_report();
    report
        .scenarios
        .iter()
        .filter(|scenario| scenario.status != Status::Passed)
        .map(|scenario| scenario.scenario.to_string())
        .collect()
}

fn number(row: &BTreeMap<String, Node>, field: &str) -> f64 {
    match row.get(field) {
        Some(Node::Number(n)) => n.get(),
        other => panic!("{field} is {other:?}"),
    }
}

/// The rule, "Express parcels over 20 kg are refused", has two leaves. The suite catches an
/// implementation that drops the weight leaf (the implementor's mutant); it must equally catch one
/// that drops the service leaf, which refuses every heavy Standard parcel the rule dispatches.
#[test]
fn adv_an_implementation_that_ignores_the_stored_service_fails_the_suite() {
    // Control: the rule as specified passes, so a red below is the mutant, not the harness.
    assert_eq!(
        failing(PARCELS, |row| row["service"]
            == Node::Text("Express".into())
            && number(row, "weight_kg") > 20.0),
        Vec::<String>::new()
    );
    let failed = failing(PARCELS, |row| number(row, "weight_kg") > 20.0);
    assert!(
        !failed.is_empty(),
        "a Dispatch that refuses every parcel over 20 kg, Standard included, passes the whole \
         generated suite"
    );
}

/// An ess/6 `{field, equals}` model keeps its generated suite: the design promises "`{field,
/// equals}` documents keep ess/6 and their bytes", and the brief names this check. At the base the
/// `answered` default of `calls.core.Answer` asserts its post-state row once, after the command
/// (13 steps, measured with `ess 0.32.1`); nothing about the model changed.
#[test]
fn adv_an_ess6_field_equals_model_keeps_its_generated_default_scenario() {
    let suite = synthesis(HISTORY).suite;
    for id in [
        "calls.core.Answer/outcome/answered",
        "calls.core.Call/transition/bridge/by/calls.core.Answer/answered",
    ] {
        let steps = &scenario(&suite, id).expect("scenario").steps;
        let answer = steps
            .iter()
            .rposition(|step| matches!(step, ScenarioStep::ExecuteCommand { command, .. } if command.to_string() == "calls.core.Answer"))
            .unwrap();
        let views_after = steps[answer + 1..]
            .iter()
            .filter(|step| matches!(step, ScenarioStep::ExpectView { .. }))
            .count();
        assert_eq!(
            (steps.len(), views_after),
            (13, 1),
            "{id} changed for an ess/6 model that does not use the predicate form: {}",
            serde_json::to_string_pretty(steps).unwrap()
        );
    }
}

fn with_field(model: &str, field: &str, guard: &str) -> String {
    model
        .replace(
            "      - {name: weight_kg, type: Integer}\n    lifecycle:",
            &format!("      - {{name: weight_kg, type: Integer}}\n      - {field}\n    lifecycle:"),
        )
        .replace(
            "      - {name: weight_kg, type: Integer}\n    outcomes:\n      - name: created",
            &format!(
                "      - {{name: weight_kg, type: Integer}}\n      - {field}\n    outcomes:\n      - name: created"
            ),
        )
        .replace(GUARD, &format!("        when_subject:\n          predicate: {guard}\n"))
        + &format!("      - {field}\n")
}

fn with_mapped(field_decl: &str, name: &str, guard: &str) -> String {
    with_field(PARCELS, field_decl, guard).replace(
        "        sets: {service: input.service, weight_kg: input.weight_kg}\n",
        &format!(
            "        sets: {{service: input.service, weight_kg: input.weight_kg, {name}: input.{name}}}\n"
        ),
    )
}

/// The brief: "every form the grammar admits after unit A1 (`defined`, `.count`, ordinals,
/// byte-wise text, Timestamp instants) works here" — admitted by validation *and* witnessed.
/// Re-pinned in correction round 1 (F3, deferred) as a refusal, and flipped by
/// `story:count-guards-above-one-are-synthesized`: the witness builds lists of `N + 1` and `N`
/// elements for `.count` against `N`, over input and stored fields alike, so both branches have a
/// scenario and the row that selects the refusal was created with two labels.
#[test]
fn adv_a_count_guard_over_a_stored_list_is_witnessed_by_two_elements() {
    let model = with_mapped(
        "{name: labels, type: List<String>}",
        "labels",
        "'labels.count > 1'",
    );
    both_witnessed(&model);
    let result = synthesis(&model);
    let created: Vec<usize> = scenario(&result.suite, REFUSAL)
        .unwrap()
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, input, .. }
                if command.to_string() == "shipping.parcel.Create" =>
            {
                input.get("labels").and_then(|value| match value {
                    ess_conformance::ScenarioValue::Literal {
                        value: Node::Seq(labels),
                    } => Some(labels.len()),
                    _ => None,
                })
            }
            _ => None,
        })
        .collect();
    assert_eq!(created, [2], "the refusing row holds two labels");
    // And the rule, run: a Dispatch that refuses a parcel with more than one label passes.
    assert_eq!(
        failing(
            &model,
            |row| matches!(row.get("labels"), Some(Node::Seq(labels)) if labels.len() > 1)
        ),
        Vec::<String>::new()
    );
}

#[test]
fn adv_an_ordinal_guard_over_a_stored_list_is_witnessed() {
    both_witnessed(&with_mapped(
        "{name: labels, type: List<String>}",
        "labels",
        "'labels.0 == fragile'",
    ));
}

#[test]
fn adv_a_timestamp_guard_over_a_stored_instant_is_witnessed() {
    both_witnessed(&with_mapped(
        "{name: weighed_at, type: Timestamp}",
        "weighed_at",
        "'weighed_at < \"2026-01-01T00:00:00Z\"'",
    ));
}

#[test]
fn adv_a_byte_wise_text_guard_over_a_stored_string_is_witnessed() {
    both_witnessed(&with_mapped(
        "{name: label, type: String}",
        "label",
        "'label < m'",
    ));
}

#[test]
fn adv_a_boolean_guard_over_a_stored_flag_is_witnessed() {
    both_witnessed(&with_mapped(
        "{name: fragile, type: Boolean}",
        "fragile",
        "'fragile == true'",
    ));
}

#[test]
fn adv_defined_over_an_optional_stored_field_is_witnessed() {
    both_witnessed(&with_mapped(
        "{name: note, type: Optional<String>}",
        "note",
        "'defined(note)'",
    ));
}

/// An ordering over an `Optional` stored field: the default's witness must be a row the guard
/// decides (an absent weight is `Unknown` and selects no branch, never the default).
#[test]
fn adv_an_ordering_over_an_optional_stored_field_is_witnessed() {
    let model = PARCELS
        .replace(
            "      - {name: weight_kg, type: Integer}\n    lifecycle:",
            "      - {name: weight_kg, type: Optional<Integer>}\n    lifecycle:",
        )
        .replace(
            "      - {name: weight_kg, type: Integer}\n    outcomes:\n      - name: created",
            "      - {name: weight_kg, type: Optional<Integer>}\n    outcomes:\n      - name: created",
        )
        .replace(
            "      - {name: weight_kg, type: Integer}\n",
            "      - {name: weight_kg, type: Optional<Integer>}\n",
        );
    both_witnessed(&model);
}

/// The subject predicate is conjunctive with an ordinary input `when:` on the same refusal.
#[test]
fn adv_a_stored_guard_conjoined_with_an_input_guard_is_witnessed() {
    let model = PARCELS
        .replace(
            "  - name: shipping.parcel.Dispatch\n    input:\n      - {name: parcel_id, type: Uuid}\n",
            "  - name: shipping.parcel.Dispatch\n    input:\n      - {name: parcel_id, type: Uuid}\n      - {name: priority, type: shipping.parcel.Service}\n",
        )
        .replace(GUARD, &format!("{GUARD}        when: priority == Express\n"));
    both_witnessed(&model);
}

/// Two refusals on different stored fields beside one default: each is witnessed on a row only it
/// selects.
#[test]
fn adv_two_refusals_on_different_stored_fields_are_each_witnessed() {
    let model = PARCELS
        .replace(
            "  - name: shipping.parcel.ExpressOverweight\n    fields: []\n",
            "  - name: shipping.parcel.ExpressOverweight\n    fields: []\n  - name: shipping.parcel.Overweight\n    fields: []\n",
        )
        .replace(
            GUARD,
            "        when_subject: {predicate: service == Express}\n",
        )
        .replace(
            "        error: shipping.parcel.ExpressOverweight\n",
            "        error: shipping.parcel.ExpressOverweight\n      - name: refused-heavy\n        when_subject: {predicate: weight_kg > 50}\n        error: shipping.parcel.Overweight\n",
        );
    let result = synthesis(&model);
    for id in [
        REFUSAL,
        "shipping.parcel.Dispatch/outcome/refused-heavy",
        SUCCESS,
    ] {
        assert!(
            refusals_about(&result, id).is_empty() && scenario(&result.suite, id).is_some(),
            "{id}: {:?}",
            refusals_about(&result, id)
        );
    }
}

/// Re-pinned in correction round 1 (F3, deferred) as a refusal, and flipped by
/// `story:count-guards-above-one-are-synthesized`. Control for the `.count` case: the same guard
/// over the *input* of the creating command, where the gap the stored-field case inherited lived.
/// Both branches have a scenario, and the input that satisfies the guard holds two elements.
#[test]
fn adv_control_the_same_count_guard_over_input_is_witnessed_by_two_elements() {
    let model = with_mapped(
        "{name: labels, type: List<String>}",
        "labels",
        "'weight_kg > 20'",
    )
    .replace(
        "      - name: created\n        creates: shipping.parcel.Parcel\n",
        "      - name: bulky\n        when: labels.count > 1\n        error: shipping.parcel.ExpressOverweight\n      - name: created\n        creates: shipping.parcel.Parcel\n",
    );
    let result = synthesis(&model);
    for id in [
        "shipping.parcel.Create/outcome/bulky",
        "shipping.parcel.Create/outcome/created",
    ] {
        assert!(
            refusals_about(&result, id).is_empty() && scenario(&result.suite, id).is_some(),
            "{id}: {:?}",
            refusals_about(&result, id)
        );
    }
    let bulky = scenario(&result.suite, "shipping.parcel.Create/outcome/bulky").unwrap();
    let labels: Vec<usize> = bulky
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand { input, .. } => {
                input.get("labels").and_then(|value| match value {
                    ess_conformance::ScenarioValue::Literal {
                        value: Node::Seq(labels),
                    } => Some(labels.len()),
                    _ => None,
                })
            }
            _ => None,
        })
        .collect();
    assert_eq!(labels, [2], "the satisfying input holds two elements");
}

/// Past the witness cap, a stored count guard is named by what it needs rather than told to drop
/// the field from the command's input (`story:count-guards-above-one-are-synthesized`, third
/// acceptance line).
#[test]
fn adv_a_stored_count_guard_past_the_cap_is_refused_as_a_count_not_as_a_missing_value() {
    let result = synthesis(&with_mapped(
        "{name: labels, type: List<String>}",
        "labels",
        "'labels.count > 5000'",
    ));
    let about = refusals_about(&result, REFUSAL);
    assert!(scenario(&result.suite, REFUSAL).is_none(), "{about:?}");
    assert!(
        about
            .iter()
            .any(|refusal| refusal.contains("ESS-SYNTH-018")),
        "{about:?}"
    );
    assert!(
        about
            .iter()
            .all(|refusal| !refusal.contains("drop it from the command's input")),
        "{about:?}"
    );
}

/// The emitted Go and TypeScript runtimes run the parcels suite and agree with the Rust runner: the
/// rule passes in both lanes, and every mutant the Rust runner fails is failed in both lanes.
/// The implementor's lane check stops at `emit(..).unwrap()`; nothing ran the emitted runtimes
/// against the absent-subject step (an `execute_command` with no outcome expectation after it).
#[test]
fn adv_go_and_typescript_lanes_agree_with_rust_on_the_parcels_suite() {
    let suite = synthesis(PARCELS).suite;
    let root = std::env::temp_dir().join(format!("ess-adv-parcels-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    for artifact in ess_conformance::go::emit(&suite).unwrap() {
        let path = root.join(artifact.path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, artifact.contents).unwrap();
    }
    std::fs::write(
        root.join("go.mod"),
        "module example.invalid/parcels\n\ngo 1.24\n",
    )
    .unwrap();
    std::fs::write(
        root.join("essconform/parcels_test.go"),
        include_str!("fixtures/stored-field-guards-adversary-runtime.go"),
    )
    .unwrap();
    for artifact in ess_conformance::ts::emit(&suite).unwrap() {
        let path = root.join("typescript").join(artifact.path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, artifact.contents).unwrap();
    }
    let ts = root.join("typescript/essconform");
    std::fs::write(
        ts.join("parcels.mjs"),
        include_str!("fixtures/stored-field-guards-adversary-runtime.mjs"),
    )
    .unwrap();
    std::fs::write(
        ts.join("runtime-test.tsconfig.json"),
        r#"{"extends":"./tsconfig.json","compilerOptions":{"types":[],"noCheck":true}}"#,
    )
    .unwrap();
    let compiled = std::process::Command::new("tsc")
        .args(["--project", "runtime-test.tsconfig.json"])
        .current_dir(&ts)
        .output()
        .unwrap();
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stdout)
    );
    let mut disagreements = Vec::new();
    for (mutant, passes) in [("", true), ("weight", false), ("fields", false)] {
        for (tool, args, directory) in [
            ("go", vec!["test", "./essconform", "-count=1", "-v"], &root),
            ("node", vec!["--test", "parcels.mjs"], &ts),
        ] {
            let output = std::process::Command::new(tool)
                .args(args)
                .env("ESS_PARCELS_MUTANT", mutant)
                .env("ESS_REPORT_FORMAT", "2")
                .env("GOWORK", "off")
                .current_dir(directory)
                .output()
                .unwrap();
            if output.status.success() != passes {
                disagreements.push(format!(
                    "{tool} mutant={mutant:?} passed={}\n{}\n{}",
                    output.status.success(),
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                ));
            }
        }
    }
    std::fs::remove_dir_all(&root).unwrap();
    assert!(
        disagreements.is_empty(),
        "{}",
        disagreements.join("\n----\n")
    );
}
