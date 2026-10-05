//! Typed text operands in the generated applications (beyond10x/ess#200, final review decision 3).
//!
//! `crates/verify/ess-conformance/tests/fixtures/typed-text-operands.yaml` declares every string
//! operator against a view parameter, `{param: <name>}`, and against a command input,
//! `{input: <name>}`, in a plain `when:`, a `when_subject:` and a `when_related:` predicate. Its
//! views' queries and its commands' behaviour are generated, not owed, for Rust and Go: the
//! generated route decodes the query string through the view-query decoder and the generated
//! query applies the parameter. Each generated system is built — warnings denied, Go gofmt-clean
//! and vet clean — beside a harness that holds only storage and identities, and passes the suite
//! the specification synthesizes, through the route a request reaches: the in-process dispatcher
//! for Rust, a real socket for Go. Each emitted seam is then patched, one fault at a time — the
//! operand ignored, its spelling compared, the operator applied the wrong way round — and the same
//! suite must fail.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

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
use ess_synth::{synthesize_for, CapabilityKind, SynthesisDisposition, Target};

const MODEL: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/typed-text-operands.yaml");

fn compile_text(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("well formed: {error}"));
    let specification = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("validates:\n{errors}"));
    let mut sources = SourceMap::new();
    sources.insert("model.yaml".to_owned(), text.to_owned());
    compile_locating(&specification, &sources, &["model.yaml".to_owned()])
        .unwrap_or_else(|diagnostics| panic!("resolves:\n{diagnostics}"))
}

const VIEWS: [&str; 4] = [
    "directory.people.Contacts",
    "directory.people.ByName",
    "directory.people.ByPhone",
    "directory.people.ByNote",
];

const COMMANDS: [&str; 4] = [
    "directory.people.AddContact",
    "directory.people.Screen",
    "directory.people.Match",
    "directory.people.Dial",
];

#[test]
fn t200_every_query_and_behaviour_is_generated_for_rust_and_go() {
    let ir = compile_text(MODEL);
    for target in [Target::Rust, Target::Go] {
        let synthesis = synthesize_for(&ir, target).expect("the model synthesizes");
        for view in VIEWS {
            assert_eq!(
                synthesis
                    .plan
                    .disposition_of(CapabilityKind::ViewQuery, view),
                Some(&SynthesisDisposition::Generated),
                "{target:?}: `{view}` is applied by its generated query"
            );
        }
        for command in COMMANDS {
            assert_eq!(
                synthesis
                    .plan
                    .disposition_of(CapabilityKind::CommandBehavior, command),
                Some(&SynthesisDisposition::Generated),
                "{target:?}: `{command}` is decided by generated behaviour"
            );
        }
    }
    let rust = synthesize_for(&ir, Target::Rust).expect("Rust");
    let behaviour = &rust.artifacts["crates/directory-types/src/behaviour.rs"].contents;
    // The query takes the parameters the route decodes, at their declared types, bound by
    // position so that no parameter name shadows what the body reads.
    assert!(
        behaviour.contains("fn by_name(&self, param_0: String)")
            && behaviour.contains("fn by_phone(&self, param_0: Option<crate::people::Phone>)"),
        "the generated query takes the parameters the route decodes:\n{behaviour}"
    );
}

#[test]
fn t200_a_parameter_read_another_way_keeps_the_query_owed() {
    let owed = MODEL.replace(
        "filter: {name: {starts_with: {param: q}}}",
        "filter: {all: [{name: {starts_with: {param: q}}}, name == param.q]}",
    );
    assert_ne!(owed, MODEL);
    let ir = compile_text(&owed);
    for target in [Target::Rust, Target::Go] {
        let synthesis = synthesize_for(&ir, target).expect("the model synthesizes");
        assert!(
            matches!(
                synthesis
                    .plan
                    .disposition_of(CapabilityKind::ViewQuery, "directory.people.ByName"),
                Some(SynthesisDisposition::Obligation { .. })
            ),
            "{target:?}: an equality with a parameter is not applied by a generated query"
        );
        assert_eq!(
            synthesis
                .plan
                .disposition_of(CapabilityKind::ViewQuery, "directory.people.ByNote"),
            Some(&SynthesisDisposition::Generated),
            "{target:?}"
        );
    }
}

// ---- building and driving one generated system ----------------------------------------------

