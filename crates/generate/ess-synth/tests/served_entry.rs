//! A served component is runnable without handwritten storage, context or entry code.
use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_synth::{synthesize_for, Target};
use std::io::{BufRead as _, BufReader, Read as _, Write as _};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::OnceLock;

const MODEL: &str = include_str!("fixtures/served-notes/system.yaml");

fn ir(text: &str) -> EssIr {
    let specification = Specification::assemble([(
        Source::new("system.yaml"),
        RawSpecFile::parse(text).expect("fixture parses"),
    )])
    .unwrap_or_else(|error| panic!("fixture validates: {error}"));
    let mut sources = SourceMap::new();
    sources.insert("system.yaml", text);
    compile(&specification, &sources).unwrap_or_else(|error| panic!("fixture resolves: {error}"))
}

#[test]
fn the_served_notes_plan_has_no_obligation() {
    for target in [Target::Go, Target::Rust] {
        let synthesis = synthesize_for(&ir(MODEL), target).unwrap();
        assert!(
            synthesis.plan.obligations().next().is_none(),
            "{:?}",
            synthesis.plan
        );
    }
}

#[test]
fn the_go_entry_point_serves_the_fixture_with_no_hand_written_code() {
    exercise(&built().go);
}

#[test]
fn the_rust_entry_point_serves_the_fixture_with_no_hand_written_code() {
    exercise(&built().rust);
    exercise(&built().single);
}

#[test]
fn reusable_types_keep_the_default_dependency_graph_empty() {
    let output = root().join("default-library");
    for (path, artifact) in synthesize_for(&ir(MODEL), Target::Rust).unwrap().artifacts {
        let path = output.join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, artifact.contents).unwrap();
    }
    let result = Command::new(std::env::var_os("CARGO").unwrap())
        .args([
            "tree",
            "--offline",
            "-p",
            "notes-types",
            "--edges",
            "normal",
        ])
        .current_dir(&output)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let tree = String::from_utf8(result.stdout).unwrap();
    assert_eq!(
        tree.lines().count(),
        1,
        "default types dependencies: {tree}"
    );
    let result = Command::new(std::env::var_os("CARGO").unwrap())
        .args([
            "check",
            "--offline",
            "-p",
            "notes-types",
            "--target",
            "wasm32-unknown-unknown",
            "--target-dir",
        ])
        .arg(output.join("target"))
        .current_dir(&output)
        .env_remove("CARGO_TARGET_DIR")
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .env("RUSTFLAGS", "-D warnings")
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
}

struct Binaries {
    go: PathBuf,
    rust: PathBuf,
    single: PathBuf,
}

fn root() -> PathBuf {
    std::env::temp_dir().join(format!("ess-served-entry-{}", std::process::id()))
}

fn build(model: &EssIr, target: Target, single: bool, label: &str, component: &str) -> PathBuf {
    static BUILD: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let _guard = BUILD
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let output = root().join(label);
    std::fs::create_dir_all(&output).unwrap();
    let layout = if single {
        ess_synth::OutputLayout::Crate
    } else {
        ess_synth::OutputLayout::Workspace
    };
    let synthesis = ess_synth::synthesize_laid_out(model, target, layout).unwrap();
    for (path, artifact) in synthesis.artifacts {
        let path = output.join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, artifact.contents).unwrap();
    }
    let binary = root().join(format!("{label}-server"));
    let mut command = if target == Target::Go {
        let formatted = Command::new("gofmt")
            .args(["-d", "."])
            .current_dir(&output)
            .output()
            .expect("Go is required for this acceptance lane");
        assert!(formatted.status.success());
        assert!(
            formatted.stdout.is_empty(),
            "generated output is gofmt-clean: {}",
            String::from_utf8_lossy(&formatted.stdout)
        );
        let vetted = Command::new("go")
            .args(["vet", "./..."])
            .current_dir(&output)
            .env("GOPROXY", "off")
            .env("GOTOOLCHAIN", "local")
            .output()
            .unwrap();
        assert!(
            vetted.status.success(),
            "{}",
            String::from_utf8_lossy(&vetted.stderr)
        );
        let mut command = Command::new("go");
        command
            .args(["build", "-o"])
            .arg(&binary)
            .arg(format!("./cmd/{component}-server"));
        command.env("GOPROXY", "off").env("GOTOOLCHAIN", "local");
        command
    } else {
        let mut command = Command::new(std::env::var_os("CARGO").unwrap());
        command
            .args(["build", "--offline", "--bin"])
            .arg(format!("{component}-server"));
        if single {
            command.args(["--features", "server"]);
        }
        command
            .args(["--target-dir"])
            .arg(root().join("rust-target"));
        command
            .env_remove("CARGO_TARGET_DIR")
            .env_remove("CARGO_ENCODED_RUSTFLAGS")
            .env("CARGO_INCREMENTAL", "0")
            .env("CARGO_PROFILE_DEV_DEBUG", "0")
            .env("RUSTFLAGS", "-D warnings")
            .env("CARGO_BUILD_JOBS", "2");
        command
    };
    let result = command.current_dir(&output).output().unwrap();
    eprintln!(
        "{command:?}\n{}{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(
        result.status.success(),
        "generated {label} executable builds"
    );
    if target == Target::Rust {
        std::fs::copy(
            root().join(format!("rust-target/debug/{component}-server")),
            &binary,
        )
        .unwrap();
    }
    binary
}

