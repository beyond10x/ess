//! The external decision must see the request actually executing, including direct calls.
//! The retained names-only baseline admitted substituted requests. These harnesses now compare
//! typed executing input while retaining the original outcome, storage and event assertions.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_gen::http::{self, Served};
use ess_synth::{
    synthesize_laid_out, OutputLayout, Synthesis, SynthesisFailure, Target, TargetFailureCode,
};

const MODEL: &str = r"format: ess/20
system: renewal
version: v1
domain: renewal.lease
types:
  - name: renewal.lease.Details
    kind: struct
    fields:
      - {name: note, type: String}
      - {name: count, type: Integer}
entities:
  - name: renewal.lease.Lease
    identity: {name: id, type: String}
    fields:
      - {name: revision, type: Integer}
      - {name: expires, type: Timestamp}
      - {name: evidence, type: String}
      - {name: units, type: Integer}
      - {name: details, type: Optional<renewal.lease.Details>}
    invariants: [revision >= 0]
    lifecycle: {initial: Active, states: [Active], terminal: [Active]}
commands:
  - name: renewal.lease.Renew
    input:
      - {name: id, type: String}
      - {name: expected_revision, type: Integer}
      - {name: expires, type: Timestamp}
      - {name: evidence, type: String}
      - {name: units, type: Integer}
      - {name: details, type: Optional<renewal.lease.Details>}
    outcomes:
      - name: invalid
        when: expected_revision < 0
        error: renewal.lease.Invalid
      - name: stale
        when_subject: {predicate: revision != input.expected_revision}
        error: renewal.lease.Stale
      - name: missing-proof
        external: No trusted request-bound decision is present.
        error: renewal.lease.MissingProof
      - name: mismatched-proof
        external: The executing request differs from the exact authorized request.
        error: renewal.lease.MismatchedProof
      - name: renewed
        updates: renewal.lease.Lease
        instance: id
        sets:
          revision: {increment: 1}
          expires: input.expires
          evidence: input.evidence
          units: input.units
          details: input.details
        emits: [renewal.lease.Renewed]
        payload:
          renewal.lease.Renewed:
            id: input.id
            expires: input.expires
            evidence: input.evidence
            units: input.units
            details: input.details
      - name: unknown
        unknown_instance: true
        error: renewal.lease.Unknown
errors:
  - name: renewal.lease.Invalid
  - name: renewal.lease.Stale
  - name: renewal.lease.MissingProof
  - name: renewal.lease.MismatchedProof
  - name: renewal.lease.Unknown
events:
  - name: renewal.lease.Renewed
    fields:
      - {name: id, type: String}
      - {name: expires, type: Timestamp}
      - {name: evidence, type: String}
      - {name: units, type: Integer}
      - {name: details, type: Optional<renewal.lease.Details>}
components:
  - component: renewal-service
    owns: {domains: [renewal.lease]}
    accepts: {commands: [renewal.lease.Renew]}
    publishes: {events: [renewal.lease.Renewed]}
    reached_by: network
";

const RUST_HARNESS: &str = r#"use std::cell::RefCell;
use std::rc::Rc;
use renewal_types::behaviour::{Context, ExternalCommand, Generated, LeaseStorage};
use renewal_types::lease::{self, obligations::RenewBehavior};
use renewal_types::primitives::Timestamp;
use renewal_server::{http, renewal_service as served, wire};

struct State {
    held: lease::LeaseSnapshot,
    proof: Option<lease::Renew>,
    calls: Vec<String>,
    writes: usize,
}
#[derive(Clone)]
struct Ports(Rc<RefCell<State>>);
impl LeaseStorage for Ports {
    fn get(&self, id: &String) -> Option<lease::LeaseSnapshot> {
        let s = self.0.borrow(); (s.held.data.id == *id).then(|| s.held.clone())
    }
    fn put(&mut self, held: lease::LeaseSnapshot) {
        let mut s = self.0.borrow_mut(); s.held = held; s.writes += 1;
    }
    fn delete(&mut self, _: &String) { panic!("renew never deletes"); }
}
impl Context for Ports {
    fn external(&mut self, command: ExternalCommand<'_>, outcome: &'static str) -> bool {
        assert_eq!(command.name(), "renewal.lease.Renew");
        let ExternalCommand::RenewalLeaseRenew(input) = command;
        let mut s = self.0.borrow_mut();
        s.calls.push(outcome.to_owned());
        match outcome {
            "missing-proof" => s.proof.is_none(),
            // Compare the actual executing input with the exact prepared proof.
            "mismatched-proof" => s.proof.as_ref() != Some(input),
            _ => panic!("unexpected external branch"),
        }
    }
}
fn request() -> lease::Renew {
    lease::Renew { id: "lease-1".to_owned(), expected_revision: 0,
        expires: Timestamp("2026-10-03T13:00:00Z".to_owned()), evidence: "receipt-A".to_owned(),
        units: 9007199254740993,
        details: Some(lease::Details { note: "approved".to_owned(), count: i64::MAX }) }
}
fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let mode = &a[0]; let case = &a[1];
    let expected = request(); let mut actual = expected.clone();
    let refused = match case.as_str() {
        "matching" => false,
        "missing" => true,
        "expires" => { actual.expires = Timestamp("2026-10-03T14:00:00Z".to_owned()); true },
        "evidence" => { actual.evidence = "receipt-B".to_owned(); true },
        "integer" => { actual.units -= 1; true },
        "nested" => { actual.details.as_mut().unwrap().count -= 1; true },
        "optional" => { actual.details = None; true },
        "invalid" => { actual.expected_revision = -1; true },
        "stale" => { actual.expected_revision = 1; true },
        "unknown" => { actual.id = "missing-lease".to_owned(); true },
        _ => panic!("unknown case"),
    };
    let before = lease::LeaseSnapshot { state: lease::LeaseState::Active,
        data: lease::LeaseData { id: expected.id.clone(), revision: 0,
            expires: Timestamp("2026-10-03T12:00:00Z".to_owned()), evidence: "old".to_owned(),
            units: 0, details: None } };
    let ports = Ports(Rc::new(RefCell::new(State { held: before.clone(),
        proof: (case != "missing").then_some(expected), calls: Vec::new(), writes: 0 })));
    let wanted = match case.as_str() {
        "missing" => "missing-proof", "invalid" => "invalid", "stale" => "stale",
        "unknown" => "unknown", _ if refused => "mismatched-proof", _ => "renewed",
    };
    if mode == "direct" {
        let result = Generated::new(ports.clone()).renew(actual.clone()).expect("declared result");
        let (outcome, events) = match result {
            lease::RenewOutcome::Renewed { renewed } => {
                assert_eq!(renewed.id, actual.id); assert_eq!(renewed.expires, actual.expires);
                assert_eq!(renewed.evidence, actual.evidence); assert_eq!(renewed.units, actual.units);
                assert_eq!(renewed.details, actual.details); ("renewed", 1)
            },
            lease::RenewOutcome::MissingProof { .. } => ("missing-proof", 0),
            lease::RenewOutcome::MismatchedProof { .. } => ("mismatched-proof", 0),
            lease::RenewOutcome::Invalid { .. } => ("invalid", 0),
            lease::RenewOutcome::Stale { .. } => ("stale", 0),
            lease::RenewOutcome::Unknown { .. } => ("unknown", 0),
        };
        assert_eq!(outcome, wanted, "exact request-bound outcome");
        assert_eq!(events, usize::from(!refused), "refusal publishes no event");
    } else {
        let service = renewal_service::RenewalService::new(Generated::new(ports.clone()));
        let mut system = renewal_system::System::new(service);
        let mut input = String::new();
        wire::encode_command_renewal_lease_renew(&actual, &mut input);
        let response = served::dispatch(&mut system, &http::Request {
            method: a[2].clone(), path: a[3].clone(), query: String::new(), headers: Vec::new(),
            body: input.into_bytes(),
        });
        assert!(response.body.contains(&format!("\"outcome\":\"{wanted}\"")), "served outcome: {}", response.body);
        if refused { assert!(!response.body.contains("renewal.lease.Renewed"), "refusal publishes no event"); }
        else { assert!(response.body.contains("renewal.lease.Renewed")); }
        assert!(system.published().is_empty(), "dispatch drains published events");
    }
    let s = ports.0.borrow();
    if refused { assert_eq!(s.held, before, "refusal preserves the complete stored row"); assert_eq!(s.writes, 0); }
    else { assert_eq!(s.held.data.revision, 1); assert_eq!(s.held.data.expires, actual.expires);
        assert_eq!(s.held.data.evidence, actual.evidence); assert_eq!(s.held.data.units, actual.units);
        assert_eq!(s.held.data.details, actual.details); assert_eq!(s.writes, 1); }
    let expected_calls: Vec<&str> = match case.as_str() {
        "invalid" | "stale" | "unknown" => vec![], "missing" => vec!["missing-proof"],
        _ => vec!["missing-proof", "mismatched-proof"],
    };
    assert_eq!(s.calls, expected_calls, "local guards precede ordered external decisions");
    println!("PASS {mode} {case}");
}
"#;