/// A directory under the build's temporary directory, removed when dropped.
struct Scratch(PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn write_tree(root: &Path, synthesis: &ess_synth::Synthesis) {
    for artifact in synthesis.artifacts.values() {
        let path = root.join(&artifact.path);
        std::fs::create_dir_all(path.parent().expect("a file has a parent")).expect("mkdir");
        std::fs::write(&path, &artifact.contents).expect("write");
    }
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

/// Runs cargo offline in `directory`, warnings denied where `strict`.
fn cargo(directory: &Path, target: &Path, arguments: &[&str], strict: bool) -> (bool, String) {
    let output = Command::new(std::env::var_os("CARGO").expect("Cargo supplies its executable"))
        .args(arguments)
        .arg("--offline")
        .current_dir(directory)
        .env("CARGO_TARGET_DIR", target)
        .env("RUSTFLAGS", if strict { "-D warnings" } else { "" })
        .env("CARGO_INCREMENTAL", "0")
        .env_remove("RUSTC_WRAPPER")
        .output()
        .expect("cargo runs");
    (
        output.status.success(),
        format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ),
    )
}

/// Where Go is, or `None` when this machine has none — said out loud, never passed silently.
fn go() -> Option<String> {
    let output = Command::new("go").arg("version").output().ok()?;
    output.status.success().then(|| "go".to_owned())
}

fn go_tool(directory: &Path, tool: &str, arguments: &[&str]) -> (bool, String) {
    let output = Command::new(tool)
        .args(arguments)
        .current_dir(directory)
        .env("GOFLAGS", "-mod=mod")
        .env("GOPROXY", "off")
        .env("GOWORK", "off")
        .output()
        .expect("the Go tool runs");
    (
        output.status.success(),
        format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ),
    )
}

/// Every scenario of the suite the specification synthesizes against one harness process, by
/// verdict, with the count of scenarios the suite holds.
fn verdicts(ir: &EssIr, binary: &Path, routes: &[String]) -> BTreeMap<String, Status> {
    let suite = ess_conformance::synthesize(ir).suite;
    let admitted = AdmittedSuite::from_suite(&suite).unwrap_or_else(|error| panic!("{error}"));
    let target = Served::start(binary, routes, ir);
    let report = Runner::for_suite(&suite)
        .run_admitted(&admitted, &target)
        .into_report();
    assert_eq!(report.scenarios.len(), suite.scenarios.len());
    report
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result.status))
        .collect()
}

fn not_passed(verdicts: &BTreeMap<String, Status>) -> Vec<String> {
    verdicts
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .map(|(id, status)| format!("{id}: {status:?}"))
        .collect()
}

/// What one patch of the emitted code does wrong, the spelling it replaces, its replacement, and
/// the scenario prefixes that must fail for it.
struct Fault {
    label: &'static str,
    from: &'static str,
    to: &'static str,
}

/// Asserts the healthy verdicts pass, then that each fault fails the scenarios that decide the
/// operand — a view read and a command each — by outcome or row.
fn caught(label: &str, healthy: &BTreeMap<String, Status>, faulty: &BTreeMap<String, Status>) {
    assert_eq!(not_passed(healthy), Vec::<String>::new());
    let failing = not_passed(faulty);
    for prefix in [
        "directory.people.AddContact/outcome/added",
        "directory.people.Screen/outcome/",
        "directory.people.Match/outcome/",
        "directory.people.Dial/outcome/",
    ] {
        assert!(
            failing.iter().any(|line| line.starts_with(prefix)),
            "{label} fails a scenario of {prefix}: {failing:#?}"
        );
    }
    assert!(
        failing.iter().all(|line| line.contains("Failed")),
        "{label} fails by outcome or row, not by setup: {failing:#?}"
    );
}

/// The Rust faults, as patches of the generated behaviour: every typed operand renders as
/// `value.<op>(operand.as_str())`.
const RUST_FAULTS: [Fault; 3] = [
    Fault {
        label: "rust-ignores-operand",
        from: ".map(|(value, operand)| value.",
        to: ".map(|(value, operand)| true || value.",
    },
    Fault {
        label: "rust-literal-spelling",
        from: "(operand.as_str())",
        to: "({ let _ = &operand; \"{param: q}\" })",
    },
    Fault {
        label: "rust-reversed",
        from: "|(value, operand)|",
        to: "|(operand, value)|",
    },
];

