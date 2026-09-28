//! A library return is observable without a published event or a persisted entity.
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::{
    authored, report::Status, target::*, AdmittedSuite, ConformanceSuite, Runner,
};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;
use std::{cell::Cell, collections::BTreeMap};

const MODEL: &str = r"format: ess/17
system: library
version: v1
domain: library.api
types:
  - name: library.api.Item
    kind: struct
    fields:
      - {name: label, type: String}
      - {name: ordinal, type: Integer}
commands:
  - name: library.api.Read
    response:
      - {name: value, type: String}
      - {name: sequence, type: 'List<Integer>'}
      - {name: item, type: library.api.Item}
    outcomes:
      - name: returned
        returns: true
";

const SCENARIO: &str = r"type: ess-scenario/4
domain: library.api
scenario: pure-return
summary: The actual return contains the declared literal and ordered values.
timeline:
  - at: 2026-09-28T00:00:00Z
    command: library.api.Read
    outcome: returned
    response: {value: actual, sequence: [1, 2, 2], item: {label: nested, ordinal: 9007199254740993}}
";

fn ir() -> ess_compiler::EssIr {
    let spec = Specification::assemble([(
        Source::new("library.yaml"),
        RawSpecFile::parse(MODEL).expect("direct-return source parses"),
    )])
    .expect("direct-return source validates");
    compile(&spec, &SourceMap::new()).expect("direct-return source compiles")
}

fn suite() -> ConformanceSuite {
    let model = ir();
    let authored = authored::compile(&model, &[authored::Source::new("return.yaml", SCENARIO)]);
    assert!(authored.refusals.is_empty(), "{:?}", authored.refusals);
    let mut suite = ess_conformance::synthesize(&model).suite;
    suite.scenarios = authored.scenarios;
    suite.select_fresh_format();
    suite
}

#[test]
fn pure_return_correct_literal_compiles_without_events() {
    let suite = suite();
    assert_eq!(suite.provenance.suite_version.major(), 28);
    let bytes = suite.to_canonical_json().unwrap();
    assert!(bytes.contains("expect_direct_response"), "{bytes}");
    assert!(!bytes.contains("query_view"));
    assert!(!bytes.contains("expect_event"));
}

struct Library {
    response: Option<BTreeMap<String, Node>>,
    calls: Cell<usize>,
}

impl ConformanceTarget for Library {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("direct-return-library", "test"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        self.calls.set(self.calls.get() + 1);
        let mut result = SemanticCommandResult::took(ess_conformance::scenario::OutcomeRef::new(
            request.command,
            "returned".parse().unwrap(),
        ));
        result.response.clone_from(&self.response);
        Ok(result)
    }
    fn query_view(&self, _: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        panic!("no view exists")
    }
    fn observe_events(
        &self,
        _: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        panic!("no event exists")
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        panic!("no external outcome")
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        panic!("no event exists")
    }
}

fn actual() -> BTreeMap<String, Node> {
    BTreeMap::from([
        ("value".into(), Node::Text("actual".into())),
        (
            "sequence".into(),
            Node::Seq([1_i64, 2, 2].map(|value| Node::Number(value.into())).into()),
        ),
        (
            "item".into(),
            Node::Map(BTreeMap::from([
                ("label".into(), Node::Text("nested".into())),
                (
                    "ordinal".into(),
                    Node::Number(9_007_199_254_740_993_i64.into()),
                ),
            ])),
        ),
    ])
}