fn built() -> &'static Binaries {
    static BINARIES: OnceLock<Binaries> = OnceLock::new();
    BINARIES.get_or_init(|| {
        let model = ir(MODEL);
        Binaries {
            go: build(&model, Target::Go, false, "go", "notes"),
            rust: build(&model, Target::Rust, false, "rust", "notes"),
            single: build(&model, Target::Rust, true, "single", "notes"),
        }
    })
}

struct Running {
    child: Child,
    port: u16,
}
impl Drop for Running {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn start(binary: &Path, callers: bool, directory: Option<&Path>) -> Running {
    let mut command = Command::new(binary);
    command.args(["--listen", "127.0.0.1:0"]);
    if callers {
        command.args(["--callers", "actor-header"]);
    }
    if let Some(directory) = directory {
        command.arg("--static").arg(directory);
    }
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();
    let mut lines = BufReader::new(child.stdout.take().unwrap()).lines();
    let line = lines.next().expect("startup record").unwrap();
    let record: serde_json::Value = serde_json::from_str(&line).unwrap();
    let port = u16::try_from(record["runtime"]["port"].as_u64().unwrap()).unwrap();
    std::thread::spawn(move || for _ in lines {});
    Running { child, port }
}

fn request(
    server: &Running,
    method: &str,
    path: &str,
    actor: Option<&str>,
    body: &str,
) -> (u16, Vec<u8>) {
    let mut stream = std::net::TcpStream::connect(("127.0.0.1", server.port)).unwrap();
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(10)))
        .unwrap();
    write!(stream, "{method} {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: {}\r\n", body.len()).unwrap();
    if let Some(actor) = actor {
        write!(stream, "Authorization: Actor {actor}\r\n").unwrap();
    }
    write!(stream, "\r\n{body}").unwrap();
    let mut answer = Vec::new();
    stream.read_to_end(&mut answer).unwrap();
    let boundary = answer
        .windows(4)
        .position(|bytes| bytes == b"\r\n\r\n")
        .unwrap();
    let status = std::str::from_utf8(&answer[..boundary])
        .unwrap()
        .split_whitespace()
        .nth(1)
        .unwrap()
        .parse()
        .unwrap();
    (status, answer[boundary + 4..].to_vec())
}

fn route(name: &str) -> String {
    let model = ir(MODEL);
    let component = model.components().values().next().unwrap();
    ess_gen::http::routes(&model, component)
        .into_iter()
        .find(|route| match route.serves {
            ess_gen::http::Served::Command(command) => {
                model.command(command).name.to_string().ends_with(name)
            }
            ess_gen::http::Served::View(view) => model.view(view).name.to_string().ends_with(name),
        })
        .unwrap()
        .path
}

fn json_request(
    server: &Running,
    method: &str,
    path: &str,
    actor: Option<&str>,
    body: &str,
) -> (u16, serde_json::Value) {
    let (status, bytes) = request(server, method, path, actor, body);
    (
        status,
        serde_json::from_slice(&bytes)
            .unwrap_or_else(|error| panic!("{error}: {}", String::from_utf8_lossy(&bytes))),
    )
}

