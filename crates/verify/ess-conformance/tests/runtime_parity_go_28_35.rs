//! Actual generated Go callbacks, compared with the native frozen suite contracts.
mod support_go;
mod support_go_prerequisite;

use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::{
    authored,
    coverage::{AdmittedInput, Origins, Scope},
    coverage_build::{build, CoverageSource},
    AdmittedSuite, ConformanceSuite,
};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use serde_json::{json, Value};

enum Input {
    Ordinary(AdmittedSuite),
    Covered(AdmittedInput),
}
impl Input {
    fn selected(&self) -> &AdmittedSuite {
        match self {
            Self::Ordinary(s) => s,
            Self::Covered(i) => i.selected(),
        }
    }
    fn package(&self, label: &str, extra: &[(&str, &str)]) -> std::path::PathBuf {
        match self {
            Self::Ordinary(s) => support_go::package(label, s.suite(), extra),
            Self::Covered(i) => support_go::package_input(label, i, extra),
        }
    }
}

fn constant(source: &str, name: &str) -> String {
    let prefix = format!("const {name}: &str = r\"");
    source
        .split_once(&prefix)
        .unwrap()
        .1
        .split_once("\";")
        .unwrap()
        .0
        .to_owned()
}
fn model(text: &str) -> ess_compiler::EssIr {
    let raw = RawSpecFile::parse(text).unwrap();
    let spec = Specification::assemble([(Source::new("parity.yaml"), raw)]).unwrap();
    compile(&spec, &SourceMap::new()).unwrap()
}
fn direct(covered: bool) -> Input {
    let source = include_str!("direct_returns.rs");
    let ir = model(&constant(source, "MODEL"));
    let timeline = constant(source, "SCENARIO");
    if covered {
        return Input::Covered(
            build(
                &ir,
                &[CoverageSource::new("return.yaml", timeline).unwrap()],
                Scope::System,
                Origins::GeneratedAndAuthored,
            )
            .unwrap(),
        );
    }
    let mut suite = ess_conformance::synthesize(&ir).suite;
    suite.scenarios =
        authored::compile(&ir, &[authored::Source::new("return.yaml", timeline)]).scenarios;
    suite.select_fresh_format();
    Input::Ordinary(AdmittedSuite::from_suite(&suite).unwrap())
}
fn delivery(covered: bool) -> Input {
    let ir = model(include_str!("fixtures/delivery-context.yaml"));
    let ids: Vec<_> = ["delivery", "flow", "mapping", "on-failure"]
        .map(|name| format!("received/binding/{name}").parse().unwrap())
        .into();
    if covered {
        return Input::Covered(
            build(&ir, &[], Scope::System, Origins::Generated)
                .unwrap()
                .select(&ids)
                .unwrap(),
        );
    }
    let mut suite = ess_conformance::synthesize(&ir).suite;
    suite.scenarios.retain(|id, _| ids.contains(id));
    Input::Ordinary(AdmittedSuite::from_suite(&suite).unwrap())
}
fn structured(covered: bool) -> Input {
    let source = include_str!("authored_structured_instances.rs");
    let ir = model(include_str!("fixtures/structured-instances.yaml"));
    let timeline = format!(
        "{}{}assert:\n  - view: release.rings.Rollouts\n    contains: {{ring_sequence: [{{$instance: a}}, {{$instance: b}}]}}\n",
        constant(source, "ARRANGED"),
        constant(source, "PLANNED")
    );
    if covered {
        return Input::Covered(
            build(
                &ir,
                &[CoverageSource::new("rollout.yaml", timeline).unwrap()],
                Scope::System,
                Origins::Authored,
            )
            .unwrap(),
        );
    }
    let mut suite = ConformanceSuite::new(ess_conformance::SuiteProvenance::of(&ir));
    suite.scenarios =
        authored::compile(&ir, &[authored::Source::new("rollout.yaml", timeline)]).scenarios;
    suite.select_fresh_format_for(&ir);
    Input::Ordinary(AdmittedSuite::from_suite(&suite).unwrap())
}