fn model() -> EssIr {
    compile_model(MODEL)
}

fn compile_model(text: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("binding.yaml"),
        RawSpecFile::parse(text).expect("well formed model"),
    )])
    .unwrap_or_else(|errors| panic!("model validates: {errors}"));
    let mut sources = SourceMap::new();
    sources.insert("binding.yaml", text);
    compile(&spec, &sources).unwrap_or_else(|errors| panic!("model compiles: {errors}"))
}

fn write(synthesis: &Synthesis, directory: &Path) {
    for (relative, artifact) in &synthesis.artifacts {
        let file = directory.join(relative);
        std::fs::create_dir_all(file.parent().expect("parent")).expect("mkdir");
        std::fs::write(file, &artifact.contents).expect("write artifact");
    }
}

fn run_cargo(directory: &Path, target: &Path) -> Output {
    Command::new(std::env::var_os("CARGO").expect("cargo executable"))
        .args(["build", "--offline", "--quiet", "--target-dir"])
        .arg(target)
        .current_dir(directory)
        .env_remove("CARGO_TARGET_DIR")
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .env("RUSTFLAGS", "-D warnings")
        .env("CARGO_BUILD_JOBS", "1")
        .env("CARGO_INCREMENTAL", "0")
        .output()
        .expect("cargo runs")
}

fn rust_case(layout: OutputLayout, label: &str, cases: &[&str]) {
    rust_case_with_source(layout, label, cases, MODEL, RUST_HARNESS);
}

fn rust_case_with_source(
    layout: OutputLayout,
    label: &str,
    cases: &[&str],
    model: &str,
    source: &str,
) {
    let ir = compile_model(model);
    let generated = synthesize_laid_out(&ir, Target::Rust, layout).expect("Rust generation");
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("external-binding-{label}-{}", std::process::id()));
    std::fs::create_dir(&root).expect("fresh owned fixture root");
    write(&generated, &root.join("renewal"));
    let harness = root.join("harness");
    std::fs::create_dir_all(harness.join("src")).expect("mkdir");
    let (deps, source) = match layout {
        OutputLayout::Crate => (
            "renewal = { path = \"../renewal\", features = [\"server\"] }\n".to_owned(),
            source
                .replace("renewal_server::", "renewal::server::")
                .replace("renewal_types::", "renewal::")
                .replace("renewal_system::", "renewal::system::")
                .replace(
                    "renewal_service::RenewalService",
                    "renewal::ports::renewal_service::RenewalService",
                ),
        ),
        OutputLayout::Workspace => (
            [
                "renewal-types",
                "renewal-server",
                "renewal-system",
                "renewal-service",
            ]
            .iter()
            .fold(String::new(), |mut deps, name| {
                let _ = writeln!(deps, "{name} = {{ path = \"../renewal/crates/{name}\" }}");
                deps
            }),
            source.to_owned(),
        ),
    };
    std::fs::write(harness.join("Cargo.toml"), format!(
        "[package]\nname = \"external-binding-harness\"\nversion = \"0.0.0\"\nedition = \"2021\"\n[dependencies]\n{deps}\n[workspace]\n"
    )).expect("manifest");
    std::fs::write(harness.join("src/main.rs"), source).expect("harness");
    let target = root.join("target");
    let compiled = run_cargo(&harness, &target);
    std::fs::write(root.join("compile.stderr"), &compiled.stderr).expect("retain compile output");
    assert!(
        compiled.status.success(),
        "harness must compile; not semantic RED: {} (retained {})",
        String::from_utf8_lossy(&compiled.stderr),
        root.display()
    );
    let component = ir.components().values().next().expect("component");
    let route = http::routes(&ir, component)
        .into_iter()
        .find(|r| matches!(r.serves, Served::Command(_)))
        .expect("command route");
    for mode in ["direct", "http"] {
        for case in cases {
            let output = Command::new(target.join("debug/external-binding-harness"))
                .args([mode, case, route.method.as_str(), &route.path])
                .output()
                .expect("harness runs");
            std::fs::write(root.join(format!("{mode}-{case}.stderr")), &output.stderr)
                .expect("retain runtime evidence");
            assert!(output.status.success(), "semantic request-binding assertion failed ({mode}/{case}); fixture retained at {}:\n{}",
                root.display(), String::from_utf8_lossy(&output.stderr));
        }
    }
    // Keep generated source and compiler/runtime evidence for review. The coordinator
    // retires only this task's proven-idle targets after the final evidence is frozen.
}

