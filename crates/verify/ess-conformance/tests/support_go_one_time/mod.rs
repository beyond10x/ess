//! Real callbacks into the frozen shared stateful Service; no prerecorded target answers.
use super::support_one_time::{self, Mode, Service, FIRST};
#[path = "../support_one_time/fields.rs"]
pub mod field_controls;
#[path = "../support_one_time/resources.rs"]
pub mod resources;
use ess_conformance::{scenario::Elapsed, target::*};
use ess_primitives::{node::Node, time::Timestamp};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    thread::{self, JoinHandle},
};

pub struct Snapshot {
    pub trace: Vec<&'static str>,
    pub plaintexts: Vec<String>,
    pub calls: usize,
}

pub struct Host {
    pub address: String,
    handle: Option<JoinHandle<Snapshot>>,
}

impl Host {
    pub fn start(mode: Mode) -> Self {
        Self::start_with(move || Service::new(mode))
    }

    pub fn resource(mode: resources::ResourceMode) -> Self {
        Self::start_with(move || resources::ResourceService::new(mode))
    }

    pub fn fields(mode: field_controls::FieldMode) -> Self {
        Self::start_with(move || field_controls::FieldService::new(mode))
    }

    fn start_with<T: LiveService>(factory: impl FnOnce() -> T + Send + 'static) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap().to_string();
        let handle = thread::spawn(move || {
            let service = factory();
            for stream in listener.incoming() {
                let mut stream = stream.unwrap();
                let mut line = String::new();
                BufReader::new(&mut stream).read_line(&mut line).unwrap();
                let request: Value = serde_json::from_str(&line).unwrap();
                if request["method"] == "stop" {
                    break;
                }
                let reply = match dispatch(&service, &request) {
                    Ok(value) => json!({"ok":value}),
                    Err(error) => {
                        json!({"error":error.to_string(),"unsupported":error.is_unsupported()})
                    }
                };
                writeln!(stream, "{reply}").unwrap();
            }
            Snapshot {
                trace: service.inner().trace(),
                plaintexts: service.plaintexts(),
                calls: service.inner().calls.get(),
            }
        });
        Self {
            address,
            handle: Some(handle),
        }
    }

    pub fn stop(mut self) -> Snapshot {
        let mut stream = TcpStream::connect(&self.address).unwrap();
        writeln!(stream, "{{\"method\":\"stop\"}}").unwrap();
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

trait LiveService: ConformanceTarget + 'static {
    fn inner(&self) -> &Service;
    fn plaintexts(&self) -> Vec<String> {
        self.inner().returned_plaintexts()
    }
}
impl LiveService for Service {
    fn inner(&self) -> &Service {
        self
    }
}
impl LiveService for resources::ResourceService {
    fn inner(&self) -> &Service {
        &self.inner
    }
}
impl LiveService for field_controls::FieldService {
    fn inner(&self) -> &Service {
        &self.inner
    }
    fn plaintexts(&self) -> Vec<String> {
        self.returned_plaintexts()
    }
}

fn dispatch(service: &impl ConformanceTarget, request: &Value) -> Result<Value, TargetError> {
    let args = &request["args"];
    let text = |key: &str| args[key].as_str().unwrap();
    let correlation = ess_primitives::ids::CorrelationId::new(
        args["Correlation"].as_str().unwrap_or("one-time-1"),
    )
    .unwrap();
    let deadline = Deadline::at(Timestamp::from_epoch_millis(
        args["Deadline"]["Attempts"].as_u64().unwrap_or(0) * 100,
    ));
    match request["method"].as_str().unwrap() {
        "identity" => {
            let identity = service.identity()?;
            Ok(json!({"Name":identity.name,"Version":identity.version}))
        }
        "begin" | "end" => {
            let context = ScenarioContext::new(text("Scenario").parse().unwrap(), correlation);
            if request["method"] == "begin" {
                service.begin_scenario(&context)?;
            } else {
                service.end_scenario(&context)?;
            }
            Ok(Value::Null)
        }
        "execute" => execute(service, args),
        "query" => {
            let result = service.query_view(SemanticViewRequest {
                view: text("View").parse().unwrap(),
                params: fields(&args["Params"]),
                correlation,
                deadline,
                consistency: ess_primitives::consistency::QueryConsistency::Current,
            })?;
            Ok(json!({"Rows":result.rows}))
        }
        "events" => {
            let result = service.observe_events(EventObservationRequest {
                event: text("Event").parse().unwrap(),
                correlation,
                deadline,
            })?;
            Ok(json!(result
                .into_iter()
                .map(|event| json!({"Event":event.event,"Payload":event.payload}))
                .collect::<Vec<_>>()))
        }
        "mark" => {
            service.mark_instant(InstantMark {
                instant: text("Instant").parse().unwrap(),
                correlation,
            })?;
            Ok(Value::Null)
        }
        "elapsed" => {
            let result = service.observe_elapsed(ElapsedObservationRequest {
                instant: text("Instant").parse().unwrap(),
                hold: Elapsed::seconds(u32::try_from(args["Hold"].as_u64().unwrap()).unwrap()),
                watching: args["Watching"]
                    .as_str()
                    .filter(|value| !value.is_empty())
                    .map(|value| value.parse().unwrap()),
                correlation,
            })?;
            Ok(json!({"ElapsedMillis":result.elapsed_ms,"Published":result.published}))
        }
        method => panic!("unexpected shared Service callback {method}"),
    }
}

fn execute(service: &impl ConformanceTarget, args: &Value) -> Result<Value, TargetError> {
    let result = service.execute_command(SemanticCommandRequest {
        command: args["Command"].as_str().unwrap().parse().unwrap(),
        actor: args["Actor"]
            .as_str()
            .filter(|value| !value.is_empty())
            .map(|value| value.parse().unwrap()),
        caller: (!args["Caller"].is_null()).then(|| fields(&args["Caller"])),
        input: fields(&args["Input"]),
        correlation: ess_primitives::ids::CorrelationId::new(args["Correlation"].as_str().unwrap())
            .unwrap(),
    })?;
    Ok(json!({
        "Outcome":result.outcome.map(|value|value.outcome.to_string()),
        "Response":result.response,
        "Error":result.error.as_ref().map(|value|value.error.to_string()).unwrap_or_default(),
        "ErrorPayload":result.error.map(|value|value.fields),
        "Consistency":result.consistency,
        "DirectEvents":result.direct_events.into_iter().map(|event|json!({"Event":event.event,"Payload":event.payload})).collect::<Vec<_>>()
    }))
}
