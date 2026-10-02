//! Live adapters call the same stateful Rust target; no answer transcript is replayed.
use super::Plant;
use ess_conformance::target::*;
use ess_primitives::{ids::CorrelationId, node::Node, time::Timestamp};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    thread,
};

pub struct Host {
    pub address: String,
    handle: Option<thread::JoinHandle<Plant>>,
}
impl Host {
    pub fn start(target: Plant) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap().to_string();
        let handle = thread::spawn(move || {
            for stream in listener.incoming() {
                let mut stream = stream.unwrap();
                let mut line = String::new();
                BufReader::new(&mut stream).read_line(&mut line).unwrap();
                let request: Value = serde_json::from_str(&line).unwrap();
                if request["method"] == "stop" {
                    break;
                }
                let reply = match dispatch(&target, &request) {
                    Ok(value) => json!({"ok":value}),
                    Err(error) => {
                        json!({"error":error.to_string(),"unsupported":error.is_unsupported()})
                    }
                };
                writeln!(stream, "{reply}").unwrap();
            }
            target
        });
        Self {
            address,
            handle: Some(handle),
        }
    }
    pub fn stop(mut self) -> Plant {
        writeln!(
            TcpStream::connect(&self.address).unwrap(),
            "{{\"method\":\"stop\"}}"
        )
        .unwrap();
        self.handle.take().unwrap().join().unwrap()
    }
}
impl Drop for Host {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            if let Ok(mut stream) = TcpStream::connect(&self.address) {
                let _ = writeln!(stream, "{{\"method\":\"stop\"}}");
            }
            let _ = handle.join();
        }
    }
}
fn fields(value: &Value) -> BTreeMap<String, Node> {
    if value.is_null() {
        BTreeMap::new()
    } else {
        serde_json::from_value(value.clone()).unwrap()
    }
}
fn dispatch(target: &Plant, request: &Value) -> Result<Value, TargetError> {
    let args = &request["args"];
    let text = |key: &str| args[key].as_str().unwrap();
    let correlation = CorrelationId::new("live-312").unwrap();
    let deadline = Deadline::at(Timestamp::from_epoch_millis(0));
    match request["method"].as_str().unwrap() {
        "identity" => {
            target.identity()?;
            Ok(json!({"Name":"adversary-287-fixture","Version":"1"}))
        }
        "begin" | "end" => {
            let context = ScenarioContext::new(text("Scenario").parse().unwrap(), correlation);
            if request["method"] == "begin" {
                target.begin_scenario(&context)?;
            } else {
                target.end_scenario(&context)?;
            }
            Ok(Value::Null)
        }
        "execute" => {
            let result = target.execute_command(SemanticCommandRequest {
                command: text("Command").parse().unwrap(),
                actor: args["Actor"]
                    .as_str()
                    .filter(|s| !s.is_empty())
                    .map(|s| s.parse().unwrap()),
                caller: (!args["Caller"].is_null()).then(|| fields(&args["Caller"])),
                input: fields(&args["Input"]),
                correlation,
            })?;
            Ok(
                json!({"Outcome":result.outcome.map(|o|o.outcome.to_string()),
                "Error":result.error.as_ref().map(|e|e.error.to_string()).unwrap_or_default(),
                "ErrorPayload":result.error.map(|e|e.fields),
                "Consistency":result.consistency,
                "DirectEvents":result.direct_events.into_iter().map(|e|json!({"Event":e.event,"Payload":e.payload})).collect::<Vec<_>>()}),
            )
        }
        "query" => {
            let result = target.query_view(SemanticViewRequest {
                view: text("View").parse().unwrap(),
                params: fields(&args["Params"]),
                correlation,
                deadline,
                consistency: ess_primitives::consistency::QueryConsistency::Current,
            })?;
            Ok(json!({"Rows":result.rows}))
        }
        "events" => {
            let result = target.observe_events(EventObservationRequest {
                event: text("Event").parse().unwrap(),
                correlation,
                deadline,
            })?;
            Ok(json!(result
                .into_iter()
                .map(|e| json!({"Event":e.event,"Payload":e.payload}))
                .collect::<Vec<_>>()))
        }
        other => panic!("unexpected actual callback {other}"),
    }
}

