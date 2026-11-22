//! The cases a generated server carrying a finite `Binary64` is held to, shared by
//! `binary64_rust.rs` and `binary64_go.rs`: the fixture model, the harnesses' line protocol, the
//! wire table and the synthesized suite.
//!
//! A harness speaks one JSON object per line each way, as `integer_spellings.rs`' harnesses do:
//! `{"op":"reset"}`, `{"op":"command","command":…,"actor":…,"body":…}` and
//! `{"op":"view","view":…,"query":…}`, each answered `{"status":N,"body":…}` with the served
//! body verbatim. The wire cases read the raw answer line, never a value a JSON parser rebuilt,
//! so a token is checked as the server wrote it.

#![allow(dead_code)]

use std::cell::RefCell;
use std::io::{BufRead as _, BufReader, Write as _};
use std::path::Path;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::time::{Duration, Instant};

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile_locating;
use ess_compiler::source::SourceMap;
use ess_conformance::report::Status;
use ess_conformance::scenario::{CommandRef, ErrorRef, EventRef, OutcomeRef};
use ess_conformance::target::{
    ConformanceTarget, DeclaredErrorValue, EventObservationRequest, ExternalOutcomeControl,
    ImplementationIdentity, ObservedEvent, RedeliveryRequest, ScenarioContext,
    SemanticCommandRequest, SemanticCommandResult, SemanticViewRequest, SemanticViewResult,
    TargetError, ViewRow,
};
use ess_conformance::{AdmittedSuite, Runner};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::consistency::ConsistencyToken;
use ess_primitives::node::Node;
use serde_json::{json, Value};

/// The model, read by `task binary64-harness` too, which generates the Rust server from it.
pub const MODEL: &str = include_str!("../fixtures/binary64-served/model.yaml.in");
pub const ADD: &str = "demo.search.AddDoc";
pub const DOCS: &str = "demo.search.DocsByAdded";
pub const CLERK: &str = "demo.search.Clerk";

/// Each score token sent, and the token the server must write back.
pub const WRITTEN: [(&str, &str); 15] = [
    ("0.73", "0.73"),
    ("-0.0", "-0.0"),
    ("-0", "-0.0"),
    ("3", "3.0"),
    ("1e16", "1e16"),
    ("1e15", "1000000000000000.0"),
    ("0.00001", "0.00001"),
    ("0.000001", "1e-6"),
    ("9007199254740993", "9007199254740992.0"),
    ("5e-324", "5e-324"),
    ("1e-400", "0.0"),
    ("-1e-400", "-0.0"),
    ("1.7976931348623157e308", "1.7976931348623157e308"),
    ("123.456", "123.456"),
    ("1.5e300", "1.5e300"),
];

/// Each score a server must refuse with `400`, and what the refusal says it found.
pub const REFUSED: [(&str, &str); 6] = [
    ("1e400", "the number 1e400"),
    ("-1e400", "the number -1e400"),
    (
        "1.7976931348623159e308",
        "the number 1.7976931348623159e308",
    ),
    ("1e999999999", "the number 1e999999999"),
    ("\"0.73\"", "a string"),
    ("true", "a boolean"),
];

/// Compiles `source` as the one file `model.yaml`.
pub fn compile(source: &str) -> EssIr {
    let raw = RawSpecFile::parse(source).unwrap_or_else(|error| panic!("well formed: {error}"));
    let specification = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("validates:\n{errors}"));
    let mut sources = SourceMap::new();
    sources.insert("model.yaml".to_owned(), source.to_owned());
    compile_locating(&specification, &sources, &["model.yaml".to_owned()])
        .unwrap_or_else(|diagnostics| panic!("resolves:\n{diagnostics}"))
}

/// The fixture model, compiled.
pub fn model() -> EssIr {
    compile(MODEL)
}

/// Every route of the model's component, as `name method path` triples.
pub fn routes(ir: &EssIr) -> Vec<String> {
    let mut routes = Vec::new();
    for component in ir.components().values() {
        for route in ess_gen::http::routes(ir, component) {
            let name = match route.serves {
                ess_gen::http::Served::Command(handle) => ir.command(handle).name.to_string(),
                ess_gen::http::Served::View(handle) => ir.view(handle).name.to_string(),
            };
            routes.extend([name, route.method.as_str().to_owned(), route.path]);
        }
    }
    routes
}