fn run(response: Option<BTreeMap<String, Node>>, expected: Status) {
    let admitted = AdmittedSuite::from_suite(&suite()).unwrap();
    let library = Library {
        response,
        calls: Cell::new(0),
    };
    let run = Runner::for_suite(admitted.suite()).run_admitted(&admitted, &library);
    assert_eq!(library.calls.get(), 1);
    assert_eq!(run.scenarios.len(), 1);
    assert_eq!(
        run.scenarios[0].scenario.to_string(),
        "library.api/authored/pure-return"
    );
    assert_eq!(run.scenarios[0].status, expected, "{:?}", run.scenarios);
    let report = ess_conformance::counts::CountReport::from_run(&run, &admitted).unwrap();
    let bytes = report.to_canonical_json().unwrap();
    assert!(bytes.contains(admitted.digest()));
    assert_eq!(
        ess_conformance::counts::CountReport::from_json(&bytes, &admitted).unwrap(),
        report
    );
}

#[test]
fn pure_return_correct_literal_passes_without_events() {
    run(Some(actual()), Status::Passed);
}

#[test]
fn pure_return_wrong_literal_fails_named_scenario() {
    let mut response = actual();
    response.insert("value".into(), Node::Text("wrong".into()));
    run(Some(response), Status::Failed);
}

#[test]
fn pure_return_payloads_have_an_independent_lossless_resource_bound() {
    let text = "x".repeat(4097);
    let source = SCENARIO.replace("value: actual", &format!("value: {text}"));
    let model = ir();
    let compiled = authored::compile(&model, &[authored::Source::new("large.yaml", source)]);
    assert!(compiled.refusals.is_empty(), "{:?}", compiled.refusals);
    let mut suite = ess_conformance::synthesize(&model).suite;
    suite.scenarios = compiled.scenarios;
    suite.select_fresh_format();
    let admitted = AdmittedSuite::from_suite(&suite).unwrap();
    let mut response = actual();
    response.insert("value".into(), Node::Text(text));
    let library = Library {
        response: Some(response),
        calls: Cell::new(0),
    };
    let run = Runner::for_suite(admitted.suite()).run_admitted(&admitted, &library);
    assert_eq!(
        run.scenarios[0].status,
        Status::Passed,
        "{:?}",
        run.scenarios
    );

    let generated = ess_conformance::synthesize(&model);
    let admitted = AdmittedSuite::from_suite(&generated.suite).unwrap();
    let mut response = actual();
    response.insert(
        "sequence".into(),
        Node::Seq(vec![Node::Number(1_i64.into()); 65]),
    );
    let mut library = Library {
        response: Some(response),
        calls: Cell::new(0),
    };
    let run = Runner::for_suite(admitted.suite()).run_admitted(&admitted, &library);
    assert_eq!(
        run.scenarios[0].status,
        Status::Passed,
        "{:?}",
        run.scenarios
    );
    library
        .response
        .as_mut()
        .unwrap()
        .insert("value".into(), Node::Text("x".repeat(1_048_576)));
    let run = Runner::for_suite(admitted.suite()).run_admitted(&admitted, &library);
    assert_eq!(run.scenarios[0].status, Status::Failed);
    assert!(format!("{:?}", run.scenarios).contains("resource"));
}

#[test]
fn pure_return_absent_field_and_extra_field_refuse() {
    run(None, Status::Failed);
    let mut missing = actual();
    missing.remove("value");
    run(Some(missing), Status::Failed);
    let mut extra = actual();
    extra.insert("extra".into(), Node::Bool(true));
    run(Some(extra), Status::Failed);
}

#[test]
fn pure_return_complete_nested_values_preserve_order_and_duplicates() {
    for values in [vec![2_i64, 1, 2], vec![1, 2], vec![1, 2, 2, 2]] {
        let mut response = actual();
        response.insert(
            "sequence".into(),
            Node::Seq(
                values
                    .into_iter()
                    .map(|value| Node::Number(value.into()))
                    .collect(),
            ),
        );
        run(Some(response), Status::Failed);
    }
    let mut response = actual();
    let Node::Map(item) = response.get_mut("item").unwrap() else {
        panic!("item is an object")
    };
    item.insert(
        "ordinal".into(),
        Node::Number(9_007_199_254_740_992_i64.into()),
    );
    run(Some(response), Status::Failed);
    let mut response = actual();
    let Node::Map(item) = response.get_mut("item").unwrap() else {
        panic!("item is an object")
    };
    item.insert("extra".into(), Node::Null);
    run(Some(response), Status::Failed);
}

