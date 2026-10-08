//! The emitted TypeScript runtime gives the reference runner's verdict on every
//! binding-arrangement control (beyond10x/ess#266, beyond10x/ess#267,
//! `docs/design/binding-arrangement-and-drop.md`).
//!
//! The TypeScript runner drives each target live, over a socket, through a driver that maps every
//! callback onto the same Rust target the reference runner is given: the interpreter's own
//! dispatcher, or that dispatcher with one thing wrong. Each run publishes report/2; its outcomes
//! must equal the reference run's, scenario by scenario.

mod support_binding_arrangement;

use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;
use std::thread;

use ess_conformance::synthesize::synthesize;
use ess_conformance::target::*;
use ess_conformance::{AdmittedSuite, CountReport, Runner};
use ess_primitives::ids::CorrelationId;
use ess_primitives::node::Node;
use ess_primitives::time::Timestamp;
use serde_json::{json, Value};
use support_binding_arrangement::*;

fn admitted() -> &'static AdmittedSuite {
    static SUITE: OnceLock<AdmittedSuite> = OnceLock::new();
    SUITE.get_or_init(|| {
        AdmittedSuite::from_suite(&synthesize(&model(FIXTURE)).suite)
            .expect("the suite is admitted")
    })
}

/// The emitted package, compiled once.
fn package() -> &'static PathBuf {
    static PACKAGE: OnceLock<PathBuf> = OnceLock::new();
    PACKAGE.get_or_init(|| {
        // Cached for the whole process, so it lives under the build's directory and not in
        // $TMPDIR: a static is never dropped, so no scratch guard could remove it.
        let directory = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("binding-arrangement-ts-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&directory);
        for artifact in ess_conformance::ts::emit(admitted().suite()).unwrap() {
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
            .expect("the TypeScript compiler is required for this lane");
        assert!(
            output.status.success(),
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        std::fs::write(package.join("live.mjs"), DRIVER).unwrap();
        package
    })
}

/// Every callback, forwarded to the host as one JSON line; every answer, mapped back.
const DRIVER: &str = r"
import {readFileSync} from 'node:fs';
import {connect} from 'node:net';
import {runWith, unsupported, goMarshal, JsonNumber} from './dist/runtime.js';
const [file,address] = process.argv.slice(2);
async function call(method,args=null) {
 const [host,port]=address.split(':');
 return await new Promise((resolve,reject)=>{
  const socket=connect({host,port:Number(port)});let bytes='';
  socket.on('error',reject);
  socket.on('connect',()=>socket.write(goMarshal({method,args})+'\n'));
  socket.on('data',chunk=>{bytes+=chunk.toString();if(bytes.includes('\n')) {socket.end();try {
   const reply=JSON.parse(bytes,(_key,value,context)=>typeof value==='number'?new JsonNumber(context.source):value);
   if(reply.error!==undefined)reject(reply.unsupported?unsupported(reply.error):new Error(reply.error));else resolve(reply.ok);
  }catch(error){reject(error);}}});
 });
}
const target=()=>({
 identity:async()=>call('identity'),
 beginScenario:async value=>call('begin',value), endScenario:async value=>call('end',value),
 executeCommand:async request=>{const value=await call('execute',request);return {
   outcome:value.outcome??undefined,error:value.error??undefined,errorPayload:value.errorPayload??undefined,
   consistency:value.consistency??undefined,response:value.response??undefined,directEvents:value.directEvents};},
 queryView:async request=>call('query',request),
 observeEvents:async request=>call('events',request),
 observeInvocations:async request=>call('invocations',request),
 configureExternalOutcome:async request=>call('configure',request),
 configureExternalOutcomeRepeatedly:async request=>call('configure',request),
 redeliverEvent:async request=>call('redeliver',request)
});
let failed=false;
const scope={diagnostic(){},skip(){},async test(name,body){try{await body(this);}catch(error){failed=true;console.error(`${name}: ${String(error)}`);}}};
try {await runWith(scope,target,readFileSync(file,'utf8'));if(failed)process.exitCode=1;}
catch(error){console.error(String(error));process.exitCode=2;}
";

fn nodes(value: &Value) -> std::collections::BTreeMap<String, Node> {
    if value.is_null() {
        std::collections::BTreeMap::new()
    } else {
        serde_json::from_value(value.clone()).unwrap()
    }
}

fn events(events: Vec<ObservedEvent>) -> Value {
    Value::Array(
        events
            .into_iter()
            .map(|event| json!({"event": event.event, "payload": event.payload}))
            .collect(),
    )
}

/// The `execute` callback: one command, and everything it answered, refusals included.
fn execute(target: &impl ConformanceTarget, args: &Value) -> Result<Value, TargetError> {
    let text = |key: &str| args[key].as_str().unwrap_or_default().to_owned();
    let result = target.execute_command(SemanticCommandRequest {
        command: text("command").parse().unwrap(),
        actor: Some(text("actor"))
            .filter(|actor| !actor.is_empty())
            .map(|actor| actor.parse().unwrap()),
        caller: (!args["caller"].is_null()).then(|| nodes(&args["caller"])),
        input: nodes(&args["input"]),
        correlation: CorrelationId::new(text("correlation")).unwrap(),
    })?;
    Ok(json!({
        "outcome": result.outcome.map(|outcome| outcome.outcome.to_string()),
        "error": result.error.as_ref().map(|error| error.error.to_string()),
        "errorPayload": result.error.map(|error| error.fields),
        "consistency": result.consistency.map(|token| token.to_string()),
        "response": result.response,
        "directEvents": events(result.direct_events),
    }))
}

/// One callback of the TypeScript runner, answered by `target`.
fn answer(target: &impl ConformanceTarget, request: &Value) -> Result<Value, TargetError> {
    let args = &request["args"];
    let text = |key: &str| args[key].as_str().unwrap_or_default().to_owned();
    let correlation = || CorrelationId::new(text("correlation")).unwrap();
    let deadline = || {
        Deadline::at(Timestamp::from_epoch_millis(
            args["deadline"]["attempts"].as_u64().unwrap_or(0),
        ))
    };
    match request["method"].as_str().unwrap() {
        "identity" => target
            .identity()
            .map(|identity| json!({"name": identity.name, "version": identity.version})),
        method @ ("begin" | "end") => {
            let context = ScenarioContext::new(text("scenario").parse().unwrap(), correlation());
            if method == "begin" {
                target.begin_scenario(&context)?;
            } else {
                target.end_scenario(&context)?;
            }
            Ok(Value::Null)
        }
        "execute" => execute(target, args),
        "query" => {
            let consistency = match text("atLeast").as_str() {
                "" => ess_primitives::consistency::QueryConsistency::Current,
                token => ess_primitives::consistency::QueryConsistency::at_least(
                    ess_primitives::consistency::ConsistencyToken::new(token).unwrap(),
                ),
            };
            let result = target.query_view(SemanticViewRequest {
                view: text("view").parse().unwrap(),
                params: nodes(&args["params"]),
                correlation: correlation(),
                deadline: deadline(),
                consistency,
            })?;
            Ok(json!({"rows": result.rows}))
        }
        "events" => Ok(events(target.observe_events(EventObservationRequest {
            event: text("event").parse().unwrap(),
            correlation: correlation(),
            deadline: deadline(),
        })?)),
        "invocations" => Ok(Value::Array(
            target
                .observe_invocations(InvocationObservationRequest {
                    binding: ess_conformance::scenario::BindingRef::new(
                        ess_domain::binding::BindingName::new(text("binding")).unwrap(),
                    ),
                    command: text("command").parse().unwrap(),
                    correlation: correlation(),
                    deadline: deadline(),
                })?
                .into_iter()
                .map(|invocation| json!({"command": invocation.command, "input": invocation.input}))
                .collect(),
        )),
        "configure" => {
            let control = ExternalOutcomeControl {
                force: ess_conformance::scenario::OutcomeRef::new(
                    text("command").parse().unwrap(),
                    text("outcome").parse().unwrap(),
                ),
                correlation: correlation(),
            };
            match args["times"].as_u64() {
                Some(times) => target.configure_external_outcome_repeatedly(
                    control,
                    std::num::NonZeroU32::new(u32::try_from(times).unwrap()).unwrap(),
                )?,
                None => target.configure_external_outcome(control)?,
            }
            Ok(Value::Null)
        }
        "redeliver" => {
            target.redeliver_event(RedeliveryRequest {
                event: text("event").parse().unwrap(),
                correlation: correlation(),
            })?;
            Ok(Value::Null)
        }
        method => panic!("unknown callback {method}"),
    }
}

/// Serves one target to one TypeScript run, then stops.
fn serve<T: ConformanceTarget + 'static>(
    make: impl FnOnce() -> T + Send + 'static,
) -> (String, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap().to_string();
    let handle = thread::spawn(move || {
        let target = make();
        for stream in listener.incoming() {
            let mut stream = stream.unwrap();
            let mut line = String::new();
            BufReader::new(&mut stream).read_line(&mut line).unwrap();
            let request: Value = serde_json::from_str(&line).unwrap();
            if request["method"] == "stop" {
                break;
            }
            let reply = match answer(&target, &request) {
                Ok(value) => json!({"ok": value}),
                Err(error) => {
                    json!({"error": error.to_string(), "unsupported": error.is_unsupported()})
                }
            };
            writeln!(stream, "{reply}").unwrap();
        }
    });
    (address, handle)
}

/// The TypeScript verdicts and the reference verdicts for one target, as report/2 outcomes.
fn compare<T: ConformanceTarget + 'static>(
    label: &str,
    make: impl Fn() -> T + Send + Clone + 'static,
) -> (Value, Value) {
    let native = Runner::for_suite(admitted().suite()).run_admitted(admitted(), &make());
    let expected: Value = serde_json::from_str(
        &CountReport::from_run(&native, admitted())
            .unwrap()
            .to_canonical_json()
            .unwrap(),
    )
    .unwrap();

    let package = package();
    let input = package.join(format!("{label}.json"));
    let report = package.join(format!("{label}-report.json"));
    std::fs::write(&input, admitted().original_json()).unwrap();
    let (address, handle) = serve(make);
    let output = Command::new("node")
        .arg(package.join("live.mjs"))
        .arg(&input)
        .arg(&address)
        .env("ESS_REPORT_FORMAT", "2")
        .env("ESS_REPORT_OUT", &report)
        .output()
        .unwrap();
    let mut stop = TcpStream::connect(&address).unwrap();
    writeln!(stop, "{{\"method\":\"stop\"}}").unwrap();
    handle.join().unwrap();
    assert_ne!(
        output.status.code(),
        Some(2),
        "{label}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let actual: Value =
        serde_json::from_str(&std::fs::read_to_string(&report).unwrap_or_else(|error| {
            panic!(
                "{label}: {error}: {}",
                String::from_utf8_lossy(&output.stderr)
            )
        }))
        .unwrap();
    (actual["outcomes"].clone(), expected["outcomes"].clone())
}

/// The status report/2 files `id` under: its `outcomes` lists scenario ids by status.
fn verdict(outcomes: &Value, id: &str) -> String {
    outcomes
        .as_object()
        .and_then(|by_status| {
            by_status.iter().find_map(|(status, ids)| {
                ids.as_array()?
                    .iter()
                    .any(|listed| listed == id)
                    .then(|| status.clone())
            })
        })
        .unwrap_or_else(|| panic!("`{id}` has a verdict: {outcomes}"))
}

fn not_passed(outcomes: &Value) -> Vec<String> {
    outcomes
        .as_object()
        .unwrap()
        .iter()
        .filter(|(status, _)| *status != "passed")
        .flat_map(|(_, ids)| ids.as_array().unwrap().iter())
        .map(|id| id.as_str().unwrap().to_owned())
        .collect()
}

#[test]
fn typescript_passes_the_honest_binding_running_target() {
    let (actual, expected) = compare("honest", || interpreted(FIXTURE));
    assert_eq!(actual, expected);
    assert_eq!(not_passed(&actual), Vec::<String>::new(), "{actual}");
}

#[test]
fn typescript_fails_a_dispatcher_without_the_binding_and_a_destination_left_ineligible() {
    let (actual, expected) = compare("disabled", || interpreted(&without_created_starts()));
    assert_eq!(actual, expected);
    assert_eq!(verdict(&actual, "created-starts/binding/flow"), "failed");
    let (actual, expected) = compare("stopped", || {
        interpreted(&lands_stopped("jobs.job.Created"))
    });
    assert_eq!(actual, expected);
    assert_eq!(verdict(&actual, "created-starts/binding/flow"), "failed");
    assert_eq!(
        verdict(&actual, "created-starts/binding/delivery"),
        "failed"
    );
}

#[test]
fn typescript_fails_every_drop_control() {
    for (label, outcomes) in [
        (
            "zero",
            compare("drop-zero", || interpreted(&without_kicked_starts())),
        ),
        (
            "retry",
            compare("drop-retry", || interpreted(&kicked_retries())),
        ),
        (
            "malformed",
            compare("drop-malformed", || {
                Faulty::new(FIXTURE, Fault::MalformedRetry)
            }),
        ),
        (
            "success",
            compare("drop-success", || Faulty::new(FIXTURE, Fault::SwallowForce)),
        ),
        (
            "late",
            // The TypeScript harness gives each eventual step 8 attempts, so the every-invocation
            // step asks 8 times and the count step's first answer is one; the retry shows on the
            // count step's fourth ask. The reference runner's window is longer, so there it shows
            // already during the every-invocation step, and both fail the count.
            compare("drop-late", || Faulty::new(FIXTURE, Fault::LateRetry(11))),
        ),
    ] {
        let (actual, expected) = outcomes;
        assert_eq!(actual, expected, "{label}");
        assert_eq!(verdict(&actual, DROP), "failed", "{label}");
    }
}