/// One running harness, and the events one scenario published.
pub struct Served {
    child: RefCell<Child>,
    stdin: RefCell<ChildStdin>,
    stdout: RefCell<BufReader<ChildStdout>>,
    published: RefCell<Vec<ObservedEvent>>,
    sequence: RefCell<u64>,
}

impl Served {
    /// Starts the harness `binary`, handing it the model's routes.
    pub fn start(binary: &Path, ir: &EssIr) -> Self {
        let mut child = Command::new(binary)
            .args(routes(ir))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("the harness starts");
        let stdin = child.stdin.take().expect("piped");
        let stdout = BufReader::new(child.stdout.take().expect("piped"));
        Self {
            child: RefCell::new(child),
            stdin: RefCell::new(stdin),
            stdout: RefCell::new(stdout),
            published: RefCell::default(),
            sequence: RefCell::new(0),
        }
    }

    /// One request line out, the answer line back verbatim.
    pub fn raw(&self, request: &Value) -> String {
        let mut stdin = self.stdin.borrow_mut();
        writeln!(stdin, "{request}").expect("the harness reads");
        stdin.flush().expect("the harness reads");
        let mut line = String::new();
        self.stdout
            .borrow_mut()
            .read_line(&mut line)
            .expect("the harness answers");
        line.trim_end().to_owned()
    }

    /// One request line out, one `{status, body}` line back, parsed.
    pub fn ask(&self, request: &Value) -> (u64, Value) {
        let line = self.raw(request);
        let answer: Value =
            serde_json::from_str(&line).unwrap_or_else(|error| panic!("`{line}`: {error}"));
        (
            answer["status"].as_u64().unwrap_or_default(),
            answer["body"].clone(),
        )
    }

    /// Empties the harness and this client's record of what it published.
    pub fn reset(&self) {
        self.ask(&json!({"op": "reset"}));
        self.published.borrow_mut().clear();
        *self.sequence.borrow_mut() = 0;
    }

    /// Sends `body`, as written, to the `AddDoc` route.
    pub fn add(&self, body: &str) -> (u64, Value) {
        self.ask(&json!({"op": "command", "command": ADD, "actor": CLERK, "body": body}))
    }

    /// The raw answer line of the `DocsByAdded` view.
    pub fn docs_line(&self) -> String {
        self.raw(&json!({"op": "view", "view": DOCS, "query": ""}))
    }

    fn tick(&self) -> u64 {
        let mut sequence = self.sequence.borrow_mut();
        *sequence += 1;
        *sequence
    }
}

impl Drop for Served {
    fn drop(&mut self) {
        let _ = self.child.borrow_mut().kill();
        let _ = self.child.borrow_mut().wait();
    }
}

