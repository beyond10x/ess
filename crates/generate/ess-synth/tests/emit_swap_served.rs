//! `emit-swap` against generated services: the Rust and the Go server this crate synthesizes for
//! `ess-conformance/tests/fixtures/emit-swap-served.yaml`, each run by the native runner and by the
//! generated Go and TypeScript runners (beyond10x/ess#295).
//!
//! The model has a creating (`PutItem/created`), an updating (`PutItem/updated`) and a moving
//! (`ArchiveItem/archived`) outcome with a compatible alternative event, and one creating outcome
//! (`BookSlot/booked`) with none. Every command reaches the generated behaviour through the
//! generated HTTP surface, over the in-memory storage ports of the upsert-by-existence harnesses
//! (`tests/fixtures/upsert-by-existence-harness/`, `…-go-harness/`), which hold no behaviour.
//!
//! The native audit runs each suite through a Rust adapter of the harness's line protocol. The
//! generated runners run the same emitted suites through `tests/fixtures/emit-swap-runners/`, the
//! same adapter in Go and in TypeScript, and write report/2 beside each suite; `collect` scores
//! them. Each swap is killed by the event the healthy service published, through all three
//! runners and both services, with the same unavailable site; the TypeScript runner with its event
//! expectations discarded lets the moving and the updating swap survive.
//!
//! The Go server and the Go runner are skipped, and said out loud, where the machine has no `go`;
//! the TypeScript runner where it has no `tsc` and `node`.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::io::{BufRead as _, BufReader, Write as _};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Output, Stdio};

use ess_compiler::ir::EssIr;
use ess_compiler::source::SourceMap;
use ess_conformance::mutate::{
    self, Document, MutantClass, MutationReport, Verdict, BASELINE_DIR, REPORT_FILE, SUITE_FILE,
};
use ess_domain::spec::RawSpecFile;
use ess_domain::system::Source;
use ess_primitives::node::Node;
use ess_synth::{synthesize_for, Target};

const MODEL: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/emit-swap-served.yaml");
const LABEL: &str = "emit-swap-served.yaml";
const SWAP: &[MutantClass] = &[MutantClass::EmitSwap];

const CREATED: &str =
    "emit-swap/demo.items.PutItem/created/demo.items.ItemStored/demo.items.ItemRestored";
const UPDATED: &str =
    "emit-swap/demo.items.PutItem/updated/demo.items.ItemStored/demo.items.ItemRestored";
const ARCHIVED: &str =
    "emit-swap/demo.items.ArchiveItem/archived/demo.items.ItemArchived/demo.items.ItemReviewed";
const BOOKED: &str = "emit-swap/demo.items.BookSlot/booked/demo.items.SlotBooked";

fn documents() -> (Vec<Document>, SourceMap) {
    let mut texts = SourceMap::new();
    texts.insert(LABEL.to_owned(), MODEL.to_owned());
    (
        vec![(Source::new(LABEL), RawSpecFile::parse(MODEL).unwrap())],
        texts,
    )
}

fn model() -> EssIr {
    let (files, texts) = documents();
    mutate::compile(files, &texts).expect("the fixture compiles")
}

fn scratch(label: &str) -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("emit-swap-{label}-{}", std::process::id()))
}

fn tool(name: &str) -> bool {
    let found = Command::new(name)
        .arg(if name == "go" { "version" } else { "--version" })
        .output()
        .is_ok_and(|output| output.status.success());
    if !found {
        println!("skipped: no `{name}` on PATH, so this case was not run");
    }
    found
}

