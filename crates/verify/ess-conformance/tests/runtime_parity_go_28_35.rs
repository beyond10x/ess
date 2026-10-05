//! Actual generated Go callbacks, compared with the native frozen suite contracts.
mod support_go;
mod support_go_one_time;
mod support_go_prerequisite;
mod support_one_time;

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
import("encoding/json"; "fmt"; "os"; "testing"; "reflect"; "net")
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
func TestOneTimeAdmission(t *testing.T){
 root:=os.Getenv("PARITY_ADMISSION_VECTORS");entries,err:=os.ReadDir(root);if err!=nil{t.Fatal(err)}
 if len(entries)==0{t.Fatal("missing admission vectors")}
 for _,entry:=range entries{t.Run(entry.Name(),func(t *testing.T){
  raw,err:=os.ReadFile(root+"/"+entry.Name());if err!=nil{t.Fatal(err)};value,err:=strictSuiteJSON(string(raw));if err!=nil{t.Fatal(err)}
  document:=value.(map[string]any);admitted:=true
  for id,rawScenario:=range document["scenarios"].(map[string]any){scenario:=rawScenario.(map[string]any);major:=0;if _,err:=fmt.Sscanf(document["provenance"].(map[string]any)["suite_version"].(string),"ess-conformance/%d",&major);err!=nil{t.Fatal(err)};policy,err:=admitOneTimeTrace(scenario,major);if err!=nil{t.Log(err);admitted=false;continue}
   if os.Getenv("PARITY_IDENTIFIERS")!=""{if err:=admitOneTimeCell(id,scenario,policy,major);err!=nil{t.Log(err);admitted=false;continue}}
   if canonicalRoot:=os.Getenv("PARITY_CANONICAL_VECTORS");canonicalRoot!=""{expectedRaw,err:=os.ReadFile(canonicalRoot+"/"+entry.Name());if err==nil{expected,err:=strictSuiteJSON(string(expectedRaw));if err!=nil{t.Fatal(err)};wanted:=expected.(map[string]any)["scenarios"].(map[string]any)[id].(map[string]any)["one_time_response"];actualWire,err:=directWire(policy);if err!=nil{t.Fatal(err)};actual,err:=strictJSON(string(actualWire));if err!=nil{t.Fatal(err)};if !reflect.DeepEqual(actual,wanted){t.Fatalf("canonical policy differs:\nGo %s\nnative %v",actualWire,wanted)}}}
  }
  expected:=len(entry.Name())>=6&&entry.Name()[:6]=="valid-";if admitted!=expected{t.Fatalf("native admission=%v, Go policy admission=%v",expected,admitted)}
 })}
}
func TestOneTimeIdentifierGrammar(t *testing.T){
 raw,err:=os.ReadFile(os.Getenv("PARITY_IDENTIFIER_GRAMMAR"));if err!=nil{t.Fatal(err)}
 var vectors [][]any;if err=json.Unmarshal(raw,&vectors);err!=nil{t.Fatal(err)};if len(vectors)!=13{t.Fatal("identifier vectors changed")}
 for _,vector:=range vectors{expected:=vector[0].(bool);written:=vector[1].(string);_,err:=oneTimeCellIdentity(written);if (err==nil)!=expected || (scenarioIdentity(written)==nil)!=expected{t.Errorf("%s: policy identity admission=%v, coverage identity admission=%v, expected=%v",written,err==nil,scenarioIdentity(written)==nil,expected)}}
}
func TestOneTimeFullAdmission(t *testing.T){
 root:=os.Getenv("PARITY_ADMISSION_VECTORS");entries,err:=os.ReadDir(root);if err!=nil{t.Fatal(err)}
 for _,entry:=range entries{t.Run(entry.Name(),func(t *testing.T){raw,err:=os.ReadFile(root+"/"+entry.Name());if err!=nil{t.Fatal(err)};_,err=admitRunInput(string(raw));expected:=len(entry.Name())>=6&&entry.Name()[:6]=="valid-";if (err==nil)!=expected{t.Fatalf("native admission=%v, actual runtime admission=%v: %v",expected,err==nil,err)}})}
}
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