#[test]
fn pure_return_unknown_response_coordinate_refuses_at_compile() {
    for written in [
        SCENARIO.replace("value: actual", "unknown: actual"),
        SCENARIO.replace("value: actual", "value: 3"),
    ] {
        let result = authored::compile(&ir(), &[authored::Source::new("invalid.yaml", written)]);
        assert!(result.scenarios.is_empty());
        assert_eq!(result.refusals.len(), 1);
        assert!(result.refusals[0].to_string().contains("response"));
    }
}

#[test]
fn pure_return_old_formats_refuse_before_target_effects() {
    for major in 1..=27 {
        let mut old = suite();
        old.provenance.suite_version =
            ess_conformance::scenario::SuiteFormat::parse(&format!("ess-conformance/{major}"))
                .unwrap();
        let error = AdmittedSuite::from_suite(&old).unwrap_err();
        assert!(
            error
                .issues
                .iter()
                .any(|issue| issue.reason == "UnsupportedVocabulary"),
            "{error}"
        );
    }
    for major in 1..=3 {
        let old = SCENARIO.replace("ess-scenario/4", &format!("ess-scenario/{major}"));
        let result = authored::compile(&ir(), &[authored::Source::new("old.yaml", old)]);
        assert!(result.scenarios.is_empty());
        assert_eq!(result.refusals.len(), 1);
        assert!(result.refusals[0].to_string().contains("ess-scenario/4"));
    }
}

#[test]
fn direct_return_admission_tracks_every_command_invocation_form() {
    use ess_conformance::ScenarioStep;
    let other = "library.api.Other".parse().unwrap();
    for invocation in [
        ScenarioStep::ExecuteCommand {
            command: other,
            actor: None,
            caller: BTreeMap::new(),
            input: BTreeMap::new(),
        },
        ScenarioStep::ExecuteCommandWithoutInput {
            command: "library.api.Other".parse().unwrap(),
            actor: None,
            caller: BTreeMap::new(),
        },
    ] {
        let mut invalid = suite();
        let steps = &mut invalid.scenarios.values_mut().next().unwrap().steps;
        let observation = steps
            .iter()
            .position(|step| matches!(step, ScenarioStep::ExpectDirectResponse { .. }))
            .unwrap();
        steps.insert(observation, invocation);
        let error = AdmittedSuite::from_suite(&invalid)
            .expect_err("a return assertion cannot name an earlier invocation");
        assert!(error
            .issues
            .iter()
            .any(|issue| issue.reason == "InvalidResponse"));
    }
}

#[test]
fn pure_return_source_requires_its_format_and_a_successful_typed_response() {
    for source in [
        MODEL.replace("ess/17", "ess/16"),
        MODEL.replace("        returns: true", "        returns: true\n        accepts: nothing"),
        MODEL.replace("        returns: true", "        returns: true\n        error: library.api.Failed") + "errors:\n  - name: library.api.Failed\n    fields: []\n",
        "format: ess/17\nsystem: library\nversion: v1\ndomain: library.api\ncommands:\n  - name: library.api.Read\n    outcomes:\n      - {name: returned, returns: true}\n".into(),
    ] {
        let raw = RawSpecFile::parse(&source).unwrap();
        let error = Specification::assemble([(Source::new("invalid.yaml"), raw)]).unwrap_err();
        assert!(!error.to_string().is_empty());
    }
}

#[test]
fn pure_return_generated_shape_is_checked_without_authored_literals() {
    let generated = ess_conformance::synthesize(&ir());
    assert!(generated.refusals.is_empty(), "{:?}", generated.refusals);
    let admitted = AdmittedSuite::from_suite(&generated.suite).unwrap();
    for (response, status) in [(Some(actual()), Status::Passed), (None, Status::Failed)] {
        let target = Library {
            response,
            calls: Cell::new(0),
        };
        let run = Runner::for_suite(admitted.suite()).run_admitted(&admitted, &target);
        assert_eq!(run.scenarios.len(), 1);
        assert_eq!(run.scenarios[0].status, status);
    }
}