fn shown(output: &Output) -> String {
    format!(
        "exit {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn write_artifacts(synthesis: &ess_synth::Synthesis, root: &Path) {
    for (relative, artifact) in &synthesis.artifacts {
        let destination = root.join(relative);
        std::fs::create_dir_all(destination.parent().unwrap()).unwrap();
        std::fs::write(destination, &artifact.contents).unwrap();
    }
}

// ---- the generated services -------------------------------------------------------------------

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
        let root = scratch("services");
        let _ = std::fs::remove_dir_all(&root);
        let kept = root.join("kept");
        std::fs::create_dir_all(&kept).unwrap();
        let synthesis = synthesize_for(&ir, Target::Rust).expect("the model synthesizes to Rust");
        write_artifacts(&synthesis, &root.join("demo"));
        let harness = root.join("harness");
        std::fs::create_dir_all(harness.join("src")).unwrap();
        let dependencies = ["demo-server", "demo-system", "items-service", "demo-types"]
            .iter()
            .fold(String::new(), |mut out, name| {
                let _ = writeln!(out, "{name} = {{ path = \"../demo/crates/{name}\" }}");
                out
            });
        std::fs::write(
            harness.join("Cargo.toml"),
            format!(
                "[package]\nname = \"emit-swap-harness\"\nversion = \"0.0.0\"\nedition = \
                 \"2021\"\npublish = false\n\n[workspace]\n\n[[bin]]\nname = \"harness\"\npath = \
                 \"src/main.rs\"\n\n[dependencies]\n{dependencies}"
            ),
        )
        .unwrap();
        std::fs::copy(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/upsert-by-existence-harness/main.rs"),
            harness.join("src/main.rs"),
        )
        .unwrap();
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
        std::fs::copy(target.join("debug/harness"), kept.join("rust")).unwrap();
        // The generated service is in the binary; its build tree is not needed again.
        let _ = std::fs::remove_dir_all(&target);
        let go = tool("go").then(|| {
            let synthesis = synthesize_for(&ir, Target::Go).expect("the model synthesizes to Go");
            let tree = root.join("go-demo");
            write_artifacts(&synthesis, &tree);
            let harness = root.join("go-harness");
            std::fs::create_dir_all(&harness).unwrap();
            std::fs::write(
                harness.join("go.mod"),
                "module emitswapharness\n\ngo 1.21\n\nrequire example.invalid/demo v0.0.0\n\n\
                 replace example.invalid/demo => ../go-demo\n",
            )
            .unwrap();
            std::fs::copy(
                Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("tests/fixtures/upsert-by-existence-go-harness/main.go"),
                harness.join("main.go"),
            )
            .unwrap();
            let binary = kept.join("go");
            let output = go(
                &harness,
                &["build", "-o", binary.to_str().unwrap(), "."],
                &[],
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
            go,
        }
    })
}

/// Every generated service this machine can build, by name.
fn services() -> Vec<(&'static str, PathBuf)> {
    let built = built();
    let mut found = vec![("rust", built.rust.clone())];
    found.extend(built.go.clone().map(|binary| ("go", binary)));
    found
}

fn go(directory: &Path, arguments: &[&str], env: &[(&str, String)]) -> Output {
    let mut command = Command::new("go");
    command
        .args(arguments)
        .current_dir(directory)
        .env("GOFLAGS", "-mod=mod")
        .env("GOPROXY", "off")
        .env("GOWORK", "off");
    for (key, value) in env {
        command.env(key, value);
    }
    command.output().expect("go runs")
}

/// The route table, as the harness takes it: `name method path` triples.
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

// ---- the native runner's adapter -----------------------------------------------------------------

/// One running harness, and the events one scenario published.
struct Served {
    child: RefCell<Child>,
    stdin: RefCell<ChildStdin>,
    stdout: RefCell<BufReader<ChildStdout>>,
    published: RefCell<Vec<ess_conformance::ObservedEvent>>,
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