fn one_time_driver() -> String {
    let driver = driver().replace(" if result!=nil{", r#"
 if method=="execute" && os.Getenv("PARITY_ORPHAN_ERROR")!="" {
  var value map[string]json.RawMessage; if err=json.Unmarshal(response.Ok,&value);err!=nil{return err}
  if len(firstOneTime)==0 {
   var fields map[string]json.RawMessage; if err=json.Unmarshal(value["Response"],&fields);err!=nil{return err}; firstOneTime=fields["secret"]
  }else{
   value["ErrorPayload"],err=json.Marshal(map[string]json.RawMessage{"copied":firstOneTime});if err!=nil{return err};response.Ok,err=json.Marshal(value);if err!=nil{return err}
  }
 }
 if result!=nil{"#);
    format!(
        "{}\n{}",
        driver,
        r#"
var firstOneTime json.RawMessage
func(parityTarget) MarkInstant(mark InstantMark)error{return call("mark",mark,nil)}
func(parityTarget) ObserveElapsed(request ElapsedRequest)(value ElapsedObservation,err error){err=call("elapsed",request,&value);return}
"#
    )
}

#[test]
fn orphan_error_payload_is_still_an_actual_disclosure_surface() {
    // Native DeclaredErrorValue cannot represent this malformed pairing. The Go adapter
    // mutant copies a real earlier returned field into ErrorPayload with an empty name.
    let root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/one-time-execution");
    let manifest: Vec<Value> =
        serde_json::from_str(&std::fs::read_to_string(root.join("manifest.json")).unwrap())
            .unwrap();
    let mut expected = manifest
        .into_iter()
        .find(|case| case["case"] == "healthy")
        .unwrap();
    expected["status"] = json!("failed");
    expected["counts"]["passed"] = json!(0);
    expected["counts"]["failed"] = json!(1);
    expected["case"] = json!("adapter-orphan-error-payload");
    let path = root.join("healthy.json");
    let admitted = AdmittedSuite::from_json(&std::fs::read_to_string(&path).unwrap()).unwrap();
    let directory = support_go::package(
        "one-time-orphan-error",
        admitted.suite(),
        &[("parity_test.go", &one_time_driver())],
    );
    let host = support_go_one_time::Host::start(support_one_time::Mode::Healthy);
    let result = support_go::go_test(
        &directory,
        "TestParity",
        &[
            ("PARITY_ADDRESS", &host.address),
            ("PARITY_DOCUMENT", path.to_str().unwrap()),
            ("PARITY_ORPHAN_ERROR", "1"),
        ],
    );
    let snapshot = host.stop();
    check_one_time_execution(&directory, &expected, &admitted, &result, &snapshot).unwrap();
}

#[test]
fn go_executes_the_frozen_one_time_observer_protocol_controls() {
    let root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/one-time-execution");
    let manifest: Vec<Value> =
        serde_json::from_str(&std::fs::read_to_string(root.join("manifest.json")).unwrap())
            .unwrap();
    assert_eq!(manifest.len(), support_one_time::Mode::ALL.len());
    let mut directory = None;
    let mut failures = Vec::new();
    for case in manifest {
        let name = case["case"].as_str().unwrap();
        let mode: support_one_time::Mode = serde_json::from_value(case["case"].clone()).unwrap();
        let (expected, code) = mode.expected();
        assert_eq!(case["status"], serde_json::to_value(expected).unwrap());
        assert_eq!(case["required_code"], code);
        let path = root.join(case["suite"].as_str().unwrap());
        let bytes = std::fs::read_to_string(&path).unwrap();
        let admitted = AdmittedSuite::from_json(&bytes).unwrap();
        assert_eq!(
            serde_json::to_value(admitted.suite()).unwrap(),
            serde_json::to_value(support_one_time::admitted(mode).suite()).unwrap()
        );
        if let Err(error) = ess_conformance::go::emit(admitted.suite()) {
            failures.push(format!(
                "{name}: emitter refusal before target callbacks: {error}"
            ));
            continue;
        }
        let directory = directory.get_or_insert_with(|| {
            support_go::package(
                "one-time-live",
                admitted.suite(),
                &[("parity_test.go", &one_time_driver())],
            )
        });
        let host = support_go_one_time::Host::start(mode);
        let result = support_go::go_test(
            directory,
            "TestParity",
            &[
                ("PARITY_ADDRESS", &host.address),
                ("PARITY_DOCUMENT", path.to_str().unwrap()),
            ],
        );
        let snapshot = host.stop();
        if let Err(error) =
            check_one_time_execution(directory, &case, &admitted, &result, &snapshot)
        {
            failures.push(format!("{name}: {error}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn malformed_one_time_documents_refuse_before_any_target_callback() {
    let admitted = support_one_time::admitted(support_one_time::Mode::Healthy);
    let directory = support_go::package(
        "one-time-full-admission",
        admitted.suite(),
        &[("parity_test.go", &one_time_driver())],
    );
    for fixture_dir in ["one-time-response", "one-time-response-identifiers"] {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(fixture_dir);
        let admission = support_go::go_test(
            &directory,
            "TestOneTimeFullAdmission",
            &[("PARITY_ADMISSION_VECTORS", root.to_str().unwrap())],
        );
        assert!(admission.success, "{}", admission.log);
        for entry in std::fs::read_dir(root).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if entry.file_name().to_string_lossy().starts_with("valid-") {
                continue;
            }
            assert!(AdmittedSuite::from_json(&std::fs::read_to_string(&path).unwrap()).is_err());
            let host = support_go_one_time::Host::start(support_one_time::Mode::Healthy);
            let result = support_go::go_test(
                &directory,
                "TestParity",
                &[
                    ("PARITY_ADDRESS", &host.address),
                    ("PARITY_DOCUMENT", path.to_str().unwrap()),
                ],
            );
            let snapshot = host.stop();
            assert!(
                !result.success,
                "invalid document executed: {}",
                path.display()
            );
            assert!(result.outcomes.is_empty());
            assert!(
                snapshot.trace.is_empty(),
                "invalid document reached callbacks: {:?}",
                snapshot.trace
            );
        }
    }
}

#[test]
fn go_one_time_observation_bounds_match_the_shared_native_resources() {
    use support_go_one_time::resources::ResourceMode;
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let manifest: Vec<Value> = serde_json::from_str(
        &std::fs::read_to_string(root.join("one-time-resources.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(manifest.len(), ResourceMode::ALL.len());
    let path = root.join("one-time-execution/view.json");
    let admitted = AdmittedSuite::from_json(&std::fs::read_to_string(&path).unwrap()).unwrap();
    let directory = support_go::package(
        "one-time-resources",
        admitted.suite(),
        &[("parity_test.go", &one_time_driver())],
    );
    for (mode, mut expected) in ResourceMode::ALL.into_iter().zip(manifest) {
        assert_eq!(serde_json::to_value(mode).unwrap(), expected["case"]);
        assert_eq!(
            serde_json::to_value(mode.expected()).unwrap(),
            expected["status"]
        );
        let payload = serde_json::to_string(&mode.rows()).unwrap();
        assert_eq!(json!(payload.len()), expected["canonical_payload_bytes"]);
        expected["counts"].as_object_mut().unwrap().remove("total");
        expected["required_code"] = json!(if mode.expected()
            == ess_conformance::report::Status::Passed
        {
            "ESS-CF-DISCLOSURE"
        } else {
            "ESS-CF-TARGET"
        });
        let host = support_go_one_time::Host::resource(mode);
        let result = support_go::go_test(
            &directory,
            "TestParity",
            &[
                ("PARITY_ADDRESS", &host.address),
                ("PARITY_DOCUMENT", path.to_str().unwrap()),
            ],
        );
        let snapshot = host.stop();
        check_one_time_execution(&directory, &expected, &admitted, &result, &snapshot).unwrap();
    }
}

#[test]
fn go_keeps_each_marked_field_exemption_at_its_exact_position() {
    use support_go_one_time::field_controls::{self, FieldMode};
    let root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/one-time-fields");
    let manifest: Vec<Value> =
        serde_json::from_str(&std::fs::read_to_string(root.join("manifest.json")).unwrap())
            .unwrap();
    let path = root.join("suite.json");
    let admitted = AdmittedSuite::from_json(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(admitted.suite(), field_controls::admitted().suite());
    assert_eq!(manifest.len(), FieldMode::ALL.len());
    let directory = support_go::package(
        "one-time-fields",
        admitted.suite(),
        &[("parity_test.go", &one_time_driver())],
    );
    for (mode, mut expected) in FieldMode::ALL.into_iter().zip(manifest) {
        assert_eq!(serde_json::to_value(mode).unwrap(), expected["case"]);
        assert_eq!(
            serde_json::to_value(mode.expected()).unwrap(),
            expected["status"]
        );
        expected["counts"].as_object_mut().unwrap().remove("total");
        let host = support_go_one_time::Host::fields(mode);
        let result = support_go::go_test(
            &directory,
            "TestParity",
            &[
                ("PARITY_ADDRESS", &host.address),
                ("PARITY_DOCUMENT", path.to_str().unwrap()),
            ],
        );
        let snapshot = host.stop();
        check_one_time_execution(&directory, &expected, &admitted, &result, &snapshot).unwrap();
    }
}

#[test]
fn go_executes_the_original_suite35_coverage_input_and_parent_inventory() {
    let root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/one-time-coverage");
    let manifest: Vec<Value> =
        serde_json::from_str(&std::fs::read_to_string(root.join("manifest.json")).unwrap())
            .unwrap();
    let path = root.join("input.json");
    let input = AdmittedInput::from_json(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(
        input
            .selected()
            .suite()
            .provenance
            .suite_version
            .to_string(),
        "ess-conformance/35"
    );
    let directory = support_go::package_input(
        "one-time-coverage",
        &input,
        &[("parity_test.go", &one_time_driver())],
    );
    for mut expected in manifest {
        let mode: support_one_time::Mode =
            serde_json::from_value(expected["case"].clone()).unwrap();
        expected["counts"].as_object_mut().unwrap().remove("total");
        let host = support_go_one_time::Host::start(mode);
        let result = support_go::go_test(
            &directory,
            "TestParity",
            &[
                ("PARITY_ADDRESS", &host.address),
                ("PARITY_DOCUMENT", path.to_str().unwrap()),
            ],
        );
        let snapshot = host.stop();
        check_one_time_execution(&directory, &expected, input.selected(), &result, &snapshot)
            .unwrap();
    }
}

#[test]
fn go_repeated_event_windows_keep_one_operation_start_and_final_scans() {
    let root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/one-time-windows");
    let manifest: Vec<Value> =
        serde_json::from_str(&std::fs::read_to_string(root.join("manifest.json")).unwrap())
            .unwrap();
    assert_eq!(manifest.len(), 2);
    let path = root.join("suite.json");
    let admitted = AdmittedSuite::from_json(&std::fs::read_to_string(&path).unwrap()).unwrap();
    let directory = support_go::package(
        "one-time-windows",
        admitted.suite(),
        &[("parity_test.go", &one_time_driver())],
    );
    for mut expected in manifest {
        let mode: support_one_time::Mode =
            serde_json::from_value(expected["case"].clone()).unwrap();
        expected["counts"].as_object_mut().unwrap().remove("total");
        let host = support_go_one_time::Host::start(mode);
        let result = support_go::go_test(
            &directory,
            "TestParity",
            &[
                ("PARITY_ADDRESS", &host.address),
                ("PARITY_DOCUMENT", path.to_str().unwrap()),
            ],
        );
        let snapshot = host.stop();
        check_one_time_execution(&directory, &expected, &admitted, &result, &snapshot).unwrap();
    }
}

fn check_one_time_execution(
    directory: &std::path::Path,
    case: &Value,
    admitted: &AdmittedSuite,
    result: &support_go::GoRun,
    snapshot: &support_go_one_time::Snapshot,
) -> Result<(), String> {
    let name = case["case"].as_str().unwrap();
    let expected = case["status"].as_str().unwrap();
    let raw = std::fs::read_to_string(directory.join("report.json"))
        .map_err(|error| format!("no report: {error}; {}", result.log))?;
    for value in snapshot
        .plaintexts
        .iter()
        .map(String::as_str)
        .chain([support_one_time::FIRST])
    {
        if !value.is_empty() && (raw.contains(value) || result.log.contains(value)) {
            return Err("observed plaintext escaped in report or diagnostic".into());
        }
    }
    let report: Value = serde_json::from_str(&raw).map_err(|error| error.to_string())?;
    ess_conformance::counts::CountReport::from_json(&raw, admitted)
        .map_err(|error| format!("invalid execution binding: {error}"))?;
    if report["producer_profile"] != "go-scenario-status/2" {
        return Err("one-time execution used a legacy status profile".into());
    }
    let mut counts = report["counts"].clone();
    if counts.as_object_mut().unwrap().remove("total") != Some(json!(1)) || counts != case["counts"]
    {
        return Err(format!(
            "wrong counts: {counts}, expected {}",
            case["counts"]
        ));
    }
    let id = admitted
        .suite()
        .scenarios
        .keys()
        .next()
        .unwrap()
        .to_string();
    if result.outcomes.len() != 1 || result.outcomes.get(&id).map(String::as_str) != Some(expected)
    {
        return Err(format!(
            "wrong exact outcomes: {:?}; {}",
            result.outcomes, result.log
        ));
    }
    if serde_json::to_value(&snapshot.trace).unwrap() != case["callback_trace"] {
        return Err(format!(
            "wrong callback trace: {:?}, expected {}",
            snapshot.trace, case["callback_trace"]
        ));
    }
    if !result.log.contains(case["required_code"].as_str().unwrap())
        || result.success != (expected == "passed")
    {
        return Err(format!("wrong diagnostic code or exit: {}", result.log));
    }
    eprintln!(
        "one-time case={name} suite={} id={id} counts={counts} trace={:?} calls={} exit_success={}",
        admitted.suite().provenance.suite_version,
        snapshot.trace,
        snapshot.calls,
        result.success
    );
    Ok(())
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

#[test]
fn go_prepares_one_time_policy_admission_from_the_25_immutable_contract_vectors() {
    let root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/one-time-response");
    let entries = std::fs::read_dir(&root)
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(entries.len(), 25);
    for entry in entries {
        let name = entry.file_name().to_string_lossy().into_owned();
        let raw = std::fs::read_to_string(entry.path()).unwrap();
        assert_eq!(
            AdmittedSuite::from_json(&raw).is_ok(),
            name.starts_with("valid-"),
            "{name}"
        );
    }
    let directory = direct(false).package("one-time-admission", &[("parity_test.go", &driver())]);
    let result = support_go::go_test(
        &directory,
        "TestOneTimeAdmission",
        &[("PARITY_ADMISSION_VECTORS", root.to_str().unwrap())],
    );
    eprintln!("{}", result.log);
    assert!(result.success, "{}", result.log);
    assert!(
        result.outcomes.is_empty(),
        "policy preparation must not publish an execution report"
    );
}

fn one_time_constraint_cases() -> [(bool, Value); 47] {
    [
        (true, json!(true)),
        (true, json!(false)),
        (true, json!("value")),
        (true, json!("defined(value)")),
        (true, json!("value.count >= 4")),
        (true, json!("value.count <= value.count")),
        (true, json!("value == 'abc'")),
        (true, json!("value != 'abc'")),
        (true, json!("value >= 'abc'")),
        (true, json!("not value.count < 4")),
        (true, json!({"all":[]})),
        (true, json!({"none":[]})),
        (true, json!({"value":{"starts_with":"ab"}})),
        (true, json!({"value":{"ends_with":"ab"}})),
        (true, json!({"value":{"contains":"ab"}})),
        (true, json!({"value":{"equals_ignore_case":""}})),
        (true, json!({"value":{"in_ignore_case":["ab"]}})),
        (true, json!({"value":{"in":["ab","ac"]}})),
        (true, json!({"none":[false],"not":false})),
        (true, json!({"all_of":null})),
        (true, json!({"not":{"not":"value.count >= 4"}})),
        (true, json!({"value":{"in":null}})),
        (false, json!({"value":{"equals":"true"}})),
        (true, json!({"value":{"equals":"\"true\""}})),
        (true, json!({"value":{"eq":"\"a.b\""}})),
        (true, json!({"value":{"eq":"\"a and b\""}})),
        (true, json!("true")),
        (true, json!("false")),
        (true, json!("exists(value)")),
        (true, json!("missing(value)")),
        (true, json!("defined ( value )")),
        (true, json!("value.count >= 9007199254740993")),
        (
            true,
            json!({"value.count":{"gte":9_007_199_254_740_993_u64}}),
        ),
        (true, json!({"value.count":{"gte":1e2}})),
        (true, json!({"value.count":{"in":[1e2, 1e-7]}})),
        (true, json!({"value.count":{"in":[-0.0]}})),
        (true, json!(format!("value.count >= {}1", "0".repeat(600)))),
        (true, json!("value.count >= 1e-2147483648")),
        (
            true,
            json!({"value.count":{"in":[1, 2, 9_007_199_254_740_993_u64]}}),
        ),
        (
            true,
            json!({"value.count":{"in":[1e-6, 1e-5, 1e15, 1e16, 1e20, -0.0]}}),
        ),
        (false, json!("value == null")),
        (false, json!("value == a && b")),
        (false, json!("value.bad == 1")),
        (false, json!("missing.count >= 4")),
        (false, json!("value.count == 'abc'")),
        (false, json!({"value":{"starts_with":""}})),
        (
            false,
            json!({"forall":{"in":"value","as":"item","that":true}}),
        ),
    ]
}

#[test]
fn go_one_time_string_constraint_grammar_matches_native_admission() {
    let original: Value = serde_json::from_str(include_str!(
        "fixtures/one-time-response/valid-constrained-string.json"
    ))
    .unwrap();
    let cases = one_time_constraint_cases();
    assert_eq!(cases.len(), 47);
    let directory =
        direct(false).package("one-time-string-grammar", &[("parity_test.go", &driver())]);
    let vectors = directory.join("admission-vectors");
    std::fs::create_dir(&vectors).unwrap();
    let canonical = directory.join("canonical-vectors");
    std::fs::create_dir(&canonical).unwrap();
    for (index, (expected, predicate)) in cases.into_iter().enumerate() {
        let mut wire = original.clone();
        for scenario in wire["scenarios"].as_object_mut().unwrap().values_mut() {
            scenario["one_time_response"]["origins"][0]["response"]["constraints"]
                ["credentials.api.Secret"]["invariants"] = json!([predicate]);
        }
        // Preserve the JSON token -0 as well as -0.0; Value serialization normalizes it.
        let raw = wire.to_string().replace("[-0.0]", "[-0]");
        assert_eq!(
            AdmittedSuite::from_json(&raw).is_ok(),
            expected,
            "constraint {index}: {raw}"
        );
        let prefix = if expected { "valid" } else { "invalid" };
        let name = format!("{prefix}-{index:02}.json");
        if expected {
            let admitted = AdmittedSuite::from_json(&raw).unwrap();
            std::fs::write(
                canonical.join(&name),
                serde_json::to_vec(admitted.suite()).unwrap(),
            )
            .unwrap();
        }
        std::fs::write(vectors.join(name), raw).unwrap();
    }
    let result = support_go::go_test(
        &directory,
        "TestOneTimeAdmission",
        &[
            ("PARITY_ADMISSION_VECTORS", vectors.to_str().unwrap()),
            ("PARITY_CANONICAL_VECTORS", canonical.to_str().unwrap()),
        ],
    );
    eprintln!("{}", result.log);
    assert!(result.success, "{}", result.log);
    assert!(result.outcomes.is_empty());
}

#[test]
fn go_prepares_the_frozen_disclosure_identity_grammar_and_bindings() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let grammar = root.join("one-time-response-identifier-grammar.json");
    let vectors: Vec<(bool, String)> =
        serde_json::from_str(&std::fs::read_to_string(&grammar).unwrap()).unwrap();
    assert_eq!(vectors.len(), 13);
    for (expected, written) in vectors {
        assert_eq!(
            ess_conformance::ScenarioId::parse(&written).is_ok(),
            expected,
            "{written}"
        );
    }
    let suites = root.join("one-time-response-identifiers");
    let entries = std::fs::read_dir(&suites)
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(entries.len(), 11);
    for entry in entries {
        let name = entry.file_name().to_string_lossy().into_owned();
        let raw = std::fs::read_to_string(entry.path()).unwrap();
        assert_eq!(
            AdmittedSuite::from_json(&raw).is_ok(),
            name.starts_with("valid-"),
            "{name}"
        );
    }
    let directory = direct(false).package("one-time-identifiers", &[("parity_test.go", &driver())]);
    for name in ["TestOneTimeIdentifierGrammar", "TestOneTimeAdmission"] {
        let result = support_go::go_test(
            &directory,
            name,
            &[
                ("PARITY_IDENTIFIER_GRAMMAR", grammar.to_str().unwrap()),
                ("PARITY_ADMISSION_VECTORS", suites.to_str().unwrap()),
                ("PARITY_IDENTIFIERS", "1"),
            ],
        );
        eprintln!("{}", result.log);
        assert!(result.success, "{}", result.log);
    }
}

#[test]
fn old_coverage_refusals_keep_native_disclosure_identity_grammar() {
    let input = direct(true);
    let mut document: Value = serde_json::from_str(input.selected().original_json()).unwrap();
    document["coverage"]["refused"] = json!([{
        "origin":"generated", "scenario":"library.api.Read/disclosure/returned/value/origin/as/anonymous",
        "subject":{"kind":"command","name":"library.api.Read"}, "source":null,
        "code":"ESS-SYNTH-001", "message":"candidate could not be arranged",
        "effect":"candidate_not_emitted", "retained":null,"scope":"in_scope","needs":[]
    }]);
    document["coverage"]["counts"]["refused"] = json!(1);
    let raw = document.to_string();
    AdmittedSuite::from_json(&raw).expect("refused IDs carry grammar, not executable vocabulary");
    let directory = input.package("old-refusal-identity", &[("parity_test.go", &driver())]);
    let vectors = directory.join("vectors");
    std::fs::create_dir_all(&vectors).unwrap();
    std::fs::write(vectors.join("valid-old-refused-disclosure-id.json"), raw).unwrap();
    let result = support_go::go_test(
        &directory,
        "TestOneTimeFullAdmission",
        &[("PARITY_ADMISSION_VECTORS", vectors.to_str().unwrap())],
    );
    assert!(result.success, "{}", result.log);
}

#[test]
fn go_one_time_authority_counts_canonical_metadata_bytes() {
    let original: Value = serde_json::from_str(include_str!(
        "fixtures/one-time-response/valid-constrained-string.json"
    ))
    .unwrap();
    let directory = direct(false).package(
        "one-time-canonical-authority",
        &[("parity_test.go", &driver())],
    );
    let vectors = directory.join("admission-vectors");
    let canonical = directory.join("canonical-vectors");
    std::fs::create_dir(&vectors).unwrap();
    std::fs::create_dir(&canonical).unwrap();
    for (index, size) in [1024_usize, 1_048_576, 1_048_577].into_iter().enumerate() {
        let mut wire = original.clone();
        let scenario = wire["scenarios"]
            .as_object_mut()
            .unwrap()
            .values_mut()
            .next()
            .unwrap();
        let policy = &mut scenario["one_time_response"];
        let response = &mut policy["origins"][0]["response"];
        let summary_prefix = "é\u{2028}\u{2029}\\u2028";
        response["fields"][0]["naming"] = json!({"summary":summary_prefix, "code":"Secret"});
        response["fields"][0]["wire"] = Value::Null;
        response["fields"].as_array_mut().unwrap().push(json!({
            "name":"extra", "type":"Optional<Map<String, Json>>",
            "naming":{"display":"Other"}, "presence":"omitted_when_absent"
        }));
        response["declarations"]["credentials.api.Secret"]["of"] = json!(" String ");
        let constraints = response["constraints"]["credentials.api.Secret"]
            .as_object_mut()
            .unwrap();
        constraints.remove("alphabet");
        constraints.remove("prefix");
        constraints.insert("invariants".into(), json!([{"all_of":[true]}]));
        let typed: ess_conformance::one_time_response::Trace =
            serde_json::from_value(policy.clone()).unwrap();
        let base_size = serde_json::to_vec(&typed).unwrap().len();
        assert!(base_size <= size);
        policy["origins"][0]["response"]["fields"][0]["naming"]["summary"] =
            json!(format!("{summary_prefix}{}", "x".repeat(size - base_size)));
        let typed: ess_conformance::one_time_response::Trace =
            serde_json::from_value(policy.clone()).unwrap();
        assert_eq!(serde_json::to_vec(&typed).unwrap().len(), size);
        let raw = wire.to_string();
        let admitted = AdmittedSuite::from_json(&raw);
        let expected = size <= 1_048_576;
        assert_eq!(
            admitted.is_ok(),
            expected,
            "canonical size {size}: {admitted:?}"
        );
        let prefix = if expected { "valid" } else { "invalid" };
        let name = format!("{prefix}-{index}.json");
        if let Ok(admitted) = admitted {
            std::fs::write(
                canonical.join(&name),
                serde_json::to_vec(admitted.suite()).unwrap(),
            )
            .unwrap();
        }
        std::fs::write(vectors.join(name), raw).unwrap();
    }
    let result = support_go::go_test(
        &directory,
        "TestOneTimeAdmission",
        &[
            ("PARITY_ADMISSION_VECTORS", vectors.to_str().unwrap()),
            ("PARITY_CANONICAL_VECTORS", canonical.to_str().unwrap()),
        ],
    );
    eprintln!("{}", result.log);
    assert!(result.success, "{}", result.log);
}

#[test]
fn go_exploration_refuses_marked_source_before_callbacks_or_recording() {
    let source="format: ess/21\nsystem: credentials\nversion: v1\ndomain: credentials.api\ncommands:\n  - name: credentials.api.Issue\n    response: [{name: secret, type: String}]\n    outcomes: [{name: issued, returns: true, one_time_response: [secret]}]\n";
    for marked in [true, false] {
        let source = if marked {
            source.to_owned()
        } else {
            source.replace(", one_time_response: [secret]", "")
        };
        let ir = model(&source);
        // Explorer generation accepts a selected empty suite, so the authoritative source IR
        // must govern this preflight independently of scenario selection.
        let suite = ConformanceSuite::new(ess_conformance::SuiteProvenance::of(&ir));
        let directory = support_go::directory("one-time-explorer-preflight");
        for artifact in ess_conformance::go::emit_with_model(&suite, &ir).unwrap() {
            std::fs::write(directory.join(artifact.path), artifact.contents).unwrap();
        }
        std::fs::write(
            directory.join("go.mod"),
            "module example.invalid/goparity\n\ngo 1.21\n",
        )
        .unwrap();
        std::fs::write(directory.join("essconform/parity_test.go"), driver()).unwrap();
        std::fs::write(directory.join("essconform/preflight_test.go"),r#"package essconform
import("testing";"os";"strings")
func TestMarkedExplorer(t *testing.T){
 marked:=os.Getenv("PARITY_MARKED")=="yes"
 _,err:=exploreLoad(exploreIR,exploreSuite)
 if !marked{if err!=nil{t.Fatal(err)};return}
 if err==nil||!strings.Contains(err.Error(),"UnsupportedOneTimeDisclosure"){t.Errorf("marked source was not refused by model loading: %v",err)}
 calls:=0;factory:=func()Target{calls++;return parityTarget{}}
 _,err=Explore(factory,ExploreOptions{Seeds:1,Steps:1})
 if err==nil||!strings.Contains(err.Error(),"UnsupportedOneTimeDisclosure"){t.Errorf("serial explorer did not refuse marked source: %v",err)}
 out:=os.Getenv("PARITY_HISTORY_OUT")
 _,err=ExploreConcurrent(factory,ConcurrentOptions{Path:"source.yaml",Out:out,Seeds:1,Clients:2,Calls:1})
 if err==nil||!strings.Contains(err.Error(),"UnsupportedOneTimeDisclosure"){t.Errorf("concurrent explorer did not refuse marked source: %v",err)}
 if calls!=0{t.Errorf("marked source constructed %d targets",calls)}
 if _,err:=os.Stat(out);!os.IsNotExist(err){t.Errorf("marked source created a history directory")}
}
"#).unwrap();
        std::fs::write(directory.join("source.yaml"), source).unwrap();
        let history = directory.join("history-must-not-exist");
        let result = support_go::go_test(
            &directory,
            "TestMarkedExplorer",
            &[
                ("PARITY_MARKED", if marked { "yes" } else { "no" }),
                ("PARITY_HISTORY_OUT", history.to_str().unwrap()),
            ],
        );
        eprintln!("{}", result.log);
        assert!(result.success, "{}", result.log);
        assert!(result.outcomes.is_empty());
    }
}

/// The `ess-conformance-execution/1` context a protected one-time Go run writes carries the fixed
/// redacted label, the host's build and two digests: no target-returned identity, no captured
/// value and no hash of either (beyond10x/ess#296).
#[test]
fn go_execution_context_of_a_protected_run_carries_only_the_redacted_label_and_digests() {
    use ess_conformance::known_failures::{sha256, ExecutionContext};
    let root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/one-time-execution");
    let path = root.join("healthy.json");
    let admitted = AdmittedSuite::from_json(&std::fs::read_to_string(&path).unwrap()).unwrap();
    let directory = support_go::package(
        "one-time-execution-context",
        admitted.suite(),
        &[("parity_test.go", &one_time_driver())],
    );
    let build = format!("sha256:{}", "c".repeat(64));
    let context = directory.join("execution.json");
    let _ = std::fs::remove_file(&context);
    let host = support_go_one_time::Host::start(support_one_time::Mode::IdentitySuccess);
    let result = support_go::go_test(
        &directory,
        "TestParity",
        &[
            ("PARITY_ADDRESS", &host.address),
            ("PARITY_DOCUMENT", path.to_str().unwrap()),
            ("ESS_IMPLEMENTATION_BUILD", &build),
            ("ESS_EXECUTION_CONTEXT_OUT", context.to_str().unwrap()),
        ],
    );
    let snapshot = host.stop();
    let report = std::fs::read_to_string(directory.join("report.json"))
        .unwrap_or_else(|error| panic!("no report: {error}\n{}", result.log));
    let text = std::fs::read_to_string(&context)
        .unwrap_or_else(|error| panic!("no execution context: {error}\n{}", result.log));
    for secret in snapshot
        .plaintexts
        .iter()
        .map(String::as_str)
        .chain([support_one_time::FIRST])
        .filter(|secret| !secret.is_empty())
    {
        assert!(
            !text.contains(secret),
            "a protected value entered the context"
        );
        let hashed = sha256(secret.as_bytes());
        assert!(
            !text.contains(&hashed["sha256:".len()..]),
            "a protected value's hash entered the context"
        );
    }
    let read = ExecutionContext::from_json(&text).unwrap_or_else(|refusal| panic!("{refusal}"));
    read.admit(&report, &admitted)
        .unwrap_or_else(|refusal| panic!("{refusal}"));
    assert_eq!(read.implementation(), "one-time-protected-target ");
    assert_eq!(read.implementation_build(), build);
    assert_eq!(
        read.to_canonical_json(),
        text,
        "Go writes the canonical bytes"
    );
}