#[test]
fn pure_return_nested_presence_is_preserved_and_checked() {
    use ess_conformance::scenario::CommandRef;
    for (policy, absent, present) in [
        (
            "null_when_absent",
            Node::Map(BTreeMap::new()),
            Node::Map(BTreeMap::from([("note".into(), Node::Null)])),
        ),
        (
            "omitted_when_absent",
            Node::Map(BTreeMap::from([("note".into(), Node::Null)])),
            Node::Map(BTreeMap::new()),
        ),
    ] {
        let source = MODEL.replace("      - {name: ordinal, type: Integer}", &format!("      - {{name: ordinal, type: Integer}}\n      - {{name: note, type: 'Optional<String>', presence: {policy}}}"));
        let spec = Specification::assemble([(
            Source::new("presence.yaml"),
            RawSpecFile::parse(&source).unwrap(),
        )])
        .unwrap();
        let model = compile(&spec, &SourceMap::new()).unwrap();
        let command = model
            .commands()
            .get(&"library.api.Read".parse().unwrap())
            .unwrap();
        let observation = ess_conformance::direct_response::Observation::of(
            &model,
            command,
            None,
            BTreeMap::new(),
        )
        .unwrap();
        assert_eq!(
            observation.command,
            CommandRef::new("library.api.Read".parse().unwrap())
        );
        for (note, succeeds) in [(absent, false), (present, true)] {
            let Node::Map(mut note) = note else {
                unreachable!()
            };
            let mut response = actual();
            let Node::Map(item) = response.get_mut("item").unwrap() else {
                unreachable!()
            };
            item.append(&mut note);
            assert_eq!(
                observation.compare(Some(&response)).is_ok(),
                succeeds,
                "{policy}: {response:?}"
            );
            let expected = BTreeMap::from([("item".into(), response["item"].clone())]);
            assert_eq!(
                ess_conformance::direct_response::Observation::of(&model, command, None, expected)
                    .is_ok(),
                succeeds,
                "authored nested literals must honor {policy}"
            );
            let generated = ess_conformance::synthesize(&model);
            let admitted = AdmittedSuite::from_suite(&generated.suite).unwrap();
            let target = Library {
                response: Some(response),
                calls: Cell::new(0),
            };
            let run = Runner::for_suite(admitted.suite()).run_admitted(&admitted, &target);
            let expected_status = if succeeds {
                Status::Passed
            } else {
                Status::Failed
            };
            assert_eq!(run.scenarios[0].status, expected_status, "{policy}");
        }
    }
}

#[test]
fn pure_return_json_obeys_the_same_recursive_resource_bounds() {
    let source = MODEL.replace(
        "{name: item, type: library.api.Item}",
        "{name: item, type: Json}",
    );
    let spec = Specification::assemble([(
        Source::new("json.yaml"),
        RawSpecFile::parse(&source).unwrap(),
    )])
    .unwrap();
    let model = compile(&spec, &SourceMap::new()).unwrap();
    let command = model
        .commands()
        .get(&"library.api.Read".parse().unwrap())
        .unwrap();
    let observation =
        ess_conformance::direct_response::Observation::of(&model, command, None, BTreeMap::new())
            .unwrap();
    let nested = |depth| (0..depth).fold(Node::Null, |inner, _| Node::Seq(vec![inner]));
    for (value, allowed) in [
        (Node::Seq(vec![Node::Null; 65_536]), true),
        (Node::Seq(vec![Node::Null; 65_537]), false),
        (nested(127), true),
        (nested(130), false),
    ] {
        let mut response = actual();
        response.insert("item".into(), value.clone());
        assert_eq!(observation.compare(Some(&response)).is_ok(), allowed);
        assert_eq!(
            ess_conformance::direct_response::Observation::of(
                &model,
                command,
                None,
                BTreeMap::from([("item".into(), value)])
            )
            .is_ok(),
            allowed
        );
    }
}

