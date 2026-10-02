//! A served view's declared parameters reach its port from the query string
//! (`story:served-view-params`, defect B of beyond10x/ess#311).
//!
//! The model declares one view with four parameters: a required `String` carried under the wire
//! name `who`, a required `Integer`, an `Optional<Boolean>` and an `Optional` enum. Its query stays
//! an obligation (the epic excludes generating one for a view with parameters), so each harness
//! implements the owed query itself and answers one row whose `task_id` spells every parameter it
//! received. The Rust and the Go servers are built, started on an ephemeral port, asked over a
//! socket, and killed by their PID.
//!
//! The last case takes the query keys from `ess_ui_check::binding` — the route table a renderer
//! reads — rather than from this file, so the names a UI sends are the names both servers decode.

use std::fmt::Write as _;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};

use ess_compiler::EssIr;
use ess_gen::http::{self, Served};
use ess_synth::{synthesize_laid_out, OutputLayout, Synthesis, Target};

const MODEL: &str = "format: ess/18
system: desk
version: v1
domain: desk.work
types:
  - {name: desk.work.Level, kind: enum, variants: [Low, High]}
entities:
  - name: desk.work.Task
    identity: {name: task_id, type: String}
    fields:
      - {name: owner, type: String}
      - {name: hours, type: Integer}
      - {name: urgent, type: Boolean}
      - {name: level, type: desk.work.Level}
    lifecycle:
      initial: Open
      states: [Open, Done]
      terminal: [Done]
      transitions:
        - {name: finish, from: [Open], to: Done}
events:
  - name: desk.work.TaskFinished
    fields:
      - {name: task_id, type: String}
errors:
  - name: desk.work.Late
    fields:
      - {name: reason, type: String}
commands:
  - name: desk.work.FinishTask
    input:
      - {name: task_id, type: String}
    outcomes:
      - name: finished
        moves: desk.work.Task.finish
        instance: task_id
        emits: [desk.work.TaskFinished]
        payload:
          desk.work.TaskFinished: {task_id: input.task_id}
      - name: late
        external: the task is past due
        error: desk.work.Late
views:
  - name: desk.work.ForOwner
    source: desk.work.Task
    consistency: eventual
    params:
      - {name: owner, type: String, wire: who}
      - {name: min_hours, type: Integer}
      - {name: urgent, type: Optional<Boolean>}
      - {name: level, type: Optional<desk.work.Level>}
    filter: [owner == param.owner, hours >= param.min_hours, urgent == param.urgent, level == param.level]
    fields:
      - {name: task_id, type: String}
      - {name: owner, type: String}
components:
  - component: desk-service
    owns: {domains: [desk.work]}
    accepts: {commands: [desk.work.FinishTask]}
    publishes: {events: [desk.work.TaskFinished]}
    reached_by: network
";

/// The owed query, answered by one row that spells what the port was handed.
const RUST_HARNESS: &str = r#"use desk_types::obligation::UnmetObligation;
use desk_types::work::{self, Level};

struct Desk;

impl work::obligations::FinishTaskBehavior for Desk {
    fn finish_task(
        &mut self,
        _input: work::FinishTask,
    ) -> Result<work::FinishTaskOutcome, UnmetObligation> {
        Err(UnmetObligation { capability: "command behaviour", source: "desk.work.FinishTask" })
    }
}

impl work::obligations::ForOwnerQuery for Desk {
    fn for_owner(
        &self,
        owner: String,
        min_hours: i64,
        urgent: Option<bool>,
        level: Option<Level>,
    ) -> Result<Vec<work::ForOwner>, UnmetObligation> {
        let urgent = urgent.map_or_else(|| "-".to_owned(), |urgent| urgent.to_string());
        let level = match level {
            None => "-",
            Some(Level::Low) => "Low",
            Some(Level::High) => "High",
        };
        Ok(vec![work::ForOwner {
            task_id: format!("owner={owner};min_hours={min_hours};urgent={urgent};level={level}"),
            owner,
        }])
    }
}

fn main() {
    let mut system = desk_system::System::new(desk_service::DeskService::new(Desk));
    desk_server::desk_service::serve(&mut system, "127.0.0.1:0").expect("the surface serves");
}
"#;

/// The same harness, for the Go server.
const GO_HARNESS: &str = r#"package main