#[test]
fn matching_and_missing_proof_controls_compile_and_execute() {
    rust_case(
        OutputLayout::Crate,
        "controls",
        &["matching", "missing", "invalid", "stale", "unknown"],
    );
}

#[test]
fn substituted_request_is_refused_by_direct_and_http_crate() {
    rust_case(
        OutputLayout::Crate,
        "crate",
        &["expires", "evidence", "integer", "nested", "optional"],
    );
}

#[test]
fn substituted_request_is_refused_by_direct_and_http_workspace() {
    rust_case(
        OutputLayout::Workspace,
        "workspace",
        &["expires", "evidence", "integer", "nested", "optional"],
    );
}

const GO_HARNESS: &str = r#"package main

import (
    "bufio"
    "bytes"
    "encoding/json"
    "fmt"
    "io"
    "math"
    "net/http"
    "os"
    "reflect"
    "time"
    "example.invalid/renewal/components/renewalservice"
    "example.invalid/renewal/server"
    "example.invalid/renewal/system"
    "example.invalid/renewal/types/behaviour"
    "example.invalid/renewal/types/lease"
    "example.invalid/renewal/types/primitives"
)

type ports struct {
    held lease.LeaseSnapshot
    proof *lease.Renew
    calls []string
    writes int
}
func (p *ports) Get(id string) (lease.LeaseSnapshot, bool) { return p.held, id == p.held.Data.Id }
func (p *ports) Put(row lease.LeaseSnapshot) { p.held = row; p.writes++ }
func (p *ports) Delete(_ string) { panic("renew never deletes") }
func (p *ports) External(command behaviour.ExternalCommand, outcome string) bool {
    if command.Name() != "renewal.lease.Renew" { panic("wrong command") }
    input, ok := command.(behaviour.ExternalCommandRenewalLeaseRenew)
    if !ok { panic("wrong typed command") }
    p.calls = append(p.calls, outcome)
    switch outcome {
    case "missing-proof": return p.proof == nil
    // Compare every typed field; no JSON conversion or integer rounding.
    case "mismatched-proof": return p.proof == nil || !reflect.DeepEqual(*p.proof, input.Input)
    default: panic("unexpected external branch")
    }
}
func request() lease.Renew {
    return lease.Renew{Id: "lease-1", ExpectedRevision: 0,
        Expires: primitives.NewTimestamp("2026-10-03T13:00:00Z"), Evidence: "receipt-A",
        Units: 9007199254740993, Details: &lease.Details{Note: "approved", Count: math.MaxInt64}}
}
func require(ok bool, message string) { if !ok { panic(message) } }
func main() {
    mode, vector := os.Args[1], os.Args[2]
    expected, actual := request(), request() // independent nested values, no shared proof alias
    refused := true
    switch vector {
    case "matching": refused = false
    case "missing":
    case "expires": actual.Expires = primitives.NewTimestamp("2026-10-03T14:00:00Z")
    case "evidence": actual.Evidence = "receipt-B"
    case "integer": actual.Units--
    case "nested": actual.Details.Count--
    case "optional": actual.Details = nil
    case "invalid": actual.ExpectedRevision = -1
    case "stale": actual.ExpectedRevision = 1
    case "unknown": actual.Id = "missing-lease"
    default: panic("unknown vector")
    }
    before := lease.LeaseSnapshot{State: lease.LeaseStateActive{}, Data: lease.LeaseData{
        Id: "lease-1", Revision: 0, Expires: primitives.NewTimestamp("2026-10-03T12:00:00Z"),
        Evidence: "old", Units: 0, Details: nil}}
    p := &ports{held: before, proof: &expected}
    if vector == "missing" { p.proof = nil }
    generated := behaviour.New(behaviour.Ports{LeaseStorage: p, Context: p})
    wanted := "mismatched-proof"
    switch vector {
    case "matching": wanted = "renewed"
    case "missing": wanted = "missing-proof"
    case "invalid", "stale", "unknown": wanted = vector
    }
    events, outcome := 0, ""
    if mode == "direct" {
        result, err := generated.Renew(actual)
        require(err == nil, "declared result")
        switch value := result.(type) {
        case lease.RenewOutcomeRenewed:
            outcome, events = "renewed", 1
            require(value.Renewed.Id == actual.Id && value.Renewed.Expires == actual.Expires &&
                value.Renewed.Evidence == actual.Evidence && value.Renewed.Units == actual.Units &&
                reflect.DeepEqual(value.Renewed.Details, actual.Details), "event exact typed input")
        case lease.RenewOutcomeMissingProof: outcome = "missing-proof"
        case lease.RenewOutcomeMismatchedProof: outcome = "mismatched-proof"
        case lease.RenewOutcomeInvalid: outcome = "invalid"
        case lease.RenewOutcomeStale: outcome = "stale"
        case lease.RenewOutcomeUnknown: outcome = "unknown"
        default: panic("unexpected outcome type")
        }
    } else {
        sys := system.NewSystem(renewalservice.New(generated))
        reader, writer, err := os.Pipe(); require(err == nil, "startup pipe")
        require(reader.SetReadDeadline(time.Now().Add(5*time.Second)) == nil, "bounded startup")
        original := os.Stdout; os.Stdout = writer
        go func() { _ = server.ServeRenewalService(sys, "127.0.0.1:0") }()
        lines := bufio.NewReader(reader); port := 0
        for {
            line, err := lines.ReadBytes('\n'); require(err == nil, "startup record")
            var record struct { Event string `json:"event"`; Runtime struct { Port int `json:"port"` } `json:"runtime"` }
            require(json.Unmarshal(line, &record) == nil, "startup JSON")
            if record.Event == "system.ready" { port = record.Runtime.Port; break }
        }
        os.Stdout = original; _ = reader.Close(); _ = writer.Close()
        var details any
        if actual.Details != nil { details = map[string]any{"note": actual.Details.Note, "count": actual.Details.Count} }
        body, err := json.Marshal(map[string]any{"id": actual.Id, "expected_revision": actual.ExpectedRevision,
            "expires": actual.Expires.Value(), "evidence": actual.Evidence, "units": actual.Units, "details": details})
        require(err == nil, "input JSON")
        req, err := http.NewRequest(os.Args[3], fmt.Sprintf("http://127.0.0.1:%d%s", port, os.Args[4]), bytes.NewReader(body))
        require(err == nil, "request")
        req.Header.Set("Content-Type", "application/json")
        response, err := (&http.Client{Timeout: 5*time.Second}).Do(req); require(err == nil, "served request")
        raw, err := io.ReadAll(response.Body); _ = response.Body.Close(); require(err == nil, "served body")
        var answer struct { Outcome string `json:"outcome"`; Published []json.RawMessage `json:"published"` }
        require(json.Unmarshal(raw, &answer) == nil, "answer JSON")
        outcome, events = answer.Outcome, len(answer.Published)
    }
    require(outcome == wanted, fmt.Sprintf("exact request-bound outcome: got %s want %s", outcome, wanted))
    if refused {
        require(events == 0, "refusal publishes no event")
        require(reflect.DeepEqual(p.held, before) && p.writes == 0, "refusal preserves complete stored row")
    } else {
        require(events == 1 && p.writes == 1 && p.held.Data.Revision == 1, "accepted write and event")
        require(p.held.Data.Expires == actual.Expires && p.held.Data.Evidence == actual.Evidence &&
            p.held.Data.Units == actual.Units && reflect.DeepEqual(p.held.Data.Details, actual.Details), "stored exact input")
    }
    var calls []string
    switch vector { case "invalid", "stale", "unknown":
    case "missing": calls = []string{"missing-proof"}
    default: calls = []string{"missing-proof", "mismatched-proof"} }
    require(reflect.DeepEqual(p.calls, calls), "local guards precede ordered external decisions")
    fmt.Println("PASS", mode, vector)
}
"#;