#[test]
fn pure_return_generators_refuse_unsupported_execution() {
    for error in [
        ess_conformance::go::emit(&suite()).unwrap_err(),
        ess_conformance::ts::emit(&suite()).unwrap_err(),
    ] {
        assert_eq!(error.issues[0].reason, "UnsupportedTarget");
    }
}

#[test]
fn pure_return_native_binary64_remains_explicitly_unsupported() {
    let source = MODEL.replace("type: String", "type: Binary64");
    let spec = Specification::assemble([(
        Source::new("binary.yaml"),
        RawSpecFile::parse(&source).unwrap(),
    )])
    .unwrap();
    let model = compile(&spec, &SourceMap::new()).unwrap();
    let result = authored::compile(&model, &[authored::Source::new("return.yaml", SCENARIO)]);
    assert!(result.scenarios.is_empty());
    assert!(matches!(
        result.refusals[0].cause,
        authored::Cause::UnsupportedBinary64 { .. }
    ));
}

#[test]
fn pure_return_legacy_suite_bytes_unchanged() {
    let bytes = include_str!("../../../../suites/generated/billing/suite.json");
    let suite = ConformanceSuite::from_json(bytes).unwrap();
    assert_eq!(suite.to_canonical_json().unwrap(), bytes);
    let source = include_str!("fixtures/response-payload.yaml");
    let spec = Specification::assemble([(
        Source::new("legacy.yaml"),
        RawSpecFile::parse(source).unwrap(),
    )])
    .unwrap();
    let model = compile(&spec, &SourceMap::new()).unwrap();
    assert!(!model.to_compact_json().contains("\"returns\""));
    let synthesis = ess_conformance::synthesize(&model);
    assert_eq!(synthesis.suite.provenance.suite_version.major(), 8);
    assert!(!synthesis
        .suite
        .to_canonical_json()
        .unwrap()
        .contains("expect_direct_response"));
}

#[test]
fn pure_return_coverage_and_report_retain_exact_admitted_bytes() {
    use ess_conformance::{
        coverage::{Origins, Scope},
        coverage_build::{build, CoverageSource},
    };
    let model = ir();
    let source = CoverageSource::new("return.yaml", SCENARIO).unwrap();
    let input = build(
        &model,
        &[source],
        Scope::System,
        Origins::GeneratedAndAuthored,
    )
    .unwrap();
    let admitted = input.selected();
    assert_eq!(admitted.suite().provenance.suite_version.major(), 29);
    assert_eq!(admitted.suite().scenarios.len(), 2);
    let target = Library {
        response: Some(actual()),
        calls: Cell::new(0),
    };
    let run = Runner::for_suite(admitted.suite()).run_admitted(admitted, &target);
    let report = ess_conformance::counts::CountReport::from_run(&run, admitted).unwrap();
    assert_eq!(report.counts().passed, 2);
    assert_eq!(report.counts().failed, 0);
    assert_eq!(report.counts().unsupported, 0);
    assert_eq!(report.counts().skipped, 0);
    assert_eq!(report.counts().error, 0);
    for error in [
        ess_conformance::go::emit_input(&input).unwrap_err(),
        ess_conformance::ts::emit_input(&input).unwrap_err(),
    ] {
        assert_eq!(error.issues[0].reason, "UnsupportedTarget");
    }
    assert!(report
        .to_canonical_json()
        .unwrap()
        .contains(admitted.digest()));
}

