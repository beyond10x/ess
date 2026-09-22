//! Fixture inputs are chosen before execution, independently of observed output.
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};

const MODEL: &str = include_str!("fixtures/pre-execution-fixtures.yaml");

use ess_conformance::{
    report::Status, target::*, AdmittedSuite, ConformanceSuite, Runner, ScenarioStep,
};
use ess_primitives::node::Node;
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
};

fn suite() -> ConformanceSuite {
    ess_conformance::synthesize::synthesize(&fixture_ir()).suite
}

fn fixture_ir() -> ess_compiler::ir::EssIr {
    let spec = Specification::assemble([(
        Source::new("fixtures.yaml"),
        RawSpecFile::parse(MODEL).unwrap(),
    )])
    .unwrap();
    compile(&spec, &SourceMap::new()).unwrap()
}

#[derive(Default)]
struct Target {
    mode: &'static str,
    resolved: Cell<usize>,
    begun: Cell<usize>,
    ended: Cell<usize>,
    inputs: RefCell<Vec<BTreeMap<String, Node>>>,
}

impl ConformanceTarget for Target {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("fixture-values", "1"))
    }
    fn fixture_values(
        &self,
        _: &ScenarioContext,
        _: &ess_conformance::fixtures::Contract,
    ) -> Result<BTreeMap<String, Node>, TargetError> {
        self.resolved.set(self.resolved.get() + 1);
        if self.mode == "unsupported" {
            return Err(TargetError::unsupported("fixture values", "absent"));
        }
        let mut values = BTreeMap::from([
            (
                "current-principal-id".into(),
                Node::Text("d248c830-37a4-466b-8319-af71667f1364".into()),
            ),
            (
                "current-principal-email".into(),
                Node::Text(format!("principal-{}@example.com", self.resolved.get())),
            ),
        ]);
        match self.mode {
            "wrong-type" => {
                values.insert("current-principal-id".into(), Node::Bool(true));
            }
            "invalid-uuid" => {
                values.insert(
                    "current-principal-id".into(),
                    Node::Text("not-a-uuid".into()),
                );
            }
            "missing" => {
                values.remove("current-principal-email");
            }
            "unknown" => {
                values.insert("unknown".into(), Node::Null);
            }
            _ => {}
        }
        Ok(values)
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        assert_eq!(
            self.resolved.get(),
            self.begun.get() + 1,
            "fixtures resolve before the session opens"
        );
        self.begun.set(self.begun.get() + 1);
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.ended.set(self.ended.get() + 1);
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        self.inputs.borrow_mut().push(request.input.clone());
        let mut result = SemanticCommandResult::took(ess_compiler::refs::OutcomeRef::new(
            request.command,
            "opened".parse().unwrap(),
        ));
        let mut event = ObservedEvent::new("fixturetest.session.Opened".parse().unwrap());
        event.payload = request.input;
        if self.mode == "wrong-then-right" {
            let mut wrong = event.clone();
            wrong
                .payload
                .insert("email".into(), Node::Text("wrong@example.com".into()));
            result.direct_events.push(wrong);
        }
        if self.mode == "split-output" {
            let mut second = event.clone();
            event
                .payload
                .insert("email".into(), Node::Text("wrong@example.com".into()));
            second
                .payload
                .insert("region".into(), Node::Text("wrong-region".into()));
            result.direct_events.push(second);
        }
        if self.mode == "wrong-output" {
            event
                .payload
                .insert("email".into(), Node::Text("different@example.com".into()));
        }
        result.direct_events.push(event);
        Ok(result)
    }
    fn query_view(&self, _: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        Err(TargetError::unsupported("view", "unused"))
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        Err(TargetError::unsupported("external", "unused"))
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        Err(TargetError::unsupported("redelivery", "unused"))
    }
    fn observe_events(
        &self,
        _: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Err(TargetError::unsupported("events", "unused"))
    }
    fn observe_invocations(
        &self,
        _: InvocationObservationRequest,
    ) -> Result<Vec<ObservedInvocation>, TargetError> {
        Err(TargetError::unsupported("invocations", "unused"))
    }
}