fn go_case(label: &str, cases: &[&str]) {
    go_case_with_source(label, cases, MODEL, GO_HARNESS);
}

fn go_case_with_source(label: &str, cases: &[&str], model: &str, source: &str) {
    let ir = compile_model(model);
    let generated =
        synthesize_laid_out(&ir, Target::Go, OutputLayout::Workspace).expect("Go generation");
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "external-binding-go-{label}-{}",
        std::process::id()
    ));
    std::fs::create_dir(&root).expect("fresh owned fixture root");
    write(&generated, &root.join("renewal"));
    let harness = root.join("harness");
    std::fs::create_dir_all(&harness).expect("mkdir");
    std::fs::write(harness.join("go.mod"), "module renewalharness\n\ngo 1.21\n\nrequire example.invalid/renewal v0.0.0\n\nreplace example.invalid/renewal => ../renewal\n").expect("Go manifest");
    std::fs::write(harness.join("main.go"), source).expect("Go harness");
    let binary = root.join("go-harness");
    let compiled = Command::new("go")
        .args(["build", "-o"])
        .arg(&binary)
        .arg(".")
        .current_dir(&harness)
        .env("GOFLAGS", "-mod=mod -p=1 -buildvcs=false")
        .env("GOMAXPROCS", "1")
        .env("GOPROXY", "off")
        .env("GOWORK", "off")
        .env("GOCACHE", root.join("go-cache"))
        .output()
        .expect("required Go toolchain runs");
    std::fs::write(root.join("compile.stderr"), &compiled.stderr)
        .expect("retain Go compiler output");
    assert!(
        compiled.status.success(),
        "Go harness must compile; not semantic RED: {} (retained {})",
        String::from_utf8_lossy(&compiled.stderr),
        root.display()
    );
    let component = ir.components().values().next().expect("component");
    let route = http::routes(&ir, component)
        .into_iter()
        .find(|r| matches!(r.serves, Served::Command(_)))
        .expect("command route");
    for mode in ["direct", "http"] {
        for case in cases {
            let output = Command::new(&binary)
                .args([mode, case, route.method.as_str(), &route.path])
                .output()
                .expect("Go harness runs");
            std::fs::write(root.join(format!("{mode}-{case}.stderr")), &output.stderr)
                .expect("retain Go runtime evidence");
            assert!(output.status.success(), "semantic Go request-binding assertion failed ({mode}/{case}); fixture retained at {}:\n{}",
                root.display(), String::from_utf8_lossy(&output.stderr));
        }
    }
}

#[test]
fn go_matching_and_missing_proof_controls_compile_and_execute() {
    go_case(
        "controls",
        &["matching", "missing", "invalid", "stale", "unknown"],
    );
}

#[test]
fn go_substituted_request_is_refused_by_direct_and_http() {
    go_case(
        "substitution",
        &["expires", "evidence", "integer", "nested", "optional"],
    );
}

// The three qualified command names deliberately flatten to two candidates. Naming.code keeps
// their payload types and Go method names distinct; it must not change canonical identities.
const COLLISIONS: &str = r"format: ess/20
system: renewal
version: v1
domain: renewal.input
commands:
  - name: renewal.input.AB
    naming: {code: First}
    input: [{name: value, type: Integer}]
    outcomes:
      - {name: refused, external: trusted decision, error: renewal.input.Refused}
      - {name: accepted, emits: [renewal.input.Accepted]}
  - name: renewal.input.AB2
    naming: {code: Third}
    input: [{name: value, type: Integer}]
    outcomes:
      - {name: refused, external: trusted decision, error: renewal.input.Refused}
      - {name: accepted, emits: [renewal.input.Accepted]}
  - name: renewal.input.A_B
    naming: {code: Second}
    input: [{name: value, type: Integer}]
    outcomes:
      - {name: refused, external: trusted decision, error: renewal.input.Refused}
      - {name: accepted, emits: [renewal.input.Accepted]}
events:
  - name: renewal.input.Accepted
errors:
  - name: renewal.input.Refused
components:
  - component: renewal-service
    owns: {domains: [renewal.input]}
    accepts: {commands: [renewal.input.AB, renewal.input.AB2, renewal.input.A_B]}
    publishes: {events: [renewal.input.Accepted]}
    reached_by: network
";

const UNUSED: &str = r"format: ess/20
system: renewal
version: v1
domain: renewal.input
commands:
  - name: renewal.input.Plain
    input: []
    outcomes: [{name: accepted, emits: [renewal.input.Accepted]}]
  - name: renewal.input.Owed
    input: []
    outcomes:
      - {name: refused, external: trusted decision, error: renewal.input.Unfilled}
      - {name: accepted, emits: [renewal.input.Accepted]}
events:
  - name: renewal.input.Accepted
errors:
  - name: renewal.input.Unfilled
    fields: [{name: explanation, type: String}]
components:
  - component: renewal-service
    owns: {domains: [renewal.input]}
    accepts: {commands: [renewal.input.Plain, renewal.input.Owed]}
    publishes: {events: [renewal.input.Accepted]}
    reached_by: network
";

fn fresh_root(label: &str) -> PathBuf {
    let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("external-binding-{label}-{}", std::process::id()));
    std::fs::create_dir(&path).expect("fresh owned fixture root");
    path
}