const DRIVER: &str = r#"package essconform
import("encoding/json"; "fmt"; "os"; "testing"; "net")
type parityTarget struct{}
func call(method string,args any,result any)error{
 socket,err:=net.Dial("tcp",os.Getenv("PARITY_ADDRESS"));if err!=nil{return err};defer socket.Close()
 if err=json.NewEncoder(socket).Encode(map[string]any{"method":method,"args":args});err!=nil{return err}
 var response struct{Ok json.RawMessage;Error string;Unsupported bool};if err=json.NewDecoder(socket).Decode(&response);err!=nil{return err}
 if response.Unsupported{return fmt.Errorf("%w: %s",ErrUnsupported,response.Error)};if response.Error!=""{return fmt.Errorf("%s",response.Error)}
 if result!=nil{decoder:=json.NewDecoder(bytes.NewReader(response.Ok));decoder.UseNumber();return decoder.Decode(result)};return nil
}
func(parityTarget) Identity()(value Identity,err error){err=call("identity",nil,&value);return}
func(parityTarget) BeginScenario(value ScenarioContext)error{return call("begin",value,nil)}
func(parityTarget) EndScenario(value ScenarioContext)error{return call("end",value,nil)}
func(parityTarget) ExecuteCommand(request CommandRequest)(value CommandResult,err error){err=call("execute",request,&value);return}
func(parityTarget) QueryView(request ViewRequest)(value ViewResult,err error){err=call("query",request,&value);return}
func(parityTarget) ObserveEvents(request EventObservationRequest)(value []ObservedEvent,err error){err=call("events",request,&value);return}
func(parityTarget) ObserveInvocations(request InvocationObservationRequest)(value []Invocation,err error){err=call("invocations",request,&value);return}
func(parityTarget) RedeliverEvent(value RedeliveryRequest)error{return call("redeliver",value,nil)}
func(parityTarget) ConfigureExternalOutcome(value ExternalOutcomeControl)error{return call("configure",value,nil)}
func(parityTarget) DeliverEvent(value EventDeliveryRequest)error{return call("deliver",value,nil)}
func TestParity(t *testing.T){if path:=os.Getenv("PARITY_DOCUMENT");path!=""{raw,err:=os.ReadFile(path);if err!=nil{t.Fatal(err)};suiteJSON=string(raw)};Run(t,func()Target{return parityTarget{}})}
func TestProducerBoundaries(t *testing.T){
 suite,err:=admitRunInput(suiteJSON);if err!=nil{t.Fatal(err)}
 results:=[]scenarioResult{};for id:=range suite.Scenarios{results=append(results,scenarioResult{id:id,status:statusSkipped})}
 document,err:=countDocument(suite,Identity{Name:"serialization-control",Version:"1"},results,0);if err!=nil{t.Fatal(err)}
 counts:=document["counts"].(map[string]uint64);if counts["skipped"]!=uint64(len(results))||counts["unsupported"]!=0||document["execution_status"]!="inconclusive"{t.Fatalf("explicit skipped collapsed: %v",document)}
 for _,test:=range []struct{status,want string}{{statusUnsupported,"inconclusive"},{statusError,"failed"}}{
  legacy:=append([]scenarioResult{},results...);for i:=range legacy{legacy[i].status=test.status}
  writeReport(t,suite,Identity{Name:"legacy",Version:"1"},legacy,len(legacy));raw,err:=os.ReadFile(os.Getenv("ESS_REPORT_OUT"));if err!=nil{t.Fatal(err)};var report map[string]any;if err=json.Unmarshal(raw,&report);err!=nil{t.Fatal(err)}
  if report["format"]!="ess-conformance-report/1"||report["status"]!=test.want{t.Fatalf("legacy boundary changed: %s",raw)}
 }
 writeCountReport(t,suite,Identity{Name:"serialization-control",Version:"1"},results,len(results),false)
}
"#;

