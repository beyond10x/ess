//! A whole number reaches an `Integer` through the generated decoders however it is spelt.
//!
//! The published contract for `Integer` is JSON Schema's `type: integer`, which counts any number
//! with a zero fraction as an integer, and the suite ESS synthesizes writes an integer literal as
//! the binary64 it holds: `3.0`. The generated Rust `integer_at` and Go `integerAt` read the number
//! text as an `i64` and refused `3.0`, so an implementation built on them could not take the suite
//! as written. They now read every number whose exact decimal value is a whole number in `i64`
//! range — `3`, `3.0`, `3e0`, `30e-1`, `-0.0` — and still refuse a fraction, a magnitude past 64
//! bits and a hostile exponent, quickly and with the refusal they always gave.
//!
//! The model is one command with `limit: Integer` and `page: Optional<Integer>`, a view over the
//! record it creates, and a `reached_by: network` component. Each generated server is built
//! offline beside a harness that holds only storage and identities, and is reached through the
//! route a request takes: the in-process dispatcher for Rust, a real socket for Go. The Go cases
//! are skipped, and said out loud, where this machine has no `go`.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::io::{BufRead as _, BufReader, Write as _};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Output, Stdio};
use std::time::{Duration, Instant};

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile_locating;
use ess_compiler::source::SourceMap;
use ess_conformance::report::Status;
use ess_conformance::scenario::{CommandRef, EventRef, OutcomeRef};
use ess_conformance::target::{
    ConformanceTarget, EventObservationRequest, ExternalOutcomeControl, ImplementationIdentity,
    ObservedEvent, RedeliveryRequest, ScenarioContext, SemanticCommandRequest,
    SemanticCommandResult, SemanticViewRequest, SemanticViewResult, TargetError, ViewRow,
};
use ess_conformance::{AdmittedSuite, ConformanceSuite, Runner};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::consistency::ConsistencyToken;
use ess_primitives::node::Node;
use ess_synth::{synthesize_for, Target};
use serde_json::{json, Value};

const MODEL: &str = "\
format: ess/25
system: demo
version: v1
domain: demo.board
summary: A board holds a label, a size limit and the page it opens on.
types:
  - {name: demo.board.BoardId, kind: newtype, of: Uuid}
entities:
  - name: demo.board.Board
    identity: {name: board_id, type: demo.board.BoardId}
    fields:
      - {name: label, type: String}
      - {name: limit, type: Integer}
      - {name: page, type: Optional<Integer>}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
events:
  - name: demo.board.BoardOpened
    fields: [{name: board_id, type: demo.board.BoardId}]
actors:
  - name: demo.board.Operator
    may: [demo.board.Open]
commands:
  - name: demo.board.Open
    input:
      - {name: label, type: String}
      - {name: limit, type: Integer}
      - {name: page, type: Optional<Integer>}
    outcomes:
      - name: opened
        creates: demo.board.Board
        instance: board_id
        emits: [demo.board.BoardOpened]
        payload: {demo.board.BoardOpened: {board_id: {generated: true}}}
        sets: {label: input.label, limit: input.limit, page: input.page}
views:
  - name: demo.board.Boards
    source: demo.board.Board
    consistency: read_your_writes
    fields:
      - {name: board_id, type: demo.board.BoardId}
      - {name: label, type: String}
      - {name: limit, type: Integer}
      - {name: page, type: Optional<Integer>}
components:
  - component: board-service
    summary: Opens boards and lists them.
    owns: {domains: [demo.board]}
    accepts: {commands: [demo.board.Open]}
    publishes: {events: [demo.board.BoardOpened]}
    reached_by: network
";

/// An authored scenario whose step states its integers as an author writes them.
const AUTHORED: &str = "\
type: ess-scenario/1
domain: demo.board
scenario: open-board
summary: Opening a board lists it with its limit and its page.
arrange:
  - instance: board
    entity: demo.board.Board
timeline:
  - at: 2026-01-05T09:00:00Z
    command: demo.board.Open
    actor: demo.board.Operator
    input: {label: morning, limit: 3, page: 1}
    outcome: opened
    capture: {instance: board, event: demo.board.BoardOpened, field: board_id}
assert:
  - view: demo.board.Boards
    contains: {board_id: {$instance: board}, limit: 3, page: 1}
";