#[test]
fn t200_the_generated_rust_system_applies_every_operand_and_each_fault_fails() {
    let ir = compile_text(MODEL);
    let synthesis = synthesize_for(&ir, Target::Rust).expect("the model synthesizes");
    let scratch = Scratch(
        Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("typed-text-operands-rust-{}", std::process::id())),
    );
    let _ = std::fs::remove_dir_all(&scratch.0);
    let tree = scratch.0.join("directory");
    write_tree(&tree, &synthesis);
    let harness = scratch.0.join("harness");
    std::fs::create_dir_all(harness.join("src")).expect("mkdir");
    std::fs::write(
        harness.join("Cargo.toml"),
        "[package]\nname = \"typed-text-operands-harness\"\nversion = \"0.0.0\"\nedition = \
         \"2021\"\npublish = false\n\n[workspace]\n\n[[bin]]\nname = \"harness\"\npath = \
         \"src/main.rs\"\n\n[dependencies]\ndirectory-types = { path = \
         \"../directory/crates/directory-types\" }\ndirectory-server = { path = \
         \"../directory/crates/directory-server\" }\ndirectory-service = { path = \
         \"../directory/crates/directory-service\" }\ndirectory-system = { path = \
         \"../directory/crates/directory-system\" }\n",
    )
    .expect("write");
    std::fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/typed-text-operands-harness/main.rs"),
        harness.join("src/main.rs"),
    )
    .expect("the harness copies");
    let target = scratch.0.join("target");
    let (built, log) = cargo(
        &tree,
        &target,
        &["check", "--workspace", "--all-targets"],
        true,
    );
    assert!(
        built,
        "the generated workspace builds with -D warnings:\n{log}"
    );
    let (built, log) = cargo(&harness, &target, &["build"], true);
    assert!(
        built,
        "the harness builds against the generated ports:\n{log}"
    );
    let routes = routes(&ir);
    let binary = target.join("debug/harness");
    let healthy = verdicts(&ir, &binary, &routes);
    assert_eq!(not_passed(&healthy), Vec::<String>::new());
    eprintln!(
        "{} scenarios passed against the generated Rust system",
        healthy.len()
    );

    let behaviour = tree.join("crates/directory-types/src/behaviour.rs");
    let generated = std::fs::read_to_string(&behaviour).expect("the behaviour reads");
    for fault in &RUST_FAULTS {
        assert!(
            generated.matches(fault.from).count() >= 6,
            "{}: every typed operand of a view and a guard renders `{}`",
            fault.label,
            fault.from
        );
        std::fs::write(&behaviour, generated.replace(fault.from, fault.to)).expect("patch");
        let (built, log) = cargo(&harness, &target, &["build"], false);
        assert!(built, "{}: the patched system builds:\n{log}", fault.label);
        caught(fault.label, &healthy, &verdicts(&ir, &binary, &routes));
    }
    std::fs::write(&behaviour, &generated).expect("restore");
}

/// The Go faults, as patches of the one helper every typed operand is tested through.
const GO_FAULTS: [Fault; 3] = [
    Fault {
        label: "go-ignores-operand",
        from: "\treturn textMatch(value, op, *operand)\n",
        to: "\tif value == nil {\n\t\treturn unknown\n\t}\n\treturn known(true)\n",
    },
    Fault {
        label: "go-literal-spelling",
        from: "\treturn textMatch(value, op, *operand)\n",
        to: "\treturn textMatch(value, op, \"{param: q}\")\n",
    },
    Fault {
        label: "go-reversed",
        from: "\treturn textMatch(value, op, *operand)\n",
        to: "\tif value == nil {\n\t\treturn unknown\n\t}\n\treturn textMatch(operand, op, *value)\n",
    },
];