fn go_build(directory: &Path, cache: &Path, args: &[&str]) -> Output {
    Command::new("go")
        .arg("build")
        .args(args)
        .current_dir(directory)
        .env("GOFLAGS", "-mod=mod -p=1 -buildvcs=false")
        .env("GOMAXPROCS", "1")
        .env("GOPROXY", "off")
        .env("GOWORK", "off")
        .env("GOCACHE", cache)
        .output()
        .expect("required Go toolchain runs")
}

fn behaviour_source(synthesis: &Synthesis, target: Target, layout: OutputLayout) -> &str {
    let path = match (target, layout) {
        (Target::Rust, OutputLayout::Crate) => "src/behaviour.rs",
        (Target::Rust, OutputLayout::Workspace) => "crates/renewal-types/src/behaviour.rs",
        (Target::Go, _) => "types/behaviour/behaviour.go",
        _ => panic!("test target"),
    };
    &synthesis.artifacts[path].contents
}

fn assert_repeatable(ir: &EssIr, target: Target, layout: OutputLayout, first: &Synthesis) {
    let second = synthesize_laid_out(ir, target, layout).expect("repeat generation");
    assert_eq!(first.artifacts.len(), second.artifacts.len());
    for (path, artifact) in &first.artifacts {
        assert_eq!(artifact.contents, second.artifacts[path].contents, "{path}");
    }
}

#[test]
fn collision_alias_and_input_local_contracts_compile_in_both_targets() {
    // Embedded/direct is a supported profile; the unchanged network model is refused below.
    let ir = compile_model(&COLLISIONS.replace("    reached_by: network\n", ""));
    let root = fresh_root("collisions");
    for (target, layout, label) in [
        (Target::Rust, OutputLayout::Crate, "rust-crate"),
        (Target::Rust, OutputLayout::Workspace, "rust-workspace"),
        (Target::Go, OutputLayout::Workspace, "go"),
    ] {
        let generated =
            synthesize_laid_out(&ir, target, layout).expect("collision model generates");
        assert_repeatable(&ir, target, layout, &generated);
        let code = behaviour_source(&generated, target, layout);
        for (variant, payload, canonical) in [
            ("RenewalInputAB", "First", "renewal.input.AB"),
            ("RenewalInputAB2", "Third", "renewal.input.AB2"),
            ("RenewalInputAB3", "Second", "renewal.input.A_B"),
        ] {
            if target == Target::Rust {
                assert!(
                    code.contains(&format!("{variant}(&'a crate::input::{payload})")),
                    "{code}"
                );
                assert!(
                    code.contains(&format!("Self::{variant}(_) => \"{canonical}\"")),
                    "{code}"
                );
                assert!(
                    code.contains(&format!("ExternalCommand::{variant}(&input)")),
                    "{code}"
                );
            } else {
                assert!(
                    code.contains(&format!("type ExternalCommand{variant} struct")),
                    "{code}"
                );
                assert!(code.contains(&format!("Input input.{payload}")), "{code}");
                assert!(code.contains(&format!("return \"{canonical}\"")), "{code}");
                // `input` is a package name: the allocated local is `input_`, not `input`.
                assert!(
                    code.contains(&format!("ExternalCommand{variant}{{Input: input_}}")),
                    "{code}"
                );
            }
        }
        let directory = root.join(label).join("generated");
        write(&generated, &directory);
        let built = if target == Target::Rust {
            run_cargo(&directory, &directory.join("target"))
        } else {
            go_build(&directory, &root.join("go-cache"), &["./..."])
        };
        std::fs::write(directory.join("compile.stderr"), &built.stderr)
            .expect("retain compiler output");
        assert!(
            built.status.success(),
            "{label}: {}",
            String::from_utf8_lossy(&built.stderr)
        );
        run_collision_harness(&directory, &root.join("go-cache"), target, layout);
    }
}

#[test]
fn no_use_and_owed_only_models_compile_without_an_external_port() {
    let root = fresh_root("unused");
    // The second model still declares external, but the entire command is owed because the
    // error payload has no source. Plain is generated so the behaviour module is exercised.
    for (label, text) in [
        (
            "no-use",
            UNUSED.replace("      - {name: refused, external: trusted decision, error: renewal.input.Unfilled}\n", ""),
        ),
        ("owed-only", UNUSED.to_owned()),
    ] {
        let ir = compile_model(&text);
        for (target, layout, suffix) in [
            (Target::Rust, OutputLayout::Crate, "rust-crate"),
            (Target::Rust, OutputLayout::Workspace, "rust-workspace"),
            (Target::Go, OutputLayout::Workspace, "go"),
        ] {
            let generated =
                synthesize_laid_out(&ir, target, layout).expect("unused model generates");
            let code = behaviour_source(&generated, target, layout);
            assert!(!code.contains("ExternalCommand"), "{label}: {code}");
            assert!(
                !code.contains("fn external(") && !code.contains("External(command"),
                "{label}: {code}"
            );
            let directory = root.join(format!("{label}-{suffix}"));
            write(&generated, &directory);
            let built = if target == Target::Rust {
                run_cargo(&directory, &directory.join("target"))
            } else {
                go_build(&directory, &root.join("go-cache"), &["./..."])
            };
            std::fs::write(directory.join("compile.stderr"), &built.stderr)
                .expect("retain compiler output");
            assert!(
                built.status.success(),
                "{label}/{suffix}: {}",
                String::from_utf8_lossy(&built.stderr)
            );
        }
    }
}