#[test]
fn go_executes_direct_responses_and_reports_target_statuses() {
    for covered in [false, true] {
        let input = direct(covered);
        let directory = input.package(
            "direct-prerequisite",
            &[(
                "parity_test.go",
                &DRIVER.replace("\"net\")", "\"net\"; \"bytes\")"),
            )],
        );
        for (mode, status) in [
            ("correct", "passed"),
            ("unsupported", "unsupported"),
            ("unsupported-end-error", "error"),
            ("wrong-end-error", "failed"),
            ("error", "error"),
            ("missing", "failed"),
            ("extra", "failed"),
            ("rounded", "failed"),
            ("reorder", "failed"),
        ] {
            let target = support_go_prerequisite::Fixture::new("direct", mode);
            let native = ess_conformance::Runner::for_suite(input.selected().suite())
                .run_admitted(input.selected(), &target);
            if !covered || !["rounded", "reorder", "wrong-end-error"].contains(&mode) {
                assert!(
                    native
                        .scenarios
                        .iter()
                        .all(|scenario| scenario.status.to_string() == status),
                    "native {mode}: {:?}",
                    native.scenarios
                );
            }
            let expected: std::collections::BTreeMap<_, _> = native
                .scenarios
                .iter()
                .map(|scenario| (scenario.scenario.to_string(), scenario.status.to_string()))
                .collect();
            let host = support_go_prerequisite::Host::start("direct", mode);
            let result = support_go::go_test(
                &directory,
                "TestParity",
                &[("PARITY_ADDRESS", &host.address)],
            );
            let callbacks = host.stop();
            assert_eq!(
                result.outcomes.len(),
                if covered { 2 } else { 1 },
                "{}",
                result.log
            );
            assert_eq!(result.outcomes, expected, "{mode}: {}", result.log);
            compare_report(&directory, input.selected(), &native, &result, mode);
            assert_eq!(
                callbacks,
                *target.trace.lock().unwrap(),
                "{mode} callbacks differ"
            );
        }
    }
}

#[test]
fn go_generates_delivery_context_suites_30_and_31() {
    for covered in [false, true] {
        let input = delivery(covered);
        assert_eq!(input.selected().suite().scenarios.len(), 4);
        let directory = input.package(
            "delivery-prerequisite",
            &[(
                "parity_test.go",
                &DRIVER.replace("\"net\")", "\"net\"; \"bytes\")"),
            )],
        );
        for mode in [
            "correct",
            "ignore-context",
            "stale-context",
            "late-wrong",
            "observation-unsupported",
            "observation-error",
            "unsupported",
            "error",
        ] {
            let target = support_go_prerequisite::Fixture::new("delivery", mode);
            let native = ess_conformance::Runner::for_suite(input.selected().suite())
                .run_admitted(input.selected(), &target);
            let expected: std::collections::BTreeMap<_, _> = native
                .scenarios
                .iter()
                .map(|scenario| (scenario.scenario.to_string(), scenario.status.to_string()))
                .collect();
            let host = support_go_prerequisite::Host::start("delivery", mode);
            let result = support_go::go_test(
                &directory,
                "TestParity",
                &[("PARITY_ADDRESS", &host.address)],
            );
            let callbacks = host.stop();
            assert_eq!(result.outcomes, expected, "{mode}: {}", result.log);
            compare_report(&directory, input.selected(), &native, &result, mode);
            assert_eq!(
                callbacks
                    .iter()
                    .filter(|entry| entry["method"] == "deliver")
                    .collect::<Vec<_>>(),
                target
                    .trace
                    .lock()
                    .unwrap()
                    .iter()
                    .filter(|entry| entry["method"] == "deliver")
                    .collect::<Vec<_>>()
            );
        }
    }
}

