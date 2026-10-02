//! One live target implementation for native and foreign runtime parity.
use ess_conformance::{interpret::Interpreted, target::*};
use ess_primitives::{node::Node, time::Timestamp};
use serde_json::{json, Value};
use std::{
    cell::RefCell,
    collections::BTreeMap,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
};

#[derive(Default)]
struct State {
    events: Vec<ObservedEvent>,
    invocations: Vec<ObservedInvocation>,
    deliveries: Vec<EventDeliveryRequest>,
    forced: bool,
    observations: usize,
}
pub struct Fixture {
    kind: String,
    mode: String,
    state: RefCell<State>,
    interpreted: Option<Interpreted>,
    pub trace: Arc<Mutex<Vec<Value>>>,
}
impl Fixture {
    pub fn new(kind: &str, mode: &str) -> Self {
        Self {
            kind: kind.into(),
            mode: mode.into(),
            state: RefCell::default(),
            interpreted: (kind == "structured").then(|| {
                Interpreted::for_model(super::model(include_str!(
                    "../fixtures/structured-instances.yaml"
                )))
            }),
            trace: Arc::default(),
        }
    }
    fn record(&self, method: &str, args: Value) {
        let mut entry = json!({"method":method});
        entry["args"] = args;
        self.trace.lock().unwrap().push(entry);
    }
    fn failure(&self, operation: &str) -> Result<(), TargetError> {
        match self.mode.as_str() {
            "unsupported" | "unsupported-teardown" => Err(TargetError::unsupported(
                operation,
                "fixture lacks capability",
            )),
            "error" => Err(TargetError::unavailable(operation, "fixture unavailable")),
            _ => Ok(()),
        }
    }
    fn react(&self, state: &mut State, request: &EventDeliveryRequest) {
        let mut input = BTreeMap::from([
            ("account_id".into(), request.context["account_id"].clone()),
            ("message_id".into(), request.payload["message_id"].clone()),
            ("peer".into(), request.payload["from"].clone()),
        ]);
        if self.mode == "ignore-context" || self.mode == "wrong-then-observation-error" {
            input.insert("account_id".into(), Node::Text("wrong".into()));
        }
        let mut invocation = ObservedInvocation::new(
            "received".parse().unwrap(),
            "demo.inbox.RecordMessage".parse().unwrap(),
        );
        invocation.input = input.clone();
        state.invocations.push(invocation.clone());
        if state.forced {
            state.forced = false;
            state.invocations.push(invocation);
        }
        state.events.push(
            ObservedEvent::new("demo.inbox.MessageRecorded".parse().unwrap())
                .with("account_id", input["account_id"].clone())
                .with("message_id", input["message_id"].clone()),
        );
    }
}
impl ConformanceTarget for Fixture {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.record("identity", Value::Null);
        Ok(ImplementationIdentity::new("prerequisite-fixture", "1"))
    }
    fn begin_scenario(&self, context: &ScenarioContext) -> Result<(), TargetError> {
        self.record("begin", json!({"scenario":context.scenario}));
        *self.state.borrow_mut() = State::default();
        if let Some(inner) = &self.interpreted {
            inner.begin_scenario(context)?;
        }
        Ok(())
    }
    fn end_scenario(&self, context: &ScenarioContext) -> Result<(), TargetError> {
        self.record("end", json!({"scenario":context.scenario}));
        if let Some(inner) = &self.interpreted {
            inner.end_scenario(context)?;
        }
        if self.mode.ends_with("-teardown") {
            return Err(TargetError::unavailable("end", "teardown unavailable"));
        }
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        self.record(
            "execute",
            json!({"command":request.command,"input":request.input}),
        );
        self.failure("execute")?;
        if let Some(inner) = &self.interpreted {
            return inner.execute_command(request);
        }
        assert_eq!(self.kind, "direct");
        let mut response: BTreeMap<String,Node>=serde_json::from_value(json!({"value":"actual","sequence":[1,2,2],"item":{"label":"nested","ordinal":9_007_199_254_740_993_i64}})).unwrap();
        match self.mode.as_str() {
            "wrong" => {
                response.insert("value".into(), Node::Text("wrong".into()));
            }
            "extra" => {
                response.insert("extra".into(), Node::Null);
            }
            "missing" | "missing-teardown" => {
                response.remove("value");
            }
            "rounded" => {
                let Node::Map(item) = response.get_mut("item").unwrap() else {
                    unreachable!()
                };
                item.insert(
                    "ordinal".into(),
                    Node::Number(9_007_199_254_740_992_i64.into()),
                );
            }
            "reorder" => {
                response.insert(
                    "sequence".into(),
                    serde_json::from_value(json!([2, 1, 2])).unwrap(),
                );
            }
            _ => {}
        }
        if self.mode == "large" {
            response.insert("value".into(), Node::Text("x".repeat(4097)));
            response.insert(
                "sequence".into(),
                Node::Seq(vec![Node::Number(1_i64.into()); 65]),
            );
        }
        if self.mode == "byte-edge" {
            response.insert("value".into(), Node::Text(String::new()));
            let overhead = serde_json::to_vec(&response).unwrap().len();
            response.insert("value".into(), Node::Text("x".repeat(1_048_576 - overhead)));
        }
        if self.mode == "oversized" {
            response.insert("value".into(), Node::Text("x".repeat(1_048_576)));
        }
        if let Some(depth) = self.mode.strip_prefix("json-") {
            let depth: usize = depth.parse().unwrap();
            response.insert(
                "value".into(),
                (0..depth).fold(Node::Null, |value, _| Node::Seq(vec![value])),
            );
        }
        if self.mode == "note-null" {
            let Node::Map(item) = response.get_mut("item").unwrap() else {
                unreachable!()
            };
            item.insert("note".into(), Node::Null);
        }
        let mut result = SemanticCommandResult::took(ess_compiler::refs::OutcomeRef::new(
            request.command,
            "returned".parse().unwrap(),
        ));
        if self.mode != "no-response" {
            result.response = Some(response);
        }
        Ok(result)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        self.record(
            "query",
            json!({"view":request.view,"params":request.params}),
        );
        self.failure("query")?;
        self.interpreted
            .as_ref()
            .expect("structured query")
            .query_view(request)
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        self.record("events", json!({"event":request.event}));
        self.failure("events")?;
        if let Some(inner) = &self.interpreted {
            return inner.observe_events(request);
        }
        Ok(self
            .state
            .borrow()
            .events
            .iter()
            .filter(|event| event.event == request.event)
            .cloned()
            .collect())
    }
    fn observe_invocations(
        &self,
        request: InvocationObservationRequest,
    ) -> Result<Vec<ObservedInvocation>, TargetError> {
        self.record(
            "invocations",
            json!({"binding":request.binding,"command":request.command}),
        );
        self.failure("invocations")?;
        let mut state = self.state.borrow_mut();
        state.observations += 1;
        if self.mode == "wrong-then-observation-error" && state.observations == 2 {
            return Err(TargetError::unavailable(
                "invocations",
                "second observation unavailable",
            ));
        }
        if self.mode == "late-wrong" && state.observations == 2 {
            if let Some(mut bad) = state.invocations.last().cloned() {
                bad.input
                    .insert("account_id".into(), Node::Text("wrong".into()));
                state.invocations.push(bad);
            }
        }
        Ok(state.invocations.clone())
    }
    fn configure_external_outcome(
        &self,
        request: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        self.record("configure", json!({"force":request.force}));
        self.state.borrow_mut().forced = true;
        Ok(())
    }
    fn deliver_event(&self, request: EventDeliveryRequest) -> Result<(), TargetError> {
        self.record("deliver",json!({"event":request.event,"authority":request.authority,"payload":request.payload,"context":request.context}));
        self.failure("deliver")?;
        let mut state = self.state.borrow_mut();
        state.deliveries.push(request.clone());
        self.react(&mut state, &request);
        Ok(())
    }
    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        self.record("redeliver", json!({"event":request.event}));
        self.failure("redeliver")?;
        let mut state = self.state.borrow_mut();
        let mut delivery = state.deliveries.last().unwrap().clone();
        if self.mode == "stale-context" {
            delivery.context = state.deliveries[0].context.clone();
        }
        self.react(&mut state, &delivery);
        Ok(())
    }
}