import (
	"fmt"

	"example.invalid/desk/components/deskservice"
	"example.invalid/desk/server"
	"example.invalid/desk/system"
	"example.invalid/desk/types/obligation"
	"example.invalid/desk/types/work"
)

type desk struct{}

func (desk) FinishTask(work.FinishTask) (work.FinishTaskOutcome, *obligation.UnmetObligation) {
	return nil, &obligation.UnmetObligation{Capability: "command behaviour", Source: "desk.work.FinishTask"}
}

func (desk) ForOwner(owner string, minHours int64, urgent *bool, level *work.Level) ([]work.ForOwner, *obligation.UnmetObligation) {
	spelledUrgent := "-"
	if urgent != nil {
		spelledUrgent = fmt.Sprint(*urgent)
	}
	spelledLevel := "-"
	if level != nil {
		switch (*level).(type) {
		case work.LevelLow:
			spelledLevel = "Low"
		case work.LevelHigh:
			spelledLevel = "High"
		}
	}
	return []work.ForOwner{{
		TaskId: fmt.Sprintf("owner=%s;min_hours=%d;urgent=%s;level=%s", owner, minHours, spelledUrgent, spelledLevel),
		Owner:  owner,
	}}, nil
}

func main() {
	if err := server.ServeDeskService(system.NewSystem(deskservice.New(desk{})), "127.0.0.1:0"); err != nil {
		panic(err)
	}
}
"#;

fn sources() -> Vec<(String, String)> {
    vec![("model.yaml".to_owned(), MODEL.to_owned())]
}

fn ir() -> EssIr {
    ess_ui_check::compile_sources(&sources(), Path::new("model.yaml"))
        .unwrap_or_else(|error| panic!("the model compiles: {error}"))
}

/// The path `http::routes` serves the view at.
fn view_path() -> String {
    let ir = ir();
    let component = ir.components().values().next().expect("one component");
    http::routes(&ir, component)
        .into_iter()
        .find(|route| matches!(route.serves, Served::View(_)))
        .expect("the view is served")
        .path
}

/// Where this test binary builds, under Cargo's own scratch directory rather than `/tmp`.
fn scratch(label: &str) -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("view-params-served-{label}"))
}

fn write(synthesis: &Synthesis, directory: &Path) {
    let _ = std::fs::remove_dir_all(directory);
    for (relative, artifact) in &synthesis.artifacts {
        let destination = directory.join(relative);
        std::fs::create_dir_all(destination.parent().expect("a parent")).expect("mkdir");
        std::fs::write(&destination, &artifact.contents).expect("write");
    }
}