#[test]
fn go_generates_structured_value_suites_32_and_33() {
    for covered in [false, true] {
        let input = structured(covered);
        assert_eq!(input.selected().suite().scenarios.len(), 1);
        let directory = input.package(
            "structured-prerequisite",
            &[(
                "parity_test.go",
                &DRIVER.replace("\"net\")", "\"net\"; \"bytes\")"),
            )],
        );
        for mode in ["correct", "reversed"] {
            let target = support_go_prerequisite::Fixture::new("structured", mode);
            let native = ess_conformance::Runner::for_suite(input.selected().suite())
                .run_admitted(input.selected(), &target);
            assert!(native.scenarios.iter().all(|scenario| scenario.status
                == if mode == "correct" {
                    ess_conformance::report::Status::Passed
                } else {
                    ess_conformance::report::Status::Failed
                }));
            let host = support_go_prerequisite::Host::start("structured", mode);
            let result = support_go::go_test(
                &directory,
                "TestParity",
                &[("PARITY_ADDRESS", &host.address)],
            );
            let callbacks = host.stop();
            compare_report(&directory, input.selected(), &native, &result, mode);
            assert_eq!(result.outcomes.len(), 1);
            let commands = |trace: &[Value]| {
                trace
                    .iter()
                    .filter(|entry| entry["method"] == "execute")
                    .cloned()
                    .collect::<Vec<_>>()
            };
            assert_eq!(
                commands(&callbacks),
                commands(&target.trace.lock().unwrap())
            );
        }
    }
}

#[test]
fn malformed_direct_authority_is_refused_by_native_before_execution() {
    let input = direct(false);
    let mut value: Value = serde_json::from_str(input.selected().original_json()).unwrap();
    value["provenance"]["suite_version"] = json!("ess-conformance/26");
    assert!(AdmittedSuite::from_json(&value.to_string()).is_err());
}

#[test]
fn explicit_skipped_and_legacy_report_boundaries_keep_their_categories() {
    let input = direct(false);
    let directory = input.package("report-boundaries", &[("parity_test.go", &driver())]);
    let result = support_go::go_test(&directory, "TestProducerBoundaries", &[]);
    assert!(result.success, "{}", result.log);
    assert!(result.outcomes.values().all(|status| status == "skipped"));
    let raw = std::fs::read_to_string(directory.join("report.json")).unwrap();
    let report = ess_conformance::counts::CountReport::from_json(&raw, input.selected()).unwrap();
    assert_eq!(report.counts().skipped, 1);
    assert_eq!(report.counts().unsupported, 0);
}

fn driver() -> String {
    DRIVER.replace("\"net\")", "\"net\"; \"bytes\")")
}

fn compare_report(
    directory: &std::path::Path,
    admitted: &AdmittedSuite,
    native: &ess_conformance::runner::ExecutedRun,
    go: &support_go::GoRun,
    mode: &str,
) {
    use ess_conformance::counts::CountReport;
    let text = std::fs::read_to_string(directory.join("report.json")).unwrap();
    let document: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(document["producer_profile"], "go-scenario-status/2");
    let actual = CountReport::from_json(&text, admitted).unwrap();
    let expected = CountReport::from_run(native, admitted).unwrap();
    assert_eq!(actual.counts(), expected.counts(), "{mode}");
    assert_eq!(
        actual.execution_status(),
        expected.execution_status(),
        "{mode}"
    );
    assert_eq!(
        actual.conformance_status(),
        expected.conformance_status(),
        "{mode}"
    );
    assert_eq!(
        go.success,
        native
            .scenarios
            .iter()
            .all(|scenario| scenario.status == ess_conformance::report::Status::Passed),
        "{mode}: {}",
        go.log
    );
    eprintln!(
        "parity mode={mode} suite={} outcomes={:?} counts={:?} exit_success={}",
        admitted.suite().provenance.suite_version,
        go.outcomes,
        actual.counts(),
        go.success
    );
}