pub const GO: &str = r#"package essconform
import("bytes";"encoding/json";"fmt";"net";"os";"testing")
type liveTarget struct{}
func call(method string,args any,result any)error{
 socket,err:=net.Dial("tcp",os.Getenv("PARITY_ADDRESS"));if err!=nil{return err};defer socket.Close()
 if err=json.NewEncoder(socket).Encode(map[string]any{"method":method,"args":args});err!=nil{return err}
 var response struct{Ok json.RawMessage;Error string;Unsupported bool};if err=json.NewDecoder(socket).Decode(&response);err!=nil{return err}
 if response.Unsupported{return fmt.Errorf("%w: %s",ErrUnsupported,response.Error)};if response.Error!=""{return fmt.Errorf("%s",response.Error)}
 if result!=nil{decoder:=json.NewDecoder(bytes.NewReader(response.Ok));decoder.UseNumber();return decoder.Decode(result)};return nil
}
func(liveTarget)Identity()(v Identity,e error){e=call("identity",nil,&v);return}
func(liveTarget)BeginScenario(v ScenarioContext)error{return call("begin",v,nil)}
func(liveTarget)EndScenario(v ScenarioContext)error{return call("end",v,nil)}
func(liveTarget)ExecuteCommand(v CommandRequest)(r CommandResult,e error){e=call("execute",v,&r);return}
func(liveTarget)QueryView(v ViewRequest)(r ViewResult,e error){e=call("query",v,&r);return}
func(liveTarget)ObserveEvents(v EventObservationRequest)(r []ObservedEvent,e error){e=call("events",v,&r);return}
func(liveTarget)ObserveInvocations(v InvocationObservationRequest)(r []Invocation,e error){e=call("invocations",v,&r);return}
func(liveTarget)RedeliverEvent(v RedeliveryRequest)error{return call("redeliver",v,nil)}
func(liveTarget)ConfigureExternalOutcome(v ExternalOutcomeControl)error{return call("configure",v,nil)}
func TestLive(t *testing.T){Run(t,func()Target{return liveTarget{}})}
"#;

pub const TS: &str = r"
import {readFileSync} from 'node:fs';
import {connect} from 'node:net';
import {runWith,unsupported,strictJSON,goMarshal} from './dist/runtime.js';
const [file,address]=process.argv.slice(2);
async function call(method,args=null){const [host,port]=address.split(':');return await new Promise((resolve,reject)=>{
 const socket=connect({host,port:Number(port)});let bytes='';socket.on('error',reject);
 socket.on('connect',()=>socket.write(goMarshal({method,args})+'\n'));
 socket.on('data',chunk=>{bytes+=chunk.toString();if(bytes.includes('\n')){socket.end();try{
 const reply=strictJSON(bytes);if(reply.error)reject(reply.unsupported?unsupported(reply.error):new Error(reply.error));else resolve(reply.ok);
 }catch(error){reject(error)}}});});}
const upper=v=>Object.fromEntries(Object.entries(v).map(([k,v])=>[k[0].toUpperCase()+k.slice(1),v]));
const events=v=>(v??[]).map(e=>({event:e.Event,payload:e.Payload}));
const target=()=>({
 identity:async()=>{const v=await call('identity');return{name:v.Name,version:v.Version}},
 beginScenario:v=>call('begin',upper(v)),endScenario:v=>call('end',upper(v)),
 executeCommand:async r=>{const v=await call('execute',upper(r));return{outcome:v.Outcome??'',error:v.Error??'',errorPayload:v.ErrorPayload??undefined,consistency:v.Consistency??undefined,directEvents:events(v.DirectEvents)}},
 queryView:async r=>({rows:(await call('query',upper(r))).Rows}),observeEvents:async r=>events(await call('events',upper(r))),
 observeInvocations:r=>call('invocations',upper(r)),redeliverEvent:r=>call('redeliver',upper(r)),configureExternalOutcome:r=>call('configure',upper(r))
});
let failed=false;const scope={diagnostic:console.log,skip(){},async test(name,body){try{await body(this)}catch(error){failed=true;console.error(name,String(error))}}};
try{await runWith(scope,target,readFileSync(file,'utf8'));if(failed)process.exitCode=1}catch(error){console.error(String(error));process.exitCode=2}
";