fn run(suite: &ConformanceSuite, target: &Target) -> Status {
    let admitted = AdmittedSuite::from_suite(suite).unwrap();
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, target)
        .scenarios[0]
        .status
}

#[test]
fn independent_values_pass_and_wrong_observed_values_fail() {
    for (mode, expected) in [
        ("valid", Status::Passed),
        ("wrong-output", Status::Failed),
        ("split-output", Status::Failed),
        ("wrong-then-right", Status::Failed),
    ] {
        let target = Target {
            mode,
            ..Default::default()
        };
        assert_eq!(run(&suite(), &target), expected, "{mode}");
        assert_eq!(target.begun.get(), 1);
        assert_eq!(target.ended.get(), 1);
        assert_eq!(
            target.inputs.borrow()[0]["email"],
            Node::Text("principal-1@example.com".into())
        );
    }
}

#[test]
fn invalid_provider_data_stops_before_target_activity_and_missing_capability_skips() {
    for mode in [
        "wrong-type",
        "invalid-uuid",
        "missing",
        "unknown",
        "unsupported",
    ] {
        let target = Target {
            mode,
            ..Default::default()
        };
        let expected = if mode == "unsupported" {
            Status::Unsupported
        } else {
            Status::Error
        };
        assert_eq!(run(&suite(), &target), expected, "{mode}");
        assert_eq!(target.begun.get(), 0, "{mode}");
        assert_eq!(target.ended.get(), 0, "{mode}");
        assert!(target.inputs.borrow().is_empty(), "{mode}");
    }
}

#[test]
fn values_are_resolved_once_and_isolated_per_scenario() {
    let mut suite = suite();
    let (id, scenario) = suite.scenarios.iter().next().unwrap();
    let id = id
        .to_string()
        .replace("/opened", "/opened-again")
        .parse()
        .unwrap();
    suite.scenarios.insert(id, scenario.clone());
    let target = Target::default();
    assert_eq!(run(&suite, &target), Status::Passed);
    assert_eq!(target.resolved.get(), 2);
    let inputs = target.inputs.borrow();
    assert_ne!(inputs[0]["email"], inputs[1]["email"]);
}

#[test]
fn fixture_admission_refuses_old_versions_unbound_references_and_late_preludes() {
    let suite = suite();
    let json = suite.to_canonical_json().unwrap();
    assert_eq!(suite.provenance.suite_version.major(), 12);
    for major in 1..12 {
        assert!(
            AdmittedSuite::from_json(
                &json.replace("ess-conformance/12", &format!("ess-conformance/{major}"))
            )
            .is_err(),
            "{major}"
        );
    }
    for mutation in ["missing", "late", "duplicate", "unreferenced"] {
        let mut suite = suite.clone();
        let steps = &mut suite.scenarios.values_mut().next().unwrap().steps;
        match mutation {
            "missing" => {
                steps.remove(0);
            }
            "late" => steps.swap(0, 1),
            "duplicate" => steps.push(steps[0].clone()),
            "unreferenced" => {
                if let ScenarioStep::ResolveFixtures { fixtures } = &mut steps[0] {
                    fixtures.fields[0].name = "unknown".parse().unwrap();
                }
            }
            _ => unreachable!(),
        }
        assert!(AdmittedSuite::from_suite(&suite).is_err(), "{mutation}");
    }
}

#[test]
fn source_refuses_old_format_unknown_fields_and_conflicting_fixture_types() {
    for text in [
        MODEL.replace("ess/7", "ess/6"),
        MODEL.replace(
            "email: current-principal-email",
            "undeclared: current-principal-email",
        ),
        MODEL.replace("current-principal-email", "current-principal-id"),
    ] {
        let raw = RawSpecFile::parse(&text).unwrap();
        assert!(Specification::assemble([(Source::new("fixtures.yaml"), raw)]).is_err());
    }
}