/// An `AddDoc` body whose `hit` holds `label` `a` and the members `hit`.
pub fn body(added: &str, ratio: &str, hit: &str) -> String {
    format!(r#"{{"added":{added},"ratio":{ratio},"hit":{{"label":"a",{hit}}}}}"#)
}

/// The text of each `"name":` member in `line`, up to its next `,`, `}` or `]`, in order.
pub fn tokens(line: &str, name: &str) -> Vec<String> {
    let key = format!("\"{name}\":");
    let mut found = Vec::new();
    let mut from = 0;
    while let Some(at) = line[from..].find(&key) {
        let rest = &line[from + at + key.len()..];
        let end = rest.find([',', '}', ']']).unwrap_or(rest.len());
        found.push(rest[..end].to_owned());
        from += at + key.len();
    }
    found
}

/// Every score token is written back as the shortest binary64 token, in every position.
pub fn writes_every_token(language: &str, binary: &Path) {
    let ir = model();
    let served = Served::start(binary, &ir);
    for (sent, written) in WRITTEN {
        served.reset();
        let (status, answer) = served.add(&body(
            "1.5",
            "0.5",
            &format!(r#""score":{sent},"many":[],"by_name":{{}}"#),
        ));
        assert_eq!(
            answer["outcome"], "added",
            "{language}: sending {sent}: {status} {answer}"
        );
        let line = served.docs_line();
        assert_eq!(
            tokens(&line, "score"),
            vec![written.to_owned()],
            "{language}: sending {sent}: {line}"
        );
    }
    served.reset();
    let (status, answer) = served.add(&body(
        "1.5",
        "0.5",
        r#""score":1.5,"maybe":1e16,"many":[-0,2.5e-7],"by_name":{"k":3}"#,
    ));
    assert_eq!(
        answer["outcome"], "added",
        "{language}: optional, list and map: {status} {answer}"
    );
    let line = served.docs_line();
    for expected in [
        r#""maybe":1e16"#,
        r#""many":[-0.0,2.5e-7]"#,
        r#""by_name":{"k":3.0}"#,
    ] {
        assert!(
            line.contains(expected),
            "{language}: `{expected}` is missing from {line}"
        );
    }
}

/// A token no finite binary64 holds is refused quickly with `400`, and nothing is stored.
pub fn refuses_what_no_binary64_holds(language: &str, binary: &Path) {
    let ir = model();
    let served = Served::start(binary, &ir);
    for (sent, found) in REFUSED {
        served.reset();
        let request = body(
            "1.5",
            "0.5",
            &format!(r#""score":{sent},"many":[],"by_name":{{}}"#),
        );
        let started = Instant::now();
        let (status, answer) = served.add(&request);
        let took = started.elapsed();
        assert_eq!(status, 400, "{language}: sending {sent}: {answer}");
        assert!(
            answer.to_string().contains(&format!(
                "hit.score: expected a finite Binary64 number, found {found}"
            )),
            "{language}: sending {sent}: {answer}"
        );
        assert!(
            took < Duration::from_secs(2),
            "{language}: sending {sent} took {took:?}"
        );
        let line = served.docs_line();
        assert!(
            tokens(&line, "score").is_empty(),
            "{language}: sending {sent} stored a row: {line}"
        );
    }
    served.reset();
    let (status, answer) = served.add(&body(
        "1.5",
        "1.0000000000000002",
        r#""score":1.5,"many":[],"by_name":{}"#,
    ));
    assert_eq!(status, 400, "{language}: a ratio past 1: {answer}");
    assert!(
        answer.to_string().contains("ratio"),
        "{language}: a ratio past 1: {answer}"
    );
}

/// The filter and the order compare by value, and the guard splits at the binary64 below 0.5.
pub fn compares_by_value(language: &str, binary: &Path) {
    let ir = model();
    let served = Served::start(binary, &ir);
    served.reset();
    for added in ["2.5", "10.25", "0.75", "0.7500000000000001", "1e16"] {
        let (status, answer) =
            served.add(&body(added, "0.5", r#""score":1.5,"many":[],"by_name":{}"#));
        assert_eq!(
            answer["outcome"], "added",
            "{language}: adding {added}: {status} {answer}"
        );
    }
    let line = served.docs_line();
    assert_eq!(
        tokens(&line, "added"),
        ["1e16", "10.25", "2.5", "0.7500000000000001"],
        "{language}: {line}"
    );
    served.reset();
    let (status, answer) = served.add(&body("0.5", "0.5", r#""score":1.5,"many":[],"by_name":{}"#));
    assert_eq!(
        answer["outcome"], "added",
        "{language}: adding 0.5: {status} {answer}"
    );
    let request = json!({
        "op": "command",
        "command": ADD,
        "actor": CLERK,
        "body": body(
            "0.49999999999999994",
            "0.5",
            r#""score":1.5,"many":[],"by_name":{}"#,
        ),
    });
    let line = served.raw(&request);
    let answer: Value =
        serde_json::from_str(&line).unwrap_or_else(|error| panic!("`{line}`: {error}"));
    assert_eq!(
        answer["body"]["outcome"], "rejected",
        "{language}: the binary64 below 0.5 is rejected: {line}"
    );
    assert!(
        line.contains(r#""added":0.49999999999999994"#),
        "{language}: the refusal carries its token: {line}"
    );
}

fn failure(observation: &str, answer: &Value) -> TargetError {
    TargetError::unavailable(observation, answer.to_string())
}

/// A wire value as a suite value.
fn node_of(value: &Value) -> Node {
    serde_json::from_value(value.clone()).expect("the wire writes JSON a node reads")
}

impl ConformanceTarget for Served {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new(
            "binary64-served",
            env!("CARGO_PKG_VERSION"),
        ))
    }

    fn begin_scenario(&self, _scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.reset();
        Ok(())
    }

    /// The input exactly as the suite carries it: `serde_json::to_value`, no number rewritten.
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let observation = format!("invoking `{}`", request.command);
        let body = serde_json::to_value(&request.input).expect("a suite input is JSON");
        let (status, answer) = self.ask(&json!({
            "op": "command",
            "command": request.command.to_string(),
            "actor": request.actor.as_ref().map(ToString::to_string),
            "body": body.to_string(),
        }));
        if status == 403 && answer["refused"] == "not granted" {
            return Err(TargetError::not_granted(answer["actor"].as_str()));
        }
        if status == 501 {
            return Ok(SemanticCommandResult::undeclared());
        }
        let Some(outcome) = answer["outcome"].as_str() else {
            return Err(failure(&observation, &answer));
        };
        let outcome = OutcomeRef::new(
            CommandRef::new(request.command.name().clone()),
            outcome
                .parse()
                .map_err(|_| failure(&observation, &answer))?,
        );
        let error = match answer.get("refusal") {
            Some(refusal) => {
                let name: ErrorRef = refusal["error"]
                    .as_str()
                    .and_then(|name| name.parse().ok())
                    .ok_or_else(|| failure(&observation, &answer))?;
                let mut declared = DeclaredErrorValue::new(name);
                if let Some(Value::Object(payload)) = refusal.get("payload") {
                    for (field, value) in payload {
                        if !value.is_null() {
                            declared = declared.with(field.clone(), node_of(value));
                        }
                    }
                }
                Some(declared)
            }
            None => None,
        };
        let mut direct_events = Vec::new();
        for published in answer["published"].as_array().into_iter().flatten() {
            let reference: EventRef = published["event"]
                .as_str()
                .and_then(|name| name.parse().ok())
                .ok_or_else(|| failure(&observation, &answer))?;
            let mut event = ObservedEvent::new(reference);
            if let Some(Value::Object(payload)) = published.get("payload") {
                for (field, value) in payload {
                    event = event.with(field.clone(), node_of(value));
                }
            }
            let event = event
                .in_activity(request.correlation.clone())
                .at(self.tick());
            self.published.borrow_mut().push(event.clone());
            direct_events.push(event);
        }
        let consistency = Some(
            ConsistencyToken::new(format!("seq:{}", self.tick()))
                .map_err(|why| TargetError::unavailable(observation.clone(), why.to_string()))?,
        );
        Ok(SemanticCommandResult {
            outcome: Some(outcome),
            error,
            consistency,
            direct_events,
            response: None,
        })
    }

    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let observation = format!("reading `{}`", request.view);
        let (status, answer) = self.ask(&json!({
            "op": "view",
            "view": request.view.to_string(),
            "query": "",
        }));
        let Some(rows) = answer["rows"].as_array().filter(|_| status == 200) else {
            return Err(failure(&observation, &answer));
        };
        Ok(SemanticViewResult::of(rows.iter().map(|row| {
            let read: ViewRow = row
                .as_object()
                .into_iter()
                .flatten()
                .map(|(field, value)| (field.clone(), node_of(value)))
                .collect();
            read
        })))
    }

    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Ok(self
            .published
            .borrow()
            .iter()
            .filter(|event| event.event == request.event)
            .cloned()
            .collect())
    }

    fn configure_external_outcome(
        &self,
        request: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        Err(TargetError::unsupported(
            format!("forcing `{}`", request.force),
            "the model declares no external branch",
        ))
    }

    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        Err(TargetError::unsupported(
            format!("delivering `{}` again", request.event),
            "the model declares no binding",
        ))
    }

    fn end_scenario(&self, _scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.published.borrow_mut().clear();
        Ok(())
    }
}

/// The synthesized suite passes against the served model, scenario by scenario.
pub fn passes_the_synthesized_suite(language: &str, binary: &Path) {
    let ir = model();
    let suite = ess_conformance::synthesize(&ir).suite;
    let admitted = AdmittedSuite::from_suite(&suite).unwrap_or_else(|error| panic!("{error}"));
    let report = Runner::for_suite(&suite)
        .run_admitted(&admitted, &Served::start(binary, &ir))
        .into_report();
    let mut verdicts: Vec<(String, Status)> = report
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result.status))
        .collect();
    verdicts.sort_by(|left, right| left.0.cmp(&right.0));
    let ids: Vec<&str> = verdicts.iter().map(|(id, _)| id.as_str()).collect();
    assert_eq!(
        ids,
        [
            "demo.search.AddDoc/outcome/added",
            "demo.search.AddDoc/outcome/rejected"
        ],
        "{language}: {verdicts:#?}"
    );
    assert!(
        verdicts.iter().all(|(_, status)| *status == Status::Passed),
        "{language}: {verdicts:#?}"
    );
}