#[test]
fn legacy_names_only_contexts_require_explicit_migration() {
    let root = fresh_root("legacy");
    let ir = model();
    let generated =
        synthesize_laid_out(&ir, Target::Rust, OutputLayout::Crate).expect("Rust generation");
    write(&generated, &root.join("renewal"));
    let harness = root.join("rust-harness");
    std::fs::create_dir_all(harness.join("src")).expect("mkdir");
    std::fs::write(harness.join("Cargo.toml"), "[package]\nname=\"legacy-context\"\nversion=\"0.0.0\"\nedition=\"2021\"\n[dependencies]\nrenewal={path=\"../renewal\"}\n[workspace]\n").expect("manifest");
    std::fs::write(harness.join("src/main.rs"), "pub struct Old;\nimpl renewal::behaviour::Context for Old {\nfn external(&mut self, _: &'static str, _: &'static str) -> bool { false }\n}\nfn main() {}\n").expect("legacy harness");
    let built = run_cargo(&harness, &root.join("rust-target"));
    std::fs::write(root.join("legacy-rust.stderr"), &built.stderr)
        .expect("retain expected failure");
    let stderr = String::from_utf8_lossy(&built.stderr);
    assert!(
        !built.status.success() && stderr.contains("E0053") && stderr.contains("ExternalCommand"),
        "expected signature mismatch, not arbitrary failure: {stderr}"
    );
    let generated =
        synthesize_laid_out(&ir, Target::Go, OutputLayout::Workspace).expect("Go generation");
    write(&generated, &root.join("go-renewal"));
    let harness = root.join("go-harness");
    std::fs::create_dir(&harness).expect("mkdir");
    std::fs::write(harness.join("go.mod"), "module legacyharness\n\ngo 1.21\nrequire example.invalid/renewal v0.0.0\nreplace example.invalid/renewal => ../go-renewal\n").expect("Go manifest");
    std::fs::write(harness.join("main.go"), "package main\nimport \"example.invalid/renewal/types/behaviour\"\ntype old struct{}\nfunc(old) External(string,string) bool {return false}\nvar _ behaviour.Context = old{}\nfunc main(){}\n").expect("Go legacy harness");
    let built = go_build(&harness, &root.join("go-cache"), &["."]);
    std::fs::write(root.join("legacy-go.stderr"), &built.stderr).expect("retain expected failure");
    let stderr = String::from_utf8_lossy(&built.stderr);
    assert!(
        !built.status.success()
            && stderr.contains("wrong type for method External")
            && stderr.contains("ExternalCommand"),
        "expected signature mismatch, not arbitrary failure: {stderr}"
    );
}

#[test]
fn served_colliding_codecs_remain_an_explicit_typed_refusal() {
    let ir = compile_model(COLLISIONS); // Exact original network fixture, names and aliases.
    for layout in [OutputLayout::Crate, OutputLayout::Workspace] {
        let Err(SynthesisFailure::Target(failure)) = synthesize_laid_out(&ir, Target::Rust, layout)
        else {
            panic!("served codec collisions must not be silently admitted");
        };
        assert_eq!(failure.causes().len(), 3);
        for (cause, codec) in failure.causes().iter().zip([
            "decode_command_renewal_input_a_b",
            "encode_command_renewal_input_a_b",
            "encode_outcome_renewal_input_a_b",
        ]) {
            assert_eq!(cause.code(), TargetFailureCode::WireCollision);
            assert_eq!(cause.sources(), ["renewal.input.AB", "renewal.input.A_B"]);
            assert!(cause.detail().contains(codec), "{}", cause.detail());
        }
    }
}

