//! A fully declared view's query is generated over the storage port
//! (`story:generated-view-queries`).
//!
//! The fixture `tests/fixtures/generated-views.yaml` declares every view construct the story names
//! as expressible: a projection of an entity's own fields and `state`, `filter:` views (over the
//! state, and over an `Optional` field whose absent value the filter cannot decide), an
//! `order_by:` view, and `aggregation:` views — grouped by an `Optional` key with rows lacking it,
//! grouped by two keys under a filter, grouped over a `Timestamp` input, and ungrouped — computing
//! `count`, `count_distinct` (over a value some rows lack, and over `state`), `sum`, `avg`, `min`
//! and `max`. Its Rust tree is synthesized in both layouts, the in-memory storage port in
//! `tests/fixtures/generated-views-harness/` is placed beside it (it holds no query and no
//! behaviour), and the suite the same specification synthesizes runs against the generated queries.

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
use ess_primitives::facts::FactValue;
use ess_primitives::node::Node;
use ess_synth::{
    synthesize_for, synthesize_laid_out, CapabilityKind, OutputLayout, SynthesisDisposition,
    SynthesisPlan, Target,
};

/// The fixture's text.
fn fixture() -> String {
    std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/generated-views.yaml"),
    )
    .expect("the fixture is readable")
}

/// One YAML document, compiled.
fn compile_text(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("well formed: {error}"));
    let specification = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("validates:\n{errors}"));
    let mut sources = SourceMap::new();
    sources.insert("model.yaml".to_owned(), text.to_owned());
    compile_locating(&specification, &sources, &["model.yaml".to_owned()])
        .unwrap_or_else(|diagnostics| panic!("resolves:\n{diagnostics}"))
}

/// Every view the fixture declares.
const VIEWS: [&str; 8] = [
    "ledger.work.Tasks",
    "ledger.work.DoneTasks",
    "ledger.work.LongTasks",
    "ledger.work.TasksByOwner",
    "ledger.work.HoursByTeam",
    "ledger.work.DueByOwner",
    "ledger.work.CostByOwner",
    "ledger.work.Totals",
];

#[test]
fn every_fully_declared_view_query_is_generated_over_a_listing_storage_port() {
    let ir = compile_text(&fixture());
    let synthesis = synthesize_for(&ir, Target::Rust).expect("the fixture synthesizes");
    for view in VIEWS {
        assert_eq!(
            synthesis
                .plan
                .disposition_of(CapabilityKind::ViewQuery, view),
            Some(&SynthesisDisposition::Generated),
            "`{view}` is fully declared, so its query is generated"
        );
    }
    let behaviour = &synthesis.artifacts["crates/ledger-types/src/behaviour.rs"].contents;
    assert!(
        behaviour.contains("fn list(&self) -> Vec<crate::work::TaskSnapshot>;"),
        "the storage port lists an entity's rows:\n{behaviour}"
    );
    for query in [
        "crate::work::obligations::TasksQuery for Generated<P>",
        "crate::work::obligations::TotalsQuery for Generated<P>",
    ] {
        assert!(behaviour.contains(query), "`{query}` missing:\n{behaviour}");
    }
    let domain = &synthesis.artifacts["crates/ledger-types/src/work.rs"].contents;
    assert!(domain.contains("pub trait TasksQuery"), "{domain}");
    assert!(
        !domain.contains("impl TasksQuery for Unimplemented"),
        "a generated query is not stubbed as owed:\n{domain}"
    );
    let markdown = &synthesis.artifacts["PLAN.md"].contents;
    assert!(
        !markdown.contains("how the projection is kept current"),
        "no view query is owed:\n{markdown}"
    );
}

/// A model whose one view uses a construct the generated query does not reproduce, and the phrase
/// its obligation must name.
struct Kept {
    view: &'static str,
    names: &'static str,
}

/// The fixture's entity and commands, with one further view appended per case.
fn with_view(view: &str) -> String {
    let base = fixture();
    let (head, _) = base
        .split_once("views:\n")
        .expect("the fixture declares views");
    let (_, components) = base
        .split_once("components:\n")
        .expect("the fixture declares components");
    format!("{head}views:\n{view}\ncomponents:\n{components}")
}