fn reported(what: &str, output: &Output) {
    eprintln!(
        "{what}\n{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

/// The Rust server, built once per test binary; the path of its executable.
fn rust_server() -> &'static Path {
    static BUILT: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();
    BUILT.get_or_init(|| {
        let synthesis = synthesize_laid_out(&ir(), Target::Rust, OutputLayout::Workspace)
            .expect("the model synthesizes for Rust");
        let root = scratch("rust");
        let _ = std::fs::remove_dir_all(&root);
        write(&synthesis, &root.join("desk"));
        let harness = root.join("harness");
        std::fs::create_dir_all(harness.join("src")).expect("mkdir");
        let dependencies = ["desk-server", "desk-system", "desk-service", "desk-types"]
            .iter()
            .fold(String::new(), |mut lines, name| {
                let _ = writeln!(lines, "{name} = {{ path = \"../desk/crates/{name}\" }}");
                lines
            });
        std::fs::write(
            harness.join("Cargo.toml"),
            format!(
                "[package]\nname = \"view-params-harness\"\nversion = \"0.0.0\"\nedition = \
                 \"2021\"\n\n[dependencies]\n{dependencies}\n[workspace]\n"
            ),
        )
        .expect("write");
        std::fs::write(harness.join("src/main.rs"), RUST_HARNESS).expect("write");
        let target = root.join("target");
        let built = Command::new(std::env::var_os("CARGO").expect("Cargo supplies its executable"))
            .args(["build", "--offline", "--quiet", "--target-dir"])
            .arg(&target)
            .current_dir(&harness)
            .env_remove("CARGO_TARGET_DIR")
            .env_remove("CARGO_ENCODED_RUSTFLAGS")
            .env_remove("RUSTC_WRAPPER")
            .env("CARGO_INCREMENTAL", "0")
            .output()
            .expect("cargo runs");
        reported("cargo build (the Rust harness)", &built);
        assert!(built.status.success(), "the Rust harness builds");
        // The executable outlives its build tree, which is removed: only the server is needed.
        let binary = scratch("rust-server");
        std::fs::copy(target.join("debug/view-params-harness"), &binary).expect("copy");
        let _ = std::fs::remove_dir_all(&root);
        binary
    })
}

/// The Go server, built once per test binary; the path of its executable.
fn go_server() -> &'static Path {
    static BUILT: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();
    BUILT.get_or_init(|| {
        let synthesis = synthesize_laid_out(&ir(), Target::Go, OutputLayout::Workspace)
            .expect("the model synthesizes for Go");
        let root = scratch("go");
        let _ = std::fs::remove_dir_all(&root);
        let module = root.join("desk");
        write(&synthesis, &module);
        std::fs::create_dir_all(module.join("cmd/harness")).expect("mkdir");
        std::fs::write(module.join("cmd/harness/main.go"), GO_HARNESS).expect("write");
        let binary = scratch("go-server");
        let cache = std::env::var_os("GOCACHE").map_or_else(|| root.join("gocache"), PathBuf::from);
        let built = Command::new("go")
            .args(["build", "-o"])
            .arg(&binary)
            .arg("./cmd/harness")
            .current_dir(&module)
            .env("GOCACHE", cache)
            .env("GOFLAGS", "-mod=mod")
            .env("GOPROXY", "off")
            .env("GOTOOLCHAIN", "local")
            .output()
            .unwrap_or_else(|error| panic!("`go` runs: {error}"));
        reported("go build (the Go harness)", &built);
        assert!(built.status.success(), "the Go harness builds");
        let _ = std::fs::remove_dir_all(&root);
        binary
    })
}

/// One started server, killed by its PID when the case ends, however it ends.
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

fn start(executable: &Path) -> Running {
    let mut child = Command::new(executable)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap_or_else(|error| panic!("{} starts: {error}", executable.display()));
    let stdout = child.stdout.take().expect("piped");
    let mut lines = BufReader::new(stdout).lines();
    let first = lines
        .next()
        .expect("the server writes its startup record")
        .expect("UTF-8");
    let record: serde_json::Value = serde_json::from_str(&first).expect("a JSON startup line");
    let port =
        u16::try_from(record["runtime"]["port"].as_u64().expect("the bound port")).expect("a port");
    // The rest of the record is read and dropped on a thread of its own, so the server never
    // blocks on a full pipe.
    std::thread::spawn(move || for _ in lines {});
    Running { child, port }
}

/// `GET target` over a socket: the status and the body.
fn get(server: &Running, target: &str) -> (u16, serde_json::Value) {
    let mut stream = TcpStream::connect(("127.0.0.1", server.port)).expect("the server accepts");
    write!(
        stream,
        "GET {target} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n"
    )
    .expect("the request is written");
    let mut answer = String::new();
    stream
        .read_to_string(&mut answer)
        .expect("the answer is read");
    let (head, body) = answer
        .split_once("\r\n\r\n")
        .unwrap_or_else(|| panic!("an HTTP answer: {answer}"));
    let status = head
        .split(' ')
        .nth(1)
        .and_then(|status| status.parse().ok())
        .unwrap_or_else(|| panic!("a status line: {head}"));
    let body = serde_json::from_str(body).unwrap_or_else(|error| panic!("{error}: {body}"));
    (status, body)
}

/// What the port was handed, as the harness's one row spells it.
fn handed(status: u16, body: &serde_json::Value) -> String {
    assert_eq!(status, 200, "{body}");
    body["rows"][0]["task_id"]
        .as_str()
        .unwrap_or_else(|| panic!("one row: {body}"))
        .to_owned()
}