#[test]
fn direct_profile_preserves_presence_and_payload_local_bounds() {
    let original = constant(include_str!("direct_returns.rs"), "MODEL");
    for (edit,mode,wanted) in [
        (original.clone(),"large",ess_conformance::report::Status::Passed),
        (original.clone(),"oversized",ess_conformance::report::Status::Failed),
        (original.replace("name: value, type: String","name: value, type: Json"),"json-128",ess_conformance::report::Status::Passed),
        (original.replace("name: value, type: String","name: value, type: Json"),"json-129",ess_conformance::report::Status::Failed),
        (original.replace("      - {name: ordinal, type: Integer}","      - {name: ordinal, type: Integer}\n      - {name: note, type: 'Optional<String>', presence: null_when_absent}"),"note-null",ess_conformance::report::Status::Passed),
        (original.replace("      - {name: ordinal, type: Integer}","      - {name: ordinal, type: Integer}\n      - {name: note, type: 'Optional<String>', presence: null_when_absent}"),"correct",ess_conformance::report::Status::Failed),
        (original.replace("      - {name: ordinal, type: Integer}","      - {name: ordinal, type: Integer}\n      - {name: note, type: 'Optional<String>', presence: omitted_when_absent}"),"note-null",ess_conformance::report::Status::Failed),
        (original.replace("      - {name: ordinal, type: Integer}","      - {name: ordinal, type: Integer}\n      - {name: note, type: 'Optional<String>', presence: omitted_when_absent}"),"correct",ess_conformance::report::Status::Passed),
    ] {
        let suite=ess_conformance::synthesize(&model(&edit)).suite;
        let admitted=AdmittedSuite::from_suite(&suite).unwrap();
        let target=support_go_prerequisite::Fixture::new("direct",mode);
        let native=ess_conformance::Runner::for_suite(&suite).run_admitted(&admitted,&target);assert_eq!(native.scenarios[0].status,wanted,"{mode}");
        let directory=support_go::package("direct-bounds",&suite,&[("parity_test.go",&driver())]);
        let host=support_go_prerequisite::Host::start("direct",mode);let result=support_go::go_test(&directory,"TestParity",&[("PARITY_ADDRESS",&host.address)]);host.stop();
        assert_eq!(result.outcomes.len(),1,"{mode}: {}",result.log);assert_eq!(result.outcomes.values().next().unwrap(),&wanted.to_string(),"{mode}: {}",result.log);
    }
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "Keep each corrupt document next to its common zero-callback assertion"
)]
fn malformed_prerequisite_documents_refuse_before_all_callbacks() {
    for (kind, input, old) in [
        ("direct", direct(false), 26),
        ("delivery", delivery(false), 28),
        ("structured", structured(false), 30),
    ] {
        let directory = input.package("admission-prerequisite", &[("parity_test.go", &driver())]);
        let original: Value = serde_json::from_str(input.selected().original_json()).unwrap();
        let mut corruptions = Vec::new();
        let mut older = original.clone();
        older["provenance"]["suite_version"] = json!(format!("ess-conformance/{old}"));
        corruptions.push(older);
        let mut unknown = original.clone();
        let steps = unknown["scenarios"]
            .as_object_mut()
            .unwrap()
            .values_mut()
            .next()
            .unwrap()["steps"]
            .as_array_mut()
            .unwrap();
        steps[0]["unrecognized"] = json!(true);
        corruptions.push(unknown);
        if kind == "direct" {
            let mut invalid = original.clone();
            let steps = invalid["scenarios"]
                .as_object_mut()
                .unwrap()
                .values_mut()
                .next()
                .unwrap()["steps"]
                .as_array_mut()
                .unwrap();
            let field = &mut steps
                .iter_mut()
                .find(|step| step["step"] == "expect_direct_response")
                .unwrap()["response"]["fields"][0];
            field["wire"] = json!("wire_name");
            field["naming"] = json!({"display":"Display name"});
            corruptions.push(invalid);
            for field in ["command", "declarations"] {
                let mut invalid = original.clone();
                let steps = invalid["scenarios"]
                    .as_object_mut()
                    .unwrap()
                    .values_mut()
                    .next()
                    .unwrap()["steps"]
                    .as_array_mut()
                    .unwrap();
                let response = steps
                    .iter_mut()
                    .find(|step| step["step"] == "expect_direct_response")
                    .unwrap();
                response["response"][field] = if field == "command" {
                    json!("library.api.Other")
                } else {
                    json!({})
                };
                corruptions.push(invalid);
            }
        }
        if kind == "structured" {
            let mut invalid = original.clone();
            let steps = invalid["scenarios"]
                .as_object_mut()
                .unwrap()
                .values_mut()
                .next()
                .unwrap()["steps"]
                .as_array_mut()
                .unwrap();
            let command = steps
                .iter_mut()
                .find(|step| step["command"] == "release.rings.PlanRollout")
                .unwrap();
            command["input"]["ring_sequence"]["items"][0] =
                json!({"kind":"now_offset","seconds":1});
            corruptions.push(invalid);
        }
        for (index, invalid) in corruptions.iter().enumerate() {
            let raw = invalid.to_string();
            assert!(
                AdmittedSuite::from_json(&raw).is_err(),
                "native admitted {kind}/{index}"
            );
            let path = directory.join("invalid.json");
            std::fs::write(&path, raw).unwrap();
            let host = support_go_prerequisite::Host::start(kind, "correct");
            let result = support_go::go_test(
                &directory,
                "TestParity",
                &[
                    ("PARITY_ADDRESS", &host.address),
                    ("PARITY_DOCUMENT", path.to_str().unwrap()),
                ],
            );
            let trace = host.stop();
            assert!(!result.success, "Go admitted {kind}/{index}");
            assert!(result.outcomes.is_empty(), "invalid suite published report");
            assert!(
                trace.is_empty(),
                "invalid suite reached callbacks: {trace:?}"
            );
        }
    }
}

