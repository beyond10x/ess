//! Retained replies are observed before retry rather than supplied by the target.
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::{report::Status, target::*, AdmittedSuite, Runner, ScenarioStep};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;
use std::{cell::Cell, collections::BTreeMap};

const MODEL: &str = include_str!("fixtures/retained-replay.yaml");

fn synthesis() -> ess_conformance::synthesize::Synthesis {
    let spec = Specification::assemble([(
        Source::new("replay.yaml"),
        RawSpecFile::parse(MODEL).unwrap(),
    )])
    .unwrap();
    ess_conformance::synthesize::synthesize(&compile(&spec, &SourceMap::new()).unwrap())
}

#[test]
fn replay_synthesis_captures_original_result_and_identity_before_real_retry() {
    let result = synthesis();
    assert!(result.refusals.is_empty(), "{:?}", result.refusals);
    let scenario = result
        .suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == "retained.core.Seed/outcome/replayed")
        .unwrap()
        .1;
    let text = serde_json::to_string(&scenario.steps).unwrap();
    assert!(text.contains("capture_command_result"), "{text}");
    assert!(text.contains("expect_replay_result"), "{text}");
    assert!(text.contains("capture_instance"), "{text}");
    assert!(text.contains("snapshot_complete_subject"), "{text}");
    assert!(!text.contains("configure_external_outcome"), "{text}");
    assert_eq!(result.suite.provenance.suite_version.major(), 12);
}

