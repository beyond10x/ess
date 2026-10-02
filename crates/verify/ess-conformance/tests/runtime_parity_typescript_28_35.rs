//! TypeScript execution parity against the frozen native suite contracts.

use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::{
    authored,
    coverage::{AdmittedInput, Origins, Scope},
    coverage_build::{build, CoverageSource},
    AdmittedSuite, ConformanceSuite,
};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use serde_json::{json, Value};

enum FixtureSuite {
    Ordinary(AdmittedSuite),
    Covered(AdmittedInput),
}
impl FixtureSuite {
    fn serialized(&self) -> String {
        match self {
            Self::Ordinary(s) => s.original_json().to_owned(),
            Self::Covered(s) => s.document().to_canonical_json().unwrap(),
        }
    }
    fn selected(&self) -> &AdmittedSuite {
        match self {
            Self::Ordinary(s) => s,
            Self::Covered(s) => s.selected(),
        }
    }
    fn emit(
        &self,
    ) -> Result<Vec<ess_conformance::ts::TsArtifact>, ess_conformance::AdmissionError> {
        match self {
            Self::Ordinary(s) => ess_conformance::ts::emit(s.suite()),
            Self::Covered(s) => ess_conformance::ts::emit_input(s),
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
fn direct(covered: bool) -> FixtureSuite {
    let source = include_str!("direct_returns.rs");
    let ir = model(&constant(source, "MODEL"));
    let timeline = constant(source, "SCENARIO");
    if covered {
        return FixtureSuite::Covered(
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
    FixtureSuite::Ordinary(AdmittedSuite::from_suite(&suite).unwrap())
}
fn delivery(covered: bool) -> FixtureSuite {
    let ir = model(include_str!("fixtures/delivery-context.yaml"));
    let ids: Vec<_> = ["delivery", "flow", "mapping", "on-failure"]
        .map(|name| format!("received/binding/{name}").parse().unwrap())
        .into();
    if covered {
        return FixtureSuite::Covered(
            build(&ir, &[], Scope::System, Origins::Generated)
                .unwrap()
                .select(&ids)
                .unwrap(),
        );
    }
    let mut suite = ess_conformance::synthesize(&ir).suite;
    suite.scenarios.retain(|id, _| ids.contains(id));
    FixtureSuite::Ordinary(AdmittedSuite::from_suite(&suite).unwrap())
}
fn structured(covered: bool) -> FixtureSuite {
    let source = include_str!("authored_structured_instances.rs");
    let ir = model(include_str!("fixtures/structured-instances.yaml"));
    let timeline = format!(
        "{}{}",
        constant(source, "ARRANGED"),
        constant(source, "PLANNED")
    );
    if covered {
        return FixtureSuite::Covered(
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
    FixtureSuite::Ordinary(AdmittedSuite::from_suite(&suite).unwrap())
}

#[test]
fn typescript_emits_complete_direct_response_suites_28_and_29() {
    for covered in [false, true] {
        let input = direct(covered);
        assert_eq!(
            input.selected().suite().provenance.suite_version.major(),
            if covered { 29 } else { 28 }
        );
        input
            .emit()
            .expect("the TypeScript runner executes direct responses");
    }
}

#[test]
fn typescript_emits_complete_delivery_context_suites_30_and_31() {
    for covered in [false, true] {
        let input = delivery(covered);
        assert_eq!(input.selected().suite().scenarios.len(), 4);
        input
            .emit()
            .expect("the TypeScript runner executes delivery context");
    }
}

#[test]
fn typescript_emits_complete_structured_value_suites_32_and_33() {
    for covered in [false, true] {
        let input = structured(covered);
        assert_eq!(input.selected().suite().scenarios.len(), 1);
        input
            .emit()
            .expect("the TypeScript runner resolves structured values");
    }
}

#[test]
fn malformed_direct_authority_is_refused_by_native_before_execution() {
    let input = direct(false);
    let mut value: Value = serde_json::from_str(input.selected().original_json()).unwrap();
    value["provenance"]["suite_version"] = json!("ess-conformance/26");
    assert!(AdmittedSuite::from_json(&value.to_string()).is_err());
}

use ess_conformance::{counts::CountReport, target::*, Runner};
use std::{
    cell::Cell,
    path::{Path, PathBuf},
    process::Command,
    sync::OnceLock,
};

fn ordinary_status_suite() -> AdmittedSuite {
    let original = direct(false);
    let mut value: Value = serde_json::from_str(original.selected().original_json()).unwrap();
    value["provenance"]["suite_version"] = json!("ess-conformance/26");
    for scenario in value["scenarios"].as_object_mut().unwrap().values_mut() {
        scenario["steps"]
            .as_array_mut()
            .unwrap()
            .retain(|step| step["step"] != "expect_direct_response");
    }
    AdmittedSuite::from_json(&value.to_string()).unwrap()
}

fn runtime_package() -> &'static PathBuf {
    static PACKAGE: OnceLock<PathBuf> = OnceLock::new();
    PACKAGE.get_or_init(|| {
        let directory = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../target/backlog-input/ts-prerequisite-runtime");
        std::fs::create_dir_all(&directory).unwrap();
        for artifact in ess_conformance::ts::emit(ordinary_status_suite().suite()).unwrap() {
            let path = directory.join(artifact.path);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, artifact.contents).unwrap();
        }
        let package = directory.join("essconform");
        let mut compile = Command::new("tsc");
        if let Some(modules) = std::env::var_os("ESS_TYPES_NODE") {
            compile
                .arg("--typeRoots")
                .arg(Path::new(&modules).join("@types"));
        }
        let output = compile
            .args(["--project", "tsconfig.json", "--noCheck"])
            .current_dir(&package)
            .output()
            .expect("TypeScript compiler is required for parity");
        assert!(
            output.status.success(),
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        std::fs::write(package.join("parity.mjs"), DRIVER).unwrap();
        std::fs::write(package.join("live.mjs"), LIVE_DRIVER).unwrap();
        package
    })
}

const DRIVER: &str = r#"
import {readFileSync} from 'node:fs';
import {runWith, unsupported, strictJSON} from './dist/runtime.js';
const [file, mode] = process.argv.slice(2);
const calls = [];
const target = () => ({
 identity: async () => ({name:'parity',version:'1'}),
 beginScenario: async () => {calls.push('begin');},
 endScenario: async () => {calls.push('end');},
 executeCommand: async () => {
   calls.push('execute');
   if (mode === 'unsupported') throw unsupported('capability absent');
   if (mode === 'error') throw new Error('target unavailable');
   return {outcome:'returned',response:strictJSON('{"value":"actual","sequence":[1,2,2],"item":{"label":"nested","ordinal":9007199254740993}}')};
 },
 queryView: async () => {throw new Error('unexpected query');},
 observeEvents: async () => [], configureExternalOutcome: async () => {}, redeliverEvent: async () => {}, observeInvocations: async () => []
});
const scope = {diagnostic(){}, skip(){}, async test(_name, body){try {await body(this);} catch {}}};
try {await runWith(scope,target,readFileSync(file,'utf8')); console.log(JSON.stringify({calls}));}
catch (error) { console.log(JSON.stringify({calls,error:String(error)})); process.exitCode=2; }
"#;

struct StatusTarget<'a> {
    mode: &'a str,
    calls: Cell<usize>,
}
impl ConformanceTarget for StatusTarget<'_> {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("parity", "1"))
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
        match self.mode {
            "unsupported" => Err(TargetError::unsupported("execute", "capability absent")),
            "error" => Err(TargetError::unavailable("execute", "target unavailable")),
            _ => Ok(SemanticCommandResult::took(
                ess_conformance::scenario::OutcomeRef::new(
                    request.command,
                    "returned".parse().unwrap(),
                ),
            )),
        }
    }
    fn query_view(&self, _: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        panic!("unexpected query")
    }
    fn observe_events(
        &self,
        _: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Ok(vec![])
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        Ok(())
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        Ok(())
    }
}

#[test]
fn typescript_reports_native_error_and_unsupported_counts_without_normalization() {
    let suite = ordinary_status_suite();
    let directory = runtime_package();
    let input = directory.join("statuses.json");
    std::fs::write(&input, suite.original_json()).unwrap();
    for mode in ["correct", "error", "unsupported"] {
        let target = StatusTarget {
            mode,
            calls: Cell::new(0),
        };
        let native = Runner::for_suite(suite.suite()).run_admitted(&suite, &target);
        assert_eq!(target.calls.get(), 1);
        let expected: Value = serde_json::from_str(
            &CountReport::from_run(&native, &suite)
                .unwrap()
                .to_canonical_json()
                .unwrap(),
        )
        .unwrap();
        let report = directory.join(format!("{mode}-report.json"));
        let output = Command::new("node")
            .arg(directory.join("parity.mjs"))
            .arg(&input)
            .arg(mode)
            .env("ESS_REPORT_FORMAT", "2")
            .env("ESS_REPORT_OUT", &report)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let observed: Value = serde_json::from_slice(&std::fs::read(report).unwrap()).unwrap();
        assert_eq!(
            observed["counts"], expected["counts"],
            "{mode}: raw terminal counts"
        );
        assert_eq!(
            observed["outcomes"], expected["outcomes"],
            "{mode}: exact scenario verdicts"
        );
        let callbacks: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(callbacks["calls"], json!(["begin", "execute", "end"]));
    }
}

mod support_typescript_prerequisite;

const LIVE_DRIVER: &str = r"
import {readFileSync} from 'node:fs';
import {connect} from 'node:net';
import {runWith, unsupported, strictJSON, goMarshal, JsonNumber} from './dist/runtime.js';
const [file,address] = process.argv.slice(2);
async function call(method,args=null) {
 const [host,port]=address.split(':');
 return await new Promise((resolve,reject)=>{
  const socket=connect({host,port:Number(port)});let bytes='';
  socket.on('error',reject);
  socket.on('connect',()=>socket.write(goMarshal({method,args})+'\n'));
  socket.on('data',chunk=>{bytes+=chunk.toString();if(bytes.includes('\n')) {socket.end();try {
   const reply=JSON.parse(bytes,(_key,value,context)=>typeof value==='number'?new JsonNumber(context.source):value);if(reply.error)reject(reply.unsupported?unsupported(reply.error):new Error(reply.error));else resolve(reply.ok);
  }catch(error){reject(error);}}});
 });
}
const upper = value => Object.fromEntries(Object.entries(value).map(([key,item])=>[key[0].toUpperCase()+key.slice(1),item]));
const events = values => (values??[]).map(value=>({event:value.Event,payload:value.Payload}));
const target=()=>({
 identity:async()=>{const value=await call('identity');return {name:value.Name,version:value.Version};},
 beginScenario:async value=>call('begin',upper(value)), endScenario:async value=>call('end',upper(value)),
 executeCommand:async request=>{const value=await call('execute',upper(request));return {outcome:value.Outcome??'',response:value.Response??undefined,directEvents:events(value.DirectEvents)};},
 queryView:async request=>{const value=await call('query',upper(request));return {rows:value.Rows};},
 observeEvents:async request=>events(await call('events',upper(request))),
 observeInvocations:async request=>(await call('invocations',upper(request))).map(value=>({command:value.Command,input:value.Input})),
 configureExternalOutcome:async request=>call('configure',upper(request)),
 redeliverEvent:async request=>call('redeliver',upper(request)), deliverEvent:async request=>call('deliver',upper(request))
});
let failed=false;
const scope={diagnostic(){},skip(){},async test(name,body){try{await body(this);}catch(error){failed=true;console.error(`${name}: ${String(error)}`);}}};
try {await runWith(scope,target,readFileSync(file,'utf8'));if(failed)process.exitCode=1;}
catch(error){console.error(String(error));process.exitCode=2;}
";

fn live_run(
    input: &AdmittedSuite,
    document: &str,
    kind: &str,
    mode: &str,
    label: &str,
) -> (Value, Vec<Value>) {
    let directory = runtime_package();
    let input_path = directory.join(format!("{label}.json"));
    let report = directory.join(format!("{label}-report.json"));
    std::fs::write(&input_path, document).unwrap();
    let driver = directory.join("live.mjs");
    let host = support_typescript_prerequisite::Host::start(kind, mode);
    let output = Command::new("node")
        .arg(driver)
        .arg(&input_path)
        .arg(&host.address)
        .env("ESS_REPORT_FORMAT", "2")
        .env("ESS_REPORT_OUT", &report)
        .output()
        .unwrap();
    let trace = host.stop();
    assert_ne!(
        output.status.code(),
        Some(2),
        "{label}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let observed: Value =
        serde_json::from_slice(&std::fs::read(&report).unwrap_or_else(|e| {
            panic!("{label}: {e}: {}", String::from_utf8_lossy(&output.stderr))
        }))
        .unwrap();
    CountReport::from_json(&std::fs::read_to_string(&report).unwrap(), input)
        .expect("shared reader admits exact produced /2 report");
    assert_eq!(observed["producer_profile"], "go-scenario-status/2");
    let failed = observed["counts"]["failed"].as_u64().unwrap()
        + observed["counts"]["error"].as_u64().unwrap()
        + observed["counts"]["unsupported"].as_u64().unwrap();
    assert_eq!(
        output.status.success(),
        failed == 0,
        "{label}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    (observed, trace)
}

#[test]
fn typescript_native_live_callbacks_match_for_all_prerequisite_versions() {
    for covered in [false, true] {
        for (kind, input, modes) in [
            (
                "direct",
                direct(covered),
                vec![
                    "correct",
                    "unsupported",
                    "error",
                    "missing",
                    "extra",
                    "rounded",
                    "reorder",
                    "missing-teardown",
                    "unsupported-teardown",
                ],
            ),
            (
                "delivery",
                delivery(covered),
                vec![
                    "correct",
                    "ignore-context",
                    "stale-context",
                    "late-wrong",
                    "unsupported",
                    "error",
                ],
            ),
            ("structured", structured(covered), vec!["correct"]),
        ] {
            for mode in modes {
                let target = support_typescript_prerequisite::Fixture::new(kind, mode);
                let native = Runner::for_suite(input.selected().suite())
                    .run_admitted(input.selected(), &target);
                let expected: Value = serde_json::from_str(
                    &CountReport::from_run(&native, input.selected())
                        .unwrap()
                        .to_canonical_json()
                        .unwrap(),
                )
                .unwrap();
                let label = format!("{kind}-{covered}-{mode}");
                let (actual, trace) =
                    live_run(input.selected(), &input.serialized(), kind, mode, &label);
                assert_eq!(actual["counts"], expected["counts"], "{label}: counts");
                assert_eq!(
                    actual["outcomes"], expected["outcomes"],
                    "{label}: outcomes"
                );
                assert_eq!(
                    actual["execution_status"], expected["execution_status"],
                    "{label}: execution status"
                );
                assert_eq!(
                    actual["conformance_status"], expected["conformance_status"],
                    "{label}: conformance status"
                );
                let native_trace = target.trace.lock().unwrap();
                if kind == "direct" {
                    assert_eq!(trace, *native_trace, "{label}: callback trace");
                } else {
                    let selected = |trace: &[Value]| {
                        trace
                            .iter()
                            .filter(|entry| {
                                entry["method"]
                                    == if kind == "structured" {
                                        "execute"
                                    } else {
                                        "deliver"
                                    }
                            })
                            .cloned()
                            .collect::<Vec<_>>()
                    };
                    assert_eq!(
                        selected(&trace),
                        selected(&native_trace),
                        "{label}: resolved inputs"
                    );
                }
            }
        }
    }
}

#[test]
fn typescript_direct_profile_preserves_presence_and_payload_local_bounds() {
    let original = constant(include_str!("direct_returns.rs"), "MODEL");
    for (index,(edit,mode,wanted)) in [
        (original.clone(),"large",ess_conformance::report::Status::Passed),
        (original.clone(),"byte-edge",ess_conformance::report::Status::Passed),
        (original.clone(),"oversized",ess_conformance::report::Status::Failed),
        (original.replace("name: value, type: String","name: value, type: Json"),"json-128",ess_conformance::report::Status::Passed),
        (original.replace("name: value, type: String","name: value, type: Json"),"json-129",ess_conformance::report::Status::Failed),
        (original.replace("      - {name: ordinal, type: Integer}","      - {name: ordinal, type: Integer}\n      - {name: note, type: 'Optional<String>', presence: null_when_absent}"),"note-null",ess_conformance::report::Status::Passed),
        (original.replace("      - {name: ordinal, type: Integer}","      - {name: ordinal, type: Integer}\n      - {name: note, type: 'Optional<String>', presence: null_when_absent}"),"correct",ess_conformance::report::Status::Failed),
        (original.replace("      - {name: ordinal, type: Integer}","      - {name: ordinal, type: Integer}\n      - {name: note, type: 'Optional<String>', presence: omitted_when_absent}"),"note-null",ess_conformance::report::Status::Failed),
        (original.replace("      - {name: ordinal, type: Integer}","      - {name: ordinal, type: Integer}\n      - {name: note, type: 'Optional<String>', presence: omitted_when_absent}"),"correct",ess_conformance::report::Status::Passed),
    ].into_iter().enumerate() {
        let suite=ess_conformance::synthesize(&model(&edit)).suite;
        let admitted=AdmittedSuite::from_suite(&suite).unwrap();
        let target=support_typescript_prerequisite::Fixture::new("direct",mode);
        let native=Runner::for_suite(&suite).run_admitted(&admitted,&target);
        assert_eq!(native.scenarios[0].status,wanted,"{mode}");
        let (actual,_)=live_run(&admitted,admitted.original_json(),"direct",mode,&format!("bounds-{index}-{mode}"));
        let expected:Value=serde_json::from_str(&CountReport::from_run(&native,&admitted).unwrap().to_canonical_json().unwrap()).unwrap();
        assert_eq!(actual["outcomes"],expected["outcomes"],"bounds {mode}");
    }
}

#[test]
fn malformed_typescript_prerequisite_documents_refuse_before_all_callbacks() {
    for (kind, input, old) in [
        ("direct", direct(false), 26),
        ("delivery", delivery(false), 28),
        ("structured", structured(false), 30),
    ] {
        let original: Value = serde_json::from_str(input.selected().original_json()).unwrap();
        let mut corruptions = Vec::new();
        let mut older = original.clone();
        older["provenance"]["suite_version"] = json!(format!("ess-conformance/{old}"));
        corruptions.push(older);
        let mut unknown = original.clone();
        let steps = scenario_steps(&mut unknown);
        steps[0]["unrecognized"] = json!(true);
        corruptions.push(unknown);
        if kind == "direct" {
            for field in ["command", "declarations", "fields"] {
                let mut invalid = original.clone();
                let steps = scenario_steps(&mut invalid);
                let response = steps
                    .iter_mut()
                    .find(|step| step["step"] == "expect_direct_response")
                    .unwrap();
                response["response"][field] = if field == "command" {
                    json!("library.api.Other")
                } else if field == "fields" {
                    let mut fields = response["response"]["fields"].clone();
                    fields[0]["wire"] = json!("external");
                    fields[0]["naming"] = json!({"display":"External"});
                    fields
                } else {
                    json!({})
                };
                corruptions.push(invalid);
            }
        }
        if kind == "structured" {
            let mut invalid = original.clone();
            let steps = scenario_steps(&mut invalid);
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
            let directory = runtime_package();
            let path = directory.join(format!("invalid-{kind}-{index}.json"));
            std::fs::write(&path, raw).unwrap();
            let driver = directory.join(format!("invalid-{kind}-{index}.mjs"));
            std::fs::write(&driver, LIVE_DRIVER).unwrap();
            let report = directory.join(format!("invalid-{kind}-{index}-report.json"));
            let _ = std::fs::remove_file(&report);
            let host = support_typescript_prerequisite::Host::start(kind, "correct");
            let output = Command::new("node")
                .arg(driver)
                .arg(&path)
                .arg(&host.address)
                .env("ESS_REPORT_FORMAT", "2")
                .env("ESS_REPORT_OUT", &report)
                .output()
                .unwrap();
            let trace = host.stop();
            assert_eq!(
                output.status.code(),
                Some(2),
                "TypeScript admitted {kind}/{index}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert!(!report.exists(), "invalid suite published a report");
            assert!(
                trace.is_empty(),
                "invalid suite reached callbacks: {trace:?}"
            );
        }
    }
}

#[test]
fn typescript_structured_depth_admission_matches_native_envelope_limit() {
    let original = structured(false);
    for depth in [56, 64, 65] {
        let mut document: Value =
            serde_json::from_str(original.selected().original_json()).unwrap();
        let steps = scenario_steps(&mut document);
        let command = steps
            .iter_mut()
            .find(|step| step["command"] == "release.rings.PlanRollout")
            .unwrap();
        command["input"]["ring_sequence"] = (0..depth).fold(
            json!({"kind":"literal","value":"id"}),
            |value, _| json!({"kind":"list","items":[value]}),
        );
        let raw = document.to_string();
        if let Ok(admitted) = AdmittedSuite::from_json(&raw) {
            assert_eq!(depth, 56);
            let target = support_typescript_prerequisite::Fixture::new("structured", "correct");
            let native = Runner::for_suite(admitted.suite()).run_admitted(&admitted, &target);
            let expected: Value = serde_json::from_str(
                &CountReport::from_run(&native, &admitted)
                    .unwrap()
                    .to_canonical_json()
                    .unwrap(),
            )
            .unwrap();
            let (actual, _) = live_run(
                &admitted,
                &raw,
                "structured",
                "correct",
                "structured-depth56",
            );
            assert_eq!(actual["outcomes"], expected["outcomes"]);
        } else {
            assert!(depth >= 64);
            let directory = runtime_package();
            let path = directory.join(format!("depth-{depth}.json"));
            std::fs::write(&path, raw).unwrap();
            let host = support_typescript_prerequisite::Host::start("structured", "correct");
            let output = Command::new("node")
                .arg(directory.join("live.mjs"))
                .arg(path)
                .arg(&host.address)
                .env("ESS_REPORT_FORMAT", "2")
                .output()
                .unwrap();
            assert_eq!(output.status.code(), Some(2));
            assert!(host.stop().is_empty());
        }
    }
}

fn scenario_steps(document: &mut Value) -> &mut Vec<Value> {
    document["scenarios"]
        .as_object_mut()
        .unwrap()
        .values_mut()
        .next()
        .unwrap()["steps"]
        .as_array_mut()
        .unwrap()
}

#[test]
fn typescript_unbound_nested_instance_is_native_error_before_command_callback() {
    let input = structured(false);
    let mut document: Value = serde_json::from_str(input.selected().original_json()).unwrap();
    let command = scenario_steps(&mut document)
        .iter_mut()
        .find(|step| step["command"] == "release.rings.PlanRollout")
        .unwrap();
    command["input"]["ring_sequence"]["items"][0] = json!({"kind":"instance","instance":"unbound"});
    let raw = document.to_string();
    let admitted = AdmittedSuite::from_json(&raw).unwrap();
    let target = support_typescript_prerequisite::Fixture::new("structured", "correct");
    let native = Runner::for_suite(admitted.suite()).run_admitted(&admitted, &target);
    assert_eq!(
        native.scenarios[0].status,
        ess_conformance::report::Status::Error
    );
    let expected: Value = serde_json::from_str(
        &CountReport::from_run(&native, &admitted)
            .unwrap()
            .to_canonical_json()
            .unwrap(),
    )
    .unwrap();
    let (actual, trace) = live_run(
        &admitted,
        &raw,
        "structured",
        "correct",
        "structured-unbound",
    );
    assert_eq!(actual["outcomes"], expected["outcomes"]);
    assert!(!trace
        .iter()
        .any(|entry| entry["args"]["command"] == "release.rings.PlanRollout"));
}

#[test]
fn typescript_stops_target_error_after_prior_failed_delivery_assertion() {
    let input = delivery(false);
    let mut document: Value = serde_json::from_str(input.selected().original_json()).unwrap();
    document["scenarios"]
        .as_object_mut()
        .unwrap()
        .retain(|key, _| key.ends_with("/mapping"));
    let steps = scenario_steps(&mut document);
    let first = steps[0].clone();
    let last = steps[2].clone();
    let mut observation = steps[1].clone();
    observation["step"] = json!("expect_every_invocation");
    *steps = vec![first, observation.clone(), observation, last];
    let raw = document.to_string();
    let admitted = AdmittedSuite::from_json(&raw).unwrap();
    let target =
        support_typescript_prerequisite::Fixture::new("delivery", "wrong-then-observation-error");
    let native = Runner::for_suite(admitted.suite()).run_admitted(&admitted, &target);
    assert_eq!(
        native.scenarios[0].status,
        ess_conformance::report::Status::Failed
    );
    let expected: Value = serde_json::from_str(
        &CountReport::from_run(&native, &admitted)
            .unwrap()
            .to_canonical_json()
            .unwrap(),
    )
    .unwrap();
    let (actual, trace) = live_run(
        &admitted,
        &raw,
        "delivery",
        "wrong-then-observation-error",
        "delivery-stop-after-error",
    );
    assert_eq!(actual["outcomes"], expected["outcomes"]);
    assert_eq!(trace, *target.trace.lock().unwrap());
    assert_eq!(
        trace
            .iter()
            .filter(|entry| entry["method"] == "deliver")
            .count(),
        1
    );
    assert_eq!(
        trace
            .iter()
            .filter(|entry| entry["method"] == "invocations")
            .count(),
        2
    );
}