#[test]
fn fixture_inputs_cannot_replace_generated_guard_witnesses() {
    let text = MODEL.replace(
        "      - name: opened\n",
        "      - name: opened\n        when: email != \"\"\n",
    );
    let raw = RawSpecFile::parse(&text).unwrap();
    let error = Specification::assemble([(Source::new("guarded.yaml"), raw)]).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("cannot replace a field read by an outcome predicate"),
        "{error}"
    );
}

#[test]
fn fixture_inventory_and_filtered_parent_retain_new_vocabulary() {
    use ess_conformance::coverage::{AdmittedInput, Origins, Scope};
    let spec = Specification::assemble([(
        Source::new("fixtures.yaml"),
        RawSpecFile::parse(MODEL).unwrap(),
    )])
    .unwrap();
    let ir = compile(&spec, &SourceMap::new()).unwrap();
    let input = ess_conformance::coverage_build::build(&ir, &[], Scope::System, Origins::Generated)
        .unwrap();
    assert_eq!(
        input.selected().suite().provenance.suite_version.major(),
        13
    );
    let ids: Vec<_> = input.selected().suite().scenarios.keys().cloned().collect();
    let filtered = input.select(&ids).unwrap();
    let json = filtered.document().to_canonical_json().unwrap();
    let reparsed = AdmittedInput::from_json(&json).unwrap();
    assert_eq!(
        reparsed.selected().suite().provenance.suite_version.major(),
        13
    );
    assert_eq!(
        Runner::for_suite(reparsed.selected().suite())
            .run_admitted(reparsed.selected(), &Target::default())
            .scenarios[0]
            .status,
        Status::Passed
    );
    assert!(ess_conformance::web::emit(&ir, input.selected().suite())
        .unwrap_err()
        .to_string()
        .contains("fixture"));
}

#[test]
fn fixture_contract_rejects_recursive_unrelated_and_constrained_types() {
    let suite = suite();
    let ScenarioStep::ResolveFixtures { fixtures } =
        &suite.scenarios.values().next().unwrap().steps[0]
    else {
        panic!("fixture prelude")
    };
    let raw = serde_json::to_value(fixtures).unwrap();
    for mutation in [
        "recursive",
        "unrelated",
        "constraint",
        "duplicate",
        "binary64",
    ] {
        let mut changed = raw.clone();
        match mutation {
            "recursive" => {
                changed["declarations"]["fixturetest.session.PrincipalId"]["of"] =
                    "fixturetest.session.PrincipalId".into();
            }
            "unrelated" => {
                changed["declarations"]["fixturetest.session.Unused"] =
                    serde_json::json!({"kind":"newtype","of":"String"});
            }
            "constraint" => {
                changed["declarations"]["fixturetest.session.PrincipalId"]["invariants"] =
                    serde_json::json!(["value != null"]);
            }
            "duplicate" => {
                let field = changed["fields"][0].clone();
                changed["fields"].as_array_mut().unwrap().push(field);
            }
            "binary64" => changed["fields"][0]["type"] = "Binary64".into(),
            _ => unreachable!(),
        }
        assert!(
            serde_json::from_value::<ess_conformance::fixtures::Contract>(changed).is_err(),
            "{mutation}"
        );
    }
}

const AUTHORED: &str = r"
type: ess-scenario/3
domain: fixturetest.session
scenario: independent-principal
summary: A provisioned principal keeps independent expectations
fixtures:
  current-principal-id: fixturetest.session.PrincipalId
  current-principal-email: String
timeline:
  - at: '2026-09-22T00:00:00Z'
    command: fixturetest.session.Open
    input:
      principal_id: {$fixture: current-principal-id}
      email: {$fixture: current-principal-email}
      region: eu
    outcome: opened
    events:
      - event: fixturetest.session.Opened
        payload:
          principal_id: {$fixture: current-principal-id}
          email: {$fixture: current-principal-email}
          region: eu