fn replay_suite() -> ess_conformance::ConformanceSuite {
    let mut suite = synthesis().suite;
    suite
        .scenarios
        .retain(|id, _| id.to_string().ends_with("/outcome/replayed"));
    suite
}
fn response() -> BTreeMap<String, Node> {
    BTreeMap::from([
        (
            "revision_id".into(),
            Node::Text("00000000-0000-4000-8000-000000000037".into()),
        ),
        ("stamp".into(), Node::Text("2026-09-22T01:02:03Z".into())),
        ("number".into(), native_integer(9_007_199_254_740_993)),
        (
            "values".into(),
            Node::Seq(vec![native_integer(i64::MIN), native_integer(i64::MAX)]),
        ),
    ])
}
fn native_integer(actual: i64) -> Node {
    Node::Number(ess_primitives::facts::Number::from(actual))
}
struct Backend {
    mode: &'static str,
    calls: Cell<usize>,
    queries: Cell<usize>,
}
impl ConformanceTarget for Backend {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("retained-fixture", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.calls.set(0);
        self.queries.set(0);
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let retry = self.calls.get() > 0;
        self.calls.set(self.calls.get() + 1);
        let mut result = SemanticCommandResult::took(ess_compiler::refs::OutcomeRef::new(
            request.command,
            if retry { "replayed" } else { "seeded" }.parse().unwrap(),
        ));
        result.consistency =
            Some(ess_primitives::consistency::ConsistencyToken::new("actual-write").unwrap());
        let mut response = response();
        if retry {
            match self.mode {
                "next-result" => {
                    response.insert(
                        "number".into(),
                        serde_json::from_str("9007199254740992").unwrap(),
                    );
                }
                "new-stamp" => {
                    response.insert("stamp".into(), Node::Text("2026-09-22T01:02:04Z".into()));
                }
                "null-optional" => {
                    response.insert("optional".into(), Node::Null);
                }
                "extra-field" => {
                    response.insert("extra".into(), Node::Null);
                }
                "reordered-list" => {
                    response.insert(
                        "values".into(),
                        serde_json::from_str("[9223372036854775807,-9223372036854775808]").unwrap(),
                    );
                }
                "error" => {
                    result.error = Some(DeclaredErrorValue {
                        error: "retained.core.Failed".parse().unwrap(),
                        fields: BTreeMap::new(),
                    });
                }
                _ => {}
            }
        }
        if !retry || self.mode != "missing-response" {
            result.response = Some(response);
        }
        if !retry || self.mode == "extra-event" {
            let mut event = ObservedEvent::new(
                if retry {
                    "retained.core.Undeclared"
                } else {
                    "retained.core.Seeded"
                }
                .parse()
                .unwrap(),
            );
            event.payload.insert(
                "record_id".into(),
                Node::Text("00000000-0000-4000-8000-000000000037".into()),
            );
            result.direct_events.push(event);
            if !retry && self.mode == "ambiguous-original-event" {
                result.direct_events.push(result.direct_events[0].clone());
            }
        }
        Ok(result)
    }
    fn query_view(&self, _: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let row: BTreeMap<String, Node> = serde_json::from_str(r#"{"record_id":"00000000-0000-4000-8000-000000000037","value":"actual document","stamp":"2026-09-22T01:02:03Z","state":"Committed"}"#).unwrap();
        let mut row = row;
        if self.calls.get() > 1 && self.mode == "mutated-subject" {
            row.insert("stamp".into(), Node::Text("2026-09-22T01:02:04Z".into()));
        }
        self.queries.set(self.queries.get() + 1);
        Ok(SemanticViewResult { rows: vec![row] })
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        panic!("replay must never use fault injection")
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
#[test]
fn actual_original_exact_result_and_complete_subject_are_deciding_observations() {
    let suite = AdmittedSuite::from_suite(&replay_suite()).unwrap();
    for mode in [
        "valid",
        "next-result",
        "new-stamp",
        "null-optional",
        "extra-field",
        "reordered-list",
        "missing-response",
        "extra-event",
        "error",
        "mutated-subject",
        "ambiguous-original-event",
    ] {
        let target = Backend {
            mode,
            calls: Cell::new(0),
            queries: Cell::new(0),
        };
        let result = Runner::for_suite(suite.suite()).run_admitted(&suite, &target);
        assert_eq!(
            result.scenarios[0].status,
            if mode == "valid" {
                Status::Passed
            } else {
                Status::Failed
            },
            "{mode}: {:?}",
            result.scenarios[0]
        );
        assert_eq!(
            target.calls.get(),
            2,
            "origin and real retry must run: {mode}"
        );
        assert_eq!(target.queries.get(), 2, "both snapshots must run: {mode}");
    }
}
#[test]
fn replay_envelope_refuses_old_labels_unknown_fields_and_unbound_or_overwritten_snapshots() {
    let suite = replay_suite();
    let json = suite.to_canonical_json().unwrap();
    for old in 1..12 {
        assert!(AdmittedSuite::from_json(
            &json.replace("ess-conformance/12", &format!("ess-conformance/{old}"))
        )
        .is_err());
    }
    for mutation in [
        "missing",
        "overwritten",
        "wrong-command",
        "wrong-snapshot",
        "wrong-instance",
    ] {
        let mut bad = suite.clone();
        let steps = &mut bad.scenarios.values_mut().next().unwrap().steps;
        let index = steps
            .iter()
            .position(|s| matches!(s, ScenarioStep::CaptureCommandResult { .. }))
            .unwrap();
        match mutation {
            "missing" => {
                steps.remove(index);
            }
            "overwritten" => steps.insert(index, steps[index].clone()),
            _ => {
                let capture = steps
                    .iter_mut()
                    .find_map(|s| match s {
                        ScenarioStep::ExpectReplayResult { capture } => Some(capture),
                        _ => None,
                    })
                    .unwrap();
                match mutation {
                    "wrong-command" => {
                        capture.replay.command = "retained.core.Other".parse().unwrap();
                    }
                    "wrong-snapshot" => capture.snapshot = "other".parse().unwrap(),
                    _ => capture.instance = "other".parse().unwrap(),
                }
            }
        }
        assert!(AdmittedSuite::from_suite(&bad).is_err(), "{mutation}");
    }
}
#[test]
fn replay_response_refuses_floating_representations_recursively() {
    for ty in [
        "Decimal",
        "Binary64",
        "Optional<Decimal>",
        "List<Binary64>",
        "Map<String, Decimal>",
    ] {
        let source = MODEL.replace("type: Integer}", &format!("type: '{ty}'}}"));
        let spec = Specification::assemble([(
            Source::new("retained.yaml"),
            RawSpecFile::parse(&source).unwrap(),
        )])
        .unwrap();
        let result =
            ess_conformance::synthesize::synthesize(&compile(&spec, &SourceMap::new()).unwrap());
        assert!(
            !result.refusals.is_empty()
                && !result
                    .suite
                    .scenarios
                    .keys()
                    .any(|id| id.to_string().ends_with("/outcome/replayed")),
            "{ty}"
        );
    }
}

#[test]
fn replay_requires_fresh_views_and_original_invocation_identity() {
    for mutation in [
        "no-origin-query",
        "no-retry-query",
        "late-snapshot",
        "wrong-field",
        "wrong-event",
        "stale-event-binding",
        "unknown-identity",
        "missing-identity",
    ] {
        let mut json: serde_json::Value =
            serde_json::from_str(&replay_suite().to_canonical_json().unwrap()).unwrap();
        let steps = json["scenarios"]
            .as_object_mut()
            .unwrap()
            .values_mut()
            .next()
            .unwrap()["steps"]
            .as_array_mut()
            .unwrap();
        let index = |steps: &[serde_json::Value], kind: &str| {
            steps.iter().position(|v| v["step"] == kind).unwrap()
        };
        match mutation {
            "no-origin-query" => {
                let n = index(steps, "query_view");
                steps.remove(n);
            }
            "no-retry-query" => {
                let n = steps
                    .iter()
                    .rposition(|v| v["step"] == "query_view")
                    .unwrap();
                steps.remove(n);
            }
            "late-snapshot" => {
                let n = index(steps, "snapshot_complete_subject");
                let snapshot = steps.remove(n);
                let n = index(steps, "expect_replay_result");
                steps.insert(n, snapshot);
            }
            "stale-event-binding" => {
                let n = index(steps, "capture_instance");
                let binding = steps.remove(n);
                steps.insert(0, binding);
            }
            _ => {
                for step in steps.iter_mut().filter(|s| s["capture"].is_object()) {
                    match mutation {
                        "wrong-field" => {
                            step["capture"]["identity"]["field"] = serde_json::json!("other_id");
                        }
                        "wrong-event" => {
                            step["capture"]["identity"]["event"] =
                                serde_json::json!("retained.core.Other");
                        }
                        "unknown-identity" => {
                            step["capture"]["identity"]["extra"] = serde_json::json!(true);
                        }
                        _ => {
                            step["capture"].as_object_mut().unwrap().remove("identity");
                        }
                    }
                }
            }
        }
        assert!(
            AdmittedSuite::from_json(&serde_json::to_string(&json).unwrap()).is_err(),
            "{mutation}"
        );
    }
}

#[test]
fn original_identity_reads_exact_input_or_one_direct_event() {
    use ess_conformance::replay::Identity;
    let expected = Node::Text("actual".into());
    let input = BTreeMap::from([("id".into(), expected.clone())]);
    let selected = Identity::Input { field: "id".into() };
    assert_eq!(selected.read(&input, &[]).unwrap(), &expected);
    assert!(selected.read(&BTreeMap::new(), &[]).is_err());
    let mut event = ObservedEvent::new("retained.core.Seeded".parse().unwrap());
    event.payload = input.clone();
    let created = Identity::Event {
        event: event.event.clone(),
        field: "id".into(),
    };
    assert_eq!(
        created.read(&input, std::slice::from_ref(&event)).unwrap(),
        &expected
    );
    assert!(created.read(&input, &[]).is_err());
    assert!(created.read(&input, &[event.clone(), event]).is_err());
}

#[test]
fn replay_identity_field_uses_the_declared_field_grammar() {
    let suite = replay_suite();
    let capture = suite
        .scenarios
        .values()
        .flat_map(|s| &s.steps)
        .find_map(|s| match s {
            ScenarioStep::CaptureCommandResult { capture } => Some(capture),
            _ => None,
        })
        .unwrap();
    for invalid in ["", "bad field", "_leading", "1leading", "nested.field"] {
        let mut malformed = capture.clone();
        malformed.identity = ess_conformance::replay::Identity::Input {
            field: invalid.into(),
        };
        assert!(malformed.validate().is_err(), "{invalid}");
    }
}

#[test]
fn move_replay_cannot_substitute_a_neighbour_for_original_input_identity() {
    let spec = Specification::assemble([(
        Source::new("commit.yaml"),
        RawSpecFile::parse(include_str!("fixtures/retained-commit.yaml")).unwrap(),
    )])
    .unwrap();
    let mut suite =
        ess_conformance::synthesize::synthesize(&compile(&spec, &SourceMap::new()).unwrap()).suite;
    suite
        .scenarios
        .retain(|id, _| id.to_string() == "retained.core.Commit/outcome/replayed");
    assert_eq!(suite.scenarios.len(), 1);
    AdmittedSuite::from_suite(&suite).unwrap();
    let mut bad = suite.clone();
    let steps = &mut bad.scenarios.values_mut().next().unwrap().steps;
    let original = steps
        .iter()
        .find_map(|s| match s {
            ScenarioStep::CaptureCommandResult { capture } => Some(capture.instance.clone()),
            _ => None,
        })
        .unwrap();
    let neighbour: ess_conformance::scenario::InstanceName = "neighbour".parse().unwrap();
    let mut binding = steps.iter().find(|s| matches!(s, ScenarioStep::CaptureInstance { instance, .. } if instance == &original)).unwrap().clone();
    if let ScenarioStep::CaptureInstance { instance, .. } = &mut binding {
        *instance = neighbour.clone();
    }
    let at = steps
        .iter()
        .position(|s| matches!(s, ScenarioStep::CaptureCommandResult { .. }))
        .unwrap();
    steps.insert(at, binding);
    for step in steps {
        match step {
            ScenarioStep::CaptureCommandResult { capture }
            | ScenarioStep::ExpectReplayResult { capture } => capture.instance = neighbour.clone(),
            ScenarioStep::SnapshotCompleteSubject { subject, .. } => {
                for value in subject.values_mut() {
                    if let ess_conformance::ScenarioValue::Instance { instance } = value {
                        *instance = neighbour.clone();
                    }
                }
            }
            _ => {}
        }
    }
    assert!(AdmittedSuite::from_suite(&bad).is_err());
}

#[test]
fn native_integer_adapter_preserves_adjacent_values_and_i64_endpoints() {
    for actual in [
        9_007_199_254_740_992,
        9_007_199_254_740_993,
        i64::MIN,
        i64::MAX,
    ] {
        let Node::Number(observed) = native_integer(actual) else {
            panic!("native Integer adapter returned another kind")
        };
        assert_eq!(
            observed.as_i64(),
            Some(actual),
            "native handler Integer must arrive exactly"
        );
    }
    let suite = replay_suite();
    let observation = suite
        .scenarios
        .values()
        .flat_map(|s| &s.steps)
        .find_map(|s| match s {
            ScenarioStep::CaptureCommandResult { capture } => Some(capture),
            _ => None,
        })
        .unwrap();
    for actual in [
        0,
        1,
        -1,
        9_007_199_254_740_992,
        9_007_199_254_740_993,
        i64::MIN,
        i64::MAX,
    ] {
        let mut value = response();
        value.insert("number".into(), native_integer(actual));
        let Node::Number(observed) = value["number"] else {
            panic!("native adapter must return a number")
        };
        assert_eq!(
            observed.as_i64(),
            Some(actual),
            "actual native handler value must survive observation"
        );
        observation.admit_result(Some(&value)).unwrap();
        let encoded = serde_json::to_string(&value).unwrap();
        let decoded = serde_json::from_str(&encoded).unwrap();
        observation.compare(&value, Some(&decoded)).unwrap();
        let mut wrong = value.clone();
        wrong.insert(
            "number".into(),
            native_integer(actual.checked_add(1).unwrap_or(i64::MAX - 1)),
        );
        assert!(
            observation.compare(&value, Some(&wrong)).is_err(),
            "{actual}"
        );
    }
}

#[test]
fn emit_replay_runtime_artifacts_when_requested() {
    let Some(root) = std::env::var_os("ESS_REPLAY_RUNTIME_OUT") else {
        return;
    };
    emit_runtime(&std::path::PathBuf::from(root));
}

#[test]
fn generated_go_replay_runtime_executes_actual_results_and_strict_admission() {
    let root = std::env::temp_dir().join(format!("ess-retained-go-{}", std::process::id()));
    emit_runtime(&root);
    let result = std::process::Command::new("go")
        .args(["test", "-json", "-count=1", "./essconform"])
        .env("GOWORK", "off")
        .env("GOPROXY", "off")
        .env("GOMAXPROCS", "2")
        .current_dir(&root)
        .output()
        .expect("required Go toolchain executes retained-result witnesses");
    let output = String::from_utf8(result.stdout).unwrap();
    let records: Vec<serde_json::Value> = output
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    let count = |action: &str| {
        records
            .iter()
            .filter(|r| r["Test"].is_string() && r["Action"] == action)
            .count()
    };
    println!(
        "Go retained-result runtime: {} passed; {} failed; {} skipped\n{output}",
        count("pass"),
        count("fail"),
        count("skip")
    );
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(count("pass") > 0, "an empty runtime lane proves nothing");
    assert_eq!((count("fail"), count("skip")), (0, 0));
    std::fs::remove_dir_all(root).unwrap();
}

fn external_stale_synthesis() -> ess_conformance::synthesize::Synthesis {
    let model = include_str!("fixtures/retained-commit.yaml");
    let start = model.find("  - name: retained.core.Stale\n").unwrap();
    let end = model.find("  - name: retained.core.Commit\n").unwrap();
    let model = format!("{}{}", &model[..start], &model[end..]).replace("      - name: replayed\n", "      - name: stale\n        external: canonical head advanced\n        moves: retained.core.Transaction.stale\n        instance: transaction_id\n        emits: [retained.core.Changed]\n      - name: replayed\n");
    let spec = Specification::assemble([(
        Source::new("external-stale.yaml"),
        RawSpecFile::parse(&model).unwrap(),
    )])
    .unwrap();
    ess_conformance::synthesize::synthesize(&compile(&spec, &SourceMap::new()).unwrap())
}

#[test]
fn external_stale_arrangement_preserves_real_commit_and_validate_refusal_obligations() {
    let result = external_stale_synthesis();
    assert!(result.refusals.is_empty(), "{:?}", result.refusals);
    let mut suite = result.suite;
    suite.scenarios.retain(|id, _| {
        id.to_string().starts_with("retained.core.Commit/")
            || id.to_string().ends_with("/refuses/retained.core.Commit")
            || id.to_string()
                == "retained.core.Transaction/state/Stale/refuses/retained.core.Validate"
    });
    assert!(suite.scenarios.keys().any(|id| id
        .to_string()
        .contains("/state/Stale/refuses/retained.core.Commit")));
    assert!(suite.scenarios.keys().any(|id| id
        .to_string()
        .contains("/state/Stale/refuses/retained.core.Validate")));
    let admitted = AdmittedSuite::from_suite(&suite).unwrap();
    let target = StateBackend {
        mode: "external-stale",
        state: std::cell::RefCell::new(String::new()),
        note: std::cell::RefCell::new(Node::Null),
        reached: std::cell::RefCell::new(Vec::new()),
        forced_stale: Cell::new(false),
    };
    let report = Runner::for_suite(admitted.suite()).run_admitted(&admitted, &target);
    assert!(
        report
            .scenarios
            .iter()
            .all(|scenario| scenario.status == Status::Passed),
        "{report:?}"
    );
}

#[test]
fn source7_named_wrong_state_refusals_require_complete_effect_free_observation() {
    let model = include_str!("fixtures/retained-commit.yaml");
    let start = model.find("  - name: retained.core.Commit\n").unwrap();
    let end = model.find("views:\n").unwrap();
    let model = format!("{}{}", &model[..start], &model[end..])
        .replace(", Committed", "")
        .replace(
            "        - {name: commit, from: [Validated], to: Committed}\n",
            "",
        );
    for version in [6, 7] {
        let text = model.replace("ess/7", &format!("ess/{version}"));
        let spec = Specification::assemble([(
            Source::new("wrong-state.yaml"),
            RawSpecFile::parse(&text).unwrap(),
        )])
        .unwrap();
        let ir = compile(&spec, &SourceMap::new()).unwrap();
        let result = ess_conformance::synthesize::synthesize(&ir);
        assert!(result.refusals.is_empty(), "{:?}", result.refusals);
        let scenario = result
            .suite
            .scenarios
            .iter()
            .find(|(id, _)| {
                id.to_string()
                    == "retained.core.Transaction/state/Stale/refuses/retained.core.Validate"
            })
            .unwrap()
            .1;
        assert_eq!(
            scenario
                .steps
                .iter()
                .any(|s| matches!(s, ScenarioStep::ExpectNoEvents)),
            version == 7
        );
        assert_eq!(
            scenario
                .steps
                .iter()
                .any(|s| matches!(s, ScenarioStep::SnapshotCompleteSubject { .. })),
            version == 7
        );
    }
    let incomplete = model.replace("      - {name: note, type: String}\n", "");
    let spec = Specification::assemble([(
        Source::new("incomplete.yaml"),
        RawSpecFile::parse(&incomplete).unwrap(),
    )])
    .unwrap();
    let result =
        ess_conformance::synthesize::synthesize(&compile(&spec, &SourceMap::new()).unwrap());
    assert!(
        !result.refusals.is_empty(),
        "an incomplete observer must refuse synthesis"
    );
}

fn emit_runtime(root: &std::path::Path) {
    std::fs::create_dir_all(root.join("essconform")).unwrap();
    std::fs::write(
        root.join("go.mod"),
        "module replay-conformance\n\ngo 1.25\n",
    )
    .unwrap();
    std::fs::write(
        root.join("essconform/retained_replay_test.go"),
        include_str!("fixtures/retained-replay-runtime.go"),
    )
    .unwrap();
    let suite = replay_suite();
    std::fs::write(root.join("suite.json"), suite.to_canonical_json().unwrap()).unwrap();
    let source = Specification::assemble([(
        Source::new("replay.yaml"),
        RawSpecFile::parse(MODEL).unwrap(),
    )])
    .unwrap();
    let ir = compile(&source, &SourceMap::new()).unwrap();
    let covered = ess_conformance::coverage_build::build(
        &ir,
        &[],
        ess_conformance::coverage::Scope::System,
        ess_conformance::coverage::Origins::Generated,
    )
    .unwrap();
    assert_eq!(
        covered.selected().suite().provenance.suite_version.major(),
        13
    );
    std::fs::write(
        root.join("coverage.json"),
        covered.selected().original_json(),
    )
    .unwrap();
    let commit = Specification::assemble([(
        Source::new("commit.yaml"),
        RawSpecFile::parse(include_str!("fixtures/retained-commit.yaml")).unwrap(),
    )])
    .unwrap();
    let mut commit =
        ess_conformance::synthesize::synthesize(&compile(&commit, &SourceMap::new()).unwrap())
            .suite;
    commit.scenarios.retain(|id, _| {
        id.to_string().starts_with("retained.core.Commit/")
            || id.to_string().ends_with("/refuses/retained.core.Commit")
    });
    std::fs::write(
        root.join("commit.json"),
        commit.to_canonical_json().unwrap(),
    )
    .unwrap();
    let mut external = external_stale_synthesis().suite;
    external.scenarios.retain(|id, _| {
        id.to_string().starts_with("retained.core.Commit/")
            || id.to_string().ends_with("/refuses/retained.core.Commit")
            || id.to_string()
                == "retained.core.Transaction/state/Stale/refuses/retained.core.Validate"
    });
    std::fs::write(
        root.join("external-stale.json"),
        external.to_canonical_json().unwrap(),
    )
    .unwrap();
    for artifact in ess_conformance::ts::emit(&suite).unwrap() {
        let path = root.join("typescript").join(artifact.path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, artifact.contents).unwrap();
    }
    for artifact in ess_conformance::go::emit(&suite).unwrap() {
        let path = root.join(artifact.path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, artifact.contents).unwrap();
    }
}

#[test]
fn finite_default_refusals_execute_every_rejected_held_state_without_effects() {
    let spec = Specification::assemble([(
        Source::new("commit.yaml"),
        RawSpecFile::parse(include_str!("fixtures/retained-commit.yaml")).unwrap(),
    )])
    .unwrap();
    let ir = compile(&spec, &SourceMap::new()).unwrap();
    let command = ir
        .commands()
        .get(&"retained.core.Commit".parse().unwrap())
        .unwrap();
    let refusal = command
        .outcomes
        .iter()
        .find(|o| o.name.as_str() == "refused")
        .unwrap();
    assert!(
        refusal.subject.is_none(),
        "selection must not manufacture a mutation"
    );
    let result = ess_conformance::synthesize::synthesize(&ir);
    let ids: Vec<_> = result
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .filter(|s| s.contains("/refuses/") && s.ends_with("retained.core.Commit"))
        .collect();
    for state in ["Proposed", "Rejected", "Stale"] {
        assert!(
            ids.iter().any(|id| id.contains(state)),
            "{state}: {ids:?}, refusals {:?}",
            result.refusals
        );
    }
    assert!(
        !ids.iter()
            .any(|id| id.contains("Committed") || id.contains("Validated")),
        "{ids:?}"
    );
    assert!(result.refusals.is_empty(), "{:?}", result.refusals);
}

struct StateBackend {
    mode: &'static str,
    state: std::cell::RefCell<String>,
    note: std::cell::RefCell<Node>,
    reached: std::cell::RefCell<Vec<String>>,
    forced_stale: Cell<bool>,
}
impl ConformanceTarget for StateBackend {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("held-state-fixture", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        *self.state.borrow_mut() = String::new();
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let name = request.command.to_string();
        let held = self.state.borrow().clone();
        let (outcome, state, event) = match name.as_str() {
            "retained.core.Propose" => {
                *self.note.borrow_mut() = request.input["note"].clone();
                ("proposed", "Proposed", Some("retained.core.Proposed"))
            }
            "retained.core.Validate" if held == "Proposed" => {
                ("validated", "Validated", Some("retained.core.Changed"))
            }
            "retained.core.Validate" => ("refused", held.as_str(), None),
            "retained.core.Reject" => ("rejected", "Rejected", Some("retained.core.Changed")),
            "retained.core.Stale" => ("stale", "Stale", Some("retained.core.Changed")),
            "retained.core.Commit" => {
                self.reached.borrow_mut().push(held.clone());
                match held.as_str() {
                    "Validated" if self.forced_stale.replace(false) => {
                        ("stale", "Stale", Some("retained.core.Changed"))
                    }
                    "Validated" => ("committed", "Committed", Some("retained.core.Changed")),
                    "Committed" => ("replayed", "Committed", None),
                    _ => (
                        "refused",
                        held.as_str(),
                        if self.mode == "refusal-event" {
                            Some("retained.core.Undeclared")
                        } else {
                            None
                        },
                    ),
                }
            }
            _ => panic!("unexpected command {name}"),
        };
        *self.state.borrow_mut() = state.into();
        let mut result = SemanticCommandResult::took(ess_compiler::refs::OutcomeRef::new(
            request.command,
            outcome.parse().unwrap(),
        ));
        result.consistency =
            Some(ess_primitives::consistency::ConsistencyToken::new("observed-write").unwrap());
        if outcome == "refused" {
            if self.mode != "missing-refusal" {
                result.error = Some(DeclaredErrorValue::new(
                    "retained.core.TransactionStateConflict".parse().unwrap(),
                ));
            }
            if self.mode == "refusal-mutation" {
                *self.note.borrow_mut() = Node::Text("changed by refusal".into());
            }
        }
        if outcome == "committed" || outcome == "replayed" {
            result.response = Some(serde_json::from_str(r#"{"revision":37}"#).unwrap());
        }
        if let Some(event) = event {
            let mut event = ObservedEvent::new(event.parse().unwrap());
            if outcome == "proposed" {
                event.payload.insert(
                    "transaction_id".into(),
                    Node::Text("00000000-0000-4000-8000-000000000037".into()),
                );
            }
            result.direct_events.push(event);
        }
        Ok(result)
    }
    fn query_view(&self, _: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        Ok(SemanticViewResult {
            rows: vec![BTreeMap::from([
                (
                    "transaction_id".into(),
                    Node::Text("00000000-0000-4000-8000-000000000037".into()),
                ),
                ("note".into(), self.note.borrow().clone()),
                ("state".into(), Node::Text(self.state.borrow().clone())),
            ])],
        })
    }
    fn configure_external_outcome(
        &self,
        request: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        assert_eq!(self.mode, "external-stale");
        assert_eq!(request.force.to_string(), "retained.core.Commit/stale");
        self.forced_stale.set(true);
        Ok(())
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
#[test]
fn all_five_held_states_execute_and_each_refusal_is_effect_free() {
    let spec = Specification::assemble([(
        Source::new("commit.yaml"),
        RawSpecFile::parse(include_str!("fixtures/retained-commit.yaml")).unwrap(),
    )])
    .unwrap();
    let mut suite =
        ess_conformance::synthesize::synthesize(&compile(&spec, &SourceMap::new()).unwrap()).suite;
    suite.scenarios.retain(|id, _| {
        id.to_string().starts_with("retained.core.Commit/")
            || id.to_string().ends_with("/refuses/retained.core.Commit")
    });
    let admitted = AdmittedSuite::from_suite(&suite).unwrap();
    for mode in [
        "valid",
        "refusal-event",
        "missing-refusal",
        "refusal-mutation",
    ] {
        let target = StateBackend {
            mode,
            state: std::cell::RefCell::new(String::new()),
            note: std::cell::RefCell::new(Node::Null),
            reached: std::cell::RefCell::new(Vec::new()),
            forced_stale: Cell::new(false),
        };
        let report = Runner::for_suite(admitted.suite()).run_admitted(&admitted, &target);
        if mode == "valid" {
            assert!(
                report.scenarios.iter().all(|s| s.status == Status::Passed),
                "{report:?}"
            );
        }
        for state in ["Proposed", "Rejected", "Stale"] {
            assert!(
                target.reached.borrow().iter().any(|s| s == state),
                "{state} was not executed"
            );
            let scenario = report
                .scenarios
                .iter()
                .find(|s| {
                    s.scenario
                        .to_string()
                        .contains(&format!("/state/{state}/refuses/"))
                })
                .unwrap();
            assert_eq!(
                scenario.status,
                if mode == "valid" {
                    Status::Passed
                } else {
                    Status::Failed
                },
                "{mode}: {scenario:?}"
            );
        }
        for state in ["Validated", "Committed"] {
            assert!(
                target.reached.borrow().iter().any(|s| s == state),
                "{state} was not executed"
            );
        }
    }
}

// Adversary cases: use the real generated witness with incomplete actual observations.
struct AdversaryProjectionBackend {
    inner: Backend,
    projection: &'static str,
}
impl ConformanceTarget for AdversaryProjectionBackend {
    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        self.inner.redeliver_event(request)
    }
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.inner.identity()
    }
    fn begin_scenario(&self, context: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.begin_scenario(context)
    }
    fn end_scenario(&self, context: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.end_scenario(context)
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        self.inner.execute_command(request)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let mut observed = self.inner.query_view(request)?;
        for row in &mut observed.rows {
            match self.projection {
                "identity-only" => row.retain(|field, _| field == "record_id"),
                "missing-stamp" => {
                    row.remove("stamp");
                }
                "wrong-stamp-type" => {
                    row.insert("stamp".into(), Node::Bool(false));
                }
                _ => {}
            }
        }
        Ok(observed)
    }
    fn configure_external_outcome(
        &self,
        request: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        self.inner.configure_external_outcome(request)
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        self.inner.observe_events(request)
    }
    fn observe_invocations(
        &self,
        request: InvocationObservationRequest,
    ) -> Result<Vec<ObservedInvocation>, TargetError> {
        self.inner.observe_invocations(request)
    }
}

#[test]
fn adversary_replay_requires_complete_actual_subject_rows_not_only_complete_view_declarations() {
    let suite = AdmittedSuite::from_suite(&replay_suite()).unwrap();
    for projection in [
        "complete",
        "identity-only",
        "missing-stamp",
        "wrong-stamp-type",
    ] {
        let target = AdversaryProjectionBackend {
            inner: Backend {
                mode: "valid",
                calls: Cell::new(0),
                queries: Cell::new(0),
            },
            projection,
        };
        let report = Runner::for_suite(suite.suite()).run_admitted(&suite, &target);
        assert_eq!(
            report.scenarios[0].status,
            if projection == "complete" { Status::Passed } else { Status::Failed },
            "{projection}: a complete schema cannot certify fields absent or ill-typed in both actual query rows: {:?}",
            report.scenarios[0]
        );
    }
}

#[test]
fn adversary_replay_must_reach_its_own_state_guard_after_the_origin() {
    let source = include_str!("fixtures/retained-commit.yaml").replace(
        "when_subject_state: Committed",
        "when_subject_state: Rejected",
    );
    let specification = Specification::assemble([(
        Source::new("unreachable-retained-state.yaml"),
        RawSpecFile::parse(&source).unwrap(),
    )])
    .unwrap();
    let result = ess_conformance::synthesize::synthesize(
        &compile(&specification, &SourceMap::new()).unwrap(),
    );
    let generated = result
        .suite
        .scenarios
        .keys()
        .any(|id| id.to_string() == "retained.core.Commit/outcome/replayed");
    assert!(
        !generated && !result.refusals.is_empty(),
        "the original commit ends in terminal Committed, so the same subject cannot satisfy replay's Rejected guard; generation must name the unavailable witness, not invoke it in Committed: {:?}",
        result.refusals
    );
}

struct AdversaryRefusalProjectionBackend {
    inner: StateBackend,
}
impl ConformanceTarget for AdversaryRefusalProjectionBackend {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.inner.identity()
    }
    fn begin_scenario(&self, context: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.begin_scenario(context)
    }
    fn end_scenario(&self, context: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.end_scenario(context)
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        self.inner.execute_command(request)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let mut observed = self.inner.query_view(request)?;
        for row in &mut observed.rows {
            row.remove("note");
        }
        Ok(observed)
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
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        self.inner.observe_events(request)
    }
    fn observe_invocations(
        &self,
        request: InvocationObservationRequest,
    ) -> Result<Vec<ObservedInvocation>, TargetError> {
        self.inner.observe_invocations(request)
    }
}

#[test]
fn adversary_source7_wrong_state_refusal_requires_actual_complete_subject_observation() {
    let mut suite = external_stale_synthesis().suite;
    suite.scenarios.retain(|id, _| {
        id.to_string() == "retained.core.Transaction/state/Stale/refuses/retained.core.Validate"
    });
    assert_eq!(suite.scenarios.len(), 1);
    let admitted = AdmittedSuite::from_suite(&suite).unwrap();
    let target = AdversaryRefusalProjectionBackend {
        inner: StateBackend {
            mode: "external-stale",
            state: std::cell::RefCell::new(String::new()),
            note: std::cell::RefCell::new(Node::Null),
            reached: std::cell::RefCell::new(Vec::new()),
            forced_stale: Cell::new(false),
        },
    };
    let report = Runner::for_suite(admitted.suite()).run_admitted(&admitted, &target);
    assert_eq!(report.scenarios[0].status, Status::Failed,
        "the new source7 refusal witness must not certify preservation of an omitted actual subject field: {:?}", report.scenarios[0]);
}

#[test]
fn adversary_go_replay_requires_complete_actual_subject_rows() {
    let root =
        std::env::temp_dir().join(format!("ess-adversary-retained-go-{}", std::process::id()));
    emit_runtime(&root);
    std::fs::write(
        root.join("essconform/adversary_projection_test.go"),
        include_str!("fixtures/adversary-retained-projection.go"),
    )
    .unwrap();
    let result = std::process::Command::new("go")
        .args([
            "test",
            "-json",
            "-count=1",
            "./essconform",
            "-run",
            "^TestAdversarySnapshotProjection$",
        ])
        .env("GOWORK", "off")
        .env("GOPROXY", "off")
        .env("GOMAXPROCS", "2")
        .current_dir(&root)
        .output()
        .unwrap();
    println!(
        "{}\n{}\nretained fixture: {}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr),
        root.display()
    );
    assert!(
        result.status.success(),
        "Go must reject incomplete actual replay subject rows"
    );
}

#[test]
fn replay_conditions_must_hold_for_the_original_input_not_a_new_candidate() {
    for retry in [
        "when: document == 'different'",
        "external: retained context\n        when: document == 'different'",
        "",
    ] {
        let mut source = MODEL
            .replace(
                "- name: seeded\n",
                "- name: seeded\n        when: document == 'original'\n",
            )
            .replace(
                "external: retained logical input and trusted context match",
                retry,
            );
        if !retry.is_empty() {
            source = source.replace("events:\n", "      - name: refused\n        error: retained.core.Refused\nerrors:\n  - name: retained.core.Refused\n    fields: []\nevents:\n");
        }
        let specification = Specification::assemble([(
            Source::new("same-input.yaml"),
            RawSpecFile::parse(&source).unwrap(),
        )])
        .unwrap();
        let result = ess_conformance::synthesize::synthesize(
            &compile(&specification, &SourceMap::new()).unwrap(),
        );
        assert!(
            !result
                .suite
                .scenarios
                .keys()
                .any(|id| id.to_string() == "retained.core.Seed/outcome/replayed"),
            "{retry}: retry changed the original input's selection"
        );
        assert!(!result.refusals.is_empty());
    }
}

struct CompleteRowsBackend {
    inner: Backend,
    phase: usize,
    fault: &'static str,
}
impl ConformanceTarget for CompleteRowsBackend {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.inner.identity()
    }
    fn begin_scenario(&self, context: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.begin_scenario(context)
    }
    fn end_scenario(&self, context: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.end_scenario(context)
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        self.inner.execute_command(request)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let mut result = self.inner.query_view(request)?;
        if self.phase == 0 || self.phase == self.inner.calls.get() {
            for row in &mut result.rows {
                match self.fault {
                    "missing-value" => {
                        row.remove("value");
                    }
                    "missing-stamp" => {
                        row.remove("stamp");
                    }
                    "missing-state" => {
                        row.remove("state");
                    }
                    "wrong-stamp" => {
                        row.insert("stamp".into(), Node::Bool(false));
                    }
                    "wrong-state" => {
                        row.insert("state".into(), Node::Text("Undeclared".into()));
                    }
                    "extra" => {
                        row.insert("extra".into(), Node::Text("retained extra".into()));
                    }
                    "null-optional" => {
                        row.insert("optional".into(), Node::Null);
                    }
                    _ => {}
                }
            }
        }
        Ok(result)
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
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        self.inner.observe_events(request)
    }
    fn observe_invocations(
        &self,
        request: InvocationObservationRequest,
    ) -> Result<Vec<ObservedInvocation>, TargetError> {
        self.inner.observe_invocations(request)
    }
}

#[test]
fn original_complete_rows_refuse_before_any_retry_callback() {
    complete_row_faults_at(1);
}

#[test]
fn post_command_complete_rows_execute_typed_admission_before_equality() {
    complete_row_faults_at(2);
}

fn complete_row_faults_at(phase: usize) {
    let suite = AdmittedSuite::from_suite(&replay_suite()).unwrap();
    for fault in [
        "missing-value",
        "missing-stamp",
        "missing-state",
        "wrong-stamp",
        "wrong-state",
    ] {
        let target = CompleteRowsBackend {
            inner: Backend {
                mode: "valid",
                calls: Cell::new(0),
                queries: Cell::new(0),
            },
            phase,
            fault,
        };
        let report = Runner::for_suite(suite.suite()).run_admitted(&suite, &target);
        assert_eq!(
            report.scenarios[0].status,
            Status::Failed,
            "{phase}/{fault}"
        );
        assert_eq!(
            target.inner.calls.get(),
            phase,
            "typed original admission must stop before retry: {fault}"
        );
        assert!(
            report.scenarios[0]
                .checks
                .iter()
                .any(|check| check
                    .diagnostic
                    .as_ref()
                    .is_some_and(
                        |diagnostic| diagnostic.expected.iter().any(|expected| expected
                            == "an actual row satisfying its complete declared shape")
                    )),
            "{phase}/{fault}: generic inequality is not evidence of typed admission"
        );
    }
}

#[test]
fn complete_rows_preserve_optional_absence_null_and_extra_keys() {
    let mut suite = replay_suite();
    for step in &mut suite.scenarios.values_mut().next().unwrap().steps {
        if let ScenarioStep::SnapshotCompleteSubject { shape, .. } = step {
            shape.fields.push(ess_domain::Field::new(
                "optional",
                "Optional<String>".parse().unwrap(),
            ));
        }
    }
    let suite = AdmittedSuite::from_suite(&suite).unwrap();
    for (phase, fault, expected) in [
        (0, "none", Status::Passed),
        (0, "null-optional", Status::Passed),
        (2, "null-optional", Status::Failed),
        (0, "extra", Status::Passed),
        (2, "extra", Status::Failed),
    ] {
        let target = CompleteRowsBackend {
            inner: Backend {
                mode: "valid",
                calls: Cell::new(0),
                queries: Cell::new(0),
            },
            phase,
            fault,
        };
        let report = Runner::for_suite(suite.suite()).run_admitted(&suite, &target);
        assert_eq!(report.scenarios[0].status, expected, "{phase}/{fault}");
        assert_eq!(target.inner.calls.get(), 2);
    }
}

#[test]
fn complete_shape_recursively_checks_collections_and_named_variants() {
    let mut shape = replay_suite()
        .scenarios
        .values()
        .next()
        .unwrap()
        .steps
        .iter()
        .find_map(|step| match step {
            ScenarioStep::SnapshotCompleteSubject { shape, .. } => Some(shape.clone()),
            _ => None,
        })
        .unwrap();
    shape.fields.push(ess_domain::Field::new(
        "nested",
        "List<Map<String, Integer>>".parse().unwrap(),
    ));
    shape.fields.push(ess_domain::Field::new(
        "optional",
        "Optional<String>".parse().unwrap(),
    ));
    let mut row: BTreeMap<String, Node> = serde_json::from_str(r#"{"record_id":"00000000-0000-4000-8000-000000000037","value":"actual","stamp":"2026-09-22T01:02:03Z","state":"Committed","nested":[{"a":9007199254740993}],"extra":true}"#).unwrap();
    shape.admit_row(&row).unwrap();
    row.insert("optional".into(), Node::Null);
    shape.admit_row(&row).unwrap();
    row.insert(
        "nested".into(),
        serde_json::from_str(r#"[{"a":false}]"#).unwrap(),
    );
    assert!(shape.admit_row(&row).is_err());
    row.insert(
        "nested".into(),
        serde_json::from_str(r#"[{"a":1}]"#).unwrap(),
    );
    row.insert("state".into(), Node::Text("NotAState".into()));
    assert!(shape.admit_row(&row).is_err());
}

#[test]
fn complete_descriptor_and_pair_refuse_weakening_before_callbacks() {
    for mutation in [
        "missing-shape",
        "missing-fields",
        "unknown-shape-field",
        "optional-identity",
        "unknown-identity",
        "empty-selector",
        "wrong-selector",
        "wrong-view",
        "legacy-before",
        "legacy-after",
        "missing-before",
        "missing-after",
        "unrelated-declaration",
        "old-envelope",
    ] {
        let mut json: serde_json::Value =
            serde_json::from_str(&replay_suite().to_canonical_json().unwrap()).unwrap();
        let steps = json["scenarios"]
            .as_object_mut()
            .unwrap()
            .values_mut()
            .next()
            .unwrap()["steps"]
            .as_array_mut()
            .unwrap();
        let before = steps
            .iter()
            .position(|step| step["step"] == "snapshot_complete_subject")
            .unwrap();
        let after = steps
            .iter()
            .position(|step| step["step"] == "expect_complete_subject_unchanged")
            .unwrap();
        match mutation {
            "missing-shape" => {
                steps[before].as_object_mut().unwrap().remove("shape");
            }
            "missing-fields" => {
                steps[before]["shape"]
                    .as_object_mut()
                    .unwrap()
                    .remove("fields");
            }
            "unknown-shape-field" => steps[before]["shape"]["invented"] = true.into(),
            "optional-identity" => {
                steps[before]["shape"]["fields"][0]["type"] = "Optional<Uuid>".into();
            }
            "unknown-identity" => steps[before]["shape"]["identity_field"] = "other".into(),
            "empty-selector" => steps[before]["subject"] = serde_json::json!({}),
            "wrong-selector" => {
                steps[before]["subject"] =
                    serde_json::json!({"value":{"kind":"instance","instance":"record"}});
            }
            "wrong-view" => steps[after]["view"] = "retained.core.Other".into(),
            "legacy-before" => {
                steps[before]["step"] = "snapshot_subject".into();
                steps[before].as_object_mut().unwrap().remove("shape");
            }
            "legacy-after" => steps[after]["step"] = "expect_subject_unchanged".into(),
            "missing-before" => {
                steps.remove(before);
            }
            "missing-after" => {
                steps.remove(after);
            }
            "unrelated-declaration" => {
                steps[before]["shape"]["declarations"]["retained.core.Unrelated"] =
                    serde_json::json!({"kind":"enum","variants":["Unused"]});
            }
            "old-envelope" => json["provenance"]["suite_version"] = "ess-conformance/10".into(),
            _ => unreachable!(),
        }
        assert!(
            AdmittedSuite::from_json(&serde_json::to_string(&json).unwrap()).is_err(),
            "{mutation}"
        );
    }
}

#[test]
fn legacy_preservation_and_subjectless_errors_coexist_with_complete_snapshots() {
    let spec = Specification::assemble([(
        Source::new("history.yaml"),
        RawSpecFile::parse(include_str!("fixtures/subject-history.yaml")).unwrap(),
    )])
    .unwrap();
    let legacy =
        ess_conformance::synthesize::synthesize(&compile(&spec, &SourceMap::new()).unwrap());
    assert!(legacy.refusals.is_empty());
    let legacy_bytes = legacy.suite.to_canonical_json().unwrap();
    assert!(legacy_bytes.contains("snapshot_subject"));
    assert!(!legacy_bytes.contains("snapshot_complete_subject"));
    let mut mixed = replay_suite();
    for (id, scenario) in legacy.suite.scenarios {
        mixed.insert(id, scenario).unwrap();
    }
    AdmittedSuite::from_suite(&mixed).unwrap();
    let mut generic = replay_suite();
    generic.scenarios.values_mut().next().unwrap().steps = vec![
        ScenarioStep::ExecuteCommand {
            command: "retained.core.Seed".parse().unwrap(),
            actor: None,
            input: BTreeMap::new(),
        },
        ScenarioStep::ExpectError {
            error: "retained.core.Refused".parse().unwrap(),
            fields: BTreeMap::new(),
        },
        ScenarioStep::ExpectNoEvents,
    ];
    AdmittedSuite::from_suite(&generic).unwrap();
}

#[test]
fn complementary_complete_views_observe_the_actual_subject() {
    let source = MODEL.replace("      - {name: stamp, type: Timestamp}\n      - {name: state", "      - {name: state") + "  - name: retained.core.Stamps\n    source: retained.core.Record\n    consistency: read_your_writes\n    fields:\n      - {name: record_id, type: Uuid}\n      - {name: stamp, type: Timestamp}\n";
    let spec = Specification::assemble([(
        Source::new("complementary.yaml"),
        RawSpecFile::parse(&source).unwrap(),
    )])
    .unwrap();
    let mut result =
        ess_conformance::synthesize::synthesize(&compile(&spec, &SourceMap::new()).unwrap());
    assert!(result.refusals.is_empty(), "{:?}", result.refusals);
    result
        .suite
        .scenarios
        .retain(|id, _| id.to_string().ends_with("/outcome/replayed"));
    assert_eq!(
        result
            .suite
            .scenarios
            .values()
            .next()
            .unwrap()
            .steps
            .iter()
            .filter(|step| matches!(step, ScenarioStep::SnapshotCompleteSubject { .. }))
            .count(),
        2
    );
    let admitted = AdmittedSuite::from_suite(&result.suite).unwrap();
    let target = Backend {
        mode: "valid",
        calls: Cell::new(0),
        queries: Cell::new(0),
    };
    let report = Runner::for_suite(admitted.suite()).run_admitted(&admitted, &target);
    assert_eq!(report.scenarios[0].status, Status::Passed);
}

#[test]
fn unsupported_complete_subject_observers_refuse_by_name() {
    for ty in [
        "Decimal",
        "Binary64",
        "Optional<Decimal>",
        "List<Binary64>",
        "Map<String, Decimal>",
    ] {
        let source = MODEL
            .replace(
                "      - {name: stamp, type: Timestamp}\n    lifecycle:",
                &format!("      - {{name: stamp, type: '{ty}'}}\n    lifecycle:"),
            )
            .replace(
                "      - {name: stamp, type: Timestamp}\n      - {name: state",
                &format!("      - {{name: stamp, type: '{ty}'}}\n      - {{name: state"),
            );
        let spec = Specification::assemble([(
            Source::new("unsupported-subject.yaml"),
            RawSpecFile::parse(&source).unwrap(),
        )])
        .unwrap();
        let result =
            ess_conformance::synthesize::synthesize(&compile(&spec, &SourceMap::new()).unwrap());
        assert!(!result.refusals.is_empty());
        assert!(
            !result
                .suite
                .scenarios
                .keys()
                .any(|id| id.to_string().ends_with("/outcome/replayed")),
            "{ty}"
        );
    }
}

#[test]
fn replay_eligibility_keeps_external_input_and_actual_state_observation() {
    for guard in ["true", "document != 'different'"] {
        let source = MODEL.replace(
            "external: retained logical input and trusted context match",
            &format!(
                "external: retained logical input and trusted context match\n        when: {guard}"
            ),
        );
        let spec = Specification::assemble([(
            Source::new("external-eligibility.yaml"),
            RawSpecFile::parse(&source).unwrap(),
        )])
        .unwrap();
        let result =
            ess_conformance::synthesize::synthesize(&compile(&spec, &SourceMap::new()).unwrap());
        assert!(result.refusals.is_empty(), "{guard}: {:?}", result.refusals);
        assert!(result
            .suite
            .scenarios
            .keys()
            .any(|id| id.to_string().ends_with("/outcome/replayed")));
    }
    let spec = Specification::assemble([(
        Source::new("commit.yaml"),
        RawSpecFile::parse(include_str!("fixtures/retained-commit.yaml")).unwrap(),
    )])
    .unwrap();
    let result =
        ess_conformance::synthesize::synthesize(&compile(&spec, &SourceMap::new()).unwrap());
    let replay = result
        .suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == "retained.core.Commit/outcome/replayed")
        .unwrap()
        .1;
    let capture = replay
        .steps
        .iter()
        .position(|step| matches!(step, ScenarioStep::CaptureCommandResult { .. }))
        .unwrap();
    let retry = replay.steps[capture..]
        .iter()
        .position(|step| matches!(step, ScenarioStep::ExecuteCommand { .. }))
        .unwrap()
        + capture;
    assert!(replay.steps[capture..retry].iter().any(|step| matches!(step, ScenarioStep::ExpectView { expectation: ess_conformance::scenario::ViewExpectation::Contains { fields }, .. } if fields.get("state").and_then(ess_conformance::ScenarioValue::as_literal) == Some(&Node::Text("Committed".into())))), "replay must observe its declared held state after origin capture");
}

struct AdversaryR2StateObservation {
    inner: StateBackend,
    wrong_post_state: bool,
}

impl ConformanceTarget for AdversaryR2StateObservation {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.inner.identity()
    }
    fn begin_scenario(&self, context: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.begin_scenario(context)
    }
    fn end_scenario(&self, context: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.end_scenario(context)
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        self.inner.execute_command(request)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let mut result = self.inner.query_view(request)?;
        if self.wrong_post_state && *self.inner.state.borrow() == "Committed" {
            for row in &mut result.rows {
                row.insert("state".into(), Node::Text("Rejected".into()));
            }
        }
        Ok(result)
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
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        self.inner.observe_events(request)
    }
    fn observe_invocations(
        &self,
        request: InvocationObservationRequest,
    ) -> Result<Vec<ObservedInvocation>, TargetError> {
        self.inner.observe_invocations(request)
    }
}

#[test]
fn adversary_r2_replay_checks_actual_guard_state_before_retry() {
    let specification = Specification::assemble([(
        Source::new("actual-state.yaml"),
        RawSpecFile::parse(include_str!("fixtures/retained-commit.yaml")).unwrap(),
    )])
    .unwrap();
    let mut result = ess_conformance::synthesize::synthesize(
        &compile(&specification, &SourceMap::new()).unwrap(),
    );
    result
        .suite
        .scenarios
        .retain(|id, _| id.to_string() == "retained.core.Commit/outcome/replayed");
    assert_eq!(result.suite.scenarios.len(), 1);
    let suite = AdmittedSuite::from_suite(&result.suite).unwrap();
    for wrong_post_state in [false, true] {
        let target = AdversaryR2StateObservation {
            inner: StateBackend {
                mode: "valid",
                state: std::cell::RefCell::default(),
                note: std::cell::RefCell::new(Node::Null),
                reached: std::cell::RefCell::default(),
                forced_stale: Cell::new(false),
            },
            wrong_post_state,
        };
        let report = Runner::for_suite(suite.suite()).run_admitted(&suite, &target);
        assert_eq!(
            report.scenarios[0].status,
            if wrong_post_state {
                Status::Failed
            } else {
                Status::Passed
            },
            "a valid enum value is insufficient when the replay guard requires Committed"
        );
        if wrong_post_state {
            let checks = &report.scenarios[0].checks;
            let guard_failure = checks
                .iter()
                .position(|check| {
                    check.status == Status::Failed
                        && check.diagnostic.as_ref().is_some_and(|diagnostic| {
                            diagnostic
                                .expected
                                .iter()
                                .any(|text| text.contains("Committed"))
                        })
                })
                .expect("the actual expected state must produce the failed check");
            let replay_result = checks
                .iter()
                .position(|check| check.about == "outcome retained.core.Commit/replayed")
                .unwrap();
            assert!(guard_failure < replay_result);
        }
    }
}

#[test]
fn adversary_r2_complete_rows_validate_union_payload_and_exact_integer_bounds() {
    let shape: ess_conformance::subject::SubjectShape = serde_json::from_str(
        r#"{
            "identity_field":"record_id",
            "fields":[{"name":"record_id","type":"Uuid"},{"name":"payload","type":"List<retained.core.Payload>"}],
            "declarations":{
                "retained.core.Payload":{"kind":"union","tag":"kind","variants":{"counts":"Map<String, Integer>","choice":"retained.core.Choice"}},
                "retained.core.Choice":{"kind":"enum","variants":["First","Second"]}
            }
        }"#,
    )
    .unwrap();
    for (payload, valid) in [
        (
            r#"[{"kind":"counts","value":{"min":-9223372036854775808,"large":9007199254740993,"max":9223372036854775807}}]"#,
            true,
        ),
        (r#"[{"kind":"choice","value":"Second"}]"#, true),
        (
            r#"[{"kind":"counts","value":{"overflow":9223372036854775808}}]"#,
            false,
        ),
        (r#"[{"kind":"counts","value":{"wrong":false}}]"#, false),
        (r#"[{"kind":"choice","value":"Unknown"}]"#, false),
        (r#"[{"kind":"choice"}]"#, false),
        (r#"[{"kind":"unknown","value":{}}]"#, false),
    ] {
        let row = BTreeMap::from([
            (
                "record_id".into(),
                Node::Text("00000000-0000-4000-8000-000000000037".into()),
            ),
            ("payload".into(), serde_json::from_str(payload).unwrap()),
        ]);
        assert_eq!(shape.admit_row(&row).is_ok(), valid, "{payload}");
    }
}