    fn ask(&self, request: &serde_json::Value) -> serde_json::Value {
        let mut stdin = self.stdin.borrow_mut();
        writeln!(stdin, "{request}").expect("the harness reads");
        stdin.flush().expect("the harness reads");
        let mut line = String::new();
        self.stdout
            .borrow_mut()
            .read_line(&mut line)
            .expect("the harness answers");
        serde_json::from_str(&line).unwrap_or_else(|error| panic!("`{line}`: {error}"))
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

fn node_of(value: &serde_json::Value) -> Node {
    serde_json::from_value(value.clone()).expect("the wire writes JSON a node reads")
}

fn failure(observation: &str, answer: &serde_json::Value) -> ess_conformance::TargetError {
    ess_conformance::TargetError::unavailable(observation, answer.to_string())
}

impl ess_conformance::ConformanceTarget for Served {
    fn identity(
        &self,
    ) -> Result<ess_conformance::ImplementationIdentity, ess_conformance::TargetError> {
        Ok(ess_conformance::ImplementationIdentity::new(
            "emit-swap-served-generated",
            "1",
        ))
    }

    fn begin_scenario(
        &self,
        _: &ess_conformance::ScenarioContext,
    ) -> Result<(), ess_conformance::TargetError> {
        let answer = self.ask(&serde_json::json!({"op": "reset"}));
        if answer["ok"] != serde_json::json!(true) {
            return Err(failure("opening a scenario", &answer));
        }
        self.published.borrow_mut().clear();
        *self.sequence.borrow_mut() = 0;
        Ok(())
    }

    fn end_scenario(
        &self,
        _: &ess_conformance::ScenarioContext,
    ) -> Result<(), ess_conformance::TargetError> {
        self.published.borrow_mut().clear();
        Ok(())
    }

    fn execute_command(
        &self,
        request: ess_conformance::SemanticCommandRequest,
    ) -> Result<ess_conformance::SemanticCommandResult, ess_conformance::TargetError> {
        use ess_conformance::scenario::{CommandRef, ErrorRef, EventRef, OutcomeRef};
        let observation = format!("invoking `{}`", request.command);
        let body = serde_json::Value::Object(
            request
                .input
                .iter()
                .map(|(name, value)| (name.clone(), json_of(value)))
                .collect(),
        );
        let answer = self.ask(&serde_json::json!({
            "op": "command",
            "command": request.command.to_string(),
            "actor": request.actor.as_ref().map(ToString::to_string),
            "body": body.to_string(),
        }));
        let status = answer["status"]
            .as_u64()
            .ok_or_else(|| failure(&observation, &answer))?;
        let served = &answer["answer"];
        if status == 403 && served["refused"] == "not granted" {
            return Err(ess_conformance::TargetError::not_granted(
                served["actor"].as_str(),
            ));
        }
        if status == 501 {
            return Ok(ess_conformance::SemanticCommandResult::undeclared());
        }
        let outcome = served["outcome"]
            .as_str()
            .ok_or_else(|| failure(&observation, &answer))?;
        let outcome = OutcomeRef::new(
            CommandRef::new(request.command.name().clone()),
            outcome
                .parse()
                .map_err(|_| failure(&observation, &answer))?,
        );
        let error = match served.get("error") {
            Some(error) => {
                let name: ErrorRef = error
                    .as_str()
                    .and_then(|name| name.parse().ok())
                    .ok_or_else(|| failure(&observation, &answer))?;
                let mut declared = ess_conformance::DeclaredErrorValue::new(name);
                if let Some(serde_json::Value::Object(payload)) = served.get("payload") {
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
        for published in served["published"].as_array().into_iter().flatten() {
            let reference: EventRef = published["event"]
                .as_str()
                .and_then(|name| name.parse().ok())
                .ok_or_else(|| failure(&observation, &answer))?;
            let mut event = ess_conformance::ObservedEvent::new(reference);
            if let Some(serde_json::Value::Object(payload)) = published.get("payload") {
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
            ess_primitives::consistency::ConsistencyToken::new(format!("seq:{}", self.tick()))
                .map_err(|why| {
                    ess_conformance::TargetError::unavailable(observation.clone(), why.to_string())
                })?,
        );
        Ok(ess_conformance::SemanticCommandResult {
            outcome: Some(outcome),
            error,
            consistency,
            direct_events,
            response: None,
        })
    }

    fn query_view(
        &self,
        request: ess_conformance::SemanticViewRequest,
    ) -> Result<ess_conformance::SemanticViewResult, ess_conformance::TargetError> {
        let observation = format!("reading `{}`", request.view);
        let answer = self.ask(&serde_json::json!({"op": "view", "view": request.view.to_string()}));
        let Some(rows) = answer["answer"]["rows"].as_array() else {
            return Err(failure(&observation, &answer));
        };
        Ok(ess_conformance::SemanticViewResult::of(rows.iter().map(
            |row| {
                row.as_object()
                    .into_iter()
                    .flatten()
                    .map(|(field, value)| (field.clone(), node_of(value)))
                    .collect::<ess_conformance::target::ViewRow>()
            },
        )))
    }

    fn observe_events(
        &self,
        request: ess_conformance::EventObservationRequest,
    ) -> Result<Vec<ess_conformance::ObservedEvent>, ess_conformance::TargetError> {
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
        _: ess_conformance::ExternalOutcomeControl,
    ) -> Result<(), ess_conformance::TargetError> {
        Err(ess_conformance::TargetError::unsupported(
            "forcing an outcome",
            "none is external",
        ))
    }

    fn redeliver_event(
        &self,
        _: ess_conformance::RedeliveryRequest,
    ) -> Result<(), ess_conformance::TargetError> {
        Err(ess_conformance::TargetError::unsupported(
            "redelivering",
            "the model declares no binding",
        ))
    }
}

// ---- the audits ---------------------------------------------------------------------------------

fn verdicts(report: &MutationReport) -> BTreeMap<&str, Verdict> {
    report
        .mutants
        .iter()
        .map(|entry| (entry.id.as_str(), entry.verdict))
        .collect()
}

fn unavailable(report: &MutationReport) -> Vec<&str> {
    report
        .unavailable_sites
        .iter()
        .flatten()
        .map(|site| site.id.as_str())
        .collect()
}

/// The audit of every `emit-swap` site, each suite run by the native runner against `binary`.
fn native(binary: &Path) -> MutationReport {
    let (files, texts) = documents();
    let ir = model();
    mutate::audit(&files, &texts, SWAP, || Served::start(binary, &ir))
        .unwrap_or_else(|refusal| panic!("{refusal}"))
}

#[test]
fn the_native_runner_kills_every_swap_against_each_generated_service() {
    for (service, binary) in services() {
        let report = native(&binary);
        println!(
            "native runner against the generated {service} service: {}",
            report.render_text().lines().next().unwrap()
        );
        assert_eq!(report.baseline.not_scored.len(), 0, "{service}");
        assert_eq!(
            verdicts(&report),
            BTreeMap::from([
                (ARCHIVED, Verdict::Killed),
                (CREATED, Verdict::Killed),
                (UPDATED, Verdict::Killed),
            ]),
            "{service}: {}",
            report.render_text()
        );
        for (id, killer) in [
            (CREATED, "demo.items.PutItem/outcome/created"),
            (UPDATED, "demo.items.PutItem/outcome/updated"),
            (ARCHIVED, "demo.items.ArchiveItem/outcome/archived"),
        ] {
            let entry = report.mutants.iter().find(|entry| entry.id == id).unwrap();
            assert!(
                entry.killers.iter().flatten().any(|it| it == killer),
                "{service} {id}: {:?}",
                entry.killers
            );
        }
        assert_eq!(unavailable(&report), [BOOKED], "{service}");
    }
}

/// Where a generated runner's package is written for one suite.
struct Package {
    dir: PathBuf,
}

/// The generated Go runner's package for the suite text `suite`, with the served adapter.
fn go_package(root: &Path, suite: &str) -> Package {
    let parsed = ess_conformance::ConformanceSuite::from_json(suite).expect("an emitted suite");
    let dir = root.to_path_buf();
    let _ = std::fs::remove_dir_all(&dir);
    for artifact in ess_conformance::go::emit(&parsed).expect("the suite emits to Go") {
        let path = dir.join(&artifact.path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, artifact.contents).unwrap();
    }
    // The exact emitted bytes, which the report binds.
    std::fs::write(dir.join("essconform/suite.json"), suite).unwrap();
    std::fs::write(dir.join("go.mod"), "module emitswaprunner\n\ngo 1.21\n").unwrap();
    std::fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/emit-swap-runners/served_test.go"),
        dir.join("essconform/served_test.go"),
    )
    .unwrap();
    Package { dir }
}

/// The TypeScript runtime's event expectations, each answered as met without looking: a runner
/// that discards them.
const TS_DISCARDED: [(&str, &str); 4] = [
    (
        "      case 'expect_event':\n        return this.expectEvent(index, step);\n",
        "      case 'expect_event':\n        return true;\n",
    ),
    (
        "        return payload !== null && this.expectEventValues(index, { ...step, payload });\n",
        "        return payload !== null;\n",
    ),
    (
        "      case 'expect_no_event':\n        return this.expectNoEvent(index, step);\n",
        "      case 'expect_no_event':\n        return true;\n",
    ),
    (
        "      case 'eventually_event':\n        return this.eventuallyEvent(index, step);\n",
        "      case 'eventually_event':\n        return true;\n",
    ),
];

/// The generated TypeScript runner's package for the suite text `suite`, compiled, with the served
/// adapter; with its event expectations discarded where `blind`.
fn typescript_package(root: &Path, suite: &str, blind: bool) -> Package {
    let parsed = ess_conformance::ConformanceSuite::from_json(suite).expect("an emitted suite");
    let _ = std::fs::remove_dir_all(root);
    for artifact in ess_conformance::ts::emit(&parsed).expect("the suite emits to TypeScript") {
        let path = root.join(&artifact.path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, artifact.contents).unwrap();
    }
    let dir = root.join(ess_conformance::ts::PACKAGE);
    std::fs::write(dir.join("suite.json"), suite).unwrap();
    if blind {
        let runtime = dir.join("src/runtime.ts");
        let mut source = std::fs::read_to_string(&runtime).unwrap();
        for (kept, discarded) in TS_DISCARDED {
            assert_eq!(
                source.matches(kept).count(),
                1,
                "one `{kept}` in the runtime"
            );
            source = source.replace(kept, discarded);
        }
        std::fs::write(&runtime, source).unwrap();
    }
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/emit-swap-runners");
    for file in ["served.mjs", "driver.mjs"] {
        std::fs::copy(fixtures.join(file), dir.join(file)).unwrap();
    }
    std::fs::write(
        dir.join("runtime-test.tsconfig.json"),
        r#"{"extends":"./tsconfig.json","compilerOptions":{"types":[],"noCheck":true}}"#,
    )
    .unwrap();
    let compiled = Command::new("tsc")
        .args(["--project", "runtime-test.tsconfig.json"])
        .current_dir(&dir)
        .output()
        .expect("tsc runs");
    assert!(compiled.status.success(), "{}", shown(&compiled));
    Package { dir }
}

/// Runs `package` of `runner` over its suite against `binary`, writing report/2 to `report`.
fn run_package(runner: &str, package: &Package, binary: &Path, report: &Path) -> Output {
    let routes = routes(&model()).join(" ");
    let env = [
        ("ESS_REPORT_FORMAT", "2".to_owned()),
        ("ESS_REPORT_OUT", report.to_str().unwrap().to_owned()),
        ("ESS_SERVED_BINARY", binary.to_str().unwrap().to_owned()),
        ("ESS_SERVED_ROUTES", routes),
    ];
    let output = if runner == "go" {
        go(
            &package.dir,
            &["test", "./essconform", "-run", "^TestServed$", "-count=1"],
            &env,
        )
    } else {
        let mut command = Command::new("node");
        command
            .args(["--test", "driver.mjs"])
            .current_dir(&package.dir);
        for (key, value) in &env {
            command.env(key, value);
        }
        command.output().expect("node runs")
    };
    assert!(report.is_file(), "{runner}: {}", shown(&output));
    output
}

/// The emission's suites, each run by `runner` against `binary`, collected.
fn collected(runner: &str, binary: &Path, blind: bool) -> MutationReport {
    let (files, texts) = documents();
    let emission = mutate::emit(&files, &texts, SWAP).expect("emits");
    let root = scratch(&format!(
        "{runner}-{}-{}",
        binary.file_name().unwrap().to_string_lossy(),
        if blind { "blind" } else { "seeing" }
    ));
    let _ = std::fs::remove_dir_all(&root);
    let mut written = emission.files.clone();
    let mut dirs = vec![BASELINE_DIR.to_owned()];
    dirs.extend(
        emission
            .manifest
            .mutants
            .iter()
            .filter_map(|mutant| mutant.dir.clone()),
    );
    for (index, dir) in dirs.iter().enumerate() {
        let suite = &emission.files[&format!("{dir}/{SUITE_FILE}")];
        let at = root.join(format!("suite-{index}"));
        let package = if runner == "go" {
            go_package(&at, suite)
        } else {
            typescript_package(&at, suite, blind)
        };
        let report = at.join("report.json");
        let output = run_package(runner, &package, binary, &report);
        let text = std::fs::read_to_string(&report).unwrap();
        let value: serde_json::Value = serde_json::from_str(&text).unwrap();
        println!(
            "{runner} runner, {dir}: {} report/2 counts {}",
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .rfind(|line| ["ok ", "FAIL", "# pass", "# fail"]
                    .iter()
                    .any(|it| line.starts_with(it)))
                .unwrap_or_default()
                .trim(),
            value["counts"]
        );
        written.insert(format!("{dir}/{REPORT_FILE}"), text);
    }
    let report = mutate::collect(|path| written.get(path).cloned())
        .unwrap_or_else(|refusal| panic!("{runner}: {refusal}"));
    let _ = std::fs::remove_dir_all(&root);
    report
}

#[test]
fn the_generated_runners_kill_every_swap_against_each_generated_service_as_the_native_one_does() {
    let mut runners = Vec::new();
    if tool("go") {
        runners.push("go");
    }
    if tool("tsc") && tool("node") {
        runners.push("typescript");
    }
    for (service, binary) in services() {
        let native = native(&binary);
        for runner in &runners {
            let report = collected(runner, &binary, false);
            println!(
                "{runner} runner against the generated {service} service: {}",
                report.render_text().lines().next().unwrap()
            );
            assert_eq!(
                report.implementation, "emit-swap-served-generated 1",
                "{runner} {service}"
            );
            assert_eq!(
                verdicts(&report),
                verdicts(&native),
                "{runner} {service}: {}",
                report.render_text()
            );
            // Every scenario the native runner killed with also fails under the generated runner.
            // The generated runners may name more: where a later scenario captures the created
            // identity from the event the mutant names and the service published the other one,
            // they report `failed` and the native runner `error`, which kills nothing.
            for (collected, direct) in report.mutants.iter().zip(&native.mutants) {
                let generated = collected.killers.clone().unwrap_or_default();
                for killer in direct.killers.iter().flatten() {
                    assert!(
                        generated.contains(killer),
                        "{runner} {service} {}: {killer} not in {generated:?}",
                        collected.id
                    );
                }
            }
            assert_eq!(report.counts, native.counts, "{runner} {service}");
            assert_eq!(
                report.unavailable_sites, native.unavailable_sites,
                "{runner} {service}: the same unavailable site through every route"
            );
        }
    }
}

#[test]
fn a_typescript_runner_that_discards_event_expectations_fails_the_audit() {
    if !(tool("tsc") && tool("node")) {
        return;
    }
    let report = collected("typescript", &built().rust, true);
    println!(
        "typescript runner without event expectations: {}",
        report.render_text().lines().next().unwrap()
    );
    // The moving and the updating swap survive: only an event expectation told them apart. The
    // creating swap is still killed, by the later scenarios that capture the new identity from the
    // event the mutant names, which arranging a scenario needs and this runner keeps.
    assert_eq!(
        verdicts(&report),
        BTreeMap::from([
            (ARCHIVED, Verdict::Survived),
            (CREATED, Verdict::Killed),
            (UPDATED, Verdict::Survived),
        ]),
        "{}",
        report.render_text()
    );
    assert_eq!(report.counts.survived, 2);
    assert_eq!(unavailable(&report), [BOOKED]);
}