#[test]
fn a_view_construct_the_generated_query_does_not_reproduce_keeps_the_query_owed() {
    let cases = [
        Kept {
            view: "  - name: ledger.work.ForOwner
    source: ledger.work.Task
    params: [{name: owner, type: String}]
    filter: owner == param.owner
    fields:
      - {name: task_id, type: ledger.work.TaskId}
",
            names: "a view parameter (`params:`)",
        },
        Kept {
            view: "  - name: ledger.work.ByHours
    source: ledger.work.Task
    order_by: [hours]
    fields:
      - {name: task_id, type: ledger.work.TaskId}
      - {name: hours, type: Optional<Integer>}
",
            names: "an order over the optional field `hours`",
        },
        Kept {
            view: "  - name: ledger.work.LateOwners
    source: ledger.work.Task
    filter: owner > \"m\"
    fields:
      - {name: task_id, type: ledger.work.TaskId}
",
            names: "an ordering over text",
        },
    ];
    for case in cases {
        let ir = compile_text(&with_view(case.view));
        let view = ir
            .views()
            .keys()
            .next()
            .expect("the case declares one view")
            .to_string();
        let plan = SynthesisPlan::of(&ir);
        match plan.disposition_of(CapabilityKind::ViewQuery, &view) {
            Some(SynthesisDisposition::Obligation(obligation)) => {
                let why = obligation.reason.describes();
                assert!(
                    why.contains(case.names),
                    "`{view}` names `{}` as what kept it owed, not: {why}",
                    case.names
                );
            }
            other => panic!("`{view}` stays an obligation, not {other:?}"),
        }
        assert!(plan.to_markdown().contains(case.names));
        // An owed query keeps its stub, and nothing lists rows for a view nothing generates.
        let synthesis = synthesize_for(&ir, Target::Rust).expect("the case synthesizes");
        let behaviour = &synthesis.artifacts["crates/ledger-types/src/behaviour.rs"].contents;
        assert!(!behaviour.contains("fn list("), "{behaviour}");
        let domain = &synthesis.artifacts["crates/ledger-types/src/work.rs"].contents;
        assert!(domain.contains("for Unimplemented"), "{domain}");
    }
}

#[test]
fn a_paged_view_is_an_obligation_in_the_plan() {
    let ir = compile_text(&with_view(
        "  - name: ledger.work.Page
    source: ledger.work.Task
    params:
      - {name: page, type: Integer}
      - {name: size, type: Integer}
    order_by: [task_id asc]
    paging: {page: page, size: size}
    fields:
      - {name: task_id, type: ledger.work.TaskId}
",
    ));
    let plan = SynthesisPlan::of(&ir);
    match plan.disposition_of(CapabilityKind::ViewQuery, "ledger.work.Page") {
        Some(SynthesisDisposition::Obligation(obligation)) => {
            assert!(
                obligation
                    .reason
                    .describes()
                    .contains("a paged view (`paging:`)"),
                "{obligation:?}"
            );
        }
        other => panic!("a paged view stays owed, not {other:?}"),
    }
}

// ---- the generated tree, built and run against its own suite ----------------------------------

