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
            if covered { 35 } else { 34 }
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
    value["provenance"]
        .as_object_mut()
        .unwrap()
        .remove("scenario_initial_state");
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
    value["provenance"]
        .as_object_mut()
        .unwrap()
        .remove("scenario_initial_state");
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

#[path = "support_one_time/fields.rs"]
#[allow(dead_code)]
mod disclosure_fields;
#[path = "support_one_time/resources.rs"]
mod disclosure_resources;
#[allow(dead_code)]
mod support_one_time;
use support_one_time::{Mode, Service, FIRST};
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

fn expected_depth_suite(covered: bool, depth: usize) -> String {
    // Intentionally keep the original compact bytes, including the step tag after response.
    expected_depth_document(covered, depth).to_string()
}

fn expected_depth_document(covered: bool, depth: usize) -> Value {
    let source = constant(include_str!("direct_returns.rs"), "MODEL")
        .replace("name: value, type: String", "name: value, type: Json");
    let ir = model(&source);
    let mut document: Value = if covered {
        serde_json::from_str(
            build(&ir, &[], Scope::System, Origins::Generated)
                .unwrap()
                .selected()
                .original_json(),
        )
        .unwrap()
    } else {
        serde_json::to_value(ess_conformance::synthesize(&ir).suite).unwrap()
    };
    let response = scenario_steps(&mut document)
        .iter_mut()
        .find(|step| step["step"] == "expect_direct_response")
        .unwrap();
    response["response"]["expected"]["value"] =
        (0..depth).fold(Value::Null, |value, _| json!([value]));
    document
}

fn refused_depth_document(raw: &str, label: &str) {
    assert!(
        AdmittedSuite::from_json(raw).is_err(),
        "native admitted {label}"
    );
    let directory = runtime_package();
    let input = directory.join(format!("{label}.json"));
    let report = directory.join(format!("{label}-report.json"));
    std::fs::write(&input, raw).unwrap();
    let _ = std::fs::remove_file(&report);
    let host = support_typescript_prerequisite::Host::start("direct", "json-128");
    let output = Command::new("node")
        .arg(directory.join("live.mjs"))
        .arg(input)
        .arg(&host.address)
        .env("ESS_REPORT_FORMAT", "2")
        .env("ESS_REPORT_OUT", &report)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2), "TypeScript admitted {label}");
    assert!(!report.exists());
    assert!(host.stop().is_empty());
}

#[test]
fn typescript_direct_expected_literal_has_native_payload_depth_and_exact_parents() {
    for covered in [false, true] {
        let raw = expected_depth_suite(covered, 128);
        let admitted =
            AdmittedSuite::from_json(&raw).expect("native admits expected Json depth128");
        let target = support_typescript_prerequisite::Fixture::new("direct", "json-128");
        let native = Runner::for_suite(admitted.suite()).run_admitted(&admitted, &target);
        assert_eq!(
            native.scenarios[0].status,
            ess_conformance::report::Status::Passed
        );
        let (actual, _) = live_run(
            &admitted,
            &raw,
            "direct",
            "json-128",
            &format!("expected-depth128-{covered}"),
        );
        assert_eq!(actual["counts"]["passed"], 1);
        if covered {
            let input = AdmittedInput::from_suite(admitted).unwrap();
            let ids = input
                .selected()
                .suite()
                .scenarios
                .keys()
                .cloned()
                .collect::<Vec<_>>();
            let selected = input.select(&ids).unwrap();
            let wire = selected.document().to_canonical_json().unwrap();
            let (actual, _) = live_run(
                selected.selected(),
                &wire,
                "direct",
                "json-128",
                "expected-depth128-selected",
            );
            assert_eq!(actual["counts"]["passed"], 1);
            assert_eq!(
                selected.document().parent_suites[0],
                raw,
                "selection retains exact original parent bytes"
            );
        }
    }
}

#[test]
fn typescript_direct_depth_allowance_never_escapes_its_finite_envelope_path() {
    refused_depth_document(&expected_depth_suite(false, 129), "expected-depth129");
    let base = expected_depth_suite(false, 0);
    for (label, wrapped) in [("unrelated-deep", false), ("forged-expected-deep", true)] {
        let mut document: Value = serde_json::from_str(&base).unwrap();
        let nested = (0..128).fold(Value::Null, |value, _| json!([value]));
        let literal = if wrapped {
            json!({"step":"expect_direct_response","response":{"expected":{"value":nested}}})
        } else {
            nested
        };
        scenario_steps(&mut document)[0]["input"] =
            json!({"unrelated":{"kind":"literal","value":literal}});
        refused_depth_document(&document.to_string(), label);
    }
    let mut older = expected_depth_document(false, 128);
    older["provenance"]["suite_version"] = json!("ess-conformance/26");
    older["provenance"]
        .as_object_mut()
        .unwrap()
        .remove("scenario_initial_state");
    refused_depth_document(&older.to_string(), "old-deep-direct");
}