pub fn wasm(suite: &ess_conformance::AdmittedSuite) -> Value {
    use std::fmt::Write as _;
    use std::{fs, path::Path, process::Command};
    let root = super::support_go::directory("initial-state-wasm");
    fs::create_dir_all(root.join("src")).unwrap();
    let conformance = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut manifest = "[package]\nname = \"initial-state-host\"\nversion = \"0.0.0\"\nedition = \"2021\"\n[workspace]\n[lib]\ncrate-type = [\"cdylib\"]\n[dependencies]\nserde_json = {version = \"1\", features = [\"arbitrary_precision\"]}\n".to_owned();
    for (name, path) in [
        ("ess-conformance", conformance.to_owned()),
        (
            "ess-primitives",
            conformance.join("../../specify/ess-primitives"),
        ),
        (
            "ess-compiler",
            conformance.join("../../specify/ess-compiler"),
        ),
        ("ess-domain", conformance.join("../../specify/ess-domain")),
    ] {
        writeln!(
            manifest,
            "{name} = {{path = {}}}",
            serde_json::to_string(&path.canonicalize().unwrap()).unwrap()
        )
        .unwrap();
    }
    fs::write(root.join("Cargo.toml"), manifest).unwrap();
    // Compile the identical stateful target into WASM. Test functions stay in the host crate.
    let source = include_str!("../adversary_287_pass1.rs")
        .split("#[test]")
        .next()
        .unwrap();
    fs::write(
        root.join("src/lib.rs"),
        format!("#![allow(dead_code, unused_imports)]\n{source}\n{WASM_HOST}"),
    )
    .unwrap();
    let run = |command: &mut Command| {
        let output = command.output().unwrap();
        assert!(
            output.status.success(),
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        output
    };
    run(Command::new("cargo")
        .args(["generate-lockfile", "--offline"])
        .current_dir(&root));
    let cache = std::env::var_os("CARGO_TARGET_DIR")
        .map_or_else(|| root.join("target"), std::path::PathBuf::from);
    run(Command::new("cargo")
        .args([
            "build",
            "--offline",
            "--locked",
            "--target",
            "wasm32-unknown-unknown",
        ])
        .arg("--target-dir")
        .arg(&cache)
        .current_dir(&root));
    let page = include_str!("../../../../generate/ess-synth/src/web/page.rs");
    let glue = page
        .split("const GLUE_BODY: &str = r#\"")
        .nth(1)
        .unwrap()
        .split("\"#;")
        .next()
        .unwrap();
    fs::write(root.join("bridge.mjs"),format!("{glue}\nexport const EXPORTS=['ess_input_reserve','ess_dispatch','ess_output_len']; export const REALIZE='ess_realize';\n")).unwrap();
    fs::write(root.join("suite.json"), suite.original_json()).unwrap();
    fs::write(
        root.join("driver.mjs"),
        r"
import {readFileSync} from 'node:fs';import {open} from './bridge.mjs';
const module=await open(readFileSync(process.argv[2]));
const suite=readFileSync('suite.json','utf8');
console.log(JSON.stringify([0,1,2,3].map(fault=>module.request({suite,fault}))));
",
    )
    .unwrap();
    let output = run(Command::new("node")
        .arg("driver.mjs")
        .arg(cache.join("wasm32-unknown-unknown/debug/initial_state_host.wasm"))
        .current_dir(&root));
    serde_json::from_slice(&output.stdout).unwrap()
}

const WASM_HOST: &str = r#"
thread_local! {
 static INPUT: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
 static OUTPUT: RefCell<String> = const { RefCell::new(String::new()) };
}
#[no_mangle] pub extern "C" fn ess_input_reserve(length:u32)->u32 {
 INPUT.with(|held| { let mut held=held.borrow_mut(); *held=vec![0;length as usize];held.as_mut_ptr() as u32 })
}
#[no_mangle] pub extern "C" fn ess_dispatch()->u32 {
 let answer=INPUT.with(|held| {
  let request:serde_json::Value=serde_json::from_slice(&held.borrow()).unwrap();
  let suite=AdmittedSuite::from_json(request["suite"].as_str().unwrap()).unwrap();
  let fault=match request["fault"].as_u64().unwrap(){0=>Fault::None,1=>Fault::KeysByCaller,2=>Fault::OnlyFirstCaller,_=>Fault::CannotIsolate};
  let target=plant(fault);
  let report=Runner::for_suite(suite.suite()).run_admitted(&suite,&target);
  let count=ess_conformance::counts::CountReport::from_run(&report,&suite).unwrap();
  let codes:std::collections::BTreeSet<_>=report.scenarios.iter().flat_map(|s|s.diagnostics()).map(|d|d.code.as_str()).collect();
  serde_json::json!({"report":serde_json::from_str::<serde_json::Value>(&count.to_canonical_json().unwrap()).unwrap(),"codes":codes,"calls":target.calls.borrow().len()}).to_string()
 });
 OUTPUT.with(|held| { *held.borrow_mut()=answer;held.borrow().as_ptr() as u32 })
}
#[no_mangle] pub extern "C" fn ess_output_len()->u32 {OUTPUT.with(|held|held.borrow().len() as u32)}
"#;