";

fn authored(text: &str) -> ess_conformance::authored::Authoring {
    let spec = Specification::assemble([(
        Source::new("fixtures.yaml"),
        RawSpecFile::parse(MODEL).unwrap(),
    )])
    .unwrap();
    ess_conformance::authored::compile(
        &compile(&spec, &SourceMap::new()).unwrap(),
        &[ess_conformance::authored::Source::new(
            "authored.yaml",
            text,
        )],
    )
}

#[test]
fn authored_fixture_types_reach_equality_without_replacing_literal_inputs() {
    let result = authored(AUTHORED);
    assert!(result.refusals.is_empty(), "{:?}", result.refusals);
    let mut suite = suite();
    suite.scenarios = result.scenarios;
    suite.select_fresh_format();
    assert_eq!(run(&suite, &Target::default()), Status::Passed);
    assert_eq!(
        run(
            &suite,
            &Target {
                mode: "wrong-output",
                ..Default::default()
            }
        ),
        Status::Failed
    );
    assert_eq!(
        run(
            &suite,
            &Target {
                mode: "split-output",
                ..Default::default()
            }
        ),
        Status::Failed
    );
    let literal = AUTHORED
        .replace(
            "      email: {$fixture: current-principal-email}",
            "      email: literal@example.com",
        )
        .replace("  current-principal-email: String\n", "");
    let result = authored(&literal);
    assert!(result.refusals.is_empty(), "{:?}", result.refusals);
    let serialized = serde_json::to_string(&result.scenarios).unwrap();
    assert!(serialized.contains("literal@example.com"));
    assert!(!serialized.contains("current-principal-email"));
}

#[test]
fn authored_fixtures_refuse_old_formats_wrong_types_unknown_names_and_unused_declarations() {
    for text in [
        AUTHORED.replace("ess-scenario/3", "ess-scenario/2"),
        AUTHORED.replace(
            "current-principal-email: String",
            "current-principal-email: Boolean",
        ),
        AUTHORED.replace("$fixture: current-principal-email", "$fixture: unknown"),
        AUTHORED.replace("fixtures:\n", "fixtures:\n  unused: String\n"),
    ] {
        let result = authored(&text);
        assert!(!result.refusals.is_empty(), "{text}");
        assert!(result.scenarios.is_empty());
    }
}

#[test]
fn emitted_runtimes_preserve_independent_fixture_values_and_reject_mutations() {
    let suite = suite();
    native_fixture_cases(
        "ordinary",
        ess_conformance::go::emit(&suite).unwrap(),
        ess_conformance::ts::emit(&suite).unwrap(),
    );
}

#[test]
fn emitted_coverage_runtimes_preserve_fixtures_with_exact_parent_lineage() {
    use ess_conformance::coverage::{Origins, Scope};
    let input = ess_conformance::coverage_build::build(
        &fixture_ir(),
        &[],
        Scope::System,
        Origins::Generated,
    )
    .unwrap();
    let ids: Vec<_> = input.selected().suite().scenarios.keys().cloned().collect();
    let filtered = input.select(&ids).unwrap();
    native_fixture_cases(
        "coverage",
        ess_conformance::go::emit_input(&filtered).unwrap(),
        ess_conformance::ts::emit_input(&filtered).unwrap(),
    );
}