fn reaches_the_port(executable: &Path) {
    let server = start(executable);
    let path = view_path();
    // Percent-encoded and `+`-encoded spaces, an undeclared key and an absent optional.
    let (status, body) = get(
        &server,
        &format!("{path}?who=ada%20l+ovelace&min_hours=3&urgent=true&level=High&page=2"),
    );
    assert_eq!(
        handed(status, &body),
        "owner=ada l ovelace;min_hours=3;urgent=true;level=High"
    );
    let (status, body) = get(&server, &format!("{path}?min_hours=-4&who=bob"));
    assert_eq!(
        handed(status, &body),
        "owner=bob;min_hours=-4;urgent=-;level=-"
    );
    // The declared name is not the query key: `owner` is carried as `who`.
    let (status, body) = get(&server, &format!("{path}?owner=bob&min_hours=1"));
    assert_eq!(status, 400, "{body}");
}

#[test]
fn a_view_param_reaches_the_port_from_the_query_string_rust() {
    reaches_the_port(rust_server());
}

#[test]
fn a_view_param_reaches_the_port_from_the_query_string_go() {
    reaches_the_port(go_server());
}

/// A refusal this surface makes: `400`, with one member, `refused`, naming the query key.
fn refused(server: &Running, target: &str, names: &str) -> String {
    let (status, body) = get(server, target);
    assert_eq!(status, 400, "`{target}`: {body}");
    let members = body
        .as_object()
        .unwrap_or_else(|| panic!("an object: {body}"));
    assert_eq!(members.len(), 1, "`{target}`: {body}");
    let detail = members["refused"]
        .as_str()
        .unwrap_or_else(|| panic!("`refused` is text: {body}"));
    assert!(
        detail.contains(names),
        "`{target}` names `{names}`: {detail}"
    );
    detail.to_owned()
}

#[test]
fn a_missing_required_param_is_a_400_refusal() {
    let path = view_path();
    let mut missing = Vec::new();
    for executable in [rust_server(), go_server()] {
        let server = start(executable);
        missing.push(refused(&server, &format!("{path}?min_hours=3"), "who"));
        refused(&server, &path, "who");
        refused(&server, &format!("{path}?who=ada"), "min_hours");
        // Undecodable values of every scalar kind the model declares.
        refused(
            &server,
            &format!("{path}?who=ada&min_hours=three"),
            "min_hours",
        );
        refused(
            &server,
            &format!("{path}?who=ada&min_hours=1.5"),
            "min_hours",
        );
        refused(
            &server,
            &format!("{path}?who=ada&min_hours=1&urgent=yes"),
            "urgent",
        );
        refused(
            &server,
            &format!("{path}?who=ada&min_hours=1&level=Medium"),
            "level",
        );
        // A declared key sent twice is two answers to one question.
        refused(
            &server,
            &format!("{path}?who=ada&who=bob&min_hours=1"),
            "who",
        );
        refused(&server, &format!("{path}?who=%ZZ&min_hours=1"), "who");
    }
    assert_eq!(
        missing[0], missing[1],
        "the two servers refuse a missing parameter in the same words"
    );
}

/// `examples/<name>`, every `.yaml` file under it in a stable order, as `(label, text)` sources.
fn example(name: &str) -> EssIr {
    let base = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../examples")
        .join(name);
    let mut found = Vec::new();
    let mut pending = vec![base.clone()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).expect("the example is readable") {
            let path = entry.expect("an entry").path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|it| it == "yaml") {
                found.push(path);
            }
        }
    }
    found.sort();
    let sources: Vec<(String, String)> = found
        .iter()
        .map(|path| {
            (
                path.strip_prefix(&base)
                    .expect("inside the example")
                    .display()
                    .to_string(),
                std::fs::read_to_string(path).expect("readable"),
            )
        })
        .collect();
    ess_ui_check::compile_sources(&sources, &base)
        .unwrap_or_else(|error| panic!("`{name}` compiles: {error}"))
}

fn artifact<'a>(synthesis: &'a Synthesis, path: &str) -> &'a str {
    &synthesis
        .artifacts
        .get(path)
        .unwrap_or_else(|| panic!("`{path}` is emitted"))
        .contents
}