#[test]
fn t200_the_generated_go_system_applies_every_operand_and_each_fault_fails() {
    let Some(go) = go() else {
        eprintln!("no Go toolchain on this machine; the generated Go system is unchecked here");
        return;
    };
    let ir = compile_text(MODEL);
    let synthesis = synthesize_for(&ir, Target::Go).expect("the model synthesizes to Go");
    let scratch = Scratch(
        Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("typed-text-operands-go-{}", std::process::id())),
    );
    let _ = std::fs::remove_dir_all(&scratch.0);
    let tree = scratch.0.join("directory");
    write_tree(&tree, &synthesis);
    let harness = scratch.0.join("harness");
    std::fs::create_dir_all(&harness).expect("mkdir");
    std::fs::write(
        harness.join("go.mod"),
        "module directoryharness\n\ngo 1.21\n\nrequire example.invalid/directory v0.0.0\n\nreplace \
         example.invalid/directory => ../directory\n",
    )
    .expect("write");
    std::fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/typed-text-operands-go-harness/main.go"),
        harness.join("main.go"),
    )
    .expect("the harness copies");
    let (formatted, unformatted) = go_tool(&tree, "gofmt", &["-d", "."]);
    assert!(
        formatted && unformatted.trim().is_empty(),
        "the generated Go is gofmt-clean:\n{unformatted}"
    );
    let (vetted, log) = go_tool(&tree, &go, &["vet", "./..."]);
    assert!(vetted, "the generated Go is vet clean:\n{log}");
    let binary = scratch.0.join("harness-bin");
    let build = |label: &str| {
        let (built, log) = go_tool(
            &harness,
            &go,
            &["build", "-o", binary.to_str().expect("a UTF-8 path"), "."],
        );
        assert!(
            built,
            "{label}: the harness builds against the generated ports:\n{log}"
        );
    };
    build("healthy");
    let routes = routes(&ir);
    let healthy = verdicts(&ir, &binary, &routes);
    assert_eq!(not_passed(&healthy), Vec::<String>::new());
    eprintln!(
        "{} scenarios passed against the generated Go system",
        healthy.len()
    );

    let behaviour = tree.join("types/behaviour/behaviour.go");
    let generated = std::fs::read_to_string(&behaviour).expect("the behaviour reads");
    for fault in &GO_FAULTS {
        assert_eq!(
            generated.matches(fault.from).count(),
            1,
            "{}: the one helper every typed operand is tested through",
            fault.label
        );
        std::fs::write(&behaviour, generated.replace(fault.from, fault.to)).expect("patch");
        build(fault.label);
        caught(fault.label, &healthy, &verdicts(&ir, &binary, &routes));
    }
    std::fs::write(&behaviour, &generated).expect("restore");
}

// ---- the adapter: the suite's requests, over the harness's line protocol ----------------------

/// The harness process and one scenario's published events.
struct Served {
    child: RefCell<Child>,
    stdin: RefCell<ChildStdin>,
    stdout: RefCell<BufReader<ChildStdout>>,
    published: RefCell<Vec<ObservedEvent>>,
    sequence: RefCell<u64>,
    optional_fields: BTreeMap<String, Vec<String>>,
}

impl Served {
    fn start(binary: &Path, arguments: &[String], ir: &EssIr) -> Self {
        let mut child = Command::new(binary)
            .args(arguments)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("the harness starts");
        let stdin = child.stdin.take().expect("piped");
        let stdout = BufReader::new(child.stdout.take().expect("piped"));
        let optional = |fields: &[ess_compiler::ir::ResolvedField]| -> Vec<String> {
            fields
                .iter()
                .filter(|field| field.type_ref.is_optional())
                .map(|field| field.name.clone())
                .collect()
        };
        let optional_fields = ir
            .events()
            .values()
            .map(|event| (event.name.to_string(), optional(&event.fields)))
            .chain(
                ir.views()
                    .values()
                    .map(|view| (view.name.to_string(), optional(&view.fields))),
            )
            .collect();
        Self {
            child: RefCell::new(child),
            stdin: RefCell::new(stdin),
            stdout: RefCell::new(stdout),
            published: RefCell::default(),
            sequence: RefCell::new(0),
            optional_fields,
        }
    }