const COLLISION_RUST: &str = r#"use renewal_types::behaviour::{Context, ExternalCommand, Generated};
use renewal_types::input::{self, obligations::{FirstBehavior, SecondBehavior, ThirdBehavior}};
#[derive(Default)]
struct Ports { calls: Vec<(&'static str, i64)> }
impl Context for Ports {
    fn external(&mut self, command: ExternalCommand<'_>, outcome: &'static str) -> bool {
        assert_eq!(outcome, "refused");
        let (expected, value) = match command {
            ExternalCommand::RenewalInputAB(input) => ("renewal.input.AB", input.value),
            ExternalCommand::RenewalInputAB2(input) => ("renewal.input.AB2", input.value),
            ExternalCommand::RenewalInputAB3(input) => ("renewal.input.A_B", input.value),
        };
        assert_eq!(command.name(), expected);
        self.calls.push((command.name(), value));
        value < 0
    }
}
fn main() {
    let mut generated = Generated::new(Ports::default());
    let value = 9007199254740993;
    assert!(matches!(generated.first(input::First { value }), Ok(input::FirstOutcome::Accepted { .. })));
    assert!(matches!(generated.third(input::Third { value: value + 1 }), Ok(input::ThirdOutcome::Accepted { .. })));
    assert!(matches!(generated.second(input::Second { value: value + 2 }), Ok(input::SecondOutcome::Accepted { .. })));
    assert!(matches!(generated.first(input::First { value: -1 }), Ok(input::FirstOutcome::Refused { .. })));
    assert!(matches!(generated.third(input::Third { value: -2 }), Ok(input::ThirdOutcome::Refused { .. })));
    assert!(matches!(generated.second(input::Second { value: -3 }), Ok(input::SecondOutcome::Refused { .. })));
    assert_eq!(generated.ports.calls, vec![
        ("renewal.input.AB", value), ("renewal.input.AB2", value + 1), ("renewal.input.A_B", value + 2),
        ("renewal.input.AB", -1), ("renewal.input.AB2", -2), ("renewal.input.A_B", -3),
    ]);
}
"#;

const COLLISION_GO: &str = r#"package main
import (
    "reflect"
    "example.invalid/renewal/types/behaviour"
    "example.invalid/renewal/types/input"
)
type call struct { name string; value int64 }
type ports struct { calls []call }
func (p *ports) External(command behaviour.ExternalCommand, outcome string) bool {
    if outcome != "refused" { panic("wrong branch") }
    var expected string; var value int64
    switch current := command.(type) {
    case behaviour.ExternalCommandRenewalInputAB: expected, value = "renewal.input.AB", current.Input.Value
    case behaviour.ExternalCommandRenewalInputAB2: expected, value = "renewal.input.AB2", current.Input.Value
    case behaviour.ExternalCommandRenewalInputAB3: expected, value = "renewal.input.A_B", current.Input.Value
    default: panic("unknown typed request")
    }
    if command.Name() != expected { panic("canonical identity changed") }
    p.calls = append(p.calls, call{command.Name(), value})
    return value < 0
}
func main() {
    p := &ports{}; generated := behaviour.New(behaviour.Ports{Context:p})
    value := int64(9007199254740993)
    first, err := generated.First(input.First{Value:value}); if err != nil { panic(err) }
    if _, ok := first.(input.FirstOutcomeAccepted); !ok { panic("first accept") }
    third, err := generated.Third(input.Third{Value:value+1}); if err != nil { panic(err) }
    if _, ok := third.(input.ThirdOutcomeAccepted); !ok { panic("third accept") }
    second, err := generated.Second(input.Second{Value:value+2}); if err != nil { panic(err) }
    if _, ok := second.(input.SecondOutcomeAccepted); !ok { panic("second accept") }
    first, err = generated.First(input.First{Value:-1}); if err != nil { panic(err) }
    if _, ok := first.(input.FirstOutcomeRefused); !ok { panic("first refusal") }
    third, err = generated.Third(input.Third{Value:-2}); if err != nil { panic(err) }
    if _, ok := third.(input.ThirdOutcomeRefused); !ok { panic("third refusal") }
    second, err = generated.Second(input.Second{Value:-3}); if err != nil { panic(err) }
    if _, ok := second.(input.SecondOutcomeRefused); !ok { panic("second refusal") }
    want := []call{{"renewal.input.AB",value},{"renewal.input.AB2",value+1},{"renewal.input.A_B",value+2},
        {"renewal.input.AB",-1},{"renewal.input.AB2",-2},{"renewal.input.A_B",-3}}
    if !reflect.DeepEqual(p.calls,want) { panic("typed callback calls changed") }
}
"#;

fn run_collision_harness(directory: &Path, cache: &Path, target: Target, layout: OutputLayout) {
    let harness = directory.parent().expect("fixture parent").join("harness");
    std::fs::create_dir_all(harness.join("src")).expect("mkdir");
    let (built, binary) = if target == Target::Rust {
        let (dependency, source) = if layout == OutputLayout::Crate {
            (
                "renewal={path=\"../generated\"}",
                COLLISION_RUST.replace("renewal_types::", "renewal::"),
            )
        } else {
            (
                "renewal-types={path=\"../generated/crates/renewal-types\"}",
                COLLISION_RUST.to_owned(),
            )
        };
        std::fs::write(harness.join("Cargo.toml"), format!("[package]\nname=\"collision-harness\"\nversion=\"0.0.0\"\nedition=\"2021\"\n[dependencies]\n{dependency}\n[workspace]\n")).expect("manifest");
        std::fs::write(harness.join("src/main.rs"), source).expect("harness");
        let build = directory.join("target");
        (
            run_cargo(&harness, &build),
            build.join("debug/collision-harness"),
        )
    } else {
        std::fs::write(harness.join("go.mod"), "module collisionharness\n\ngo 1.21\nrequire example.invalid/renewal v0.0.0\nreplace example.invalid/renewal => ../generated\n").expect("manifest");
        std::fs::write(harness.join("main.go"), COLLISION_GO).expect("harness");
        let binary = harness.join("collision-harness");
        (
            go_build(
                &harness,
                cache,
                &["-o", binary.to_str().expect("test path"), "."],
            ),
            binary,
        )
    };
    std::fs::write(harness.join("compile.stderr"), &built.stderr).expect("retain compile output");
    assert!(
        built.status.success(),
        "{}",
        String::from_utf8_lossy(&built.stderr)
    );
    let result = Command::new(&binary)
        .output()
        .expect("collision harness runs");
    std::fs::write(harness.join("runtime.stderr"), &result.stderr).expect("retain runtime output");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
}

#[test]
fn served_code_alias_preserves_direct_and_http_request_binding() {
    let model = MODEL.replace(
        "  - name: renewal.lease.Renew\n    input:",
        "  - name: renewal.lease.Renew\n    naming: {code: Extend}\n    input:",
    );
    assert_ne!(model, MODEL);
    let rust = RUST_HARNESS
        .replace("RenewBehavior", "ExtendBehavior")
        .replace("lease::Renew", "lease::Extend")
        .replace("lease::Extended", "lease::Renewed")
        .replace(".renew(", ".extend(");
    let go = GO_HARNESS
        .replace("lease.Renew", "lease.Extend")
        .replace("lease.Extended", "lease.Renewed")
        .replace("\"renewal.lease.Extend\"", "\"renewal.lease.Renew\"")
        .replace(".Renew(", ".Extend(");
    let cases = [
        "matching", "missing", "expires", "evidence", "integer", "nested", "optional", "invalid",
        "stale", "unknown",
    ];
    rust_case_with_source(OutputLayout::Crate, "alias-crate", &cases, &model, &rust);
    rust_case_with_source(
        OutputLayout::Workspace,
        "alias-workspace",
        &cases,
        &model,
        &rust,
    );
    go_case_with_source("alias", &cases, &model, &go);
}

const PORT_MODEL: &str = r"format: ess/20
system: renewal
version: v1
domain: renewal.DOMAIN
commands:
  - name: renewal.DOMAIN.Send
    input: [{name: value, type: Integer}]
    outcomes:
      - {name: refused, when: value < 0, error: renewal.DOMAIN.Refused}
      - name: accepted
        emits: [renewal.DOMAIN.Sent]
        payload:
          renewal.DOMAIN.Sent: {value: input.value}
events:
  - name: renewal.DOMAIN.Sent
    fields: [{name: value, type: Integer}]
errors:
  - name: renewal.DOMAIN.Refused
components:
  - component: renewal-service
    owns: {domains: [renewal.DOMAIN]}
    accepts: {commands: [renewal.DOMAIN.Send]}
    publishes: {events: [renewal.DOMAIN.Sent]}
";

const PORT_GO: &str = r#"package main
import (
    "example.invalid/renewal/components/renewalservice"
    "example.invalid/renewal/types/behaviour"
    held "example.invalid/renewal/types/DOMAIN"
    "example.invalid/renewal/types/obligation"
)
type refused struct{}
func (refused) Send(held.Send) (held.SendOutcome, *obligation.UnmetObligation) {
    return nil, &obligation.UnmetObligation{Capability:"test refusal", Source:"exact sentinel"}
}
func main() {
    service := renewalservice.New(behaviour.New(behaviour.Ports{}))
    value := int64(9007199254740993)
    answer, err := service.Send(held.Send{Value:value})
    if err != nil { panic(err) }
    accepted, ok := answer.(held.SendOutcomeAccepted)
    if !ok || accepted.Sent.Value != value { panic("handler changed exact input/outcome") }
    events := service.DrainOutbox()
    if len(events) != 1 { panic("one published occurrence") }
    event, ok := events[0].(renewalservice.PublishedEventSent)
    if !ok || event.Event.Value != value { panic("event payload changed") }
    if len(service.DrainOutbox()) != 0 { panic("outbox did not drain") }
    answer, err = service.Send(held.Send{Value:-1})
    if err != nil { panic(err) }
    if _, ok := answer.(held.SendOutcomeRefused); !ok { panic("local refusal changed") }
    if len(service.DrainOutbox()) != 0 { panic("refusal published") }
    service = renewalservice.New(refused{})
    answer, err = service.Send(held.Send{Value:value})
    if answer != nil || err == nil || err.Capability != "test refusal" || err.Source != "exact sentinel" { panic("unmet delegation changed") }
    if len(service.DrainOutbox()) != 0 { panic("unmet published") }
}
"#;

#[test]
fn go_handler_package_local_matrix_compiles_runs_and_is_deterministic() {
    let root = fresh_root("port-matrix");
    // These are real legal package names. Repeated '_' suffixing is a separate synthetic
    // allocator unit test: package_ident strips authored underscores before package allocation.
    for domain in ["c", "input", "outcome", "unmet", "value", "lease"] {
        let ir = compile_model(&PORT_MODEL.replace("DOMAIN", domain));
        let generated = synthesize_laid_out(&ir, Target::Go, OutputLayout::Workspace)
            .expect("port model generates");
        assert_repeatable(&ir, Target::Go, OutputLayout::Workspace, &generated);
        let code = &generated.artifacts["components/renewalservice/renewalservice.go"].contents;
        let [receiver, parameter, outcome, unmet, value] =
            ["c", "input", "outcome", "unmet", "value"].map(|base| {
                if base == domain {
                    format!("{base}_")
                } else {
                    base.to_owned()
                }
            });
        assert!(
            code.contains(&format!(
                "func ({receiver} *RenewalService) Send({parameter} {domain}.Send)"
            )),
            "{code}"
        );
        assert!(
            code.contains(&format!(
                "{outcome}, {unmet} := {receiver}.behaviors.Send({parameter})"
            )),
            "{code}"
        );
        assert!(code.contains(&format!("return nil, {unmet}")), "{code}");
        assert!(
            code.contains(&format!("switch {value} := {outcome}.(type)")),
            "{code}"
        );
        assert!(code.contains(&format!("{receiver}.outbox = append({receiver}.outbox, PublishedEventSent{{Event: {value}.Sent}})")), "{code}");
        assert!(code.contains(&format!("return {outcome}, nil")), "{code}");
        let directory = root.join(domain).join("generated");
        write(&generated, &directory);
        let built = go_build(&directory, &root.join("go-cache"), &["./..."]);
        std::fs::write(directory.join("compile.stderr"), &built.stderr)
            .expect("retain full build output");
        assert!(
            built.status.success(),
            "{domain}: {}",
            String::from_utf8_lossy(&built.stderr)
        );
        let harness = root.join(domain).join("harness");
        std::fs::create_dir(&harness).expect("mkdir");
        std::fs::write(harness.join("go.mod"), "module portharness\n\ngo 1.21\nrequire example.invalid/renewal v0.0.0\nreplace example.invalid/renewal => ../generated\n").expect("manifest");
        std::fs::write(harness.join("main.go"), PORT_GO.replace("DOMAIN", domain))
            .expect("harness");
        let binary = harness.join("port-harness");
        let built = go_build(
            &harness,
            &root.join("go-cache"),
            &["-o", binary.to_str().expect("test path"), "."],
        );
        std::fs::write(harness.join("compile.stderr"), &built.stderr)
            .expect("retain harness build output");
        assert!(
            built.status.success(),
            "{domain}: {}",
            String::from_utf8_lossy(&built.stderr)
        );
        let run = Command::new(&binary).output().expect("port harness runs");
        std::fs::write(harness.join("runtime.stderr"), &run.stderr).expect("retain runtime output");
        assert!(
            run.status.success(),
            "{domain}: {}",
            String::from_utf8_lossy(&run.stderr)
        );
    }
}

// Exact ordinary component bytes retained from baseline-run-2 before the port correction.
const ORIGINAL_COMPONENT: &str = r#"// generated from renewal v1
// model digest bd9369ee5b13977ab7a12e04821d6e452385a8217163e8ecdbb16c69d6aed911
// contract digest 6ed5b0073d7232d7638b4c413988e29e5f514366f85af43ec43928ab20545a4a
// do not edit: regenerate with `ess synthesize`

// Package renewalservice is renewal-service — the `renewal-service` component of `renewal` v1.
//
// The component's outer surface exactly as the specification declares it: accepted commands as
// methods, declared views as queries, published events as a typed outbox. The behaviour behind
// every handler is an implementation obligation — see the PLAN.md beside this module — and
// until one is satisfied, its stub answers with a typed refusal naming what is owed.
package renewalservice

import (
	"example.invalid/renewal/types/lease"
	"example.invalid/renewal/types/obligation"
)

// Behaviors bundles every behaviour and query this component owes.
//
// Constructing the port over each bounded context's `Unimplemented` yields a component
// that compiles and refuses, in the type system, everything not yet implemented.
type Behaviors interface {
	lease.RenewBehavior
}

// PublishedEvent is an event this component declares it publishes, on its way to the system's
// transport.
//
// A closed set: the marker method below is unexported, so no type outside this package can
// join it. Go cannot check that a `switch` over it handles every case — that is a target-stage
// weakening of what the specification declares, recorded in TARGET.md, not a gap in the model.
type PublishedEvent interface {
	isPublishedEvent()
}

// PublishedEventRenewed is `renewal.lease.Renewed`.
type PublishedEventRenewed struct {
	// Event is what was published.
	Event lease.Renewed
}

func (PublishedEventRenewed) isPublishedEvent() {}

// RenewalService is renewal-service — the port over the component's obligations.
//
// The behaviours and the outbox are unexported: commands enter through the methods below,
// and the system's transport is the only thing that drains what they published.
type RenewalService struct {
	// behaviors is everything this component owes.
	behaviors Behaviors
	// outbox holds what has been published since the last drain.
	outbox []PublishedEvent
}

// New builds a port over the given obligation implementations.
func New(behaviors Behaviors) *RenewalService {
	return &RenewalService{behaviors: behaviors}
}

// DrainOutbox hands over everything published since the last drain, in publication order.
//
// The system's transport calls this; anything else reading it is taking events the
// transport will then never deliver.
func (c *RenewalService) DrainOutbox() []PublishedEvent {
	drained := c.outbox
	c.outbox = nil
	return drained
}

// Renew accepts `renewal.lease.Renew`: runs the behaviour obligation, then publishes the declared events
// the outcome carries.
//
// The second result is the typed refusal of an unmet obligation — never a domain outcome,
// which always arrives as a variant of the outcome interface, refusals included.
func (c *RenewalService) Renew(input lease.Renew) (lease.RenewOutcome, *obligation.UnmetObligation) {
	outcome, unmet := c.behaviors.Renew(input)
	if unmet != nil {
		return nil, unmet
	}
	switch value := outcome.(type) {
	case lease.RenewOutcomeInvalid:
	case lease.RenewOutcomeStale:
	case lease.RenewOutcomeMissingProof:
	case lease.RenewOutcomeMismatchedProof:
	case lease.RenewOutcomeRenewed:
		c.outbox = append(c.outbox, PublishedEventRenewed{Event: value.Renewed})
	case lease.RenewOutcomeUnknown:
	}
	return outcome, nil
}
"#;

#[test]
fn ordinary_go_component_bytes_are_unchanged() {
    let generated = synthesize_laid_out(&model(), Target::Go, OutputLayout::Workspace)
        .expect("ordinary generation");
    assert_eq!(
        generated.artifacts["components/renewalservice/renewalservice.go"].contents,
        ORIGINAL_COMPONENT
    );
}