fn exercise(binary: &Path) {
    let server = start(binary, true, None);
    let (status, added) = json_request(
        &server,
        "POST",
        &route("AddNote"),
        Some("Writer"),
        r#"{"title":"first"}"#,
    );
    assert_eq!(status, 202, "{added}");
    let id = added["published"][0]["payload"]["id"].as_str().unwrap();
    let (status, rows) = json_request(&server, "GET", &route("Notes"), None, "");
    assert_eq!(status, 200);
    assert_eq!(rows["rows"][0]["id"], id);
    assert_eq!(rows["rows"][0]["title"], "first");
    let (status, archived) = json_request(
        &server,
        "POST",
        &route("ArchiveNote"),
        Some("Writer"),
        &format!(r#"{{"id":"{id}"}}"#),
    );
    assert_eq!(status, 202, "{archived}");
    assert_eq!(
        json_request(&server, "GET", &route("Notes"), None, "").1["rows"][0]["state"],
        "Archived"
    );
    for actor in [Some("Reader"), None] {
        let (status, refusal) = json_request(
            &server,
            "POST",
            &route("AddNote"),
            actor,
            r#"{"title":"denied"}"#,
        );
        assert_eq!(status, 403);
        assert_eq!(refusal["refused"], "not granted");
    }
    let server = start(binary, false, None);
    assert_eq!(
        json_request(
            &server,
            "POST",
            &route("AddNote"),
            Some("Writer"),
            r#"{"title":"denied"}"#
        )
        .0,
        403
    );
}

#[test]
fn the_static_directory_is_served_beside_the_api() {
    let directory = root().join("public");
    std::fs::create_dir_all(&directory).unwrap();
    std::fs::write(directory.join("index.html"), "<h1>Notes</h1>").unwrap();
    std::fs::write(directory.join("asset.png"), [0, 255, 128, 42]).unwrap();
    let secret = root().join("outside.txt");
    std::fs::write(&secret, "outside secret").unwrap();
    #[cfg(unix)]
    {
        let _ = std::os::unix::fs::symlink(&secret, directory.join("escape.txt"));
    }
    let view = route("Notes");
    let api_file = directory.join(view.trim_start_matches('/'));
    std::fs::create_dir_all(api_file.parent().unwrap()).unwrap();
    std::fs::write(api_file, "API MUST WIN").unwrap();
    for binary in [&built().go, &built().rust, &built().single] {
        let server = start(binary, true, Some(&directory));
        assert_eq!(
            request(&server, "GET", "/", None, ""),
            (200, b"<h1>Notes</h1>".to_vec())
        );
        assert_eq!(
            request(&server, "GET", "/asset.png", None, ""),
            (200, vec![0, 255, 128, 42])
        );
        assert_eq!(
            json_request(&server, "GET", &view, None, ""),
            (200, serde_json::json!({"rows":[]}))
        );
        assert_eq!(request(&server, "POST", &view, None, "").0, 405);
        for path in [
            "/../outside.txt",
            "/%2e%2e/outside.txt",
            "/escape.txt",
            "/..%5coutside.txt",
        ] {
            let (status, body) = request(&server, "GET", path, None, "");
            assert_ne!(status, 200, "{path}");
            assert!(!String::from_utf8_lossy(&body).contains("outside secret"));
        }
    }
}

#[test]
fn the_store_lists_in_identity_order() {
    for binary in [&built().go, &built().rust] {
        let server = start(binary, true, None);
        for n in 0..24 {
            assert_eq!(
                json_request(
                    &server,
                    "POST",
                    &route("AddNote"),
                    Some("Writer"),
                    &format!(r#"{{"title":"{n}"}}"#)
                )
                .0,
                202
            );
        }
        let answer = json_request(&server, "GET", &route("Notes"), None, "").1;
        let ids: Vec<_> = answer["rows"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| row["id"].as_str().unwrap())
            .collect();
        assert_eq!(ids.len(), 24);
        assert!(ids.windows(2).all(|pair| pair[0] < pair[1]), "{ids:?}");
    }
}

#[test]
fn the_generated_context_assigns_distinct_uuids() {
    for binary in [&built().go, &built().rust] {
        let server = start(binary, true, None);
        let mut ids = std::collections::BTreeSet::new();
        for _ in 0..32 {
            let (status, answer) = json_request(
                &server,
                "POST",
                &route("AddNote"),
                Some("Writer"),
                r#"{"title":"UUID"}"#,
            );
            assert_eq!(status, 202);
            let id = answer["published"][0]["payload"]["id"].as_str().unwrap();
            assert_eq!(id.len(), 36);
            assert_eq!(&id[14..15], "4");
            assert!(matches!(&id[19..20], "8" | "9" | "a" | "b"));
            assert!(ids.insert(id.to_owned()));
        }
    }
}

fn directory_ir(base: &Path) -> EssIr {
    let mut pending = vec![base.to_path_buf()];
    let mut paths = Vec::new();
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                pending.push(path);
            } else if path
                .extension()
                .is_some_and(|extension| extension == "yaml")
            {
                paths.push(path);
            }
        }
    }
    paths.sort();
    let mut sources = SourceMap::new();
    let mut parsed = Vec::new();
    for path in paths {
        let text = std::fs::read_to_string(&path).unwrap();
        let label = path.strip_prefix(base).unwrap().display().to_string();
        parsed.push((
            Source::new(label.clone()),
            RawSpecFile::parse(&text).unwrap(),
        ));
        sources.insert(label, text);
    }
    compile(&Specification::assemble(parsed).unwrap(), &sources).unwrap()
}