    /// One request line out, one `{status, body}` line back.
    fn ask(&self, request: &serde_json::Value) -> (u64, serde_json::Value) {
        let mut stdin = self.stdin.borrow_mut();
        writeln!(stdin, "{request}").expect("the harness reads");
        stdin.flush().expect("the harness reads");
        let mut line = String::new();
        self.stdout
            .borrow_mut()
            .read_line(&mut line)
            .expect("the harness answers");
        if std::env::var_os("TYPED_TEXT_OPERANDS_TRACE").is_some() {
            eprintln!("> {request}\n< {}", line.trim_end());
        }
        let answer: serde_json::Value =
            serde_json::from_str(&line).unwrap_or_else(|error| panic!("`{line}`: {error}"));
        (
            answer["status"].as_u64().unwrap_or_default(),
            answer["body"].clone(),
        )
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

/// A suite value as JSON, a number in its exact spelling.
fn json_of(node: &Node) -> serde_json::Value {
    match node {
        Node::Null => serde_json::Value::Null,
        Node::Bool(value) => serde_json::Value::Bool(*value),
        Node::Number(number) => {
            serde_json::from_str(&number.to_string()).expect("a number's spelling is JSON")
        }
        Node::Text(text) => serde_json::Value::String(text.clone()),
        Node::Seq(items) => serde_json::Value::Array(items.iter().map(json_of).collect()),
        Node::Map(members) => serde_json::Value::Object(
            members
                .iter()
                .map(|(name, value)| (name.clone(), json_of(value)))
                .collect(),
        ),
    }
}

/// A wire value as a suite value.
fn node_of(value: &serde_json::Value) -> Node {
    serde_json::from_value(value.clone()).expect("the wire writes JSON a node reads")
}

/// One query-string component, form-encoded: unreserved bytes as they are, every other byte as
/// `%XX`.
fn form_encoded(text: &str) -> String {
    text.bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                char::from(byte).to_string()
            }
            other => format!("%{other:02X}"),
        })
        .collect()
}

fn failure(observation: &str, answer: &serde_json::Value) -> TargetError {
    TargetError::unavailable(observation, answer.to_string())
}

impl ConformanceTarget for Served {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new(
            "typed-text-operands",
            env!("CARGO_PKG_VERSION"),
        ))
    }

    fn begin_scenario(&self, _scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.ask(&serde_json::json!({"op": "reset"}));
        self.published.borrow_mut().clear();
        *self.sequence.borrow_mut() = 0;
        Ok(())
    }

    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let observation = format!("invoking `{}`", request.command);
        let input: serde_json::Map<String, serde_json::Value> = request
            .input
            .iter()
            .map(|(name, value)| (name.clone(), json_of(value)))
            .collect();
        let (status, answer) = self.ask(&serde_json::json!({
            "op": "command",
            "command": request.command.to_string(),
            "body": serde_json::Value::Object(input).to_string(),
        }));
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
        let error = match answer.get("error") {
            Some(refused) => {
                let name: ErrorRef = refused
                    .as_str()
                    .and_then(|name| name.parse().ok())
                    .ok_or_else(|| failure(&observation, &answer))?;
                Some(DeclaredErrorValue::new(name))
            }
            None => None,
        };
        let mut direct_events = Vec::new();
        for published in answer["published"].as_array().into_iter().flatten() {
            let name = published["event"]
                .as_str()
                .ok_or_else(|| failure(&observation, &answer))?;
            let reference: EventRef = name.parse().map_err(|_| failure(&observation, &answer))?;
            let mut event = ObservedEvent::new(reference);
            if let Some(serde_json::Value::Object(payload)) = published.get("payload") {
                for (field, value) in payload {
                    let optional = self
                        .optional_fields
                        .get(name)
                        .is_some_and(|fields| fields.contains(field));
                    if value.is_null() && optional {
                        continue;
                    }
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
        let query = request
            .params
            .iter()
            .map(|(name, value)| {
                let text = match value {
                    Node::Text(text) => text.clone(),
                    other => json_of(other).to_string(),
                };
                format!("{}={}", form_encoded(name), form_encoded(&text))
            })
            .collect::<Vec<_>>()
            .join("&");
        let (status, answer) = self.ask(&serde_json::json!({
            "op": "view",
            "view": request.view.to_string(),
            "query": query,
        }));
        let Some(rows) = answer["rows"].as_array().filter(|_| status == 200) else {
            return Err(failure(&observation, &answer));
        };
        let optional = self
            .optional_fields
            .get(&request.view.to_string())
            .cloned()
            .unwrap_or_default();
        Ok(SemanticViewResult::of(rows.iter().map(|row| {
            let mut read: ViewRow = row
                .as_object()
                .into_iter()
                .flatten()
                .map(|(field, value)| (field.clone(), node_of(value)))
                .collect();
            for field in &optional {
                read.entry(field.clone()).or_insert(Node::Null);
            }
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