pub struct Host {
    pub address: String,
    handle: Option<thread::JoinHandle<Vec<Value>>>,
}
impl Host {
    pub fn start(kind: &str, mode: &str) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap().to_string();
        let kind = kind.to_owned();
        let mode = mode.to_owned();
        let handle = thread::spawn(move || {
            let fixture = Fixture::new(&kind, &mode);
            for stream in listener.incoming() {
                let mut stream = stream.unwrap();
                let mut line = String::new();
                BufReader::new(&mut stream).read_line(&mut line).unwrap();
                let request: Value = serde_json::from_str(&line).unwrap();
                if request["method"] == "stop" {
                    break;
                }
                let result = dispatch(&fixture, &request);
                let reply = match result {
                    Ok(value) => json!({"ok":value}),
                    Err(error) => {
                        json!({"error":error.to_string(),"unsupported":error.is_unsupported()})
                    }
                };
                writeln!(stream, "{reply}").unwrap();
            }
            let trace = fixture.trace.lock().unwrap().clone();
            trace
        });
        Self {
            address,
            handle: Some(handle),
        }
    }
    pub fn stop(mut self) -> Vec<Value> {
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
fn dispatch(target: &Fixture, request: &Value) -> Result<Value, TargetError> {
    let args = &request["args"];
    let text = |key: &str| args[key].as_str().unwrap();
    let correlation = ess_primitives::ids::CorrelationId::new("parity-1").unwrap();
    let deadline = Deadline::at(Timestamp::from_epoch_millis(0));
    match request["method"].as_str().unwrap() {
        "identity" => target
            .identity()
            .map(|_| json!({"Name":"prerequisite-fixture","Version":"1"})),
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
                    .filter(|value| !value.is_empty())
                    .map(|value| value.parse().unwrap()),
                caller: (!args["Caller"].is_null()).then(|| fields(&args["Caller"])),
                input: fields(&args["Input"]),
                correlation,
            })?;
            Ok(
                json!({"Outcome":result.outcome.map(|outcome|outcome.outcome.to_string()),"Response":result.response,"DirectEvents":result.direct_events.into_iter().map(|event|json!({"Event":event.event,"Payload":event.payload})).collect::<Vec<_>>()}),
            )
        }
        "events" => {
            let result = target.observe_events(EventObservationRequest {
                event: text("Event").parse().unwrap(),
                correlation,
                deadline,
            })?;
            Ok(Value::Array(
                result
                    .into_iter()
                    .map(|event| json!({"Event":event.event,"Payload":event.payload}))
                    .collect(),
            ))
        }
        "invocations" => {
            let result = target.observe_invocations(InvocationObservationRequest {
                binding: text("Binding").parse().unwrap(),
                command: text("Command").parse().unwrap(),
                correlation,
                deadline,
            })?;
            Ok(Value::Array(
                result
                    .into_iter()
                    .map(
                        |invocation| json!({"Command":invocation.command,"Input":invocation.input}),
                    )
                    .collect(),
            ))
        }
        "configure" => {
            target.configure_external_outcome(ExternalOutcomeControl {
                force: ess_compiler::refs::OutcomeRef::new(
                    text("Command").parse().unwrap(),
                    text("Outcome").parse().unwrap(),
                ),
                correlation,
            })?;
            Ok(Value::Null)
        }
        "deliver" => {
            target.deliver_event(EventDeliveryRequest {
                event: text("Event").parse().unwrap(),
                authority: text("Authority").into(),
                payload: fields(&args["Payload"]),
                context: fields(&args["Context"]),
                correlation,
            })?;
            Ok(Value::Null)
        }
        "redeliver" => {
            target.redeliver_event(RedeliveryRequest {
                event: text("Event").parse().unwrap(),
                correlation,
            })?;
            Ok(Value::Null)
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
        method => panic!("unknown callback {method}"),
    }
}