/// A scratch directory under this test binary's target, removed when dropped.
struct Scratch(PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Writes every artifact of a synthesis under `root`.
fn write_tree(root: &Path, synthesis: &ess_synth::Synthesis) {
    for artifact in synthesis.artifacts.values() {
        let path = root.join(&artifact.path);
        std::fs::create_dir_all(path.parent().expect("a file has a parent")).expect("mkdir");
        std::fs::write(&path, &artifact.contents).expect("write");
    }
}

/// Runs cargo offline in `directory`, with warnings denied, returning its combined output.
fn cargo(directory: &Path, target: &Path, arguments: &[&str]) -> (bool, String) {
    let output = Command::new(std::env::var_os("CARGO").expect("Cargo supplies its executable"))
        .args(arguments)
        .arg("--offline")
        .current_dir(directory)
        .env("CARGO_TARGET_DIR", target)
        .env("RUSTFLAGS", "-D warnings")
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

/// The harness crate's manifest, without its dependencies: those depend on the layout.
const HARNESS_MANIFEST: &str = "[package]
name = \"generated-views-harness\"
version = \"0.0.0\"
edition = \"2021\"
publish = false

[workspace]

[[bin]]
name = \"harness\"
path = \"src/main.rs\"

[dependencies]
";

/// The workspace layout's dependencies: three of its crates, by path.
const WORKSPACE_DEPENDENCIES: &str = "ledger-types = { path = \"../ledger/crates/ledger-types\" }
ledger-server = { path = \"../ledger/crates/ledger-server\" }
ledger-service = { path = \"../ledger/crates/ledger-service\" }
";

/// The single-crate layout's one dependency, with its HTTP surface switched on.
const CRATE_DEPENDENCIES: &str = "ledger = { path = \"../ledger\", features = [\"server\"] }\n";

/// The harness source, as committed.
fn harness_source() -> String {
    std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/generated-views-harness/main.rs"),
    )
    .expect("the harness is readable")
}

#[test]
fn the_generated_queries_build_with_warnings_denied_and_pass_their_own_suite() {
    let ir = compile_text(&fixture());
    let synthesis = synthesize_for(&ir, Target::Rust).expect("the fixture synthesizes");
    run_suite(
        &ir,
        &synthesis,
        "workspace",
        WORKSPACE_DEPENDENCIES,
        &harness_source(),
        &["check", "--workspace", "--all-targets"],
    );
}

#[test]
fn the_single_crate_layout_passes_the_same_suite_with_its_server_feature() {
    let ir = compile_text(&fixture());
    let synthesis = synthesize_laid_out(&ir, Target::Rust, OutputLayout::Crate)
        .expect("the fixture synthesizes as one crate");
    let harness = harness_source()
        .replace("ledger_server::", "ledger::server::")
        .replace("ledger_service::", "ledger::ports::ledger_service::")
        .replace("ledger_types::", "ledger::");
    run_suite(
        &ir,
        &synthesis,
        "crate",
        CRATE_DEPENDENCIES,
        &harness,
        &["check", "--all-targets", "--features", "server"],
    );
}

/// Writes the synthesis and the harness beside it, builds both with warnings denied, and runs the
/// suite the specification synthesizes against the harness.
fn run_suite(
    ir: &EssIr,
    synthesis: &ess_synth::Synthesis,
    label: &str,
    dependencies: &str,
    harness_source: &str,
    check: &[&str],
) {
    let scratch = Scratch(
        Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("generated-views-{label}-{}", std::process::id())),
    );
    let _ = std::fs::remove_dir_all(&scratch.0);
    let tree = scratch.0.join("ledger");
    write_tree(&tree, synthesis);
    let harness = scratch.0.join("harness");
    std::fs::create_dir_all(harness.join("src")).expect("mkdir");
    std::fs::write(
        harness.join("Cargo.toml"),
        format!("{HARNESS_MANIFEST}{dependencies}"),
    )
    .expect("write");
    std::fs::write(harness.join("src/main.rs"), harness_source).expect("write the harness");
    let target = scratch.0.join("target");

    let (built, log) = cargo(&tree, &target, check);
    assert!(
        built,
        "the generated {label} builds with -D warnings:\n{log}"
    );
    let (built, log) = cargo(&harness, &target, &["build"]);
    assert!(
        built,
        "the harness builds against the generated ports:\n{log}"
    );

    assert_suite_passes(ir, &target.join("debug/harness"), &[]);
    drop(scratch);
}

/// Runs the suite the specification synthesizes against one harness process, which every scenario
/// must pass, and which must read every view.
fn assert_suite_passes(ir: &EssIr, binary: &Path, arguments: &[String]) {
    let synthesized = ess_conformance::synthesize(ir);
    let suite = synthesized.suite;
    let admitted = AdmittedSuite::from_suite(&suite).unwrap_or_else(|error| panic!("{error}"));
    let target = Harnessed::start(binary, arguments, ir);
    let report = Runner::for_suite(&suite)
        .run_admitted(&admitted, &target)
        .into_report();
    let failed: Vec<String> = report
        .scenarios
        .iter()
        .filter(|scenario| scenario.status != Status::Passed)
        .map(|scenario| format!("{scenario:#?}"))
        .collect();
    assert!(
        failed.is_empty(),
        "{} of {} scenarios did not pass:\n{}",
        failed.len(),
        report.scenarios.len(),
        failed.join("\n")
    );
    // Every view reaches the suite: a view the suite synthesizes nothing for proves nothing about
    // its generated query.
    let mut read = std::collections::BTreeSet::new();
    views_read(
        &serde_json::to_value(&suite).expect("a suite serializes"),
        &mut read,
    );
    for view in VIEWS {
        assert!(
            read.contains(view),
            "the suite reads `{view}` in some scenario; it reads {read:?}"
        );
    }
    eprintln!(
        "{} of {} scenarios passed against the generated queries",
        report.scenarios.len(),
        suite.scenarios.len()
    );
    assert_eq!(report.scenarios.len(), suite.scenarios.len());
    drop(target);
}

/// `story:go-generated-behaviour`: the generated Go queries — projections, filters, an order and
/// every aggregate — over an in-memory storage port that holds no query
/// (`tests/fixtures/generated-views-go-harness/main.go`), are gofmt-clean, vet clean and build, and
/// pass the suite the specification synthesizes through the generated HTTP surface over a real
/// socket.
#[test]
fn the_go_queries_build_and_pass_their_own_suite() {
    let Some(go) = go() else {
        eprintln!("no Go toolchain on this machine; the generated Go queries are unchecked here");
        return;
    };
    let ir = compile_text(&fixture());
    let synthesis = synthesize_for(&ir, Target::Go).expect("the fixture synthesizes to Go");
    for view in VIEWS {
        assert_eq!(
            synthesis
                .plan
                .disposition_of(CapabilityKind::ViewQuery, view),
            Some(&SynthesisDisposition::Generated),
            "`{view}`"
        );
    }
    let behaviour = &synthesis.artifacts["types/behaviour/behaviour.go"].contents;
    for view in VIEWS {
        let method = view.rsplit('.').next().unwrap_or_default();
        let signature = format!(
            "func (b *Generated) {method}() ([]work.{method}, *obligation.UnmetObligation) {{"
        );
        assert!(
            behaviour.contains(&signature),
            "`{signature}` missing:\n{behaviour}"
        );
    }
    let scratch = Scratch(
        Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("generated-views-go-{}", std::process::id())),
    );
    let _ = std::fs::remove_dir_all(&scratch.0);
    let tree = scratch.0.join("ledger");
    write_tree(&tree, &synthesis);
    let harness = scratch.0.join("harness");
    std::fs::create_dir_all(&harness).expect("mkdir");
    std::fs::write(
        harness.join("go.mod"),
        "module ledgerharness\n\ngo 1.21\n\nrequire example.invalid/ledger v0.0.0\n\nreplace \
         example.invalid/ledger => ../ledger\n",
    )
    .expect("write");
    std::fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/generated-views-go-harness/main.go"),
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
    let (built, log) = go_tool(
        &harness,
        &go,
        &["build", "-o", binary.to_str().expect("a UTF-8 path"), "."],
    );
    assert!(
        built,
        "the harness builds against the generated ports:\n{log}"
    );

    let mut routes = Vec::new();
    for component in ir.components().values() {
        for route in ess_gen::http::routes(&ir, component) {
            let name = match route.serves {
                ess_gen::http::Served::Command(handle) => ir.command(handle).name.to_string(),
                ess_gen::http::Served::View(handle) => ir.view(handle).name.to_string(),
            };
            routes.extend([name, route.method.as_str().to_owned(), route.path]);
        }
    }
    assert_suite_passes(&ir, &binary, &routes);
    drop(scratch);
}

/// Where Go is, or `None` when this machine has none — said out loud, never passed silently.
fn go() -> Option<String> {
    let output = Command::new("go").arg("version").output().ok()?;
    output.status.success().then(|| "go".to_owned())
}

/// Runs a Go tool in `directory` with nothing fetched from a network, returning its output.
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

/// Every view some step of a serialized suite names.
fn views_read(value: &serde_json::Value, read: &mut std::collections::BTreeSet<String>) {
    match value {
        serde_json::Value::Object(members) => {
            for (key, member) in members {
                if let (true, serde_json::Value::String(view)) = (key == "view", member) {
                    read.insert(view.clone());
                }
                views_read(member, read);
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                views_read(item, read);
            }
        }
        _ => {}
    }
}

// ---- the adapter: the suite's requests, over the harness's line protocol ----------------------

/// The harness process and one scenario's published events.
struct Harnessed {
    child: RefCell<Child>,
    stdin: RefCell<ChildStdin>,
    stdout: RefCell<BufReader<ChildStdout>>,
    published: RefCell<Vec<ObservedEvent>>,
    sequence: RefCell<u64>,
    optional_fields: BTreeMap<String, Vec<String>>,
    /// The `Decimal` fields of each command's input and each view's row: the suite carries a
    /// decimal as a number, and the wire as its decimal string.
    decimal_fields: BTreeMap<String, Vec<String>>,
}

/// The names of `fields` whose type is a `Decimal`, optional or not.
fn decimals(fields: &[ess_compiler::ir::ResolvedField]) -> Vec<String> {
    fields
        .iter()
        .filter(|field| {
            field.type_ref.required()
                == &ess_compiler::ir::ResolvedTypeRef::Primitive {
                    name: ess_domain::types::Primitive::Decimal,
                }
        })
        .map(|field| field.name.clone())
        .collect()
}

impl Harnessed {
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
        let decimal_fields = ir
            .commands()
            .values()
            .map(|command| (command.name.to_string(), decimals(&command.input)))
            .chain(
                ir.views()
                    .values()
                    .map(|view| (view.name.to_string(), decimals(&view.fields))),
            )
            .collect();
        Self {
            child: RefCell::new(child),
            stdin: RefCell::new(stdin),
            stdout: RefCell::new(stdout),
            published: RefCell::default(),
            sequence: RefCell::new(0),
            optional_fields,
            decimal_fields,
        }
    }

    /// One request line out, one answer line back.
    fn ask(&self, request: &serde_json::Value) -> serde_json::Value {
        let mut stdin = self.stdin.borrow_mut();
        writeln!(stdin, "{request}").expect("the harness reads");
        stdin.flush().expect("the harness reads");
        let mut line = String::new();
        self.stdout
            .borrow_mut()
            .read_line(&mut line)
            .expect("the harness answers");
        if std::env::var_os("GENERATED_VIEWS_TRACE").is_some() {
            eprintln!("> {request}\n< {}", line.trim_end());
        }
        serde_json::from_str(&line).unwrap_or_else(|error| panic!("`{line}`: {error}"))
    }

    fn tick(&self) -> u64 {
        let mut sequence = self.sequence.borrow_mut();
        *sequence += 1;
        *sequence
    }
}