#[test]
fn serialized_direct_literals_have_the_native_payload_local_depth_budget() {
    let model_text = constant(include_str!("direct_returns.rs"), "MODEL")
        .replace("name: value, type: String", "name: value, type: Json");
    let suite = ess_conformance::synthesize(&model(&model_text)).suite;
    let directory = support_go::package("direct-depth", &suite, &[("parity_test.go", &driver())]);
    for (depth, payload_local, accepted) in
        [(128, true, true), (129, true, false), (128, false, false)]
    {
        let mut wire = serde_json::to_value(&suite).unwrap();
        let nested = (0..depth).fold(Value::Null, |value, _| json!([value]));
        let steps = wire["scenarios"]
            .as_object_mut()
            .unwrap()
            .values_mut()
            .next()
            .unwrap()["steps"]
            .as_array_mut()
            .unwrap();
        if payload_local {
            let response = &mut steps
                .iter_mut()
                .find(|step| step["step"] == "expect_direct_response")
                .unwrap()["response"];
            response["expected"]["value"] = nested;
            response["fields"][0]["naming"] =
                json!({"display":"Nested naming authority", "code":"ValueCode"});
            response["fields"][0]["wire"] = Value::Null;
            response["fields"][0]["presence"] = Value::Null;
        } else {
            // A similarly named ordinary payload must not receive the direct-return reset.
            steps[0]["input"] =
                json!({"response":{"kind":"literal","value":{"expected":{"value":nested}}}});
        }
        let text = wire.to_string();
        let admission = AdmittedSuite::from_json(&text);
        assert_eq!(
            admission.is_ok(),
            accepted,
            "depth={depth}, local={payload_local}: {admission:?}"
        );
        let path = directory.join("depth.json");
        std::fs::write(&path, &text).unwrap();
        let host = support_go_prerequisite::Host::start("direct", "json-128");
        let result = support_go::go_test(
            &directory,
            "TestParity",
            &[
                ("PARITY_ADDRESS", &host.address),
                ("PARITY_DOCUMENT", path.to_str().unwrap()),
            ],
        );
        let trace = host.stop();
        if let Ok(admitted) = admission {
            let target = support_go_prerequisite::Fixture::new("direct", "json-128");
            let native = ess_conformance::Runner::for_suite(admitted.suite())
                .run_admitted(&admitted, &target);
            compare_report(&directory, &admitted, &native, &result, "deep literal");
            assert!(result.success, "{}", result.log);
        } else {
            assert!(!result.success);
            assert!(trace.is_empty(), "invalid deep suite reached {trace:?}");
        }
    }
}