fn refuses(model: &EssIr, label: &str, component: &str, expected: &str) {
    for (target, language) in [(Target::Go, "go"), (Target::Rust, "rust")] {
        let binary = build(
            model,
            target,
            false,
            &format!("{label}-{language}"),
            component,
        );
        let result = Command::new(binary)
            .args(["--listen", "127.0.0.1:0"])
            .output()
            .unwrap();
        assert!(!result.status.success());
        let stderr = String::from_utf8_lossy(&result.stderr);
        assert!(stderr.contains(expected), "{stderr}");
        assert!(
            result.stdout.is_empty(),
            "must refuse before binding or announcing"
        );
    }
}

#[test]
fn an_entry_point_with_owed_commands_refuses_to_start_naming_them() {
    let model =
        directory_ir(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../examples/gatepass"));
    refuses(
        &model,
        "owed",
        "pass-service",
        "gatepass.visit.RegisterVisit",
    );
}

#[test]
fn an_entry_point_needing_caller_attributes_refuses_to_start_naming_them() {
    let text = MODEL
        .replace(
            "may: [notes.notes.AddNote",
            "attributes: [{name: signature, type: String}]\n    may: [notes.notes.AddNote",
        )
        .replace(
            "sets: {title: input.title}",
            "sets: {title: {caller: signature}}",
        );
    refuses(
        &ir(&text),
        "caller",
        "notes",
        "caller attribute `signature`",
    );
    let text = MODEL.replace(
        "sets: {title: input.title}",
        "sets: {title: {generated: true}}",
    );
    refuses(&ir(&text), "assigned", "notes", "assigned value `String`");
    let text = MODEL.replace("      - name: added\n", "      - {name: offline, external: the writer is offline, error: notes.notes.NotFound}\n      - name: added\n");
    refuses(
        &ir(&text),
        "external",
        "notes",
        "external outcome `notes.notes.AddNote.offline`",
    );
}

#[test]
fn the_context_supplies_uuid_newtypes_and_system_clock_timestamps() {
    let text = MODEL.replace("domain: notes.notes\n", "domain: notes.notes\ntypes:\n  - {name: notes.notes.NoteId, kind: newtype, of: Uuid}\n")
        .replace("type: Uuid", "type: notes.notes.NoteId")
        .replace("      - {name: title, type: String}\n    lifecycle:", "      - {name: title, type: String}\n      - {name: created, type: Timestamp}\n    lifecycle:")
        .replace("sets: {title: input.title}", "sets: {title: input.title, created: {generated: true}}")
        .replace("      - {name: state, type: notes.notes.Note.State}", "      - {name: state, type: notes.notes.Note.State}\n      - {name: created, type: Timestamp}");
    let model = ir(&text);
    for (target, label) in [(Target::Go, "clock-go"), (Target::Rust, "clock-rust")] {
        let binary = build(&model, target, false, label, "notes");
        let before = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let server = start(&binary, true, None);
        assert_eq!(
            json_request(
                &server,
                "POST",
                &route("AddNote"),
                Some("Writer"),
                r#"{"title":"clock"}"#
            )
            .0,
            202
        );
        let answer = json_request(&server, "GET", &route("Notes"), None, "").1;
        let created = answer["rows"][0]["created"].as_str().unwrap();
        let instant = ess_primitives::time::Rfc3339Instant::parse_rfc3339(created)
            .expect("the generated timestamp is RFC3339");
        let after = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let lower =
            ess_primitives::time::Rfc3339Instant::from_epoch_millis(i64::try_from(before).unwrap())
                .unwrap();
        let upper = ess_primitives::time::Rfc3339Instant::from_epoch_millis(
            i64::try_from(after + 1).unwrap(),
        )
        .unwrap();
        assert!(
            instant >= lower && instant <= upper,
            "{created} is between the observed clocks"
        );
    }
}

#[test]
fn the_fixture_suite_passes_against_the_generated_go_and_rust_servers() {
    let suite = ess_conformance::synthesize(&ir(MODEL)).suite;
    let admitted = ess_conformance::AdmittedSuite::from_suite(&suite).unwrap();
    assert!(!suite.scenarios.is_empty());
    for binary in [&built().go, &built().rust] {
        let target = HttpTarget {
            binary,
            running: std::cell::RefCell::new(None),
            events: std::cell::RefCell::new(Vec::new()),
        };
        let report = ess_conformance::Runner::for_suite(&suite)
            .run_admitted(&admitted, &target)
            .into_report();
        eprintln!(
            "{} conformance: {} scenarios: {:?}",
            binary.display(),
            report.scenarios.len(),
            report.scenarios
        );
        assert!(
            report
                .scenarios
                .iter()
                .all(|scenario| scenario.status == ess_conformance::report::Status::Passed),
            "{report:?}"
        );
    }
}

struct HttpTarget<'a> {
    binary: &'a Path,
    running: std::cell::RefCell<Option<Running>>,
    events: std::cell::RefCell<Vec<ess_conformance::ObservedEvent>>,
}