#[test]
fn pure_return_depth_boundary_survives_original_suite_bytes() {
    use ess_conformance::{
        coverage::{Origins, Scope},
        coverage_build::build,
        ScenarioStep,
    };
    let source = MODEL.replace("type: String}", "type: Json}");
    let spec = Specification::assemble([(
        Source::new("depth.yaml"),
        RawSpecFile::parse(&source).unwrap(),
    )])
    .unwrap();
    let model = compile(&spec, &SourceMap::new()).unwrap();
    let input = build(&model, &[], Scope::System, Origins::Generated).unwrap();
    let suite = ess_conformance::synthesize(&model).suite;
    for bytes in [
        suite.to_canonical_json().unwrap(),
        input.selected().original_json().into(),
    ] {
        for depth in [128, 129] {
            let nested = (0..depth).fold(Node::Null, |inner, _| Node::Seq(vec![inner]));
            let mut document: serde_json::Value = serde_json::from_str(&bytes).unwrap();
            for scenario in document["scenarios"].as_object_mut().unwrap().values_mut() {
                for step in scenario["steps"].as_array_mut().unwrap() {
                    if step["step"] == "expect_direct_response" {
                        step["response"]["expected"]["value"] =
                            serde_json::to_value(&nested).unwrap();
                    }
                }
            }
            let original = serde_json::to_string(&document).unwrap();
            let admitted = AdmittedSuite::from_json(&original);
            if depth == 129 {
                assert!(admitted.is_err(), "over-depth literal was admitted");
                continue;
            }
            let admitted = admitted.expect("depth-128 literal survives its suite envelope");
            assert_eq!(admitted.original_json(), original);
            if admitted.coverage().is_some() {
                let input =
                    ess_conformance::coverage::AdmittedInput::from_suite(admitted.clone()).unwrap();
                let ids = input
                    .selected()
                    .suite()
                    .scenarios
                    .keys()
                    .cloned()
                    .collect::<Vec<_>>();
                let selected = input.select(&ids).unwrap();
                let carrier = selected.document().to_canonical_json().unwrap();
                let read = ess_conformance::coverage::AdmittedInput::from_json(&carrier).unwrap();
                assert_eq!(read.parents()[0].original_json(), original);
            }
            let response = admitted
                .suite()
                .scenarios
                .values()
                .next()
                .unwrap()
                .steps
                .iter()
                .find_map(|step| {
                    if let ScenarioStep::ExpectDirectResponse { response } = step {
                        Some(response)
                    } else {
                        None
                    }
                })
                .unwrap();
            response.validate().unwrap();
            let mut actual = actual();
            actual.insert("value".into(), nested);
            response.compare(Some(&actual)).unwrap();
            let target = Library {
                response: Some(actual),
                calls: Cell::new(0),
            };
            let run = Runner::for_suite(admitted.suite()).run_admitted(&admitted, &target);
            assert_eq!(run.scenarios[0].status, Status::Passed);
        }
    }
}

#[test]
fn pure_return_depth_allowance_does_not_expand_other_suite_positions() {
    use ess_conformance::{
        scenario::{ScenarioValue, SuiteFormat},
        ScenarioStep,
    };
    let mut suite = suite();
    for scenario in suite.scenarios.values_mut() {
        scenario
            .steps
            .retain(|step| !matches!(step, ScenarioStep::ExpectDirectResponse { .. }));
        for step in &mut scenario.steps {
            if let ScenarioStep::ExecuteCommand { input, .. } = step {
                let nested = (0..128).fold(Node::Null, |inner, _| Node::Seq(vec![inner]));
                input.insert("nested".into(), ScenarioValue::literal(nested));
            }
        }
    }
    for major in [1, 26, 28] {
        suite.provenance.suite_version =
            SuiteFormat::parse(&format!("ess-conformance/{major}")).unwrap();
        let error = AdmittedSuite::from_json(&suite.to_canonical_json().unwrap()).unwrap_err();
        assert_eq!(error.issues[0].reason, "InvalidDocument");
        assert_eq!(error.issues[0].detail, "JSON nesting exceeds 128");
    }
}