/// Every line, byte for byte as the committed trees carried them before view parameters were
/// served: a view without parameters keeps its query's signature in every seam that spells it.
#[test]
fn a_view_without_params_keeps_its_bytes() {
    let billing = example("billing");
    let gatepass = example("gatepass");
    let rust = |ir: &EssIr| {
        synthesize_laid_out(ir, Target::Rust, OutputLayout::Workspace).expect("Rust synthesizes")
    };
    let go = |ir: &EssIr| {
        synthesize_laid_out(ir, Target::Go, OutputLayout::Workspace).expect("Go synthesizes")
    };
    let billing_rust = rust(&billing);
    for (path, line) in [
        (
            "crates/billing-types/src/invoice.rs",
            "        fn outstanding_invoices(&self) -> Result<Vec<super::OutstandingInvoices>, \
             crate::obligation::UnmetObligation>;\n",
        ),
        (
            "crates/billing-types/src/invoice.rs",
            "    impl OutstandingInvoicesQuery for Unimplemented {\n        fn \
             outstanding_invoices(&self) -> Result<Vec<super::OutstandingInvoices>, \
             crate::obligation::UnmetObligation> {\n",
        ),
        (
            "crates/billing-types/src/behaviour.rs",
            "    fn outstanding_invoices(&self) -> Result<Vec<crate::invoice::OutstandingInvoices>, \
             UnmetObligation> {\n        \
             crate::invoice::obligations::OutstandingInvoicesQuery::outstanding_invoices(&self.\
             ports)\n    }\n",
        ),
        (
            "crates/invoice-service/src/lib.rs",
            "    pub fn outstanding_invoices(&self) -> \
             Result<Vec<billing_types::invoice::OutstandingInvoices>, \
             billing_types::obligation::UnmetObligation> {\n        \
             self.behaviors.outstanding_invoices()\n    }\n",
        ),
    ] {
        let text = artifact(&billing_rust, path);
        assert!(text.contains(line), "`{path}` keeps\n{line}\nin\n{text}");
    }
    let gatepass_rust = rust(&gatepass);
    let surface = "crates/gatepass-server/src/pass_service.rs";
    for line in [
        "            http::answer(run_gatepass_visit_expected_visits(system))\n        }\n",
        "        \"gatepass.visit.ExpectedVisits\" => run_gatepass_visit_expected_visits(system),\n",
        "fn run_gatepass_visit_expected_visits<PassServiceBehaviors>(system: \
         &gatepass_system::System<PassServiceBehaviors>) -> Result<(u16, String), \
         entry::Refused>\n",
        "    match system.pass_service.expected_visits() {\n",
    ] {
        let text = artifact(&gatepass_rust, surface);
        assert!(text.contains(line), "`{surface}` keeps\n{line}\nin\n{text}");
    }
    let billing_go = go(&billing);
    for (path, line) in [
        (
            "types/invoice/invoice.go",
            "\tOutstandingInvoices() ([]OutstandingInvoices, *obligation.UnmetObligation)\n",
        ),
        (
            "types/invoice/invoice.go",
            "func (Unimplemented) OutstandingInvoices() ([]OutstandingInvoices, \
             *obligation.UnmetObligation) {\n",
        ),
        (
            "types/behaviour/behaviour.go",
            "func (b *Generated) OutstandingInvoices() ([]invoice.OutstandingInvoices, \
             *obligation.UnmetObligation) {\n\treturn b.ports.Owed.OutstandingInvoices()\n}\n",
        ),
        (
            "components/invoiceservice/invoiceservice.go",
            "func (c *InvoiceService) OutstandingInvoices() ([]invoice.OutstandingInvoices, \
             *obligation.UnmetObligation) {\n\treturn c.behaviors.OutstandingInvoices()\n}\n",
        ),
    ] {
        let text = artifact(&billing_go, path);
        assert!(text.contains(line), "`{path}` keeps\n{line}\nin\n{text}");
    }
    let gatepass_go = go(&gatepass);
    for line in [
        "\t\treturn serveGatepassVisitExpectedVisits(system)\n",
        "func serveGatepassVisitExpectedVisits(system *system.System) response {\n\trows, unmet \
         := system.PassService.ExpectedVisits()\n",
    ] {
        let text = artifact(&gatepass_go, "server/passservice.go");
        assert!(
            text.contains(line),
            "`server/passservice.go` keeps\n{line}\nin\n{text}"
        );
    }
}