impl ess_conformance::ConformanceTarget for HttpTarget<'_> {
    fn identity(
        &self,
    ) -> Result<ess_conformance::ImplementationIdentity, ess_conformance::TargetError> {
        Ok(ess_conformance::ImplementationIdentity::new(
            "generated-served-notes",
            "1",
        ))
    }
    fn begin_scenario(
        &self,
        _: &ess_conformance::ScenarioContext,
    ) -> Result<(), ess_conformance::TargetError> {
        *self.running.borrow_mut() = Some(start(self.binary, true, None));
        self.events.borrow_mut().clear();
        Ok(())
    }
    fn end_scenario(
        &self,
        _: &ess_conformance::ScenarioContext,
    ) -> Result<(), ess_conformance::TargetError> {
        self.running.borrow_mut().take();
        Ok(())
    }
    fn execute_command(
        &self,
        request: ess_conformance::SemanticCommandRequest,
    ) -> Result<ess_conformance::SemanticCommandResult, ess_conformance::TargetError> {
        let input = serde_json::to_string(&request.input).unwrap();
        let actor = request.actor.as_ref().map(ToString::to_string);
        let (status, answer) = json_request(
            self.running.borrow().as_ref().unwrap(),
            "POST",
            &route(&request.command.to_string()),
            actor.as_deref(),
            &input,
        );
        if status == 403 {
            return Err(ess_conformance::TargetError::not_granted(
                answer["actor"].as_str(),
            ));
        }
        let Some(outcome) = answer["outcome"].as_str() else {
            return Err(ess_conformance::TargetError::unavailable(
                "command",
                answer.to_string(),
            ));
        };
        let mut result = ess_conformance::SemanticCommandResult::took(
            ess_compiler::refs::OutcomeRef::new(request.command.clone(), outcome.parse().unwrap()),
        );
        if let Some(error) = answer["error"].as_str() {
            let mut value = ess_conformance::DeclaredErrorValue::new(error.parse().unwrap());
            if let Some(payload) = answer["payload"].as_object() {
                for (field, node) in payload {
                    value = value.with(field, serde_json::from_value(node.clone()).unwrap());
                }
            }
            result = result.with_error(value);
        }
        for published in answer["published"].as_array().into_iter().flatten() {
            let mut event = ess_conformance::ObservedEvent::new(
                published["event"].as_str().unwrap().parse().unwrap(),
            )
            .in_activity(request.correlation.clone());
            for (field, value) in published["payload"].as_object().unwrap() {
                event = event.with(field, serde_json::from_value(value.clone()).unwrap());
            }
            self.events.borrow_mut().push(event.clone());
            result = result.emitting(event);
        }
        Ok(result.with_consistency(
            ess_primitives::consistency::ConsistencyToken::new("synchronous-http").unwrap(),
        ))
    }
    fn query_view(
        &self,
        request: ess_conformance::SemanticViewRequest,
    ) -> Result<ess_conformance::SemanticViewResult, ess_conformance::TargetError> {
        let (status, answer) = json_request(
            self.running.borrow().as_ref().unwrap(),
            "GET",
            &route(&request.view.to_string()),
            None,
            "",
        );
        assert_eq!(status, 200);
        Ok(ess_conformance::SemanticViewResult::of(
            answer["rows"]
                .as_array()
                .unwrap()
                .iter()
                .map(|row| serde_json::from_value(row.clone()).unwrap()),
        ))
    }
    fn observe_events(
        &self,
        request: ess_conformance::EventObservationRequest,
    ) -> Result<Vec<ess_conformance::ObservedEvent>, ess_conformance::TargetError> {
        Ok(self
            .events
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
            "external outcome",
            "fixture has none",
        ))
    }
    fn redeliver_event(
        &self,
        _: ess_conformance::RedeliveryRequest,
    ) -> Result<(), ess_conformance::TargetError> {
        Err(ess_conformance::TargetError::unsupported(
            "redelivery",
            "fixture has no bindings",
        ))
    }
}