const OPEN: &str = "demo.board.Open";
const BOARDS: &str = "demo.board.Boards";
const OPERATOR: &str = "demo.board.Operator";

fn model() -> EssIr {
    let raw = RawSpecFile::parse(MODEL).unwrap_or_else(|error| panic!("well formed: {error}"));
    let specification = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("validates:\n{errors}"));
    let mut sources = SourceMap::new();
    sources.insert("model.yaml".to_owned(), MODEL.to_owned());
    compile_locating(&specification, &sources, &["model.yaml".to_owned()])
        .unwrap_or_else(|diagnostics| panic!("resolves:\n{diagnostics}"))
}

/// Every route of the model's component, as `name method path` triples.
fn routes(ir: &EssIr) -> Vec<String> {
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

fn shown(output: &Output) -> String {
    format!(
        "exit {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn write_tree(root: &Path, synthesis: &ess_synth::Synthesis) {
    for artifact in synthesis.artifacts.values() {
        let path = root.join(&artifact.path);
        std::fs::create_dir_all(path.parent().expect("a file has a parent")).expect("mkdir");
        std::fs::write(&path, &artifact.contents).expect("write");
    }
}

fn has_go() -> bool {
    let found = Command::new("go")
        .arg("version")
        .output()
        .is_ok_and(|output| output.status.success());
    if !found {
        println!("skipped: no `go` on PATH, so the generated Go server was not run");
    }
    found
}

fn go(directory: &Path, arguments: &[&str]) -> Output {
    Command::new("go")
        .args(arguments)
        .current_dir(directory)
        .env("GOFLAGS", "-mod=mod")
        .env("GOPROXY", "off")
        .env("GOWORK", "off")
        .output()
        .expect("go runs")
}

// ---- the generated servers, built once --------------------------------------------------------

/// The harness built against the generated Rust server, and against the generated Go server where
/// this machine has Go.
struct Built {
    rust: PathBuf,
    go: Option<PathBuf>,
}

fn built() -> &'static Built {
    static BUILT: std::sync::OnceLock<Built> = std::sync::OnceLock::new();
    BUILT.get_or_init(|| {
        let ir = model();
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("integer-spellings-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let kept = root.join("kept");
        std::fs::create_dir_all(&kept).expect("mkdir");

        let synthesis = synthesize_for(&ir, Target::Rust).expect("the model synthesizes to Rust");
        write_tree(&root.join("demo"), &synthesis);
        let harness = root.join("harness");
        std::fs::create_dir_all(harness.join("src")).expect("mkdir");
        std::fs::write(
            harness.join("Cargo.toml"),
            "[package]\nname = \"integer-spellings-harness\"\nversion = \"0.0.0\"\nedition = \
             \"2021\"\npublish = false\n\n[workspace]\n\n[[bin]]\nname = \"harness\"\npath = \
             \"src/main.rs\"\n\n[dependencies]\ndemo-types = { path = \
             \"../demo/crates/demo-types\" }\ndemo-server = { path = \
             \"../demo/crates/demo-server\" }\nboard-service = { path = \
             \"../demo/crates/board-service\" }\ndemo-system = { path = \
             \"../demo/crates/demo-system\" }\n",
        )
        .expect("write");
        std::fs::copy(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/integer-spellings-harness/main.rs"),
            harness.join("src/main.rs"),
        )
        .expect("the harness copies");
        let target = root.join("target");
        let output =
            Command::new(std::env::var_os("CARGO").expect("Cargo supplies its executable"))
                .args(["build", "--offline", "--quiet", "--target-dir"])
                .arg(&target)
                .current_dir(&harness)
                .env_remove("CARGO_TARGET_DIR")
                .env_remove("CARGO_ENCODED_RUSTFLAGS")
                .env_remove("RUSTC_WRAPPER")
                .env("CARGO_INCREMENTAL", "0")
                .env("RUSTFLAGS", "-D warnings")
                .output()
                .expect("cargo runs");
        assert!(
            output.status.success(),
            "the Rust harness builds: {}",
            shown(&output)
        );
        std::fs::copy(target.join("debug/harness"), kept.join("rust")).expect("keep");
        // The generated service is in the binary; its build tree is not needed again.
        let _ = std::fs::remove_dir_all(&target);

        let go_binary = has_go().then(|| {
            let synthesis = synthesize_for(&ir, Target::Go).expect("the model synthesizes to Go");
            let tree = root.join("go-demo");
            write_tree(&tree, &synthesis);
            let harness = root.join("go-harness");
            std::fs::create_dir_all(&harness).expect("mkdir");
            std::fs::write(
                harness.join("go.mod"),
                "module integerspellingsharness\n\ngo 1.21\n\nrequire example.invalid/demo \
                 v0.0.0\n\nreplace example.invalid/demo => ../go-demo\n",
            )
            .expect("write");
            std::fs::copy(
                Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("tests/fixtures/integer-spellings-go-harness/main.go"),
                harness.join("main.go"),
            )
            .expect("the harness copies");
            let binary = kept.join("go");
            let output = go(
                &harness,
                &["build", "-o", binary.to_str().expect("a UTF-8 path"), "."],
            );
            assert!(
                output.status.success(),
                "the Go harness builds: {}",
                shown(&output)
            );
            binary
        });
        Built {
            rust: kept.join("rust"),
            go: go_binary,
        }
    })
}

// ---- one harness process ----------------------------------------------------------------------

/// One running harness, and the events one scenario published.
struct Served {
    child: RefCell<Child>,
    stdin: RefCell<ChildStdin>,
    stdout: RefCell<BufReader<ChildStdout>>,
    published: RefCell<Vec<ObservedEvent>>,
    sequence: RefCell<u64>,
}

impl Served {
    fn start(binary: &Path, ir: &EssIr) -> Self {
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

    /// One request line out, one `{status, body}` line back.
    fn ask(&self, request: &Value) -> (u64, Value) {
        let mut stdin = self.stdin.borrow_mut();
        writeln!(stdin, "{request}").expect("the harness reads");
        stdin.flush().expect("the harness reads");
        let mut line = String::new();
        self.stdout
            .borrow_mut()
            .read_line(&mut line)
            .expect("the harness answers");
        let answer: Value =
            serde_json::from_str(&line).unwrap_or_else(|error| panic!("`{line}`: {error}"));
        (
            answer["status"].as_u64().unwrap_or_default(),
            answer["body"].clone(),
        )
    }

    fn reset(&self) {
        self.ask(&json!({"op": "reset"}));
        self.published.borrow_mut().clear();
        *self.sequence.borrow_mut() = 0;
    }

    /// Sends `body`, as written, to the `Open` route.
    fn open(&self, body: &str) -> (u64, Value) {
        self.ask(&json!({"op": "command", "command": OPEN, "actor": OPERATOR, "body": body}))
    }

    /// The rows of the `Boards` view.
    fn boards(&self) -> Vec<Value> {
        let (status, answer) = self.ask(&json!({"op": "view", "view": BOARDS, "query": ""}));
        assert_eq!(status, 200, "{answer}");
        answer["rows"].as_array().cloned().unwrap_or_default()
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

// ---- the decoder cases ------------------------------------------------------------------------

/// Each whole number, however spelt, is read as the integer it is, and the view lists it so.
fn reads_whole_numbers(language: &str, binary: &Path) {
    let served = Served::start(binary, &model());
    for (spelling, integer) in [
        ("3.0", 3),
        ("3.00", 3),
        ("3e0", 3),
        ("30e-1", 3),
        ("0.3e1", 3),
        ("-0.0", 0),
    ] {
        served.reset();
        let (status, answer) = served.open(&format!(r#"{{"label":"a","limit":{spelling}}}"#));
        assert_eq!(
            answer["outcome"], "opened",
            "{language}: `limit: {spelling}` is accepted: {status} {answer}"
        );
        let rows = served.boards();
        assert_eq!(rows.len(), 1, "{language}: {rows:?}");
        assert_eq!(
            rows[0]["limit"],
            json!(integer),
            "{language}: `limit: {spelling}` reads back as {integer}"
        );
    }
    served.reset();
    let (status, answer) = served.open(r#"{"label":"a","limit":3,"page":1.0}"#);
    assert_eq!(
        answer["outcome"], "opened",
        "{language}: `page: 1.0` is accepted: {status} {answer}"
    );
    assert_eq!(served.boards()[0]["page"], json!(1), "{language}");
}

/// A fraction, a magnitude past 64 bits and a hostile exponent are still refused, as before.
fn refuses_what_no_integer_holds(language: &str, binary: &Path) {
    let served = Served::start(binary, &model());
    for spelling in [
        "1.5",
        "3.000000000000000001",
        "9223372036854775808.0",
        "-9223372036854775809.0",
        "1e999999999",
        "1e-999999999",
    ] {
        served.reset();
        let started = Instant::now();
        let (status, answer) = served.open(&format!(r#"{{"label":"a","limit":{spelling}}}"#));
        let took = started.elapsed();
        assert_eq!(status, 400, "{language}: `limit: {spelling}`: {answer}");
        let expected = format!("expected an integer, found the number {spelling}");
        assert!(
            answer.to_string().contains(&expected),
            "{language}: `limit: {spelling}` is refused with `{expected}`: {answer}"
        );
        assert!(
            took < Duration::from_secs(2),
            "{language}: `limit: {spelling}` took {took:?}"
        );
        assert!(served.boards().is_empty(), "{language}: nothing was opened");
    }
}

#[test]
fn the_rust_server_reads_an_integral_number_with_a_zero_fraction_as_the_integer() {
    reads_whole_numbers("rust", &built().rust);
}

#[test]
fn the_rust_server_still_refuses_a_fraction_an_out_of_range_integer_and_a_hostile_exponent() {
    refuses_what_no_integer_holds("rust", &built().rust);
}

#[test]
fn the_go_server_reads_an_integral_number_with_a_zero_fraction_as_the_integer() {
    let Some(binary) = &built().go else { return };
    reads_whole_numbers("go", binary);
}

#[test]
fn the_go_server_still_refuses_a_fraction_an_out_of_range_integer_and_a_hostile_exponent() {
    let Some(binary) = &built().go else { return };
    refuses_what_no_integer_holds("go", binary);
}

// ---- the suite, forwarded as written ----------------------------------------------------------

/// The synthesized suite with the authored scenario in it, as `ess verify conform synthesize`
/// assembles it.
fn suite(ir: &EssIr) -> ConformanceSuite {
    let mut suite = ess_conformance::synthesize(ir).suite;
    let authoring = ess_conformance::authored::compile(
        ir,
        &[ess_conformance::authored::Source::new(
            "open-board.yaml",
            AUTHORED,
        )],
    );
    let refusals: Vec<String> = authoring.refusals.iter().map(ToString::to_string).collect();
    assert!(refusals.is_empty(), "{refusals:#?}");
    assert_eq!(authoring.scenarios.len(), 1);
    for (id, scenario) in authoring.scenarios {
        suite.insert(id, scenario).expect("one id");
    }
    suite.select_fresh_format_for(ir);
    suite
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
            "integer-spellings",
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
            error: None,
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
            let mut read: ViewRow = row
                .as_object()
                .into_iter()
                .flatten()
                .map(|(field, value)| (field.clone(), node_of(value)))
                .collect();
            read.entry("page".to_owned()).or_insert(Node::Null);
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

fn verdicts(ir: &EssIr, suite: &ConformanceSuite, binary: &Path) -> BTreeMap<String, Status> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    let target = Served::start(binary, ir);
    let report = Runner::for_suite(suite)
        .run_admitted(&admitted, &target)
        .into_report();
    assert_eq!(report.scenarios.len(), suite.len());
    report
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result.status))
        .collect()
}

#[test]
fn the_synthesized_suite_passes_against_each_generated_server_with_inputs_forwarded_as_written() {
    let ir = model();
    let suite = suite(&ir);
    let built = built();
    let mut servers = vec![("rust", built.rust.clone())];
    servers.extend(built.go.clone().map(|binary| ("go", binary)));
    for (server, binary) in servers {
        let verdicts = verdicts(&ir, &suite, &binary);
        assert!(
            verdicts.contains_key("demo.board/authored/open-board"),
            "{server}: {verdicts:#?}"
        );
        assert!(
            verdicts.contains_key("demo.board.Open/outcome/opened"),
            "{server}: {verdicts:#?}"
        );
        let not_passed: Vec<String> = verdicts
            .iter()
            .filter(|(_, status)| **status != Status::Passed)
            .map(|(id, status)| format!("{id}: {status:?}"))
            .collect();
        assert!(
            not_passed.is_empty(),
            "{server}: every scenario passes: {not_passed:#?}"
        );
        eprintln!("{server}: {} scenarios passed", verdicts.len());
    }
}

/// The suite's spelling of an integer is frozen (`docs/design/review-conformance-coverage.md`):
/// the decoders moved, the suite did not.
#[test]
fn the_suite_keeps_the_frozen_spelling_of_an_integer_literal() {
    let ir = model();
    let suite = serde_json::to_value(suite(&ir)).expect("a suite is JSON");
    let scenarios = &suite["scenarios"];
    let authored =
        serde_json::to_string_pretty(&scenarios["demo.board/authored/open-board"]).expect("JSON");
    assert!(
        authored.contains("\"value\": 3.0") && authored.contains("\"value\": 1.0"),
        "the authored `limit: 3, page: 1` is written as 3.0 and 1.0:\n{authored}"
    );
    assert!(!authored.contains("\"value\": 3\n"), "{authored}");
    let witness =
        serde_json::to_string_pretty(&scenarios["demo.board.Open/outcome/opened"]).expect("JSON");
    assert!(
        witness.contains("\"value\": 1.0"),
        "the synthesized witness is written as 1.0:\n{witness}"
    );
}

// ---- a number nobody parsed ----------------------------------------------------------------------

/// The committed Rust JSON module, which `ess generate synthesize` writes from this crate.
#[allow(dead_code, missing_docs, clippy::all, clippy::pedantic)]
#[path = "../../../../generated/rust/gatepass/crates/gatepass-server/src/json.rs"]
mod json;

/// Spellings outside the JSON number grammar: a byte that is not a digit where a digit belongs, or a
/// digit group left empty. The parser never yields them, but a number can be built by hand.
const OUTSIDE_THE_GRAMMAR: [&str; 12] = [
    "1.:e1", "2.;e1", "1:", "1e:", "1e/", "1.", "1e", "1e+", "", "-", ".", "e5",
];

#[test]
fn the_rust_reader_refuses_a_number_built_by_hand_outside_the_grammar() {
    for spelling in OUTSIDE_THE_GRAMMAR {
        let value = json::Value::Number(spelling.to_owned());
        let answer = json::integer_at(&value, "limit", "an integer");
        assert!(answer.is_err(), "{spelling:?}: {answer:?}");
    }
}

#[test]
fn the_go_reader_refuses_a_number_built_by_hand_outside_the_grammar() {
    if !has_go() {
        return;
    }
    let wire = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../generated/go/gatepass/server/wire.go"),
    )
    .expect("the committed Go wire");
    let cut = |from: &str, to: &str| {
        let start = wire
            .find(from)
            .unwrap_or_else(|| panic!("`{from}` in wire.go"));
        let end = start
            + wire[start..]
                .find(to)
                .unwrap_or_else(|| panic!("`{to}` in wire.go"));
        wire[start..end].to_owned()
    };
    let mut program = String::from(
        "package main\n\nimport (\n\t\"encoding/json\"\n\t\"fmt\"\n\t\"strconv\"\n\t\"strings\"\n)\n\n\
         var _ = strconv.Itoa\nvar _ = strings.TrimSpace\n\n",
    );
    program.push_str(&cut("type DecodeError struct", "// nested is one step"));
    program.push_str(&cut("func integerAt(", "// decimalAt is"));
    program.push_str("func main() {\n\tfor _, spelling := range []string{");
    for spelling in OUTSIDE_THE_GRAMMAR {
        write!(program, "{spelling:?}, ").expect("a string takes text");
    }
    program.push_str(
        "} {\n\t\tif held, err := integerAt(json.Number(spelling), \"limit\", \"an integer\"); \
         err == nil {\n\t\t\tfmt.Printf(\"%q: %d\\n\", spelling, held)\n\t\t}\n\t}\n}\n",
    );
    let directory = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("integer-spellings-grammar-{}", std::process::id()));
    std::fs::create_dir_all(&directory).expect("mkdir");
    std::fs::write(directory.join("main.go"), program).expect("write main.go");
    let output = go(&directory, &["run", "main.go"]);
    let _ = std::fs::remove_dir_all(&directory);
    assert!(output.status.success(), "{}", shown(&output));
    let accepted = String::from_utf8_lossy(&output.stdout);
    assert!(accepted.is_empty(), "accepted:\n{accepted}");
}