#[test]
fn typescript_one_time_dto_matches_all_immutable_native_admission_vectors() {
    let fixture_root =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/one-time-response");
    let mut files: Vec<_> = std::fs::read_dir(fixture_root)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .collect();
    files.sort();
    assert_eq!(files.len(), 25, "original23 plus two eventual-read vectors");
    check_one_time_documents(&files, "immutable");
}

fn check_one_time_documents(files: &[PathBuf], label: &str) {
    let package = runtime_package();
    let driver = package.join(format!("one-time-contract-{label}.mjs"));
    std::fs::write(
        &driver,
        r"
import {readFileSync} from 'node:fs';
import {strictJSON, admitSuite, admittedSuiteVersions, goMarshal} from './dist/runtime.js';
let port;
try { port = await import('./dist/one_time_response.js'); } catch {}
const authorities = [];
const fullAdmission = process.argv.slice(2).map(file => { try {admitSuite(readFileSync(file,'utf8'));return true;} catch{return false;} });
const results = process.argv.slice(2).map(file => {
  try {
    const document = strictJSON(readFileSync(file, 'utf8'));
    const policies = [];
    for (const scenario of Object.values(document.scenarios))
      policies.push(port.admitOneTimeTrace(scenario.one_time_response, scenario).authority);
    authorities.push(policies);
    return true;
  } catch { authorities.push(null); return false; }
});
process.stdout.write(goMarshal({results, fullAdmission, authorities, versions:admittedSuiteVersions()}));
",
    )
    .unwrap();
    let output = Command::new("node")
        .arg(driver)
        .args(files)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let actual: Value = serde_json::from_slice(&output.stdout).unwrap();
    let mut disagreements = Vec::new();
    for (index, file) in files.iter().enumerate() {
        let raw = std::fs::read_to_string(file).unwrap();
        let admitted = AdmittedSuite::from_json(&raw);
        if actual["results"][index] != admitted.is_ok()
            || actual["fullAdmission"][index] != admitted.is_ok()
        {
            disagreements.push(file.file_name().unwrap().to_string_lossy().to_string());
        }
        if let Ok(admitted) = admitted {
            let policies = admitted
                .suite()
                .scenarios
                .values()
                .map(|scenario| scenario.one_time_response.as_ref())
                .collect::<Vec<_>>();
            // Compare serialized spelling too: Value equality coalesces -0.0 and 0.0.
            let observed_policy = actual["authorities"][index].to_string();
            let native_policy = serde_json::to_value(policies).unwrap().to_string();
            assert!(
                observed_policy == native_policy,
                "canonical authority {}",
                file.display()
            );
        }
    }
    assert!(
        disagreements.is_empty(),
        "native/TypeScript DTO admission disagrees: {disagreements:?}"
    );
    assert!(
        actual["versions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|version| version == "ess-conformance/34"),
        "the implemented observer executes suite34"
    );
}

#[test]
fn typescript_emits_all_shared_one_time_execution_suites() {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/one-time-execution");
    let manifest: Vec<Value> =
        serde_json::from_str(&std::fs::read_to_string(fixtures.join("manifest.json")).unwrap())
            .unwrap();
    let mut refused = Vec::new();
    for case in manifest {
        let file = fixtures.join(case["suite"].as_str().unwrap());
        let admitted = AdmittedSuite::from_json(&std::fs::read_to_string(file).unwrap()).unwrap();
        if ess_conformance::ts::emit(admitted.suite()).is_err() {
            refused.push(case["case"].clone());
        }
    }
    assert!(
        refused.is_empty(),
        "live-executed suites refused by emitter: {refused:?}"
    );
}

#[test]
fn typescript_old_coverage_refusal_keeps_native_disclosure_id_grammar() {
    let input = direct(true);
    let mut value: Value = serde_json::from_str(input.selected().original_json()).unwrap();
    value["coverage"]["refused"] = json!([{
        "origin":"generated", "scenario":"library.api.Read/disclosure/returned/value/origin/as/anonymous",
        "subject":{"kind":"command","name":"library.api.Read"}, "source":null,
        "code":"ESS-SYNTH-001", "message":"candidate could not be arranged",
        "effect":"candidate_not_emitted", "retained":null,"scope":"in_scope","needs":[]
    }]);
    value["coverage"]["counts"]["refused"] = json!(1);
    let raw = value.to_string();
    AdmittedSuite::from_json(&raw)
        .expect("native IDs in a refusal are grammar, not executable vocabulary");
    let file = runtime_package().join("old-refused-disclosure-id.json");
    std::fs::write(&file, raw).unwrap();
    let output = Command::new("node").arg("--input-type=module").arg("-e")
        .arg("import {readFileSync} from 'node:fs'; import {admitSuite} from './dist/runtime.js'; admitSuite(readFileSync(process.argv[1],'utf8'));")
        .arg(file).current_dir(runtime_package()).output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn typescript_one_time_identifiers_match_native_grammar_and_authority() {
    let fixture_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let grammar = fixture_root.join("one-time-response-identifier-grammar.json");
    let mut files: Vec<_> = std::fs::read_dir(fixture_root.join("one-time-response-identifiers"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    files.sort();
    assert_eq!(files.len(), 11);
    let package = runtime_package();
    let driver = package.join("one-time-identifiers.mjs");
    std::fs::write(&driver, r"
import {readFileSync} from 'node:fs';
import {strictJSON, goMarshal} from './dist/runtime.js';
import * as port from './dist/one_time_response.js';
const grammar = strictJSON(readFileSync(process.argv[2], 'utf8'));
const parsed = grammar.map(([, id]) => { try { port.parseDisclosureId(id); return true; } catch { return false; } });
const admitted = process.argv.slice(3).map(file => {
  try {
    const document = strictJSON(readFileSync(file, 'utf8'));
    for (const [id, scenario] of Object.entries(document.scenarios)) {
      const trace = port.admitOneTimeTrace(scenario.one_time_response, scenario);
      port.admitDisclosureId(id, scenario, trace, document.provenance.suite_version);
    }
    return true;
  } catch { return false; }
});
process.stdout.write(goMarshal({parsed, admitted}));
").unwrap();
    let output = Command::new("node")
        .arg(driver)
        .arg(&grammar)
        .args(&files)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let observed: Value = serde_json::from_slice(&output.stdout).unwrap();
    let vectors: Vec<(bool, String)> =
        serde_json::from_str(&std::fs::read_to_string(grammar).unwrap()).unwrap();
    let mut mismatches = Vec::new();
    for (index, (expected, id)) in vectors.iter().enumerate() {
        assert_eq!(ess_conformance::ScenarioId::parse(id).is_ok(), *expected);
        if observed["parsed"][index] != *expected {
            mismatches.push(id.clone());
        }
    }
    for (index, file) in files.iter().enumerate() {
        let expected = AdmittedSuite::from_json(&std::fs::read_to_string(file).unwrap()).is_ok();
        if observed["admitted"][index] != expected {
            mismatches.push(file.file_name().unwrap().to_string_lossy().to_string());
        }
    }
    assert!(
        mismatches.is_empty(),
        "native/TypeScript disclosure identity disagreement: {mismatches:?}"
    );
}

fn one_time_driver() -> &'static PathBuf {
    static DRIVER: OnceLock<PathBuf> = OnceLock::new();
    DRIVER.get_or_init(|| {
    // Same lossless network adapter as prerequisite controls, extended only for the shared
    // service's declared-error and measured-window target surfaces.
    let driver = LIVE_DRIVER
        .replace("socket.on('error',reject);", "socket.setEncoding('utf8');socket.on('error',reject);")
        .replace("response:value.Response??undefined,directEvents", "response:value.Response??undefined,error:value.Error??'',errorPayload:value.ErrorPayload??undefined,consistency:value.Consistency??'',directEvents")
        .replace("redeliverEvent:async request", "markInstant:async request=>call('mark',upper(request)),observeElapsed:async request=>{const value=await call('elapsed',upper(request));return {elapsedMillis:value.ElapsedMillis,published:value.Published};},redeliverEvent:async request")
        .replace("const scope={diagnostic(){},skip(){}", "const scope={diagnostic(message){console.log(message);},skip(){}");
    let driver_path = runtime_package().join("one-time-live.mjs");
    std::fs::write(&driver_path, driver).unwrap();
    driver_path
    })
}

#[test]
fn typescript_one_time_live_observer_matches_all_shared_native_cases() {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/one-time-execution");
    let manifest: Vec<Value> =
        serde_json::from_str(&std::fs::read_to_string(fixtures.join("manifest.json")).unwrap())
            .unwrap();
    assert_eq!(manifest.len(), Mode::ALL.len());
    let package = runtime_package();
    let driver_path = one_time_driver();
    let mut failures = Vec::new();
    for case in manifest {
        let label = case["case"].as_str().unwrap();
        let input = fixtures.join(case["suite"].as_str().unwrap());
        let admitted = AdmittedSuite::from_json(&std::fs::read_to_string(&input).unwrap()).unwrap();
        let report = package.join(format!("one-time-{label}-report.json"));
        if report.exists() {
            std::fs::remove_file(&report).unwrap();
        }
        let mode = serde_json::from_value(case["case"].clone()).unwrap();
        let host = support_typescript_prerequisite::DisclosureHost::start(mode);
        let output = Command::new("node")
            .arg(driver_path)
            .arg(input)
            .arg(&host.address)
            .env("ESS_REPORT_FORMAT", "2")
            .env("ESS_REPORT_OUT", &report)
            .output()
            .unwrap();
        let (trace, captured) = host.stop();
        let diagnostic = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let report_bytes = std::fs::read_to_string(&report).unwrap_or_default();
        for value in captured
            .iter()
            .map(String::as_str)
            .chain([support_one_time::FIRST])
        {
            assert!(
                !diagnostic.contains(value) && !report_bytes.contains(value),
                "observed plaintext persisted for {label}"
            );
        }
        let expected_exit = i32::from(case["status"] != "passed");
        if output.status.code() != Some(expected_exit) || report_bytes.is_empty() {
            failures.push(format!(
                "{label}: exit {:?}, report exists {}",
                output.status.code(),
                !report_bytes.is_empty()
            ));
            continue;
        }
        CountReport::from_json(&report_bytes, &admitted).expect("actual produced report admits");
        let actual: Value = serde_json::from_str(&report_bytes).unwrap();
        let mut counts = actual["counts"].clone();
        assert_eq!(
            counts.as_object_mut().unwrap().remove("total"),
            Some(json!(1))
        );
        if counts != case["counts"]
            || json!(trace) != case["callback_trace"]
            || !diagnostic.contains(case["required_code"].as_str().unwrap())
        {
            failures.push(format!(
                "{label}: counts {counts}, callbacks {trace:?}, required diagnostic {}",
                diagnostic.contains(case["required_code"].as_str().unwrap())
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "shared live disclosure mismatches: {failures:?}"
    );
}

#[test]
fn typescript_one_time_live_resources_match_native_encoded_boundaries() {
    use disclosure_resources::ResourceMode;
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let manifest: Vec<Value> = serde_json::from_str(
        &std::fs::read_to_string(fixtures.join("one-time-resources.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(manifest.len(), ResourceMode::ALL.len());
    let input = fixtures.join("one-time-execution/view.json");
    let admitted = AdmittedSuite::from_json(&std::fs::read_to_string(&input).unwrap()).unwrap();
    let mut failures = Vec::new();
    for (mode, case) in ResourceMode::ALL.into_iter().zip(manifest) {
        assert_eq!(
            serde_json::to_value(mode.expected()).unwrap(),
            case["status"]
        );
        assert_eq!(serde_json::to_value(mode).unwrap(), case["case"]);
        let label = case["case"].as_str().unwrap();
        let report = runtime_package().join(format!("resource-{label}.json"));
        if report.exists() {
            std::fs::remove_file(&report).unwrap();
        }
        let host = support_typescript_prerequisite::DisclosureHost::start_resource(mode);
        let output = Command::new("node")
            .arg(one_time_driver())
            .arg(&input)
            .arg(&host.address)
            .env("ESS_REPORT_FORMAT", "2")
            .env("ESS_REPORT_OUT", &report)
            .output()
            .unwrap();
        let (trace, captured) = host.stop();
        let report_bytes = std::fs::read_to_string(&report).unwrap_or_default();
        let diagnostic = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        for value in captured
            .iter()
            .map(String::as_str)
            .chain([support_one_time::FIRST])
        {
            assert!(
                !diagnostic.contains(value) && !report_bytes.contains(value),
                "observed plaintext persisted for {label}"
            );
        }
        if output.status.code() != Some(i32::from(case["status"] != "passed"))
            || report_bytes.is_empty()
        {
            failures.push(format!(
                "{label}: exit {:?}, report exists {}",
                output.status.code(),
                !report_bytes.is_empty()
            ));
            continue;
        }
        CountReport::from_json(&report_bytes, &admitted).unwrap();
        let actual: Value = serde_json::from_str(&report_bytes).unwrap();
        let code = if case["status"] == "passed" {
            "ESS-CF-DISCLOSURE"
        } else {
            "ESS-CF-TARGET"
        };
        if actual["counts"] != case["counts"]
            || json!(trace) != case["callback_trace"]
            || !diagnostic.contains(code)
        {
            failures.push(format!(
                "{label}: counts {}, callbacks {trace:?}, required diagnostic {}",
                actual["counts"],
                diagnostic.contains(code)
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "shared resource mismatches: {failures:?}"
    );
}

#[test]
fn typescript_one_time_fields_windows_and_original_coverage35_match_native() {
    use disclosure_fields::FieldMode;
    use support_typescript_prerequisite::DisclosureHost;
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    for group in ["one-time-fields", "one-time-coverage", "one-time-windows"] {
        let covered = group == "one-time-coverage";
        let directory = fixtures.join(group);
        let input = directory.join(if covered { "input.json" } else { "suite.json" });
        let original = std::fs::read_to_string(&input).unwrap();
        let admitted = if covered {
            let carrier = AdmittedInput::from_json(&original).unwrap();
            assert_eq!(carrier.parents().len(), 1);
            assert_eq!(
                carrier.selected().suite().provenance.suite_version.major(),
                35
            );
            carrier.selected().clone()
        } else {
            AdmittedSuite::from_json(&original).unwrap()
        };
        let manifest: Vec<Value> = serde_json::from_str(
            &std::fs::read_to_string(directory.join("manifest.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(
            manifest.len(),
            if group == "one-time-windows" { 2 } else { 4 }
        );
        for (index, case) in manifest.into_iter().enumerate() {
            let host = if group == "one-time-fields" {
                let mode = FieldMode::ALL[index];
                assert_eq!(serde_json::to_value(mode).unwrap(), case["case"]);
                assert_eq!(
                    serde_json::to_value(mode.expected()).unwrap(),
                    case["status"]
                );
                DisclosureHost::start_fields(mode)
            } else {
                DisclosureHost::start(serde_json::from_value(case["case"].clone()).unwrap())
            };
            let report = runtime_package().join(format!("{group}-{index}.json"));
            if report.exists() {
                std::fs::remove_file(&report).unwrap();
            }
            let output = Command::new("node")
                .arg(one_time_driver())
                .arg(&input)
                .arg(&host.address)
                .env("ESS_REPORT_FORMAT", "2")
                .env("ESS_REPORT_OUT", &report)
                .output()
                .unwrap();
            let (trace, captured) = host.stop();
            let raw = std::fs::read_to_string(&report).unwrap_or_default();
            let diagnostic = format!(
                "{}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            for value in captured
                .iter()
                .map(String::as_str)
                .chain([support_one_time::FIRST])
            {
                assert!(
                    !raw.contains(value) && !diagnostic.contains(value),
                    "private value persisted: {group}/{index}"
                );
            }
            assert_eq!(
                output.status.code(),
                Some(i32::from(case["status"] != "passed")),
                "{group}/{index}: {diagnostic}"
            );
            CountReport::from_json(&raw, &admitted).unwrap();
            let actual: Value = serde_json::from_str(&raw).unwrap();
            assert_eq!(actual["counts"], case["counts"], "{group}/{index}");
            assert_eq!(json!(trace), case["callback_trace"], "{group}/{index}");
            assert!(
                diagnostic.contains(case["required_code"].as_str().unwrap()),
                "{group}/{index}"
            );
            assert_eq!(std::fs::read_to_string(&input).unwrap(), original);
        }
    }
}

#[test]
fn typescript_one_time_observer_preserves_special_observed_object_keys() {
    private_adapter_observation(
        "special-key",
        "return {rows:[Object.fromEntries([['__proto__',value.Rows[0].audit]])]};",
        "failed",
        "ESS-CF-DISCLOSURE",
    );
}

#[test]
fn typescript_one_time_observer_refuses_invalid_unicode_without_repair() {
    for (label, replacement) in [
        ("surrogate-value", r"return {rows:[{audit:'\ud800'}]};"),
        (
            "surrogate-key",
            r"return {rows:[Object.fromEntries([['\ud800','public']])]};",
        ),
    ] {
        private_adapter_observation(label, replacement, "unsupported", "ESS-CF-TARGET");
    }
}

fn private_adapter_observation(label: &str, replacement: &str, status: &str, code: &str) {
    let package = runtime_package();
    let driver = std::fs::read_to_string(one_time_driver()).unwrap();
    assert!(driver.contains("return {rows:value.Rows};"));
    // Relocate the same real service's leaked value to a valid own JSON object key. Neither
    // the target's sensitive bytes nor its actual callback execution is replaced by a fixture answer.
    let driver = driver.replace("return {rows:value.Rows};", replacement);
    let path = package.join(format!("one-time-{label}.mjs"));
    std::fs::write(&path, driver).unwrap();
    let input =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/one-time-execution/view.json");
    let report = package.join(format!("one-time-{label}-report.json"));
    if report.exists() {
        std::fs::remove_file(&report).unwrap();
    }
    let host = support_typescript_prerequisite::DisclosureHost::start(Mode::View);
    let output = Command::new("node")
        .arg(path)
        .arg(input)
        .arg(&host.address)
        .env("ESS_REPORT_FORMAT", "2")
        .env("ESS_REPORT_OUT", &report)
        .output()
        .unwrap();
    let (trace, captured) = host.stop();
    assert!(trace.contains(&"execute_command") && trace.contains(&"query_view"));
    let raw = std::fs::read_to_string(report).unwrap();
    let result: Value = serde_json::from_str(&raw).unwrap();
    assert_eq!(
        result["counts"][status], 1,
        "adapter observation boundary: {label}"
    );
    assert_eq!(output.status.code(), Some(1));
    let printed = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    for value in captured {
        assert!(!raw.contains(&value) && !printed.contains(&value));
    }
    assert!(printed.contains(code));
}

#[test]
fn typescript_one_time_ordinary_event_batch_eventual_bound() {
    for bytes in [500_000, 600_000] {
        ordinary_event_batch(false, bytes);
    }
}

#[test]
fn typescript_one_time_ordinary_event_batch_refusal_bound() {
    for bytes in [500_000, 600_000] {
        ordinary_event_batch(true, bytes);
    }
}

fn ordinary_event_batch(refusal: bool, bytes: usize) {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/one-time-execution/healthy.json");
    let mut document: Value =
        serde_json::from_str(&std::fs::read_to_string(fixture).unwrap()).unwrap();
    let scenario = document["scenarios"]
        .as_object_mut()
        .unwrap()
        .values_mut()
        .next()
        .unwrap();
    let steps = scenario["steps"].as_array_mut().unwrap();
    if refusal {
        steps.extend([
            json!({"step":"execute_command","command":"credentials.api.Issue","actor":"credentials.api.Denied"}),
            json!({"step":"expect_not_granted","actor":"credentials.api.Denied","unpublished":["credentials.api.Issued"]}),
        ]);
    } else {
        steps.push(json!({"step":"eventually_event","event":"credentials.api.Issued"}));
    }
    scenario["source"]
        .as_array_mut()
        .unwrap()
        .push(json!({"kind":"event","name":"credentials.api.Issued"}));
    let original = serde_json::to_string(&document).unwrap();
    let admitted = AdmittedSuite::from_json(&original).unwrap();
    let label = format!("event-batch-{refusal}-{bytes}");
    let input = runtime_package().join(format!("{label}.json"));
    std::fs::write(&input, original).unwrap();
    let driver = std::fs::read_to_string(one_time_driver()).unwrap()
        .replace("const target=()=>", "let eventReads=0;const target=()=>")
        .replace("const value=await call('execute',upper(request));return", "const value=await call('execute',upper(request));if(request.actor==='credentials.api.Denied')return {notGranted:true,notGrantedActor:request.actor,directEvents:[]};return")
        .replace("observeEvents:async request=>events(await call('events',upper(request))),", &format!("observeEvents:async request=>{{await call('events',upper(request));return ++eventReads===1?[{{event:request.event,payload:{{audit:'x'.repeat({bytes})}}}},{{event:request.event,payload:{{audit:'y'.repeat({bytes})}}}}]:[];}},"));
    let driver_path = runtime_package().join(format!("{label}.mjs"));
    std::fs::write(&driver_path, driver).unwrap();
    let report_path = runtime_package().join(format!("{label}-report.json"));
    if report_path.exists() {
        std::fs::remove_file(&report_path).unwrap();
    }
    let host = support_typescript_prerequisite::DisclosureHost::start(Mode::Healthy);
    let output = Command::new("node")
        .arg(driver_path)
        .arg(input)
        .arg(&host.address)
        .env("ESS_REPORT_FORMAT", "2")
        .env("ESS_REPORT_OUT", &report_path)
        .output()
        .unwrap();
    let (trace, captured) = host.stop();
    assert!(trace.contains(&"observe_events"));
    let raw = std::fs::read_to_string(report_path).unwrap_or_default();
    let diagnostic = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    for value in captured {
        assert!(!raw.contains(&value) && !diagnostic.contains(&value));
    }
    CountReport::from_json(&raw, &admitted).unwrap();
    let result: Value = serde_json::from_str(&raw).unwrap();
    let over = bytes == 600_000;
    assert_eq!(
        result["counts"],
        json!({"total":1,"passed":usize::from(!over),"failed":0,"error":0,"unsupported":usize::from(over),"skipped":0}),
        "{label}: complete event batch"
    );
    assert_eq!(output.status.code(), Some(i32::from(over)));
    assert!(diagnostic.contains(if over {
        "ESS-CF-TARGET"
    } else {
        "ESS-CF-DISCLOSURE"
    }));
}

#[test]
fn typescript_one_time_malformed_and_old_authority_refuses_before_callbacks() {
    let fixture_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let mut checked = 0;
    for group in ["one-time-response", "one-time-response-identifiers"] {
        for entry in std::fs::read_dir(fixture_root.join(group)).unwrap() {
            let file = entry.unwrap().path();
            let raw = std::fs::read_to_string(&file).unwrap();
            if AdmittedSuite::from_json(&raw).is_err() {
                refused_depth_document(
                    &raw,
                    &format!("{group}-{}", file.file_stem().unwrap().to_string_lossy()),
                );
                checked += 1;
            }
        }
    }
    assert!(checked >= 20, "the malformed vector inventory actually ran");
    let original =
        std::fs::read_to_string(fixture_root.join("one-time-response/valid-string.json")).unwrap();
    for version in ["ess-conformance/32", "ess-conformance/36"] {
        let raw = original.replace("ess-conformance/34", version);
        refused_depth_document(
            &raw,
            &format!("one-time-version-{}", version.replace('/', "-")),
        );
    }
}

#[test]
fn typescript_one_time_string_constraint_grammar_and_bounds_match_native() {
    let mut variations = vec![
        ("compact-count", json!("value.count >= 4")),
        (
            "compact-leading-zero-count",
            json!(format!("value.count >= {}4", "0".repeat(600))),
        ),
        (
            "compact-underflow-count",
            json!("value.count >= 1e-2147483648"),
        ),
        ("compact-fact-count", json!("value.count == value.count")),
        ("compact-text", json!("value == abcd")),
        ("compact-quoted-numeric", json!("value == \"1234\"")),
        ("compact-true", json!("true")),
        ("compact-false", json!("false")),
        ("compact-defined", json!("defined(value)")),
        ("compact-exists", json!("exists(value)")),
        ("compact-missing", json!("missing(value)")),
        ("compact-negated", json!("not not value")),
        ("compact-spaced-defined", json!("defined ( value )")),
        (
            "compact-large-count",
            json!("value.count >= 9007199254740993"),
        ),
        (
            "structured-large-count",
            json!({"value.count":{"gte":9_007_199_254_740_993_u64}}),
        ),
        (
            "structured-count-membership",
            json!({"value.count":{"in":[1e2,1e-7]}}),
        ),
        ("compact-null", json!("value == null")),
        ("compact-conjunction", json!("value == a && b")),
        ("scalar-count", json!({"value.count":4})),
        (
            "negative-zero-membership",
            json!({"value.count":{"in":[-0.0,0.0]}}),
        ),
        ("scalar-text", json!({"value":"abcd"})),
        ("compound-count", json!({"value.count":{"gte":4,"lt":8}})),
        ("all-alias", json!({"all_of":["value", "value.count >= 4"]})),
        ("none-alias", json!({"none_of_these":["value.count < 4"]})),
        ("none-with-not", json!({"none":[false],"not":false})),
        ("empty-all", json!({"all":null})),
        ("empty-any", json!({"any":null})),
        ("empty-not", json!({"not":null})),
        ("list-shorthand", json!({"value":["abcd", "abce"]})),
        ("list-alias", json!({"value":{"in":["abcd", "abce"]}})),
        ("none-alias-literals", json!({"value":{"not_in":"abdd"}})),
        ("defined-operator", json!({"value":{"defined":true}})),
        ("truthy-operator", json!({"value":{"truthy":true}})),
        ("starts-with", json!({"value":{"starts_with":"ab"}})),
        (
            "starts-with-numeric-text",
            json!({"value":{"starts_with":"123"}}),
        ),
        ("in-fold-empty", json!({"value":{"in_ignore_case":[]}})),
        ("empty-starts-with", json!({"value":{"starts_with":""}})),
        (
            "equals-fold",
            json!({"value":{"equals_ignore_case":"ABCD"}}),
        ),
        (
            "in-fold",
            json!({"value":{"in_ignore_case":["ABCD","ABCE"]}}),
        ),
        ("invalid-count-operand", json!("value.count >= abcd")),
        ("invalid-field-literal", json!("value == value")),
        ("invalid-path", json!("value.missing == abcd")),
        ("invalid-list-operand", json!({"value":{"any_of":[1]}})),
        (
            "invalid-quantified-string",
            json!({"forall":{"in":"value","as":"item","that":true}}),
        ),
        (
            "invalid-unknown-operator",
            json!({"value":{"invented":true}}),
        ),
    ];
    variations.push((
        "depth32",
        (0..32).fold(json!("value"), |inner, _| json!({"not":inner})),
    ));
    variations.push((
        "depth33",
        (0..33).fold(json!("value"), |inner, _| json!({"not":inner})),
    ));
    check_constraint_variations(variations);
}

fn check_constraint_variations(variations: Vec<(&str, Value)>) {
    let base: Value = serde_json::from_str(include_str!(
        "fixtures/one-time-response/valid-constrained-string.json"
    ))
    .unwrap();
    let directory = runtime_package().join("constraint-vectors");
    std::fs::create_dir_all(&directory).unwrap();
    let mut files = Vec::new();
    for (label, predicate) in variations {
        let mut document = base.clone();
        for scenario in document["scenarios"].as_object_mut().unwrap().values_mut() {
            scenario["one_time_response"]["origins"][0]["response"]["constraints"]
                ["credentials.api.Secret"]["invariants"] = json!([predicate]);
        }
        let file = directory.join(format!("{label}.json"));
        std::fs::write(&file, document.to_string()).unwrap();
        files.push(file);
    }
    let signed_zero = directory.join("negative-zero-token.json");
    let raw = std::fs::read_to_string(directory.join("negative-zero-membership.json")).unwrap();
    assert!(raw.contains("[-0.0,0.0]"));
    std::fs::write(&signed_zero, raw.replace("[-0.0,0.0]", "[-0,0.0]")).unwrap();
    files.push(signed_zero);
    check_one_time_documents(&files, "grammar");
}

#[test]
fn typescript_one_time_authority_uses_canonical_metadata_byte_bound() {
    let mut base: Value = serde_json::from_str(include_str!(
        "fixtures/one-time-response/valid-constrained-string.json"
    ))
    .unwrap();
    for scenario in base["scenarios"].as_object_mut().unwrap().values_mut() {
        let response = &mut scenario["one_time_response"]["origins"][0]["response"];
        response["fields"][0]["naming"] = json!({"summary":"", "display":null});
        response["fields"][0]["wire"] = Value::Null;
        response["fields"]
            .as_array_mut()
            .unwrap()
            .push(json!({"name":"metadata","type":" Optional< Map<String, Json> > "}));
        response["declarations"]["credentials.api.Secret"]["of"] = json!(" String ");
        response["constraints"]["credentials.api.Secret"]
            .as_object_mut()
            .unwrap()
            .remove("alphabet");
        response["constraints"]["credentials.api.Secret"]
            .as_object_mut()
            .unwrap()
            .remove("prefix");
    }
    let admitted = AdmittedSuite::from_json(&base.to_string()).unwrap();
    let trace = admitted
        .suite()
        .scenarios
        .values()
        .next()
        .unwrap()
        .one_time_response
        .as_ref()
        .unwrap();
    let minimal_size = serde_json::to_vec(trace).unwrap().len();
    let directory = runtime_package().join("authority-byte-vectors");
    std::fs::create_dir_all(&directory).unwrap();
    let mut files = Vec::new();
    for (pattern_index, pattern) in ["x", "\u{2028}", r"\u2028"].into_iter().enumerate() {
        let pattern_bytes = serde_json::to_string(pattern).unwrap().len() - 2;
        for size in [1024, 1_048_576, 1_048_577] {
            let mut document = base.clone();
            for scenario in document["scenarios"].as_object_mut().unwrap().values_mut() {
                scenario["one_time_response"]["origins"][0]["response"]["fields"][0]["naming"]
                    ["summary"] = json!(format!(
                    "{}{}",
                    pattern.repeat((size - minimal_size) / pattern_bytes),
                    "x".repeat((size - minimal_size) % pattern_bytes)
                ));
            }
            let raw = document.to_string();
            assert_eq!(AdmittedSuite::from_json(&raw).is_ok(), size <= 1_048_576);
            let file = directory.join(format!("bytes-{size}-{pattern_index}.json"));
            std::fs::write(&file, raw).unwrap();
            files.push(file);
        }
    }
    check_one_time_documents(&files, "bytes");
}

#[test]
fn typescript_both_explorers_refuse_private_models_before_callbacks_or_recording() {
    let ir = model("format: ess/21\nsystem: credentials\nversion: v1\ndomain: credentials.api\ncommands:\n  - name: credentials.api.Issue\n    response: [{name: secret, type: String}]\n    outcomes: [{name: issued, returns: true, one_time_response: [secret]}]\n");
    // The independently supplied model is the authority even if the suite contains only
    // older executable steps. This exercises both actual explorer entry points.
    let suite = ess_conformance::synthesize(&ir).suite;
    let directory = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../target/backlog-input/ts-one-time-explorer");
    for artifact in ess_conformance::ts::emit_with_model(&suite, &ir).unwrap() {
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
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    let driver = package.join("private-explorer.mjs");
    std::fs::write(
        &driver,
        r"
import {existsSync} from 'node:fs';
import {loadModel, explore, exploreConcurrent} from './dist/explore.js';
const out = process.argv[2];
let callbacks = 0;
const factory = () => { callbacks++; throw new Error('target must never be opened'); };
const refusals = [];
for (const operation of [() => loadModel(), () => explore(factory, {seeds:1, steps:1}),
    () => exploreConcurrent(factory, {seed:1, calls:1, path:'contract.yaml', out})]) {
  try { await operation(); refusals.push(false); }
  catch(error) { refusals.push(error.message === 'UnsupportedOneTimeDisclosure'); }
}
process.stdout.write(JSON.stringify({refusals, callbacks, written:existsSync(out)}));
",
    )
    .unwrap();
    let out = package.join(format!("must-not-record-{}", std::process::id()));
    assert!(!out.exists());
    let output = Command::new("node").arg(driver).arg(&out).output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let result: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        result,
        json!({"refusals":[true,true,true],"callbacks":0,"written":false})
    );
}