/// A value for one bound parameter, written as its scalar is, and how the harness spells it back.
fn value_for(scalar: &str) -> (&'static str, &'static str) {
    match scalar {
        "String" => ("grace", "grace"),
        "Integer" => ("7", "7"),
        "Boolean" => ("false", "false"),
        other => panic!("the model binds no `{other}` parameter"),
    }
}

/// The route a renderer reads, from the binding an `ess-ui` document makes to the model, with each
/// parameter in the query under the key the binding names.
fn bound_target() -> (String, String) {
    let document = ess_ui::load_str(
        "format: ess-ui/1\napp: desk\nmodel: desk\nplacement_profile: fat\n\
         shells: {app: {regions: {main: {kind: page_outlet}}}}\n\
         navigation: {home: p, sections: [{name: all, pages: [p]}]}\n\
         pages: {p: {kind: detail_page, title: P, params: {who: string, hours: number, \
         urgent: boolean}, sections: [{name: tasks, component: collection, reads: {view: \
         work.ForOwner, params: {owner: params.who, min_hours: params.hours, urgent: \
         params.urgent}}}]}}\n",
    )
    .unwrap_or_else(|error| panic!("the document loads: {error}"));
    let binding = ess_ui_check::binding(&document, &sources())
        .unwrap_or_else(|error| panic!("the document binds: {error}"));
    let route = &binding.components["desk-service"].views["desk.work.ForOwner"];
    assert_eq!(
        route.path,
        view_path(),
        "the binding routes where the servers serve"
    );
    let mut query = Vec::new();
    let mut expected = Vec::new();
    for param in &route.params {
        if param.name == "level" {
            continue;
        }
        let (sent, spelled) = value_for(&param.scalar);
        query.push(format!("{}={sent}", param.wire));
        expected.push((param.name.clone(), spelled));
    }
    assert!(
        route.params.iter().any(|param| param.wire == "who"),
        "the binding carries `owner` under its wire name: {route:?}"
    );
    let spelled = |name: &str| {
        expected
            .iter()
            .find(|(bound, _)| bound == name)
            .map_or("-", |(_, spelled)| *spelled)
    };
    (
        format!("{}?{}", route.path, query.join("&")),
        format!(
            "owner={};min_hours={};urgent={};level=-",
            spelled("owner"),
            spelled("min_hours"),
            spelled("urgent")
        ),
    )
}

#[test]
fn a_param_bound_by_the_ui_binding_reaches_the_port_through_both_servers() {
    let (target, expected) = bound_target();
    for executable in [rust_server(), go_server()] {
        let server = start(executable);
        let (status, body) = get(&server, &target);
        assert_eq!(handed(status, &body), expected, "`{target}`");
    }
}

/// A query value carries one scalar, so a served view whose parameter is a struct is refused by
/// both serving targets, named at the parameter, rather than served with a parameter no request
/// can supply — the line `ess_ui_check::binding` draws for a renderer.
#[test]
fn a_non_scalar_param_of_a_served_view_is_refused_by_the_serving_targets() {
    let model = MODEL
        .replace(
            "types:\n",
            "types:\n  - {name: desk.work.Span, kind: struct, fields: [{name: least, type: Integer}]}\n",
        )
        .replace(
            "      - {name: level, type: Optional<desk.work.Level>}\n",
            "      - {name: level, type: Optional<desk.work.Level>}\n      - {name: span, type: \
             desk.work.Span}\n",
        )
        .replace(
            "level == param.level]",
            "level == param.level, hours >= param.span.least]",
        );
    let ir = ess_ui_check::compile_sources(&[("model.yaml".to_owned(), model)], Path::new("m"))
        .unwrap_or_else(|error| panic!("the model compiles: {error}"));
    for target in [Target::Rust, Target::Go] {
        match synthesize_laid_out(&ir, target, OutputLayout::Workspace) {
            Err(ess_synth::SynthesisFailure::Target(failure)) => {
                let sources: Vec<&String> = failure
                    .causes()
                    .iter()
                    .flat_map(ess_synth::TargetFailureCause::sources)
                    .collect();
                assert_eq!(
                    sources,
                    ["views.desk.work.ForOwner.params.span"],
                    "{target:?}"
                );
            }
            Err(other) => panic!("{target:?} refuses the parameter, not: {other:?}"),
            Ok(_) => panic!("{target:?} serves a struct parameter no query value carries"),
        }
    }
}