fn native_fixture_cases(
    label: &str,
    go: Vec<ess_conformance::go::GoArtifact>,
    ts: Vec<ess_conformance::ts::TsArtifact>,
) {
    let root = std::env::temp_dir().join(format!(
        "ess-pre-execution-fixtures-{label}-{}",
        std::process::id()
    ));
    let ts = fixture_packages(&root, go, ts);
    for mode in [
        "valid",
        "wrong-output",
        "split-output",
        "wrong-then-right",
        "invalid",
        "missing",
        "unknown",
        "unsupported",
    ] {
        for (tool, args, directory) in [
            ("go", vec!["test", "./essconform", "-count=1", "-v"], &root),
            ("node", vec!["--test", "fixtures.mjs"], &ts),
        ] {
            let output = std::process::Command::new(tool)
                .args(args)
                .env("ESS_FIXTURE_CASE", mode)
                .env("ESS_REPORT_FORMAT", "2")
                .env("GOWORK", "off")
                .env_remove("ESS_REPORT_OUT")
                .current_dir(directory)
                .output()
                .unwrap();
            let log = format!(
                "{}\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            std::fs::write(root.join(format!("{tool}-{mode}.log")), &log).unwrap();
            assert_eq!(
                output.status.success(),
                matches!(mode, "valid" | "unsupported"),
                "{label} {tool} {mode}: {log}"
            );
            let executes = matches!(
                mode,
                "valid" | "wrong-output" | "split-output" | "wrong-then-right"
            );
            assert_eq!(
                log.contains("fixture session begun"),
                executes,
                "{label} {tool} {mode}: {log}"
            );
            assert_eq!(
                log.contains("fixture session ended"),
                executes,
                "{label} {tool} {mode}: {log}"
            );
            if !executes {
                assert!(
                    log.contains("fixture values"),
                    "{label} {tool} {mode}: {log}"
                );
            }
        }
    }
    std::fs::remove_dir_all(root).unwrap();
}

fn fixture_packages(
    root: &std::path::Path,
    go: Vec<ess_conformance::go::GoArtifact>,
    ts: Vec<ess_conformance::ts::TsArtifact>,
) -> std::path::PathBuf {
    std::fs::create_dir_all(root).unwrap();
    for artifact in go {
        let path = root.join(artifact.path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, artifact.contents).unwrap();
    }
    std::fs::write(
        root.join("go.mod"),
        "module example.invalid/fixtures\n\ngo 1.24\n",
    )
    .unwrap();
    std::fs::write(
        root.join("essconform/fixtures_test.go"),
        include_str!("fixtures/pre-execution-fixtures-runtime.go"),
    )
    .unwrap();
    for artifact in ts {
        let path = root.join("typescript").join(artifact.path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, artifact.contents).unwrap();
    }
    let ts = root.join("typescript/essconform");
    std::fs::write(
        ts.join("fixtures.mjs"),
        include_str!("fixtures/pre-execution-fixtures-runtime.mjs"),
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
    ts
}

#[test]
fn source_declared_fixture_inputs_reach_requests_and_event_equalities() {
    let raw = RawSpecFile::parse(MODEL).expect("fixture source parses");
    let spec = Specification::assemble([(Source::new("fixtures.yaml"), raw)])
        .expect("fixture source validates");
    let ir = compile(&spec, &SourceMap::new()).unwrap();
    let synthesis = ess_conformance::synthesize::synthesize(&ir);
    assert!(synthesis.refusals.is_empty(), "{:?}", synthesis.refusals);
    let serialized = serde_json::to_value(&synthesis.suite).unwrap();
    let steps = serialized["scenarios"]["fixturetest.session.Open/outcome/opened"]["steps"]
        .as_array()
        .unwrap();
    assert_eq!(steps[0]["step"], "resolve_fixtures");
    let execute = steps
        .iter()
        .find(|step| step["step"] == "execute_command")
        .unwrap();
    assert_eq!(execute["input"]["principal_id"]["kind"], "fixture");
    assert_eq!(
        execute["input"]["principal_id"]["fixture"],
        "current-principal-id"
    );
    let equality = steps
        .iter()
        .find(|step| step["step"] == "expect_event_values")
        .expect("the resolved input must remain an equality assertion, not only a shape check");
    assert_eq!(
        equality["payload"]["principal_id"],
        execute["input"]["principal_id"]
    );
    assert_eq!(equality["payload"]["email"], execute["input"]["email"]);
}