impl Drop for Harnessed {
    fn drop(&mut self) {
        let _ = self.child.borrow_mut().kill();
        let _ = self.child.borrow_mut().wait();
    }
}

/// A suite value as JSON, a number in its exact spelling (`1`, never `1.0`).
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

fn failure(observation: &str, answer: &serde_json::Value) -> TargetError {
    TargetError::unavailable(observation, answer.to_string())
}

impl ConformanceTarget for Harnessed {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new(
            "generated-views",
            env!("CARGO_PKG_VERSION"),
        ))
    }

    fn begin_scenario(&self, _scenario: &ScenarioContext) -> Result<(), TargetError> {
        let answer = self.ask(&serde_json::json!({"op": "reset"}));
        if answer["ok"] != serde_json::json!(true) {
            return Err(failure("opening a scenario", &answer));
        }
        self.published.borrow_mut().clear();
        *self.sequence.borrow_mut() = 0;
        Ok(())
    }

    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let observation = format!("invoking `{}`", request.command);
        let decimal = self
            .decimal_fields
            .get(&request.command.to_string())
            .cloned()
            .unwrap_or_default();
        let input: serde_json::Map<String, serde_json::Value> = request
            .input
            .iter()
            .map(|(name, value)| match value {
                Node::Number(number) if decimal.contains(name) => {
                    (name.clone(), serde_json::Value::String(number.to_string()))
                }
                _ => (name.clone(), json_of(value)),
            })
            .collect();
        let answer = self.ask(&serde_json::json!({
            "op": "command",
            "command": request.command.to_string(),
            "input": input,
        }));
        if answer.get("undeclared").is_some() {
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
        let answer = self.ask(&serde_json::json!({"op": "view", "view": request.view.to_string()}));
        let Some(rows) = answer["rows"].as_array() else {
            return Err(failure(&observation, &answer));
        };
        let optional = self
            .optional_fields
            .get(&request.view.to_string())
            .cloned()
            .unwrap_or_default();
        let decimal = self
            .decimal_fields
            .get(&request.view.to_string())
            .cloned()
            .unwrap_or_default();
        Ok(SemanticViewResult::of(rows.iter().map(|row| {
            // The wire leaves an absent optional field out; the row holds it as `null`.
            let mut read: ViewRow = row
                .as_object()
                .into_iter()
                .flatten()
                .map(|(field, value)| match value {
                    serde_json::Value::String(text) if decimal.contains(field) => {
                        match FactValue::parse_literal(text) {
                            FactValue::Number(number) => (field.clone(), Node::Number(number)),
                            _ => (field.clone(), node_of(value)),
                        }
                    }
                    _ => (field.clone(), node_of(value)),
                })
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
            "the fixture declares no external branch",
        ))
    }

    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        Err(TargetError::unsupported(
            format!("delivering `{}` again", request.event),
            "the fixture declares no binding",
        ))
    }

    fn end_scenario(&self, _scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.published.borrow_mut().clear();
        Ok(())
    }
}