#[test]
fn assertion_failure_does_not_turn_a_later_execution_error_into_continuation() {
    for tag in ["expect_invocation", "expect_every_invocation"] {
        let input = delivery(false);
        let mut suite = input.selected().suite().clone();
        suite
            .scenarios
            .retain(|id, _| id.to_string() == "received/binding/delivery");
        let mut wire = serde_json::to_value(&suite).unwrap();
        let scenario = wire["scenarios"]
            .as_object_mut()
            .unwrap()
            .values_mut()
            .next()
            .unwrap();
        let original = scenario["steps"].as_array().unwrap();
        let deliver = original
            .iter()
            .find(|step| step["step"] == "deliver_event")
            .unwrap()
            .clone();
        let mut assertion = original
            .iter()
            .find(|step| step["step"] == "expect_every_invocation")
            .unwrap()
            .clone();
        assertion["step"] = json!(tag);
        assertion["selecting"] = json!({});
        if tag == "expect_invocation" {
            assertion.as_object_mut().unwrap().remove("selecting");
        }
        scenario["steps"] = json!([deliver.clone(), assertion.clone(), assertion, deliver]);
        let admitted =
            AdmittedSuite::from_suite(AdmittedSuite::from_json(&wire.to_string()).unwrap().suite())
                .unwrap();
        let target = support_go_prerequisite::Fixture::new("delivery", "failure-then-error");
        let native =
            ess_conformance::Runner::for_suite(admitted.suite()).run_admitted(&admitted, &target);
        assert_eq!(
            native.scenarios[0].status,
            ess_conformance::report::Status::Failed
        );
        let directory = support_go::package(
            "stop-after-error",
            admitted.suite(),
            &[("parity_test.go", &driver())],
        );
        let host = support_go_prerequisite::Host::start("delivery", "failure-then-error");
        let result = support_go::go_test(
            &directory,
            "TestParity",
            &[("PARITY_ADDRESS", &host.address)],
        );
        let trace = host.stop();
        compare_report(&directory, &admitted, &native, &result, tag);
        assert_eq!(
            trace,
            *target.trace.lock().unwrap(),
            "{tag}: extra callbacks after execution error"
        );
    }
}

#[test]
fn unresolved_structured_instances_are_execution_errors_before_the_command() {
    let input = structured(false);
    let mut wire: Value = serde_json::from_str(input.selected().original_json()).unwrap();
    let scenario = wire["scenarios"]
        .as_object_mut()
        .unwrap()
        .values_mut()
        .next()
        .unwrap();
    let command = scenario["steps"]
        .as_array()
        .unwrap()
        .iter()
        .find(|step| step["command"] == "release.rings.PlanRollout")
        .unwrap()
        .clone();
    scenario["steps"] = json!([command]);
    let admitted =
        AdmittedSuite::from_suite(AdmittedSuite::from_json(&wire.to_string()).unwrap().suite())
            .unwrap();
    let target = support_go_prerequisite::Fixture::new("structured", "correct");
    let native =
        ess_conformance::Runner::for_suite(admitted.suite()).run_admitted(&admitted, &target);
    assert_eq!(
        native.scenarios[0].status,
        ess_conformance::report::Status::Error
    );
    let directory = support_go::package(
        "unresolved-structured",
        admitted.suite(),
        &[("parity_test.go", &driver())],
    );
    let host = support_go_prerequisite::Host::start("structured", "correct");
    let result = support_go::go_test(
        &directory,
        "TestParity",
        &[("PARITY_ADDRESS", &host.address)],
    );
    let trace = host.stop();
    compare_report(
        &directory,
        &admitted,
        &native,
        &result,
        "unresolved structured",
    );
    assert_eq!(trace, *target.trace.lock().unwrap());
    assert!(!trace.iter().any(|entry| entry["method"] == "execute"));
}
